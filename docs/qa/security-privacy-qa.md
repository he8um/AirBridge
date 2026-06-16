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

## Restore Sandbox Verification Safety (Gate 1)

**Goal:** Confirm that the sandbox verification command makes no Airtable API calls, requires no token, makes no writes of any kind, and always returns `noChangesMade: true` and `writesEnabled: false`.

**SV-01: No token accepted.**
- `SandboxVerificationRequest` has no `token` field.
- The `verify_restore_sandbox_environment` Tauri command has no token parameter.
- `RestoreSandboxVerificationPanel` renders no token input field.
- The Rust unit test `result_serialization_has_no_token` confirms no token key in the serialized result.

**SV-02: No Airtable API calls.**
- All 10 checks are local-only (target mode, write gate state, request fields).
- No HTTP client is constructed or called.
- CHK-10 is always `Skipped` — the live metadata check is deferred to a future release.
- Network monitoring shows zero connections to `api.airtable.com` during verification.

**SV-03: No write operations.**
- No files are written. No Airtable records, tables, or fields are created.
- `evaluate_write_gate()` is called read-only to confirm the gate returns `Disabled`.
- `writesEnabled` is always `false` in the result.
- `networkWritesAttempted` is always `false` in the result.

**SV-04: No full path in result.**
- `SandboxVerificationRequest` has no `path` field.
- `SandboxVerificationResult` has no field containing a filesystem path.
- The Rust unit test `result_serialization_has_no_full_path` confirms no absolute path in the serialized result.

**SV-05: `noChangesMade` always true.**
- Every code path in `verify_sandbox_environment` sets `no_changes_made: true`.
- 22 Rust unit tests assert this property.
- 25 frontend tests assert this property via mock service contract tests.

**SV-06: No execute button.**
- `RestoreSandboxVerificationPanel` does not render an "Execute", "Start Restore", or "Run" button.
- The frontend test `no execute button anywhere` confirms this by scanning all rendered buttons.

---

## Restore Confirmation Gate Safety (Gate 2)

**Goal:** Confirm that the confirmation gate makes no Airtable API calls, requires no token, makes no writes, and always returns `noChangesMade: true` and `writesEnabled: false` — even when confirmed.

**CF-01: No token accepted.**
- `RestoreConfirmationRequest` has no `token` field.
- `validate_restore_confirmation_gate` has no token parameter.
- `RestoreConfirmationPanel` renders no token input (no `type="password"` input).
- The Rust test `result_serialization_has_no_token` confirms no `"token"` key in serialized result.

**CF-02: No Airtable API calls.**
- All 5 checks are local (write gate state, string comparison, sandbox status string).
- No HTTP client is constructed or called.
- Network monitoring shows zero connections to `api.airtable.com` during validation.

**CF-03: No write operations.**
- No files written. No Airtable records, tables, or fields created.
- `writesEnabled` is always `false`. `networkWritesAttempted` is always `false`.
- `Confirmed` status does NOT enable restore writes — it only records that the text was correct.

**CF-04: No full path in result.**
- `RestoreConfirmationRequest` has no filesystem path field.
- `RestoreConfirmationResult` has no field containing a path.
- `requiredText` is sanitized: path separators and non-alphanumeric characters are stripped.
- The Rust test `result_serialization_has_no_full_path` confirms no absolute path in result.

**CF-05: `noChangesMade` always true.**
- Every code path in `validate_restore_confirmation` sets `no_changes_made: true`.
- 30 Rust unit tests assert this property across all status variants.
- 39 frontend tests assert this property via mock service contract tests.

**CF-06: No execute button.**
- `RestoreConfirmationPanel` does not render "Execute", "Run Restore", or "Start Restore".
- The frontend test `no execute button anywhere` confirms this by scanning all rendered buttons.

**CF-07: Blocked sandbox propagates.**
- If `sandboxVerificationStatus` is `"blocked"`, confirmation result is `"blocked"` regardless of entered text.
- This prevents Gate 2 from being satisfied while Gate 1 is blocked.

---

## Restore Target Empty Verification Safety (Gate 3)

**Goal:** Confirm that the target empty verification gate makes no Airtable write API calls, requires no token, makes no writes, and always returns `noChangesMade: true` and `writesEnabled: false` — even when verified.

**TEV-01: No token accepted.**
- `TargetEmptyVerificationRequest` has no `token` field.
- `verify_restore_target_empty` has no token parameter.
- `RestoreTargetEmptyVerificationPanel` renders no token input.

**TEV-02: No Airtable write API calls.**
- All 5 checks are local (mode string, count values, write gate state).
- No HTTP client is constructed or called in the write path.

**TEV-03: No write operations.**
- No files written. No Airtable records, tables, or fields created.
- `writesEnabled` is always `false`. `networkWritesAttempted` is always `false`.
- `Verified` status does NOT enable restore writes.

**TEV-04: No full path in result.**
- `TargetEmptyVerificationRequest` has no filesystem path field.
- `TargetEmptyVerificationResult` has no field containing a path.
- The Rust test `result_serialization_has_no_full_path` confirms no absolute path in result.

**TEV-05: `noChangesMade` always true.**
- Every code path in `verify_target_empty` sets `no_changes_made: true`.
- 29 Rust unit tests assert this property.
- 47 frontend tests assert this property via mock service contract and panel tests.

**TEV-06: No execute button.**
- `RestoreTargetEmptyVerificationPanel` does not render "Execute", "Run Restore", or "Start Restore".
- The frontend test `no execute button in panel` confirms this.

**TEV-07: Unsupported target mode is blocked.**
- Only `"newBase"` and `"emptyExistingBase"` are allowed. Any other string returns `blocked`.
- TEV-02 check fails for unknown modes; overall status becomes `blocked`.

---

## Restore Destructive Operation Policy Safety (Gate 4)

**Goal:** Confirm that the destructive operation policy gate makes no Airtable API calls, requires no token, makes no writes, and always returns `noChangesMade: true` and `writesEnabled: false`.

**DOP-01: No token accepted.**
- The `verify_destructive_operation_policy_gate` Tauri command has no `token` parameter.
- The `RestoreDestructiveOperationPolicyPanel` component does not render a token input field.
- No token flows through the destructive operation policy code path.

**DOP-02: No Airtable API calls.**
- `verify_destructive_operation_policy` accepts no HTTP client or base client argument.
- No HTTP calls are made by this command.
- All checks run purely against the declared operations list.

**DOP-03: No write operations.**
- No Airtable record, table, field, or base is created, updated, or deleted.
- `networkWritesAttempted` is always `false` in the result.

**DOP-04: No full path in result.**
- `DestructiveOperationPolicyResult` has no path field.
- `DestructiveOperationPolicyRequest` has no path field.
- Serialized result does not contain `/Users/`, `/home/`, or `:\\`.

**DOP-05: `noChangesMade` always true.**
- All code paths in `verify_destructive_operation_policy` set `no_changes_made: true`.
- Rust unit tests assert this for every status branch.

**DOP-06: No execute button.**
- `RestoreDestructiveOperationPolicyPanel` does not render any button with execute, run, or restore semantics.
- Panel tests assert no button with text matching `/execute/i` or `/run restore/i` is present.

**DOP-07: Compliant status does not enable writes.**
- `writesEnabled` is always `false` regardless of policy status.
- `Compliant` is a policy check result, not an execution gate — it does not change the write engine state.

---

## Restore Attachment Upload Policy Safety (Gate 5)

**Goal:** Confirm that the attachment upload policy gate makes no Airtable API calls, requires no token, makes no writes, never uploads attachment file bytes, and always returns `noChangesMade: true` and `writesEnabled: false`.

**AUP-01: No token accepted.**
- The `verify_attachment_upload_policy_gate` Tauri command has no `token` parameter.
- The `RestoreAttachmentUploadPolicyPanel` component does not render a token input field.
- No token flows through the attachment upload policy code path.

**AUP-02: No Airtable API calls.**
- `verify_attachment_upload_policy` accepts no HTTP client or base client argument.
- No HTTP calls are made by this command.
- All checks run purely against the declared attachment fields list.

**AUP-03: No write operations.**
- No Airtable record, table, field, or base is created, updated, or deleted.
- `networkWritesAttempted` is always `false` in the result.

**AUP-04: No full attachment URL in any result field.**
- `dl.airtable.com` and `airtableusercontent.com` never appear in any serialized result field.
- Rust serialization test asserts this for every output field.

**AUP-05: No full path in result.**
- `AttachmentUploadPolicyResult` has no path field.
- `AttachmentUploadPolicyRequest` has no path field.
- Serialized result does not contain `/Users/`, `/home/`, or `:\\`.

**AUP-06: `noChangesMade` always true.**
- All code paths in `verify_attachment_upload_policy` set `no_changes_made: true`.
- Rust unit tests assert this for every status branch.

**AUP-07: No execute button.**
- `RestoreAttachmentUploadPolicyPanel` does not render any button with execute, run, or restore semantics.
- Panel tests assert no button with text matching `/execute/i` or `/run restore/i` is present.

**AUP-08: Compliant status does not enable writes.**
- `writesEnabled` is always `false` regardless of policy status.
- `Compliant` is a policy check result, not an execution gate — it does not change the write engine state.

**AUP-09: Attachment file bytes are never uploaded.**
- `UploadRequested` intent is blocked (status `Blocked`), not permitted.
- `DownloadRequested` intent produces a warning only — no download is attempted.
- No attachment binary data flows through any code path in this gate.

---

## Restore Schema Record Order Policy Safety (Gate 6)

**Goal:** Confirm that the schema record order policy gate makes no Airtable API calls, requires no token, makes no writes, contains no record payload, and always returns `noChangesMade: true` and `writesEnabled: false`.

**SRO-01: No token accepted.**
- The `verify_schema_record_order_policy_gate` Tauri command has no `token` parameter.
- The `RestoreSchemaRecordOrderPolicyPanel` component does not render a token input field.
- No token flows through the schema record order policy code path.

**SRO-02: No Airtable API calls.**
- `verify_schema_record_order_policy` accepts no HTTP client or base client argument.
- No HTTP calls are made by this command.
- All checks run purely against the declared phase list.

**SRO-03: No write operations.**
- No Airtable record, table, field, or base is created, updated, or deleted.
- `networkWritesAttempted` is always `false` in the result.

**SRO-04: No record payload in any result field.**
- `SchemaRecordOrderPolicyResult` contains no raw record data, no record IDs, and no field values.
- Rust serialization tests assert this for every output field.

**SRO-05: No full path in result.**
- `SchemaRecordOrderPolicyResult` has no path field.
- `SchemaRecordOrderPolicyRequest` has no path field.
- Serialized result does not contain `/Users/`, `/home/`, or `:\\`.

**SRO-06: `noChangesMade` always true.**
- All code paths in `verify_schema_record_order_policy` set `no_changes_made: true`.
- Rust unit tests assert this for every status branch.

**SRO-07: No execute button.**
- `RestoreSchemaRecordOrderPolicyPanel` does not render any button with execute, run, or restore semantics.
- Panel tests assert no button with text matching `/execute/i` or `/run restore/i` is present.

**SRO-08: Compliant status does not enable writes.**
- `writesEnabled` is always `false` regardless of policy status.
- `Compliant` is a phase ordering check result, not an execution gate — it does not change the write engine state.

**SRO-09: Schema must precede records in declared phase list.**
- Any request where a record-create phase appears before or without a schema phase causes `Blocked`.
- The `records-before-schema` or `missing-schema-with-records` ordering violation is included in the result.

---

## Restore Sandbox Write Testing Policy Safety (Gate 7)

**Goal:** Confirm that the sandbox write testing policy gate makes no Airtable API calls, requires no token, makes no writes, contains no record payload, and always returns `noChangesMade: true` and `writesEnabled: false`.

**SWT-01: No token accepted.**
- The `verify_sandbox_write_testing_policy_gate` Tauri command has no `token` parameter.
- The `RestoreSandboxWriteTestingPolicyPanel` component does not render a token input field.
- No token flows through the sandbox write testing policy code path.

**SWT-02: No Airtable API calls.**
- `verify_sandbox_write_testing_policy` accepts no HTTP client or base client argument.
- No HTTP calls are made by this command.
- All checks run purely against the declared request fields and evidence struct.

**SWT-03: No write operations.**
- No Airtable record, table, field, or base is created, updated, or deleted.
- `networkWritesAttempted` is always `false` in the result.

**SWT-04: No record payload in any result field.**
- `SandboxWriteTestingPolicyResult` contains no raw record data, no record IDs, and no field values.
- Rust serialization tests assert this for every output field.

**SWT-05: No full path in any field.**
- `SandboxWriteTestEvidence.testPackageFilename` accepts a basename only. Any value containing `/` or `\` is flagged as incomplete evidence (Warning status).
- `SandboxWriteTestingPolicyResult` has no path field.
- Serialized result does not contain `/Users/`, `/home/`, or `:\\`.

**SWT-06: `noChangesMade` always true.**
- All code paths in `verify_sandbox_write_testing_policy` set `no_changes_made: true`.
- Rust unit tests assert this for every status branch.

**SWT-07: No execute button.**
- `RestoreSandboxWriteTestingPolicyPanel` does not render any button with execute, run, or restore semantics.
- Panel tests assert no button with text matching `/execute/i` or `/run restore/i` is present.

**SWT-08: Compliant status does not enable writes.**
- `writesEnabled` is always `false` regardless of policy status.
- `Compliant` is a testing evidence check result, not an execution gate — it does not change the write engine state.

**SWT-09: Production/unknown target blocked.**
- Any request where `targetClassification` is `production` or `unknown` causes `Blocked`.
- `SWT-02` check row shows `failed` for these classifications.

---

## Restore Live Write Confirmation Policy Safety (Gate 8)

**Goal:** Confirm that the live write confirmation policy gate makes no Airtable API calls, requires no token, makes no writes, contains no record payload, never enables writes even when confirmed, and always returns `noChangesMade: true` and `writesEnabled: false`.

**LWC-SEC-01: No token accepted.**
- The `verify_live_write_confirmation_policy_gate` Tauri command has no `token` parameter.
- `LiveWriteConfirmationPolicyRequest` contains no token field.
- `LiveWriteConfirmationPolicyResult` contains no token field.
- `RestoreLiveWriteConfirmationPolicyPanel` does not render a token input (`type="password"` or `name="token"`).
- Rust serialization tests assert no `pat_` prefix or `"token"` key in any result JSON.

**LWC-SEC-02: No Airtable API calls.**
- `verify_live_write_confirmation_policy` accepts no HTTP client or base client argument.
- No HTTP calls are made by this command.
- All checks run against the declared request fields only.

**LWC-SEC-03: No write operations.**
- No Airtable record, table, field, or base is created, updated, or deleted.
- `networkWritesAttempted` is always `false` in all result branches.

**LWC-SEC-04: No record payload in any result field.**
- `LiveWriteConfirmationPolicyResult` contains no raw record data, no record IDs, and no field values.
- Rust serialization tests assert no `"fields"` or `"recordId"` key in the result JSON.

**LWC-SEC-05: No full path in any field.**
- `LiveWriteConfirmationPolicyResult` has no path field.
- Serialized result does not contain `/Users/`, `/home/`, or `:\\`.
- Target label is sanitised before appearing in the required phrase (path separators stripped).

**LWC-SEC-06: `noChangesMade` always true.**
- All code paths in `verify_live_write_confirmation_policy` set `no_changes_made: true`.
- Rust unit tests assert this for every status branch (confirmed, warning, blocked, rejected).

**LWC-SEC-07: No execute button.**
- `RestoreLiveWriteConfirmationPolicyPanel` does not render any button with execute, run, or restore semantics.
- Panel tests assert no "succeeded" language is present.

**LWC-SEC-08: Confirmed status does not enable writes.**
- `writesEnabled` is always `false` regardless of policy status.
- `Confirmed` validates the confirmation contract only — it does not change the write engine state.
- Rust unit tests assert `writesEnabled: false` in every result branch including `Confirmed`.

**LWC-SEC-09: Blocked prior gate prevents confirmation.**
- Any prior gate with `blocked` status causes `Blocked` policy status even if the text matches.
- Gate 7 (sandbox write testing) blocked status also causes `Blocked`.
- Rust unit tests cover every prior gate blocked scenario.

---

## Restore Rate-Limit and Backoff Policy Safety (Gate 9)

**Goal:** Confirm that the rate-limit and backoff policy gate makes no Airtable API calls, requires no token, makes no writes, contains no record payload, never enables writes even when compliant, and always returns `noChangesMade: true` and `writesEnabled: false`.

**RLB-SEC-01: No token accepted.**
- The `verify_rate_limit_backoff_policy_gate` Tauri command has no `token` parameter.
- `RateLimitBackoffPolicyRequest` contains no token field.
- `RateLimitBackoffPolicyResult` contains no token field.
- `RestoreRateLimitBackoffPolicyPanel` does not render a token input (`type="password"` or `name="token"`).
- Frontend serialization tests assert no `pat_` prefix or `"token"` key in any result JSON.

**RLB-SEC-02: No Airtable API calls.**
- `verify_rate_limit_backoff_policy` accepts no HTTP client or base client argument.
- No HTTP calls are made by this command.
- All checks run against the declared request fields only.

**RLB-SEC-03: No write operations.**
- No Airtable record, table, field, or base is created, updated, or deleted.
- `networkWritesAttempted` is always `false` in all result branches.

**RLB-SEC-04: No record payload in any result field.**
- `RateLimitBackoffPolicyResult` contains no raw record data, no record IDs, and no field values.
- The plan summary mirrors only numeric and boolean policy parameters.

**RLB-SEC-05: No full path in any field.**
- `RateLimitBackoffPolicyResult` has no path field.
- Serialized result does not contain `/Users/`, `/home/`, or `:\\`.
- Frontend tests assert this for every result branch.

**RLB-SEC-06: `noChangesMade` always true.**
- All code paths in `verify_rate_limit_backoff_policy` set `no_changes_made: true`.
- Rust unit tests assert this for every status branch (compliant, warning, blocked).

**RLB-SEC-07: No execute button.**
- `RestoreRateLimitBackoffPolicyPanel` does not render any button with execute, run, or restore semantics.
- Panel tests assert no "succeeded" language is present.

**RLB-SEC-08: Compliant status does not enable writes.**
- `writesEnabled` is always `false` regardless of policy status.
- `Compliant` validates the declared throttling plan only — it does not change the write engine state.
- Rust unit tests assert `writesEnabled: false` in every result branch including `Compliant`.

**RLB-SEC-09: No-plan causes immediate blocked (short-circuit).**
- When no `RateLimitBackoffPlan` is provided, the function returns after RLB-02 with 2 checks and `Blocked` status.
- No subsequent check fields are evaluated.
- Rust unit tests assert `checks.len() == 2` in the no-plan case.

---

## Restore Checkpoint Durability Policy Safety (Gate 10)

**Goal:** Confirm that the checkpoint durability policy gate makes no Airtable API calls, requires no token, makes no writes, contains no record payload, never enables writes even when compliant, and always returns `noChangesMade: true` and `writesEnabled: false`.

**CDP-SEC-01: No token accepted.**
- The `verify_checkpoint_durability_policy_gate` Tauri command has no `token` parameter.
- `CheckpointDurabilityPolicyRequest` contains no token field.
- `CheckpointDurabilityPolicyResult` contains no token field.
- `RestoreCheckpointDurabilityPolicyPanel` does not render a token input (`type="password"` or `name="token"`).
- Frontend serialization tests assert no `pat_` prefix or `"token"` key in any result JSON.

**CDP-SEC-02: No Airtable API calls.**
- `verify_checkpoint_durability_policy` accepts no HTTP client or base client argument.
- No HTTP calls are made by this command.
- All checks run against the declared request fields only.

**CDP-SEC-03: No write operations.**
- No Airtable record, table, field, or base is created, updated, or deleted.
- `networkWritesAttempted` is always `false` in all result branches.

**CDP-SEC-04: No record payload in any result field.**
- `CheckpointDurabilityPolicyResult` contains no raw record data, no record IDs, and no field values.
- The plan summary mirrors only boolean policy parameters and the declared durability backend string.

**CDP-SEC-05: No full path in any field.**
- `CheckpointDurabilityPolicyResult` has no path field.
- Serialized result does not contain `/Users/`, `/home/`, or `:\\`.
- Frontend tests assert this for every result branch.

**CDP-SEC-06: `noChangesMade` always true.**
- All code paths in `verify_checkpoint_durability_policy` set `no_changes_made: true`.
- Rust unit tests assert this for every status branch (compliant, warning, blocked).

**CDP-SEC-07: No execute button.**
- `RestoreCheckpointDurabilityPolicyPanel` does not render any button with execute, run, or restore semantics.
- Panel tests assert no "succeeded" language is present.

**CDP-SEC-08: Compliant status does not enable writes.**
- `writesEnabled` is always `false` regardless of policy status.
- `Compliant` validates the declared checkpoint plan only — it does not change the write engine state.
- Rust unit tests assert `writesEnabled: false` in every result branch including `Compliant`.

**CDP-SEC-09: No-plan causes immediate blocked (short-circuit).**
- When no `CheckpointDurabilityPlan` is provided, the function returns after CDP-02 with 2 checks and `Blocked` status.
- No subsequent check fields are evaluated.
- Rust unit tests assert `checks.len() == 2` in the no-plan case.

---

## Restore Final Validation Policy Safety (Gate 11)

**Goal:** Confirm that the final validation policy gate makes no Airtable API calls, requires no token, makes no writes, contains no record payload, never enables writes even when compliant, never introduces a restore success state, and always returns `noChangesMade: true` and `writesEnabled: false`.

**FVP-SEC-01: No token accepted.**
- The `verify_final_validation_policy_gate` Tauri command has no `token` parameter.
- `FinalValidationPolicyRequest` contains no token field.
- `FinalValidationPolicyResult` contains no token field.
- `RestoreFinalValidationPolicyPanel` does not render a token input (`type="password"` or `name="token"`).
- Frontend serialization tests assert no `pat_` prefix or `"token"` key in any result JSON.

**FVP-SEC-02: No Airtable API calls.**
- `verify_final_validation_policy` accepts no HTTP client or base client argument.
- No HTTP calls are made by this command.
- All checks run against the declared request fields only.

**FVP-SEC-03: No write operations.**
- No Airtable record, table, field, or base is created, updated, or deleted.
- `networkWritesAttempted` is always `false` in all result branches.

**FVP-SEC-04: No record payload in any result field.**
- `FinalValidationPolicyResult` contains no raw record data, no record IDs, and no field values.
- The plan summary mirrors only boolean policy parameters.

**FVP-SEC-05: No full path in any field.**
- `FinalValidationPolicyResult` has no path field.
- Serialized result does not contain `/Users/`, `/home/`, or `:\\`.
- Frontend tests assert this for every result branch.

**FVP-SEC-06: `noChangesMade` always true.**
- All code paths in `verify_final_validation_policy` set `no_changes_made: true`.
- Rust unit tests assert this for every status branch (compliant, warning, blocked).

**FVP-SEC-07: No execute button.**
- `RestoreFinalValidationPolicyPanel` does not render any button with execute, run, or restore semantics.
- Panel tests assert no "succeeded" language is present.

**FVP-SEC-08: Compliant status does not enable writes.**
- `writesEnabled` is always `false` regardless of policy status.
- `Compliant` validates the declared final validation plan only — it does not change the write engine state.
- Rust unit tests assert `writesEnabled: false` in every result branch including `Compliant`.

**FVP-SEC-09: Compliant status does not introduce a restore success state.**
- The compliant result message explicitly states writes remain disabled.
- No "succeeded" or "restore complete" language appears in any result branch.
- Frontend tests assert no "succeeded" text is visible in the panel for any status.

**FVP-SEC-10: No-plan causes immediate blocked (short-circuit).**
- When no `FinalValidationPlan` is provided, the function returns after FVP-02 with 2 checks and `Blocked` status.
- No subsequent check fields are evaluated.
- Rust unit tests assert `checks.len() == 2` in the no-plan case.

---

## Restore Write Phase Ordering Policy Safety (Gate 12)

**WPO-SEC-01: No token accepted.**
- The `verify_write_phase_ordering_policy_gate` Tauri command accepts `WritePhaseOrderingPolicyRequest` which has no `token` field.
- The `RestoreWritePhaseOrderingPolicyPanel` component renders no token input field.
- No token flows through the write phase ordering policy code path.

**WPO-SEC-02: No Airtable API calls.**
- The policy evaluator reads only from the declared phase list in memory.
- No HTTP client is constructed or called during policy verification.
- Network monitoring during a phase ordering policy check shows zero connections to `api.airtable.com`.

**WPO-SEC-03: No write operations.**
- The `verify_write_phase_ordering_policy` function performs no Airtable writes, no filesystem writes, and no database writes.
- `noChangesMade` is always `true`. `networkWritesAttempted` is always `false`.

**WPO-SEC-04: No record payload in any result field.**
- `WritePhaseOrderingPolicyResult` has no `fields`, `records`, `recordId`, or `payload` field.
- The per-phase summary entries contain only kind, status, canonical position, and optional skip reason — no record data.
- Rust unit tests assert the serialized JSON contains no `"fields"` or `"recordId"` keys.

**WPO-SEC-05: No full path in any field.**
- `WritePhaseOrderingPolicyRequest` has no filesystem path field.
- `WritePhaseOrderingPolicyResult` has no filesystem path field.
- Skip reasons in declared phases are not validated as paths — they are free-form human-readable strings.
- Rust unit tests assert the serialized JSON contains no `/Users/` or `/home/` patterns.

**WPO-SEC-06: `noChangesMade` always true.**
- All result branches (Compliant, Warning, Blocked) set `no_changes_made: true`.
- Rust unit tests assert this for every code path.

**WPO-SEC-07: No execute button.**
- The `RestoreWritePhaseOrderingPolicyPanel` renders only a "Verify write phase ordering policy" button.
- No "Execute restore", "Start restore", or "Run restore" button exists in this panel.
- Frontend tests assert all button labels do not match execute/start/run restore patterns.

**WPO-SEC-08: Compliant status does not enable writes.**
- A `Compliant` result from `verify_write_phase_ordering_policy` does not change `evaluate_write_gate()` behavior.
- `writesEnabled` is always `false` in `WritePhaseOrderingPolicyResult`.
- Rust unit tests assert `writesEnabled: false` in every result branch including `Compliant`.

**WPO-SEC-09: Compliant status does not introduce a restore success state.**
- The compliant result message explicitly states writes remain disabled.
- No "succeeded" or "restore complete" language appears in any result branch.
- Frontend tests assert no "succeeded" text is visible in the panel for any status.

**WPO-SEC-10: No-phase-list causes immediate blocked (short-circuit).**
- When no `phases` field is provided, the function returns after WPO-02 with 2 checks and `Blocked` status.
- No subsequent check fields are evaluated.
- Rust unit tests assert `checks.len() == 2` in the no-phases case.

---

## Restore Failure Modes Policy Safety (Gate 13)

**Goal:** Confirm that the failure modes policy gate makes no Airtable API calls, requires no token, makes no writes, contains no record payload, never enables writes even when compliant, never introduces a restore success state, and always returns `noChangesMade: true` and `writesEnabled: false`.

**FMP-SEC-01: No token accepted.**
- The `verify_failure_modes_policy_gate` Tauri command accepts `FailureModesPolicyRequest` which has no `token` field.
- The `RestoreFailureModesPolicyPanel` component renders no token input field.
- No token flows through the failure modes policy code path.
- Rust serialization tests assert no `"token"` or `"apiKey"` key in any result JSON.

**FMP-SEC-02: No Airtable API calls.**
- The policy evaluator reads only from the declared handling plans in memory.
- No HTTP client is constructed or called during policy verification.
- Network monitoring during a failure modes policy check shows zero connections to `api.airtable.com`.

**FMP-SEC-03: No write operations.**
- The `verify_failure_modes_policy` function performs no Airtable writes, no filesystem writes, and no database writes.
- `noChangesMade` is always `true`. `networkWritesAttempted` is always `false`.

**FMP-SEC-04: No record payload in any result field.**
- `FailureModesPolicyResult` has no `fields`, `records`, `recordId`, or `payload` field.
- The handling summary entries contain only mode name, stop behavior, and boolean flags — no record data.
- Rust unit tests assert the serialized JSON contains no `"fields"` or `"recordId"` keys.

**FMP-SEC-05: No full path in any field.**
- `FailureModesPolicyRequest` has no filesystem path field.
- `FailureModesPolicyResult` has no filesystem path field.
- Rust unit tests assert the serialized JSON contains no `/Users/` or `/home/` patterns.

**FMP-SEC-06: `noChangesMade` always true.**
- All result branches (Compliant, Warning, Blocked) set `no_changes_made: true`.
- Rust unit tests assert this for every code path.

**FMP-SEC-07: No execute button.**
- The `RestoreFailureModesPolicyPanel` renders only a "Verify failure modes policy" button.
- No "Execute restore", "Start restore", or "Run restore" button exists in this panel.
- Frontend tests assert all button labels do not match execute/start/run restore patterns.

**FMP-SEC-08: Compliant status does not enable writes.**
- A `Compliant` result from `verify_failure_modes_policy` does not change `evaluate_write_gate()` behavior.
- `writesEnabled` is always `false` in `FailureModesPolicyResult`.
- Rust unit tests assert `writesEnabled: false` in every result branch including `Compliant`.

**FMP-SEC-09: Compliant status does not introduce a restore success state.**
- The compliant result message explicitly states writes remain disabled.
- No "succeeded" or "restore complete" language appears in any result branch.
- Frontend tests assert no "succeeded" text is visible in the panel for any status.

**FMP-SEC-10: All stop behaviors unconditionally stop writes.**
- All four `FailureStopBehavior` variants (`StopAndReport`, `StopPreserveCheckpointAndReport`, `StopAfterRetryLimit`, `BlockAndRequireManualReview`) return `true` from `stops_writes()`.
- There is no stop behavior variant that permits write continuation after a failure.
- Rust unit tests assert `stops_writes()` returns `true` for every variant.

**FMP-SEC-11: No-plans causes immediate blocked (short-circuit).**
- When no `handlingPlans` field is provided, the function returns after FMP-02 with 2 checks and `Blocked` status.
- No subsequent check fields are evaluated.
- Rust unit tests assert `checks.len() == 2` in the no-plans case.

---

## Restore Rollback Limitation Policy Safety (Gate 14)

**Goal:** Confirm that the rollback limitation policy gate makes no Airtable API calls, requires no token, performs no automatic rollback or cleanup operations, never introduces a restore success state, and always returns `writesEnabled: false`.

**RLP-SEC-01: No token accepted.**
- `RollbackLimitationPolicyRequest` has no `token` field in its Rust struct definition.
- `RollbackLimitationPolicyResult` has no `token` field in any result variant.
- `RollbackLimitationPlan` has no `token` field.
- No token is forwarded to any downstream function.
- Rust unit test `no_token_or_path_in_serialized_result` asserts no `pat_` or `token` string appears in the serialized result.

**RLP-SEC-02: No Airtable API calls.**
- `verify_rollback_limitation_policy` calls only `evaluate_write_gate()` and in-memory logic.
- No HTTP client is constructed or called.
- No Airtable endpoint is referenced in the function.

**RLP-SEC-03: No write operations.**
- `verify_rollback_limitation_policy` produces no writes to any storage system, file, or network endpoint.
- `no_changes_made` is always `true`. `network_writes_attempted` is always `false`.
- Rust unit tests assert both invariants across all result branches.

**RLP-SEC-04: No record payload in any result field.**
- `RollbackLimitationPolicyResult` contains status, checks, message, plan summary (boolean/string flags only), and safety invariant fields.
- `RollbackLimitationPlan` contains only enum values and boolean flags — no record IDs, no field values, no base IDs.
- The plan summary mirrors only safe boolean/string flags from the plan.

**RLP-SEC-05: No full path in any field.**
- No filesystem path appears in any request or result field.
- Rust unit test `no_token_or_path_in_serialized_result` asserts no `/Users/` or `/home/` string appears in the serialized result.

**RLP-SEC-06: `noChangesMade` always true.**
- Every code path in `verify_rollback_limitation_policy` sets `no_changes_made: true` via the `build_result` helper.
- Rust unit tests assert `no_changes_made` across compliant, warning, and blocked results.

**RLP-SEC-07: No execute button or cleanup button.**
- `RestoreRollbackLimitationPolicyPanel` renders no execute, restore, cleanup, delete-all, or revert control.
- Frontend tests assert no "execute", "start restore", "cleanup", "delete all", or "revert" text is visible in the panel.

**RLP-SEC-08: Compliant status does not enable writes.**
- `writes_enabled` is always `false` in `build_result` — independent of policy status.
- Rust unit tests assert `!result.writes_enabled` for both compliant and blocked results.

**RLP-SEC-09: Compliant status does not introduce a restore success state.**
- The compliant result message explicitly states writes remain disabled.
- No "succeeded" or "restore complete" language appears in any result branch.
- Frontend tests assert no "succeeded" text is visible in the panel for any status.

**RLP-SEC-10: No automatic destructive rollback path exists.**
- `automaticDestructiveRollback`, `automaticDeleteCleanup`, and `automaticUpdateRevertCleanup` rollback behaviors are checked and return `Blocked` — they are declaration-only types and trigger no actual rollback code.
- There is no function in `rollback_limitation_policy.rs` that calls a delete, update, or revert API.
- Rust unit tests assert `Blocked` for all three destructive behavior variants.

**RLP-SEC-11: Manual cleanup requires separate explicit future action.**
- The `manualCleanupRequiresSeparateAction` field must be `true` for the plan to pass RLP-09.
- `false` triggers `Blocked` — the policy cannot be satisfied while claiming automatic cleanup is triggered.
- No automatic cleanup flow exists anywhere in the restore implementation.

**RLP-SEC-12: No-plan causes immediate blocked (short-circuit).**
- When no `plan` field is provided, the function returns after RLP-02 with 2 checks and `Blocked` status.
- No subsequent check fields are evaluated.
- Rust unit tests assert `checks.len() == 2` in the no-plan case.

---

## Restore Final Validation Enforcement Policy Safety (Gate 15)

**Goal:** Confirm that the final validation enforcement policy gate makes no Airtable API calls, requires no token, never labels any result complete without final validation explicitly passing, never introduces a restore success state, and always returns `writesEnabled: false`.

**FVE-SEC-01: No token accepted.**
- `FinalValidationEnforcementPolicyRequest` has no `token` field in its Rust struct definition.
- `FinalValidationEnforcementPolicyResult` has no `token` field in any result variant.
- `FinalValidationEnforcementPlan` has no `token` field.
- No token is forwarded to any downstream function.
- Rust unit test `no_token_or_path_in_serialized_result` asserts no `pat_` or `token` string appears in the serialized result.

**FVE-SEC-02: No Airtable API calls.**
- `verify_final_validation_enforcement_policy` calls only `evaluate_write_gate()` and in-memory logic.
- No HTTP client is constructed or called.
- No Airtable endpoint is referenced in the function.

**FVE-SEC-03: No write operations.**
- `verify_final_validation_enforcement_policy` produces no writes to any storage system, file, or network endpoint.
- `no_changes_made` is always `true`. `network_writes_attempted` is always `false`.
- Rust unit tests assert both invariants across all result branches.

**FVE-SEC-04: No record payload in any result field.**
- `FinalValidationEnforcementPolicyResult` contains status, checks, message, enforcement summary (validation state strings and boolean flags only), and safety invariant fields.
- `FinalValidationEnforcementPlan` contains only enum validation states and boolean flags — no record IDs, no field values, no base IDs.
- The enforcement summary mirrors only safe state strings and boolean flags.

**FVE-SEC-05: No full path in any field.**
- No filesystem path appears in any request or result field.
- Rust unit test `no_token_or_path_in_serialized_result` asserts no `/Users/` or `/home/` string appears in the serialized result.

**FVE-SEC-06: `noChangesMade` always true.**
- Every code path in `verify_final_validation_enforcement_policy` sets `no_changes_made: true`.
- Rust unit tests assert `no_changes_made` across compliant, warning, and blocked results.

**FVE-SEC-07: No execute button.**
- `RestoreFinalValidationEnforcementPolicyPanel` renders no execute, restore, or write-start control.
- Frontend tests assert no "execute" or "start restore" text is visible in the panel.

**FVE-SEC-08: Compliant status does not enable writes.**
- `writes_enabled` is always `false` — independent of policy status.
- Rust unit tests assert `!result.writes_enabled` for all result branches.

**FVE-SEC-09: Compliant status does not introduce a restore success state.**
- The compliant result message explicitly states writes remain disabled.
- No "succeeded" or "restore complete" language appears in any result branch.
- Rust unit test `no_success_state_in_result_message` asserts no success language in any result.
- Frontend tests assert no "succeeded" text is visible in the panel for any status.

**FVE-SEC-10: No result may be labeled complete without final validation passing.**
- The completion guard invariant `blocks_completion_without_final_validation` must be `true` for FVE-03 to pass.
- Any `ValidationCompletionState` other than `Passed` (or `NotRequired` with reason) blocks the policy.
- `Skipped`, `Partial`, and `NotDeclared` states always produce `Blocked`.
- Rust unit tests assert `Blocked` for each blocking validation state variant.

**FVE-SEC-11: Completion guard fully declared.**
- All three `RestoreCompletionGuard` invariants must be explicitly `true`: `blocks_completion_without_final_validation`, `blocks_partial_validation_as_completion`, `failedValidationBlocksCompletion`.
- Missing guard or any invariant set to `false` causes `Blocked` on FVE-03.
- Rust unit tests assert `Blocked` for missing guard and for incomplete guard.

**FVE-SEC-12: No-plan causes immediate blocked (short-circuit).**
- When no `plan` field is provided, the function returns after FVE-02 with 2 checks and `Blocked` status.
- No subsequent check fields are evaluated.
- Rust unit tests assert `checks.len() == 2` in the no-plan case.

---

## Restore Sensitive Data Safety Policy Safety (Gate 16)

**Goal:** Confirm that the sensitive data safety policy gate makes no Airtable API calls, requires no token, never exposes sensitive material through any restore write surface, never introduces a restore success state, and always returns `writesEnabled: false`.

**SDS-SEC-01: No token accepted.**
- `SensitiveDataSafetyPolicyRequest` has no `token` field in its Rust struct definition.
- `SensitiveDataSafetyPolicyResult` has no `token` field in any result variant.
- `SensitiveDataSafetyPlan` has no `token` field.
- Rust unit test `no_token_or_path_in_serialized_result` asserts no `pat_` or `token` string appears in the serialized result.

**SDS-SEC-02: No Airtable API calls.**
- `verify_sensitive_data_safety_policy` calls only `evaluate_write_gate()` and in-memory logic.
- No HTTP client is constructed or called.
- No Airtable endpoint is referenced in the function.

**SDS-SEC-03: No write operations.**
- `verify_sensitive_data_safety_policy` produces no writes to any storage system, file, or network endpoint.
- `no_changes_made` is always `true`. `network_writes_attempted` is always `false`.
- Rust unit tests assert both invariants across all result branches.

**SDS-SEC-04: No record payload in any result field.**
- `SensitiveDataSafetyPolicyResult` contains status, checks, message, safety summary (counts and boolean flags only), and safety invariant fields.
- `SensitiveDataSafetyPlan` contains redaction coverage entries and boolean flags — no record IDs, no field values, no base IDs.
- The safety summary mirrors only safe counts and boolean flags.

**SDS-SEC-05: No full path in any field.**
- No filesystem path appears in any request or result field.
- Rust unit test `no_token_or_path_in_serialized_result` asserts no `/Users/` or `/home/` string appears in the serialized result.

**SDS-SEC-06: No package path in any field.**
- Package references are filename-only. No package path appears in any request or result field.
- `SDS-06` check enforces `package_references_filename_only: true`.

**SDS-SEC-07: No attachment URL in any field.**
- No attachment URL appears in any request or result field.
- `SDS-08` check enforces `no_attachment_url_in_results: true`.

**SDS-SEC-08: No raw HTTP data in any field.**
- No raw HTTP request or response body appears in any request or result field.
- `SDS-09` check enforces `no_raw_http_in_results: true`.

**SDS-SEC-09: `noChangesMade` always true.**
- Every code path in `verify_sensitive_data_safety_policy` sets `no_changes_made: true`.
- Rust unit tests assert `no_changes_made` across compliant, warning, and blocked results.

**SDS-SEC-10: No execute button.**
- `RestoreSensitiveDataSafetyPolicyPanel` renders no execute, restore, or write-start control.
- Frontend tests assert no "execute" or "start restore" text is visible in the panel.

**SDS-SEC-11: Compliant status does not enable writes.**
- `writes_enabled` is always `false` — independent of policy status.
- Rust unit tests assert `!result.writes_enabled` for all result branches.

**SDS-SEC-12: Compliant status does not introduce a restore success state.**
- The compliant result message explicitly states writes remain disabled.
- No "succeeded" or "restore complete" language appears in any result branch.
- Rust unit test `no_success_state_in_result_message` asserts no success language in any result.
- Frontend tests assert no "succeeded" text is visible in the panel for any status.

**SDS-SEC-13: All 10 exposure surfaces must have redaction coverage.**
- `SDS-03` checks that all 10 required surfaces are present in `redaction_coverage`.
- Missing surface causes `Blocked`.
- Rust unit tests assert `Blocked` for each missing surface variant.

**SDS-SEC-14: No-plan causes immediate blocked (short-circuit).**
- When no `plan` field is provided, the function returns after SDS-02 with 2 checks and `Blocked` status.
- No subsequent check fields are evaluated.
- Rust unit tests assert `checks.len() == 2` in the no-plan case.

**SDS-SEC-15: SDS-12 warning only — unnamed rules do not block.**
- Unnamed redaction rules produce `Warning` only — this is intentional; auditability is reduced but safety invariants are not violated.
- Rust unit tests assert `Warning` (not `Blocked`) for unnamed rules with all other flags compliant.

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

---

## Schema Write Engine Foundation Security (V0.1)

**Goal:** Confirm that the schema write request plan builder makes no Airtable API calls, accepts no token, never creates any Airtable schema objects, and always returns `noChangesMade: true`.

**SW-01: No token in request or result.**
- `SchemaWriteRequestPlanRequest` has no `token` field (Rust struct and TypeScript interface).
- `SchemaWriteRequestPlanResult` has no `token` field.
- The serialized JSON request and result do not contain `"token"` or `"apiKey"` keys.
- Rust and frontend tests assert `JSON.stringify()` contains neither key.

**SW-02: No Airtable API calls.**
- `execute_schema_write_dry_run` calls only `evaluate_write_gate()` — no HTTP client is constructed.
- No `AirtableClient`, no `reqwest` call, and no network socket is opened during the command.
- Network monitoring during a schema write plan operation shows zero connections to `api.airtable.com`.

**SW-03: No Airtable schema created.**
- The request plan builder (`build_schema_write_request_plan`) produces a read-only ordered plan.
- There is no `create_table` / `create_field` / `create_base` call reachable from this code path.
- The write gate (`evaluate_write_gate`) has one outcome: `Disabled/DisabledByProductPolicy`.

**SW-04: No `succeeded` status.**
- `SchemaWriteOperationStatus` has three variants: `Planned`, `Blocked`, `Disabled`.
- There is no `Succeeded` variant — it was intentionally omitted.
- The serialized JSON result never contains `"succeeded"` as a status value.
- Rust and frontend tests assert this property.

**SW-05: `noChangesMade` always true.**
- `SchemaWriteRequestPlan.no_changes_made` and `SchemaWriteDryRunResult.no_changes_made` are hard-coded `true` at every code path.
- Rust and frontend tests assert this property for both disabled and blocked result paths.

**SW-06: `networkWritesAttempted` always false.**
- `SchemaWriteRequestPlan.network_writes_attempted` and `SchemaWriteDryRunResult.network_writes_attempted` are hard-coded `false` at every code path.
- Rust and frontend tests assert this property.

**SW-07: Write gate always disabled for schema write plan.**
- `evaluate_write_gate()` returns `Disabled/DisabledByProductPolicy` regardless of plan content.
- Rust unit tests assert always-disabled, always-product-policy behavior.
- The schema write engine cannot advance past disabled — there is no enabled branch.

**SW-08: IPC fallback is safe.**
- When Tauri IPC is unavailable, `liveAirBridgeService.previewSchemaWriteRequestPlan()` returns a disabled fallback with `noChangesMade: true`, `networkWritesAttempted: false`, zero op counts, and no token field.
- Frontend tests assert the fallback result does not contain `"succeeded"`.

**SW-09: Schema write plan does not affect restore write gate.**
- Calling `previewSchemaWriteRequestPlan` does not change the outcome of `previewRestoreWriteEngine`.
- Frontend tests assert that `previewRestoreWriteEngine` still returns `status: "disabled"` after a schema write plan is previewed.

---

## Credential Storage Security (V0.1)

**Goal:** Confirm that OS keychain credential storage never exposes, echoes, persists to disk, or
logs the token value. Saving is optional; the keychain unavailable state is handled safely.

**CS-01: Token never returned by commands.**
- `get_credential_storage_status` result has no `token` or `secret` field.
- `save_airtable_token_to_keychain` result has no `token` or `secret` field.
- `remove_airtable_token_from_keychain` result has no `token` or `secret` field.
- All three result structs are serialized to JSON; Rust unit tests assert no `"token"` or `"secret"` key appears.

**CS-02: Token not in plaintext on disk.**
- The token is forwarded to the OS keychain API via the `keyring` crate (`set_secret(bytes)`).
- No file in the AirBridge application directory contains the token.
- No SQLite database entry, `localStorage` entry, or `sessionStorage` entry contains the token.
- The application data directory can be inspected to verify: no `.json`, `.db`, `.sqlite`, or `.log` file contains the token string.

**CS-03: Token never in logs.**
- The Rust commands pass the token from request to keychain without logging it.
- All error paths map to `safe_message()` static strings — none contain the token value.
- `ensure_no_token_in_message()` is applied to all display strings before they are returned.

**CS-04: Token input is always masked in the UI.**
- `CredentialStorageCard` renders the token input with `type="password"`.
- The raw token value is never rendered outside the masked input.
- After a successful save, the input is cleared (empty string) and removed from the DOM.
- Frontend tests assert `screen.queryByTestId("credential-token-input") === null` after a save.

**CS-05: No localStorage or sessionStorage usage.**
- `CredentialStorageCard` does not call `localStorage.setItem` or `sessionStorage.setItem`.
- Frontend tests spy on `Storage.prototype.setItem` and assert no call contains the token value or a credential-related key.

**CS-06: `CredentialSaveRequest` is not serializable.**
- `CredentialSaveRequest` in Rust derives `Deserialize` only — it cannot be serialized back to JSON.
- The token cannot be included in any response, event, or log via accidental serialization.

**CS-07: Keychain unavailable is handled safely.**
- When `OsKeychainStore::availability()` returns `Unavailable`, all three commands return safe results with `hasSavedToken: false` and a static message — no token is inspected or returned.
- The UI renders an unavailable notice and hides the token input and save button.
- No error message contains the token value.

**CS-08: Credential storage does not affect the restore write gate.**
- `evaluate_write_gate()` returns `Disabled/DisabledByProductPolicy` regardless of whether a token is saved.
- Rust unit test `credential_storage_does_not_affect_restore_write_gate` asserts the gate status is `Disabled` before and after a credential status check.
- Saving a token via the mock service and then calling `previewRestoreWriteEngine` returns `status: "disabled"` — frontend tests assert this.

**CS-09: Mock service stores presence only — not the token value.**
- `_mockCredentialStore` is a `Map<CredentialKind, boolean>` — it records whether a save succeeded, not the token string.
- `JSON.stringify(saveResult)` does not contain the test sentinel token — frontend tests assert this.

**CS-10: IPC fallback results are safe.**
- When Tauri IPC is unavailable, `liveAirBridgeService.saveAirtableTokenToKeychain()` returns a safe fallback with `success: false`, `hasSavedToken: false`, and a static unavailable message.
- The fallback contains no token field and no token value.

---

## Record Write Engine Foundation Security (V0.1)

**Goal:** Confirm that the record write request plan builder makes no Airtable API calls, accepts no token, never creates or modifies any Airtable records, contains no raw record payloads, and always returns `noChangesMade: true`.

**RW-01: No token in request or result.**
- `RecordWriteRequestPlanRequest` has no `token` field (Rust struct and TypeScript interface).
- `RecordWriteRequestPlanResult` has no `token` field.
- The serialized JSON request and result do not contain `"token"` or `"apiKey"` keys.
- Rust and frontend tests assert `JSON.stringify()` contains neither key.

**RW-02: No Airtable API calls.**
- `execute_record_write_dry_run` calls only `evaluate_write_gate()` — no HTTP client is constructed.
- No `AirtableClient`, no `reqwest` call, and no network socket is opened during the command.
- Network monitoring during a record write plan operation shows zero connections to `api.airtable.com`.

**RW-03: No records created, updated, or deleted.**
- The request plan builder (`build_record_write_request_plan`) produces a read-only ordered plan.
- There is no `create_records` / `update_records` / `delete_records` call reachable from this code path.
- The write gate (`evaluate_write_gate`) has one outcome: `Disabled/DisabledByProductPolicy`.

**RW-04: No raw record payloads in result.**
- `RecordWriteRequestPlanResult` contains only counts and operation metadata.
- No field values, record IDs, or record contents appear in the result.
- Frontend tests assert `JSON.stringify(result)` does not contain `"records":`, `"payload":`, or `"newRecordId"`.

**RW-05: No `succeeded` status.**
- `RecordWriteOperationStatus` has three variants: `Planned`, `Blocked`, `Disabled`.
- There is no `Succeeded` variant — it was intentionally omitted.
- The serialized JSON result never contains `"succeeded"` as a status value.
- Rust and frontend tests assert this property.

**RW-06: `noChangesMade` always true.**
- `RecordWriteRequestPlan.no_changes_made` and `RecordWriteDryRunResult.no_changes_made` are hard-coded `true` at every code path.
- Rust and frontend tests assert this property for both disabled and blocked result paths.

**RW-07: `networkWritesAttempted` always false.**
- `RecordWriteRequestPlan.network_writes_attempted` and `RecordWriteDryRunResult.network_writes_attempted` are hard-coded `false` at every code path.
- Rust and frontend tests assert this property.

**RW-08: Write gate always disabled for record write plan.**
- `evaluate_write_gate()` returns `Disabled/DisabledByProductPolicy` regardless of plan content.
- Rust unit tests assert always-disabled, always-product-policy behavior.
- The record write engine cannot advance past disabled — there is no enabled branch.

**RW-09: Old-to-new record ID mapping deferred to execution.**
- `UpdateLinkedRecordBatch` operations note explicitly: "ID mapping unavailable until execution".
- No actual Airtable record IDs (`rec…`) appear in the plan result.
- The plan faithfully represents `RestoreRecordMappingStrategy::UnavailableUntilExecution`.

**RW-10: IPC fallback is safe.**
- When Tauri IPC is unavailable, `liveAirBridgeService.previewRecordWriteRequestPlan()` returns a disabled fallback with `noChangesMade: true`, `networkWritesAttempted: false`, zero op counts, and no token field.
- Frontend tests assert the fallback result does not contain `"succeeded"`.

**RW-11: Record write plan does not affect other write gates.**
- Calling `previewRecordWriteRequestPlan` does not change the outcome of `previewRestoreWriteEngine` or `previewSchemaWriteRequestPlan`.
- Frontend tests assert both gates still return `status: "disabled"` after a record write plan is previewed.

---

## Live Write Safety Contract Non-Regression (V0.1)

**Goal:** Confirm that no live write path has become reachable and that all contract invariants still hold.

**LW-01: Write gate still returns disabled.**
- `evaluate_write_gate()` returns `Disabled/DisabledByProductPolicy`.
- Rust unit tests in `write_gate.rs` and `write_safety_contract.rs` (CONTRACT-01) assert this.

**LW-02: No Succeeded status in any write type.**
- `RestoreWriteEngineStatus`, `SchemaWriteOperationStatus`, and `RecordWriteOperationStatus` all lack a `Succeeded` variant.
- Rust tests in `write_safety_contract.rs` (CONTRACT-02) confirm serialized values never contain `"succeeded"`.

**LW-03: noChangesMade always true across all write foundations.**
- Write safety report, schema write dry-run result, and record write dry-run result all set `no_changes_made: true`.
- Rust tests in `write_safety_contract.rs` (CONTRACT-03) assert this across all three code paths.

**LW-04: networkWritesAttempted always false across all write foundations.**
- Schema and record write plan results both set `network_writes_attempted: false`.
- Rust tests in `write_safety_contract.rs` (CONTRACT-04) assert this.

**LW-05: restore_success_possible always false.**
- `RestoreWriteSafetyReport.restore_success_possible` is hard-coded `false`.
- Rust test CONTRACT-05 asserts this.

**LW-06: No token in any write result.**
- Write gate message, safety report, schema write plan result, and record write plan result contain no `"token"` or `"apiKey"` key.
- Rust tests in `write_safety_contract.rs` (CONTRACT-12) assert this for all four result types.

**LW-07: No full path in any write result.**
- Write gate message and safety report contain no `/Users/`, `/home/`, or `/tmp/` string.
- Rust tests in `write_safety_contract.rs` (CONTRACT-15) assert this.

**LW-08: Attachment phase still disabled.**
- `RestoreWriteSafetyReport.writes_enabled` is `false`.
- Record write plan with empty attachment fields produces `attachment_op_count: 0`.
- Rust test CONTRACT-16 asserts both.

**LW-09: No Airtable write endpoint reachable.**
- No `create_records`, `update_records`, `create_table`, `create_field`, or equivalent Airtable write function is callable from any restore write planning code path.
- Verified by the absence of any such call in `write_gate.rs`, `write_safety.rs`, `write_engine.rs`, `schema_write_executor.rs`, and `record_write_executor.rs`.

**LW-10: Safety contract tests in `write_safety_contract.rs`.**
- 20 tests in `restore::write_safety_contract::tests` all pass.
- Each test is named with its `CONTRACT-XX` identifier for traceability to the written contract.
