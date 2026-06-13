# Restore Record Import Planning

## Overview

The restore record import planner converts package record metadata and schema/restore plans into a
safe, ordered import plan. The planner produces a `RestoreRecordImportPlan` describing batch
structure, field import policies, linked record second-pass updates, attachment handling, retry
policy, and per-table checkpoints — without performing any Airtable write operations.

## Design Constraints

- No Airtable API calls.
- No token required in the planning request.
- No file extraction or writes.
- Result filename is never a full path.
- `noChangesMade` is always `true`.
- No record creation, no linked record update, no attachment upload/download in this layer.

## Entry Point

**Tauri command:** `create_restore_record_import_plan`  
**Rust handler:** `commands::restore::create_restore_record_import_plan`  
**Planner function:** `restore::record_import_planner::create_record_import_plan`

## Request Structure

```
RestoreRecordImportPlanRequest {
    package_filename: String,     // filename only — no path
    dry_run_status: String,       // "ready" | "readyWithWarnings" | "blocked"
    schema_plan_status: String,   // "ready" | "readyWithWarnings" | "blocked"
    target_mode: RestoreTargetMode,
    target_base_name: Option<String>,
    tables: Vec<RecordImportTableInput>,
    // No token field
}
```

## Gate Conditions

The planner returns `Blocked` if:
1. `dry_run_status` is not `"ready"` or `"readyWithWarnings"`.
2. `schema_plan_status` is not `"ready"` or `"readyWithWarnings"`.
3. `tables` is empty.

## Batch Strategy

- Batch size: **10** (`AIRTABLE_WRITE_BATCH_SIZE`).
- **First pass:** create records without linked record fields.
- **Second pass:** update linked record fields after first-pass ID mapping is complete.
- Batch count uses integer ceiling division: `ceil(record_count / 10)`.
- If `record_count` is unknown, batch count is `None` and batches are empty.

## Field Import Policy

| Field type | Policy |
|---|---|
| `singleLineText`, `multilineText`, `email`, `url`, `phoneNumber`, `number`, `currency`, `percent`, `rating`, `checkbox`, `date`, `dateTime`, `duration`, `barcode`, `singleSelect`, `multipleSelects` | `Include` |
| `multipleRecordLinks` | `DeferToLinkedRecordPass` |
| `multipleAttachments` | `MetadataOnly` |
| `formula`, `rollup`, `lookup`, `autoNumber`, `createdTime`, `lastModifiedTime`, `count`, `singleCollaborator`, `multipleCollaborators`, `createdBy`, `lastModifiedBy` | `Skip` |
| unknown | `Skip` |

## Record ID Mapping

Strategy: `MapSourceRecordIdToCreatedRecordId`.

Source record IDs from the backup are mapped to new Airtable record IDs created during the first
pass. The mapping is only available at execution time — the plan describes the strategy but does not
contain pre-assigned IDs.

`remapping_required` is `true` if the table has any `multipleRecordLinks` fields.

## Attachment Policy

All `multipleAttachments` fields use `MetadataOnly` policy. File bytes are not downloaded or
re-uploaded. Users must manually re-attach files after restore.

## Checkpoint Plan

Each table plan includes a `RestoreRecordImportCheckpointPlan` describing:
- The batch index at which a checkpoint would be recorded.
- A `source_record_id_offset_placeholder` (`<source_record_id_at_checkpoint>`) — replaced at
  execution time with the actual record ID offset.
- The `completed_phase` at the checkpoint.

## Retry Policy

Default retry policy:
- `max_retries_on_rate_limit`: 5
- `initial_backoff_ms`: 1000
- `backoff_multiplier`: 2.0

On a 429 response, the engine waits for `Retry-After` (or `initial_backoff_ms` if absent) and
retries. Backoff doubles each attempt up to the maximum.

## Modules

| Module | Responsibility |
|---|---|
| `restore/record_import_plan.rs` | All model structs and enums |
| `restore/record_import_batches.rs` | Batch count computation and batch plan builders |
| `restore/record_mapping.rs` | Record ID mapping plan |
| `restore/linked_record_updates.rs` | Second-pass linked record update plans |
| `restore/attachment_restore_policy.rs` | Attachment import policy builders |
| `restore/record_import_warnings.rs` | Per-table warning generation |
| `restore/record_import_planner.rs` | Main orchestrator |

## Warning Codes

| Code | Condition |
|---|---|
| `RECORD_COUNT_UNKNOWN` | `record_count` is `None` for a table |
| `ATTACHMENT_METADATA_ONLY` | Table has attachment fields |
| `COMPUTED_FIELDS_SKIPPED` | Table has fields with `Skip` policy |
| `LINKED_RECORD_SECOND_PASS_REQUIRED` | Table has linked record fields |

## Frontend

- **Panel:** `RestoreRecordImportPlanPanel` (`features/backups/RestoreRecordImportPlanPanel.tsx`)
- **Page:** Rendered in `RestorePage.tsx` after `RestoreSchemaPlanPanel`, before the execution gate.
- No token input in the panel. No execute button in the panel.

## Related Documents

- [restore-schema-creation-planning.md](restore-schema-creation-planning.md)
- [restore-execution-command-contract.md](restore-execution-command-contract.md)
- [restore-dry-run-planning.md](restore-dry-run-planning.md)
