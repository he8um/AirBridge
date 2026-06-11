# Domain Model Architecture

## Overview

AirBridge organizes its TypeScript code into a domain layer that is deliberately separate from UI components, state management, and services. This separation keeps the type system stable and predictable as the application grows.

---

## Why a Separate Domain Layer

UI components change frequently: layout shifts, new screens are added, presentation concerns evolve. If type definitions live inside component files or are scattered across the codebase, even small UI refactors can break type references throughout.

By placing all domain types in `src/domain/`, the rest of the codebase imports from a single, stable location. A component can be deleted or rewritten without touching the types that describe the data it displayed. A service can be replaced without breaking anything that depends on the shape of a backup package or connection profile.

This also makes it straightforward to reason about the data model independently of how it is rendered or fetched.

---

## Domain Modules

### `connection.ts`

Describes the state of an Airtable token connection. A `ConnectionProfile` tracks whether the connection is active, when it was last verified, and the result of each individual permission check (schema read, record read, schema write, record write). The permission check model allows the UI to show granular feedback rather than a binary connected/disconnected state.

### `airtable.ts`

Represents the structural hierarchy of Airtable itself: workspaces, bases, tables, and fields. These types mirror the Airtable data model as AirBridge understands it. `AirtableFieldType` is an exhaustive union covering every known field type, which is essential for the compatibility layer to make accurate restore decisions.

### `backup.ts`

Covers two distinct concepts:

- **BackupPackageSummary**: A completed, persisted backup artifact. It has a file path, a size, a record count, and a status. It is the output of a backup operation.
- **BackupJobSummary**: An in-progress or completed execution that produced (or attempted to produce) a package. Jobs track progress and errors during execution.

### `restore.ts`

Mirrors the backup model for the restore direction:

- **RestorePlanSummary**: A validated plan that maps a backup package to a target base. Plans carry compatibility warnings before any data is written.
- **RestoreJobSummary**: An execution of a plan, which may be a dry run or a live restore. Dry runs complete without writing data and produce a report of what would have been skipped.

### `report.ts`

A structured document produced at the end of a backup, restore, validation, or compatibility check. Reports contain a list of `ReportItem` entries with severity levels (info, warning, error), field and table context, and human-readable descriptions. Reports are persisted alongside packages and can be reviewed at any time.

### `log.ts`

Low-level, timestamped log entries emitted during job execution. Log entries have a level (debug, info, warning, error) and an optional reference to the job that produced them. Logs are ephemeral in the current implementation and are not persisted to disk separately from reports.

### `compatibility.ts`

Encodes the rules for how each Airtable field type behaves during backup and restore. A `FieldCompatibilityRule` states, for a given field type, whether it is fully restorable, partially restorable, metadata-only, or unsupported. This is the single source of truth for compatibility decisions and is used by both the restore plan validator and the compatibility report generator.

### `job.ts`

A lightweight, normalized job shape used by the UI to display running operations. `JobSummary` is a projection of either a `BackupJobSummary` or `RestoreJobSummary` into a common format. This allows a single "active jobs" panel to display any running operation without branching on job type.

---

## Mock State Approach

During the current development phase, `src/state/mockState.ts` provides a complete, realistic in-memory fixture of type `AppState`. All data in the fixture is synthetic: no real tokens, no real workspace IDs, no real email addresses, no real file paths.

The mock state serves several purposes:

- UI components can be built and styled against realistic data without a running backend.
- Selectors and hooks can be unit-tested against a known, stable fixture.
- Onboarding new contributors is faster because the app starts in a meaningful state immediately.

---

## Future Replacement Path

The current stack is:

```
React UI → useAppState (useState + MOCK_STATE) → mockAirBridgeService → MOCK_STATE
```

The target stack is:

```
React UI → useAppState (useState + Tauri commands) → Rust backend → Airtable API
```

The replacement path is:

1. Implement Tauri commands in Rust that return JSON matching the domain types defined here.
2. Replace `mockAirBridgeService` with a `tauriAirBridgeService` that calls `invoke()` for each operation.
3. Update `useAppState` to initialize from the service layer rather than from `MOCK_STATE` directly.
4. The domain types and selectors remain unchanged throughout this migration.

No domain types need to change because the Rust backend is responsible for serializing its data into shapes that match the existing TypeScript interfaces.

---

## Synthetic Fixture Policy

All test and development data in this codebase must be fabricated. Specifically:

- Email addresses must use `example.com` domains.
- Workspace and base IDs must use synthetic prefixes (e.g., `wsExample...`, `appExample...`).
- No real Airtable personal access tokens or API keys may appear anywhere in the source tree.
- No real customer, user, or company names may appear in fixture data.
- File paths in fixture data must use generic paths (e.g., `/Users/example/airbridge/...`).

This policy protects against accidental credential exposure and ensures the codebase can be shared without redaction.

---

## Services Layer

`src/services/` abstracts data access behind a consistent interface. In the current phase, `mockAirBridgeService` returns data directly from the in-memory fixture. In production, a `tauriAirBridgeService` will call into the Rust backend via Tauri's `invoke` mechanism.

Components and hooks must not import from `mockState` directly. They should go through the service layer or through `useAppState`, which ensures the data access point is easy to swap.
