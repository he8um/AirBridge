# Local Job History

## Overview

AirBridge maintains a local activity log of recent operations so users can see what has been done without leaving the application. The history system is designed for safety: it records only summary-level information and explicitly strips everything that must not be stored.

## Safety Constraints

History items must not contain:

- API tokens or token-like values
- Full filesystem paths (filename only is stored)
- Record payloads or field values from Airtable
- Full attachment URLs (presence is noted, content is not stored)
- Local usernames derived from paths

These constraints are enforced at the Rust layer by the `history/redaction.rs` module before any value is included in a history item.

## Module Layout

| File | Purpose |
|------|---------|
| `history/models.rs` | All data types: `JobHistoryItem`, `JobHistoryKind`, `JobHistoryStatus`, `JobHistorySource`, `JobHistorySummary`, `JobHistoryWarning`, `JobHistoryError`, `JobHistoryFilter`, `JobHistoryListResult` |
| `history/redaction.rs` | Path-to-filename extraction, token-like value detection, message sanitization |
| `history/store.rs` | `JobHistoryStore` trait and `InMemoryJobHistoryStore` implementation |
| `history/summaries.rs` | Builder functions: `from_inspection`, `from_dry_run_plan`, `from_schema_plan`, `from_record_import_plan`, `from_restore_execution_blocked`, `from_backup_execution`, `build_history_item` |
| `commands/history.rs` | Tauri commands: `list_job_history`, `clear_job_history` |

## JobHistoryKind Values

| Kind | Description |
|------|------------|
| `connectionCheck` | Token connection and permission check |
| `backupPlan` | Backup plan generated |
| `recordsExportPlan` | Records export plan generated |
| `backupExecution` | Backup package written |
| `packageInspection` | Package opened and inspected |
| `restoreDryRun` | Restore dry-run plan generated |
| `restoreSchemaplan` | Restore schema creation plan generated |
| `restoreRecordImportPlan` | Restore record import plan generated |
| `restoreExecutionAttempt` | Restore execution gate attempted (blocked in V0.1) |

## JobHistoryStatus Values

| Status | Meaning |
|--------|---------|
| `planned` | Queued, not yet started |
| `running` | In progress |
| `succeeded` | Completed without warnings |
| `succeededWithWarnings` | Completed with non-blocking warnings |
| `blocked` | Precondition not met; execution did not proceed |
| `failed` | Encountered an error |
| `cancelled` | Cancelled by the user |

## Tauri Commands

### `list_job_history`

```
list_job_history(filter?: JobHistoryFilter) -> JobHistoryListResult
```

- No token in request or response.
- No full paths in response.
- Accepts optional filter by `kind`, `status`, and `limit`.
- Returns items most-recent-first.
- In V0.1 returns deterministic in-memory data.

### `clear_job_history`

```
clear_job_history() -> usize
```

- No-op in V0.1 (no persistent store). Returns 0.

## Store Abstraction

The `JobHistoryStore` trait allows a SQLite-backed implementation to be added in a future release without changing the command layer:

```rust
pub trait JobHistoryStore {
    fn add(&mut self, item: JobHistoryItem);
    fn list(&self, filter: &JobHistoryFilter) -> JobHistoryListResult;
    fn clear(&mut self);
    fn len(&self) -> usize;
}
```

`InMemoryJobHistoryStore` is the only implementation in V0.1.

## Redaction

`redaction.rs` enforces three rules:

1. **Path → filename** — `redact_path_to_filename(path)` strips all directory components from Unix and Windows paths.
2. **Token-like values** — `reject_or_redact_token_like_values(value)` detects Airtable PAT patterns (`pat…` with length > 12, all alphanumeric) and Bearer strings and replaces them with `[redacted]`.
3. **Message sanitization** — `sanitize_history_message(message)` scans a free-text message for Bearer tokens, PAT prefixes, home directory paths, and attachment URLs and redacts them.

## Frontend

The `JobHistoryPanel` component (at `features/backups/JobHistoryPanel.tsx`) renders the history on the Reports page. It:

- Calls `service.listJobHistory({ limit: 20 })` on mount.
- Shows each item's title, kind label, filename (if present), warning/error counts, validation status (if present), and timestamp.
- Renders an empty state when no items are available.
- Shows a note that history is memory-only and does not persist between sessions.
- Never renders full paths, tokens, or record payloads.

## Persistence

In V0.1 history is stored in memory only. It does not persist between application restarts.

Future work: a SQLite-backed implementation of `JobHistoryStore` that persists history to the application data directory without storing tokens, full paths, or record payloads.

## Related

- [Tauri Command Inventory](tauri-command-inventory.md)
- [Restore Record Import Planning](restore-record-import-planning.md)
- [Security and Privacy QA](../qa/security-privacy-qa.md)
