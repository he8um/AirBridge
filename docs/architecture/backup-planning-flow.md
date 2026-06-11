# Backup Planning Flow

## Status

Implemented. The backup planning flow produces a dry-run `BackupPlan` from schema data already held by the frontend. No backup file is written. No token is required at plan creation time.

## What happens

### Creating a backup plan

1. The user selects an accessible base on the Backups page.
2. The app calls `get_base_schema` (a separate command) to fetch the base's table and field metadata. This step requires a session-local token.
3. The frontend packages the returned schema into a `BackupPlanRequest` and calls `create_backup_plan`.
4. The `create_backup_plan` command receives the schema data directly in the request — it performs no API calls, reads no token, and writes no files.
5. The Rust planner classifies each field, generates warnings, and computes API read-page estimates.
6. The result (`BackupPlan`) is returned to the frontend and displayed as a dry-run summary.

### What the plan contains

| Field | Description |
|---|---|
| `baseId` / `baseName` | The selected base. |
| `scope` | `full`, `schemaOnly`, or `recordsOnly`. |
| `tableCount` / `totalFieldCount` | Aggregate counts. |
| `tables` | Per-table field lists with compatibility labels. |
| `compatibility` | Rollup of restorable / metadata-only / unknown field counts. |
| `warnings` | Per-field notices (computed fields, attachments, linked records). |
| `estimate` | Estimated Airtable API read pages. `Unknown` when record counts are absent. |
| `dryRun` | Always `true` in this phase. |
| `outputPackagePath` | Always `null` in this phase — no file is written. |

## Token flow

The token is only present during `get_base_schema`. It is not carried into `BackupPlanRequest` and is not required by `create_backup_plan`. This keeps token exposure scoped to the schema-fetch step and prevents token persistence in plan state.

## Field compatibility labels

| Label | Meaning |
|---|---|
| `restorable` | Field value can be round-tripped through a backup and restore. |
| `metadataOnly` | Field schema is captured; the value cannot be restored (formula, rollup, count, lookup, system fields). |
| `unknown` | Field behaviour during restore requires manual review (attachments, linked records). |

## Warning codes

| Code | Severity | Trigger |
|---|---|---|
| `COMPUTED_FIELD` | Info | `formula`, `rollup`, `count`, `lookup` fields. |
| `SYSTEM_FIELD` | Info | `createdTime`, `lastModifiedTime`, `createdBy`, `lastModifiedBy`, `autoNumber`, `externalSyncSource`. |
| `ATTACHMENT_METADATA_ONLY` | Warning | `multipleAttachments` fields. Metadata only — file content is not exported. |
| `LINKED_RECORD_REMAPPING` | Warning | `multipleRecordLinks` fields. Record ID references are captured; restore requires remapping. |

## API estimate calculation

- Base page size: 100 records per page.
- Per table: `ceil(recordCount / 100)` pages, or `Unknown` if record count is not available.
- Total: sum of per-table pages, or `Unknown` if any table is unknown.
- Schema requests: 1 (one base metadata call).

## Rust modules

| Module | Responsibility |
|---|---|
| `src-tauri/src/backup/planner.rs` | `create_plan()` — orchestrates field classification, warning generation, and estimate calculation. |
| `src-tauri/src/backup/warnings.rs` | `warnings_for_field()` — returns per-field warning list by field type. |
| `src-tauri/src/backup/estimates.rs` | `estimate_record_pages()` and `build_estimate()` — page math. |
| `src-tauri/src/commands/backup.rs` | `create_backup_plan` Tauri command — converts frontend request to domain types and calls `create_plan()`. |
| `src-tauri/src/models/backup_plan.rs` | All plan domain types (`BackupPlan`, `BackupPlanRequest`, `RecordReadEstimate`, etc.). |

## Frontend

| File | Responsibility |
|---|---|
| `src/backend/types.ts` | TypeScript mirror of all plan types. |
| `src/backend/commands.ts` | `createBackupPlan()` — `safeInvoke` wrapper for the Tauri command. |
| `src/services/airBridgeService.ts` | `createBackupPlan` method on the `AirBridgeService` interface. |
| `src/services/liveAirBridgeService.ts` | Production implementation — delegates to `commands.createBackupPlan`. |
| `src/services/mockAirBridgeService.ts` | Test implementation — returns deterministic plan from request data; no token required. |
| `src/pages/BackupsPage.tsx` | `BackupPlanningCard` component — base selector, schema summary, "Generate Backup Plan" button, and plan result display. |

## Stop conditions for this phase

The following operations are outside scope and will not be triggered by the planning flow:

- Record export or fetching.
- Writing any `.airbridge` package or other output file.
- Token persistence.
- Restore operations.
