use zed_extension_api::{self as zed, Command, LanguageServerId, Result, Worktree};

const LOCATOR: &str = "tlc";
const BUILD_TASK: &str = "Build TLC debug binary";
const NOT_FOUND: &str = "tlc was not found. Set lsp.tlc.binary.path in Zed settings, or build it with: cmake --build build --target tlc";

struct TlcExtension;

fn expand_tilde(path: &str, home: Option<&str>) -> String {
    let Some(home) = home.filter(|home| !home.is_empty()) else {
        return path.to_string();
    };
    let home = home.trim_end_matches('/');
    if path == "~" {
        home.to_string()
    } else if let Some(rest) = path.strip_prefix("~/") {
        format!("{home}/{rest}")
    } else {
        path.to_string()
    }
}

/// `Some(true)` when `path` is an executable file. `None` when the check itself cannot run.
fn executable_file(path: &str) -> Option<bool> {
    let output = zed::process::Command::new("/bin/sh")
        .arg("-c")
        .arg("[ -f \"$1\" ] && [ -x \"$1\" ]")
        .arg("sh")
        .arg(path)
        .output()
        .ok()?;
    Some(output.status == Some(0))
}

fn resolve_configured_path(
    path: &str,
    root: &str,
    home: Option<&str>,
    worktree: &Worktree,
) -> Result<String> {
    let path = expand_tilde(path.trim(), home);
    if path.is_empty() {
        return Err("lsp.tlc.binary.path is empty".to_string());
    }
    if !path.contains('/') {
        return worktree
            .which(&path)
            .ok_or_else(|| format!("tlc from lsp.tlc.binary.path was not found on PATH: {path}"));
    }
    let full = if path.starts_with('/') || root.is_empty() {
        path
    } else {
        let root = root.trim_end_matches('/');
        if root.is_empty() {
            format!("/{path}")
        } else {
            format!("{root}/{path}")
        }
    };
    if full.starts_with('/') && executable_file(&full) == Some(false) {
        return Err(format!(
            "tlc from lsp.tlc.binary.path is not an executable file: {full}"
        ));
    }
    Ok(full)
}

fn task_env<'a>(env: &'a [(String, String)], key: &str) -> Option<&'a str> {
    env.iter()
        .find(|(name, _)| name == key)
        .map(|(_, value)| value.as_str())
        .filter(|value| !value.is_empty())
}

/// Same path as `build-debug` in `languages/tlc/tlc-task.sh` when no output argument is passed.
fn debug_output(root: &str, relative_dir: &str, stem: &str) -> String {
    format!("{root}/build/debug/{relative_dir}/{stem}")
}

/// The `-c` argument in tasks.json is wrapped in single quotes.
/// Those quotes are shell syntax for task spawn. They are not part of the script.
fn unquote_debug_script(quoted: &str) -> Option<String> {
    let script = quoted.strip_prefix('\'')?.strip_suffix('\'')?;
    if script.contains('\'') {
        return None;
    }
    Some(script.to_string())
}

fn build_tlc(root: &str) -> String {
    let root = root.trim_end_matches('/');
    if root.is_empty() {
        "/build/tlc".to_string()
    } else {
        format!("{root}/build/tlc")
    }
}

fn discover_tlc(worktree: &Worktree) -> Result<String> {
    let root = worktree.root_path();
    if !root.is_empty() {
        let candidate = build_tlc(&root);
        if executable_file(&candidate) == Some(true) {
            return Ok(candidate);
        }
    }
    worktree.which("tlc").ok_or_else(|| NOT_FOUND.to_string())
}

impl TlcExtension {
    fn debug_label(adapter: &str) -> String {
        if adapter == "GDB" {
            "Debug TLC program (GDB)".to_string()
        } else {
            "Debug TLC program".to_string()
        }
    }
}

impl zed::Extension for TlcExtension {
    fn new() -> Self {
        Self
    }

    fn language_server_command(
        &mut self,
        language_server_id: &LanguageServerId,
        worktree: &Worktree,
    ) -> Result<Command> {
        let settings =
            zed::settings::LspSettings::for_worktree(language_server_id.as_ref(), worktree).ok();
        let binary = settings
            .as_ref()
            .and_then(|settings| settings.binary.as_ref());
        let mut env = worktree.shell_env();
        if let Some(extra) = binary.and_then(|binary| binary.env.as_ref()) {
            for (key, value) in extra {
                if let Some((_, slot)) = env.iter_mut().find(|(name, _)| name == key) {
                    slot.clone_from(value);
                } else {
                    env.push((key.clone(), value.clone()));
                }
            }
        }
        // Zed launches lsp.tlc.binary.path itself and skips this function.
        // That launch uses binary.arguments, so the settings list has to include "lsp".
        // A path with no arguments starts tlc alone.
        if let Some(path) = binary.and_then(|binary| binary.path.as_deref()) {
            if !path.trim().is_empty() {
                let home = env
                    .iter()
                    .find(|(key, _)| key == "HOME")
                    .map(|(_, value)| value.as_str());
                let command = resolve_configured_path(path, &worktree.root_path(), home, worktree)?;
                let args = binary
                    .and_then(|binary| binary.arguments.clone())
                    .unwrap_or_else(|| vec!["lsp".to_string()]);
                return Ok(Command { command, args, env });
            }
        }

        // binary.path is unset. Prefer an executable <root>/build/tlc, then tlc on PATH.
        Ok(Command {
            command: discover_tlc(worktree)?,
            args: vec!["lsp".into()],
            env,
        })
    }

    fn dap_locator_create_scenario(
        &mut self,
        locator_name: String,
        build_task: zed::TaskTemplate,
        _resolved_label: String,
        debug_adapter_name: String,
    ) -> Option<zed::DebugScenario> {
        if locator_name != LOCATOR || build_task.label != BUILD_TASK {
            return None;
        }
        if debug_adapter_name != "CodeLLDB" && debug_adapter_name != "GDB" {
            return None;
        }
        let config = zed::serde_json::json!({
            "request": "launch",
            "program": "$ZED_WORKTREE_ROOT/build/debug/$ZED_RELATIVE_DIR/$ZED_STEM",
            "cwd": "$ZED_DIRNAME",
        });
        // `debugger: start` looks the build task up again with no buffer, so a
        // language task named in `ByName` is invisible. Carry the template.
        // The extension API drops the task shell, and Zed then quotes each
        // argument for the login shell. Remove the quotes that task spawn needs
        // when it joins arguments with spaces. The wrapper has no quotes of its own.
        let mut template = build_task;
        if template.args.first().is_some_and(|arg| arg == "-c") {
            if let Some(script) = template
                .args
                .get(1)
                .and_then(|arg| unquote_debug_script(arg))
            {
                template.args[1] = script;
            }
        }
        Some(zed::DebugScenario {
            label: Self::debug_label(&debug_adapter_name),
            adapter: debug_adapter_name,
            build: Some(zed::BuildTaskDefinition::Template(
                zed::BuildTaskDefinitionTemplatePayload {
                    locator_name: None,
                    template,
                },
            )),
            config: config.to_string(),
            tcp_connection: None,
        })
    }

    fn run_dap_locator(
        &mut self,
        locator_name: String,
        build_task: zed::TaskTemplate,
    ) -> Result<zed::DebugRequest, String> {
        if locator_name != LOCATOR {
            return Err("unknown locator".to_string());
        }
        // The build task runs `tlc-task.sh build-debug` and writes
        // `<root>/build/debug/<relative dir>/<stem>`. Paths stay in the task
        // environment because Zed joins task args without quoting them.
        let root = task_env(&build_task.env, "ZED_WORKTREE_ROOT")
            .ok_or_else(|| "debug build did not set ZED_WORKTREE_ROOT".to_string())?;
        let stem = task_env(&build_task.env, "ZED_STEM")
            .ok_or_else(|| "debug build did not set ZED_STEM".to_string())?;
        let relative_dir = task_env(&build_task.env, "ZED_RELATIVE_DIR").unwrap_or("");
        let program = debug_output(root, relative_dir, stem);
        let cwd = task_env(&build_task.env, "ZED_DIRNAME")
            .map(str::to_string)
            .or_else(|| {
                task_env(&build_task.env, "ZED_FILE").and_then(|file| {
                    std::path::Path::new(file)
                        .parent()
                        .map(|dir| dir.to_string_lossy().into_owned())
                        .filter(|dir| !dir.is_empty())
                })
            });
        Ok(zed::DebugRequest::Launch(zed::LaunchRequest {
            program,
            cwd,
            args: Vec::new(),
            envs: build_task.env,
        }))
    }
}

zed::register_extension!(TlcExtension);

#[cfg(test)]
mod tests {
    use super::{debug_output, unquote_debug_script};
    use zed_extension_api::serde_json;

    #[test]
    fn debug_output_matches_the_task_script() {
        assert_eq!(
            debug_output("/proj", "src", "main"),
            "/proj/build/debug/src/main"
        );
        assert_eq!(
            debug_output("/proj", "", "main"),
            "/proj/build/debug//main"
        );
    }

    #[test]
    fn debug_build_script_unquotes_to_a_fish_safe_wrapper() {
        let tasks: Vec<serde_json::Value> =
            serde_json::from_str(include_str!("../languages/tlc/tasks.json")).unwrap();
        let build = tasks
            .iter()
            .find(|task| task["label"] == "Build TLC debug binary")
            .unwrap();
        let quoted = build["args"][1].as_str().unwrap();
        let script = unquote_debug_script(quoted).unwrap();
        assert!(!script.contains(['\'', '\\']));
        assert!(script.starts_with("script=$(base64 -d <<END\n"));
        assert!(script.contains("exec /bin/sh -c \"$script\" sh \"$@\"\n"));
    }
}
