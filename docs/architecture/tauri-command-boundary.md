# Tauri Command Boundary

## Purpose

The Tauri command boundary is the typed, auditable interface that separates the React
frontend from the Rust backend. All communication between the two layers passes through
this boundary via named commands. The frontend never calls Airtable APIs directly; it
always delegates to the Rust side through a command.

## Why the Boundary Exists

Separating frontend and backend through a named command layer provides several guarantees:

- **Type safety** — Every command has a defined input and output shape expressed in
  TypeScript (`src/backend/types.ts`) and mirrored in Rust. Mismatches are caught at
  compile time on both sides.
- **Auditability** — The full list of commands is enumerated in one place. Reviewers can
  see every operation the frontend is allowed to request.
- **Security isolation** — Sensitive values such as tokens are passed inward toward Rust
  and are never echoed back in responses. The frontend cannot read memory it did not
  explicitly request.
- **Testability** — The `safeInvoke` wrapper returns `null` when Tauri IPC is unavailable,
  allowing the command bridge to be imported and exercised in jsdom test environments
  without a running Tauri instance.

## Current Commands

| Command name               | TypeScript function        | Description                                   |
| -------------------------- | -------------------------- | --------------------------------------------- |
| `get_app_health`           | `getAppHealth`             | Returns application name, version, and status |
| `check_connection`         | `checkConnection`          | Validates a token and checks API permissions  |
| `list_workspaces`          | `listWorkspaces`           | Returns available workspaces                  |
| `list_bases`               | `listBases`                | Returns base summaries for connected workspaces |
| `list_backup_packages`     | `listBackupPackages`       | Returns known backup packages                 |
| `list_restore_plans`       | `listRestorePlans`         | Returns restore plans derived from packages   |
| `list_reports`             | `listReports`              | Returns backup and restore reports            |
| `list_logs`                | `listLogs`                 | Returns job log entries                       |
| `list_compatibility_rules` | `listCompatibilityRules`   | Returns field-level compatibility rules       |

## Current Implementation Status

`check_connection` is live — the Rust handler calls the Airtable list-bases endpoint
via `ReqwestHttpTransport` and returns a typed `ConnectionCheckResult`. Write permissions
are marked `Unknown` (not verified destructively). All other command handlers remain stubs
backed by mock state.

The `liveAirBridgeService` wires the frontend to real Tauri commands. `ConnectionForm`
accepts a `service` prop (defaults to `liveAirBridgeService`) so tests inject
`mockAirBridgeService` without touching the Tauri IPC.

## Future Path

Remaining stub handlers will be replaced by real Airtable API calls as each workflow
is implemented. The TypeScript bridge requires no changes — the typed function signatures
and the `safeInvoke` pattern remain the same. The mock service continues to serve as a
development and testing fixture independent of the Rust implementation.

## Security Expectations

### Token handling

Tokens passed to `check_connection` are forwarded to the Rust command and are not stored,
cached, logged, or included in any response. The TypeScript bridge does not retain a
reference to the token value after the `invoke` call returns.

### Typed error codes

Command failures return an `AirBridgeCommandError` with a `code` field (e.g.
`PERMISSION_DENIED`, `NETWORK_TIMEOUT`, `NOT_FOUND`). The frontend uses
`isAirBridgeCommandError` and `formatCommandError` from `src/backend/errors.ts` to handle
failures consistently without inspecting raw error strings.

### Destructive operations

Commands that write, delete, or mutate data must not be implemented on either side until
an explicit confirmation flow exists in the UI. Destructive operations require a separate
confirmation command; a single command must not perform both the check and the mutation.

### Minimal response data

Command responses include only the data strictly required by the frontend view. No
sensitive values (tokens, internal IDs that expose implementation details, raw error
stack traces) are included in responses.

## TypeScript Bridge

**File:** `src/backend/commands.ts`

The bridge exports one async function per command. Each function calls `safeInvoke<T>`,
which:

1. Dynamically imports `invoke` from `@tauri-apps/api/core` at call time (not module
   load time), so the module can be imported safely in jsdom.
2. Calls the named Tauri command with any provided arguments.
3. Returns `null` if the import or the IPC call throws for any reason (no Tauri runtime,
   command not registered, IPC timeout).

The return type of every exported function is `T | null`. Callers must handle the `null`
case, which is the expected value in all non-Tauri environments.

## Testing

Command bridge tests live in `src/test/commands.test.ts` and run under Vitest with the
jsdom environment. Because there is no Tauri runtime in jsdom, every command returns
`null`. Tests assert this behavior and also exercise the error utility functions
(`isAirBridgeCommandError`, `formatCommandError`) with both valid and invalid inputs.

The mock service tests in `src/test/airBridgeService.test.ts` verify that
`mockAirBridgeService` satisfies the `AirBridgeService` interface and that each method
returns the expected synthetic data from `MOCK_STATE`.
