use zed_extension_api::{self as zed, Command, LanguageServerId, Result, Worktree};

const LOCATOR: &str = "tlc";
const BUILD_TASK: &str = "Build TLC debug binary";

struct TlcExtension;

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
        _language_server_id: &LanguageServerId,
        worktree: &Worktree,
    ) -> Result<Command> {
        let root = worktree.root_path();
        // Wasm cannot see the project tree. The shell prefers <root>/build/tlc, then tlc on PATH.
        if root.is_empty() {
            let command = worktree.which("tlc").ok_or_else(|| {
                "tlc was not found. Build it with: cmake --build build --target tlc".to_string()
            })?;
            return Ok(Command {
                command,
                args: vec!["lsp".into()],
                env: worktree.shell_env(),
            });
        }
        let script = r#"if [ -n "$1" ] && [ -x "$1/build/tlc" ]; then exec "$1/build/tlc" lsp; fi
if command -v tlc >/dev/null 2>&1; then exec tlc lsp; fi
echo "tlc was not found. Build it with: cmake --build build --target tlc" >&2
exit 1"#;
        Ok(Command {
            command: "/bin/sh".into(),
            args: vec!["-c".into(), script.into(), "sh".into(), root],
            env: worktree.shell_env(),
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
        Some(zed::DebugScenario {
            label: Self::debug_label(&debug_adapter_name),
            adapter: debug_adapter_name,
            build: Some(zed::BuildTaskDefinition::ByName(BUILD_TASK.to_string())),
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
        // The build task is `sh -c <script> sh <root> <file> <output>`.
        let program = build_task
            .args
            .last()
            .cloned()
            .filter(|path| !path.is_empty())
            .ok_or_else(|| "debug build did not name an output".to_string())?;
        let cwd = build_task.args.iter().rev().nth(1).and_then(|file| {
            std::path::Path::new(file)
                .parent()
                .map(|dir| dir.to_string_lossy().into_owned())
                .filter(|dir| !dir.is_empty())
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
