# TLC for Zed

This extension opens `.tlc` files in Zed and runs them with the TLC compiler.

Highlighting comes from the grammar in `tlc-tree-sitter`. Completions, diagnostics, hover, go to definition, and symbols come from `tlc lsp` in the compiler. This extension only starts that server.

## Compiler path

The language server and the run, check, and debug tasks use the same `tlc` binary. The search order is:

1. `lsp.tlc.binary.path` in Zed settings. A path in the project's `.zed/settings.json` overrides the same setting in your user settings. A relative path that contains a slash starts at the project root. A path that is `~` or starts with `~/` starts at your home directory. A name with no slash is looked up on `PATH`.
2. `build/tlc` in the project root, when that file is executable.
3. `tlc` on `PATH`.

```json
{
  "lsp": {
    "tlc": {
      "binary": {
        "path": "/absolute/path/to/tlc",
        "arguments": ["lsp"]
      }
    }
  }
}
```

Zed starts `binary.path` itself and does not ask the extension for a command. `arguments` is that process's argument list, and it has to include `lsp`. A path with no arguments starts `tlc` alone. The compiler then prints its usage text and exits, and Zed reports that the server reset the connection.

When `binary.path` is unset, the extension starts `tlc lsp`. It uses `build/tlc` in the project root when that file is executable, and otherwise `tlc` on `PATH`. Run, check, and debug use that same search order, including `lsp.tlc.binary.path`, and ignore `binary.arguments`. `binary.env` is added to the language server environment.

When `binary.path` names a file that is not executable, the task reports that path and stops.

## Install

1. Open Extensions in Zed with Ctrl+Shift+X.
2. Click **Install Dev Extension**.
3. Choose this directory.
4. Open a `.tlc` file.

The same command is `zed: install dev extension` in the command palette. The `zed` program in a terminal opens files, so those words are not typed there.

Zed builds the small Rust crate in this directory when the extension is installed. That crate starts `tlc lsp`. The build target is `wasm32-wasip2`. Rustup installs that target on its own. Arch's `rust` package needs `rust-wasm` as well. Zed also checks out the grammar revision in `extension.toml` and builds it.

Hold Ctrl and click a function name to open its definition.

## Tasks

In a `.tlc` file:

- `task: spawn` and choose **Run TLC program**. That runs `tlc --run` on the file.
- **Check TLC program** typechecks the file and does not run it.
- **Build TLC debug binary** writes an executable with `-g` under `build/debug/`.
- The line `func main` has a run button. It uses the same run task.

## Debug

Open the `.tlc` file and run `debugger: start`. **Debug TLC program** builds that file with `-g` into `build/debug/` and launches it under CodeLLDB. **Debug TLC program (GDB)** uses GDB.

A click in the gutter sets a breakpoint. A file-scope `const` has no address and is not shown.
