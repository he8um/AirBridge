# Backup Job Progress and Cancellation

## Overview

This document describes the progress snapshot model and cancellation contract for backup jobs in V0.1.

The current architecture is **synchronous and snapshot-based**:
- `run_backup_job` runs to completion before returning.
- All events are collected internally by the orchestrator and returned in `BackupJobResult.events`.
- The frontend renders the event timeline from the completed response (no live streaming).
- The `cancel_backup_job` command is registered but always returns `not_running` — no background job registry exists yet.

Live progress streaming and background job management are future work (see [Future Path](#future-path)).

## Module Map

| Module | Purpose |
|---|---|
| `backup/job_progress.rs` | `BackupJobProgressSnapshot`, `BackupJobCancellationRequest`, `BackupJobCancellationResult` |
| `backup/job.rs` | `BackupJobResult.events: Vec<BackupJobEvent>` (added in V0.1.x) |
| `backup/job_events.rs` | `BackupJobEvent` tagged enum — 11 variants |
| `commands/backup_job.rs` | `cancel_backup_job` Tauri command (synchronous placeholder) |
| `backend/types.ts` | TypeScript mirrors: `BackupJobEvent`, `BackupJobProgressSnapshot`, `BackupJobCancellationResult` |
| `features/backups/BackupExecutionPanel.tsx` | Cancel button (visible while `runState === "running"`), token cleared on cancel |
| `features/backups/BackupJobResultCard.tsx` | Event timeline section (`data-testid="backup-event-timeline"`) |

## Progress Snapshot

`BackupJobProgressSnapshot` is a read-only struct that describes a job's progress at a point in time:

| Field | Type | Notes |
|---|---|---|
| `jobId` | `BackupJobId` | Job being described |
| `phase` | `BackupJobPhase` | Current pipeline phase |
| `status` | `BackupJobStatus` | Current lifecycle status |
| `completedTables` | `usize` | Tables fully exported so far |
| `totalTables` | `Option<usize>` | Total tables, if known |
| `unknownTotal` | `bool` | True when total is not yet determined |
| `currentTableId` | `Option<String>` | Table currently being exported |
| `currentTableName` | `Option<String>` | Human-readable current table name |
| `warningCount` | `usize` | Warnings accumulated so far |
| `errorCount` | `usize` | Errors accumulated so far |

Safe to serialise: no token, no absolute paths, no attachment URLs.

## Cancellation Contract

### `cancel_backup_job` Tauri command

```
Input:  job_id: String
Output: BackupJobCancellationResult { job_id, was_running, status_at_cancellation }
```

V0.1 behaviour: because `run_backup_job` is synchronous (no background thread), any `cancel_backup_job` call arrives after the job has already completed. The command always returns:

```json
{ "jobId": "...", "wasRunning": false, "statusAtCancellation": "not_running" }
```

### UI Cancel Button

`BackupExecutionPanel` shows a Cancel button while `runState === "running"`:

- The button calls `service.cancelBackupJob(activeJobId)`.
- `clearSensitiveState()` is called immediately on click — the token is cleared before the service call returns.
- After cancellation, `runState` transitions to `"done"`.
- `data-testid="cancel-backup-button"` — present only while running.

Because `run_backup_job` is synchronous in V0.1, the Cancel button is visible for a very short window (while the async service call is in flight). In a future async model it would be visible for the duration of the job.

## Event Timeline

`BackupJobResult.events` is an ordered `Vec<BackupJobEvent>` returned after job completion.

In V0.1 the orchestrator does not yet populate this field — all completed jobs return an empty `events` vec. The field is present on the struct and serialised as `"events":[]` (omitted when empty due to `skip_serializing_if`).

When the orchestrator is extended to collect events, `BackupJobResultCard` will automatically render the timeline — no frontend changes needed.

### Event Kinds

| Kind | Key fields |
|---|---|
| `jobStarted` | `baseId`, `baseName`, `tableCount` |
| `phaseStarted` | `phase` |
| `tableExportStarted` | `tableId`, `tableName` |
| `tableExportCompleted` | `tableId`, `tableName`, `recordCount`, `pagesFetched` |
| `packageWriteStarted` | — |
| `packageWriteCompleted` | `entryCount` |
| `validationStarted` | — |
| `validationCompleted` | `status`, `errorCount`, `warningCount` |
| `jobSucceeded` | `totalRecords`, `tableCount` |
| `jobFailed` | `errorCode`, `message` (sanitised) |
| `jobCancelled` | `atPhase` |

All events include `jobId`. No event includes a token, an absolute filesystem path, or a full attachment URL.

## TypeScript Types

Added to `backend/types.ts`:

```typescript
BackupJobEvent           // tagged union discriminated by `kind`
BackupJobProgressSnapshot
BackupJobCancellationRequest
BackupJobCancellationResult
```

`BackupJobResult.events?: BackupJobEvent[]` — optional; absent when the result was produced before this field was added.

## Service Layer

`AirBridgeService` interface adds:

```typescript
cancelBackupJob(jobId: string): Promise<BackupJobCancellationResult>
getBackupJobStatus(jobId: string): Promise<BackupJobProgressSnapshot | null>
```

Both implementations:
- **Live service**: `cancelBackupJob` calls `cancel_backup_job` Tauri command; falls back to `not_running` on IPC failure. `getBackupJobStatus` returns `null` (no command registered yet).
- **Mock service**: both return deterministic safe values (`not_running`, `null`).

## Safety Constraints

- No token in any event, snapshot, or cancellation result.
- No absolute filesystem path in any event, snapshot, or cancellation result.
- No attachment URLs in events.
- Token is cleared immediately on Cancel button click, before any service call.
- Cancel button is only rendered while `runState === "running"`.
- `activeJobId` is reset to `null` after cancellation and after reset.
- No background threads or background job registry in V0.1.

## Tests

`src/test/backupProgressCancellation.test.tsx`:
- `BackupJobProgressSnapshot` and `BackupJobCancellationResult` type model shapes
- Mock service: `cancelBackupJob` returns `not_running`; `getBackupJobStatus` returns `null`
- Cancel button absent when idle; visible while running; disappears after cancel
- Cancel button clears token
- Token not visible outside password input while running
- Timeline section absent when events empty or field absent
- Timeline renders all events with correct `data-event-kind` attributes
- Event timeline contains no token sentinel or absolute path
- `tableExportCompleted` event shows record count; `jobCancelled` event shows phase
- Run result with events → timeline rendered

`backup/job_progress.rs` (unit tests):
- Progress snapshot serialises with phase, status, table info
- Unknown total omits `totalTables` field
- No token sentinel in snapshot
- Cancellation request serialises job ID
- `not_running` constructor sets correct fields
- No token or absolute path in cancellation result
- All phases serialise correctly

`commands/backup_job.rs` (added tests):
- `cancel_backup_job` returns `not_running` / `wasRunning: false`
- Cancellation result serialises correctly
- No token or path in cancellation result
- Result `events` field is empty by default in V0.1
- Result with events serialises without token or path

## Future Path

- **Background job registry**: maintain a `HashMap<BackupJobId, (CancellationToken, ProgressSnapshot)>` in Tauri state. `run_backup_job` registers on start, deregisters on completion. `cancel_backup_job` and `getBackupJobStatus` look up the registry.
- **Live streaming**: emit `BackupJobEvent` to the frontend via Tauri events during execution. Frontend subscribes with `listen()` and updates a live timeline.
- **Progress snapshot polling**: wire `getBackupJobStatus` to the registry; frontend polls while `runState === "running"`.
- **Phase-boundary cancellation improvements**: currently the `CancellationToken` is polled at phase boundaries only. Mid-page-fetch cancellation would require async Tauri commands and cooperative cancellation inside the HTTP client.

See `docs/architecture/safe-backup-command-contract.md` and `docs/architecture/backup-file-picker-confirmation-flow.md` for the full execution flow and safety constraints.
