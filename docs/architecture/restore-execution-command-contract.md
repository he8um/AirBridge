# Restore Execution Command Contract

## Purpose

This document describes the safety gate contract for restore execution. The contract defines how AirBridge validates that all preconditions are met before any restore operation could occur, and explains why the current implementation returns a gated result without performing any Airtable writes.

## Design Principles

- **No Airtable writes in this version.** The write engine is explicitly disabled. No base, table, or record is created, modified, or deleted.
- **No token persistence.** The personal access token is accepted as input, forwarded to the gate check for presence validation, and discarded. It is never stored, logged, or echoed in any result.
- **No full path in results.** The `RestoreExecutionResult` exposes only the filename portion of the package path. The full absolute path is never included in any result struct, log output, or UI element.
- **No `succeeded` status.** `RestoreExecutionStatus` has three variants: `Blocked`, `ReadyButDisabled`, and `Failed`. There is no `Succeeded` variant. The gate cannot produce a success outcome because the write engine is not enabled.

## Rust Model

### `RestoreExecutionStatus`

```rust
pub enum RestoreExecutionStatus {
    Blocked,
    ReadyButDisabled,
    Failed,
}
```

`ReadyButDisabled` is returned when all seven gate checks pass, but the write engine is not enabled. This distinguishes "prerequisites not met" from "prerequisites met but execution not available yet."

### `RestoreExecutionBlockReason`

```rust
pub enum RestoreExecutionBlockReason {
    MissingPackageInspection,
    InvalidPackage,
    MissingDryRunPlan,
    DryRunBlocked,
    MissingTargetMode,
    MissingToken,
    MissingConfirmation,
    RestoreWriteEngineNotEnabled,
}
```

### `RestoreExecutionRequest`

The request struct intentionally does **not** derive `Serialize`. It can only flow inward (from IPC to Rust), never outward. This prevents the token from being accidentally serialized into any response.

```rust
pub struct RestoreExecutionRequest {
    pub package_filename: String,
    pub package_path: String,           // full path used for gate check; never in result
    pub package_validation_status: String,
    pub dry_run_status: String,
    pub target_mode: RestoreTargetMode,
    pub target_base_name: Option<String>,
    pub token: String,                  // checked for presence only; never stored
    pub confirmation: String,
}
```

### `RestoreExecutionResult`

```rust
pub struct RestoreExecutionResult {
    pub filename: String,               // file_name() only, never full path
    pub status: RestoreExecutionStatus,
    pub block_reason: Option<RestoreExecutionBlockReason>,
    pub message: String,
    pub warnings: Vec<RestoreExecutionWarning>,
    pub errors: Vec<RestoreExecutionError>,
    pub no_changes_made: bool,          // always true
}
```

`no_changes_made` is always `true`. Both the `blocked()` helper and the all-gates-pass path set this field. The invariant holds regardless of which gate fires.

## Gate Validation Order

The gate function in `restore/execution_gate.rs` applies checks in this fixed order:

| # | Check | Block reason |
|---|-------|-------------|
| 1 | `package_filename` non-empty | `MissingPackageInspection` |
| 2 | `package_validation_status` is `valid` or `warning` | `InvalidPackage` |
| 3 | `dry_run_status` non-empty | `MissingDryRunPlan` |
| 4 | `dry_run_status` is `ready` or `readyWithWarnings` | `DryRunBlocked` |
| 5 | `package_path` non-empty | `MissingTargetMode` |
| 6 | `token` non-empty | `MissingToken` |
| 7 | `confirmation` equals `"RESTORE BACKUP"` exactly | `MissingConfirmation` |
| — | All pass → write engine disabled | `RestoreWriteEngineNotEnabled` + `ReadyButDisabled` |

The early-return pattern means the first failing check determines the result. Checks 1–7 never reach Airtable; neither does the all-pass path.

## Tauri Command

```rust
#[tauri::command]
pub fn run_restore_execution(request: RestoreExecutionRequest) -> RestoreExecutionResult {
    validate_restore_execution_gate(&request)
}
```

Registered in `lib.rs` invoke handler. Takes a `RestoreExecutionRequest` (inbound only), returns a `RestoreExecutionResult` (serialized to JSON via IPC). The token in the request is consumed and not forwarded.

## TypeScript Layer

### Types (`backend/types.ts`)

```typescript
export type RestoreExecutionStatus = "blocked" | "readyButDisabled" | "failed";

export interface RestoreExecutionResult {
  filename: string;
  status: RestoreExecutionStatus;
  blockReason?: RestoreExecutionBlockReason;
  message: string;
  warnings: RestoreExecutionWarning[];
  errors: RestoreExecutionError[];
  noChangesMade: boolean;
}
```

### IPC bridge (`backend/commands.ts`)

```typescript
export async function runRestoreExecution(
  request: RestoreExecutionRequest,
): Promise<RestoreExecutionResult | null> {
  return safeInvoke<RestoreExecutionResult>("run_restore_execution", { request });
}
```

`safeInvoke` returns `null` when Tauri IPC is unavailable (test environment, web context). The service layer converts `null` into a blocked result.

### Service fallback (`liveAirBridgeService.ts`)

When the IPC call returns `null`, the service returns a blocked result with `noChangesMade: true` and an `IPC_UNAVAILABLE` error code. The UI renders this identically to any other blocked result.

## UI Gate (`RestoreExecutionGatePanel`)

The panel enforces the same prerequisites visually before the button becomes enabled:

1. Package inspected and valid (`inspectedFilename` + `inspectionStatus` in `["valid", "warning"]`)
2. Restore plan preview ready (`dryRunStatus` in `["ready", "readyWithWarnings"]`)
3. Target mode selected (always present via `RestoreTargetMode`)
4. Token provided (non-empty password field)
5. Confirmation text equals `"RESTORE BACKUP"` exactly

All five must be true for `canAttempt` to be `true`. The button is `disabled` otherwise.

After any attempt (success or cancel):
- Token state is cleared from React state
- The `<input type="password">` ref value is cleared directly
- Token is not visible in rendered output at any point

The result panel always shows "No Airtable changes were made." regardless of status. When `readyButDisabled`, an additional notice explains that restore execution is not enabled in this version.

## Confirmation Phrase

```
RESTORE BACKUP
```

The phrase is defined as `RESTORE_CONFIRMATION_PHRASE` in Rust and re-exported as `RESTORE_EXECUTION_CONFIRMATION_TEXT` and `MOCK_RESTORE_CONFIRMATION` in TypeScript. All three constants hold the same value. Tests that verify the confirmation gate use these constants rather than inline strings.

## What This Is Not

- This is not a restore execution engine. No records are written.
- This is not a dry-run. The dry-run planner is a separate subsystem (`restore/dry_run.rs`).
- This is not a token validator. The token is checked for presence only; its validity is not verified against Airtable in this step.

## Related Documents

- [Restore Dry-Run Planning](restore-dry-run-planning.md)
- [Restore Engine](restore-engine.md)
- [Safe Backup Command Contract](safe-backup-command-contract.md)
- [Security Architecture](security-architecture.md)
- [Tauri Command Boundary](tauri-command-boundary.md)
