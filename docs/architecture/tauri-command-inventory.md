# Tauri Command Inventory

This document lists all commands registered in the Tauri invoke handler as of v0.1.0-alpha. Each entry describes the command's purpose, input sensitivity, file write behavior, network access, Airtable data modification risk, and safety status.

---

## Utility

### `greet`

| Property | Value |
|----------|-------|
| Purpose | Tauri project scaffold — returns a greeting string |
| Input sensitivity | Low — accepts a plain name string |
| Writes files | No |
| Network access | No |
| Can change Airtable data | No |
| Safety status | Unused in the UI; will be removed before stable release |

### `get_app_health`

| Property | Value |
|----------|-------|
| Purpose | Returns app name, version, and status string |
| Input sensitivity | None |
| Writes files | No |
| Network access | No |
| Can change Airtable data | No |
| Safety status | Read-only; safe |

---

## Connection

### `check_connection`

| Property | Value |
|----------|-------|
| Purpose | Validates a personal access token and checks permission scopes via the Airtable API |
| Input sensitivity | High — accepts a token string |
| Writes files | No |
| Network access | Yes — calls the Airtable `/meta/whoami` endpoint |
| Can change Airtable data | No |
| Safety status | Read-only API call. Token never stored; never in result |

---

## Catalog / Schema

### `list_workspaces`

| Property | Value |
|----------|-------|
| Purpose | Returns a list of workspace summaries (static placeholder data) |
| Input sensitivity | None |
| Writes files | No |
| Network access | No |
| Can change Airtable data | No |
| Safety status | Read-only; static data |

### `list_bases`

| Property | Value |
|----------|-------|
| Purpose | Returns a list of base summaries (static placeholder data) |
| Input sensitivity | None |
| Writes files | No |
| Network access | No |
| Can change Airtable data | No |
| Safety status | Read-only; static data |

### `list_accessible_bases`

| Property | Value |
|----------|-------|
| Purpose | Lists bases accessible to a given token via the Airtable API |
| Input sensitivity | High — accepts a token string |
| Writes files | No |
| Network access | Yes — calls the Airtable `/meta/bases` endpoint |
| Can change Airtable data | No |
| Safety status | Read-only API call. Token never stored; never in result |

### `get_base_schema`

| Property | Value |
|----------|-------|
| Purpose | Fetches full base schema (tables, fields, views) for a given base ID via the Airtable API |
| Input sensitivity | High — accepts a token string and a base ID |
| Writes files | No |
| Network access | Yes — calls the Airtable schema endpoint |
| Can change Airtable data | No |
| Safety status | Read-only API call. Token never stored; never in result |

---

## Backup Planning

### `create_backup_plan`

| Property | Value |
|----------|-------|
| Purpose | Generates a backup plan from a base schema request |
| Input sensitivity | Low — no token; accepts pre-fetched schema data |
| Writes files | No |
| Network access | No |
| Can change Airtable data | No |
| Safety status | Read-only planning; no side effects |

### `create_records_export_plan`

| Property | Value |
|----------|-------|
| Purpose | Generates a records export plan with table ordering and pagination parameters |
| Input sensitivity | Low — no token; accepts plan-level metadata |
| Writes files | No |
| Network access | No |
| Can change Airtable data | No |
| Safety status | Read-only planning; no side effects |

---

## Backup Package Inspection and Validation

### `list_backup_packages`

| Property | Value |
|----------|-------|
| Purpose | Returns a list of backup package summaries (static placeholder data) |
| Input sensitivity | None |
| Writes files | No |
| Network access | No |
| Can change Airtable data | No |
| Safety status | Read-only; static data |

### `inspect_backup_package`

| Property | Value |
|----------|-------|
| Purpose | Opens a `.airbridge` package and returns its contents, manifest, schema, checksums, and validation status |
| Input sensitivity | Low — accepts a local file path |
| Writes files | No — read-only; no files extracted |
| Network access | No |
| Can change Airtable data | No |
| Safety status | **Read-only.** No token required. Full path never included in result — filename only returned |

---

## Backup Execution

### `validate_backup_output_path`

| Property | Value |
|----------|-------|
| Purpose | Validates a proposed output path before any file write |
| Input sensitivity | Low — accepts a file path string |
| Writes files | No — validation only; no side effects |
| Network access | No |
| Can change Airtable data | No |
| Safety status | Safe; checks extension, parent directory existence, traversal, and directory conflicts |

### `run_backup_job`

| Property | Value |
|----------|-------|
| Purpose | Executes a backup job: fetches records from Airtable and writes a `.airbridge` package to the output path |
| Input sensitivity | **High** — accepts a token string and an output path |
| Writes files | **Yes** — writes one `.airbridge` ZIP package to the validated output path |
| Network access | **Yes** — calls the Airtable records API for each table |
| Can change Airtable data | No — read-only Airtable access |
| Safety status | **Requires explicit output path + exact confirmation text `CREATE BACKUP`.** Token consumed; never stored or in result. Full path never in result — filename only returned. Output path validated before write. |

### `cancel_backup_job`

| Property | Value |
|----------|-------|
| Purpose | Signals the in-progress backup job to stop |
| Input sensitivity | Low — accepts a job ID |
| Writes files | No |
| Network access | No |
| Can change Airtable data | No |
| Safety status | Safe cancellation signal; no destructive behavior |

---

## Restore Dry-Run Planning

### `list_restore_plans`

| Property | Value |
|----------|-------|
| Purpose | Returns a list of restore plan summaries (static placeholder data) |
| Input sensitivity | None |
| Writes files | No |
| Network access | No |
| Can change Airtable data | No |
| Safety status | Read-only; static data |

### `create_restore_dry_run_plan`

| Property | Value |
|----------|-------|
| Purpose | Generates a restore plan preview from an existing `.airbridge` package |
| Input sensitivity | Low — accepts a package path; no token |
| Writes files | No — reads package in memory only; no files extracted |
| Network access | No |
| Can change Airtable data | No |
| Safety status | **Read-only.** No token required. Full path never in result — filename only returned. No Airtable API calls |

### `create_restore_schema_plan`

| Property | Value |
|----------|-------|
| Purpose | Creates a schema creation plan (table and field ordering) from a dry-run result |
| Input sensitivity | Low — no token; no path |
| Writes files | No |
| Network access | No |
| Can change Airtable data | No |
| Safety status | **Read-only.** No token required. Filename only in result. `noChangesMade` always `true` |

### `create_restore_record_import_plan`

| Property | Value |
|----------|-------|
| Purpose | Creates a record import batch plan from a dry-run result and schema plan |
| Input sensitivity | Low — no token; no path |
| Writes files | No |
| Network access | No |
| Can change Airtable data | No |
| Safety status | **Read-only.** No token required. Filename only in result. `noChangesMade` always `true` |

---

## Restore Execution Gate

### `run_restore_execution`

| Property | Value |
|----------|-------|
| Purpose | Validates all restore preconditions (inspection, dry-run plan, target mode, token, confirmation) and returns a blocked/disabled result |
| Input sensitivity | **High** — accepts a token string and a package path |
| Writes files | No |
| Network access | No |
| Can change Airtable data | **No — write engine is disabled. Returns `readyButDisabled` when all gates pass.** |
| Safety status | **Write engine disabled.** `noChangesMade` is always `true`. Token checked for presence only; never stored or echoed in result. Full path never in result. No `succeeded` status exists. Confirmation text `RESTORE BACKUP` required |

---

## Restore Write Safety Gates (Gates 1–8)

### `verify_restore_sandbox_environment`

| Property | Value |
|----------|-------|
| Purpose | Gate 1 — Verifies the sandbox environment before any live write path is enabled |
| Input sensitivity | Low — no token; accepts sandbox classification flags |
| Writes files | No |
| Network access | No |
| Can change Airtable data | **No — write gate always returns `Disabled`. `writesEnabled` always `false`** |
| Safety status | **No token in request or result.** `noChangesMade` always `true`. `networkWritesAttempted` always `false`. CHK-10 always skipped |

### `validate_restore_confirmation_gate`

| Property | Value |
|----------|-------|
| Purpose | Gate 2 — Validates exact user confirmation phrase before any live write is enabled |
| Input sensitivity | Low — no token; accepts entered confirmation text and optional target label |
| Writes files | No |
| Network access | No |
| Can change Airtable data | **No — `Confirmed` does NOT enable writes. `writesEnabled` always `false`** |
| Safety status | **No token.** Case-sensitive exact match only. `noChangesMade` always `true` |

### `verify_restore_target_empty`

| Property | Value |
|----------|-------|
| Purpose | Gate 3 — Checks that the restore target base is empty before any writes |
| Input sensitivity | Low — no token; accepts target mode and optional table/record counts |
| Writes files | No |
| Network access | No |
| Can change Airtable data | **No — `Verified` does NOT enable writes. `writesEnabled` always `false`** |
| Safety status | **No token.** `noChangesMade` always `true`. `networkWritesAttempted` always `false` |

### `verify_destructive_operation_policy_gate`

| Property | Value |
|----------|-------|
| Purpose | Gate 4 — Blocks any declared delete, update, or overwrite operations in the planned write set |
| Input sensitivity | Low — no token; accepts list of declared operation kinds |
| Writes files | No |
| Network access | No |
| Can change Airtable data | **No — `Compliant` does NOT enable writes. `writesEnabled` always `false`** |
| Safety status | **No token.** No record payload. `noChangesMade` always `true` |

### `verify_attachment_upload_policy_gate`

| Property | Value |
|----------|-------|
| Purpose | Gate 5 — Blocks any attachment upload intents in the planned write set |
| Input sensitivity | Low — no token; accepts list of declared attachment field intents |
| Writes files | No |
| Network access | No |
| Can change Airtable data | **No — `Compliant` does NOT enable writes. `writesEnabled` always `false`** |
| Safety status | **No token. No full attachment URL.** Attachment file bytes never transferred. `noChangesMade` always `true` |

### `verify_schema_record_order_policy_gate`

| Property | Value |
|----------|-------|
| Purpose | Gate 6 — Verifies that schema creation precedes record insertion and records precede linked-record updates |
| Input sensitivity | Low — no token; accepts declared phase order flags |
| Writes files | No |
| Network access | No |
| Can change Airtable data | **No — `Compliant` does NOT enable writes. `writesEnabled` always `false`** |
| Safety status | **No token. No record payload.** `noChangesMade` always `true` |

### `verify_sandbox_write_testing_policy_gate`

| Property | Value |
|----------|-------|
| Purpose | Gate 7 — Verifies that sandbox write testing has been performed with complete evidence before any live write |
| Input sensitivity | Low — no token; accepts target classification, sandbox verification flag, and evidence struct |
| Writes files | No |
| Network access | No |
| Can change Airtable data | **No — `Compliant` does NOT enable writes. `writesEnabled` always `false`** |
| Safety status | **No token. No record payload.** Evidence filename is basename only. `noChangesMade` always `true` |

### `verify_live_write_confirmation_policy_gate`

| Property | Value |
|----------|-------|
| Purpose | Gate 8 — Validates the live-write-specific user confirmation phrase and checks all prior gate prerequisites |
| Input sensitivity | Low — no token; accepts entered confirmation text, optional target label, and optional prior gate statuses |
| Writes files | No |
| Network access | No |
| Can change Airtable data | **No — `Confirmed` does NOT enable writes. `writesEnabled` always `false`** |
| Safety status | **No token field in request or result. No filesystem path field. No record payload.** Case-sensitive exact match. Prior blocked gates cause `Blocked`. `noChangesMade` always `true`. `networkWritesAttempted` always `false` |

---

## Restore Write Engine Skeleton

### `preview_restore_write_engine`

| Property | Value |
|----------|-------|
| Purpose | Produces a six-phase write engine skeleton preview using counts from existing planning outputs |
| Input sensitivity | Low — no token; accepts package filename and optional count fields from schema/record plans |
| Writes files | No |
| Network access | No |
| Can change Airtable data | **No — all phases disabled. Write gate always returns `Disabled/DisabledByProductPolicy`** |
| Safety status | **No token required or accepted.** `noChangesMade` always `true`. Result status is never `"succeeded"`. Full path never in result — filename only. No Airtable API calls |

---

## Write Engine Foundations (Disabled)

### `preview_schema_write_request_plan`

| Property | Value |
|----------|-------|
| Purpose | Builds a sequenced list of schema write operations (CreateTable, CreateField, DeferLinkedField, ManualAction) from a schema plan summary and passes them through the dry-run executor |
| Input sensitivity | None — no token; accepts only count fields from the schema plan |
| Writes files | No |
| Network access | No |
| Can change Airtable data | **No — write gate always returns `Disabled/DisabledByProductPolicy`. No Airtable base, table, or field is created** |
| Safety status | **No token in request or result.** `noChangesMade` always `true`. `networkWritesAttempted` always `false`. Result status is never `"succeeded"`. No raw field payloads |

### `preview_record_write_request_plan`

| Property | Value |
|----------|-------|
| Purpose | Builds a sequenced list of record write operations (CreateRecordBatch, UpdateLinkedRecordBatch, Checkpoint, PreserveMetadataOnlyAttachment, SkipComputedField) from a record import plan summary and passes them through the dry-run executor |
| Input sensitivity | None — no token; accepts only count fields from the record import plan |
| Writes files | No |
| Network access | No |
| Can change Airtable data | **No — write gate always returns `Disabled/DisabledByProductPolicy`. No Airtable records are created, updated, or deleted** |
| Safety status | **No token in request or result.** `noChangesMade` always `true`. `networkWritesAttempted` always `false`. Result status is never `"succeeded"`. No raw record payloads. Old-to-new record ID mapping deferred to execution |

---

## Credential Storage

### `get_credential_storage_status`

| Property | Value |
|----------|-------|
| Purpose | Returns the OS keychain availability and whether a saved token exists for a given credential kind |
| Input sensitivity | Low — no token; accepts a credential kind string |
| Writes files | No |
| Network access | No |
| Can change Airtable data | No |
| Safety status | **Never returns the token value.** Returns availability status and a safe display string only. Keychain unavailable state handled safely |

### `save_airtable_token_to_keychain`

| Property | Value |
|----------|-------|
| Purpose | Saves an Airtable Personal Access Token to the OS keychain |
| Input sensitivity | **High** — accepts a token string |
| Writes files | No — writes to OS keychain only (not a file) |
| Network access | No |
| Can change Airtable data | No |
| Safety status | **Token forwarded to keychain API only; never returned, logged, or stored in files.** `CredentialSaveRequest` derives `Deserialize` only — cannot be serialized back. Result has no token field. Does not affect `evaluate_write_gate()` |

### `remove_airtable_token_from_keychain`

| Property | Value |
|----------|-------|
| Purpose | Removes a saved Airtable token from the OS keychain |
| Input sensitivity | Low — no token in request |
| Writes files | No |
| Network access | No |
| Can change Airtable data | No |
| Safety status | **Never returns the token value.** Returns success status only |

---

## Job History

### `list_job_history`

| Property | Value |
|----------|-------|
| Purpose | Returns a list of recent operation summaries (connection checks, backup jobs, restore operations) from the in-memory store |
| Input sensitivity | Low — accepts an optional filter |
| Writes files | No |
| Network access | No |
| Can change Airtable data | No |
| Safety status | **No token, no full paths, no record payloads, no attachment URLs in results.** Summary-level metadata only. In-memory store; discarded on restart |

### `clear_job_history`

| Property | Value |
|----------|-------|
| Purpose | Clears all entries from the in-memory job history store |
| Input sensitivity | None |
| Writes files | No |
| Network access | No |
| Can change Airtable data | No |
| Safety status | Safe; no-op in v0.1 (returns 0) |

---

## Reports and Logs

### `list_reports`

| Property | Value |
|----------|-------|
| Purpose | Returns a list of report summaries (static placeholder data) |
| Input sensitivity | None |
| Writes files | No |
| Network access | No |
| Can change Airtable data | No |
| Safety status | Read-only; static data |

### `list_logs`

| Property | Value |
|----------|-------|
| Purpose | Returns a list of log entries (static placeholder data) |
| Input sensitivity | None |
| Writes files | No |
| Network access | No |
| Can change Airtable data | No |
| Safety status | Read-only; static data |

### `list_compatibility_rules`

| Property | Value |
|----------|-------|
| Purpose | Returns the full field type compatibility rule set |
| Input sensitivity | None |
| Writes files | No |
| Network access | No |
| Can change Airtable data | No |
| Safety status | Read-only; static rule data |

---

## Summary Table

| Command | Token | Network | Writes Files | Changes Airtable | Gate / Confirmation |
|---------|-------|---------|-------------|-----------------|---------------------|
| `greet` | No | No | No | No | None |
| `get_app_health` | No | No | No | No | None |
| `check_connection` | Yes | Yes | No | No | None |
| `list_workspaces` | No | No | No | No | None |
| `list_bases` | No | No | No | No | None |
| `list_accessible_bases` | Yes | Yes | No | No | None |
| `get_base_schema` | Yes | Yes | No | No | None |
| `create_backup_plan` | No | No | No | No | None |
| `create_records_export_plan` | No | No | No | No | None |
| `list_backup_packages` | No | No | No | No | None |
| `inspect_backup_package` | No | No | No | No | None |
| `validate_backup_output_path` | No | No | No | No | None |
| `run_backup_job` | Yes | Yes | **Yes** | No | `CREATE BACKUP` |
| `cancel_backup_job` | No | No | No | No | None |
| `list_restore_plans` | No | No | No | No | None |
| `create_restore_dry_run_plan` | No | No | No | No | None |
| `create_restore_schema_plan` | No | No | No | No | None |
| `create_restore_record_import_plan` | No | No | No | No | None |
| `run_restore_execution` | Yes | No | No | **Disabled** | `RESTORE BACKUP` |
| `preview_restore_write_engine` | No | No | No | **Disabled** | None |
| `preview_schema_write_request_plan` | No | No | No | **Disabled** | None |
| `preview_record_write_request_plan` | No | No | No | **Disabled** | None |
| `get_credential_storage_status` | No | No | No | No | None |
| `save_airtable_token_to_keychain` | Yes (write-only) | No | No | No | None |
| `remove_airtable_token_from_keychain` | No | No | No | No | None |
| `list_job_history` | No | No | No | No | None |
| `clear_job_history` | No | No | No | No | None |
| `list_reports` | No | No | No | No | None |
| `list_logs` | No | No | No | No | None |
| `verify_restore_sandbox_environment` | No | No | No | **Disabled** | Gate 1 |
| `validate_restore_confirmation_gate` | No | No | No | **Disabled** | Gate 2 — exact phrase |
| `verify_restore_target_empty` | No | No | No | **Disabled** | Gate 3 |
| `verify_destructive_operation_policy_gate` | No | No | No | **Disabled** | Gate 4 |
| `verify_attachment_upload_policy_gate` | No | No | No | **Disabled** | Gate 5 |
| `verify_schema_record_order_policy_gate` | No | No | No | **Disabled** | Gate 6 |
| `verify_sandbox_write_testing_policy_gate` | No | No | No | **Disabled** | Gate 7 |
| `verify_live_write_confirmation_policy_gate` | No | No | No | **Disabled** | Gate 8 — exact phrase |
| `list_compatibility_rules` | No | No | No | No | None |

---

## Related Documents

- [Tauri Command Boundary](tauri-command-boundary.md)
- [Safe Backup Command Contract](safe-backup-command-contract.md)
- [Restore Execution Command Contract](restore-execution-command-contract.md)
- [Security Architecture](security-architecture.md)
