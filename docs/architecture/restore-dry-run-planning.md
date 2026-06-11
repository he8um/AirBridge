# Restore Dry-Run Planning

## Purpose

The restore dry-run planner reads an existing `.airbridge` package and produces a structured plan describing what a restore would do — without making any Airtable API calls, creating any bases or tables, writing any files, or requiring a token.

The plan is package-based only. It uses data already captured inside the package (manifest, schema) to classify field compatibility, identify linked record remapping requirements, describe attachment limitations, and define the record import ordering.

## Design requirements

- No Airtable API calls at any stage.
- No token required.
- No files extracted from the package.
- No write operations.
- Full path never appears in the result or UI — filename only.
- `noChangesMade: true` is always set in every result regardless of status.
- Blocked or invalid packages return a blocked plan, not an error thrown to the frontend.

## Rust components

```
src/restore/
  mod.rs              — module exports
  plan.rs             — all plan structs and enums
  compatibility.rs    — field type classification
  ordering.rs         — record import ordering plan
  warnings.rs         — warning generation per table
  dry_run.rs          — core planner (create_dry_run_plan)
src/commands/restore.rs — Tauri command: create_restore_dry_run_plan
```

### Plan structs (restore/plan.rs)

| Struct | Purpose |
|--------|---------|
| `RestoreDryRunRequest` | Input: package path, target mode, optional base name |
| `RestoreDryRunPlan` | Top-level plan result |
| `RestorePackageSummary` | Source package metadata (filename only, no path) |
| `RestoreTablePlan` | Per-table plan with field counts and compatibility |
| `RestoreFieldPlan` | Per-field classification and note |
| `RestoreLinkedRecordPlan` | Linked field remapping details |
| `RestoreAttachmentPlan` | Attachment metadata-only limitation |
| `RestoreRecordOrderingPlan` | Import ordering steps |
| `RestoreDryRunWarning` | Warning with optional table/field context |
| `RestoreDryRunError` | Blocking error |

### Field compatibility (restore/compatibility.rs)

```
Supported            — direct restore: text, number, select, date, checkbox, etc.
PartiallySupported   — multipleRecordLinks (requires remapping)
MetadataOnly         — multipleAttachments, rollup, lookup, autoNumber, createdTime, etc.
Unsupported          — formula (definition captured, value not restorable)
ManualActionRequired — collaborator fields, unknown types
```

### Planner flow (restore/dry_run.rs)

1. Extract filename via `Path::file_name()` (no directory component).
2. Call `validate_package()` — if `Invalid`, return a blocked plan immediately.
3. Open package with `BackupPackageReader` — if error, return blocked plan.
4. Read and parse `manifest.json` — if error, return blocked plan.
5. Read and parse `schema.json` — internal serde structs, not airtable module types.
6. Build `RestorePackageSummary` from manifest fields.
7. For each table in schema: classify fields, collect linked record and attachment plans.
8. Generate warnings per table (`warnings_for_fields`).
9. Convert validation warnings with `VALIDATION_` prefix.
10. Set status: `Ready` if no warnings, `ReadyWithWarnings` otherwise.
11. Return plan with `no_changes_made: true`.

### Tauri command

```rust
#[tauri::command]
pub fn create_restore_dry_run_plan(request: RestoreDryRunRequest) -> RestoreDryRunPlan
```

No token parameter. Synchronous. Returns a plan in all cases — errors are represented as a blocked plan with `errors` array, not a Tauri error result.

## TypeScript components

```
src/backend/types.ts           — RestoreDryRunRequest, RestoreDryRunPlan, and all sub-types
src/backend/commands.ts        — createRestoreDryRunPlan (safeInvoke wrapper)
src/services/airBridgeService.ts — interface method
src/services/liveAirBridgeService.ts — live implementation (IPC fallback to blocked plan)
src/services/mockAirBridgeService.ts — deterministic mock (readyWithWarnings with warnings)
```

## UI component

`src/features/backups/RestoreDryRunPanel.tsx`

State machine: `idle → loading → done`.

Controls:
- File picker (reuses `PackageInspectionPicker`) — shows filename only, never full path
- Target mode selector: New base / Empty existing base
- Optional target base name input
- Generate plan button (disabled until file selected)

Result display:
- Status badge (Ready / Ready with warnings / Blocked)
- "No Airtable changes were made." notice (always visible)
- Package summary (source base, counts)
- Table plans with per-field compatibility badges
- Linked record remapping notices per table
- Attachment metadata-only notices per table
- Record import ordering steps
- Warnings list with table/field context

Not present: restore execution button, token input, full file path.

## Warning codes

| Code | Condition |
|------|-----------|
| `ATTACHMENT_METADATA_ONLY` | `multipleAttachments` field present in table |
| `LINKED_RECORD_REMAPPING_REQUIRED` | `multipleRecordLinks` field present |
| `COMPUTED_FIELD_NOT_RESTORED` | `rollup`, `lookup`, `autoNumber`, `createdTime`, `lastModifiedTime` |
| `UNSUPPORTED_FIELD_MANUAL_RECREATION` | `formula` field |
| `MANUAL_ACTION_REQUIRED` | collaborator fields and unknown types |
| `VALIDATION_*` | Forwarded from package validation with prefix |

## Safety guarantees

- `noChangesMade: true` in every plan result at the API level.
- No token ever flows through this command path.
- Full path is extracted to filename before any result is returned.
- No package entries are extracted to disk.
- The `blocked_plan()` helper ensures all error states still carry the safety contract.
