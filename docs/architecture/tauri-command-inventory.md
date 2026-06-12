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
| `run_restore_execution` | Yes | No | No | **Disabled** | `RESTORE BACKUP` |
| `list_reports` | No | No | No | No | None |
| `list_logs` | No | No | No | No | None |
| `list_compatibility_rules` | No | No | No | No | None |

---

## Related Documents

- [Tauri Command Boundary](tauri-command-boundary.md)
- [Safe Backup Command Contract](safe-backup-command-contract.md)
- [Restore Execution Command Contract](restore-execution-command-contract.md)
- [Security Architecture](security-architecture.md)
