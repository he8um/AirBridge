# Safe Backup Command Contract

## Overview

The safe backup command contract provides a validated, confirmation-gated Tauri command boundary for future user-triggered backup execution. The contract enforces explicit confirmation, output path validation, and token safety before any file is written.

This is a **backend-only, test-only-wired** implementation in V0.1. There is no UI file picker and no production "Run Backup" button. The command is registered with Tauri but is not called from the UI yet.

## Module Map

| Module | Purpose |
|---|---|
| `commands/backup_job.rs` | `run_backup_job` and `validate_backup_output_path` Tauri commands |
| `backup/output_path.rs` | `validate_output_path()` — pure validation, no file side effects |

## Confirmation Contract

The command enforces an explicit confirmation phrase before any write:

```
confirmation == "CREATE BACKUP"
```

- The caller must supply this exact string in the `confirmation` field of `RunBackupCommandRequest`.
- Case-sensitive. Partial matches are rejected.
- If absent or incorrect, the command returns a `CONFIRMATION_REQUIRED` safety error without touching any file.

This design makes it clear that the command writes a file to the provided output path and cannot be triggered accidentally.

## Output Path Validation Rules

`validate_output_path(path)` enforces these rules in order:

| Rule | Error Code | Notes |
|---|---|---|
| Path must not be empty | `EMPTY_PATH` | |
| Path must not contain null bytes | `NULL_BYTE` | |
| Path must not contain `..` components | `TRAVERSAL_DETECTED` | Checked via `Path::components()` |
| Extension must be `.airbridge` | `WRONG_EXTENSION` | |
| Path must not be an existing directory | `IS_DIRECTORY` | |
| Parent directory must exist | `PARENT_NOT_FOUND` | Only checked if a parent component exists |

Validation does not create, open, or write any file. It is safe to call from the UI at any time.

## Command: `validate_backup_output_path`

Read-only command. Safe to call before showing a save dialog or confirming a path.

**Input:** `path: String`

**Output:** `OutputPathValidationResult { valid, errorCode?, errorMessage? }`

**Side effects:** None — no file is created or modified.

## Command: `run_backup_job`

Writes a `.airbridge` package to the validated output path.

**Input:** `RunBackupCommandRequest`

| Field | Type | Notes |
|---|---|---|
| `token` | `String` | Consumed to build client; never stored |
| `outputPath` | `String` | Must pass all validation rules |
| `confirmation` | `String` | Must equal `"CREATE BACKUP"` |
| `baseId` | `String` | Airtable base ID |
| `baseName` | `String` | Human-readable name |
| `baseJson` | `Vec<u8>` | Pre-serialised base metadata (no token) |
| `schemaJson` | `Vec<u8>` | Pre-serialised schema JSON |
| `tableSpecs` | `Vec<RunBackupTableSpec>` | Tables to export |
| `pageSize` | `u32` | Pagination size (default 100) |
| `jobId` | `Option<String>` | Caller-supplied or auto-generated |

**Output:** `RunBackupCommandResponse`

| Field | Notes |
|---|---|
| `success` | `true` only if the job completed with `Succeeded` status |
| `packageFilename` | Filename-only (no directory, no absolute path) |
| `safetyErrors` | Present on confirmation or path rejection |
| `jobResult` | Embedded `BackupJobResult` on job-level success or failure |
| `pathValidation` | Always present |

No token appears in the response. No absolute output path appears in the response.

## Token Safety

1. `token` is received as a plain string in the request.
2. It is moved into `AirtableToken::new(token)` — the original string is consumed.
3. `AirtableToken` is moved into `AirtableClient::new(token, transport)`.
4. The client is moved into `BackupJobOrchestrator::new(client, cancellation)`.
5. The orchestrator and client are dropped at the end of `run_backup_job`.
6. No token string is included in `RunBackupCommandResponse`, `BackupJobResult`, or any event.

## Pipeline

`run_backup_job` delegates to `BackupJobOrchestrator::run()`:

```
1. Confirmation check         → CONFIRMATION_REQUIRED if wrong
2. Output path validation      → INVALID_OUTPUT_PATH if invalid
3. Build AirtableClient        → token consumed here
4. Run BackupJobOrchestrator   → full pipeline (planning → export → write → validate)
5. Return sanitized response   → filename-only, no absolute path
```

See `docs/architecture/backup-job-orchestration.md` for the orchestrator pipeline detail.

## TypeScript Mirror

`apps/desktop/src/backend/types.ts` exports matching types:

```typescript
RunBackupTableSpec
RunBackupCommandRequest   // token forwarded to Rust only; never stored
OutputPathValidationResult
BackupCommandSafetyError
RunBackupCommandResponse  // no token; filename-only path
```

`apps/desktop/src/backend/commands.ts` exports:

```typescript
validateBackupOutputPath(path: string): Promise<OutputPathValidationResult | null>
runBackupJob(request: RunBackupCommandRequest): Promise<RunBackupCommandResponse | null>
```

Both return `null` if Tauri IPC is unavailable (jsdom / browser without Tauri).

## Service Layer

`AirBridgeService` interface adds:

```typescript
validateBackupOutputPath(path: string): Promise<OutputPathValidationResult>
runBackupJob(request: RunBackupCommandRequest): Promise<RunBackupCommandResponse>
```

Mock service:
- `validateBackupOutputPath`: validates extension and empty-path only; deterministic; no file write.
- `runBackupJob`: validates confirmation and path; returns a safe mock response; no file write.

Live service: calls Tauri command bridge; no token persistence.

## Tests

`backup/output_path.rs` (unit tests):
- Empty, wrong extension, no extension, correct extension in tempdir
- Existing directory rejected
- Missing parent directory rejected
- Traversal (`..`) rejected
- Null byte rejected
- All error codes verified
- Validation creates no files

`commands/backup_job.rs` (unit tests):
- Path validation command: empty, wrong extension, missing parent, traversal, no side effects
- Confirmation: missing, wrong phrase, case-sensitive
- Path validation inside `run_backup_job`: extension, parent, traversal
- Orchestrator via mock transport: package written to tempdir, result has no token, no absolute path
- Response `packageFilename` is filename-only
- Generated package validates
- No attachment URLs in result
- Auth/permission errors map to sanitized failure
- `success: false` on failed job

Frontend tests (`safeBackupCommandContract.test.tsx`):
- Command bridge exports `validateBackupOutputPath` and `runBackupJob`
- `OutputPathValidationResult` and `RunBackupCommandResponse` type shapes
- Mock service: deterministic path validation, confirmation rejection, no file write
- UI: section present, live execution disabled, confirmation required, path validation mentioned, no enabled trigger button

## UI Integration

`runBackupJob` is wired to the production UI in `BackupExecutionPanel`. The panel:
- Opens a native save dialog via `pickBackupOutputPath()` (Tauri dialog plugin).
- Validates the output path before enabling the run button.
- Requires the user to type the confirmation text into a dedicated input field.
- Accepts a one-time token in a local `type="password"` field.
- Clears the token and confirmation after each run.

See `docs/architecture/backup-file-picker-confirmation-flow.md` for the full UI flow design.

## Safety Constraints

- No live network calls in any test.
- No token stored in orchestrator, command, events, or response.
- No absolute filesystem paths in response or events.
- No attachment URLs in response or events.
- `output_path` is used locally only — never serialised in the response.
- Only filename is returned in `packageFilename`.
- Packages written only to temp directories in tests.
- Generated `.airbridge` files are never committed to the repository.
- No token persistence anywhere in the UI flow.

## Progress and Cancellation

`BackupJobResult.events` carries an ordered event timeline after job completion.
A `cancel_backup_job` command is registered as a V0.1 placeholder (always returns `not_running`).

See `docs/architecture/backup-progress-and-cancellation.md` for the full model.

## Future Path

- Stream `BackupJobEvent` to the frontend via Tauri events for live progress.
- Add retry logic for `RateLimited` errors inside the orchestrator.
- Wire background job registry to enable real-time `cancel_backup_job` behaviour.
- Add secure credential storage using the OS keychain for repeat runs.
