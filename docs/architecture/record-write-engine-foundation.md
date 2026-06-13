# Record Write Engine Foundation

This document describes the record write engine foundation — the request plan builder, the dry-run executor skeleton, and the Tauri command that surfaces them. These components are implemented behind a hard-disabled product gate: no Airtable record writes are made at any point.

---

## Overview

The record write engine foundation adds the structural layer between the existing record import plan (which identifies what batches would need to be created and updated) and a future live write engine (which would execute them). It introduces:

- A **request plan builder** that turns a `RestoreRecordImportPlan` into a sequenced list of write operations.
- A **dry-run executor skeleton** that always consults the write gate and always returns disabled.
- A **Tauri command** (`preview_record_write_request_plan`) that exposes the plan to the frontend without requiring a token, making any Airtable calls, or creating or modifying any records.

---

## Components

### `restore/record_write_requests.rs`

Defines the types and builder for record write request plans.

#### Types

```
RecordWriteOperationKind
  CreateRecordBatch | UpdateLinkedRecordBatch | Checkpoint
  | PreserveMetadataOnlyAttachment | SkipComputedField | ManualAction

RecordWriteOperationStatus
  Planned | Blocked | Disabled
  (no Succeeded — intentionally absent)

RecordWriteOperation
  index, kind, status, table_id, table_name
  batch_index?, planned_record_count?, linked_field_count?
  attachment_policy?, skipped_field_name?, skipped_field_type?
  note, no_changes_made

RecordWriteBlockedReason
  DisabledByProductPolicy | RecordImportPlanNotReady | NoTablesInPlan

RecordWriteRequestPlan
  filename, status, blocked_reason?, operations[]
  create_batch_op_count, linked_update_op_count, checkpoint_op_count
  attachment_op_count, skipped_field_op_count, total_op_count
  total_first_pass_batches, total_second_pass_batches
  warnings[], no_changes_made, network_writes_attempted
```

`no_changes_made` is always `true`. `network_writes_attempted` is always `false`.

#### Builder: `build_record_write_request_plan`

Takes a `&RestoreRecordImportPlan` and returns a `RecordWriteRequestPlan`.

**Phase ordering (invariant):**

| Phase | Kind | Source |
|-------|------|--------|
| 1 | `CreateRecordBatch` | First-pass batches per table; one op per batch (unknown count → single representative op) |
| 2 | `UpdateLinkedRecordBatch` | Second-pass batches per table with linked fields; note states ID mapping unavailable |
| 3 | `Checkpoint` | One per table |
| 4 | `PreserveMetadataOnlyAttachment` | One per attachment policy per table |
| 5 | `SkipComputedField` | One per field with Skip policy per table |

This ordering enforces the structural dependency: record batches must be created before linked record update passes can reference the new record IDs. Checkpoints follow batch creation. Attachment metadata and skipped fields are listed last as advisory items.

**Status logic:**

- If `import_plan.status == Blocked` → returns `Blocked` with `RecordImportPlanNotReady`.
- If `table_plans.is_empty()` → returns `Blocked` with `NoTablesInPlan`.
- Otherwise → returns `Disabled` with `blocked_reason = Some(DisabledByProductPolicy)`.

No `Planned` status is returned in the current version — the product gate always prevents advancement past `Disabled`.

**Old-to-new record ID mapping:**

The `RestoreRecordMappingStrategy::UnavailableUntilExecution` value is represented faithfully. All `UpdateLinkedRecordBatch` operations include a note stating "ID mapping unavailable until execution". New record IDs are only available after first-pass creation; the plan builder cannot resolve them.

---

### `restore/record_write_executor.rs`

Executor skeleton that always consults the write gate and returns disabled. No Airtable calls are made.

#### Type: `RecordWriteDryRunResult`

```
filename, status (RestoreWriteEngineStatus), disabled_reason (RestoreWriteDisabledReason)
message, operations_planned, operations_executed (always 0)
create_batch_ops_planned, linked_update_ops_planned, checkpoint_ops_planned
attachment_ops_planned, skipped_field_ops_planned
warnings[], no_changes_made (always true), network_writes_attempted (always false)
```

#### Function: `execute_record_write_dry_run`

1. Always calls `evaluate_write_gate()` first.
2. If `request_plan.status == Blocked`, surfaces `BlockedByInvalidPlan`.
3. Otherwise returns `Disabled` with the gate's static message.
4. `operations_executed` is always `0`.
5. `no_changes_made` is always `true`.
6. `network_writes_attempted` is always `false`.

---

### `restore/record_write_result.rs`

Request and result types for the Tauri command boundary.

#### `RecordWriteRequestPlanRequest` (no token field)

```
package_filename, record_import_plan_status, table_count
total_first_pass_batches, total_second_pass_batches
attachment_field_count, skipped_field_count
```

The request carries only count fields derived from the record import plan summary. No token, no path, no record payloads, no field-level data.

#### `RecordWriteRequestPlanResult`

```
filename, status, blocked_reason?, disabled_reason?, message
create_batch_op_count, linked_update_op_count, checkpoint_op_count
attachment_op_count, skipped_field_op_count, total_op_count
total_first_pass_batches, total_second_pass_batches
warnings[], no_changes_made, network_writes_attempted
```

No `Succeeded` status value. `no_changes_made` is always `true`. `network_writes_attempted` is always `false`.

---

### `commands/restore.rs` — `preview_record_write_request_plan`

Tauri command registered as `"preview_record_write_request_plan"`.

**Inputs:** `RecordWriteRequestPlanRequest` (no token).

**Logic:**

1. Returns `Blocked(RecordImportPlanNotReady)` if `record_import_plan_status == "blocked"`.
2. Returns `Blocked(NoTablesInPlan)` if `table_count == 0`.
3. Synthesizes a minimal `RestoreRecordImportPlan` from the count fields.
4. Calls `build_record_write_request_plan` → `execute_record_write_dry_run`.
5. Always returns disabled via `evaluate_write_gate()` message.

The synthesis step distributes first-pass batches evenly across tables, derives record counts as `batch_count × AIRTABLE_WRITE_BATCH_SIZE` (10), adds synthetic linked record fields based on second-pass batch presence, and spreads attachment/skipped fields onto the first table. This avoids requiring full field data from the frontend.

---

## TypeScript Layer

### `backend/types.ts`

```typescript
type RecordWriteOperationKind =
  | "createRecordBatch" | "updateLinkedRecordBatch" | "checkpoint"
  | "preserveMetadataOnlyAttachment" | "skipComputedField" | "manualAction"
type RecordWriteOperationStatus = "planned" | "blocked" | "disabled"
type RecordWriteBlockedReason = "disabledByProductPolicy" | "recordImportPlanNotReady" | "noTablesInPlan"
interface RecordWriteOperation { ... }
interface RecordWriteRequestPlan { ... }
interface RecordWriteRequestPlanRequest { ... }   // no token field
interface RecordWriteRequestPlanResult { ... }    // no token field, no "succeeded" status, no record payloads
```

### `backend/commands.ts`

```typescript
export async function previewRecordWriteRequestPlan(
  request: RecordWriteRequestPlanRequest,
): Promise<RecordWriteRequestPlanResult | null>
```

Returns `null` if Tauri IPC is unavailable (jsdom / browser context). The live service converts `null` to a safe disabled fallback.

### `services/airBridgeService.ts`

```typescript
previewRecordWriteRequestPlan(request: RecordWriteRequestPlanRequest): Promise<RecordWriteRequestPlanResult>
```

### `services/liveAirBridgeService.ts`

Delegates to `commands.previewRecordWriteRequestPlan`. If `null` is returned, returns a safe fallback:

```typescript
{
  status: "disabled",
  disabledReason: "disabledByProductPolicy",
  noChangesMade: true,
  networkWritesAttempted: false,
  ...zeroOpCounts,
}
```

### `services/mockAirBridgeService.ts`

`previewRecordWriteRequestPlanImpl`:

- Returns `blocked/recordImportPlanNotReady` if `recordImportPlanStatus === "blocked"`.
- Returns `blocked/noTablesInPlan` if `tableCount === 0`.
- Otherwise returns `disabled` with op counts derived from request fields:
  - `createBatchOpCount = totalFirstPassBatches`
  - `linkedUpdateOpCount = totalSecondPassBatches`
  - `checkpointOpCount = tableCount`
  - `attachmentOpCount = attachmentFieldCount`
  - `skippedFieldOpCount = skippedFieldCount`
  - `totalOpCount = sum of all`
- `noChangesMade: true` and `networkWritesAttempted: false` always.

---

## Safety Invariants

| Property | Enforcement |
|----------|-------------|
| No token in request | `RecordWriteRequestPlanRequest` has no `token` field |
| No token in result | `RecordWriteRequestPlanResult` has no `token` field |
| No raw record payloads | Result contains only counts and operation metadata — no field values, record IDs, or record contents |
| No Airtable API calls | `execute_record_write_dry_run` calls only the write gate; no HTTP client constructed |
| No records created | No `create_records` / `update_records` / `delete_records` call exists in any reachable code path |
| `noChangesMade` always true | Hard-coded in all types; Rust and frontend tests assert this |
| `networkWritesAttempted` always false | Hard-coded in all types; Rust and frontend tests assert this |
| No `Succeeded` status | `RecordWriteOperationStatus` has `Planned`, `Blocked`, `Disabled` only |
| Write gate always disabled | `evaluate_write_gate()` returns `Disabled/DisabledByProductPolicy` unconditionally |
| Old-to-new ID mapping deferred | `UpdateLinkedRecordBatch` notes explicitly state ID mapping is unavailable until execution |

---

## Testing

**Rust unit tests (in `record_write_requests.rs`):** 44+ tests covering phase ordering, operation counts, blocked/disabled status, `no_changes_made`, `network_writes_attempted`, the ordering invariant, unknown record count handling, and linked record second-pass ID mapping notes.

**Rust unit tests (in `record_write_executor.rs`):** 18 tests covering the disabled path, blocked-plan path, always-zero `operations_executed`, and invariant fields.

**Rust unit tests (in `record_write_result.rs`):** 9 tests covering `disabled()` / `blocked()` constructors and invariant fields.

**Rust command tests (in `commands/restore.rs`):** 10 tests covering blocked-status input, zero-table input, ready input, disabled result, no-token, and invariant fields.

**Frontend tests (`src/test/recordWriteEngine.test.tsx`):** 26 tests covering mock service contract (status invariants, safety invariants, op counts), IPC fallback safety, and isolation from other write gates.

---

## Related Documents

- [Schema Write Engine Foundation](./schema-write-engine-foundation.md)
- [Restore Write Engine Skeleton](./restore-write-engine-skeleton.md)
- [Restore Record Import Planning](./restore-record-import-planning.md)
- [Tauri Command Inventory](./tauri-command-inventory.md)
- [Feature Status](../product/feature-status.md)
- [Known Limitations](../release/known-limitations.md)
- [Record Write Engine Foundation — Security and Privacy QA](../qa/security-privacy-qa.md)
