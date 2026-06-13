# Restore Write Engine Skeleton

## Overview

The restore write engine skeleton defines the future execution architecture for restore write
operations — schema creation, record import, linked record updates, and attachment handling.
In the current version, all write engine execution is **permanently disabled by product policy**.
This document describes the skeleton as it exists now: an architecturally complete pipeline that
produces disabled-status previews, not an enabled write system.

## Constraints

All constraints below are hard-coded and enforced structurally:

- No Airtable write calls.
- No base, table, field, or record creation.
- No linked record updates. No attachment upload or download.
- No token field in the write engine request — Airtable access is not required.
- No token persistence anywhere in the pipeline.
- No success status — `RestoreWriteEngineStatus::Succeeded` is not defined.
- No success message in the UI.
- No enabled execute button — the UI shows only a disabled notice and a skeleton preview.
- `no_changes_made: true` is structurally enforced in every result struct and can never be false.
- The single gate function `evaluate_write_gate()` returns `Disabled/DisabledByProductPolicy` and
  has no enabled branch.

## Pipeline Phases

The write engine defines six phases. In this version, all six are disabled:

| Phase | Description |
|---|---|
| `ValidateInputs` | Checks that package filename and planning counts are provided. |
| `SchemaCreation` | Would create tables and fields. Currently skipped — schema plan counts are read-only. |
| `RecordCreation` | Would import records in batched passes. Currently skipped. |
| `LinkedRecordUpdates` | Would update linked record fields after all records exist. Currently skipped. |
| `AttachmentHandling` | Would handle attachments per the configured policy. Currently skipped — policy is MetadataOnly. |
| `FinalValidation` | Would validate the completed write operation. Currently skipped. |

## Rust Modules

| Module | Path | Role |
|---|---|---|
| `write_result` | `src/restore/write_result.rs` | Core result types and enums |
| `write_safety` | `src/restore/write_safety.rs` | Safety report: always no-op |
| `write_gate` | `src/restore/write_gate.rs` | Single gate function, always disabled |
| `schema_write_skeleton` | `src/restore/schema_write_skeleton.rs` | Schema phase summary from plan counts |
| `record_write_skeleton` | `src/restore/record_write_skeleton.rs` | Record/linked/attachment phase summaries |
| `write_engine` | `src/restore/write_engine.rs` | Main orchestration — calls gate, builds summaries |

## Tauri Command

**Command name:** `preview_restore_write_engine`

**Request fields** (no token):
- `package_filename` — filename only; used for result labeling.
- `package_path` — full path; used only to derive filename via `Path::file_name()`, never echoed.
- `schema_table_count`, `schema_direct_field_count`, `schema_deferred_field_count`,
  `schema_manual_action_count`, `schema_unsupported_count` — optional; from the existing schema plan.
- `estimated_first_pass_batches`, `estimated_second_pass_batches`,
  `linked_record_update_count` — optional; from the existing record import plan.

**Result fields:**
- `filename` — filename only, never the full path.
- `status` — always `"disabled"` or `"blocked"`.
- `disabled_reason`, `message`, `phase_summaries` (all 6 phases), `events`, `no_changes_made`.

## Write Gate

`evaluate_write_gate()` in `write_gate.rs` is the single source of truth for write gate status.
It has exactly one outcome: `RestoreWriteGateDecision { status: Disabled, reason: DisabledByProductPolicy }`.
There is no enabled branch, no feature-flag check, and no conditional that could produce a non-disabled
result. This is enforced by the type system — `RestoreWriteEngineStatus::Succeeded` does not exist.

## Safety Report

`build_write_safety_report()` in `write_safety.rs` always returns:

```
RestoreWriteSafetyReport {
    writes_enabled: false,
    network_writes_attempted: false,
    token_required: false,
    no_changes_made: true,
    restore_success_possible: false,
}
```

This report is attached to every write engine result and guarantees that no write operations
occurred, regardless of inputs.

## TypeScript Types

Defined in `src/backend/types.ts`:

- `RestoreWriteEngineStatus` — `"disabled" | "blocked" | "notStarted"` (no `"succeeded"`)
- `RestoreWritePhase` — all 6 phase names
- `RestoreWriteDisabledReason` — 5 reason variants
- `RestoreWriteEvent`, `RestoreWritePhaseSummary`, `RestoreWriteEngineRequest`,
  `RestoreWriteEngineResult`

`RestoreWriteEngineRequest` has no `token` field by design.

## Service Layer

| Layer | File | Behavior |
|---|---|---|
| Command bridge | `backend/commands.ts` | `previewRestoreWriteEngine()` → `safeInvoke`; returns null if Tauri unavailable |
| Live service | `services/liveAirBridgeService.ts` | Calls command; converts null to disabled fallback (no token, no path) |
| Mock service | `services/mockAirBridgeService.ts` | Deterministic disabled result; all 6 phases; no token; no full path |
| Service interface | `services/airBridgeService.ts` | `previewRestoreWriteEngine()` method |

## UI

`RestoreWriteEnginePanel` (`src/features/backups/RestoreWriteEnginePanel.tsx`) renders:

1. **Always-visible disabled notice** — "Restore write execution is not enabled in this version."
2. **No-changes notice** (when a result is present) — "No Airtable changes were made."
3. **Phase summary list** (when a result is present) — one row per phase, all disabled.

The panel has **no execute button**, **no token input**, and **no success message path**.

The panel is included in `RestorePage.tsx` and is activated when the schema plan is ready — it
calls `previewRestoreWriteEngine()` using counts from the existing schema plan.

## What Is Not in This Version

- No Airtable write operations.
- No `Succeeded` status anywhere in the type system.
- No token in the write engine request or result.
- No full path in any result.
- No UI success path.
- No enabled execute button.
- No restore plan execution — the pipeline produces previews only.
