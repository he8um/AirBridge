# Editor Setup

## VS Code

### Rust Analyzer

The Rust crate lives at `apps/desktop/src-tauri/Cargo.toml`. When VS Code is opened at the repository root, rust-analyzer needs to be told where to find it.

The repository includes `.vscode/settings.json` with:

```json
{
  "rust-analyzer.linkedProjects": ["apps/desktop/src-tauri/Cargo.toml"],
  "rust-analyzer.check.command": "check",
  "rust-analyzer.check.extraArgs": ["--all-targets"],
  "rust-analyzer.cargo.targetDir": "apps/desktop/src-tauri/target/rust-analyzer"
}
```

`linkedProjects` tells rust-analyzer to load the Tauri crate directly, regardless of where VS Code's workspace root is.

`cargo.targetDir` writes rust-analyzer's intermediate build artefacts to a separate subdirectory inside the existing `target` tree, so they do not interfere with normal `cargo build` or `cargo test` outputs.

### Recommended extensions

`.vscode/extensions.json` lists:

- `rust-lang.rust-analyzer` — Rust language support
- `tauri-apps.tauri-vscode` — Tauri command palette helpers

VS Code will prompt to install these when the workspace is first opened.

### Troubleshooting

If rust-analyzer still shows a `FetchWorkspaceError` after opening the repository:

1. Open the Command Palette (`⇧⌘P` on macOS, `Ctrl+Shift+P` on Windows/Linux).
2. Run **Rust Analyzer: Restart Server**.
3. If the error persists, run **Developer: Reload Window**.
4. Verify the crate is discoverable from the terminal:

   ```sh
   cd apps/desktop/src-tauri
   cargo metadata --format-version 1 --no-deps
   ```

   This command should print a JSON object with `workspace_root` pointing to `apps/desktop/src-tauri`. If it fails, check that Rust and Cargo are installed and available in your shell PATH.

5. On macOS, GUI-launched VS Code may not inherit your shell PATH. If `cargo` is not found, the `rust-analyzer.server.extraEnv.PATH` setting in `.vscode/settings.json` adds the standard Cargo and Homebrew bin directories so rust-analyzer can locate the toolchain.
