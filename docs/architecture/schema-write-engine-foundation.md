# Schema Write Engine Foundation

This document describes the schema write engine foundation — the request plan builder, the dry-run executor skeleton, and the Tauri command that surfaces them. These components are implemented behind a hard-disabled product gate: no Airtable schema writes are made at any point.

---

## Overview

The schema write engine foundation adds the structural layer between the existing schema creation plan (which identifies what would need to be done) and a future live write engine (which would do it). It introduces:

- A **request plan builder** that turns a `RestoreSchemaPlan` into a sequenced list of write operations.
- A **dry-run executor skeleton** that always consults the write gate and always returns disabled.
- A **Tauri command** (`preview_schema_write_request_plan`) that exposes the plan to the frontend without requiring a token, making any Airtable calls, or creating any schema changes.

---

## Components

### `restore/schema_write_requests.rs`

Defines the types and builder for schema write request plans.

#### Types

```
SchemaWriteOperationKind
  CreateBase | CreateTable | CreateField | DeferLinkedField | ManualAction

SchemaWriteOperationStatus
  Planned | Blocked | Disabled
  (no Succeeded — intentionally absent)

SchemaWriteOperation
  index, kind, status, source_table_id, table_name
  source_field_id?, field_name?, field_type?, linked_source_table_id?
  note, no_changes_made

SchemaWriteBlockedReason
  DisabledByProductPolicy | SchemaPlanNotReady | NoTablesInPlan

SchemaWriteRequestPlan
  filename, status, blocked_reason?, operations[]
  table_op_count, field_op_count, deferred_op_count
  manual_action_count, total_op_count
  warnings[], no_changes_made, network_writes_attempted
```

`no_changes_made` is always `true`. `network_writes_attempted` is always `false`.

#### Builder: `build_schema_write_request_plan`

Takes a `&RestoreSchemaPlan` and returns a `SchemaWriteRequestPlan`.

**Phase ordering (invariant):**

| Phase | Kind | Source |
|-------|------|--------|
| 1 | `CreateTable` | One per `table_steps` entry |
| 2 | `CreateField` | `field_steps` entries where `classify_field_for_schema()` returns `CreateDirectly` or `CreateWithAdjustment` |
| 3 | `DeferLinkedField` | `deferred_steps` entries |
| 4 | `ManualAction` | `manual_action_fields` entries |

This ordering enforces the structural dependency: tables must exist before fields can be created. Deferred linked fields depend on both source and target tables existing. Manual-action fields are listed last as advisory items.

**Status logic:**

- If `schema_plan.status == Blocked` → returns `Blocked` with `SchemaPlanNotReady`.
- If `table_steps.is_empty()` → returns `Blocked` with `NoTablesInPlan`.
- Otherwise → returns `Disabled` with `blocked_reason = Some(DisabledByProductPolicy)`.

No `Planned` status is returned in the current version — the product gate always prevents advancement past `Disabled`.

---

### `restore/schema_write_executor.rs`

Executor skeleton that always consults the write gate and returns disabled. No Airtable calls are made.

#### Type: `SchemaWriteDryRunResult`

```
filename, status (RestoreWriteEngineStatus), disabled_reason (RestoreWriteDisabledReason)
message, operations_planned, operations_executed (always 0)
table_ops_planned, field_ops_planned, deferred_ops_planned, manual_action_count
warnings[], no_changes_made (always true), network_writes_attempted (always false)
```

#### Function: `execute_schema_write_dry_run`

1. Always calls `evaluate_write_gate()` first.
2. If `request_plan.status == Blocked`, surfaces `BlockedByInvalidPlan`.
3. Otherwise returns `Disabled` with the gate's static message.
4. `operations_executed` is always `0`.
5. `no_changes_made` is always `true`.
6. `network_writes_attempted` is always `false`.

---

### `restore/schema_write_result.rs`

Request and result types for the Tauri command boundary.

#### `SchemaWriteRequestPlanRequest` (no token field)

```
package_filename, schema_plan_status, table_count
direct_field_count, deferred_field_count, manual_action_count
```

The request carries only count fields derived from the schema plan summary. No token, no path, no field-level data.

#### `SchemaWriteRequestPlanResult`

```
filename, status, blocked_reason?, disabled_reason?, message
table_op_count, field_op_count, deferred_op_count
manual_action_count, total_op_count
warnings[], no_changes_made, network_writes_attempted
```

No `Succeeded` status value. `no_changes_made` is always `true`. `network_writes_attempted` is always `false`.

---

### `commands/restore.rs` — `preview_schema_write_request_plan`

Tauri command registered as `"preview_schema_write_request_plan"`.

**Inputs:** `SchemaWriteRequestPlanRequest` (no token).

**Logic:**

1. Returns `Blocked(SchemaPlanNotReady)` if `schema_plan_status == "blocked"`.
2. Returns `Blocked(NoTablesInPlan)` if `table_count == 0`.
3. Synthesizes a minimal `RestoreSchemaPlan` from the count fields.
4. Calls `build_schema_write_request_plan` → `execute_schema_write_dry_run`.
5. Always returns disabled via `evaluate_write_gate()` message.

The synthesis step avoids requiring full field data from the frontend: it builds placeholder `RestoreFieldCreationStep` entries from the count values so that the builder can produce accurate operation counts.

---

## TypeScript Layer

### `backend/types.ts`

```typescript
type SchemaWriteOperationKind = "createBase" | "createTable" | "createField" | "deferLinkedField" | "manualAction"
type SchemaWriteOperationStatus = "planned" | "blocked" | "disabled"
type SchemaWriteBlockedReason = "disabledByProductPolicy" | "schemaPlanNotReady" | "noTablesInPlan"
interface SchemaWriteOperation { ... }
interface SchemaWriteRequestPlan { ... }
interface SchemaWriteRequestPlanRequest { ... }   // no token field
interface SchemaWriteRequestPlanResult { ... }    // no token field, no "succeeded" status
```

### `backend/commands.ts`

```typescript
export async function previewSchemaWriteRequestPlan(
  request: SchemaWriteRequestPlanRequest,
): Promise<SchemaWriteRequestPlanResult | null>
```

Returns `null` if Tauri IPC is unavailable (jsdom / browser context). The live service converts `null` to a safe disabled fallback.

### `services/airBridgeService.ts`

```typescript
previewSchemaWriteRequestPlan(request: SchemaWriteRequestPlanRequest): Promise<SchemaWriteRequestPlanResult>
```

### `services/liveAirBridgeService.ts`

Delegates to `commands.previewSchemaWriteRequestPlan`. If `null` is returned, returns a safe fallback:

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

`previewSchemaWriteRequestPlanImpl`:

- Returns `blocked/schemaPlanNotReady` if `schemaPlanStatus === "blocked"`.
- Returns `blocked/noTablesInPlan` if `tableCount === 0`.
- Otherwise returns `disabled` with op counts derived from request fields.
- `noChangesMade: true` and `networkWritesAttempted: false` always.

---

## Safety Invariants

| Property | Enforcement |
|----------|-------------|
| No token in request | `SchemaWriteRequestPlanRequest` has no `token` field |
| No token in result | `SchemaWriteRequestPlanResult` has no `token` field |
| No Airtable API calls | `execute_schema_write_dry_run` calls only the write gate; no HTTP client constructed |
| No Airtable schema created | No `create_table` / `create_field` call exists in any reachable code path |
| `noChangesMade` always true | Hard-coded in all types; Rust and frontend tests assert this |
| `networkWritesAttempted` always false | Hard-coded in all types; Rust and frontend tests assert this |
| No `Succeeded` status | `SchemaWriteOperationStatus` has `Planned`, `Blocked`, `Disabled` only |
| Write gate always disabled | `evaluate_write_gate()` returns `Disabled/DisabledByProductPolicy` unconditionally |

---

## Testing

**Rust unit tests (in `schema_write_requests.rs`):** 38 tests covering phase ordering, operation counts, blocked/disabled status, `no_changes_made`, `network_writes_attempted`, and the ordering invariant.

**Rust unit tests (in `schema_write_executor.rs`):** 12 tests covering the disabled path, blocked-plan path, always-zero `operations_executed`, and invariant fields.

**Rust unit tests (in `schema_write_result.rs`):** 5 tests covering `disabled()` / `blocked()` constructors.

**Rust command tests (in `commands/restore.rs`):** 10 tests covering blocked-status input, zero-table input, ready input, disabled result, no-token, and invariant fields.

**Frontend tests (`src/test/schemaWriteEngine.test.tsx`):** 18 tests covering mock service contract, IPC fallback safety, and isolation from the restore write gate.

---

## Related Documents

- [Restore Write Engine Skeleton](./restore-write-engine-skeleton.md)
- [Restore Schema Creation Planning](./restore-schema-creation-planning.md)
- [Tauri Command Inventory](./tauri-command-inventory.md)
- [Feature Status](../product/feature-status.md)
- [Known Limitations](../release/known-limitations.md)
- [Schema Write Engine Foundation — Security and Privacy QA](../qa/security-privacy-qa.md)
