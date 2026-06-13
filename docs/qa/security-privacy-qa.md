# Security and Privacy QA

AirBridge is designed to operate entirely on the local machine. No user data, API tokens, or record content is transmitted to any server outside the device. This document describes the verification steps to confirm that these properties hold in a release build.

---

## No Network Egress During Backup/Restore

**Goal:** Confirm that AirBridge does not make any outbound network connection to third-party servers (outside of the Airtable API, which is the intentional data source/destination).

### Verification Steps

- [ ] Install a local network monitor (e.g., Little Snitch on macOS, Windows Firewall logging, or `ss`/`tcpdump` on Linux).
- [ ] Launch AirBridge with network monitoring active.
- [ ] Perform a full backup of a test base.
- [ ] Perform a dry-run restore from the backup.
- [ ] Review all outbound connections made by the AirBridge process.
- [ ] **Expected:** The only outbound connections are to `api.airtable.com` (or Airtable's documented API hostname). No connections to analytics endpoints, crash-reporting services, update servers (during operation, as opposed to explicit update checks), or any other third-party hostname.
- [ ] **Expected:** When no operation is in progress (idle state), no periodic outbound connections are made.

---

## Token Storage Security

**Goal:** Confirm that API tokens are stored only in the local app data directory and are never transmitted, logged, or embedded in backup packages.

### Verification Steps

- [ ] Add a connection with a known test token.
- [ ] Locate the app data directory (see `cross-platform-qa.md` for paths).
- [ ] Inspect the stored configuration file. Verify:
  - [ ] The token is stored in a local file, not in a browser localStorage or a cloud-synced location.
  - [ ] The token is not stored in plaintext alongside other application logs. (If it is encrypted, document the encryption method in the technical documentation.)
- [ ] Perform a backup. After completion, open every file in the backup package.
  - [ ] The token must not appear in `manifest.json`, `schema.json`, `records.jsonl`, or any other file in the package.
- [ ] Review the application log file after a backup.
  - [ ] The token string must not appear anywhere in the log.
- [ ] Simulate a failed backup (revoke the token mid-backup). Review the error messages shown in the UI and written to the log.
  - [ ] The token must not appear in the error message text.

---

## No Telemetry or Analytics

**Goal:** Confirm that the production build contains no telemetry, analytics, crash-reporting, or user-tracking code.

### Verification Steps

- [ ] Inspect the compiled frontend bundle (`dist/` or equivalent) for known analytics domain strings: `analytics`, `telemetry`, `sentry`, `datadog`, `mixpanel`, `amplitude`, `segment`, `gtag`, `ga.js`.
  - [ ] None of these strings should appear as URL fragments in the bundle.
- [ ] Inspect `Cargo.lock` and `package.json` for known telemetry crate or package names.
- [ ] Run the application with network monitoring active through a complete backup-and-restore cycle.
  - [ ] No connections to domains other than `api.airtable.com` should be observed.
- [ ] Review `tauri.conf.json` (or equivalent) for any `allowlist` entries that permit connections to non-Airtable hosts.

---

## Log File Content

**Goal:** Confirm that the log file contains operational information only, with no tokens, record field values, or personally identifiable information.

### Verification Steps

- [ ] Perform a backup of a test base containing known field values (e.g., a record with `email` field `test-check@example.com`).
- [ ] Open the application log file.
- [ ] Search the log for the token string used — **must not be present**.
- [ ] Search the log for the test email address or other known record field values — **must not be present**.
- [ ] Confirm that the log contains useful operational entries: operation start/stop timestamps, table names, field names, record counts, and error messages.
- [ ] Confirm that the log does not contain full record payloads or raw API responses that include record field values.
- [ ] Verify that the log file's file permissions restrict read access to the current user (e.g., mode `0600` on macOS/Linux).

---

## File Permissions on Output Packages

- [ ] On macOS/Linux: backup package files are created with mode `0644` (owner read/write, group and others read-only) or more restrictive. The containing directory is `0755` or more restrictive.
- [ ] On Windows: backup files are created in the user's home or chosen directory with standard user-owned permissions. They are not world-readable to other Windows users on the same machine.
- [ ] The application does not write backup files to system-wide directories (e.g., `/tmp`, `/var`, `C:\Windows\Temp`) without user consent.

---

## Input Validation (Backup File Tampering)

**Goal:** Confirm that a maliciously crafted backup file cannot cause the application to behave unsafely.

### Verification Steps

- [ ] Open a backup package whose `manifest.json` contains a very long string value (>10,000 characters) in `baseName`. Verify: the application handles this gracefully (truncates the display, shows an error, or ignores the oversized value) rather than crashing or hanging.
- [ ] Open a backup package whose `records.jsonl` contains a line that is valid JSON but has unexpected top-level keys. Verify: the application parses what it recognizes and ignores unknown keys, without crashing.
- [ ] Open a backup package whose `schema.json` references a `linkedTableId` that does not exist in the same schema. Verify: the application reports a validation warning, not a panic.
- [ ] Provide a file that claims to be a backup package but is actually a binary file (e.g., a PNG renamed to `.airbridge`). Verify: the application detects the invalid format and shows a clear error message.

---

## Redaction Policy Enforcement

If AirBridge supports user-configured field redaction:

- [ ] Configure a field (e.g., an email field) as redacted before performing a backup.
- [ ] After the backup, inspect `records.jsonl` and confirm that the redacted field's values are replaced with `null` or a placeholder, not their actual values.
- [ ] Confirm that the redaction is noted in `manifest.json` (e.g., a `redactedFields` list).
- [ ] Confirm that running a restore from a redacted backup correctly restores `null` for the redacted field, and does not attempt to fill in the original values.

---

## No Credentials in Build Artifacts or Logs

- [ ] The release binary does not contain any hardcoded API keys, tokens, or credentials. Verify by running `strings airbridge-binary | grep -i "Bearer\|patPersonal\|apikey"` — no matches.
- [ ] `tauri.conf.json` and `package.json` in the release artifact contain no embedded secrets.
- [ ] The CI build logs (if publicly visible) do not print the signing certificate password, token values, or any other secrets. Confirm that all sensitive environment variables are masked.
- [ ] The application's built-in default configuration does not reference any specific Airtable base ID, token scope, or user account.

---

## Summary Verification Table

| Property | Verification method | Expected result |
|---------|---------------------|----------------|
| No egress to non-Airtable hosts | Network monitor during full backup+restore | Only `api.airtable.com` connections seen |
| Token not in backup package | grep token string across all package files | Zero matches |
| Token not in log | grep token string in log file | Zero matches |
| No telemetry code in bundle | grep known analytics domains in frontend bundle | Zero matches |
| Record values not in log | grep known field value in log file | Zero matches |
| Backup files have restricted permissions | `ls -l` on output files | `0644` or more restrictive |
| Malformed input does not crash | Open crafted malformed package | Error message shown, no crash |


---

## Safe Backup Command Contract (V0.1)

**Goal:** Confirm that the `run_backup_job` command enforces safety requirements before writing any file.

### Confirmation Requirement

- [ ] The command refuses to run without the exact confirmation phrase `"CREATE BACKUP"`.
- [ ] Passing an empty confirmation returns a `CONFIRMATION_REQUIRED` safety error.
- [ ] Passing a partial or mis-cased phrase (e.g., `"create backup"`, `"yes"`) is rejected.
- [ ] No file is written when confirmation is rejected.

### Output Path Validation

- [ ] A path with the wrong extension (e.g., `.zip`) is rejected with `WRONG_EXTENSION`.
- [ ] An empty path is rejected with `EMPTY_PATH`.
- [ ] A path whose parent directory does not exist is rejected with `PARENT_NOT_FOUND`.
- [ ] A path containing `..` components is rejected with `TRAVERSAL_DETECTED`.
- [ ] A path that is an existing directory is rejected with `IS_DIRECTORY`.
- [ ] Validation itself creates no files (no side effects from calling `validate_backup_output_path`).

### Token Safety in Command Response

- [ ] The `run_backup_job` response does not contain the token string passed in the request.
- [ ] No event emitted by the orchestrator contains the token.
- [ ] The error message on a 401 response does not expose the token value.

### Output Path Safety in Response

- [ ] The command response does not include the full absolute output path.
- [ ] Only the filename portion (`packageFilename`) is returned.
- [ ] No absolute path components (`/Users/`, `/home/`) appear in the serialised response.

### No Attachment URLs in Package or Response

- [ ] The command result does not contain attachment URLs (`https://`, `dl.airtable.com`).
- [ ] Package entries contain no attachment URLs.

---

## Backup File Picker and Confirmation Flow (V1)

**FP-01: Token field is a password input.**
- The token field in `BackupExecutionPanel` renders with `type="password"`.
- Characters are masked. The token value is not exposed as plain text in the DOM.

**FP-02: Token cleared after run.**
- After `runBackupJob` resolves (success or failure), `clearSensitiveState()` sets the token state to an empty string.
- The token does not persist in component state after the run.

**FP-03: Token not in run response or result card.**
- `RunBackupCommandResponse` contains no token field.
- `BackupJobResultCard` renders only job status, filename, summary data, and errors — never the token.

**FP-04: Full output path not rendered.**
- `BackupExecutionPanel` displays only the filename component (via `getDisplayFileName`).
- The path validation status uses `redactOutputPath` which renders `…/filename.airbridge`.
- The result card renders only `packageFilename` (filename-only, from the Rust response).

**FP-05: No token persistence.**
- Token is held only in `useState`. It is never written to `localStorage`, `sessionStorage`, or any persistent store.

**FP-06: File picker returns path only — no file write.**
- `pickBackupOutputPath()` calls the Tauri dialog `save()` function.
- The `save()` function returns a path string; it does not write any file.
- No file is written until `runBackupJob` executes with valid confirmation.

**FP-07: jsdom test isolation.**
- All tests that involve `BackupExecutionPanel` mock `pickBackupOutputPath` via `vi.mock`.
- The Tauri dialog plugin is never invoked in tests.
- No file is written during tests.

---

## Restore Dry-Run Plan Safety (V0.1)

**Goal:** Confirm that the restore dry-run planning flow makes no Airtable API calls, requires no token, and never exposes the full package path.

**DR-01: No token requested.**
- The `create_restore_dry_run_plan` Tauri command has no `token` parameter.
- The `RestoreDryRunPanel` component does not render a token input field.
- No token flows through the dry-run code path.

**DR-02: No Airtable API calls.**
- The dry-run planner reads only from the local `.airbridge` package.
- No HTTP client code is called during plan generation.
- Network monitoring during a dry-run operation shows zero connections to `api.airtable.com`.

**DR-03: Full path not in result.**
- The `RestoreDryRunPlan` result contains `filename` (basename only), not `path`.
- `Path::file_name()` is used in Rust to strip directory components before the result is returned.
- The serialized JSON result does not contain `/Users/`, `/home/`, or `:\\`.

**DR-04: No files extracted from package.**
- The planner uses `BackupPackageReader` in-memory only.
- No package entry is written to the filesystem during plan generation.

**DR-05: `noChangesMade` always true.**
- Every code path in `create_dry_run_plan` and `blocked_plan` sets `no_changes_made: true`.
- The unit tests in `commands/restore.rs` assert this property explicitly.
- The UI always shows "No Airtable changes were made." when a plan result is rendered.

**DR-06: No restore execution button.**
- `RestoreDryRunPanel` does not render a "Start Restore", "Execute", or "Run Restore" button.
- The frontend tests assert the absence of any such button by scanning rendered button text.

---

## Restore Execution Gate Safety (V0.1)

**Goal:** Confirm that the restore execution gate makes no Airtable API calls, never stores the token, and never exposes the full package path or a success status.

**RX-01: Token is a password input and is cleared.**
- The token input in `RestoreExecutionGatePanel` renders with `type="password"`.
- Characters are masked; the token value is not visible as DOM text.
- After any attempt (or cancel), the token state and the input ref value are both cleared to empty string.
- The token is never written to `localStorage`, `sessionStorage`, or any persistent store.

**RX-02: Token not echoed in result.**
- `RestoreExecutionRequest` does not derive `Serialize` in Rust — the struct cannot be included in any response.
- `RestoreExecutionResult` has no `token` field.
- The serialized JSON result from the Tauri command does not contain the token string.
- The frontend tests assert that `JSON.stringify(result)` does not contain the test token value.

**RX-03: Full path not in result or UI.**
- `RestoreExecutionResult.filename` is populated with `Path::file_name()` — directory components are stripped.
- The serialized result does not contain `/Users/`, `/home/`, or `:\\`.
- The rendered DOM does not contain any absolute path component after an attempt.

**RX-04: No Airtable API calls.**
- `validate_restore_execution_gate` contains no HTTP client calls.
- The write engine is explicitly disabled; no Airtable API is reachable from this code path.
- Network monitoring during an attempt shows zero connections to `api.airtable.com`.

**RX-05: `noChangesMade` always true.**
- Both the `blocked()` helper and the all-gates-pass path set `no_changes_made: true`.
- Rust unit tests cover both code paths.
- The result panel always shows "No Airtable changes were made." when rendered.

**RX-06: No `succeeded` status.**
- `RestoreExecutionStatus` has no `Succeeded` variant in Rust.
- The serialized JSON result never contains `"succeeded"` as a status value.
- The rendered DOM does not contain the text "restore complete", "restore succeeded", or equivalent.

**RX-07: Gate enforces seven ordered checks.**
- The gate returns `Blocked` for each of the seven conditions (see `restore-execution-command-contract.md`).
- All-gates-pass returns `ReadyButDisabled` with `restoreWriteEngineNotEnabled` — not a success.

**RX-08: UI button requires exact confirmation.**
- The "Attempt Restore" button is `disabled` unless `confirmationText === "RESTORE BACKUP"` exactly.
- Lowercase, partial, or empty confirmation text does not enable the button.

---

## Restore Schema Creation Plan Safety (V0.1)

**Goal:** Confirm that the schema creation planning flow makes no Airtable API calls, requires no token, creates no Airtable tables or fields, and always returns `noChangesMade: true`.

**SC-01: No token requested.**
- The `create_restore_schema_plan` Tauri command has no `token` parameter.
- The `RestoreSchemaPlanPanel` component does not render a token input field.
- No token flows through the schema plan code path.

**SC-02: No Airtable API calls.**
- The schema creation planner reads only from in-memory input (dry-run result tables).
- No HTTP client is constructed or called during schema plan generation.
- Network monitoring during a schema plan operation shows zero connections to `api.airtable.com`.

**SC-03: No Airtable tables or fields created.**
- The planner produces a read-only ordered plan. It makes no writes to any Airtable base.
- There is no write engine code path reachable from `create_restore_schema_plan`.

**SC-04: Full path not in result.**
- `RestoreSchemaPlan.filename` is populated with `Path::file_name()` — directory components are stripped.
- The serialized result does not contain `/Users/`, `/home/`, or `:\\`.

**SC-05: `noChangesMade` always true.**
- Every code path in `create_schema_creation_plan` sets `no_changes_made: true`.
- Rust unit tests assert this property.
- The UI always shows "No Airtable changes were made." when a plan result is rendered.

**SC-06: No execute button.**
- `RestoreSchemaPlanPanel` does not render a "Create Tables", "Execute Schema", or similar button.
- The panel renders only plan inspection content and a "Preview Schema Creation Plan" button.

---

## Restore Record Import Plan Safety (V0.1)

**Goal:** Confirm that the record import planning flow makes no Airtable API calls, requires no token, creates no Airtable records, never resolves actual record IDs, and always returns `noChangesMade: true`.

**RI-01: No token requested.**
- The `create_restore_record_import_plan` Tauri command has no `token` parameter.
- `RestoreRecordImportPlanRequest` has no `token` field in its Rust struct definition.
- The `RestoreRecordImportPlanPanel` component does not render a token input field.
- No token flows through the record import plan code path.

**RI-02: No Airtable API calls.**
- The record import planner reads only from in-memory input (package filename, dry-run status, schema plan status, table metadata).
- No HTTP client is constructed or called during import plan generation.
- Network monitoring during a record import plan operation shows zero connections to `api.airtable.com`.

**RI-03: No Airtable records created.**
- The planner produces a read-only batch plan. It makes no writes to any Airtable base.
- There is no write engine code path reachable from `create_restore_record_import_plan`.

**RI-04: Old-to-new record ID mapping is planning-only.**
- The import plan describes the `MapSourceRecordIdToCreatedRecordId` strategy.
- No actual new record IDs are present in the plan — IDs are only available after first-pass execution.
- The plan contains no Airtable record ID values (`rec…`).

**RI-05: Full path not in result.**
- `RestoreRecordImportPlan.filename` uses the basename from the request — no directory path components.
- The serialized result does not contain `/Users/`, `/home/`, or `:\\`.

**RI-06: `noChangesMade` always true.**
- Every code path in `create_record_import_plan` and `blocked_plan` sets `no_changes_made: true`.
- Rust unit tests in `commands/restore.rs` assert this property for both ready and blocked paths.
- The UI always shows "No Airtable records were created or modified." when a plan result is rendered.

**RI-07: No execute button.**
- `RestoreRecordImportPlanPanel` does not render an "Import Records", "Execute Import", or similar button.
- The panel renders only plan inspection content and a "Preview Record Import Plan" button.

**RI-08: Gates require dry-run and schema plan readiness.**
- The command returns a `Blocked` plan if `dry_run_status` is not `"ready"` or `"readyWithWarnings"` (`DRY_RUN_BLOCKED`).
- The command returns a `Blocked` plan if `schema_plan_status` is not `"ready"` or `"readyWithWarnings"` (`SCHEMA_PLAN_BLOCKED`).
- The command returns a `Blocked` plan if `tables` is empty (`NO_TABLES`).
- All three gate paths are covered by Rust unit tests.

---

## Local Job History Safety (V0.1)

**Goal:** Confirm that the local job history stores and returns only safe summaries — no tokens, no full paths, no record payloads, and no attachment URLs.

**JH-01: No token in history items.**
- `JobHistoryItem` and `JobHistorySummary` have no token fields.
- `list_job_history` response does not contain any token-like string.
- The `JobHistoryPanel` component does not render any token-like value in the DOM.
- Rust unit tests assert the serialized result does not contain `"Bearer "` or `"patXXX"`.

**JH-02: No full paths in history items.**
- `JobHistorySummary.package_filename` is populated with `redact_path_to_filename()` — directory components are stripped before storage.
- The serialized `list_job_history` response does not contain `/Users/`, `/home/`, or `:\\`.
- The `JobHistoryPanel` renders only the filename component; Rust and frontend tests assert this.

**JH-03: No record payloads in history items.**
- History items contain only summary-level metadata: title, kind, status, filename, base name, warning/error counts, validation status, and timestamps.
- No field values, record IDs, or raw API response content is stored or returned.

**JH-04: No attachment URLs in history items.**
- History items do not include attachment download URLs.
- `sanitize_history_message()` in `redaction.rs` detects and redacts `dl.airtable.com` and `v5.airtableusercontent.com` URLs.
- Unit tests cover attachment URL redaction.

**JH-05: Memory-only storage — no credential persistence.**
- `InMemoryJobHistoryStore` does not write to disk, `localStorage`, or any persistent store.
- `clear_job_history` returns 0 in V0.1 — there is no disk store to clear.
- History is discarded on application restart.

**JH-06: `noChangesMade` correctly set for planning/inspection operations.**
- All history items generated by `from_inspection`, `from_dry_run_plan`, `from_schema_plan`, `from_record_import_plan`, and `from_restore_execution_blocked` set `no_changes_made: true`.
- Rust unit tests in `commands/history.rs` assert this property across all planning item kinds.

---

## Write Engine Skeleton Security Properties

**Goal:** Confirm that the write engine skeleton makes no Airtable API calls, requires no token,
never echoes the full package path, and never produces a succeeded status.

**WE-01: No token in write engine request.**
- `RestoreWriteEngineRequest` has no `token` field.
- TypeScript interface and Rust struct both omit `token`.
- Frontend tests confirm that the request keys do not include `"token"`.

**WE-02: No Airtable API calls in write engine.**
- `preview_write_engine()` calls only: `evaluate_write_gate()`, `build_schema_write_skeleton()`,
  `build_record_write_skeleton()`, `build_write_safety_report()`. None of these make network calls.
- No `reqwest` client, no `AirtableClient`, and no HTTP request is constructed.
- Rust unit tests assert no Airtable client is called.

**WE-03: Full path never in result.**
- `preview_write_engine()` derives the filename via `Path::file_name()` and includes only that.
- The `package_path` field is used only for filename derivation; it is never echoed.
- Rust tests assert the result contains no absolute path components.
- Frontend tests assert no `/Users/`, `/tmp/`, or `/backups/` in the rendered panel.

**WE-04: No succeeded status.**
- `RestoreWriteEngineStatus` has three variants: `Disabled`, `Blocked`, `NotStarted`.
- There is no `Succeeded` variant — it was intentionally omitted from the type definition.
- Rust tests assert the result status is never `"succeeded"`.
- Frontend tests assert `"succeeded"` does not appear in any result or rendered output.

**WE-05: `noChangesMade` always true.**
- `RestoreWriteEngineResult.no_changes_made` is set to `true` at every code path in `preview_write_engine`.
- The `SchemaWriteSkeletonPlan`, `RecordWriteSkeletonPlan`, and `RestoreWriteSafetyReport` structs
  all have `no_changes_made: true` as a hard-coded field value.
- Rust and frontend tests assert this property.

**WE-06: Write gate always disabled.**
- `evaluate_write_gate()` has exactly one outcome: `Disabled/DisabledByProductPolicy`.
- There is no enabled branch, no feature flag, and no conditional that could return a non-disabled result.
- 8 Rust unit tests assert always-disabled, always-product-policy, never-not-started behavior.

**WE-07: UI has no execute button and no token input.**
- `RestoreWriteEnginePanel` renders no `<button>` and no `<input>` elements.
- Frontend tests assert button count = 0 and input count = 0 in the rendered panel.

**WE-08: IPC fallback is safe.**
- When Tauri IPC is unavailable, `liveAirBridgeService.previewRestoreWriteEngine()` returns a
  disabled fallback with `noChangesMade: true`, empty phase summaries, and no full path.
- Frontend tests assert the fallback result does not contain `"succeeded"`.
