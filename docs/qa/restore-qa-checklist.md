# Restore QA Checklist

Use this checklist when verifying the restore functionality against a release build. Mark each item pass (P), fail (F), or not applicable (N/A). Record the build version and test date at the top.

**Build version:** ___________
**Test date:** ___________
**Tested by:** ___________
**Platform:** ___________

---

## Pre-Restore: Backup File Validation

- [ ] **Manifest is present and valid.** If `manifest.json` is absent or malformed, the restore is refused with a clear error message before any API calls are made.
- [ ] **Schema is present and valid.** If `schema.json` is absent or cannot be parsed, the restore is refused with a clear error identifying the missing file.
- [ ] **Records file is present (for full restores).** If a full restore is requested but `records.jsonl` is absent, the error message distinguishes this case from a schema-only backup.
- [ ] **Version compatibility warning.** If the `airBridgeVersion` in the manifest is newer than the running application version, a compatibility warning is shown. The user can acknowledge and proceed, or cancel.
- [ ] **Corrupted backup handling.** Use the `corrupted-backup` fixture to verify that the application shows an error state, does not crash, and presents a path back to the home screen.

---

## Dry-Run Plan Preview (Package-Based, No Airtable Writes)

- [ ] **Select file shows filename only.** After choosing a `.airbridge` file, the UI shows the filename (e.g., `backup.airbridge`) — never the full directory path.
- [ ] **Generate plan button disabled until file is selected.** Verify the button is disabled on mount and enabled only after a file is picked.
- [ ] **No token is requested.** The dry-run panel does not contain a token input field.
- [ ] **No restore execution button.** The panel does not render a "Start Restore" or equivalent button.
- [ ] **Plan shows status badge.** After generation, a status badge shows "Ready", "Ready with warnings", or "Blocked".
- [ ] **"No Airtable changes were made" notice is always present.** After generating a plan, the safety notice is visible regardless of plan status.
- [ ] **Package summary is shown.** Plan result includes source base name, provider, table count, field count, and record count.
- [ ] **Field compatibility is shown.** Each field in each table has a compatibility badge: Supported / Partial / Metadata only / Unsupported / Manual.
- [ ] **Linked record remapping warning appears.** For packages containing linked record fields, the warnings list includes a `LINKED_RECORD_REMAPPING_REQUIRED` warning.
- [ ] **Attachment metadata-only warning appears.** For packages containing attachment fields, the warnings list includes an `ATTACHMENT_METADATA_ONLY` warning.
- [ ] **Computed field warning appears.** For packages containing formula or rollup fields, the appropriate warning code is shown.
- [ ] **Record import ordering plan is shown.** The ordering section describes the four steps: create tables, create fields, import records without links, apply links.
- [ ] **Target mode selector works.** User can switch between "New base" and "Empty existing base".
- [ ] **Blocked plan for invalid package.** Selecting a corrupted or invalid package results in a blocked-status plan with an error message — the app does not crash.
- [ ] **No absolute paths in rendered output.** Inspect the DOM with developer tools; no directory separators appear in any visible text.

## Dry-Run Mode (Legacy checklist items — applies to future restore execution)

- [ ] **Dry run produces a correct plan.** For a fixture-loaded backup, the dry-run report lists all tables and fields that would be created, with accurate counts.
- [ ] **Dry run makes no writes.** After a dry run completes, the target base in Airtable is unmodified. Verify by checking the base's record count before and after.
- [ ] **Dry run identifies unsupported fields.** Any fields whose types cannot be restored to the target base are listed in the dry-run report with their names and types.
- [ ] **Dry run identifies linked record dependencies.** For backups with linked record fields, the dry-run report shows the link relationships that will be established and the order in which tables will be created.
- [ ] **Dry-run report is exportable.** The report can be copied to clipboard or saved to a file.

---

## Restore Permissions

- [ ] **Records write permission check.** If the token lacks write permission on the target base, an error is shown before any writes are attempted. The error message identifies the permission problem specifically.
- [ ] **Schema write permission check.** If the token can write records but cannot create tables or fields, a specific error is shown for the schema creation failure. The user is not left wondering why records were not created.
- [ ] **Read-back verification access.** After creating records, AirBridge may read them back to confirm. If this read fails due to permissions, a warning (not an error) is shown; the restore is considered successful.

---

## Schema Creation Plan (No Airtable Writes)

- [ ] **Schema plan panel renders.** The "Schema Creation Plan" section is visible on the Restore page between the dry-run panel and the execution gate.
- [ ] **Generate button disabled without inspection.** Before a package is inspected, the button is disabled and a "Inspect a package first." message is shown.
- [ ] **Generate button disabled when dry-run is blocked.** If the dry-run status is "blocked", the button is disabled and a "Generate a restore plan preview first." message is shown.
- [ ] **No token is requested.** The schema plan panel does not contain a token input field.
- [ ] **No restore execution button.** The panel does not render a "Start Restore" or equivalent button.
- [ ] **Plan shows status badge.** After generation, a status badge shows "Ready", "Ready with Warnings", or "Blocked".
- [ ] **Table creation steps are listed.** Each table appears as a numbered step with direct, deferred, manual, and unsupported field counts.
- [ ] **Field creation steps are listed.** Each directly-creatable field is listed with its classification.
- [ ] **Deferred linked fields are listed.** Linked record fields appear in the deferred section, not the field creation steps section.
- [ ] **Manual action fields are listed.** formula, rollup, lookup, and collaborator fields appear with their action descriptions.
- [ ] **Dependency graph is shown.** Linked record dependency edges are rendered, showing source table, target table, and remapping notice.
- [ ] **Warnings are shown.** Any attachment metadata, deferred link, unsupported, or manual-action warnings appear.
- [ ] **"No Airtable changes were made" notice is always present.** The notice is visible after generating any plan.
- [ ] **Full package path is not visible.** No absolute directory path appears in any visible element.

---

## Record Import Plan (No Airtable Writes)

- [ ] **Record import plan panel renders.** The "Record Import Plan" section is visible on the Restore page between the schema plan panel and the execution gate.
- [ ] **Generate button disabled without inspection.** Before a package is inspected, the button is disabled and a "Inspect a package first." message is shown.
- [ ] **Generate button disabled when dry-run is blocked.** If the dry-run status is "blocked", the button is disabled.
- [ ] **Generate button disabled when schema plan is blocked.** If the schema plan status is "blocked", the button is disabled and a "Generate a schema creation plan first." message is shown.
- [ ] **No token is requested.** The record import plan panel does not contain a token input field.
- [ ] **No restore execution button.** The panel does not render a "Start Restore" or equivalent button.
- [ ] **Plan shows status badge.** After generation, a status badge shows "Ready", "Ready with Warnings", or "Blocked".
- [ ] **Table import plans are listed.** Each table appears with its name, record count (or "unknown"), create batch count, and batch size.
- [ ] **Linked record second-pass section is shown.** Tables with linked record fields show a second-pass update entry.
- [ ] **Attachment metadata notice is shown.** Tables with attachment fields show a "metadata only, manual re-attachment required" notice.
- [ ] **Retry policy note is shown.** The retry configuration (max retries, initial backoff, multiplier) is visible.
- [ ] **Warnings are shown.** RECORD_COUNT_UNKNOWN, ATTACHMENT_METADATA_ONLY, COMPUTED_FIELDS_SKIPPED, LINKED_RECORD_SECOND_PASS_REQUIRED warnings are visible where applicable.
- [ ] **"No Airtable records were created or modified" notice is always present.** The notice is visible after generating any plan.
- [ ] **Full package path is not visible.** No absolute directory path appears in any visible element.
- [ ] **Batch size is 10.** All batch count calculations use a batch size of 10.

---

## Restore Execution Gate (Current Version — Write Engine Disabled)

- [ ] **Gate panel renders.** The "Restore Execution" section is visible on the Restore page with a prerequisites checklist and form inputs.
- [ ] **Not-enabled notice is always visible.** A notice reading "Restore execution is not enabled in this version" is shown at all times — before and after any attempt.
- [ ] **Token input is a password field.** The access token input uses `type="password"`. The entered value is masked and not visible in the page text.
- [ ] **Attempt button disabled without prerequisites.** With no package inspected, no dry-run plan, no token, or incorrect confirmation text, the "Attempt Restore" button is `disabled`.
- [ ] **Button enables only when all five prerequisites are met.** Package valid, dry-run ready, target mode set, token non-empty, confirmation exactly "RESTORE BACKUP".
- [ ] **Confirmation text must match exactly.** Typing "restore backup" (lowercase), "RESTORE" alone, or any other variation does not enable the button.
- [ ] **Result shows "No Airtable changes were made."** After clicking Attempt Restore, the result panel always shows this notice regardless of gate outcome.
- [ ] **Status is "Disabled" when all gates pass.** The result badge reads "Disabled" and the block reason is `restoreWriteEngineNotEnabled`.
- [ ] **Token is cleared after attempt.** After the attempt completes (or is cancelled), the token input is empty.
- [ ] **Full package path is not visible.** Inspect the DOM; no absolute directory path appears in any visible element.
- [ ] **No success or "restore complete" message.** The result panel contains no text suggesting a restore completed or succeeded.
- [ ] **Cancel button clears state.** Clicking Cancel after an attempt empties the token input and hides the result panel.

---

## Restore Execution

### Basic Restore

- [ ] **Restore to new/empty base succeeds.** Using the `simple-base` fixture and an empty target base, a full restore completes without errors. The target base in Airtable reflects all tables, fields, and records from the fixture.
- [ ] **Table creation order is correct.** Tables are created in an order that satisfies linked record dependencies — linked tables are created before the tables that reference them.
- [ ] **Field creation is correct.** Each restored field has the correct name and type. Select field choices are created in the correct order. Date formats and number precision are preserved where the API supports it.
- [ ] **Record count after restore.** The number of records in the target base matches `recordCount` in the backup manifest.

### Linked Record Remapping

- [ ] **Linked record IDs are remapped.** Using the `linked-records-base` fixture, verify that after restore, linked record fields correctly point to the restored records in the target base, not to the source record IDs from the backup.
- [ ] **Remapping is logged in the report.** The restore report shows the old-to-new record ID mapping used during the restore.
- [ ] **No broken links.** After restore, open the target base in Airtable and verify that linked record cells contain valid links, not empty or error states.

### Unsupported Fields

- [ ] **Unsupported fields are listed and skipped.** For a backup containing a field type that AirBridge cannot restore (e.g., formula, rollup, lookup), the restore proceeds for all supported fields and skips the unsupported ones.
- [ ] **Skip behavior is non-destructive.** Skipping an unsupported field does not abort the restore or leave the table in a partially created state.
- [ ] **User is clearly informed.** The restore report prominently lists every skipped field with the reason (e.g., "formula fields cannot be restored — skipped").

---

## Rate Limit Handling

- [ ] **429 responses are retried.** If the Airtable API returns a 429 during a restore, the application pauses and retries after the retry-after interval. The user sees a "Rate limited — retrying" message, not a frozen UI.
- [ ] **Retry does not duplicate records.** After a rate-limit retry, the record that was being written when the 429 occurred is not written twice.

---

## Error Recovery

- [ ] **Partial restore is reported accurately.** If a restore fails partway through (e.g., network disconnect mid-operation), the restore report shows exactly which tables and how many records were written before the failure.
- [ ] **Application does not crash on restore failure.** The UI returns to an error state with a summary, not a blank screen or unresponsive window.
- [ ] **User can retry or cancel after failure.** After a partial failure, the user can either retry the restore (understanding the risk of duplicates) or cancel and inspect the partial state in Airtable.

---

## Post-Restore Verification Steps

After completing a restore, perform the following manual checks in Airtable:

- [ ] Open the target base in Airtable and verify that all expected tables are present.
- [ ] In each table, verify that all expected fields (name and type) are present.
- [ ] In each table, verify that the record count matches the backup manifest.
- [ ] For a table with a singleSelect field, verify that the select choices are present and the records reference valid choices.
- [ ] For a table with linked records, verify that at least one linked record cell contains a valid link (not an empty cell).
- [ ] For a table with a checkbox field, verify that checked and unchecked values are preserved correctly.
- [ ] For a table with a date field, verify that date values are not shifted by time zone conversion.

---

## Sandbox Verification Checklist (Gate 1)

### Before testing

- [ ] Confirm `verify_restore_sandbox_environment` is registered in the Tauri invoke handler.
- [ ] Confirm `RestoreSandboxVerificationPanel` is rendered on the Restore page.
- [ ] Confirm no token field appears in the sandbox verification request type.

### Panel behavior

- [ ] The "Sandbox Verification (Gate 1)" section is visible on the Restore page at all times.
- [ ] The disabled notice reads "Sandbox verification checks local safety conditions only. Restore writes remain disabled in this version."
- [ ] A "Verify sandbox safety" button is shown before the first verification run.
- [ ] Clicking the button triggers `verifyRestoreSandboxEnvironment` and shows the result.
- [ ] After a result, the button changes to "Re-verify".
- [ ] No execute button is shown anywhere in the panel.
- [ ] No token input is shown.
- [ ] No success message ("Restore complete", "Restore successful", "Succeeded") is shown.
- [ ] Full filesystem path is not visible.

### Result display

- [ ] Overall status badge (`verified`, `warning`, or `blocked`) is shown.
- [ ] Verification message is shown.
- [ ] Each check row (CHK-01 through CHK-10) is shown with its label, status, and message.
- [ ] CHK-10 (Live metadata check) always shows `skipped`.
- [ ] Safety summary shows `writesEnabled: No`, `networkWritesAttempted: No`, `liveMetadataCheckPerformed: No`.
- [ ] "No Airtable changes were made" is shown in the safety summary.
- [ ] When status is `blocked`, a blocked notice is shown.
- [ ] When status is `verified` or `warning`, a "writes still disabled" notice is shown.

### Safety invariants

- [ ] `noChangesMade` is `true` in every sandbox verification result (confirmed via Rust and frontend tests).
- [ ] `writesEnabled` is always `false` in every sandbox verification result.
- [ ] `networkWritesAttempted` is always `false`.
- [ ] The sandbox verification request type has no `token` field.
- [ ] The sandbox verification result has no `token` field.
- [ ] The sandbox verification result contains no full filesystem path.

---

## Restore Confirmation Gate Checklist (Gate 2)

### Before testing

- [ ] Confirm `validate_restore_confirmation_gate` is registered in the Tauri invoke handler.
- [ ] Confirm `RestoreConfirmationPanel` is rendered on the Restore page.
- [ ] Confirm no `token` field appears in `RestoreConfirmationRequest`.
- [ ] Confirm `RestoreConfirmationResult` has no `token` field.

### Panel behavior

- [ ] The "Restore Confirmation (Gate 2)" section is visible on the Restore page at all times.
- [ ] A writes-disabled notice is shown at the top of the panel.
- [ ] The required confirmation text is displayed before the text input.
- [ ] The text input accepts freeform entry.
- [ ] The "Validate" button is disabled when the text input is empty.
- [ ] The "Validate" button is enabled after typing any non-empty text.
- [ ] No execute button ("Start Restore", "Run Restore", "Execute Restore") is shown in the panel.
- [ ] No token input is shown.
- [ ] No "Restore complete", "Restore successful", or "Succeeded" language appears.
- [ ] Full filesystem path is not visible in any rendered element.

### Result display — rejected

- [ ] After submitting wrong-case text (e.g., `restore to my base`), status badge shows `rejected`.
- [ ] After submitting partial text (e.g., `RESTORE`), status badge shows `rejected`.
- [ ] After submitting text with extra words, status badge shows `rejected`.
- [ ] Rejected notice is shown with a message explaining the mismatch.

### Result display — confirmed

- [ ] After submitting the exact required text, status badge shows `confirmed`.
- [ ] Accepted notice is shown with a message indicating the text matched.
- [ ] "Writes remain disabled" notice is still shown even when status is `confirmed`.
- [ ] No execute button appears after a confirmed result.

### Result display — blocked

- [ ] If `sandboxVerificationStatus` is `"blocked"`, result is `blocked` regardless of text.
- [ ] Blocked notice is shown with a message explaining the prerequisite failure.
- [ ] A token-like string entered as text (e.g., a PAT-format string) results in `blocked` with CHK-C04 failed.

### Result display — check rows

- [ ] CHK-C01 (write gate) is shown with `passed` status.
- [ ] CHK-C02 (sandbox prerequisite) shows `skipped` when sandbox has not been run, `passed` when verified or warning, `failed` when blocked.
- [ ] CHK-C03 (exact text match) shows `passed` when text matches, `failed` when it does not.
- [ ] CHK-C04 (no token in text) shows `passed` for normal text, `failed` for PAT-format input.
- [ ] CHK-C05 (writes remain disabled) always shows `passed`.

### Required text display

- [ ] Required text is shown before the input.
- [ ] When a `targetLabel` is set in the service call, required text reflects the label.
- [ ] When no `targetLabel` and a `sourcePackageLabel` is set, required text reflects the filename.
- [ ] When neither is set, required text is `RESTORE BACKUP`.

### Safety invariants

- [ ] `noChangesMade` is `true` in every confirmation result (confirmed via Rust and frontend tests).
- [ ] `writesEnabled` is `false` in every confirmation result, including when status is `confirmed`.
- [ ] `networkWritesAttempted` is `false` in every confirmation result.
- [ ] The confirmation request type has no `token` field.
- [ ] The confirmation result has no `token` field.
- [ ] The confirmation result contains no full filesystem path.
- [ ] A `confirmed` result does NOT enable restore writes.

---

## Restore Target Empty Verification Checklist (Gate 3)

### Before testing

- [ ] Confirm `verify_restore_target_empty` is registered in the Tauri invoke handler.
- [ ] Confirm `RestoreTargetEmptyVerificationPanel` is rendered on the Restore page.
- [ ] Confirm no `token` field appears in `TargetEmptyVerificationRequest`.
- [ ] Confirm `TargetEmptyVerificationResult` has no `token` field.

### Panel behavior

- [ ] The "Target Empty Verification (Gate 3)" section is visible on the Restore page at all times.
- [ ] A writes-disabled notice is shown at the top of the panel.
- [ ] A "Verify target is empty" button is shown before the first verification run.
- [ ] Clicking the button triggers `verifyRestoreTargetEmpty` and shows the result.
- [ ] After a result, the button label changes to "Re-verify".
- [ ] No execute button ("Start Restore", "Run Restore", "Execute Restore") is shown.
- [ ] No token input is shown.
- [ ] No "Restore complete", "Restore succeeded", or "succeeded" language appears.
- [ ] Full filesystem path is not visible in any rendered element.

### Result display

- [ ] Overall status badge (`verified`, `warning`, or `blocked`) is shown.
- [ ] Result message is shown.
- [ ] Each check row (TEV-01 through TEV-05) is shown with its ID, label, and message.
- [ ] Safety summary shows `writesEnabled: No`, `networkWritesAttempted: No`.
- [ ] "No Airtable changes were made." is shown in the safety summary.
- [ ] When status is `verified`, a verified notice reads "Restore writes remain disabled".
- [ ] When status is `warning`, a warning notice is shown.
- [ ] When status is `blocked`, a blocked notice is shown.

### Status scenarios

- [ ] `newBase` target mode → `verified` status.
- [ ] `emptyExistingBase` with 0 tables and 0 records → `verified` status.
- [ ] `emptyExistingBase` with table count > 0 → `blocked` status.
- [ ] `emptyExistingBase` with record count > 0 → `blocked` status.
- [ ] `emptyExistingBase` with counts unknown → `warning` status.
- [ ] Unsupported target mode → `blocked` status.

### Safety invariants

- [ ] `noChangesMade` is `true` in every target empty verification result.
- [ ] `writesEnabled` is `false` in every target empty verification result.
- [ ] `networkWritesAttempted` is `false` in every target empty verification result.
- [ ] The verification request type has no `token` field.
- [ ] The verification result has no `token` field.
- [ ] The verification result contains no full filesystem path.
- [ ] A `verified` result does NOT enable restore writes.

---

## Restore Destructive Operation Policy Checklist (Gate 4)

### Before testing

- [ ] Confirm `verify_destructive_operation_policy_gate` is registered in the Tauri invoke handler.
- [ ] Confirm `RestoreDestructiveOperationPolicyPanel` is rendered on the Restore page after the target empty verification section.
- [ ] Confirm no execute button is present in the panel.
- [ ] Confirm no token input is present in the panel.

### Panel behavior

- [ ] A writes-disabled notice is always shown in the destructive operation policy section.
- [ ] A "Verify operation policy" button is shown before any result is available.
- [ ] Clicking the button calls `verifyDestructiveOperationPolicy` and updates the panel.
- [ ] While loading, the verify button is disabled and shows a loading label.
- [ ] After a result is returned, the button label changes to "Re-verify".

### Result display

- [ ] Status badge shows `compliant`, `warning`, or `blocked` correctly.
- [ ] The result message is shown beneath the status badge.
- [ ] The checks table shows one row per check (DOP-01 through DOP-05).
- [ ] Each check row shows the check ID, label, status badge, and message.
- [ ] A safety summary showing `noChangesMade`, `writesEnabled`, and `networkWritesAttempted` is shown.
- [ ] A "No changes have been made to Airtable" notice is shown.

### Status scenarios

- [ ] An empty declared operations list returns `compliant`.
- [ ] All create-only operations return `compliant`.
- [ ] Any `deleteTable`, `deleteField`, `deleteRecord`, or `deleteBase` operation returns `blocked`.
- [ ] Any `updateExistingRecord`, `overwriteField`, or `overwriteTable` operation returns `blocked`.
- [ ] Any `attachmentUpload` operation returns `blocked`.
- [ ] A blocked result shows the `dop-blocked-notice` notice.
- [ ] A compliant result shows the `dop-compliant-notice` notice, which says "writes remain disabled".
- [ ] A warning result shows the `dop-warning-notice` notice.

### Safety invariants

- [ ] `noChangesMade` is `true` in every destructive operation policy result.
- [ ] `writesEnabled` is `false` in every destructive operation policy result.
- [ ] `networkWritesAttempted` is `false` in every destructive operation policy result.
- [ ] The policy request type has no `token` field.
- [ ] The policy result has no `token` field.
- [ ] The policy result contains no full filesystem path.
- [ ] A `compliant` result does NOT enable restore writes.

---

## Restore Attachment Upload Policy Checklist (Gate 5)

### Before testing

- [ ] Confirm `verify_attachment_upload_policy_gate` is registered in the Tauri invoke handler.
- [ ] Confirm `RestoreAttachmentUploadPolicyPanel` is rendered on the Restore page after the destructive operation policy section.
- [ ] Confirm no execute button is present in the panel.
- [ ] Confirm no token input is present in the panel.

### Panel behavior

- [ ] A writes-disabled notice is always shown in the attachment upload policy section.
- [ ] A "Verify attachment policy" button is shown before any result is available.
- [ ] Clicking the button calls `verifyAttachmentUploadPolicy` and updates the panel.
- [ ] While loading, the verify button is disabled and shows a loading label.
- [ ] After a result is returned, the button label changes to "Re-verify".

### Result display

- [ ] Status badge shows `compliant`, `warning`, or `blocked` correctly.
- [ ] The result message is shown beneath the status badge.
- [ ] The checks table shows one row per check (AUP-01 through AUP-05).
- [ ] Each check row shows the check ID, label, status badge, and message.
- [ ] A safety summary showing `noChangesMade`, `writesEnabled`, `networkWritesAttempted`, and `metadataOnlyFields` is shown.
- [ ] A "No changes have been made to Airtable. Attachment file bytes have not been uploaded." notice is shown.

### Status scenarios

- [ ] An empty declared attachment fields list returns `compliant`.
- [ ] All `metadataOnly` fields return `compliant`.
- [ ] Any `uploadRequested` field returns `blocked`.
- [ ] Any `downloadRequested` field returns `warning` (not `blocked`).
- [ ] Any `unknown` field returns `warning` (not `blocked`).
- [ ] A blocked result shows the `aup-blocked-notice` notice.
- [ ] A compliant result shows the `aup-compliant-notice` notice, which says "writes remain disabled".
- [ ] A warning result shows the `aup-warning-notice` notice.

### Safety invariants

- [ ] `noChangesMade` is `true` in every attachment upload policy result.
- [ ] `writesEnabled` is `false` in every attachment upload policy result.
- [ ] `networkWritesAttempted` is `false` in every attachment upload policy result.
- [ ] The policy request type has no `token` field.
- [ ] The policy result has no `token` field.
- [ ] The policy result contains no full filesystem path.
- [ ] The policy result contains no full attachment URL (`dl.airtable.com`, `airtableusercontent.com`).
- [ ] A `compliant` result does NOT enable restore writes.
- [ ] Attachment file bytes are never uploaded by any code path in this gate.

---

## Restore Schema Record Order Policy Checklist (Gate 6)

### Before testing

- [ ] Confirm `verify_schema_record_order_policy_gate` is registered in the Tauri invoke handler.
- [ ] Confirm `RestoreSchemaRecordOrderPolicyPanel` is rendered on the Restore page after the attachment upload policy section.
- [ ] Confirm no execute button is present in the panel.
- [ ] Confirm no token input is present in the panel.

### Panel behavior

- [ ] A writes-disabled notice is always shown in the schema record order policy section.
- [ ] A "Verify phase ordering" button is shown before any result is available.
- [ ] Clicking the button calls `verifySchemaRecordOrderPolicy` and updates the panel.
- [ ] While loading, the verify button is disabled and shows a loading label.
- [ ] After a result is returned, the button label changes to "Re-verify".

### Result display

- [ ] Status badge shows `compliant`, `warning`, or `blocked` correctly.
- [ ] The result message is shown beneath the status badge.
- [ ] The checks table shows one row per check (SRO-01 through SRO-05).
- [ ] Each check row shows the check ID, label, status badge, and message.
- [ ] A safety summary showing `noChangesMade`, `writesEnabled`, and `networkWritesAttempted` is shown.
- [ ] A "No changes have been made to Airtable." notice is shown.
- [ ] When ordering violations are present, a violations list is shown with each violation string.
- [ ] When no violations are present, no violations list is shown.

### Status scenarios

- [ ] An empty declared phases list returns `warning`.
- [ ] A schema-only declared phase list returns `compliant`.
- [ ] A valid full phase order (schema → records → linkedRecords → attachments → validation) returns `compliant`.
- [ ] A record phase declared before schema returns `blocked` with `records-before-schema` violation.
- [ ] A missing schema with a declared record phase returns `blocked` with `missing-schema-with-records` violation.
- [ ] A blocked schema phase returns `blocked` with `schema-phase-blocked` violation.
- [ ] A linked-record phase before record-create returns `blocked` with `linked-before-record-create` violation.
- [ ] An attachment phase before record-create returns `blocked` with `attachment-before-record-create` violation.
- [ ] An unplanned schema phase with records returns `warning` (not `blocked`).
- [ ] A `compliant` result shows the `sro-compliant-notice` notice, which says "writes remain disabled".
- [ ] A `blocked` result shows the `sro-blocked-notice` notice.
- [ ] A `warning` result shows the `sro-warning-notice` notice.

### Safety invariants

- [ ] `noChangesMade` is `true` in every schema record order policy result.
- [ ] `writesEnabled` is `false` in every schema record order policy result.
- [ ] `networkWritesAttempted` is `false` in every schema record order policy result.
- [ ] The policy request type has no `token` field.
- [ ] The policy result has no `token` field.
- [ ] The policy result contains no full filesystem path.
- [ ] The policy result contains no raw record payload (no `fields`, no record IDs).
- [ ] A `compliant` result does NOT enable restore writes.

---

## Restore Sandbox Write Testing Policy Checklist (Gate 7)

### Before testing

- [ ] Confirm `verify_sandbox_write_testing_policy_gate` is registered in the Tauri invoke handler.
- [ ] Confirm `RestoreSandboxWriteTestingPolicyPanel` is rendered on the Restore page after the schema record order policy section.
- [ ] Confirm no execute button is present in the panel.
- [ ] Confirm no token input is present in the panel.

### Panel behavior

- [ ] A writes-disabled notice is always shown in the sandbox write testing policy section.
- [ ] A "Verify sandbox testing" button is shown before any result is available.
- [ ] Clicking the button calls `verifySandboxWriteTestingPolicy` and updates the panel.
- [ ] While loading, the verify button is disabled and shows a loading label.
- [ ] After a result is returned, the button label changes to "Re-verify".

### Result display

- [ ] Status badge shows `compliant`, `warning`, or `blocked` correctly.
- [ ] The result message is shown beneath the status badge.
- [ ] The checks table shows one row per check (SWT-01 through SWT-05).
- [ ] Each check row shows the check ID, label, status badge, and message.
- [ ] A safety summary showing `noChangesMade`, `writesEnabled`, and `networkWritesAttempted` is shown.
- [ ] A "No changes have been made to Airtable." notice is shown.

### Status scenarios

- [ ] A `sandbox` target with all evidence present and all evidence fields true returns `compliant`.
- [ ] A `production` target returns `blocked`.
- [ ] An `unknown` target returns `blocked`.
- [ ] Sandbox verification not passed returns `blocked`.
- [ ] No evidence declared returns `blocked`.
- [ ] Partial evidence (some fields false) returns `warning`.
- [ ] A filename with a path separator in evidence returns `warning`.
- [ ] A `compliant` result shows the `swt-compliant-notice` notice, which says "writes remain disabled".
- [ ] A `blocked` result shows the `swt-blocked-notice` notice.
- [ ] A `warning` result shows the `swt-warning-notice` notice.

### Safety invariants

- [ ] `noChangesMade` is `true` in every sandbox write testing policy result.
- [ ] `writesEnabled` is `false` in every sandbox write testing policy result.
- [ ] `networkWritesAttempted` is `false` in every sandbox write testing policy result.
- [ ] The policy request type has no `token` field.
- [ ] The policy result has no `token` field.
- [ ] The policy result contains no full filesystem path.
- [ ] Evidence filenames are basenames only — a path separator causes `Warning`.
- [ ] A `compliant` result does NOT enable restore writes.

---

## Restore Live Write Confirmation Policy Checklist (Gate 8)

### Before testing

- [ ] Confirm `verify_live_write_confirmation_policy_gate` is registered in the Tauri invoke handler.
- [ ] Confirm `RestoreLiveWriteConfirmationPolicyPanel` is rendered on the Restore page after the sandbox write testing policy section.
- [ ] Confirm no execute button is present in the panel.
- [ ] Confirm no token input field (`type="password"` or `name="token"`) is present.

### Panel behavior

- [ ] A writes-disabled notice is always shown in the live write confirmation policy section.
- [ ] The required confirmation phrase is shown before the input field.
- [ ] The verify button is disabled when the input is empty.
- [ ] Clicking the button with a non-empty input calls `verifyLiveWriteConfirmationPolicy` and updates the panel.
- [ ] While loading, the verify button is disabled and shows a loading label.

### Result display

- [ ] Status badge shows `confirmed`, `warning`, `blocked`, or `rejected` correctly.
- [ ] The result message is shown beside the status badge.
- [ ] The checks table shows one row per check (LWC-01 through LWC-05).
- [ ] Each check row shows the check ID, label, status badge, and message.
- [ ] A safety summary showing `noChangesMade`, `writesEnabled`, and `networkWritesAttempted` is shown.
- [ ] A "No changes made." notice is shown.

### Status scenarios

- [ ] Exact phrase match with all prior gates ok returns `confirmed`.
- [ ] Wrong text returns `rejected`.
- [ ] Lowercase version of correct phrase returns `rejected`.
- [ ] Extra words appended to correct phrase return `rejected`.
- [ ] A blocked prior gate (Gate 1–6) with correct text returns `blocked`.
- [ ] A blocked Gate 7 result with correct text returns `blocked`.
- [ ] A warning prior gate with correct text returns `warning`.
- [ ] A `confirmed` result shows the `lwc-confirmed-notice`, which says "writes remain disabled".
- [ ] A `blocked` result shows the `lwc-blocked-notice`.
- [ ] A `rejected` result shows the `lwc-rejected-notice`.

### Safety invariants

- [ ] `noChangesMade` is `true` in every live write confirmation policy result.
- [ ] `writesEnabled` is `false` in every live write confirmation policy result — including `confirmed`.
- [ ] `networkWritesAttempted` is `false` in every live write confirmation policy result.
- [ ] The policy request type has no `token` field.
- [ ] The policy result has no `token` field.
- [ ] The policy result contains no full filesystem path.
- [ ] The policy result contains no record payload field.
- [ ] A `confirmed` result does NOT enable restore writes.

---

## Restore Rate-Limit and Backoff Policy Checklist (Gate 9)

### Before testing

- [ ] Confirm `verify_rate_limit_backoff_policy_gate` is registered in the Tauri invoke handler.
- [ ] Confirm `RestoreRateLimitBackoffPolicyPanel` is rendered on the Restore page after the live write confirmation policy section.
- [ ] Confirm no execute button is present in the panel.
- [ ] Confirm no token input field (`type="password"` or `name="token"`) is present.

### Panel behavior

- [ ] A writes-disabled notice is always shown in the rate-limit and backoff policy section.
- [ ] The verify button is enabled and calls `verifyRateLimitBackoffPolicy` when clicked.
- [ ] While loading, the verify button is disabled and shows a loading label.

### Result display

- [ ] Status badge shows `compliant`, `warning`, or `blocked` correctly.
- [ ] The result message is shown beside the status badge.
- [ ] The checks table shows 10 check rows for a complete plan.
- [ ] The checks table shows 2 check rows when no plan is declared (short-circuit).
- [ ] Each check row shows the check ID, status badge, and message.
- [ ] Plan summary panel shows max RPS, batch size, 429 handling, max retries, backoff strategy, stop condition, and checkpoint compatibility when a plan is declared.
- [ ] Plan summary is not shown when no plan is declared.
- [ ] A safety summary showing `noChangesMade`, `writesEnabled`, and `networkWritesAttempted` is shown.
- [ ] A "No changes made." notice is shown.

### Status scenarios

- [ ] Safe plan (RPS ≤ 5, batch ≤ 10, 429 handled, retries bounded, backoff declared, stop declared, checkpoint full) returns `compliant`.
- [ ] No plan declared returns `blocked` with 2 checks.
- [ ] RPS = 6 returns `blocked`.
- [ ] Batch size = 11 returns `blocked`.
- [ ] `handles429: false` returns `blocked`.
- [ ] `maxRetries: undefined` (unbounded) returns `blocked`.
- [ ] `hasBackoffStrategy: false` returns `blocked`.
- [ ] `hasStopCondition: false` returns `blocked`.
- [ ] `checkpointCompatibility: "partial"` returns `warning` (not blocked).
- [ ] `checkpointCompatibility: "none"` returns `warning`.
- [ ] `checkpointCompatibility: "full"` returns `compliant`.
- [ ] A `compliant` result shows the `rlb-compliant-notice`, which says "writes remain disabled".
- [ ] A `warning` result shows the `rlb-warning-notice`.
- [ ] A `blocked` result shows the `rlb-blocked-notice`.

### Safety invariants

- [ ] `noChangesMade` is `true` in every rate-limit policy result.
- [ ] `writesEnabled` is `false` in every rate-limit policy result — including `compliant`.
- [ ] `networkWritesAttempted` is `false` in every rate-limit policy result.
- [ ] The policy request type has no `token` field.
- [ ] The policy result has no `token` field.
- [ ] The policy result contains no full filesystem path.
- [ ] The policy result contains no record payload field.
- [ ] A `compliant` result does NOT enable restore writes.

---

## Restore Checkpoint Durability Policy Checklist (Gate 10)

### Before testing

- [ ] Confirm `verify_checkpoint_durability_policy_gate` is registered in the Tauri invoke handler.
- [ ] Confirm `RestoreCheckpointDurabilityPolicyPanel` is rendered on the Restore page after the rate-limit and backoff policy section.
- [ ] Confirm no execute button is present in the panel.
- [ ] Confirm no token input field (`type="password"` or `name="token"`) is present.

### Panel behavior

- [ ] A writes-disabled notice is always shown in the checkpoint durability policy section.
- [ ] The verify button is enabled and calls `verifyCheckpointDurabilityPolicy` when clicked.
- [ ] While loading, the verify button is disabled and shows a loading label.

### Result display

- [ ] Status badge shows `compliant`, `warning`, or `blocked` correctly.
- [ ] The result message is shown beside the status badge.
- [ ] The checks table shows 9 check rows for a complete plan.
- [ ] The checks table shows 2 check rows when no plan is declared (short-circuit).
- [ ] Each check row shows the check ID, status badge, and message.
- [ ] Plan summary panel shows table checkpoint, batch checkpoint, phase markers, ID mapping checkpoint, resume stop condition, linked updates flag, and durability backend when a plan is declared.
- [ ] Plan summary is not shown when no plan is declared.
- [ ] A safety summary showing `noChangesMade`, `writesEnabled`, and `networkWritesAttempted` is shown.
- [ ] A "No changes made." notice is shown.

### Status scenarios

- [ ] Complete plan (all fields true, remote backend) returns `compliant`.
- [ ] No plan declared returns `blocked` with 2 checks.
- [ ] `checkpointAfterEachTable: false` returns `blocked`.
- [ ] `checkpointAfterEachBatch: false` returns `blocked`.
- [ ] `hasPhaseMarkers: false` returns `blocked`.
- [ ] `hasLinkedUpdates: true` with `hasIdMappingCheckpoint: false` returns `blocked`.
- [ ] `hasLinkedUpdates: false` with `hasIdMappingCheckpoint: false` returns `compliant` (ID mapping not required).
- [ ] `hasResumeSafeStopCondition: false` returns `blocked`.
- [ ] `durabilityBackend: "memory"` returns `warning` (not blocked).
- [ ] `durabilityBackend` not declared returns `warning`.
- [ ] `durabilityBackend: "remote"` returns `compliant`.
- [ ] A `compliant` result shows the `cdp-compliant-notice`, which says "writes remain disabled".
- [ ] A `warning` result shows the `cdp-warning-notice`.
- [ ] A `blocked` result shows the `cdp-blocked-notice`.

### Safety invariants

- [ ] `noChangesMade` is `true` in every checkpoint durability policy result.
- [ ] `writesEnabled` is `false` in every checkpoint durability policy result — including `compliant`.
- [ ] `networkWritesAttempted` is `false` in every checkpoint durability policy result.
- [ ] The policy request type has no `token` field.
- [ ] The policy result has no `token` field.
- [ ] The policy result contains no full filesystem path.
- [ ] The policy result contains no record payload field.
- [ ] A `compliant` result does NOT enable restore writes.

---

## Write Engine Skeleton Checklist

### Before testing

- [ ] Confirm `preview_restore_write_engine` is registered in the Tauri invoke handler.
- [ ] Confirm `RestoreWriteEnginePanel` is rendered on the Restore page.
- [ ] Confirm no Airtable credentials, no base ID, and no token appear in the write engine request.

### Write engine disabled notice

- [ ] The "Write engine" section is visible on the Restore page at all times.
- [ ] The disabled notice reads "Restore write execution is not enabled in this version."
- [ ] The disabled notice is always visible, regardless of whether a schema plan is ready.

### Write engine preview (after schema plan is ready)

- [ ] After the schema plan loads, the write engine skeleton preview appears.
- [ ] "No Airtable changes were made." is visible.
- [ ] Six phase rows are shown: validateInputs, schemaCreation, recordCreation, linkedRecordUpdates, attachmentHandling, finalValidation.
- [ ] All phase rows have disabled status.
- [ ] No execute button is shown.
- [ ] No token input is shown.
- [ ] No success message is shown.

### Safety invariants

- [ ] `noChangesMade` is `true` in every write engine result (confirmed via unit tests).
- [ ] The write engine request contains no token field.
- [ ] The write engine result contains no full path.
- [ ] The write engine result status is never `"succeeded"`.
- [ ] `evaluate_write_gate()` always returns `Disabled/DisabledByProductPolicy` (confirmed via unit tests).

---

## Schema Write Engine Foundation Checklist

### Before testing

- [ ] Confirm `preview_schema_write_request_plan` is registered in the Tauri invoke handler.
- [ ] Confirm `SchemaWriteRequestPlanRequest` has no `token` field in the TypeScript type definition.
- [ ] Confirm `SchemaWriteRequestPlanResult` has no `token` field and no `"succeeded"` status value.

### Request plan builder

- [ ] A request with `schemaPlanStatus: "ready"` and `tableCount > 0` returns a result with `status: "disabled"`.
- [ ] A request with `schemaPlanStatus: "blocked"` returns `status: "blocked"` with `blockedReason: "schemaPlanNotReady"`.
- [ ] A request with `tableCount: 0` returns `status: "blocked"` with `blockedReason: "noTablesInPlan"`.
- [ ] `tableOpCount` matches the `tableCount` in the request.
- [ ] `fieldOpCount` matches the `directFieldCount` in the request.
- [ ] `deferredOpCount` matches the `deferredFieldCount` in the request.
- [ ] `manualActionCount` matches the `manualActionCount` in the request.
- [ ] `totalOpCount` equals the sum of all four op count fields.

### Safety invariants

- [ ] `noChangesMade` is `true` in every schema write plan result (confirmed via unit tests).
- [ ] `networkWritesAttempted` is `false` in every schema write plan result (confirmed via unit tests).
- [ ] The request contains no `token` field — confirmed via `JSON.stringify(request)`.
- [ ] The result contains no `token` field — confirmed via `JSON.stringify(result)`.
- [ ] The result status is never `"succeeded"`.
- [ ] `evaluate_write_gate()` always returns `Disabled/DisabledByProductPolicy` (confirmed via unit tests).
- [ ] No Airtable table, field, or base is created at any point during plan generation.

---

## Record Write Engine Foundation Checklist

### Before testing

- [ ] Confirm `preview_record_write_request_plan` is registered in the Tauri invoke handler.
- [ ] Confirm `RecordWriteRequestPlanRequest` has no `token` field in the TypeScript type definition.
- [ ] Confirm `RecordWriteRequestPlanResult` has no `token` field, no `"succeeded"` status value, and no raw record payload fields.

### Request plan builder

- [ ] A request with `recordImportPlanStatus: "ready"` and `tableCount > 0` returns a result with `status: "disabled"`.
- [ ] A request with `recordImportPlanStatus: "blocked"` returns `status: "blocked"` with `blockedReason: "recordImportPlanNotReady"`.
- [ ] A request with `tableCount: 0` returns `status: "blocked"` with `blockedReason: "noTablesInPlan"`.
- [ ] `createBatchOpCount` matches the `totalFirstPassBatches` in the request.
- [ ] `linkedUpdateOpCount` matches the `totalSecondPassBatches` in the request.
- [ ] `checkpointOpCount` matches the `tableCount` in the request.
- [ ] `attachmentOpCount` matches the `attachmentFieldCount` in the request.
- [ ] `skippedFieldOpCount` matches the `skippedFieldCount` in the request.
- [ ] `totalOpCount` equals the sum of all five op count fields.

### Safety invariants

- [ ] `noChangesMade` is `true` in every record write plan result (confirmed via unit tests).
- [ ] `networkWritesAttempted` is `false` in every record write plan result (confirmed via unit tests).
- [ ] The request contains no `token` field — confirmed via `JSON.stringify(request)`.
- [ ] The result contains no `token` field — confirmed via `JSON.stringify(result)`.
- [ ] The result contains no raw record payloads — confirmed via `JSON.stringify(result)` scan.
- [ ] The result status is never `"succeeded"`.
- [ ] `evaluate_write_gate()` always returns `Disabled/DisabledByProductPolicy` (confirmed via unit tests).
- [ ] No Airtable records are created, updated, or deleted at any point during plan generation.
- [ ] `UpdateLinkedRecordBatch` operation notes state "ID mapping unavailable until execution".

---

## Live Write Safety Contract (Pre-Enable Gate)

Before any live Airtable write path is enabled, complete the separate checklist:

- [ ] See [live-restore-write-safety-checklist.md](./live-restore-write-safety-checklist.md) — all 15 gates must pass.
- [ ] Confirm `write_safety_contract.rs` tests still pass (`cargo test -- write_safety_contract`).
- [ ] Confirm write gate still returns `Disabled/DisabledByProductPolicy`.
- [ ] Confirm `Succeeded` status does not exist in any write engine status type.
- [ ] Confirm `noChangesMade` is still always `true` across all write result types.

---

## Notes and Failures

Record any failures here with a brief description and steps to reproduce.

| Item | Status | Notes |
|------|--------|-------|
| | | |
