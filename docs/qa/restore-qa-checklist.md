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

## Restore Final Validation Policy Checklist (Gate 11)

### Before testing

- [ ] Confirm `verify_final_validation_policy_gate` is registered in the Tauri invoke handler.
- [ ] Confirm `RestoreFinalValidationPolicyPanel` is rendered on the Restore page after the checkpoint durability policy section.
- [ ] Confirm no execute button is present in the panel.
- [ ] Confirm no token input field (`type="password"` or `name="token"`) is present.

### Panel behavior

- [ ] A writes-disabled notice is always shown in the final validation policy section.
- [ ] The verify button is enabled and calls `verifyFinalValidationPolicy` when clicked.
- [ ] While loading, the verify button is disabled and shows a loading label.

### Result display

- [ ] Status badge shows `compliant`, `warning`, or `blocked` correctly.
- [ ] The result message is shown beside the status badge.
- [ ] The checks table shows 12 check rows for a complete plan.
- [ ] The checks table shows 2 check rows when no plan is declared (short-circuit).
- [ ] Each check row shows the check ID, status badge, and message.
- [ ] Plan summary panel shows all 9 fields when a plan is declared: schema count validation, table/field validation, record count validation, ID mapping validation, linked record validation, attachment metadata validation, attachment validation scope, manifest checksum validation, and blocks-success-without-validation.
- [ ] Plan summary is not shown when no plan is declared.
- [ ] A safety summary showing `noChangesMade`, `writesEnabled`, and `networkWritesAttempted` is shown.
- [ ] A "No changes made." notice is shown.

### Status scenarios

- [ ] Complete plan (all boolean fields true, metadata-only false) returns `compliant`.
- [ ] No plan declared returns `blocked` with 2 checks.
- [ ] `hasSchemaCountValidation: false` returns `blocked`.
- [ ] `hasTableFieldValidation: false` returns `blocked`.
- [ ] `hasRecordCountValidation: false` returns `blocked`.
- [ ] `hasIdMappingValidation: false` returns `blocked`.
- [ ] `hasLinkedRecordValidation: false` returns `blocked`.
- [ ] `hasAttachmentMetadataValidation: false` returns `blocked`.
- [ ] `hasManifestChecksumValidation: false` returns `blocked`.
- [ ] `blocksSuccessWithoutValidation: false` returns `blocked`.
- [ ] `attachmentValidationMetadataOnly: true` returns `warning` (not blocked).
- [ ] A `compliant` result shows the `fvp-compliant-notice`, which says "writes remain disabled".
- [ ] A `warning` result shows the `fvp-warning-notice`.
- [ ] A `blocked` result shows the `fvp-blocked-notice`.

### Safety invariants

- [ ] `noChangesMade` is `true` in every final validation policy result.
- [ ] `writesEnabled` is `false` in every final validation policy result — including `compliant`.
- [ ] `networkWritesAttempted` is `false` in every final validation policy result.
- [ ] The policy request type has no `token` field.
- [ ] The policy result has no `token` field.
- [ ] The policy result contains no full filesystem path.
- [ ] The policy result contains no record payload field.
- [ ] A `compliant` result does NOT enable restore writes.
- [ ] A `compliant` result does NOT introduce a restore success state.

---

## Restore Write Phase Ordering Policy Checklist (Gate 12)

### Before testing

- [ ] Confirm `verify_write_phase_ordering_policy_gate` is registered in the Tauri invoke handler.
- [ ] Confirm `RestoreWritePhaseOrderingPolicyPanel` is rendered on the Restore page after the final validation policy section.
- [ ] Confirm no execute button is present in the panel.
- [ ] Confirm no token input field (`type="password"` or `name="token"`) is present.

### Panel behavior

- [ ] A writes-disabled notice is always shown in the write phase ordering policy section.
- [ ] The verify button is enabled and calls `verifyWritePhaseOrderingPolicy` when clicked.
- [ ] While loading, the verify button is disabled and shows a loading label.

### Result display

- [ ] Status badge shows `compliant`, `warning`, or `blocked` correctly.
- [ ] The result message is shown beside the status badge.
- [ ] The checks table shows 10 check rows for a complete canonical phase list.
- [ ] The checks table shows 2 check rows when no phase list is declared (short-circuit).
- [ ] Each check row shows the check ID, status badge, and message.
- [ ] Phase summary table shows one row per declared phase, including kind, status, canonical position, and skip reason.
- [ ] Phase summary is not shown when no phases are declared.
- [ ] A safety summary showing `noChangesMade`, `writesEnabled`, and `networkWritesAttempted` is shown.
- [ ] A "No changes made." notice is shown.

### Status scenarios

- [ ] Canonical phase list (all 9 phases, all completed) returns `compliant`.
- [ ] No phases declared returns `blocked` with 2 checks.
- [ ] Out-of-order phases return `blocked` (WPO-03 failed).
- [ ] `record_create` active without `schema_verify` completed returns `blocked` (WPO-05 failed).
- [ ] `linked_record_update` active without `record_verify` completed returns `blocked` (WPO-06 failed).
- [ ] `final_validation` active without `linked_record_verify` completed returns `blocked` (WPO-07 failed).
- [ ] Attachment upload language in a skip reason returns `blocked` (WPO-08 failed).
- [ ] `attachment_metadata_verify` skipped with metadata-only reason returns `warning` (WPO-09 warning).
- [ ] `attachment_metadata_verify` skipped without reason returns `warning`.
- [ ] A `compliant` result shows the `wpo-compliant-notice`, which says "writes remain disabled".
- [ ] A `warning` result shows the `wpo-warning-notice`.
- [ ] A `blocked` result shows the `wpo-blocked-notice`.

### Safety invariants

- [ ] `noChangesMade` is `true` in every write phase ordering policy result.
- [ ] `writesEnabled` is `false` in every write phase ordering policy result — including `compliant`.
- [ ] `networkWritesAttempted` is `false` in every write phase ordering policy result.
- [ ] The policy request type has no `token` field.
- [ ] The policy result has no `token` field.
- [ ] The policy result contains no full filesystem path.
- [ ] The policy result contains no record payload field.
- [ ] A `compliant` result does NOT enable restore writes.
- [ ] A `compliant` result does NOT introduce a restore success state.

---

## Restore Failure Modes Policy Checklist (Gate 13)

### Before testing

- [ ] Confirm `verify_failure_modes_policy_gate` is registered in the Tauri invoke handler.
- [ ] Confirm `RestoreFailureModesPolicyPanel` is rendered on the Restore page after the write phase ordering policy section.
- [ ] Confirm no execute button is present in the panel.
- [ ] Confirm no token input field (`type="password"` or `name="token"`) is present.

### Panel behavior

- [ ] A writes-disabled notice is always shown in the failure modes policy section.
- [ ] The verify button is enabled and calls `verifyFailureModesPolicy` when clicked.
- [ ] While loading, the verify button is disabled and shows a loading label.

### Result display

- [ ] Status badge shows `compliant`, `warning`, or `blocked` correctly.
- [ ] The result message is shown beside the status badge.
- [ ] The checks table shows 11 check rows for a complete handling plan.
- [ ] The checks table shows 2 check rows when no handling plans are declared (short-circuit).
- [ ] Each check row shows the check ID, status badge, and message.
- [ ] Handling summary table shows one row per declared failure mode, including mode name, stop behavior, preserves-checkpoint flag, and captures-diagnostic-context flag.
- [ ] Handling summary is shown when a handling plan is present.
- [ ] Handling summary is not shown when no plans are declared.
- [ ] A safety summary showing `noChangesMade`, `writesEnabled`, and `networkWritesAttempted` is shown.
- [ ] A "No changes made." notice is shown.

### Status scenarios

- [ ] Complete plan (all 10 required modes declared, all stop behaviors safe, no destructive rollback, no partial-failure-labeled-success, all with diagnostic context) returns `compliant`.
- [ ] No plans declared returns `blocked` with 2 checks.
- [ ] Any required failure mode missing returns `blocked` (FMP-03 failed).
- [ ] Any plan with `triggersDestructiveRollback: true` returns `blocked` (FMP-05 failed).
- [ ] `finalValidationFailure` with `partialFailureLabeledSuccess: true` returns `blocked` (FMP-08 failed).
- [ ] Any plan with `partialFailureLabeledSuccess: true` returns `blocked` (FMP-10 failed).
- [ ] A mode with `capturesDiagnosticContext: false` returns `warning` (FMP-W-{mode} warning added).
- [ ] A `compliant` result shows the `fmp-compliant-notice`, which says "writes remain disabled".
- [ ] A `warning` result shows the `fmp-warning-notice`.
- [ ] A `blocked` result shows the `fmp-blocked-notice`.

### Safety invariants

- [ ] `noChangesMade` is `true` in every failure modes policy result.
- [ ] `writesEnabled` is `false` in every failure modes policy result — including `compliant`.
- [ ] `networkWritesAttempted` is `false` in every failure modes policy result.
- [ ] The policy request type has no `token` field.
- [ ] The policy result has no `token` field.
- [ ] The policy result contains no full filesystem path.
- [ ] The policy result contains no record payload field.
- [ ] A `compliant` result does NOT enable restore writes.
- [ ] A `compliant` result does NOT introduce a restore success state.
- [ ] All four `FailureStopBehavior` variants (`stopAndReport`, `stopPreserveCheckpointAndReport`, `stopAfterRetryLimit`, `blockAndRequireManualReview`) stop writes — none permit continuation.

---

## Restore Rollback Limitation Policy Checklist (Gate 14)

### Before testing

- [ ] Confirm `verify_rollback_limitation_policy_gate` is registered in the Tauri invoke handler.
- [ ] Confirm `RestoreRollbackLimitationPolicyPanel` is rendered on the Restore page after the failure modes policy section.
- [ ] Confirm no execute button is present in the panel.
- [ ] Confirm no cleanup, delete-all, or revert button is present in the panel.
- [ ] Confirm no token input field (`type="password"` or `name="token"`) is present.

### Panel behavior

- [ ] A writes-disabled notice is always shown in the rollback limitation policy section.
- [ ] The notice mentions that automatic rollback is not available.
- [ ] The verify button is enabled and calls `verifyRollbackLimitationPolicy` when clicked.
- [ ] While loading, the verify button is disabled and shows a loading label.

### Result display

- [ ] Status badge shows `compliant`, `warning`, or `blocked` correctly.
- [ ] The result message is shown beside the status badge.
- [ ] A "Writes disabled" tag is always shown.
- [ ] The checks list shows 12 check rows for a complete plan.
- [ ] The checks list shows 2 check rows when no plan is declared (short-circuit).
- [ ] Each check row shows the check ID, status badge, label, and message.
- [ ] Remediation text is shown for any check with a `remediation` field.
- [ ] Plan summary section is shown when a plan is present.
- [ ] Plan summary shows `rollbackBehavior`, `partialRestoreIsNotSuccess`, `recoveryGuidanceDeclared`, `includesCheckpointGuidance`, `userVisibleNotice`, and `manualCleanupRequiresSeparateAction`.
- [ ] Plan summary is not shown when no plan is declared.
- [ ] A "No changes made" footer is shown.

### Status scenarios

- [ ] Safe plan (`noAutomaticRollback`, `partialRestoreIsNotSuccess: true`, `recoveryGuidance: checkpointBasedResume`, `userVisibleLimitationNotice: true`, `noticeIncludesLimitationDetails: true`, `manualCleanupRequiresSeparateAction: true`) returns `compliant`.
- [ ] No plan declared returns `blocked` with 2 checks.
- [ ] `rollbackBehavior: automaticDestructiveRollback` returns `blocked` (RLP-03 failed).
- [ ] `rollbackBehavior: automaticDeleteCleanup` returns `blocked` (RLP-04 failed).
- [ ] `rollbackBehavior: automaticUpdateRevertCleanup` returns `blocked` (RLP-05 failed).
- [ ] `partialRestoreIsNotSuccess: false` returns `blocked` (RLP-06 failed).
- [ ] `manualCleanupRequiresSeparateAction: false` returns `blocked` (RLP-09 failed).
- [ ] `recoveryGuidance: noneDeClared` returns `warning` (RLP-07 warning).
- [ ] `recoveryGuidance: manualCleanupRequired` (non-checkpoint) returns `warning` (RLP-07 warning).
- [ ] `userVisibleLimitationNotice: false` returns `warning` (RLP-08 warning).
- [ ] `userVisibleLimitationNotice: true` but `noticeIncludesLimitationDetails: false` returns `warning` (RLP-08 warning).
- [ ] A `compliant` result message says "writes remain disabled".
- [ ] A `blocked` result message says "writes remain disabled".

### Safety invariants

- [ ] `noChangesMade` is `true` in every rollback limitation policy result.
- [ ] `writesEnabled` is `false` in every rollback limitation policy result — including `compliant`.
- [ ] `networkWritesAttempted` is `false` in every rollback limitation policy result.
- [ ] The policy request type has no `token` field.
- [ ] The policy result has no `token` field.
- [ ] The policy result contains no full filesystem path.
- [ ] The policy result contains no record payload field.
- [ ] A `compliant` result does NOT enable restore writes.
- [ ] A `compliant` result does NOT introduce a restore success state.
- [ ] No automatic rollback, delete cleanup, or update/revert cleanup operation exists in the implementation.
- [ ] Manual cleanup requires a separate explicit future user action — not triggered by the restore engine.

---

## Restore Final Validation Enforcement Policy Checklist (Gate 15)

### Before testing

- [ ] Confirm `verify_final_validation_enforcement_policy_gate` is registered in the Tauri invoke handler.
- [ ] Confirm `RestoreFinalValidationEnforcementPolicyPanel` is rendered on the Restore page after the rollback limitation policy section.
- [ ] Confirm no execute button is present in the panel.
- [ ] Confirm no token input field (`type="password"` or `name="token"`) is present.

### Panel behavior

- [ ] A writes-disabled notice is always shown in the final validation enforcement policy section.
- [ ] The notice mentions that no result may be labeled complete or successful without final validation explicitly passing.
- [ ] The verify button is enabled and calls `verifyFinalValidationEnforcementPolicy` when clicked.
- [ ] While loading, the verify button is disabled and shows a loading label.

### Result display

- [ ] Status badge shows `compliant`, `warning`, or `blocked` correctly.
- [ ] The result message is shown beside the status badge.
- [ ] A "Writes disabled" tag is always shown.
- [ ] The checks list shows 15 check rows for a complete plan.
- [ ] The checks list shows 2 check rows when no plan is declared (short-circuit).
- [ ] Each check row shows the check ID, status badge, label, and message.
- [ ] Remediation text is shown for any check with a `remediation` field.
- [ ] Enforcement summary section is shown when the plan is present (non-short-circuit).
- [ ] Enforcement summary shows schema, record count, ID mapping, linked record, attachment, manifest validation states plus guard and completion guard flags.
- [ ] Enforcement summary is not shown when no plan is declared.
- [ ] A "No changes made" footer is shown.

### Status scenarios

- [ ] Complete safe plan (all required states `passed`, full completion guard) returns `compliant`.
- [ ] No plan declared returns `blocked` with 2 checks.
- [ ] Missing completion guard returns `blocked` (FVE-03 failed).
- [ ] Completion guard with any invariant `false` returns `blocked` (FVE-03 failed).
- [ ] `schemaValidationState: failed` returns `blocked` (FVE-04 failed).
- [ ] `recordCountValidationState: partial` returns `blocked` (FVE-05 failed).
- [ ] `idMappingValidationState: notDeclared` with linked validation needed returns `blocked` (FVE-06 failed).
- [ ] `linkedRecordValidationState: skipped` returns `blocked` (FVE-12 failed).
- [ ] `attachmentMetadataValidationState` with `attachmentValidationMetadataOnly: true` returns `warning` (FVE-08 warning).
- [ ] `attachmentMetadataValidationState: notRequired` with reason returns `warning` (FVE-08 warning).
- [ ] `attachmentMetadataValidationState: notRequired` without reason returns `blocked` (FVE-08 failed).
- [ ] `packageManifestPresent: false` skips manifest check (FVE-09 auto-pass).
- [ ] `packageManifestPresent: true` and `manifestChecksumValidationState: failed` returns `blocked` (FVE-09 failed).
- [ ] A `notRequired` state with a reason on a non-attachment field returns `warning`.
- [ ] A `compliant` result message says "writes remain disabled".
- [ ] A `blocked` result message says "no result may be labeled complete".

### Safety invariants

- [ ] `noChangesMade` is `true` in every final validation enforcement policy result.
- [ ] `writesEnabled` is `false` in every final validation enforcement policy result — including `compliant`.
- [ ] `networkWritesAttempted` is `false` in every final validation enforcement policy result.
- [ ] The policy request type has no `token` field.
- [ ] The policy result has no `token` field.
- [ ] The policy result contains no full filesystem path.
- [ ] The policy result contains no record payload field.
- [ ] A `compliant` result does NOT enable restore writes.
- [ ] A `compliant` result does NOT introduce a restore success state.
- [ ] No result in any status branch is labeled "complete" or "succeeded" before final validation passes.

---

## Restore Sensitive Data Safety Policy Checklist (Gate 16)

### Before testing

- [ ] Confirm `verify_sensitive_data_safety_policy_gate` is registered in the Tauri invoke handler.
- [ ] Confirm `RestoreSensitiveDataSafetyPolicyPanel` is rendered on the Restore page after the final validation enforcement policy section.
- [ ] Confirm no execute button is present in the panel.
- [ ] Confirm no token input field (`type="password"` or `name="token"`) is present.

### Panel behavior

- [ ] A writes-disabled notice is always shown in the sensitive data safety policy section.
- [ ] The notice mentions that sensitive material must never be exposed through any restore write surface.
- [ ] The verify button is enabled and calls `verifySensitiveDataSafetyPolicy` when clicked.
- [ ] While loading, the verify button is disabled and shows a loading label.

### Result display

- [ ] Status badge shows `compliant`, `warning`, or `blocked` correctly.
- [ ] The result message is shown beside the status badge.
- [ ] A "Writes disabled" tag is always shown.
- [ ] The checks list shows 15 check rows for a complete plan.
- [ ] The checks list shows 2 check rows when no plan is declared (short-circuit).
- [ ] Each check row shows the check ID, status badge, label, and message.
- [ ] Remediation text is shown for any check with a `remediation` field.
- [ ] Safety summary section is shown when the plan is present (non-short-circuit).
- [ ] Safety summary shows surfaces covered count, total redaction rules, all-rules-named flag, and all 8 safety boolean flags.
- [ ] Safety summary is not shown when no plan is declared.
- [ ] A "No changes made" footer is shown.

### Status scenarios

- [ ] Complete safe plan (all flags `true`, all 10 surfaces covered, all rules named) returns `compliant`.
- [ ] No plan declared returns `blocked` with 2 checks.
- [ ] Missing any of the 10 required exposure surfaces returns `blocked` (SDS-03 failed).
- [ ] `noTokenInResults: false` returns `blocked` (SDS-04 failed).
- [ ] `noFullPathInResults: false` returns `blocked` (SDS-05 failed).
- [ ] `packageReferencesFilenameOnly: false` returns `blocked` (SDS-06 failed).
- [ ] `noRecordPayloadInResults: false` returns `blocked` (SDS-07 failed).
- [ ] `noAttachmentUrlInResults: false` returns `blocked` (SDS-08 failed).
- [ ] `noRawHttpInResults: false` returns `blocked` (SDS-09 failed).
- [ ] `errorMessagesUseSafeSummaries: false` returns `blocked` (SDS-10 failed).
- [ ] `summariesArePayloadFree: false` returns `blocked` (SDS-11 failed).
- [ ] Unnamed redaction rules return `warning` (SDS-12 warning — not blocked).
- [ ] A `compliant` result message says "writes remain disabled".
- [ ] A `blocked` result message says "sensitive material must never be exposed".

### Safety invariants

- [ ] `noChangesMade` is `true` in every sensitive data safety policy result.
- [ ] `writesEnabled` is `false` in every sensitive data safety policy result — including `compliant`.
- [ ] `networkWritesAttempted` is `false` in every sensitive data safety policy result.
- [ ] The policy request type has no `token` field.
- [ ] The policy result has no `token` field.
- [ ] The policy result contains no full filesystem path.
- [ ] The policy result contains no package path.
- [ ] The policy result contains no record payload field.
- [ ] The policy result contains no attachment URL field.
- [ ] The policy result contains no raw HTTP data field.
- [ ] A `compliant` result does NOT enable restore writes.
- [ ] A `compliant` result does NOT introduce a restore success state.

---

## Restore Attachment Phase Disabled Policy Checklist (Gate 17)

### Before testing

- [ ] Confirm `verify_attachment_phase_disabled_policy_gate` is registered in the Tauri invoke handler.
- [ ] Confirm `RestoreAttachmentPhaseDisabledPolicyPanel` is rendered on the Restore page after the sensitive data safety policy section.
- [ ] Confirm no execute button is present in the panel.
- [ ] Confirm no token input field (`type="password"` or `name="token"`) is present.
- [ ] Confirm no binary download, upload, URL fetch, or transfer button is present.

### Panel behavior

- [ ] A writes-disabled notice is always shown in the attachment phase disabled policy section.
- [ ] The notice mentions that binary attachment download, upload, fetch, and transfer are not permitted.
- [ ] A metadata-only notice is always shown, stating that attachment handling is metadata-only.
- [ ] The verify button is enabled and calls `verifyAttachmentPhaseDisabledPolicy` when clicked.
- [ ] While loading, the verify button is disabled and shows a loading label.

### Result display

- [ ] Status badge shows `compliant`, `warning`, or `blocked` correctly.
- [ ] The result message is shown beside the status badge.
- [ ] A "Writes disabled" tag is always shown.
- [ ] A "Metadata only" tag is always shown.
- [ ] The checks list shows 16 check rows for a complete plan.
- [ ] The checks list shows 2 check rows when no plan is declared (short-circuit).
- [ ] Each check row shows the check ID, status badge, label, and message.
- [ ] Remediation text is shown for any check with a `remediation` field.
- [ ] Phase summary section is shown when the plan is present (non-short-circuit).
- [ ] Phase summary shows all 8 boolean flags and blocked-operations count.
- [ ] Phase summary is not shown when no plan is declared.
- [ ] Operation class table is shown listing all 10 operations with permitted/blocked badges.
- [ ] A "No changes made" footer is shown.

### Status scenarios

- [ ] Complete plan (all flags set correctly) returns `compliant`.
- [ ] No plan declared returns `blocked` with 2 checks.
- [ ] `metadataInspectionEnabled: false` returns `blocked` (APD-03 failed).
- [ ] `metadataVerificationEnabled: false` without skip reason returns `blocked` (APD-04 failed).
- [ ] `metadataVerificationEnabled: false` with skip reason returns `warning` (APD-04 warning — not blocked).
- [ ] `binaryHandlingDisabled: false` returns `blocked` (APD-05 through APD-10 failed).
- [ ] `fieldMutationDisabled: false` returns `blocked` (APD-11 failed).
- [ ] `urlExposureDisabled: false` returns `blocked` (APD-12 failed).
- [ ] `phaseRequiredForCompletionDisabled: false` returns `blocked` (APD-13 failed).
- [ ] `finalValidationTreatsAsMetadataOnly: false` returns `blocked` (APD-14 failed).
- [ ] Declared binary operation with `planned: true` returns `blocked` (APD-15 failed).
- [ ] Declared binary operation with `requiredForCompletion: true` returns `blocked` (APD-16 failed).
- [ ] A `compliant` result message says "writes remain disabled".
- [ ] A `blocked` result message says "binary attachment operations must remain disabled".

### Safety invariants

- [ ] `noChangesMade` is `true` in every attachment phase disabled policy result.
- [ ] `writesEnabled` is `false` in every attachment phase disabled policy result — including `compliant`.
- [ ] `networkWritesAttempted` is `false` in every attachment phase disabled policy result.
- [ ] The policy request type has no `token` field.
- [ ] The policy result has no `token` field.
- [ ] The policy result contains no attachment binary data.
- [ ] The policy result contains no attachment URL field.
- [ ] The policy result contains no record payload field.
- [ ] A `compliant` result does NOT enable restore writes.
- [ ] A `compliant` result does NOT introduce a restore success state.
- [ ] A `compliant` result does NOT download, upload, fetch, or transfer any attachment binary.

---

## Restore Live Write Readiness Policy Checklist (Gate 18)

### Before testing

- [ ] Confirm `verify_live_write_readiness_policy_gate` is registered in the Tauri invoke handler.
- [ ] Confirm `RestoreLiveWriteReadinessPolicyPanel` is rendered on the Restore page after the attachment phase disabled policy section.
- [ ] Confirm no execute button is present in the panel.
- [ ] Confirm no enable-writes button is present.
- [ ] Confirm no token input field is present.

### Panel behavior

- [ ] A writes-disabled notice is always shown.
- [ ] The notice states that verifying this policy does not enable writes or start any restore operation.
- [ ] An advisory-only notice is always shown, stating that a Ready result does not enable write execution.
- [ ] The notice states that restore completion remains unavailable.
- [ ] The verify button is enabled and calls `verifyLiveWriteReadinessPolicy` when clicked.
- [ ] While loading, the verify button is disabled and shows a loading label.

### Result display

- [ ] Status badge shows `ready (advisory only)`, `warning`, or `blocked` correctly.
- [ ] The result message is shown beside the status badge.
- [ ] A "Writes disabled" tag is always shown.
- [ ] An "Advisory only" tag is always shown.
- [ ] The checks list shows 10 check rows for a complete gates array.
- [ ] The checks list shows 2 check rows when no gates are declared (short-circuit).
- [ ] Each check row shows the check ID, status badge, label, and message.
- [ ] Remediation text is shown for any check with a `remediation` field.
- [ ] Gate summary section is shown when gates are declared.
- [ ] Gate summary shows total gates, passed, warning, failed, not-evaluated, missing, all-declared, and live-execution-available.
- [ ] Gate summary is not shown when no gates are declared.
- [ ] A "No changes made" footer is shown.
- [ ] The footer mentions advisory only.

### Status scenarios

- [ ] All 17 required gates declared and passed returns `ready`.
- [ ] No gates declared returns `blocked` with 2 checks.
- [ ] Any required gate missing returns `blocked`.
- [ ] Any required gate with `failed` status returns `blocked` (LWR-03 failed).
- [ ] Any required gate with `notEvaluated` status returns `blocked` (LWR-08 failed).
- [ ] `liveExecutionAvailable: true` returns `blocked` (LWR-05 failed).
- [ ] Gate note with success-equivalent wording returns `blocked` (LWR-06 failed).
- [ ] Gate note with token material returns `blocked` (LWR-07 failed).
- [ ] Gate note with full path returns `blocked` (LWR-07 failed).
- [ ] Warning gate produces `warning` (LWR-04 warning — not blocked).
- [ ] A `ready` result message says "writes remain disabled".
- [ ] A `ready` result message says "advisory only".
- [ ] A `blocked` result message says "writes remain disabled".

### Safety invariants

- [ ] `noChangesMade` is `true` in every live write readiness result.
- [ ] `writesEnabled` is `false` in every live write readiness result — including `ready`.
- [ ] `networkWritesAttempted` is `false` in every live write readiness result.
- [ ] The policy request type has no `token` field.
- [ ] The policy result has no `token` field.
- [ ] The policy result contains no attachment URL field.
- [ ] The policy result contains no record payload field.
- [ ] A `ready` result does NOT enable restore writes.
- [ ] A `ready` result does NOT introduce a restore success state.
- [ ] A `ready` result does NOT start any restore operation.

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

## Schema Write Execution Preview Checklist

### Before testing

- [ ] Confirm `preview_schema_write_execution_gate` is registered in the Tauri invoke handler.
- [ ] Confirm `SchemaWriteExecutionPreviewRequest` has no `token` field.
- [ ] Confirm `SchemaWriteExecutionPreviewResult` has no `token`, `writesEnabled: true`, or `"succeeded"` status.

### During testing

- [ ] Missing sandbox prerequisite returns `blocked`.
- [ ] Missing target empty prerequisite returns `blocked`.
- [ ] Missing schema plan prerequisite returns `blocked`.
- [ ] Unsafe destructive policy returns `blocked`.
- [ ] Unsafe sensitive data returns `blocked`.
- [ ] Attachment phase not disabled returns `blocked`.
- [ ] Missing final validation enforcement returns `blocked`.
- [ ] Missing live write readiness returns `blocked`.
- [ ] All prerequisites satisfied returns `dryRunReady`.
- [ ] `dryRunReady` result has ordered steps: validate → tables → direct fields → deferred → manual → post-check.
- [ ] Table steps appear before field steps in the ordered list.
- [ ] `writesEnabled` is `false` in every result.
- [ ] `noChangesMade` is `true` in every result.
- [ ] `networkWritesAttempted` is `false` in every result.
- [ ] No token field appears in the serialized result.
- [ ] No absolute path appears in the serialized result.
- [ ] No record payload appears in the serialized result.
- [ ] No attachment URL appears in the serialized result.
- [ ] `DryRunReady` message explicitly states live schema writes remain disabled.
- [ ] `DryRunReady` message explicitly states the preview does not start any restore execution.
- [ ] `evaluate_write_gate()` still returns `Disabled/DisabledByProductPolicy` after calling the preview.
- [ ] No Airtable base, table, or field is created at any point.
- [ ] UI panel shows writes-disabled notice at all times.
- [ ] UI panel has no execute button, no enable-writes button, and no token input.
- [ ] Record write execution, linked record second pass, checkpoint execution, and final validation execution remain pending/unavailable.

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

- [ ] See [live-restore-write-safety-checklist.md](./live-restore-write-safety-checklist.md) — all 25 gates must pass.
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

---

## Final Validation Execution Preview Checklist

### Before testing

- [ ] Confirm `preview_final_validation_execution_gate` is registered in the Tauri invoke handler.
- [ ] Confirm `FinalValidationExecutionPreviewRequest` has no `token` field.
- [ ] Confirm `FinalValidationExecutionPreviewResult` has no `token`, `writesEnabled: true`, `"succeeded"` status, old/new record IDs, raw record payload, attachment URL, or raw HTTP body.

### During testing

- [ ] Missing schema write preview prerequisite returns `blocked` with FVEP-PRE-02 reason.
- [ ] Missing record write preview prerequisite returns `blocked` with FVEP-PRE-03 reason.
- [ ] Missing mapping/checkpoint preview prerequisite returns `blocked` with FVEP-PRE-04 reason.
- [ ] Missing linked second-pass preview prerequisite returns `blocked` with FVEP-PRE-05 reason.
- [ ] Missing final validation policy safe prerequisite returns `blocked` with FVEP-PRE-06 reason.
- [ ] Missing final validation enforcement policy safe prerequisite returns `blocked` with FVEP-PRE-07 reason.
- [ ] Missing sensitive data safe prerequisite returns `blocked` with FVEP-PRE-08 reason.
- [ ] Missing attachment phase disabled safe prerequisite returns `blocked` with FVEP-PRE-09 reason.
- [ ] Missing live write readiness prerequisite returns `blocked` with FVEP-PRE-10 reason.
- [ ] All 10 prerequisites satisfied returns `dryRunReady`.
- [ ] `dryRunReady` result contains exactly 8 ordered checks: FVEP-CHK-SCHEMA, FVEP-CHK-FIELDS, FVEP-CHK-RECORDS, FVEP-CHK-MAPPING, FVEP-CHK-LINKED, FVEP-CHK-ATTACH, FVEP-CHK-MANIFEST, FVEP-CHK-GUARD.
- [ ] FVEP-CHK-MANIFEST is `skipped` when `manifestPresent` is false.
- [ ] FVEP-CHK-MANIFEST is `pending` when `manifestPresent` is true.
- [ ] All other checks are `pending` when `dryRunReady`.
- [ ] Check `expectedCount` values reflect the corresponding request count fields.
- [ ] Summary `pendingCheckCount` and `nonPendingCheckCount` are correct.
- [ ] `writesEnabled` is `false` in every result.
- [ ] `noChangesMade` is `true` in every result.
- [ ] `networkWritesAttempted` is `false` in every result.
- [ ] No token field appears in the serialized result.
- [ ] No absolute path appears in the serialized result.
- [ ] No old or new record ID appears in the serialized checks or summary.
- [ ] No raw record payload or field values appear in the serialized result.
- [ ] No attachment URL appears in the serialized result.
- [ ] No raw HTTP request or response body appears in the serialized result.
- [ ] `DryRunReady` message explicitly states live final validation execution remains disabled.
- [ ] `DryRunReady` message explicitly states the preview does not start any restore execution.
- [ ] `DryRunReady` message explicitly states no checkpoint files are written.
- [ ] `DryRunReady` message explicitly states no record IDs are present.
- [ ] Panel renders `data-testid="restore-fvep-panel"`.
- [ ] Panel renders execution-disabled notice.
- [ ] Panel shows `data-testid="fvep-dry-run-badge"` when `dryRunReady`.
- [ ] Panel shows `data-testid="fvep-blocked-badge"` when `blocked`.
- [ ] Panel shows `data-testid="fvep-execution-disabled-tag"` whenever result is present.
- [ ] Panel shows `data-testid="fvep-no-changes-made"` whenever result is present.
- [ ] Panel has no execute button, no enable button, no token input.

---

## Checkpoint Metadata Store Checklist

### Before testing

- [ ] Confirm `store_restore_checkpoint_metadata` is registered in the Tauri invoke handler.
- [ ] Confirm `RestoreCheckpointStoreRequest` has no `token` field, no `output_path` field, no record payload field, no old/new record ID field, no attachment URL field.
- [ ] Confirm `RestoreCheckpointStoreResult` has no `token`, no full path, no old/new record IDs, no record payload, no raw HTTP body, no attachment URL, no `"succeeded"` status, no `writesEnabled: true`.

### During testing

- [ ] Missing `checkpointDurabilitySafe` prerequisite returns `blocked` with RCPS-PRE-02 reason.
- [ ] Missing `sensitiveDataSafe` prerequisite returns `blocked` with RCPS-PRE-03 reason.
- [ ] Missing `mappingCheckpointPreviewReady` prerequisite returns `blocked` with RCPS-PRE-04 reason.
- [ ] Missing `finalValidationPreviewReady` prerequisite returns `blocked` with RCPS-PRE-05 reason.
- [ ] All prerequisites satisfied → `stored` result with `summary` present.
- [ ] `writesEnabled` is `false` in every result.
- [ ] `networkWritesAttempted` is `false` in every result.
- [ ] `noChangesMade` is `true` in every blocked result.
- [ ] `noChangesMade` is `false` in a stored result.
- [ ] `summary.safeFilename` has no path separator (`/` or `\`).
- [ ] `summary.safeFilename` starts with `rcps-` and ends with `.json`.
- [ ] No token (`pat_`) in the serialized result.
- [ ] No absolute path in the serialized result.
- [ ] No old or new record ID in the serialized result.
- [ ] No record payload or field values in the serialized result.
- [ ] No attachment URL in the serialized result.
- [ ] No raw HTTP request or response body in the serialized result.
- [ ] Stored result message explicitly states restore execution is not triggered.
- [ ] Stored result message explicitly states live restore writes remain disabled.
- [ ] Stored result message explicitly states no sensitive data is stored.
- [ ] Stored checkpoint file on disk declares `restoreExecutionNotTriggered: true`.
- [ ] Stored checkpoint file on disk declares `noSensitiveData: true`.
- [ ] Checkpoint label with path traversal characters (e.g. `../`) is sanitized — no separator appears in filename.
- [ ] Panel renders `data-testid="restore-checkpoint-store-panel"`.
- [ ] Panel renders restore-not-triggered notice.
- [ ] Panel renders `data-testid="rcps-metadata-only-badge"`.
- [ ] Panel shows `data-testid="rcps-stored-badge"` when stored.
- [ ] Panel shows `data-testid="rcps-blocked-badge"` when blocked.
- [ ] Panel shows `data-testid="rcps-restore-not-triggered-tag"` when result is present.
- [ ] Panel shows `data-testid="rcps-summary"` with safe filename and counts when stored.
- [ ] Panel shows `data-testid="rcps-writes-disabled"` when result is present.
- [ ] Panel has no execute button, no enable button, no token input.
- [ ] Panel does not display full checkpoint directory path.
- [ ] Panel does not display old or new record IDs.

---

## Schema Write Executor Foundation Checklist (internal module)

### Before testing

- [ ] Confirm `build_schema_write_executor_plan` is internal only — no Tauri command is registered for it.
- [ ] Confirm `SchemaWriteExecutorRequest` has no `token` field, no `output_path` field, no record payload field, no old/new record ID field, no attachment URL field.
- [ ] Confirm `SchemaWriteExecutorResult` has no `token`, no full path, no old/new record IDs, no record payload, no raw HTTP body, no attachment URL, no `"succeeded"` status, no `writesEnabled: true`.
- [ ] Confirm no UI execute button or panel calls `build_schema_write_executor_plan`.
- [ ] Confirm `SchemaWriteExecutorMode` has only `disabled` and `sandboxOnly` — no `production` mode.

### During testing (Rust unit tests only)

- [ ] Mode `disabled` returns `blocked` with SWEX-PRE-02 reason.
- [ ] Explicit internal write flag not set returns `blocked` with SWEX-PRE-03 reason.
- [ ] Sandbox not verified returns `blocked` with SWEX-PRE-04 reason.
- [ ] Target not empty returns `blocked` with SWEX-PRE-05 reason.
- [ ] Live write readiness not satisfied returns `blocked` with SWEX-PRE-06 reason.
- [ ] Request plan blocked returns `blocked` with SWEX-PRE-07 reason.
- [ ] All prerequisites satisfied → `notExecuted` result (write gate still disabled).
- [ ] `writesEnabled` is `false` in every result.
- [ ] `networkWritesAttempted` is `false` in every result.
- [ ] `noChangesMade` is `true` in every result.
- [ ] No token (`pat_`) in the serialized result.
- [ ] No absolute path in the serialized result.
- [ ] No `"succeeded"`, `"restoreComplete"`, or `"restoreSuccess"` in any result.
- [ ] Steps are built in canonical order: tables first, then direct fields, then deferred linked fields, then manual actions.
- [ ] All steps have status `pending` when `notExecuted`.
- [ ] Step IDs use stable prefixes: `SWEX-TBL-`, `SWEX-FLD-`, `SWEX-DEF-`, `SWEX-MAN-`.
- [ ] `safety_snapshot.write_gate_disabled` is always `true`.
- [ ] `evaluate_write_gate()` returns `Disabled` before and after any executor call.
- [ ] Run `cargo test -- schema_write_executor::tests` — all tests pass.

---

## Record Write Executor Foundation Checklist (internal module)

### Before testing

- [ ] Confirm `build_record_write_executor_plan` is internal only — no Tauri command is registered for it.
- [ ] Confirm `RecordWriteExecutorRequest` has no `token` field, no `output_path` field, no record payload field, no old/new record ID field, no attachment URL field.
- [ ] Confirm `RecordWriteExecutorResult` has no `token`, no full path, no old/new record IDs, no record payload, no raw HTTP body, no attachment URL, no `"succeeded"` status, no `writesEnabled: true`.
- [ ] Confirm no UI execute button or panel calls `build_record_write_executor_plan`.
- [ ] Confirm `RecordWriteExecutorMode` has only `disabled` and `sandboxOnly` — no `production` mode.

### During testing (Rust unit tests only)

- [ ] Mode `disabled` returns `blocked` with RWEX-PRE-02 reason.
- [ ] Explicit internal write flag not set returns `blocked` with RWEX-PRE-03 reason.
- [ ] Sandbox not verified returns `blocked` with RWEX-PRE-04 reason.
- [ ] Target not empty returns `blocked` with RWEX-PRE-05 reason.
- [ ] Schema executor not safe returns `blocked` with RWEX-PRE-06 reason.
- [ ] Rate-limit/backoff not safe returns `blocked` with RWEX-PRE-07 reason.
- [ ] Checkpoint store not safe returns `blocked` with RWEX-PRE-08 reason.
- [ ] Live write readiness not satisfied returns `blocked` with RWEX-PRE-09 reason.
- [ ] Request plan blocked returns `blocked`.
- [ ] All prerequisites satisfied → `notExecuted` result (write gate still disabled).
- [ ] `writesEnabled` is `false` in every result.
- [ ] `networkWritesAttempted` is `false` in every result.
- [ ] `noChangesMade` is `true` in every result.
- [ ] No token (`pat_`) in the serialized result.
- [ ] No absolute path in the serialized result.
- [ ] No `"succeeded"`, `"restoreComplete"`, or `"restoreSuccess"` in any result.
- [ ] Batches are ordered: first-pass create batches before second-pass linked-update batches.
- [ ] Batch indices are sequential (0-based, no gaps).
- [ ] No batch record_count exceeds 10.
- [ ] `safety_snapshot.write_gate_disabled` is always `true`.
- [ ] `evaluate_write_gate()` returns `Disabled` before and after any executor call.
- [ ] Run `cargo test -- record_write_executor::foundation_tests` — all tests pass.

---

## Linked Second-Pass Executor Foundation Checklist (internal module)

### Before testing

- [ ] Confirm `build_linked_second_pass_executor_plan` is internal only — no Tauri command is registered for it.
- [ ] Confirm `LinkedSecondPassExecutorRequest` has no `token` field, no `output_path` field, no record payload field, no old/new record ID field, no attachment URL field.
- [ ] Confirm `LinkedSecondPassExecutorResult` has no `token`, no full path, no old/new record IDs, no record payload, no raw HTTP body, no attachment URL, no `"succeeded"` status, no `writesEnabled: true`.
- [ ] Confirm no UI execute button or panel calls `build_linked_second_pass_executor_plan`.
- [ ] Confirm `LinkedSecondPassExecutorMode` has only `disabled` and `sandboxOnly` — no `production` mode.

### During testing (Rust unit tests only)

- [ ] Mode `disabled` returns `blocked` with LSEX-PRE-02 reason.
- [ ] Explicit internal flag not set returns `blocked` with LSEX-PRE-03 reason.
- [ ] Sandbox not verified returns `blocked` with LSEX-PRE-04 reason.
- [ ] Target not empty returns `blocked` with LSEX-PRE-05 reason.
- [ ] Record executor not safe returns `blocked` with LSEX-PRE-06 reason.
- [ ] Linked second-pass preview not ready returns `blocked` with LSEX-PRE-07 reason.
- [ ] Linked second-pass preview status `Blocked` returns `blocked` with LSEX-PRE-07 reason.
- [ ] Mapping/checkpoint preview not ready returns `blocked` with LSEX-PRE-08 reason.
- [ ] Sensitive data not safe returns `blocked` with LSEX-PRE-09 reason.
- [ ] Live write readiness not satisfied returns `blocked` with LSEX-PRE-10 reason.
- [ ] Batch size > 10 returns `blocked`.
- [ ] All prerequisites satisfied → `notExecuted` result (write gate still disabled).
- [ ] Unresolved optional links are warning-safe when preview returned DryRunReady.
- [ ] `writesEnabled` is `false` in every result.
- [ ] `networkWritesAttempted` is `false` in every result.
- [ ] `noChangesMade` is `true` in every result.
- [ ] No token (`pat_`) in the serialized result.
- [ ] No absolute path in the serialized result.
- [ ] No old or new record ID in the serialized result.
- [ ] No `"succeeded"`, `"restoreComplete"`, or `"restoreSuccess"` in any result.
- [ ] Batch indices are sequential (0-based, no gaps).
- [ ] Field ordering is preserved from field_summaries.
- [ ] No batch update_count exceeds 10.
- [ ] `safety_snapshot.write_gate_disabled` is always `true`.
- [ ] Run `cargo test -- linked_second_pass_executor::tests` — all tests pass.

---

## Linked Second-Pass Execution Preview Checklist

### Before testing

- [ ] Confirm `preview_linked_second_pass_execution_gate` is registered in the Tauri invoke handler.
- [ ] Confirm `LinkedSecondPassExecutionPreviewRequest` has no `token` field.
- [ ] Confirm `LinkedSecondPassExecutionPreviewResult` has no `token`, `writesEnabled: true`, `"succeeded"` status, old/new record IDs, raw record payload, attachment URL, or raw HTTP body.

### During testing

- [ ] Missing record write preview prerequisite returns `blocked` with LSEP-PRE-02 reason.
- [ ] Missing mapping/checkpoint preview prerequisite returns `blocked` with LSEP-PRE-03 reason.
- [ ] Missing write phase ordering safe prerequisite returns `blocked`.
- [ ] Missing checkpoint durability safe prerequisite returns `blocked`.
- [ ] Missing sensitive data safe prerequisite returns `blocked`.
- [ ] Missing final validation enforcement prerequisite returns `blocked`.
- [ ] Missing live write readiness prerequisite returns `blocked`.
- [ ] Batch size > 10 returns `blocked`.
- [ ] All 8 prerequisites satisfied returns `dryRunReady`.
- [ ] Unresolved links produce a non-zero count in `mappingSummary.unresolvedLinkCount` — status is still `dryRunReady`.
- [ ] Batch `updateCount` never exceeds `batchSize` (≤ 10).
- [ ] Batch ordering is deterministic across repeated calls with the same input.
- [ ] `writesEnabled` is `false` in every result.
- [ ] `noChangesMade` is `true` in every result.
- [ ] `networkWritesAttempted` is `false` in every result.
- [ ] No token field appears in the serialized result.
- [ ] No absolute path appears in the serialized result.
- [ ] No old or new record ID appears in the serialized result.
- [ ] No raw record payload or field values appear in the serialized result.
- [ ] No attachment URL appears in the serialized result.
- [ ] No raw HTTP request or response body appears in the serialized result.
- [ ] `DryRunReady` message explicitly states live linked record updates remain disabled.
- [ ] `DryRunReady` message explicitly states the preview does not start any restore execution.
- [ ] `DryRunReady` message explicitly states no checkpoint files are written.
- [ ] `DryRunReady` message explicitly states no record IDs are present.
- [ ] Panel renders `data-testid="restore-lsep-panel"`.
- [ ] Panel renders execution-disabled notice.
- [ ] Panel shows `data-testid="lsep-dry-run-badge"` when `dryRunReady`.
- [ ] Panel shows `data-testid="lsep-blocked-badge"` when `blocked`.
- [ ] Panel shows `data-testid="lsep-execution-disabled-tag"` whenever result is present.
- [ ] Panel shows `data-testid="lsep-no-changes-made"` whenever result is present.
- [ ] Panel has no execute button, no enable button, no token input.

---

## Mapping Checkpoint Execution Preview Checklist

### Before testing

- [ ] Confirm `preview_mapping_checkpoint_execution_gate` is registered in the Tauri invoke handler.
- [ ] Confirm `MappingCheckpointExecutionPreviewRequest` has no `token` field.
- [ ] Confirm `MappingCheckpointExecutionPreviewResult` has no `token`, `writesEnabled: true`, `"succeeded"` status, raw record payload, record IDs, attachment URL, or raw HTTP body.

### During testing

- [ ] Missing record write preview prerequisite returns `blocked` with MCEP-PRE-02 reason.
- [ ] Missing checkpoint durability safe prerequisite returns `blocked`.
- [ ] Missing failure modes safe prerequisite returns `blocked`.
- [ ] Missing rollback limitation safe prerequisite returns `blocked`.
- [ ] Missing final validation enforcement prerequisite returns `blocked`.
- [ ] Missing sensitive data safe prerequisite returns `blocked`.
- [ ] Missing live write readiness prerequisite returns `blocked`.
- [ ] All 8 prerequisites satisfied returns `dryRunReady`.
- [ ] `dryRunReady` result has step `MCEP-CHK-SCHEMA` first.
- [ ] `dryRunReady` result has step `MCEP-CHK-PRE-FV` last.
- [ ] `MCEP-MAP-REC-B{n}` steps appear before `MCEP-CHK-PRE-LINK`.
- [ ] `MCEP-CHK-PRE-LINK` step is absent when first-pass batch count is 0.
- [ ] Checkpoint summary `totalCheckpointCount` equals 3 + firstPassBatches + secondPassBatches.
- [ ] `writesEnabled` is `false` in every result.
- [ ] `noChangesMade` is `true` in every result.
- [ ] `networkWritesAttempted` is `false` in every result.
- [ ] No token field appears in the serialized result.
- [ ] No absolute path appears in the serialized result.
- [ ] No record ID or field value appears in the serialized result.
- [ ] No attachment URL appears in the serialized result.
- [ ] No raw HTTP request or response body appears in the serialized result.
- [ ] `DryRunReady` message explicitly states live mapping capture and checkpoint persistence remain disabled.
- [ ] `DryRunReady` message explicitly states the preview does not start any restore execution.
- [ ] `DryRunReady` message explicitly states no checkpoint files are written.
- [ ] Panel renders `data-testid="restore-mcep-panel"`.
- [ ] Panel renders execution-disabled notice.
- [ ] Panel shows `data-testid="mcep-dry-run-badge"` when `dryRunReady`.
- [ ] Panel shows `data-testid="mcep-blocked-badge"` when `blocked`.
- [ ] Panel shows `data-testid="mcep-execution-disabled-tag"` whenever result is present.
- [ ] Panel shows `data-testid="mcep-no-changes-made"` whenever result is present.
- [ ] Panel has no execute button, no enable button, no token input.

---

## Record Write Execution Preview Checklist

### Before testing

- [ ] Confirm `preview_record_write_execution_gate` is registered in the Tauri invoke handler.
- [ ] Confirm `RecordWriteExecutionPreviewRequest` has no `token` field.
- [ ] Confirm `RecordWriteExecutionPreviewResult` has no `token`, `writesEnabled: true`, `"succeeded"` status, raw field values, raw HTTP body, or attachment URL.

### During testing

- [ ] Missing schema preview prerequisite returns `blocked` with RWEP-PRE-02 reason.
- [ ] Missing sandbox prerequisite returns `blocked`.
- [ ] Missing target empty prerequisite returns `blocked`.
- [ ] Missing record import plan returns `blocked`.
- [ ] Missing record write request plan returns `blocked`.
- [ ] Batch size > 10 returns `blocked` with RWEP-PRE-07 reason.
- [ ] Batch size 0 returns `blocked`.
- [ ] Unsafe rate-limit/backoff policy returns `blocked`.
- [ ] Unsafe checkpoint durability policy returns `blocked`.
- [ ] Unsafe sensitive data policy returns `blocked`.
- [ ] Attachment phase not disabled returns `blocked`.
- [ ] Missing final validation enforcement returns `blocked`.
- [ ] Missing live write readiness returns `blocked`.
- [ ] All 13 prerequisites satisfied returns `dryRunReady`.
- [ ] `dryRunReady` result has ordered batches: first-pass create batches before second-pass linked-update batches.
- [ ] Batch record count never exceeds batch size.
- [ ] `writesEnabled` is `false` in every result.
- [ ] `noChangesMade` is `true` in every result.
- [ ] `networkWritesAttempted` is `false` in every result.
- [ ] No token field appears in the serialized result.
- [ ] No absolute path appears in the serialized result.
- [ ] No raw record payload or field values appear in the serialized result.
- [ ] No attachment URL appears in the serialized result.
- [ ] No raw HTTP request or response body appears in the serialized result.
- [ ] `DryRunReady` message explicitly states live record writes remain disabled.
- [ ] `DryRunReady` message explicitly states the preview does not start any restore execution.
- [ ] Panel renders `data-testid="restore-rwep-panel"`.
- [ ] Panel renders "Live record writes disabled" notice.
- [ ] Panel shows `data-testid="rwep-dry-run-badge"` when `dryRunReady`.
- [ ] Panel shows `data-testid="rwep-blocked-badge"` when `blocked`.
- [ ] Panel shows `data-testid="rwep-writes-disabled-tag"` whenever result is present.
- [ ] Panel shows `data-testid="rwep-no-changes-made"` whenever result is present.
- [ ] Panel has no execute button, no enable button, no token input.

---

## Final Validation Reader Foundation Checklist (internal module)

### Before testing

- [ ] Confirm `build_final_validation_reader_plan` is internal only — no Tauri command is registered for it.
- [ ] Confirm `FinalValidationReaderRequest` has no `token` field, no `output_path` field, no record payload field, no old/new record ID field, no attachment URL field.
- [ ] Confirm `FinalValidationReaderResult` has no `token`, no full path, no old/new record IDs, no record payload, no raw HTTP body, no attachment URL, no `"succeeded"` status, no `readsEnabled: true`, no `writesEnabled: true`.
- [ ] Confirm no UI execute button or panel calls `build_final_validation_reader_plan`.
- [ ] Confirm `FinalValidationReaderMode` has only `disabled` and `sandboxOnly` — no `production` mode.

### During testing (Rust unit tests only)

- [ ] Mode `disabled` returns `blocked` with FVRD-PRE-02 reason.
- [ ] Explicit internal read flag not set returns `blocked` with FVRD-PRE-03 reason.
- [ ] Sandbox not verified returns `blocked` with FVRD-PRE-04 reason.
- [ ] Schema executor not safe returns `blocked` with FVRD-PRE-05 reason.
- [ ] Record executor not safe returns `blocked` with FVRD-PRE-06 reason.
- [ ] Linked executor not safe returns `blocked` with FVRD-PRE-07 reason.
- [ ] Final validation preview not ready returns `blocked` with FVRD-PRE-08 reason.
- [ ] Enforcement policy not safe returns `blocked` with FVRD-PRE-09 reason.
- [ ] Sensitive data not safe returns `blocked` with FVRD-PRE-10 reason.
- [ ] Attachment phase not safe returns `blocked` with FVRD-PRE-11 reason.
- [ ] All prerequisites satisfied → `notExecuted` result (validation read gate still disabled).
- [ ] `readsEnabled` is `false` in every result.
- [ ] `writesEnabled` is `false` in every result.
- [ ] `networkReadsAttempted` is `false` in every result.
- [ ] `networkWritesAttempted` is `false` in every result.
- [ ] `noChangesMade` is `true` in every result.
- [ ] No token (`pat_`) in the serialized result.
- [ ] No absolute path in the serialized result.
- [ ] No old or new record ID in the serialized result.
- [ ] No raw record payload or field values in the serialized result.
- [ ] No attachment URL in the serialized result.
- [ ] No `"succeeded"`, `"restoreComplete"`, or `"restoreSuccess"` in any result.
- [ ] Checks are ordered: FVRD-CHK-SCHEMA first, FVRD-CHK-GUARD last.
- [ ] FVRD-CHK-MANIFEST is `skipped` when `manifest_present` is false.
- [ ] FVRD-CHK-MANIFEST is `pending` when `manifest_present` is true.
- [ ] FVRD-CHK-ATTACH note does not mention binary retrieval.
- [ ] `total_check_count` equals `checks.len()`.
- [ ] `pending_check_count` equals the number of checks with status `pending`.
- [ ] `expected_count` in each check reflects the corresponding request count field.
- [ ] `safety_snapshot.read_gate_disabled` is always `true`.
- [ ] `safety_snapshot.write_gate_disabled` is always `true`.
- [ ] `evaluate_write_gate()` returns `Disabled` before and after any reader call.
- [ ] Run `cargo test -- final_validation_reader::tests` — all tests pass.

---

## Restore Orchestrator Foundation Checklist (internal module)

### Before testing

- [ ] Confirm `build_restore_orchestrator_plan` is internal only — no Tauri command is registered for it.
- [ ] Confirm `RestoreOrchestratorRequest` has no `token` field, no `output_path` field, no record payload field, no old/new record ID field, no attachment URL field.
- [ ] Confirm `RestoreOrchestratorResult` has no `token`, no full path, no old/new record IDs, no record payload, no raw HTTP body, no attachment URL, no `"succeeded"` status, no `writesEnabled: true`, no `readsEnabled: true`.
- [ ] Confirm no UI execute button or panel calls `build_restore_orchestrator_plan`.
- [ ] Confirm `RestoreOrchestratorMode` has only `disabled` and `sandboxOnly` — no `production` mode.

### During testing (Rust unit tests only)

- [ ] Mode `disabled` returns `blocked` with ORCH-PRE-02 reason.
- [ ] Sandbox not verified returns `blocked` with ORCH-PRE-03 reason.
- [ ] Target not empty returns `blocked` with ORCH-PRE-04 reason.
- [ ] Write phase ordering unsafe returns `blocked` with ORCH-PRE-05 reason.
- [ ] Failure modes unsafe returns `blocked` with ORCH-PRE-06 reason.
- [ ] Rollback limitation unsafe returns `blocked` with ORCH-PRE-07 reason.
- [ ] Live write readiness not safe returns `blocked` with ORCH-PRE-08 reason.
- [ ] Schema executor not safe returns `blocked` with ORCH-PRE-09 reason.
- [ ] Record executor not safe returns `blocked` with ORCH-PRE-10 reason.
- [ ] Linked executor not safe returns `blocked` with ORCH-PRE-11 reason.
- [ ] Final validation reader not safe returns `blocked` with ORCH-PRE-12 reason.
- [ ] All prerequisites satisfied → `notExecuted` result (write gate still disabled).
- [ ] `writesEnabled` is `false` in every result.
- [ ] `readsEnabled` is `false` in every result.
- [ ] `networkReadsAttempted` is `false` in every result.
- [ ] `networkWritesAttempted` is `false` in every result.
- [ ] `noChangesMade` is `true` in every result.
- [ ] No token (`pat_`) in the serialized result.
- [ ] No absolute path in the serialized result.
- [ ] No old or new record ID in the serialized result.
- [ ] No raw record payload or field values in the serialized result.
- [ ] No attachment URL in the serialized result.
- [ ] No `"succeeded"`, `"restoreComplete"`, or `"restoreSuccess"` in any result.
- [ ] Phases are ordered: ORCH-PH-01 (schema executor) first, ORCH-PH-08 (final guard) last.
- [ ] Checkpoint boundaries follow their respective executors (ORCH-PH-02 after ORCH-PH-01, etc.).
- [ ] `total_phase_count` equals `phases.len()` (8).
- [ ] `pending_phase_count` equals the number of phases with status `pending`.
- [ ] `safety_snapshot.write_gate_disabled` is always `true`.
- [ ] `evaluate_write_gate()` returns `Disabled` before and after any orchestrator call.
- [ ] Run `cargo test -- restore_orchestrator::tests` — all tests pass.

---

## Sandbox Gate Contract Foundation Checklist (internal module)

### Before testing

- [ ] Confirm `evaluate_sandbox_gate_contract` is internal only — no Tauri command is registered for it.
- [ ] Confirm `SandboxGateContractRequest` has no `token` field, no `output_path` field, no record payload field, no old/new record ID field, no attachment URL field.
- [ ] Confirm `SandboxGateContractResult` has no `token`, no full path, no old/new record IDs, no record payload, no raw HTTP body, no attachment URL, no `"armed"` or `"enabled"` or `"succeeded"` status, no `writesEnabled: true`, no `readsEnabled: true`.
- [ ] Confirm no UI execute button or panel calls `evaluate_sandbox_gate_contract`.
- [ ] Confirm `SandboxGateContractMode` has only `disabled` and `sandboxOnlyCandidate` — no `production` mode.
- [ ] Confirm `SandboxGateContractStatus` has only `disabled`, `blocked`, and `eligibleButNotArmed` — no `armed`, `enabled`, `succeeded`, `complete`, or `done`.
- [ ] Confirm `evaluate_write_gate()` is never modified by this module.

### During testing (Rust unit tests only)

- [ ] Mode `disabled` returns `disabled` status with empty prerequisites.
- [ ] Sandbox not safe returns `blocked` with SGC-PRE-01 in `blocked_reason`.
- [ ] Target not empty returns `blocked` with SGC-PRE-02 in `blocked_reason`.
- [ ] Confirmation gate not declared returns `blocked` with SGC-PRE-03 in `blocked_reason`.
- [ ] Destructive operation policy unsafe returns `blocked` with SGC-PRE-04 in `blocked_reason`.
- [ ] Attachment phase not safe returns `blocked` with SGC-PRE-05 in `blocked_reason`.
- [ ] Live write readiness not safe returns `blocked` with SGC-PRE-06 in `blocked_reason`.
- [ ] Orchestrator not present returns `blocked` with SGC-PRE-07 in `blocked_reason`.
- [ ] Schema executor not present returns `blocked` with SGC-PRE-08 in `blocked_reason`.
- [ ] Record executor not present returns `blocked` with SGC-PRE-09 in `blocked_reason`.
- [ ] Linked executor not present returns `blocked` with SGC-PRE-10 in `blocked_reason`.
- [ ] Final validation reader not present returns `blocked` with SGC-PRE-11 in `blocked_reason`.
- [ ] All prerequisites satisfied → `eligibleButNotArmed` result (gate NOT armed, gate NOT enabled).
- [ ] `eligibleButNotArmed` result message explicitly says "NOT armed" and "NOT enabled".
- [ ] `writesEnabled` is `false` in every result.
- [ ] `readsEnabled` is `false` in every result.
- [ ] `networkReadsAttempted` is `false` in every result.
- [ ] `networkWritesAttempted` is `false` in every result.
- [ ] `noChangesMade` is `true` in every result.
- [ ] `safety_snapshot.write_gate_disabled` is always `true`.
- [ ] `total_prereq_count` equals `prerequisites.len()` (12 when all satisfied).
- [ ] Prerequisite ordering is deterministic (SGC-PRE-01 first, SGC-PRE-12 last).
- [ ] All prerequisite IDs use the `SGC-PRE-` prefix.
- [ ] No token (`pat_`) in the serialized result.
- [ ] No absolute path in the serialized result.
- [ ] No old or new record ID in the serialized result.
- [ ] No raw record payload or field values in the serialized result.
- [ ] No attachment URL in the serialized result.
- [ ] No `"armed"`, `"enabled"`, `"succeeded"`, `"restoreComplete"`, or `"restoreSuccess"` in any serialized result.
- [ ] No `"production"` in mode serialization.
- [ ] `evaluate_write_gate()` returns `Disabled` before and after any gate contract call.
- [ ] Run `cargo test -- sandbox_gate_contract::tests` — all tests pass.

---

## Sandbox Restore Harness Foundation Checklist (internal module)

### Before testing

- [ ] Confirm `build_sandbox_restore_harness_plan` is internal only — no Tauri command is registered for it.
- [ ] Confirm `SandboxRestoreHarnessRequest` has no `token` field, no `output_path` field, no record payload field, no old/new record ID field, no attachment URL field.
- [ ] Confirm `SandboxRestoreHarnessResult` has no `token`, no full path, no old/new record IDs, no record payload, no raw HTTP body, no attachment URL, no `"armed"` or `"enabled"` or `"succeeded"` status, no `gate_armed: true`, no `writesEnabled: true`, no `readsEnabled: true`.
- [ ] Confirm no UI execute button or panel calls `build_sandbox_restore_harness_plan`.
- [ ] Confirm `SandboxRestoreHarnessMode` has only `disabled` and `sandboxOnlyDryHarness` — no `production` mode.
- [ ] Confirm `SandboxRestoreHarnessStatus` has only `blocked`, `readyNotExecuted`, and `notExecuted` — no `armed`, `enabled`, `succeeded`, `complete`, or `done`.
- [ ] Confirm `evaluate_write_gate()` is never modified by this module.
- [ ] Confirm the harness does not arm the gate — `gate_armed` is always `false`.

### During testing (Rust unit tests only)

- [ ] Mode `disabled` returns `notExecuted` status with no evaluation performed.
- [ ] Sandbox not safe returns `blocked`.
- [ ] Target not empty returns `blocked`.
- [ ] Confirmation gate not declared returns `blocked`.
- [ ] Destructive operation policy unsafe returns `blocked`.
- [ ] Attachment phase unsafe returns `blocked`.
- [ ] Write phase ordering unsafe returns `blocked`.
- [ ] Failure modes unsafe returns `blocked`.
- [ ] Rollback limitation unsafe returns `blocked`.
- [ ] All prerequisites satisfied → `readyNotExecuted` (gate NOT armed, gate NOT enabled).
- [ ] `readyNotExecuted` result message explicitly says "NOT armed" and "NOT enabled".
- [ ] `readyNotExecuted` result message says live execution "remains pending".
- [ ] `gate_armed` is `false` in every result.
- [ ] `writesEnabled` is `false` in every result.
- [ ] `readsEnabled` is `false` in every result.
- [ ] `networkReadsAttempted` is `false` in every result.
- [ ] `networkWritesAttempted` is `false` in every result.
- [ ] `noChangesMade` is `true` in every result.
- [ ] `safety_snapshot.write_gate_disabled` is always `true`.
- [ ] `safety_snapshot.gate_armed` is always `false`.
- [ ] `total_phase_count` equals `phases.len()` (8 when ready).
- [ ] Phase ordering is deterministic (SRH-PH-01 first, SRH-PH-08 last).
- [ ] All phase IDs use the `SRH-PH-` prefix.
- [ ] No token (`pat_`) in the serialized result.
- [ ] No absolute path in the serialized result.
- [ ] No old or new record ID in the serialized result.
- [ ] No raw record payload or field values in the serialized result.
- [ ] No attachment URL in the serialized result.
- [ ] No `"armed"`, `"enabled"`, `"succeeded"`, `"restoreComplete"`, or `"restoreSuccess"` in any serialized result.
- [ ] No `"production"` in mode serialization.
- [ ] `evaluate_write_gate()` returns `Disabled` before and after any harness call.
- [ ] Run `cargo test -- sandbox_restore_harness::tests` — all tests pass.

## Sandbox Enablement Readiness Report Checklist (internal module)

### Before testing

- [ ] Confirm `build_sandbox_enablement_readiness_report` is internal only — no Tauri command is registered for it.
- [ ] Confirm `SandboxEnablementReadinessRequest` has no `token` field, no `output_path` field, no record payload field, no old/new record ID field, no attachment URL field.
- [ ] Confirm `SandboxEnablementReadinessResult` has no `token`, no full path, no old/new record IDs, no record payload, no raw HTTP body, no attachment URL, no `"armed"` or `"enabled"` or `"succeeded"` status, no `gate_armed: true`, no `writesEnabled: true`, no `readsEnabled: true`.
- [ ] Confirm no UI execute button or panel calls `build_sandbox_enablement_readiness_report`.
- [ ] Confirm `SandboxEnablementReadinessStatus` has only `notReady`, `readyButDisabled`, and `blocked` — no `armed`, `enabled`, `succeeded`, `complete`, or `done`.
- [ ] Confirm `evaluate_write_gate()` is the first call and always returns `Disabled`.
- [ ] Confirm the report does not arm the gate — `gate_armed` is always `false`.
- [ ] Confirm the report has exactly 13 items (SERN-01 through SERN-13).

### During testing (Rust unit tests only)

- [ ] All prerequisites satisfied → `readyButDisabled` (gate NOT armed, gate NOT enabled).
- [ ] `readyButDisabled` result message explicitly says "NOT armed" and "NOT enabled".
- [ ] `readyButDisabled` result message says future enablement "remains separate pending work".
- [ ] `sandbox_verification_safe: false` → `notReady`.
- [ ] `target_empty_safe: false` → `notReady`.
- [ ] `confirmation_gate_declared: false` → `notReady`.
- [ ] `write_phase_ordering_safe: false` → `notReady`.
- [ ] `failure_modes_safe: false` → `notReady`.
- [ ] `rollback_limitation_safe: false` → `notReady`.
- [ ] `gate_armed` is `false` in every result.
- [ ] `writesEnabled` is `false` in every result.
- [ ] `readsEnabled` is `false` in every result.
- [ ] `networkReadsAttempted` is `false` in every result.
- [ ] `networkWritesAttempted` is `false` in every result.
- [ ] `noChangesMade` is `true` in every result.
- [ ] `safety_snapshot.write_gate_disabled` is always `true`.
- [ ] `safety_snapshot.gate_armed` is always `false`.
- [ ] SERN-01 (write gate default) is `ready` when all prereqs satisfied.
- [ ] SERN-02 (gate contract eligible) is `ready` when all prereqs satisfied.
- [ ] SERN-03 (harness readyNotExecuted) is `ready` when all prereqs satisfied.
- [ ] SERN-04 (orchestrator notExecuted) is `ready` when all prereqs satisfied.
- [ ] SERN-05 (orchestrator 8 phases) is `ready` when all prereqs satisfied.
- [ ] SERN-06 through SERN-09 (executor foundations) are `ready` when all prereqs satisfied.
- [ ] SERN-10 (checkpoint store sanitized) is `ready` when all prereqs satisfied.
- [ ] SERN-11, SERN-12, SERN-13 (safety invariants) are always `ready`.
- [ ] `total_item_count` is 13 when all prereqs satisfied.
- [ ] `ready_item_count` equals 13 when all prereqs satisfied.
- [ ] Item ordering is deterministic (SERN-01 first, SERN-13 last).
- [ ] All item IDs use the `SERN-` prefix.
- [ ] No token (`pat_`) in the serialized result.
- [ ] No absolute path in the serialized result.
- [ ] No old or new record ID in the serialized result.
- [ ] No raw record payload or field values in the serialized result.
- [ ] No attachment URL in the serialized result.
- [ ] No `"armed"`, `"enabled"`, `"succeeded"`, `"restoreComplete"`, or `"restoreSuccess"` in any serialized result.
- [ ] `evaluate_write_gate()` returns `Disabled` before and after any report call.
- [ ] Run `cargo test -- sandbox_enablement_readiness::tests` — all tests pass.

## Sandbox Gate Arming Model Checklist (internal module, not persisted)

### Before testing

- [ ] Confirm `build_sandbox_gate_arming_decision` is internal only — no Tauri command is registered for it.
- [ ] Confirm `SandboxGateArmingRequest` has no `token` field, no `output_path` field, no record payload field, no old/new record ID field, no attachment URL field.
- [ ] Confirm `SandboxGateArmingResult` has no `token`, no full path, no old/new record IDs, no record payload, no raw HTTP body, no attachment URL, no `"enabled"` or `"succeeded"` or `"executionReady"` status, no `executionEnabled: true`, no `writesEnabled: true`, no `readsEnabled: true`.
- [ ] Confirm no UI execute button or panel calls `build_sandbox_gate_arming_decision`.
- [ ] Confirm `SandboxGateArmingMode` has only `disabled` and `sandboxOnlyInternal` — no `production` mode.
- [ ] Confirm `SandboxGateArmingStatus` has only `blocked` and `armedNotExecutable` — no `enabled`, `succeeded`, `complete`, `executionReady`, or `done`.
- [ ] Confirm `evaluate_write_gate()` is never modified by this module.
- [ ] Confirm the decision is not stored globally — each call produces an independent result.
- [ ] Confirm `executionEnabled` is always `false`.
- [ ] Confirm `writesEnabled` is always `false`.
- [ ] Confirm `readsEnabled` is always `false`.

### During testing (Rust unit tests only)

- [ ] Mode `disabled` returns `blocked`.
- [ ] `explicit_internal_sandbox_arming_requested: false` returns `blocked` with SGA-CHK-02 reason.
- [ ] Sandbox verification not safe returns `blocked`.
- [ ] Target not empty returns `blocked`.
- [ ] Confirmation gate not declared returns `blocked`.
- [ ] Write phase ordering unsafe returns `blocked`.
- [ ] Failure modes unsafe returns `blocked`.
- [ ] Rollback limitation unsafe returns `blocked`.
- [ ] Readiness not `readyButDisabled` returns `blocked` with SGA-CHK-04 reason.
- [ ] All prerequisites satisfied → `armedNotExecutable` (execution NOT enabled, writes NOT enabled, reads NOT enabled).
- [ ] `armedNotExecutable` result message explicitly says "NOT enabled".
- [ ] `armedNotExecutable` result message says "not stored globally".
- [ ] `armedNotExecutable` result message says live execution "remains separate pending work".
- [ ] `gate_armed` is `true` only when status is `armedNotExecutable`.
- [ ] `gate_armed` is `false` in every `blocked` result.
- [ ] `executionEnabled` is `false` in every result.
- [ ] `writesEnabled` is `false` in every result.
- [ ] `readsEnabled` is `false` in every result.
- [ ] `networkReadsAttempted` is `false` in every result.
- [ ] `networkWritesAttempted` is `false` in every result.
- [ ] `noChangesMade` is `true` in every result.
- [ ] `safety_snapshot.write_gate_disabled` is always `true`.
- [ ] `safety_snapshot.execution_enabled` is always `false`.
- [ ] `safety_snapshot.writes_enabled` is always `false`.
- [ ] `safety_snapshot.reads_enabled` is always `false`.
- [ ] `evaluate_write_gate()` returns `Disabled` before and after any arming call.
- [ ] Two independent calls produce independent results — no global state is shared.
- [ ] An `armedNotExecutable` call followed by a `blocked` call — the second call is still `blocked`.
- [ ] No token (`pat_`) in the serialized result.
- [ ] No absolute path in the serialized result.
- [ ] No old or new record ID in the serialized result.
- [ ] No raw record payload or field values in the serialized result.
- [ ] No attachment URL in the serialized result.
- [ ] No `"enabled"`, `"succeeded"`, `"executionReady"`, `"restoreComplete"`, or `"restoreSuccess"` in any serialized result.
- [ ] Run `cargo test -- sandbox_gate_arming::tests` — all tests pass.

## Sandbox Restore Simulator Checklist (internal module, in-memory only)

### Before testing

- [ ] Confirm `run_sandbox_restore_simulator` is internal only — no Tauri command is registered for it.
- [ ] Confirm `SandboxRestoreSimulatorRequest` has no `token` field, no `output_path` field, no record payload field, no old/new record ID field, no attachment URL field.
- [ ] Confirm `SandboxRestoreSimulatorResult` has no `token`, no full path, no old/new record IDs, no record payload, no raw HTTP body, no attachment URL, no `"succeeded"` or `"executionReady"` or `"enabled"` status, no `executionEnabled: true`, no `writesEnabled: true`, no `readsEnabled: true`, no `gateArmed: true` (runtime/global).
- [ ] Confirm no UI execute button or panel calls `run_sandbox_restore_simulator`.
- [ ] Confirm `SandboxRestoreSimulatorMode` has only `disabled` and `sandboxOnlyInternalSimulation` — no `production` mode.
- [ ] Confirm `SandboxRestoreSimulatorStatus` has only `blocked` and `simulatedNotExecuted` — no `succeeded`, `complete`, `executionReady`, `enabled`, or `done`.
- [ ] Confirm `SandboxRestoreSimulatorPhaseStatus` has no `succeeded`, `complete`, or `done`.
- [ ] Confirm `evaluate_write_gate()` is never modified by this module.
- [ ] Confirm the result is not stored globally — each call produces an independent result.
- [ ] Confirm `airtableClientCalled` is always `false`.
- [ ] Confirm `checkpointFileWritten` is always `false`.

### During testing (Rust unit tests only)

- [ ] Mode `disabled` returns `blocked` with SRS-CHK-01.
- [ ] `explicit_internal_simulation_requested: false` returns `blocked` with SRS-CHK-02.
- [ ] Arming decision `blocked` returns simulator `blocked` with SRS-CHK-04.
- [ ] Harness `blocked` returns simulator `blocked` with SRS-CHK-05.
- [ ] Orchestrator `blocked` returns simulator `blocked` with SRS-CHK-06.
- [ ] All prerequisites satisfied → `simulatedNotExecuted`.
- [ ] `simulatedNotExecuted` message says "NOT armed", "NOT enabled", "No Airtable calls were made", "remains separate pending work".
- [ ] `total_phase_count` is 8 when `simulatedNotExecuted`.
- [ ] All 8 phases have `SRS-PH-` prefix.
- [ ] First phase is `SRS-PH-01` (schema write executor).
- [ ] Last phase is `SRS-PH-08` (final guard).
- [ ] Checkpoint phases (SRS-PH-02, SRS-PH-04, SRS-PH-06) are `skipped`.
- [ ] Executor/guard phases (SRS-PH-01, SRS-PH-03, SRS-PH-05, SRS-PH-07, SRS-PH-08) are `simulated`.
- [ ] Phase ordering is deterministic.
- [ ] `total_phase_count` equals `phases.len()`.
- [ ] `gate_armed` (runtime/global) is `false` in every result.
- [ ] `ephemeral_armed_decision_seen` is `true` only when all prerequisites pass.
- [ ] `executionEnabled` is `false` in every result.
- [ ] `writesEnabled` is `false` in every result.
- [ ] `readsEnabled` is `false` in every result.
- [ ] `networkReadsAttempted` is `false` in every result.
- [ ] `networkWritesAttempted` is `false` in every result.
- [ ] `noChangesMade` is `true` in every result.
- [ ] `airtableClientCalled` is `false` in every result.
- [ ] `checkpointFileWritten` is `false` in every result.
- [ ] `safety_snapshot.write_gate_disabled` is always `true`.
- [ ] `safety_snapshot.gate_armed` is always `false`.
- [ ] `safety_snapshot.execution_enabled` is always `false`.
- [ ] `evaluate_write_gate()` returns `Disabled` before and after any simulator call.
- [ ] Two independent calls produce independent results — no global state is shared.
- [ ] No token (`pat_`) in the serialized result.
- [ ] No absolute path in the serialized result.
- [ ] No old or new record ID in the serialized result.
- [ ] No raw record payload or field values in the serialized result.
- [ ] No attachment URL in the serialized result.
- [ ] No `"succeeded"`, `"enabled"`, `"executionReady"`, `"restoreComplete"`, or `"restoreSuccess"` in any serialized result.
- [ ] Run `cargo test -- sandbox_restore_simulator::tests` — all tests pass.

---

## Sandbox Linked Second-Pass Adapter Checklist (internal module, no network call)

> Scope: `restore/sandbox_linked_second_pass_adapter.rs` — internal Rust module only. No Tauri command, no TypeScript, no UI surface.

- [ ] Default disabled-mode request returns `notExecuted`.
- [ ] Missing explicit internal linked sandbox flag returns `blocked`.
- [ ] Disabled mode returns `notExecuted` with mode `disabled`.
- [ ] Arming prerequisite failure causes `blocked` with `SLSPA-CHK-04`.
- [ ] Simulator prerequisite failure causes `blocked`.
- [ ] Linked executor blocked causes `blocked` with `SLSPA-CHK-06`.
- [ ] Schema adapter not ready causes `blocked` with `SLSPA-CHK-07`.
- [ ] Record adapter not ready causes `blocked` with `SLSPA-CHK-08`.
- [ ] Insufficient mapping coverage causes `blocked` with `SLSPA-CHK-09`.
- [ ] Target base not empty causes `blocked`.
- [ ] Sandbox not verified causes `blocked`.
- [ ] `evaluate_write_gate()` returns `Disabled/DisabledByProductPolicy` before and after any adapter call.
- [ ] `readyForSandboxCall` returned only when all prerequisites are satisfied and explicit flag is true.
- [ ] `readyForSandboxCall` has `runtimeExecutionEnabled: false`.
- [ ] `readyForSandboxCall` has `appRuntimeWritesEnabled: false`.
- [ ] `readyForSandboxCall` has `appRuntimeReadsEnabled: false`.
- [ ] `networkWritesAttempted` is `false` in every result.
- [ ] `noChangesMade` is `true` in every result.
- [ ] Only `linkedUpdateBatchDescriptor` operation kind appears in output.
- [ ] No schema operation (`createTable`, `createField`) appears in any result.
- [ ] No first-pass record create operation appears in any result.
- [ ] No attachment operation appears in any result.
- [ ] Operation ordering is deterministic — two calls with the same input produce the same operation ID sequence.
- [ ] No token (`pat_`) in the serialized result.
- [ ] No absolute path in the serialized result.
- [ ] No record payload or field values in the serialized result.
- [ ] No raw HTTP body in the serialized result.
- [ ] No old or new record ID in the serialized result.
- [ ] No attachment URL in the serialized result.
- [ ] No `"succeeded"`, `"enabled"`, `"executionReady"`, `"restoreComplete"`, or `"restoreSuccess"` in any serialized result.
- [ ] No Tauri command added — module is Rust-internal only.
- [ ] No TypeScript/UI surface added.
- [ ] No production adapter path implemented.
- [ ] `NoOpLinkedSecondPassAdapter.planned_operation_count()` returns 0.
- [ ] `MockLinkedSecondPassAdapter` returns configured count without network call.
- [ ] `safety_snapshot.write_gate_disabled` is always `true`.
- [ ] `safety_snapshot.runtimeExecutionEnabled` is always `false`.
- [ ] `safety_snapshot.appRuntimeWritesEnabled` is always `false`.
- [ ] `safety_snapshot.appRuntimeReadsEnabled` is always `false`.
- [ ] `safety_snapshot.networkWritesAttempted` is always `false`.
- [ ] Run `cargo test -- sandbox_linked_second_pass_adapter::tests` — all tests pass.

---

## Sandbox Record Write Adapter Checklist (internal module, no network call)

> Scope: `restore/sandbox_record_write_adapter.rs` — internal Rust module only. No Tauri command, no TypeScript, no UI surface.

- [ ] Default disabled-mode request returns `notExecuted`.
- [ ] Missing explicit internal record sandbox flag returns `blocked`.
- [ ] Disabled mode returns `notExecuted` with mode `disabled`.
- [ ] Arming prerequisite failure causes `blocked` with `SRWA-CHK-04`.
- [ ] Simulator prerequisite failure causes `blocked`.
- [ ] Record executor blocked plan causes `blocked` with `SRWA-CHK-06`.
- [ ] Schema adapter not ready causes `blocked` with `SRWA-CHK-07`.
- [ ] Target base not empty causes `blocked`.
- [ ] Sandbox not verified causes `blocked`.
- [ ] `evaluate_write_gate()` returns `Disabled/DisabledByProductPolicy` before and after any adapter call.
- [ ] `readyForSandboxCall` returned only when all prerequisites are satisfied and explicit flag is true.
- [ ] `readyForSandboxCall` has `runtimeExecutionEnabled: false`.
- [ ] `readyForSandboxCall` has `appRuntimeWritesEnabled: false`.
- [ ] `readyForSandboxCall` has `appRuntimeReadsEnabled: false`.
- [ ] `networkWritesAttempted` is `false` in every result.
- [ ] `noChangesMade` is `true` in every result.
- [ ] Only `createRecordBatchDescriptor` operation kind appears in output.
- [ ] No schema operation (`createTable`, `createField`) appears in any result.
- [ ] No linked update operation appears in any result.
- [ ] No attachment operation appears in any result.
- [ ] Operation ordering is deterministic — two calls with the same input produce the same operation ID sequence.
- [ ] No token (`pat_`) in the serialized result.
- [ ] No absolute path in the serialized result.
- [ ] No record payload or field values in the serialized result.
- [ ] No raw HTTP body in the serialized result.
- [ ] No old or new record ID in the serialized result.
- [ ] No attachment URL in the serialized result.
- [ ] No `"succeeded"`, `"enabled"`, `"executionReady"`, `"restoreComplete"`, or `"restoreSuccess"` in any serialized result.
- [ ] No Tauri command added — module is Rust-internal only.
- [ ] No TypeScript/UI surface added.
- [ ] No production adapter path implemented.
- [ ] `NoOpRecordWriteAdapter.planned_operation_count()` returns 0.
- [ ] `MockRecordWriteAdapter` returns configured count without network call.
- [ ] `safety_snapshot.write_gate_disabled` is always `true`.
- [ ] `safety_snapshot.runtimeExecutionEnabled` is always `false`.
- [ ] `safety_snapshot.appRuntimeWritesEnabled` is always `false`.
- [ ] `safety_snapshot.appRuntimeReadsEnabled` is always `false`.
- [ ] `safety_snapshot.networkWritesAttempted` is always `false`.
- [ ] Run `cargo test -- sandbox_record_write_adapter::tests` — all tests pass.

---

## Sandbox Final Validation Adapter Checklist (internal module, no network call)

> Scope: `restore/sandbox_final_validation_adapter.rs` — internal Rust module only. No Tauri command, no TypeScript, no UI surface.

- [ ] Default disabled-mode request returns `notExecuted`.
- [ ] Missing explicit internal validation sandbox flag returns `blocked` with `SFVA-CHK-02`.
- [ ] Disabled mode returns `notExecuted` with mode `disabled`.
- [ ] Arming prerequisite failure causes `blocked` with `SFVA-CHK-04`.
- [ ] Simulator prerequisite failure causes `blocked`.
- [ ] Final validation reader plan blocked causes `blocked` with `SFVA-CHK-06`.
- [ ] Schema adapter not ready causes `blocked` with `SFVA-CHK-07`.
- [ ] Record adapter not ready causes `blocked` with `SFVA-CHK-08`.
- [ ] Linked adapter not ready causes `blocked` with `SFVA-CHK-09`.
- [ ] Final validation enforcement not safe causes `blocked`.
- [ ] Sandbox not verified causes `blocked`.
- [ ] `evaluate_write_gate()` returns `Disabled/DisabledByProductPolicy` before and after any adapter call.
- [ ] `readyForSandboxCall` returned only when all prerequisites are satisfied and explicit flag is true.
- [ ] `readyForSandboxCall` has `runtimeExecutionEnabled: false`.
- [ ] `readyForSandboxCall` has `appRuntimeWritesEnabled: false`.
- [ ] `readyForSandboxCall` has `appRuntimeReadsEnabled: false`.
- [ ] `networkReadsAttempted` is `false` in every result.
- [ ] `networkWritesAttempted` is `false` in every result.
- [ ] `noChangesMade` is `true` in every result.
- [ ] Only valid read descriptor operation kinds appear: `schemaCountReadDescriptor`, `fieldCountReadDescriptor`, `recordCountReadDescriptor`, `linkedFieldCoverageReadDescriptor`, `attachmentMetadataReadDescriptor`, `manifestChecksumReadDescriptor`, `finalGuardDescriptor`.
- [ ] No write operation kind (`createTable`, `createField`, `createRecordBatch`, `linkedUpdateBatchDescriptor`) appears in any result.
- [ ] `manifestChecksumReadDescriptor` present only when `manifest_present: true`.
- [ ] `finalGuardDescriptor` is always the last operation.
- [ ] `schemaCountReadDescriptor` is always the first operation.
- [ ] Operation ordering is deterministic — two calls with the same input produce the same operation ID sequence.
- [ ] Attachment descriptor note mentions metadata only — no download, no CDN URL.
- [ ] No token (`pat_`) in the serialized result.
- [ ] No absolute path in the serialized result.
- [ ] No record payload or field values in the serialized result.
- [ ] No raw HTTP body in the serialized result.
- [ ] No old or new record ID in the serialized result.
- [ ] No attachment URL in the serialized result.
- [ ] No `"succeeded"`, `"enabled"`, `"executionReady"`, `"restoreComplete"`, or `"restoreSuccess"` in any serialized result.
- [ ] No Tauri command added — module is Rust-internal only.
- [ ] No TypeScript/UI surface added.
- [ ] No production adapter path implemented.
- [ ] `NoOpFinalValidationReadAdapter.planned_operation_count()` returns 0.
- [ ] `MockFinalValidationReadAdapter` returns configured count without network call.
- [ ] `safety_snapshot.write_gate_disabled` is always `true`.
- [ ] `safety_snapshot.runtimeExecutionEnabled` is always `false`.
- [ ] `safety_snapshot.appRuntimeWritesEnabled` is always `false`.
- [ ] `safety_snapshot.appRuntimeReadsEnabled` is always `false`.
- [ ] `safety_snapshot.networkReadsAttempted` is always `false`.
- [ ] `safety_snapshot.networkWritesAttempted` is always `false`.
- [ ] Run `cargo test -- sandbox_final_validation_adapter::tests` — all tests pass.

---

## Sandbox Schema Write Adapter Checklist (internal module, no network call)

> Scope: `restore/sandbox_schema_write_adapter.rs` — internal Rust module only. No Tauri command, no TypeScript, no UI surface.

- [ ] Default disabled-mode request returns `notExecuted`.
- [ ] Missing explicit internal schema sandbox flag returns `blocked`.
- [ ] Disabled mode returns `notExecuted` with mode `disabled`.
- [ ] Arming prerequisite failure causes `blocked` with `SSWA-CHK-04`.
- [ ] Simulator prerequisite failure causes `blocked`.
- [ ] Executor blocked plan causes `blocked` with `SSWA-CHK-06`.
- [ ] Target base not empty causes `blocked`.
- [ ] Sandbox not verified causes `blocked`.
- [ ] `evaluate_write_gate()` returns `Disabled/DisabledByProductPolicy` before and after any adapter call.
- [ ] `readyForSandboxCall` returned only when all prerequisites are satisfied and explicit flag is true.
- [ ] `readyForSandboxCall` has `runtimeExecutionEnabled: false`.
- [ ] `readyForSandboxCall` has `appRuntimeWritesEnabled: false`.
- [ ] `readyForSandboxCall` has `appRuntimeReadsEnabled: false`.
- [ ] `networkWritesAttempted` is `false` in every result.
- [ ] `noChangesMade` is `true` in every result.
- [ ] Only `createTableDescriptor` and `createFieldDescriptor` operation kinds appear in output.
- [ ] No record operation (`createRecord`, `updateRecord`) appears in any result.
- [ ] No linked update operation appears in any result.
- [ ] No attachment operation appears in any result.
- [ ] Operation ordering is deterministic — two calls with the same input produce the same operation ID sequence.
- [ ] Table descriptor operations precede field descriptor operations.
- [ ] No token (`pat_`) in the serialized result.
- [ ] No absolute path in the serialized result.
- [ ] No record payload or field values in the serialized result.
- [ ] No raw HTTP body in the serialized result.
- [ ] No old or new record ID in the serialized result.
- [ ] No attachment URL in the serialized result.
- [ ] No `"succeeded"`, `"enabled"`, `"executionReady"`, `"restoreComplete"`, or `"restoreSuccess"` in any serialized result.
- [ ] No Tauri command added — module is Rust-internal only.
- [ ] No TypeScript/UI surface added.
- [ ] No production adapter path implemented.
- [ ] `NoOpSchemaWriteAdapter.planned_operation_count()` returns 0.
- [ ] `MockSchemaWriteAdapter` returns configured count without network call.
- [ ] `safety_snapshot.write_gate_disabled` is always `true`.
- [ ] `safety_snapshot.runtimeExecutionEnabled` is always `false`.
- [ ] `safety_snapshot.appRuntimeWritesEnabled` is always `false`.
- [ ] `safety_snapshot.appRuntimeReadsEnabled` is always `false`.
- [ ] `safety_snapshot.networkWritesAttempted` is always `false`.
- [ ] Run `cargo test -- sandbox_schema_write_adapter::tests` — all tests pass.

---

## Sandbox Adapter Chain Runner Checklist (internal module, mock/no-op only)

> Scope: `restore/sandbox_adapter_chain_runner.rs` — internal Rust module only. No Tauri command, no TypeScript, no UI surface. No live Airtable network call.

- [ ] Default disabled-mode request returns `blocked`.
- [ ] Missing explicit internal mock chain flag returns `blocked` with `SACR-CHK-02`.
- [ ] Simulator prerequisite failure causes `blocked` with `SACR-CHK-04`.
- [ ] Schema adapter not ready causes `blocked` with `SACR-CHK-05`.
- [ ] Record adapter not ready causes `blocked` with `SACR-CHK-06`.
- [ ] Linked adapter not ready causes `blocked` with `SACR-CHK-07`.
- [ ] Final validation adapter not ready causes `blocked` with `SACR-CHK-08`.
- [ ] `evaluate_write_gate()` returns `Disabled/DisabledByProductPolicy` before and after any runner call.
- [ ] `mockRunNotExecuted` returned only when all eight prerequisites are satisfied and explicit flag is true.
- [ ] `mockRunNotExecuted` has `runtimeExecutionEnabled: false`.
- [ ] `mockRunNotExecuted` has `appRuntimeWritesEnabled: false`.
- [ ] `mockRunNotExecuted` has `appRuntimeReadsEnabled: false`.
- [ ] `networkReadsAttempted` is `false` in every result.
- [ ] `networkWritesAttempted` is `false` in every result.
- [ ] `noChangesMade` is `true` in every result.
- [ ] `airtableClientCalled` is `false` in every result.
- [ ] `totalPhaseCount` is 4 when `mockRunNotExecuted`.
- [ ] Phase ordering is deterministic — SACR-PH-01, SACR-PH-02, SACR-PH-03, SACR-PH-04.
- [ ] Phase labels are schema, record, linked, final validation in that order.
- [ ] All four phases have status `mockObserved` when `mockRunNotExecuted`.
- [ ] Safe operation counts are reported per phase — no raw payloads.
- [ ] Phase `operation_count` matches corresponding adapter count field.
- [ ] No token (`pat_`) in the serialized result.
- [ ] No absolute path in the serialized result.
- [ ] No record payload or field values in the serialized result.
- [ ] No raw HTTP body in the serialized result.
- [ ] No old or new record ID in the serialized result.
- [ ] No attachment URL in the serialized result.
- [ ] No `"succeeded"`, `"enabled"`, `"executionReady"`, `"restoreComplete"`, or `"restoreSuccess"` in any serialized result.
- [ ] No Tauri command added — module is Rust-internal only.
- [ ] No TypeScript/UI surface added.
- [ ] No production adapter path implemented.
- [ ] `safety_snapshot.write_gate_disabled` is always `true`.
- [ ] `safety_snapshot.runtimeExecutionEnabled` is always `false`.
- [ ] `safety_snapshot.appRuntimeWritesEnabled` is always `false`.
- [ ] `safety_snapshot.appRuntimeReadsEnabled` is always `false`.
- [ ] `safety_snapshot.networkReadsAttempted` is always `false`.
- [ ] `safety_snapshot.networkWritesAttempted` is always `false`.
- [ ] `safety_snapshot.airtableClientCalled` is always `false`.
- [ ] Two independent calls produce independent results — no shared state.
- [ ] Message mentions "remains separate pending work" — live execution is clearly labeled pending.
- [ ] Run `cargo test -- sandbox_adapter_chain_runner::tests` — all tests pass.

---

## Live Schema Write Test Contract Checklist (internal module, contract-only)

> Scope: `restore/live_schema_write_test_contract.rs` — internal Rust module only. No Tauri command, no TypeScript, no UI surface. No live Airtable network call. No token accepted or persisted.

- [ ] Default disabled-mode request returns `blocked`.
- [ ] Missing explicit internal contract flag returns `blocked` with `LSWTC-PRE-02`.
- [ ] Schema adapter not ready causes `blocked` with `LSWTC-PRE-04`.
- [ ] Adapter chain runner not ready causes `blocked` with `LSWTC-PRE-05`.
- [ ] Any shared prerequisite failure (e.g. `failure_modes_safe=false`) causes `blocked`.
- [ ] `evaluate_write_gate()` returns `Disabled/DisabledByProductPolicy` before and after any contract call.
- [ ] `eligibleButNotExecuted` returned only when all eight prerequisites are satisfied and explicit flag is true.
- [ ] `eligibleButNotExecuted` has `contractOnly: true`.
- [ ] `eligibleButNotExecuted` has `appRuntimeExecutionEnabled: false`.
- [ ] `eligibleButNotExecuted` has `appRuntimeWritesEnabled: false`.
- [ ] `eligibleButNotExecuted` has `appRuntimeReadsEnabled: false`.
- [ ] `networkReadsAttempted` is `false` in every result.
- [ ] `networkWritesAttempted` is `false` in every result.
- [ ] `noChangesMade` is `true` in every result.
- [ ] `airtableClientCalled` is `false` in every result.
- [ ] `contract_only` is `true` in every result, including `blocked`.
- [ ] `totalPrerequisiteCount` is 8 when `eligibleButNotExecuted`.
- [ ] All 8 prerequisites have status `ready` when `eligibleButNotExecuted`.
- [ ] Required future-live conditions list is non-empty in every result.
- [ ] Required future-live conditions list mentions: sandbox-only base, record writes disabled, linked updates disabled, final validation reads disabled.
- [ ] No token (`pat_`) in the serialized result.
- [ ] No absolute path in the serialized result.
- [ ] No record payload or field values in the serialized result.
- [ ] No raw HTTP body in the serialized result.
- [ ] No old or new record ID in the serialized result.
- [ ] No attachment URL in the serialized result.
- [ ] No `"succeeded"`, `"enabled"`, `"executionReady"`, `"restoreComplete"`, or `"restoreSuccess"` in any serialized result.
- [ ] No Tauri command added — module is Rust-internal only.
- [ ] No TypeScript/UI surface added.
- [ ] No token field in the request struct.
- [ ] `safety_snapshot.write_gate_disabled` is always `true`.
- [ ] `safety_snapshot.contractOnly` is always `true`.
- [ ] `safety_snapshot.appRuntimeExecutionEnabled` is always `false`.
- [ ] `safety_snapshot.appRuntimeWritesEnabled` is always `false`.
- [ ] `safety_snapshot.appRuntimeReadsEnabled` is always `false`.
- [ ] `safety_snapshot.networkReadsAttempted` is always `false`.
- [ ] `safety_snapshot.networkWritesAttempted` is always `false`.
- [ ] `safety_snapshot.airtableClientCalled` is always `false`.
- [ ] Message mentions "remains separate pending work" — live schema write integration test is clearly labeled pending.
- [ ] Run `cargo test -- live_schema_write_test_contract::tests` — all tests pass.

## Sandbox Schema Write Integration Harness Checklist (test-only, `#[ignore]` by default)

### Opt-in gate
- [ ] `AIRBRIDGE_ENABLE_LIVE_SCHEMA_WRITE_TEST` not set → test skips without network call.
- [ ] `AIRBRIDGE_SANDBOX_AIRTABLE_TOKEN` not set → test skips without network call.
- [ ] `AIRBRIDGE_SANDBOX_TARGET_BASE_ID` not set → test skips without network call.
- [ ] Normal `cargo test` does NOT run `sandbox_schema_write_creates_table_and_verifies_contract`.
- [ ] `sandbox_schema_write_creates_table_and_verifies_contract` appears as `ignored` in default test output.

### Default suite (no env vars required)
- [ ] `missing_enable_flag_does_not_perform_network_call` passes.
- [ ] `missing_token_does_not_perform_network_call` passes.
- [ ] `missing_base_id_does_not_perform_network_call` passes.
- [ ] `evaluate_write_gate_remains_disabled_without_env_vars` passes.
- [ ] `contract_eligible_but_not_executed_with_all_prereqs_satisfied` passes.
- [ ] `schema_adapter_ready_for_sandbox_call_without_live_call` passes.
- [ ] `adapter_chain_returns_mock_run_not_executed_without_live_call` passes.
- [ ] `live_schema_write_test_does_not_introduce_tauri_command` passes.
- [ ] `no_record_endpoint_called_in_default_test_suite` passes.
- [ ] `no_attachment_endpoint_called_in_default_test_suite` passes.

### Contract verification (pre-live)
- [ ] Contract returns `eligibleButNotExecuted` before live call.
- [ ] `contract_only` is `true` in contract result.
- [ ] `airtableClientCalled` is `false` in contract result.
- [ ] `networkWritesAttempted` is `false` in contract result.
- [ ] `appRuntimeExecutionEnabled` is `false` in contract result.
- [ ] `appRuntimeWritesEnabled` is `false` in contract result.
- [ ] `appRuntimeReadsEnabled` is `false` in contract result.
- [ ] Schema adapter returns `readyForSandboxCall` before live call.
- [ ] `evaluate_write_gate()` returns `Disabled/DisabledByProductPolicy` before live call.

### Live call safety (when harness runs with all env vars)
- [ ] Only `createTable` (Metadata API) is called — no record endpoints.
- [ ] No linked update endpoint is called.
- [ ] No attachment endpoint is called.
- [ ] No final validation read endpoint is called.
- [ ] Outcome `table_name` matches requested name.
- [ ] Outcome `table_id` is non-empty.
- [ ] Serialized outcome does NOT contain token (`pat_` prefix absent).
- [ ] `evaluate_write_gate()` returns `Disabled/DisabledByProductPolicy` after live call.

### Safety invariants
- [ ] No Tauri command added for this harness.
- [ ] No TypeScript/UI surface added.
- [ ] No execute/enable/arm/run/validate button added.
- [ ] No restore success state introduced.
- [ ] App runtime execution/reads/writes remain disabled.
- [ ] Record writes remain disabled.
- [ ] Linked record updates remain disabled.
- [ ] Final validation reads remain disabled.
- [ ] Attachment handling remains disabled.
- [ ] No new dependency added to `Cargo.toml`.

### Ops
- [ ] Run `npm --prefix apps/desktop run rust:test` — all non-ignored tests pass; `sandbox_schema_write_creates_table_and_verifies_contract` appears as `ignored`.
- [ ] Run prohibited-terms scan on `tests/live_schema_write_sandbox.rs` — no matches.

---

## Live Record Write Test Contract Checklist

### Contract-only gate

- [ ] `evaluate_live_record_write_test_contract` returns `Blocked` when `mode` is `Disabled`.
- [ ] `evaluate_live_record_write_test_contract` returns `Blocked` when `explicit_internal_live_record_test_contract_requested` is `false`.
- [ ] `evaluate_live_record_write_test_contract` returns `Blocked` when live schema write contract is not eligible.
- [ ] `evaluate_live_record_write_test_contract` returns `Blocked` when sandbox record write adapter is not ready.
- [ ] `evaluate_live_record_write_test_contract` returns `Blocked` when sandbox schema write adapter is not ready.
- [ ] `evaluate_live_record_write_test_contract` returns `Blocked` when adapter chain runner is not `MockRunNotExecuted`.
- [ ] `evaluate_live_record_write_test_contract` returns `Blocked` when gate arming is not `ArmedNotExecutable`.
- [ ] `evaluate_live_record_write_test_contract` returns `Blocked` when simulator is not `SimulatedNotExecuted`.
- [ ] `evaluate_live_record_write_test_contract` returns `Blocked` when enablement readiness is not `ReadyButDisabled`.
- [ ] `evaluate_live_record_write_test_contract` returns `EligibleButNotExecuted` only when all 10 prerequisites are satisfied.

### Safety invariants

- [ ] `contract_only` is always `true`.
- [ ] `app_runtime_execution_enabled` is always `false`.
- [ ] `app_runtime_writes_enabled` is always `false`.
- [ ] `app_runtime_reads_enabled` is always `false`.
- [ ] `network_reads_attempted` is always `false`.
- [ ] `network_writes_attempted` is always `false`.
- [ ] `airtable_client_called` is always `false`.
- [ ] `no_changes_made` is always `true`.
- [ ] `evaluate_write_gate()` remains `Disabled/DisabledByProductPolicy` before and after.
- [ ] No token is accepted, stored, or returned.
- [ ] No Airtable API calls are made.
- [ ] No Tauri command exists for this contract.
- [ ] No TypeScript/UI surface exists for this contract.
- [ ] No restore success state is introduced.

### Serialization checks

- [ ] No token (`pat_`, `"token"`, `"apiKey"`, `"secret"`) in serialized result.
- [ ] No absolute path (`/Users/`, `/home/`, `/tmp/`) in serialized result.
- [ ] No record payload (`"fields":{`, `"records":[{`) in serialized result.
- [ ] No raw HTTP (`"body":{`, `"statusCode"`) in serialized result.
- [ ] No old record ID (`"oldRecordId"`, `rec_old_`) in serialized result.
- [ ] No new record ID (`"newRecordId"`, `rec_new_`) in serialized result.
- [ ] No attachment URL (`cdn.airtable.com`, `attachmentUrl`) in serialized result.
- [ ] No success state (`"succeeded"`, `restoreComplete`, `restoreSuccess`, `executionReady`) in serialized result.

### Pending work (still disabled)

- [ ] Live record write integration test remains separate pending work.
- [ ] Linked record updates remain disabled.
- [ ] Final validation reads remain disabled.
- [ ] Attachment handling remains disabled.
- [ ] Live end-to-end restore execution remains disabled.

---

## Sandbox Record Write Harness Checklist

### Opt-in gate

- [ ] Default `cargo test` does NOT run the live sandbox record write test.
- [ ] Missing `AIRBRIDGE_ENABLE_LIVE_RECORD_WRITE_TEST` does not perform a network call.
- [ ] Missing token env var does not perform a network call.
- [ ] Missing base ID env var does not perform a network call.
- [ ] Missing table ID/name env var does not perform a network call.
- [ ] `all_required_env_vars_present()` returns `false` in all cases above.

### Pre-call contract verification

- [ ] `evaluate_live_record_write_test_contract()` returns `EligibleButNotExecuted` before live call.
- [ ] `build_sandbox_record_write_adapter()` returns `ReadyForSandboxCall` before live call.
- [ ] `run_sandbox_adapter_chain()` returns `MockRunNotExecuted` before live call.
- [ ] `evaluate_write_gate()` returns `Disabled` before live call.

### Live call behavior (opt-in, ignored by default)

- [ ] Single record create only — Records API POST.
- [ ] Minimal `Name` string field — no linked fields, no attachment fields.
- [ ] No record update operations.
- [ ] No linked record update endpoint called.
- [ ] No attachment endpoint called.
- [ ] No final validation read performed.

### Post-call invariants (opt-in, ignored by default)

- [ ] `outcome.record_created` is `true`.
- [ ] `outcome.record_count` is 1.
- [ ] `outcome.table_name` is non-empty.
- [ ] `evaluate_write_gate()` still returns `Disabled` after live call.
- [ ] App runtime execution, reads, and writes remain disabled.

### Safety invariants

- [ ] Token, base ID, table ID/name never printed or asserted on value.
- [ ] No record ID exposed in sanitized outcome.
- [ ] `evaluate_write_gate()` unchanged — still `Disabled/DisabledByProductPolicy`.
- [ ] No Tauri command added.
- [ ] No TypeScript/UI surface added.
- [ ] No restore success state introduced.
- [ ] App runtime restore execution remains disabled.

### Serialization checks

- [ ] No token (`pat_`, `"token"`, `"apiKey"`) in serialized outcome.
- [ ] No record ID (`"id"`, `recNewRecord`, `rec_`) in serialized outcome.
- [ ] No raw HTTP body in serialized outcome.
- [ ] No attachment URL in serialized outcome.
- [ ] No success state (`"succeeded"`, `restoreComplete`) in serialized outcome.

### Pending work (still disabled)

- [ ] Linked record updates remain disabled.
- [ ] Final validation reads remain disabled.
- [ ] Attachment handling remains disabled.
- [ ] Live end-to-end restore execution remains disabled.


---

## Live Linked Update Test Contract Checklist (LLUTC)

### Mode and explicit flag

- [ ] Default disabled mode returns `Blocked`.
- [ ] Missing `explicit_internal_live_linked_update_test_contract_requested` returns `Blocked` at `LLUTC-PRE-02`.
- [ ] `sandboxIntegrationCandidate` mode with explicit flag proceeds to prerequisite evaluation.

### Prerequisite chain

- [ ] LLUTC-PRE-03: `evaluate_write_gate()` must return `Disabled/DisabledByProductPolicy`.
- [ ] LLUTC-PRE-04: `evaluate_live_record_write_test_contract()` must return `EligibleButNotExecuted`.
- [ ] LLUTC-PRE-05: `build_sandbox_linked_second_pass_adapter()` must return `ReadyForSandboxCall`.
- [ ] LLUTC-PRE-06: `build_sandbox_record_write_adapter()` must return `ReadyForSandboxCall`.
- [ ] LLUTC-PRE-07: `build_sandbox_schema_write_adapter()` must return `ReadyForSandboxCall`.
- [ ] LLUTC-PRE-08: `run_sandbox_adapter_chain()` must return `MockRunNotExecuted`.
- [ ] LLUTC-PRE-09: `build_sandbox_gate_arming_decision()` must return `ArmedNotExecutable`.
- [ ] LLUTC-PRE-10: `run_sandbox_restore_simulator()` must return `SimulatedNotExecuted`.
- [ ] LLUTC-PRE-11: `build_sandbox_enablement_readiness_report()` must return `ReadyButDisabled`.

### EligibleButNotExecuted — all 11 prerequisites pass

- [ ] All 11 prerequisites are `Ready` when eligible.
- [ ] `contract_only` is `true`.
- [ ] `no_changes_made` is `true`.
- [ ] `airtable_client_called` is `false`.
- [ ] `network_reads_attempted` is `false`.
- [ ] `network_writes_attempted` is `false`.
- [ ] `app_runtime_execution_enabled` is `false`.
- [ ] `app_runtime_writes_enabled` is `false`.
- [ ] `app_runtime_reads_enabled` is `false`.
- [ ] Result message mentions live linked update test remains pending.

### Safety invariants (both Blocked and EligibleButNotExecuted)

- [ ] `evaluate_write_gate()` unchanged — still `Disabled/DisabledByProductPolicy`.
- [ ] `safety_snapshot.write_gate_disabled` is `true`.
- [ ] `safety_snapshot.airtable_client_called` is `false`.
- [ ] `safety_snapshot.contract_only` is `true`.
- [ ] All runtime/network flags in snapshot are `false`.
- [ ] No Tauri command added.
- [ ] No TypeScript/UI surface added.
- [ ] No restore success state introduced.

### Required future-live conditions

- [ ] Disposable sandbox-only base required is present.
- [ ] Attachment handling remains disabled is present.
- [ ] Final validation reads remain disabled is present.
- [ ] Conditions reported in both `EligibleButNotExecuted` and `Blocked` results.

### Serialization checks

- [ ] No token (`pat_`, `"token"`, `"apiKey"`, `"secret"`) in serialized result.
- [ ] No absolute path (`/Users/`, `/home/`, `/tmp/`) in serialized result.
- [ ] No record payload (`"fields":{`, `"records":[{`) in serialized result.
- [ ] No raw HTTP body (`"body":{`, `"statusCode"`) in serialized result.
- [ ] No old record ID (`"oldRecordId"`, `rec_old_`) in serialized result.
- [ ] No new record ID (`"newRecordId"`, `rec_new_`) in serialized result.
- [ ] No attachment URL (`cdn.airtable.com`, `attachmentUrl`) in serialized result.
- [ ] No success state (`"succeeded"`, `restoreComplete`, `restoreSuccess`, `executionReady`) in serialized result.

### Pending work (still disabled)

- [ ] Live linked update integration test remains separate pending work.
- [ ] Final validation reads remain disabled.
- [ ] Attachment handling remains disabled.
- [ ] Live end-to-end restore execution remains disabled.

---

## Live Final Validation Test Contract Checklist (LFVTC)

### Mode and explicit flag

- [ ] Default disabled mode returns `Blocked`.
- [ ] Missing `explicit_internal_live_final_validation_test_contract_requested` returns `Blocked` at `LFVTC-PRE-02`.
- [ ] `sandboxIntegrationCandidate` mode with explicit flag proceeds to prerequisite evaluation.

### Prerequisite chain

- [ ] LFVTC-PRE-03: `evaluate_write_gate()` must return `Disabled/DisabledByProductPolicy`.
- [ ] LFVTC-PRE-04: `evaluate_live_linked_update_test_contract()` must return `EligibleButNotExecuted`.
- [ ] LFVTC-PRE-05: `build_sandbox_final_validation_adapter()` must return `ReadyForSandboxCall`.
- [ ] LFVTC-PRE-06: `build_sandbox_linked_second_pass_adapter()` must return `ReadyForSandboxCall`.
- [ ] LFVTC-PRE-07: `build_sandbox_record_write_adapter()` must return `ReadyForSandboxCall`.
- [ ] LFVTC-PRE-08: `build_sandbox_schema_write_adapter()` must return `ReadyForSandboxCall`.
- [ ] LFVTC-PRE-09: `run_sandbox_adapter_chain()` must return `MockRunNotExecuted`.
- [ ] LFVTC-PRE-10: `build_sandbox_gate_arming_decision()` must return `ArmedNotExecutable`.
- [ ] LFVTC-PRE-11: `run_sandbox_restore_simulator()` must return `SimulatedNotExecuted`.
- [ ] LFVTC-PRE-12: `build_sandbox_enablement_readiness_report()` must return `ReadyButDisabled`.

### EligibleButNotExecuted — all 12 prerequisites pass

- [ ] All 12 prerequisites are `Ready` when eligible.
- [ ] `contract_only` is `true`.
- [ ] `no_changes_made` is `true`.
- [ ] `airtable_client_called` is `false`.
- [ ] `network_reads_attempted` is `false`.
- [ ] `network_writes_attempted` is `false`.
- [ ] `app_runtime_execution_enabled` is `false`.
- [ ] `app_runtime_writes_enabled` is `false`.
- [ ] `app_runtime_reads_enabled` is `false`.
- [ ] Result message mentions live final validation test remains pending.

### Safety invariants (both Blocked and EligibleButNotExecuted)

- [ ] `evaluate_write_gate()` unchanged — still `Disabled/DisabledByProductPolicy`.
- [ ] `safety_snapshot.write_gate_disabled` is `true`.
- [ ] `safety_snapshot.airtable_client_called` is `false`.
- [ ] `safety_snapshot.contract_only` is `true`.
- [ ] All runtime/network flags in snapshot are `false`.
- [ ] No Tauri command added.
- [ ] No TypeScript/UI surface added.
- [ ] No restore success state introduced.

### Required future-live conditions

- [ ] Disposable sandbox-only base required is present.
- [ ] Attachment binary handling remains disabled is present.
- [ ] App runtime restore execution remains disabled is present.
- [ ] Conditions reported in both `EligibleButNotExecuted` and `Blocked` results.

### Serialization checks

- [ ] No token (`pat_`, `"token"`, `"apiKey"`, `"secret"`) in serialized result.
- [ ] No absolute path (`/Users/`, `/home/`, `/tmp/`) in serialized result.
- [ ] No record payload (`"fields":{`, `"records":[{`) in serialized result.
- [ ] No raw HTTP body (`"body":{`, `"statusCode"`) in serialized result.
- [ ] No old record ID (`"oldRecordId"`, `rec_old_`) in serialized result.
- [ ] No new record ID (`"newRecordId"`, `rec_new_`) in serialized result.
- [ ] No attachment URL (`cdn.airtable.com`, `attachmentUrl`) in serialized result.
- [ ] No success state (`"succeeded"`, `restoreComplete`, `restoreSuccess`, `executionReady`) in serialized result.

### Pending work (still disabled)

- [ ] Live final validation read integration test remains separate pending work.
- [ ] Attachment binary handling remains disabled.
- [ ] App runtime restore execution remains disabled.
- [ ] Live end-to-end restore execution remains disabled.

---

## LLUSH — Live Linked Update Sandbox Harness

Checklist for `tests/live_linked_update_sandbox.rs` and supporting models/client.

### Default (non-ignored) test pass

- [ ] `cargo test --test live_linked_update_sandbox` passes with 0 failures, 1 ignored.
- [ ] No network call is made during the default run.
- [ ] `evaluate_write_gate()` returns `Disabled` in all default tests.

### Env var guard

- [ ] Missing `AIRBRIDGE_ENABLE_LIVE_LINKED_UPDATE_TEST` causes `#[ignore]` test to skip, not fail.
- [ ] Missing `AIRBRIDGE_SANDBOX_AIRTABLE_TOKEN` causes skip.
- [ ] Missing `AIRBRIDGE_SANDBOX_TARGET_BASE_ID` causes skip.
- [ ] Missing `AIRBRIDGE_SANDBOX_LINK_SOURCE_TABLE_ID_OR_NAME` causes skip.
- [ ] Missing `AIRBRIDGE_SANDBOX_LINK_TARGET_TABLE_ID_OR_NAME` causes skip.
- [ ] Missing `AIRBRIDGE_SANDBOX_LINK_FIELD_NAME` causes skip.

### Pre-call contract gating (default tests)

- [ ] `evaluate_live_linked_update_test_contract()` returns `EligibleButNotExecuted` with all prereqs satisfied.
- [ ] `contract_only` is `true`.
- [ ] `airtable_client_called` is `false`.
- [ ] `network_writes_attempted` is `false`.
- [ ] `network_reads_attempted` is `false`.
- [ ] `app_runtime_execution_enabled` is `false`.
- [ ] `app_runtime_writes_enabled` is `false`.
- [ ] `app_runtime_reads_enabled` is `false`.
- [ ] `no_changes_made` is `true`.
- [ ] `build_sandbox_linked_second_pass_adapter()` returns `ReadyForSandboxCall`.
- [ ] `build_sandbox_record_write_adapter()` returns `ReadyForSandboxCall`.
- [ ] `build_sandbox_schema_write_adapter()` returns `ReadyForSandboxCall`.
- [ ] `run_sandbox_adapter_chain()` returns `MockRunNotExecuted`.

### Opt-in live test assertions (manual, requires sandbox setup)

- [ ] `outcome.record_updated` is `true`.
- [ ] `outcome.record_count` is `1`.
- [ ] `outcome.linked_target_count` is `1`.
- [ ] `outcome.source_table_name` is non-empty.
- [ ] Serialized `outcome` JSON contains no `pat_` token prefix.
- [ ] Serialized `outcome` JSON contains no raw record ID pattern (`recSensitive`).
- [ ] `evaluate_write_gate()` returns `Disabled` after live calls.

### Safety invariants

- [ ] Token never appears in any test output, assertion, or serialized struct.
- [ ] Record IDs never appear in any assertion message or serialized outcome.
- [ ] No Tauri command added for the harness.
- [ ] No TypeScript/UI surface added.
- [ ] No schema write performed.
- [ ] No attachment endpoint called.
- [ ] No final validation read performed.
- [ ] No record delete performed.
- [ ] `evaluate_write_gate()` returns `Disabled` before and after all live calls.
- [ ] App runtime execution, reads, and writes remain disabled.

### Pending work (still disabled after this harness)

- [ ] Attachment binary handling remains disabled.
- [ ] App runtime restore execution remains disabled.
- [ ] Live end-to-end restore execution remains disabled.

---

## LFVSH — Live Final Validation Sandbox Harness

Checklist for `tests/live_final_validation_sandbox.rs` and supporting models/client.

### Default (non-ignored) test pass

- [ ] `cargo test --test live_final_validation_sandbox` passes with 0 failures, 1 ignored.
- [ ] No network call is made during the default run.
- [ ] `evaluate_write_gate()` returns `Disabled` in all default tests.

### Env var guard

- [ ] Missing `AIRBRIDGE_ENABLE_LIVE_FINAL_VALIDATION_TEST` causes `#[ignore]` test to skip, not fail.
- [ ] Missing `AIRBRIDGE_SANDBOX_AIRTABLE_TOKEN` causes skip.
- [ ] Missing `AIRBRIDGE_SANDBOX_TARGET_BASE_ID` causes skip.
- [ ] Missing `AIRBRIDGE_SANDBOX_VALIDATION_TABLE_ID_OR_NAME` causes skip.

### Pre-call contract gating (default tests)

- [ ] `evaluate_live_final_validation_test_contract()` returns `EligibleButNotExecuted`.
- [ ] `contract_only` is `true`.
- [ ] `airtable_client_called` is `false`.
- [ ] `network_writes_attempted` is `false`.
- [ ] `network_reads_attempted` is `false`.
- [ ] `app_runtime_execution_enabled` is `false`.
- [ ] `app_runtime_writes_enabled` is `false`.
- [ ] `app_runtime_reads_enabled` is `false`.
- [ ] `no_changes_made` is `true`.
- [ ] `build_final_validation_reader_plan()` returns `NotExecuted`.
- [ ] `reads_enabled` is `false`, `writes_enabled` is `false`.
- [ ] `build_sandbox_final_validation_adapter()` returns `ReadyForSandboxCall`.
- [ ] `build_sandbox_linked_second_pass_adapter()` returns `ReadyForSandboxCall`.
- [ ] `build_sandbox_record_write_adapter()` returns `ReadyForSandboxCall`.
- [ ] `build_sandbox_schema_write_adapter()` returns `ReadyForSandboxCall`.
- [ ] `run_sandbox_adapter_chain()` returns `MockRunNotExecuted`.

### Opt-in live test assertions (manual, requires sandbox setup)

- [ ] `outcome.table_reachable` is `true`.
- [ ] If `AIRBRIDGE_SANDBOX_EXPECTED_MIN_RECORD_COUNT` set: `outcome.min_count_satisfied` is `true`.
- [ ] Serialized `outcome` JSON contains no `pat_` token prefix.
- [ ] Serialized `outcome` JSON contains no record ID pattern (`rec`).
- [ ] `evaluate_write_gate()` returns `Disabled` after live read.
- [ ] `build_final_validation_reader_plan()` still returns `NotExecuted` after live read.

### Safety invariants

- [ ] Token never appears in any test output, assertion, or serialized struct.
- [ ] Record IDs never appear in any assertion message or serialized outcome.
- [ ] Raw field values never appear in serialized outcome.
- [ ] No Tauri command added for the harness.
- [ ] No TypeScript/UI surface added.
- [ ] No records created, updated, or deleted.
- [ ] No schema write performed.
- [ ] No linked update performed.
- [ ] No attachment endpoint called.
- [ ] `evaluate_write_gate()` returns `Disabled` before and after all live calls.
- [ ] App runtime execution, reads, and writes remain disabled.

### Pending work (still disabled after this harness)

- [ ] Attachment binary handling remains disabled.
- [ ] App runtime restore execution remains disabled.
- [ ] Live end-to-end restore execution remains disabled.

---

## LE2ERTC — Live E2E Restore Test Contract

**Module:** `src/restore/live_e2e_restore_test_contract.rs`  
**Function:** `evaluate_live_e2e_restore_test_contract(request, schema_plan, record_plan)`

### Mode / flag checks

- [ ] `Disabled` mode always returns `Blocked`.
- [ ] `Disabled` mode blocked reason contains `LE2ERTC-PRE-01`.
- [ ] Missing `explicit_internal_live_e2e_restore_test_contract_requested` returns `Blocked`.
- [ ] Missing explicit flag blocked reason contains `LE2ERTC-PRE-02`.

### Sub-contract prerequisites

- [ ] FV contract not eligible (`mapping_coverage_sufficient=false`) → `Blocked` with `LE2ERTC-PRE-04`.
- [ ] Gate arming not ready (`rollback_limitation_safe=false`) → `Blocked`.
- [ ] Simulator not ready (`failure_modes_safe=false`) → `Blocked`.
- [ ] Restore harness not ready (`sandbox_verified=false`) → `Blocked`.

### Eligible state

- [ ] All prerequisites satisfied → `EligibleButNotExecuted`.
- [ ] `contract_only` is `true` when eligible.
- [ ] `contract_only` is `true` when blocked.
- [ ] `safety_snapshot.contract_only` is `true` in both states.

### Safety invariants

- [ ] `app_runtime_execution_enabled` is always `false` (eligible and blocked).
- [ ] `app_runtime_writes_enabled` is always `false` (eligible and blocked).
- [ ] `app_runtime_reads_enabled` is always `false` (eligible and blocked).
- [ ] `network_reads_attempted` is always `false` (eligible and blocked).
- [ ] `network_writes_attempted` is always `false` (eligible and blocked).
- [ ] `no_changes_made` is always `true` (eligible and blocked).
- [ ] `airtable_client_called` is always `false` (eligible and blocked).
- [ ] `safety_snapshot.airtable_client_called` is always `false`.
- [ ] `safety_snapshot.write_gate_disabled` is always `true`.
- [ ] All runtime flags in snapshot are always `false` when eligible.
- [ ] `evaluate_write_gate()` returns `Disabled` after eligible result.
- [ ] `evaluate_write_gate()` returns `Disabled` after blocked result.

### Planned phases

- [ ] 5 planned phases present when eligible (`LE2ERTC-PHASE-01` through `LE2ERTC-PHASE-05`).
- [ ] All phase statuses are `Planned` when eligible.
- [ ] 5 phases present when blocked; all statuses are `NotExecuted`.
- [ ] `planned_phase_count` equals 5 in both states.

### Prerequisites list

- [ ] 9 prerequisites present when eligible (`LE2ERTC-PRE-01` through `LE2ERTC-PRE-09`).
- [ ] All prerequisites have `Ready` status when eligible.
- [ ] `total_prerequisite_count` equals 9 when eligible.

### Required future-live conditions

- [ ] `required_future_live_conditions` is non-empty when eligible.
- [ ] `required_future_live_conditions` is non-empty when blocked.
- [ ] Contains `disposable sandbox-only base required`.
- [ ] Contains `attachment binary handling remains disabled`.
- [ ] Contains `app runtime restore execution remains disabled`.
- [ ] Contains `final non-success guard`.

### Serialization safety

- [ ] No `pat_` token prefix in serialized JSON.
- [ ] No `"token"` key in serialized JSON.
- [ ] No absolute path (`/Users/`, `/home/`, `/tmp/`) in serialized JSON.
- [ ] No `"fields":{` in serialized JSON.
- [ ] No `"records":[{` in serialized JSON.
- [ ] No `"body":{` or `"statusCode"` in serialized JSON.
- [ ] No `"oldRecordId"` or `"newRecordId"` in serialized JSON.
- [ ] No `rec_old_` or `rec_new_` in serialized JSON.
- [ ] No `cdn.airtable.com` or `attachmentUrl` in serialized JSON.
- [ ] No `"succeeded"`, `restoreComplete`, `restoreSuccess`, or `executionReady` in serialized JSON.

### Message / pending work

- [ ] No Tauri command introduced.
- [ ] No real Airtable client called (`network_reads_attempted`, `network_writes_attempted`, `airtable_client_called` all `false`).

---

## LE2ERTSH — Live E2E Restore Sandbox Harness

### Opt-in guard

- [ ] `all_required_env_vars_present()` returns `false` when `AIRBRIDGE_ENABLE_LIVE_E2E_RESTORE_TEST` is not `"true"`.
- [ ] `all_required_env_vars_present()` returns `false` when any of the 9 required env vars is missing.
- [ ] `all_required_env_vars_present()` returns `true` only when all 9 required env vars are present and enable flag is `"true"`.
- [ ] Live `#[ignore]` test returns without panicking when any required env var is missing.
- [ ] Default `cargo test` skips the live `#[ignore]` test.

### Pre-call E2E contract verification

- [ ] E2E contract returns `EligibleButNotExecuted` before any live call.
- [ ] `contract_only` is `true` before any live call.
- [ ] `airtable_client_called` is `false` before any live call.
- [ ] `network_writes_attempted` is `false` before any live call.
- [ ] `network_reads_attempted` is `false` before any live call.
- [ ] `app_runtime_execution_enabled` is `false` before any live call.
- [ ] `app_runtime_writes_enabled` is `false` before any live call.
- [ ] `app_runtime_reads_enabled` is `false` before any live call.
- [ ] `no_changes_made` is `true` before any live call.
- [ ] Adapter chain returns `MockRunNotExecuted` before any live call.

### Phase 1 — Schema write

- [ ] Schema contract returns `EligibleButNotExecuted` before phase 1 live call.
- [ ] `evaluate_write_gate()` returns `Disabled` before phase 1 live call.
- [ ] `CreateTableOutcome.table_name` matches requested name.
- [ ] `CreateTableOutcome.table_id` is non-empty.
- [ ] Serialized phase 1 outcome does not contain `pat_`.
- [ ] `evaluate_write_gate()` returns `Disabled` after phase 1 live call.

### Phase 2 — Record write

- [ ] Record contract returns `EligibleButNotExecuted` before phase 2 live call.
- [ ] `evaluate_write_gate()` returns `Disabled` before phase 2 live call.
- [ ] `CreateSandboxRecordOutcome.record_created` is `true`.
- [ ] `CreateSandboxRecordOutcome.record_count` equals 1.
- [ ] Serialized phase 2 outcome does not contain `pat_`.
- [ ] Serialized phase 2 outcome does not contain `"id"`.
- [ ] `evaluate_write_gate()` returns `Disabled` after phase 2 live call.

### Phase 3 — Linked update

- [ ] Linked update contract returns `EligibleButNotExecuted` before phase 3 live call.
- [ ] `evaluate_write_gate()` returns `Disabled` before phase 3 live call.
- [ ] Target record created successfully (ID held locally, never printed).
- [ ] Source record created successfully (ID held locally, never printed).
- [ ] `UpdateLinkedSandboxRecordOutcome.record_updated` is `true`.
- [ ] `UpdateLinkedSandboxRecordOutcome.linked_target_count` equals 1.
- [ ] Serialized phase 3 outcome does not contain `pat_`.
- [ ] Serialized phase 3 outcome does not contain `"id"`.
- [ ] `evaluate_write_gate()` returns `Disabled` after phase 3 live call.

### Phase 4 — Final validation read

- [ ] FV contract returns `EligibleButNotExecuted` before phase 4 live call.
- [ ] `evaluate_write_gate()` returns `Disabled` before phase 4 live call.
- [ ] `SandboxValidationReadOutcome.table_reachable` is `true`.
- [ ] `has_records` is `true` when `AIRBRIDGE_SANDBOX_EXPECTED_MIN_RECORD_COUNT` is set.
- [ ] `min_count_satisfied` is `true` when `AIRBRIDGE_SANDBOX_EXPECTED_MIN_RECORD_COUNT` is set.
- [ ] Serialized phase 4 outcome does not contain `pat_`.
- [ ] Serialized phase 4 outcome does not contain `rec`.
- [ ] `evaluate_write_gate()` returns `Disabled` after phase 4 live call.

### Phase 5 — Final non-runtime guard

- [ ] `evaluate_write_gate()` returns `Disabled` after all phases complete.
- [ ] `app_runtime_execution_enabled` remains `false` in final E2E contract result.
- [ ] `app_runtime_writes_enabled` remains `false` in final E2E contract result.
- [ ] `app_runtime_reads_enabled` remains `false` in final E2E contract result.
- [ ] `airtable_client_called` remains `false` in final E2E contract result.
- [ ] `no_changes_made` remains `true` in final E2E contract result.
- [ ] Serialized final E2E result does not contain `restoreSuccess`.
- [ ] Serialized final E2E result does not contain `restoreComplete`.
- [ ] Serialized final E2E result does not contain `"succeeded"`.

### Serialization safety (all phases)

- [ ] No `pat_` token prefix in any serialized outcome.
- [ ] No `"id"` key in any serialized record outcome.
- [ ] No `rec` in any serialized validation outcome.
- [ ] No `restoreSuccess`, `restoreComplete`, or `"succeeded"` in any serialized result.

### Default non-ignored tests

- [ ] 19 default tests pass in standard `cargo test`.
- [ ] Each missing env var test returns without panicking.
- [ ] Write gate invariant test passes.
- [ ] All four phase contract eligibility tests pass.
- [ ] Adapter chain mock run test passes.
- [ ] No Tauri command test passes.
- [ ] No attachment test passes.
- [ ] No restore success state test passes.
