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

## Notes and Failures

Record any failures here with a brief description and steps to reproduce.

| Item | Status | Notes |
|------|--------|-------|
| | | |
