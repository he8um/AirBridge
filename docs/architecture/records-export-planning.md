# Records Export Planning

## Status

Planning layer implemented. No live record fetching. No package writing.

## Overview

Given a backup plan produced by the backup planning flow, AirBridge can generate a records export plan. The plan describes how records will be exported in a future phase — it does not export them.

The plan is dry-run only. No Airtable API calls are made. No files are written. No tokens are required.

## What the plan produces

For each table in a backup plan:

- Record count state (`known` or `unknown`)
- Estimated API pages (ceiling division by page size if count is known; `unknown` otherwise)
- Page size (default: 100, matching the Airtable API maximum)
- JSONL output plan — target entry path inside the `.airbridge` package (`tables/{tableId}/records.jsonl`)
- Table metadata path (`tables/{tableId}/table.json`)
- Fields metadata path (`tables/{tableId}/fields.json`)
- Per-field extraction plans including linked record and attachment policies
- Linked record extraction plan when `multipleRecordLinks` fields are present
- Attachment metadata extraction plan when `multipleAttachments` fields are present
- Per-table warnings

## Page size and pagination

- Default page size: 100 (matches Airtable API maximum)
- Estimated pages formula: `ceil(record_count / page_size)`
- Zero records: 1 page (required to confirm empty)
- Unknown count: page estimate is `unknown` — determined at export time

## JSONL output plan

Each table's records will be written as JSONL (one JSON record per line) to:

```
tables/{tableId}/records.jsonl
```

Entry paths use stable table IDs, not human-readable names. No absolute filesystem paths are embedded.

## Linked record extraction plan

When a table contains `multipleRecordLinks` fields:

- Policy: `remappingRequiredForRestore`
- Record ID references are captured in the JSONL output
- Restore requires ID remapping to reconcile IDs in the destination base

## Attachment metadata extraction plan

When a table contains `multipleAttachments` fields:

- Policy: `metadataOnly` (V0.1)
- Only attachment metadata is exported: filename, URL, size, MIME type
- Attachment file content is not downloaded or stored

## Warnings

| Code | Severity | Meaning |
|---|---|---|
| `UNKNOWN_RECORD_COUNT` | warning | Record count is not yet known; page count will be determined at export time |
| `ATTACHMENT_METADATA_ONLY` | warning | Attachment fields present; only metadata exported, not file content |
| `LINKED_RECORD_REMAPPING` | warning | Linked record references captured; restore requires ID remapping |

## Progress model

The progress plan describes units of work at planning time:

| Unit | Status at planning time |
|---|---|
| Schema | `notStarted` |
| TableRecords (per table) | `notStarted` |
| LinkedReferences | `notStarted` (if linked fields present) |
| AttachmentMetadata | `notStarted` (if attachment fields present) |
| PackageWrite | `future` (requires export to complete first) |
| Validation | `future` (requires package write) |

`total_known_items` is the sum of estimated pages across all tables, available only when all record counts are known.

## Checkpoint model

The `ExportCheckpointPlan` struct models checkpoint state for a future resume capability:

- `backup_job_id` — identifies the export job
- `table_id` — table currently being exported
- `last_offset` — opaque Airtable pagination cursor (absent at start)
- `records_exported` — count of records exported so far for this table
- `updated_at` — ISO 8601 timestamp of last checkpoint

No persistence is implemented in this phase. This is a model-only definition.

## Rust module layout

```
backup/
  export_paths.rs   — entry path helpers (records.jsonl, table.json, fields.json, records.csv)
  export_plan.rs    — RecordsExportPlan, TableExportPlan, builder (create_export_plan)
  progress.rs       — ExportProgressPlan, ProgressUnit, build_progress_plan
  checkpoints.rs    — ExportCheckpointPlan (model-only, no persistence)
```

## TypeScript mirror types

Defined in `src/backend/types.ts`:

- `RecordCountState` — `{ type: "known"; count: number }` | `{ type: "unknown" }`
- `RequestEstimate` — `{ type: "known"; pages: number }` | `{ type: "unknown" }`
- `JsonlOutputPlan` — `{ entryPath, plannedOnly }`
- `LinkedRecordExtractionPlan` — `{ fieldId, fieldName, policy, restoreNote }`
- `AttachmentMetadataExtractionPlan` — `{ fieldId, fieldName, policy, contentNote }`
- `FieldExtractionPlan` — per-field extraction with optional linked/attachment sub-plans
- `TableExportPlan` — full per-table export plan
- `RecordsExportPlan` — top-level plan (always `plannedOnly: true`, no `outputPackagePath`)
- `RecordsExportPlanRequest` — input carrying a `BackupPlan`

## Future path

1. Add live record pagination via Airtable API (uses existing `pagination.rs` and `records.rs`)
2. Write JSONL lines per page into `PackageInput.tables`
3. Wire `PackageInput` into `write_package` to produce a real `.airbridge` file
4. Resume from checkpoint if export is interrupted
5. Validate the written package using the existing `validate_package` function
