# TLC for Zed

This extension opens `.tlc` files in Zed and runs them with the TLC compiler.

Highlighting comes from the grammar in `tlc-tree-sitter`. Completions, diagnostics, hover, go to definition, and symbols come from `tlc lsp` in the compiler. This extension only starts that server.

The server binary is `build/tlc` in the project root when that file exists and is executable. Otherwise it is `tlc` on `PATH`.

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
