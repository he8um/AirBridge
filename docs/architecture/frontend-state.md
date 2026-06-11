# Frontend State Architecture

## Overview

AirBridge manages its frontend state with React's built-in `useState` hook. There is no external state management library. The full state shape is defined in a single TypeScript interface, populated from a fixture during development, and derived through pure selector functions. This document explains each piece of that system.

---

## AppState Shape

`AppState` is defined in `src/state/appState.ts`. It is a flat record of arrays and selected IDs. There is no nesting of state slices or reducer structure.

```
connections          AirtableConnectionProfile[]
workspaces           AirtableWorkspace[]
bases                AirtableBaseSummary[]
backupPackages       BackupPackageSummary[]
backupJobs           BackupJobSummary[]
restorePlans         RestorePlanSummary[]
restoreJobs          RestoreJobSummary[]
reports              ReportSummary[]
logs                 JobLogEntry[]
compatibilityRules   FieldCompatibilityRule[]
selectedConnectionId string | null
selectedBaseId       string | null
```

The two `selected*` fields hold the IDs of whatever the user has focused in the UI. Derived data (the full object for the selected connection, filtered lists, counts) is always computed on the fly by selectors rather than stored redundantly.

---

## Selectors Pattern

Selectors live in `src/state/selectors.ts`. Each selector is a plain function with the signature:

```
(state: AppState, ...args) => DerivedValue
```

Selectors are pure: given the same state, they always return the same value. They do not mutate state and do not produce side effects. This makes them trivial to test in isolation.

Current selectors:

| Function | Returns |
|---|---|
| `getConnectedProfiles` | Connections with status `"connected"` |
| `getRecentBackupPackages` | Packages sorted newest-first, up to a limit |
| `getRecentReports` | Reports sorted newest-first, up to a limit |
| `getActiveJobs` | Backup and restore jobs with status `"running"` or `"queued"`, projected to `JobSummary` |
| `getPermissionSummary` | Counts of passed/failed/unknown permissions across all connections |
| `getCompatibilitySummary` | Count of compatibility rules grouped by restore support level |
| `getLogsByLevel` | Log entries filtered to a specific level |
| `getDashboardStats` | Aggregated counts for the dashboard header |

When a new derived value is needed by the UI, a new selector function is added here. No logic is computed inside components directly.

---

## useAppState Hook

`src/state/useAppState.ts` is the single entry point for state in the application. Components import and call `useAppState()` to receive both the raw state and pre-computed selector results.

```typescript
const {
  state,
  setState,
  connectedProfiles,
  recentBackups,
  recentReports,
  activeJobs,
  permissionSummary,
  compatibilitySummary,
  dashboardStats,
  getLogsByLevel,
} = useAppState();
```

The hook runs all selectors on every render. Because selectors are pure functions with no subscriptions or caching, this is straightforward and sufficient for the current data size.

`setState` is exposed for future use when writes are implemented (marking a job as cancelled, storing a new backup package result, etc.). At that point, specific action functions will be added to the hook return value rather than exposing `setState` directly to components.

---

## mockState Module

`src/state/mockState.ts` exports a single constant `MOCK_STATE` of type `AppState`. It contains a complete, self-consistent set of synthetic fixtures: two connections, one workspace, two bases, three backup packages, one backup job, one restore plan, one restore job, three reports, six log entries, and eight compatibility rules.

The mock state exists for three reasons:

1. **Development without a backend.** All UI screens can be built and iterated against meaningful data without a running Tauri backend or a real Airtable token.
2. **Consistent baseline for testing.** Selectors and any future hooks can be tested against `MOCK_STATE` as a known input.
3. **Onboarding.** A new contributor can `npm run dev` and immediately see the application in a realistic, populated state.

All data in `MOCK_STATE` is fabricated. See the synthetic fixture policy in `docs/architecture/domain-model.md`.

---

## Replacing the Mock with Real Data

The current initialization in `useAppState` is:

```typescript
const [state, setState] = useState<AppState>(MOCK_STATE);
```

When the Tauri backend is ready, the replacement path is:

1. Create `src/services/tauriAirBridgeService.ts` implementing the same interface as `mockAirBridgeService` but using Tauri's `invoke` to call Rust commands.
2. Update `useAppState` to call the service layer on mount using `useEffect`, replacing `MOCK_STATE` with the data fetched from the backend:

```typescript
const [state, setState] = useState<AppState>(EMPTY_STATE);

useEffect(() => {
  Promise.all([
    tauriAirBridgeService.listConnections(),
    tauriAirBridgeService.listBases(),
    // ...
  ]).then(([connections, bases, ...]) => {
    setState(prev => ({ ...prev, connections, bases, ... }));
  });
}, []);
```

3. Once the backend is stable, `MOCK_STATE` and `mockAirBridgeService` can be removed.

The domain types, selectors, and hook interface remain unchanged through this migration. Only the initialization path inside `useAppState` changes.

---

## Why No External State Management Library

The current scope of AirBridge does not require an external library. The reasons:

- **Single page of state.** `AppState` is a flat object. There are no deeply nested trees, no normalized entity stores, no cross-cutting subscriptions.
- **Read-heavy.** Most of the UI reads from state. Write operations (starting a job, marking a connection as connected) are infrequent and discrete.
- **No concurrent updates.** AirBridge is a single-user desktop application. There is no need to handle optimistic updates, conflict resolution, or real-time synchronization.
- **Selectors are sufficient.** Pure functions on state handle all derived data. There is no need for memoized selectors or reactive dependencies beyond what React provides.

If the scope expands significantly (for example, real-time job progress updates streaming from the Rust backend, or multiple concurrent backup operations), a more structured approach can be evaluated at that point. The current architecture does not prevent that evolution: the `AppState` interface, the selectors, and the hook boundary are all defined and can be extended incrementally.
