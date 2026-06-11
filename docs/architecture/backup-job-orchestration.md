# Backup Job Orchestration

## Overview

The backup job orchestration layer coordinates the full backup pipeline for a single run: planning, paginated record export, package writing, validation, progress event emission, and safe result modeling. It sits between the individual engine modules and any future Tauri command boundary.

This is a **backend-only, test-only-wired** implementation in V0.1. There is no UI button to trigger a live backup run. All tests use mocked HTTP transport — no live network calls are made.

## Module Map

| Module | Purpose |
|---|---|
| `backup/cancellation.rs` | `CancellationToken` — `Arc<AtomicBool>` polled at phase boundaries |
| `backup/job.rs` | Job lifecycle types: `BackupJobId`, `BackupJobStatus`, `BackupJobPhase`, request/result structs |
| `backup/job_events.rs` | `BackupJobEvent` tagged enum (11 variants), `#[serde(tag = "kind")]` |
| `backup/job_result.rs` | Builder functions for succeeded/failed/cancelled results; `validation_summary_from_report()` |
| `backup/job_orchestrator.rs` | `BackupJobOrchestrator<T>` — runs the pipeline, collects events, returns `BackupJobResult` |

## Lifecycle

```
Queued → Running → Succeeded
                 → Failed
                 → Cancelled
```

Status is encoded in `BackupJobResult.status` — there is no runtime state machine object. The orchestrator runs synchronously and returns the final result.

## Pipeline Phases

`BackupJobOrchestrator::run()` steps through these phases in order:

```
Planning
  └─ emit JobStarted, PhaseStarted(Planning)
  └─ check cancellation → JobCancelled if set

RecordsExport
  └─ emit PhaseStarted(RecordsExport)
  └─ emit TableExportStarted per table
  └─ check cancellation → JobCancelled if set
  └─ call run_export() → on error → emit JobFailed, return failed result
  └─ emit TableExportCompleted per table

PackageBuild
  └─ emit PhaseStarted(PackageBuild)
  └─ check cancellation → JobCancelled if set
  └─ emit PackageWriteStarted
  └─ call write_package()
  └─ emit PackageWriteCompleted

Validation
  └─ emit PhaseStarted(Validation), ValidationStarted
  └─ call validate_package()
  └─ emit ValidationCompleted
  └─ on invalid → emit JobFailed, return failed result

Completed
  └─ emit PhaseStarted(Completed), JobSucceeded
  └─ return succeeded result
```

Cancellation is checked after Planning and at the start of PackageBuild. Cancellation during the active `run_export()` call is handled by a transport-level mechanism in tests (the export engine does not yet poll cancellation internally).

## Cancellation

`CancellationToken` wraps `Arc<AtomicBool>`. Cancelling sets the flag; the orchestrator polls `is_cancelled()` at each phase boundary. Cloning the token shares the same atomic — any holder can cancel.

```rust
let token = CancellationToken::new();
let token2 = token.clone();
token2.cancel();
assert!(token.is_cancelled()); // true
```

## Event Model

`BackupJobEvent` is a tagged enum serialised with `#[serde(tag = "kind", rename_all = "camelCase")]`:

| Variant | When emitted |
|---|---|
| `JobStarted` | Before planning phase |
| `PhaseStarted` | At the start of each phase |
| `TableExportStarted` | Before export loop begins per table |
| `TableExportCompleted` | After each table's records are fetched |
| `PackageWriteStarted` | Before `write_package()` |
| `PackageWriteCompleted` | After `write_package()` returns |
| `ValidationStarted` | Before `validate_package()` |
| `ValidationCompleted` | After `validate_package()` returns |
| `JobSucceeded` | On successful completion |
| `JobFailed` | On any terminal error |
| `JobCancelled` | When cancellation is detected at a phase boundary |

All events carry `job_id`. No event contains a token, absolute path, or attachment URL.

## Result Model

`BackupJobResult` is safe to serialise and return across the Tauri command boundary:

- No token field.
- No absolute filesystem paths.
- No attachment URLs.
- `packageSummary` is `Option<BackupJobPackageSummary>` — absent on failure/cancellation.
- `validationSummary` is `Option<BackupJobValidationSummary>` — absent on failure/cancellation.

`BackupJobPackageSummary` is populated from `BackupPackageReader` after writing — entry count and checksum count are read back from the archive without re-exposing the output path.

## Error Mapping

`ExportEngineError` variants map to job error codes:

| Engine Error | Job Error Code | Recoverable |
|---|---|---|
| `InvalidToken` | `AUTH_FAILED` | false |
| `PermissionDenied` | `PERMISSION_DENIED` | false |
| `MissingScope` | `MISSING_SCOPE` | false |
| `RateLimited` | `RATE_LIMITED` | true |
| `NotFound` | `NOT_FOUND` | false |
| `MalformedResponse(_)` | `MALFORMED_RESPONSE` | false |
| `TransientServerError(_)` | `TRANSIENT_SERVER_ERROR` | true |
| `PageLimitReached(_)` | `PAGE_LIMIT_REACHED` | false |

## TypeScript Mirror

`apps/desktop/src/backend/types.ts` exports matching types:

```typescript
BackupJobStatus       // "queued" | "running" | "succeeded" | "failed" | "cancelled"
BackupJobPhase        // "planning" | "schema" | "recordsExport" | "packageBuild" | ...
BackupJobWarning      // { code, message, tableId? }
BackupJobError        // { code, message, recoverable }
BackupJobTableResult  // { tableId, tableName, recordCount, pagesFetched }
BackupJobPackageSummary
BackupJobValidationSummary
BackupJobResult
```

## Tests

`BackupJobOrchestrator` tests live in `backup/job_orchestrator.rs` (unit tests via `#[cfg(test)]`). Coverage includes:

- Successful single-table orchestration (all events in order, correct result fields)
- Two-page pagination (export fetches two pages, result has correct counts)
- Two-table orchestration (events emitted per table, table results collected)
- Event order verification via `kind_str()` helper
- Package summary entry and checksum counts
- No token sentinel in any event or result
- No absolute filesystem paths in any event or result
- No attachment URLs in any event or result
- 401 → `AUTH_FAILED`, 403 → `PERMISSION_DENIED`, 429 → `RATE_LIMITED`
- Cancellation before export → `JobCancelled` at `Planning` phase
- Cancellation emits `jobCancelled` event
- Cancellation after export starts (transport-level cancel mid-run)

`apps/desktop/src/test/backupJobOrchestration.test.tsx` covers TypeScript type shape and UI:

- All `BackupJobStatus` values accepted
- `packageSummary` absent on cancelled result
- `encrypted: false` for V0.1
- UI section present on Backups page
- No enabled production backup-trigger button

## Safety Constraints

- No live network calls in any test.
- No token stored in orchestrator, events, or results.
- No absolute filesystem paths in results or events.
- No attachment URLs in results or events.
- Output path (`&Path`) used only locally in `run()` — never serialised.
- Packages written only to temp directories in tests.
- Generated `.airbridge` files are never committed to the repository.
- No UI production export flow in V0.1.

## Tauri Command Integration

`BackupJobOrchestrator` is called by `run_backup_job` in `commands/backup_job.rs`. The command wraps it with an explicit confirmation check and output path validation. See `docs/architecture/safe-backup-command-contract.md` for the full command contract design.

## Future Path

- Stream `BackupJobEvent` to the frontend via Tauri events.
- Add retry logic for `RateLimited` inside the orchestrator.
- Propagate cancellation into `run_export()` at the page loop level.
- Add file picker and user-selected output path in a later phase.
