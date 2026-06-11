# Backup File Picker and Confirmation Flow

## Overview

The backup file picker and confirmation flow is the first production-facing backup execution path. It allows a user to select a `.airbridge` output file, supply a personal access token for that single run, confirm the action, and execute a backup through the existing safe command contract.

This is a conservative V1 implementation:
- No token persistence.
- No automatic retries.
- No restore operations.
- No scheduling or background jobs.
- No file is written unless the user has selected a valid output path and supplied the required confirmation text.

## Module Map

| Module | Purpose |
|---|---|
| `features/backups/backupExecutionHelpers.ts` | Pure helper functions: filename extraction, extension check, redaction, confirmation text |
| `features/backups/BackupOutputPicker.ts` | `pickBackupOutputPath()` — opens the native OS save dialog via the Tauri dialog plugin |
| `features/backups/BackupConfirmationBox.tsx` | Confirmation text input component; shows mismatch feedback |
| `features/backups/BackupJobResultCard.tsx` | Renders job result, package summary, validation summary, warnings/errors |
| `features/backups/BackupExecutionPanel.tsx` | Orchestrates the full execution flow: file pick → validate → token → confirm → run → result |
| `features/backups/index.ts` | Re-exports all public feature symbols |

## Execution Flow

```
User clicks "Choose File…"
  └─ pickBackupOutputPath() → native OS save dialog
  └─ user selects or cancels
  └─ if cancelled: no state change
  └─ if selected:
       └─ display filename only (not full path)
       └─ check extension (.airbridge required)
       └─ call service.validateBackupOutputPath(path)
       └─ show valid/invalid status

User enters token (password field, local only)
User types confirmation text ("CREATE BACKUP")

Run Backup button becomes enabled only when:
  └─ backup plan exists
  └─ records export plan exists
  └─ output path is valid
  └─ token is non-empty
  └─ confirmation text equals "CREATE BACKUP"

User clicks "Run Backup"
  └─ call service.runBackupJob({
         token,         ← consumed, not stored
         outputPath,    ← validated path
         confirmation,  ← "CREATE BACKUP"
         baseId, baseName, tableSpecs, …
       })
  └─ clearSensitiveState() — token and confirmation cleared
  └─ show result card
```

## Native Save Dialog

`pickBackupOutputPath()` uses `@tauri-apps/plugin-dialog` (`save()` function). The dialog:
- Suggests a default filename based on the base name (e.g. `MyBase.airbridge`).
- Filters to `*.airbridge` files.
- Returns the selected path string, or `null` on cancel.
- Does not write any file.

In jsdom test environments, the Tauri plugin import fails gracefully and `pickBackupOutputPath` returns `null`. Tests mock the function with `vi.mock`.

The Rust side registers `tauri_plugin_dialog::init()` in `lib.rs`. The `dialog:default` permission is added to `capabilities/default.json`.

## Path Display and Redaction

The UI never renders the absolute output path. Only the filename component is shown.

| Function | Behaviour |
|---|---|
| `getDisplayFileName(path)` | Extracts filename from forward- or backslash-separated paths |
| `redactOutputPath(path)` | Returns `…/filename.airbridge` — directory not shown |
| `hasAirbridgeExtension(path)` | Returns `true` if path ends with `.airbridge` |

## Confirmation Contract

The exact confirmation text `"CREATE BACKUP"` must be typed by the user into the confirmation field before the Run Backup button is enabled. This text is forwarded to `runBackupJob` as the `confirmation` field and checked by the Rust command before any file write.

The Rust contract (`commands/backup_job.rs`) rejects any request where `confirmation != "CREATE BACKUP"` with error code `CONFIRMATION_REQUIRED`.

## Token Safety

1. User types the token into a `type="password"` input inside `BackupExecutionPanel`.
2. The token is held only in component state (`useState`).
3. On run, the token is passed directly to `service.runBackupJob()` as part of the request.
4. After the run completes or fails, `clearSensitiveState()` sets the token state to `""`.
5. The token is not stored in `localStorage`, `sessionStorage`, or any persistent state.
6. The token does not appear in any response, result, log, or rendered element outside the password input.

## Response Safety

`RunBackupCommandResponse` (from the Rust command):
- No token field.
- No absolute filesystem path — only `packageFilename` (filename portion only).
- `BackupJobResultCard` renders only `packageFilename`, not the directory.

## Tauri Plugin Dependency

`tauri-plugin-dialog` is added:
- Rust: `tauri-plugin-dialog = "2"` in `Cargo.toml`
- Frontend: `@tauri-apps/plugin-dialog@^2` in `package.json`
- Capability: `"dialog:default"` in `capabilities/default.json`

## Tests

`src/test/backupFilePicker.test.tsx`:
- `backupExecutionHelpers` pure functions (macOS/Windows paths, redaction, extension check)
- Execution panel renders and shows all safety copy
- Run button disabled in all states: no plans, only one plan, plans but no path, plans+path but no token, plans+path+token but no confirmation
- File picker: filename-only display, no absolute path rendered, invalid extension → error, valid path → valid status
- Token field type is `password`, token value not rendered outside input
- Successful mock run → success result, token cleared, no absolute path in result
- Failed mock run → sanitized error code shown, no token in result
- `BackupJobResultCard` renders success/failure correctly without exposing token or absolute path

## Safety Constraints

- No live network calls in any test.
- File picker mocked in all tests with `vi.mock`.
- No token persistence anywhere in the flow.
- No absolute path rendered in UI or result.
- No file written in jsdom tests.
- No token in rendered output outside password input.
- Generated `.airbridge` packages are never committed to the repository.
- No restore, scheduling, or automatic execution.

## Cancel Button

`BackupExecutionPanel` shows a Cancel button while the job is running (`runState === "running"`):
- Calls `service.cancelBackupJob(activeJobId)`.
- Clears the token immediately before the service call.
- Transitions `runState` to `"done"`.

In V0.1 cancellation always returns `not_running` — the job runs synchronously and completes before the cancel call can reach it. See `docs/architecture/backup-progress-and-cancellation.md`.

## Event Timeline

`BackupJobResultCard` renders an event timeline when `jobResult.events` is non-empty.
Each event is displayed as a labelled list item with a `data-event-kind` attribute.
In V0.1 the orchestrator does not yet populate events — the timeline is hidden.

## Future Path

- Stream `BackupJobEvent` progress to the frontend via Tauri events.
- Add retry logic for recoverable errors (`RATE_LIMITED`).
- Add job history and per-job status tracking.
- Add secure credential storage using the OS keychain via a Tauri plugin.
- Wire real cancellation via background job registry and `CancellationToken`.
