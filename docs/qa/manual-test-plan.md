# Manual Test Plan

## Prerequisites

Before executing this plan:

- A release build (or development build) of AirBridge is installed and launchable on the target platform.
- A valid Airtable personal access token is available that has read access to at least one test base.
- A second token with write access to an empty test base is available for restore tests.
- The tester has access to the `fixtures/` directory for offline validation steps.
- All previous test results from this platform have been cleared (no stale state from prior runs).

---

## Smoke Test on Install

**TC-SMOKE-01: Application launches**

- Preconditions: AirBridge installed from a release artifact.
- Steps:
  1. Launch AirBridge from the system application launcher or double-clicking the installed app.
  2. Observe the startup sequence.
- Expected result: The application opens to the home screen with no crash dialogs, no console error popups, and no missing-asset placeholders.

**TC-SMOKE-02: Version string is correct**

- Preconditions: Application launched.
- Steps:
  1. Open Settings (gear icon or menu).
  2. Locate the "About" or "Version" section.
- Expected result: The version string matches the release tag (e.g., `0.1.0`). No "dev" or "debug" suffix is present.

**TC-SMOKE-03: No network traffic on idle**

- Preconditions: AirBridge launched, no user action taken, network monitor running (e.g., Little Snitch, Wireshark, or Windows Resource Monitor).
- Steps:
  1. Leave the application idle for 30 seconds.
- Expected result: No outbound network connections are initiated by the application process.

---

## Connections Flow

**TC-CONN-01: Add a valid connection**

- Preconditions: Application open, no connections configured.
- Steps:
  1. Click "Add Connection" (or equivalent).
  2. Enter a display name: "Test Connection".
  3. Paste a valid Airtable personal access token with read permissions.
  4. Click "Save" or "Connect".
- Expected result: The connection appears in the connections list with the display name. No token value is shown in the UI after saving.

**TC-CONN-02: Add a connection with an invalid token**

- Preconditions: Application open.
- Steps:
  1. Click "Add Connection".
  2. Enter a display name and a deliberately invalid token string (e.g., "notavalidtoken").
  3. Click "Save".
- Expected result: An error message is shown explaining the token is invalid or could not be verified. The connection is not saved.

**TC-CONN-03: Edit a connection's display name**

- Preconditions: At least one connection saved.
- Steps:
  1. Open the context menu or edit action for the existing connection.
  2. Change the display name to "Renamed Connection".
  3. Save.
- Expected result: The connection list shows the new display name. The token is unchanged.

**TC-CONN-04: Delete a connection**

- Preconditions: At least one connection saved.
- Steps:
  1. Select the connection.
  2. Choose "Delete" or equivalent.
  3. Confirm the deletion in the confirmation dialog if one appears.
- Expected result: The connection is removed from the list. The application returns to the empty state if no other connections exist.

---

## Backup Flow

**TC-BACK-01: Backup a base — happy path**

- Preconditions: A valid connection with read access is configured. A base with at least two tables and several records is accessible via the token.
- Steps:
  1. Select the connection.
  2. Choose the target base from the list of available bases.
  3. Choose an output location using the file picker.
  4. Click "Start Backup".
  5. Wait for the operation to complete.
- Expected result: A `.airbridge` package (or equivalent directory/archive) is written to the chosen location. A success summary is shown with table count and record count. The counts match what is visible in Airtable.

**TC-BACK-02: Backup progress is displayed**

- Preconditions: Same as TC-BACK-01 but using a base with a large number of records (100+).
- Steps:
  1. Start a backup.
  2. Observe the progress UI during the operation.
- Expected result: Progress is shown (percentage, record count, or similar). The UI does not appear frozen. Cancel is available.

**TC-BACK-03: Cancel a backup mid-flight**

- Preconditions: A backup is in progress.
- Steps:
  1. Click "Cancel" during an active backup.
- Expected result: The backup stops. A cancellation message is shown. No incomplete or corrupted package is left at the output location, or the partial package is clearly marked as incomplete.

**TC-BACK-04: Backup with schema-only option**

- Preconditions: Valid connection and base available.
- Steps:
  1. Start a backup with the "Schema only" option enabled.
  2. Complete the backup.
  3. Open the resulting package and inspect its contents.
- Expected result: The package contains `manifest.json` (with `schemaOnly: true`) and `schema.json` but no `records.jsonl` file, or the records file is empty.

**TC-BACK-05: Output location already exists**

- Preconditions: An output path that already contains a file or directory of the same name exists.
- Steps:
  1. Start a backup targeting the existing location.
- Expected result: The application either shows the user a confirmation dialog before overwriting, or creates the new backup with a disambiguating suffix. It does not silently overwrite without warning.

**TC-BACK-06: Backup with no read permission**

- Preconditions: A connection token that has been revoked or has insufficient scope.
- Steps:
  1. Attempt a backup using this token.
- Expected result: A clear error message is shown explaining the permission problem. No empty or partial package is created.

---

## Restore Flow

**TC-REST-00: Restore plan preview — no token, no writes**

- Preconditions: A valid `.airbridge` backup package exists. No Airtable connection required.
- Steps:
  1. Navigate to the Restore page.
  2. In the "Restore Plan Preview" panel, click "Choose File" and select the `.airbridge` file.
  3. Verify the filename is displayed (not the full directory path).
  4. Select a target mode ("New base" or "Empty existing base").
  5. Click "Generate Restore Plan".
- Expected result:
  - A plan is shown with status badge, package summary, table plans, and field compatibility badges.
  - The "No Airtable changes were made." notice is visible.
  - No token was requested at any point.
  - No restore execution button is rendered.
  - The full file path does not appear anywhere in the UI.

**TC-REST-00A: Restore plan preview — blocked for invalid package**

- Preconditions: A corrupted or invalid `.airbridge` file is available (or use a renamed non-package file).
- Steps:
  1. In the "Restore Plan Preview" panel, select the invalid file.
  2. Click "Generate Restore Plan".
- Expected result: A "Blocked" badge is shown with an error message. The app does not crash. The "No Airtable changes were made." notice is still present.

**TC-REST-00A2: Schema creation plan — no token, no writes**

- Preconditions: A valid `.airbridge` package has been inspected and a dry-run plan has been generated (status: ready or ready with warnings).
- Steps:
  1. In the "Schema Creation Plan" section, observe the panel.
  2. Confirm there is no token input field and no "Start Restore" button.
  3. Click "Preview Schema Creation Plan".
  4. Observe the result.
- Expected result: Table creation steps are shown in order. Field steps are listed. Deferred linked fields appear separately. Manual-action fields are listed. Warnings appear for attachments or unsupported fields where applicable. "No Airtable changes were made. This is a plan only." is visible. No restore execution button is shown. Status badge reads "Ready" or "Ready with Warnings".

**TC-REST-00A3: Schema creation plan — blocked when dry-run is blocked**

- Preconditions: Application open on Restore page. No dry-run plan has been generated.
- Steps:
  1. In the "Schema Creation Plan" section, observe the button state.
- Expected result: The generate button is disabled. A "Generate a restore plan preview first." message is shown.

**TC-REST-00B: Restore execution gate — prerequisites checklist**

- Preconditions: Application open on Restore page. No file has been inspected yet.
- Steps:
  1. Scroll to the "Restore Execution" section.
  2. Observe the prerequisites checklist.
  3. Observe the "not enabled" notice.
  4. Observe the "Attempt Restore" button state.
- Expected result: Prerequisites checklist shows five items, all in an incomplete state. "Restore execution is not enabled in this version" notice is visible. "Attempt Restore" button is `disabled`. "No Airtable changes" is mentioned in the notice.

**TC-REST-00C: Restore execution gate — token field is masked**

- Preconditions: Application open on Restore page.
- Steps:
  1. Click in the "Access Token" field in the "Restore Execution" section.
  2. Type any text.
- Expected result: The typed text is masked (shown as dots or asterisks). The text does not appear in plain view anywhere on the page.

**TC-REST-00D: Restore execution gate — full attempt with all prerequisites**

- Preconditions: A valid `.airbridge` package has been inspected (status: valid or warning). A dry-run plan has been generated (status: ready or ready with warnings). A target mode is selected.
- Steps:
  1. In the "Restore Execution" section, enter any non-empty string in the token field.
  2. In the confirmation field, type `RESTORE BACKUP` exactly.
  3. Observe the "Attempt Restore" button state — it should now be enabled.
  4. Click "Attempt Restore".
- Expected result: A result panel appears. The status badge reads "Disabled". The message states the write engine is not enabled. "No Airtable changes were made." is visible. No record or base was created in Airtable. The token field is empty after the attempt.

**TC-REST-00E: Restore execution gate — wrong confirmation keeps button disabled**

- Preconditions: Package inspected, dry-run plan ready, token non-empty.
- Steps:
  1. Type `restore backup` (lowercase) in the confirmation field.
  2. Observe the button state.
  3. Clear the field and type `RESTORE` alone.
  4. Observe the button state.
- Expected result: The "Attempt Restore" button remains `disabled` in both cases. Only the exact string `RESTORE BACKUP` enables the button.

**TC-REST-00F: Restore execution gate — cancel clears token**

- Preconditions: A successful gate attempt has completed (TC-REST-00D above).
- Steps:
  1. Click "Cancel" in the result area.
- Expected result: The result panel disappears. The token input is empty. The form returns to idle state.

**TC-REST-01: Dry-run restore**

- Preconditions: A valid `.airbridge` backup package exists (can use fixture data loaded via "Open backup"). A connection with write access to an empty target base is available.
- Steps:
  1. Open the backup package.
  2. Select "Restore" and choose the target base.
  3. Enable "Dry run" mode.
  4. Click "Start".
- Expected result: A report is shown listing what would be created (tables, fields, records) with counts. No writes are made to Airtable. The report is exportable or copyable.

**TC-REST-02: Full restore to empty base**

- Preconditions: Same as TC-REST-01 but with dry run disabled and a confirmed empty target base.
- Steps:
  1. Open the backup package.
  2. Select the target base.
  3. Click "Start Restore".
  4. Wait for completion.
- Expected result: A success summary is shown. The target base in Airtable now contains the tables, fields, and records from the backup. Record counts match the backup manifest.

**TC-REST-03: Restore with linked record remapping**

- Preconditions: The `linked-records-base` fixture package is opened. Target is a fresh empty base.
- Steps:
  1. Perform a full restore.
- Expected result: Linked record relationships are re-established in the target base. No "broken link" records remain. The restore report indicates that record IDs were remapped.

**TC-REST-04: Restore with unsupported field types**

- Preconditions: A backup containing a field type not supported by the restore target (e.g., a computed formula field) is opened.
- Steps:
  1. Attempt a restore.
- Expected result: The unsupported fields are listed in the report. They are skipped without failing the entire restore. A warning is shown, not a hard error.

**TC-REST-05: Cancel a restore mid-flight**

- Preconditions: A restore is in progress.
- Steps:
  1. Click "Cancel".
- Expected result: The restore stops. The restore report shows which records were written before cancellation. The user is advised to inspect the partial state in Airtable.

---

## Reports

**TC-REP-01: Backup summary report is accurate**

- Preconditions: A completed backup exists.
- Steps:
  1. Open the backup package.
  2. View the summary report.
- Expected result: Table names, field names, field types, and record counts are all displayed correctly and match the source base.

**TC-REP-02: Schema diff between two backups**

- Preconditions: Two backups of the same base taken at different times exist, where the schema changed between them.
- Steps:
  1. Use the "Compare" or "Diff" feature to compare the two packages.
- Expected result: Added and removed fields or tables are highlighted. The diff is readable and accurate.

**TC-REP-03: Export report to file**

- Preconditions: A backup summary report is open.
- Steps:
  1. Click "Export" and choose a location.
- Expected result: A file is written at the chosen location. The file content matches what is shown in the UI.

---

## Settings

**TC-SET-01: App data directory is shown**

- Preconditions: Application open, Settings visible.
- Steps:
  1. Navigate to the Settings view.
  2. Locate the "App data location" entry.
- Expected result: The path shown is the correct platform-appropriate path for the current OS user.

**TC-SET-02: Clear all data**

- Preconditions: At least one connection and at least one cached backup index exist.
- Steps:
  1. Click "Clear all data" (or equivalent) in Settings.
  2. Confirm the deletion in the confirmation dialog.
- Expected result: All stored connections and cached data are removed. The application returns to a fresh state on next launch.

---

## Logs

**TC-LOG-01: Log file is written**

- Preconditions: Application has been running and at least one operation (backup or restore) has been performed.
- Steps:
  1. Navigate to Settings and open the log file location, or use the "View logs" button.
- Expected result: A log file exists at the displayed location. It contains timestamped entries for the performed operations.

**TC-LOG-02: Log file does not contain tokens or record values**

- Preconditions: A backup has been performed and the log file is accessible.
- Steps:
  1. Open the log file in a text editor.
  2. Search for the token string used during the test.
  3. Search for a known field value from the backed-up records.
- Expected result: Neither the token nor any record field value appears in the log file. Table and field names may appear, but record data must not.

---

## Backup Planning (Dry-Run)

**TC-PLAN-01: Base selector is populated**

- Preconditions: A connection has been verified and accessible bases were returned.
- Steps:
  1. Navigate to the Backups page.
  2. Observe the "Backup Planning" card.
- Expected result: The base selector lists the same bases visible on the Connection page. If no bases were found, the selector shows "No accessible bases".

**TC-PLAN-02: Schema loads for a selected base**

- Preconditions: At least one base is available. An active session token is in memory.
- Steps:
  1. Select a base in the Backup Planning card.
  2. Click "Load Schema".
- Expected result: A schema summary appears showing table count and field compatibility counts. No error is shown.

**TC-PLAN-03: Generate Backup Plan produces a dry-run result**

- Preconditions: TC-PLAN-02 passed (schema is loaded).
- Steps:
  1. Click "Generate Backup Plan".
- Expected result:
  - The plan result area appears.
  - "No backup file has been created yet." is visible.
  - Table count and field count are shown.
  - Compatibility counts (restorable / metadata-only / unknown) are shown.
  - Estimated API read pages is shown (may read "unknown").

**TC-PLAN-04: Attachment warnings appear for bases with attachment fields**

- Preconditions: The selected base contains at least one `multipleAttachments` field.
- Steps:
  1. Load schema and generate a plan as in TC-PLAN-02 and TC-PLAN-03.
- Expected result: A warning notice with code `ATTACHMENT_METADATA_ONLY` is listed under "Notices". Severity label indicates a warning (not info).

**TC-PLAN-05: Linked record warnings appear for bases with linked record fields**

- Preconditions: The selected base contains at least one `multipleRecordLinks` field.
- Steps:
  1. Load schema and generate a plan as in TC-PLAN-02 and TC-PLAN-03.
- Expected result: A warning notice with code `LINKED_RECORD_REMAPPING` is listed. The table name is shown next to the code.

**TC-PLAN-06: No backup file is created**

- Preconditions: TC-PLAN-03 passed.
- Steps:
  1. After generating a plan, check the filesystem for any new `.airbridge` or backup-related files.
- Expected result: No new files have been written. The app directory and Documents folder are unchanged.

**TC-PLAN-07: Token does not appear in plan output**

- Preconditions: A plan has been generated.
- Steps:
  1. Open DevTools (if available) and inspect the IPC response from `create_backup_plan`.
  2. Search the response JSON for the token string.
- Expected result: The token string does not appear anywhere in the plan JSON.

---

## Records Export Planning

**TC-EXPORT-01: Records export plan section is visible**

- Preconditions: The Backups page loads without error.
- Steps:
  1. Open the Backups page.
  2. Locate the "Records Export Plan" section.
- Expected result: The section and "Generate Records Export Plan" button are visible. The button is disabled before a backup plan is generated.

**TC-EXPORT-02: Button enabled after backup plan generated**

- Preconditions: TC-PLAN-03 passed (backup plan generated).
- Steps:
  1. Generate a backup plan in the Backup Planning section.
  2. Observe the "Generate Records Export Plan" button.
- Expected result: The button becomes enabled after the backup plan is available.

**TC-EXPORT-03: Export plan shows known and unknown record counts**

- Preconditions: A backup plan exists with at least one table having a known record count and one with an unknown count.
- Steps:
  1. Generate a records export plan.
  2. Inspect the table rows in the result.
- Expected result: Known counts display the numeric value; unknown counts display "unknown".

**TC-EXPORT-04: Export plan shows estimated page count for known count**

- Preconditions: A backup plan has a table with a known record count (e.g. 250).
- Steps:
  1. Generate a records export plan.
  2. Read the estimated pages for that table.
- Expected result: The estimate shows 3 pages for 250 records at page size 100.

**TC-EXPORT-05: JSONL output path is shown and is relative**

- Preconditions: A records export plan has been generated.
- Steps:
  1. Inspect each table row's output path.
- Expected result: The path format is `tables/{tableId}/records.jsonl`. No leading `/`. No `Users/` or `home/` components.

**TC-EXPORT-06: Linked record and attachment policies are shown**

- Preconditions: The backup plan includes tables with linked record and attachment fields.
- Steps:
  1. Generate a records export plan.
  2. Inspect the table rows for policy labels.
- Expected result: Linked record tables show `remappingRequiredForRestore`. Attachment tables show `metadataOnly`.

**TC-EXPORT-07: Planning-only notice is displayed**

- Preconditions: A records export plan has been generated.
- Steps:
  1. Read the notice text at the top of the plan result.
- Expected result: The notice states that no records have been fetched and no backup file has been written.

---

## Backup Job Pipeline (UI — V0.1 Status Section)

**TC-JOB-01: Backup Job Pipeline section is visible on the Backups page**

- Preconditions: Application open, Backups page navigated to.
- Steps:
  1. Click "Backups" in the navigation.
  2. Scroll to the "Backup Job Pipeline" section.
- Expected result: The section is present and rendered without errors.

**TC-JOB-02: Section states live backup creation is not enabled**

- Preconditions: Backups page open.
- Steps:
  1. Read the text in the "Backup Job Pipeline" section.
- Expected result: The text states that live backup creation is not enabled yet.

**TC-JOB-03: Section states no file is created from the screen**

- Preconditions: Backups page open.
- Steps:
  1. Read the text in the "Backup Job Pipeline" section.
- Expected result: The text contains "no file is created from this screen" (or equivalent).

**TC-JOB-04: No enabled production backup-trigger button exists**

- Preconditions: Backups page open.
- Steps:
  1. Inspect all buttons on the Backups page.
  2. Look for any button labelled "Start Backup", "Run Backup", or "Create Backup".
- Expected result: No such button is enabled. Any backup-trigger button present must be disabled.

---

## Safe Backup Command Contract

These test cases cover the command contract layer: confirmation enforcement, output path validation, and response safety. All automated — manual verification is for regression confidence only.

**TC-CONTRACT-01: Confirmation phrase is required**

- Preconditions: The `run_backup_job` command is callable (tests only).
- Steps:
  1. Call `run_backup_job` with `confirmation` set to an empty string.
  2. Call `run_backup_job` with `confirmation` set to `"create backup"` (lowercase).
  3. Call `run_backup_job` with `confirmation` omitted entirely.
- Expected result: All three calls return a response with `success: false` and a safety error with code `CONFIRMATION_REQUIRED`. No file is written.

**TC-CONTRACT-02: Correct confirmation phrase is accepted**

- Preconditions: The `run_backup_job` command is callable (tests only). A valid `.airbridge` output path in a temp directory is available.
- Steps:
  1. Call `run_backup_job` with `confirmation` set to `"CREATE BACKUP"` (exact).
- Expected result: The command proceeds past the confirmation gate. Any failure at this point is from path validation or the orchestrator, not the confirmation check.

**TC-CONTRACT-03: Wrong extension rejected before any write**

- Preconditions: Correct confirmation phrase is supplied.
- Steps:
  1. Call `run_backup_job` with `outputPath` set to `/tmp/backup.zip`.
- Expected result: Response has `success: false`, `pathValidation.valid: false`, `pathValidation.errorCode: "WRONG_EXTENSION"`. No file is created at `/tmp/backup.zip`.

**TC-CONTRACT-04: Missing parent directory rejected**

- Preconditions: Correct confirmation phrase is supplied.
- Steps:
  1. Call `run_backup_job` with `outputPath` set to `/tmp/nonexistent-dir/backup.airbridge`.
- Expected result: `pathValidation.errorCode: "PARENT_NOT_FOUND"`. No file is created.

**TC-CONTRACT-05: Traversal in path rejected**

- Preconditions: Correct confirmation phrase is supplied.
- Steps:
  1. Call `run_backup_job` with `outputPath` set to `/tmp/../etc/backup.airbridge`.
- Expected result: `pathValidation.errorCode: "TRAVERSAL_DETECTED"`. No file is created.

**TC-CONTRACT-06: No token in command response**

- Preconditions: A mock-transport run completes successfully.
- Steps:
  1. Serialise `RunBackupCommandResponse` to JSON.
  2. Search the JSON string for the token sentinel value.
- Expected result: The token value does not appear anywhere in the serialised response.

**TC-CONTRACT-07: No absolute path in command response**

- Preconditions: A mock-transport run completes with a valid output path in a temp directory.
- Steps:
  1. Inspect `response.packageFilename`.
- Expected result: `packageFilename` contains only the filename component (e.g., `backup.airbridge`). It does not contain the parent directory path.

**TC-CONTRACT-08: `validate_backup_output_path` has no file side effects**

- Preconditions: None.
- Steps:
  1. Call `validate_backup_output_path` with a path to a non-existent file.
  2. Check whether that file now exists.
- Expected result: The file does not exist. The command creates no files.

**TC-CONTRACT-09: Mock service confirmation enforcement**

- Preconditions: Mock `AirBridgeService` is instantiated.
- Steps:
  1. Call `mockService.runBackupJob` with wrong `confirmation`.
- Expected result: `success: false`, `safetyErrors[0].code: "CONFIRMATION_REQUIRED"`.

**TC-CONTRACT-10: UI section visible on Backups page**

- Preconditions: Backups page is rendered.
- Steps:
  1. Read the "Backup Job Pipeline" section.
- Expected result: The section states the safe command contract is ready and that live backup execution is not enabled yet. The text references the explicit confirmation requirement and output path validation.

---

## Backup File Picker and Confirmation Flow

**TC-PICKER-01: File picker opens native save dialog**

- Preconditions: Backup plan and records export plan are generated. Backups page is open.
- Steps:
  1. Click the "Choose File…" button in the Run Backup section.
- Expected result: The OS native save dialog opens. The suggested filename ends in `.airbridge`. The dialog filters to `*.airbridge` files.

**TC-PICKER-02: Cancel leaves state unchanged**

- Preconditions: Same as TC-PICKER-01.
- Steps:
  1. Click "Choose File…".
  2. Cancel the dialog without selecting a file.
- Expected result: No filename is displayed. The Run Backup button remains disabled.

**TC-PICKER-03: Filename-only display**

- Preconditions: Valid `.airbridge` path selected.
- Steps:
  1. Select a `.airbridge` file in a nested directory.
  2. Check the displayed text next to the "Choose File…" button.
- Expected result: Only the filename is shown (e.g. `my-base.airbridge`). The directory path is not shown.

**TC-PICKER-04: Absolute path never visible on screen**

- Preconditions: Valid path selected.
- Steps:
  1. Inspect all visible text on the Backups page.
- Expected result: No absolute filesystem path (no `/Users/`, no `/home/`, no `C:\`) appears anywhere in the UI.

**TC-PICKER-05: Invalid extension shows error**

- Preconditions: Backup plan and export plan exist.
- Steps:
  1. If the OS allows it, select a `.zip` file.
- Expected result: An extension error is shown. The Run Backup button stays disabled.

**TC-PICKER-06: Token field is masked**

- Preconditions: Backups page is open.
- Steps:
  1. Click inside the token field and type a value.
- Expected result: Characters are masked (password input). The typed value is not visible as plain text.

**TC-PICKER-07: Token does not appear outside token field**

- Preconditions: Token entered.
- Steps:
  1. Type a recognizable token value.
  2. Inspect all other elements on the page.
- Expected result: The token value is not rendered anywhere outside the masked input field.

**TC-PICKER-08: Run button gating**

- Preconditions: Backups page is open.
- Steps:
  1. Verify button is disabled with no plans.
  2. Generate backup plan and export plan.
  3. Verify button is still disabled.
  4. Select a valid `.airbridge` path.
  5. Verify button is still disabled.
  6. Enter a token.
  7. Verify button is still disabled.
  8. Type "CREATE BACKUP" in the confirmation field.
  9. Verify button is now enabled.
- Expected result: Button becomes enabled only after all prerequisites are satisfied.

**TC-PICKER-09: Token and confirmation cleared after run**

- Preconditions: All prerequisites satisfied. Run executed.
- Steps:
  1. Click "Run Backup".
  2. Wait for result.
  3. Check the token field and confirmation field.
- Expected result: Both fields are empty.

**TC-PICKER-10: Result shows filename only**

- Preconditions: Successful backup run.
- Steps:
  1. Inspect the result card after a successful run.
- Expected result: The package filename is shown (e.g. `my-base.airbridge`). The directory path is not shown.

**TC-PICKER-11: Token not in result**

- Preconditions: Backup run completed (success or failure).
- Steps:
  1. Inspect the result card.
  2. Search for the token value on the page.
- Expected result: The token value does not appear in the result or anywhere else on the page.

**TC-PICKER-12: Safety copy visible**

- Preconditions: Backups page is open.
- Steps:
  1. Read the Run Backup panel.
- Expected result: All three safety statements are visible: "The full output path is not displayed", "The token is not stored", "Backup creation runs only after confirmation".

---

## Record Import Plan Test Cases

**TC-IMPORT-01: Panel renders on Restore page**

- Preconditions: Restore page is open.
- Steps:
  1. Open the Restore page.
- Expected result: The "Record Import Plan" section is visible below the schema plan section.

**TC-IMPORT-02: Button disabled before inspection**

- Preconditions: No package has been inspected.
- Steps:
  1. Observe the "Preview Record Import Plan" button.
- Expected result: Button is disabled. "Inspect a package first." message is shown.

**TC-IMPORT-03: Button disabled when dry-run blocked**

- Preconditions: Package inspected, but dry-run is not yet generated.
- Steps:
  1. Observe the button.
- Expected result: Button is disabled. "Generate a restore plan preview first." message is shown.

**TC-IMPORT-04: Button disabled when schema plan blocked**

- Preconditions: Dry-run ready, but schema plan is not yet generated.
- Steps:
  1. Observe the button.
- Expected result: Button is disabled. "Generate a schema creation plan first." message is shown.

**TC-IMPORT-05: Plan generates successfully**

- Preconditions: Package inspected, dry-run generated (ready), schema plan generated (ready).
- Steps:
  1. Click "Preview Record Import Plan".
- Expected result: Plan result section appears. Status badge shows "Ready" or "Ready with Warnings".

**TC-IMPORT-06: Table import list is shown**

- Preconditions: Plan generated successfully.
- Steps:
  1. Read the table import list.
- Expected result: Each table appears with its name, record count or "unknown", and batch count.

**TC-IMPORT-07: Batch size is 10**

- Preconditions: Plan generated for a table with known record count.
- Steps:
  1. Read the batch count for a table with 25 records.
- Expected result: 3 create batches shown (25 / 10 = 3 batches).

**TC-IMPORT-08: Attachment metadata notice is shown**

- Preconditions: Package has tables with attachment fields.
- Steps:
  1. Read the table import entry for that table.
- Expected result: "attachment fields: metadata only, manual re-attachment required" note is visible.

**TC-IMPORT-09: Linked record second-pass section is shown**

- Preconditions: Package has tables with linked record fields.
- Steps:
  1. Read the linked record update section.
- Expected result: The linked record field name, source table, and linked table are shown.

**TC-IMPORT-10: No token input in panel**

- Preconditions: Restore page is open.
- Steps:
  1. Inspect the "Record Import Plan" section.
- Expected result: No token or API key input field is present anywhere in the panel.

**TC-IMPORT-11: No execute button in panel**

- Preconditions: Restore page is open.
- Steps:
  1. Inspect the "Record Import Plan" section.
- Expected result: No "Start Restore", "Execute", or equivalent button is present.

**TC-IMPORT-12: No changes disclaimer is always shown**

- Preconditions: Plan generated (ready or blocked).
- Steps:
  1. Read the result section.
- Expected result: "No Airtable records were created or modified." message is always present.

**TC-IMPORT-13: Retry policy is visible**

- Preconditions: Plan generated.
- Steps:
  1. Read the retry policy note.
- Expected result: Max retries, initial backoff, and multiplier are visible.

**TC-IMPORT-14: Full path is not shown**

- Preconditions: Plan generated.
- Steps:
  1. Search for absolute path patterns on the page.
- Expected result: No `/Users/`, `/home/`, or `C:\` paths are visible.

---

## Job History (Reports Page)

**TC-HISTORY-01: History panel renders on Reports page**

- Preconditions: Application launched.
- Steps:
  1. Navigate to the Reports page.
  2. Scroll to the "Activity" section.
- Expected result: A list of recent activity items is visible.

**TC-HISTORY-02: Items show kind, status, and timestamp**

- Preconditions: Reports page open with history items visible.
- Steps:
  1. Read each item row.
- Expected result: Each row shows a title, a kind label, a status badge, and a timestamp.

**TC-HISTORY-03: Filename only — no full path**

- Preconditions: Reports page open.
- Steps:
  1. Inspect any item that shows a package filename.
- Expected result: Only the filename component is shown (e.g. `my-backup.airbridge`). No directory path is visible.

**TC-HISTORY-04: No token in activity history**

- Preconditions: Reports page open.
- Steps:
  1. Search the page for any token-like string.
- Expected result: No API token, Bearer string, or `pat…` value appears anywhere in the history panel.

**TC-HISTORY-05: Warning and error counts visible**

- Preconditions: Reports page open with at least one item that has warnings.
- Steps:
  1. Find an item with a warning count.
- Expected result: The warning count is displayed (e.g. "2 warnings").

**TC-HISTORY-06: Validation status shown for inspection items**

- Preconditions: A package inspection item is in the history.
- Steps:
  1. Find the package inspection history row.
- Expected result: The validation status ("valid", "invalid", or "warning") is displayed.

**TC-HISTORY-07: Persistence note is shown**

- Preconditions: Reports page open.
- Steps:
  1. Read the note below the activity list.
- Expected result: A note states that activity is memory-only and does not persist between sessions.

---

## Sandbox Verification (Gate 1)

**TC-SV-01: Disabled notice always visible**

- Preconditions: Restore page open; no backup file selected.
- Steps:
  1. Scroll to the "Sandbox Verification (Gate 1)" section.
- Expected result: A notice reads "Sandbox verification checks local safety conditions only. Restore writes remain disabled in this version." No execute button is shown. No token field is shown.

**TC-SV-02: Verify button triggers check run**

- Preconditions: Restore page open.
- Steps:
  1. Click "Verify sandbox safety".
- Expected result: A result appears showing an overall status (`verified`, `warning`, or `blocked`), 10 check rows, and a safety summary. The status will be `warning` because CHK-10 (live metadata check) is always `skipped`.

**TC-SV-03: CHK-10 always skipped**

- Preconditions: Sandbox verification result is shown.
- Steps:
  1. Locate the "Live metadata check" row in the check list.
- Expected result: Status is `skipped`. Message states the live check is not performed.

**TC-SV-04: Safety summary shows no changes**

- Preconditions: Sandbox verification result is shown.
- Steps:
  1. Locate the safety summary section.
- Expected result: `writesEnabled: No`, `networkWritesAttempted: No`, "No Airtable changes were made.", `liveMetadataCheckPerformed: No`.

**TC-SV-05: Blocked result on unsafe request**

- Preconditions: N/A (mock service only).
- Expected result: When `allowDestructiveOperations: true` is passed to the mock service, result status is `blocked`. A blocked notice is shown in the panel.

**TC-SV-06: No execute button or token**

- Preconditions: Sandbox verification result shown (any status).
- Steps:
  1. Inspect the Sandbox Verification section for buttons and input fields.
- Expected result: No "Execute", "Run Restore", or similar button is present. No token input field is present.

---

## Restore Confirmation Gate (Gate 2)

**TC-CF-01: Writes-disabled notice always visible**

- Preconditions: Restore page open.
- Steps:
  1. Scroll to the "Restore Confirmation (Gate 2)" section.
- Expected result: A writes-disabled notice is shown at the top of the confirmation panel. No execute button is shown. No token field is shown.

**TC-CF-02: Required text is displayed before the input**

- Preconditions: Restore page open.
- Steps:
  1. Observe the confirmation panel.
- Expected result: A required confirmation text label is shown above the input field. The text follows the pattern `RESTORE TO <TARGET>`, `RESTORE <FILENAME>`, or `RESTORE BACKUP` depending on context. The text does not contain path separators or token-format strings.

**TC-CF-03: Validate button disabled when input is empty**

- Preconditions: Restore page open.
- Steps:
  1. Observe the "Validate" button in the confirmation panel before typing.
- Expected result: The button is `disabled`.

**TC-CF-04: Exact match returns confirmed**

- Preconditions: Restore page open. Required text is visible in the panel.
- Steps:
  1. Type the required text exactly as shown in the panel.
  2. Click "Validate".
- Expected result: Status badge shows `confirmed`. An accepted notice is shown. "Writes remain disabled" notice is still visible. No execute button appears.

**TC-CF-05: Wrong case returns rejected**

- Preconditions: Restore page open.
- Steps:
  1. Type the required text in all lowercase (e.g., `restore backup`).
  2. Click "Validate".
- Expected result: Status badge shows `rejected`. A rejected notice is shown. CHK-C03 row shows `failed`.

**TC-CF-06: Blocked sandbox propagates to blocked confirmation**

- Preconditions: Sandbox verification has been run and returned status `blocked`.
- Steps:
  1. In the confirmation panel, type the exact required text.
  2. Click "Validate".
- Expected result: Status badge shows `blocked`, not `confirmed`. A blocked notice is shown explaining the sandbox prerequisite failed.

---

## Restore Target Empty Verification (Gate 3)

**TC-TEV-01: Writes-disabled notice always visible**

- Preconditions: Restore page open.
- Steps:
  1. Scroll to the "Target Empty Verification (Gate 3)" section.
- Expected result: A writes-disabled notice is shown. No execute button is shown. No token field is shown.

**TC-TEV-02: newBase target mode returns verified**

- Preconditions: Restore page open. Target mode is "New base" (the default).
- Steps:
  1. Click "Verify target is empty".
- Expected result: Status badge shows `verified`. A verified notice is shown. "Restore writes remain disabled" is visible in the verified notice.

**TC-TEV-03: Empty existing base (0 tables, 0 records) returns verified**

- Preconditions: Restore page open. Target mode is "Empty existing base". Table count and record count are supplied as 0.
- Steps:
  1. Click "Verify target is empty".
- Expected result: Status badge shows `verified`. TEV-03 and TEV-04 check rows show `passed`.

**TC-TEV-04: Existing base with tables returns blocked**

- Preconditions: Restore page open. Target mode is "Empty existing base". Table count is > 0.
- Steps:
  1. Click "Verify target is empty".
- Expected result: Status badge shows `blocked`. TEV-03 row shows `failed` with a message indicating the table count.

**TC-TEV-05: Counts unknown returns warning**

- Preconditions: Restore page open. Target mode is "Empty existing base". No table or record count provided.
- Steps:
  1. Click "Verify target is empty".
- Expected result: Status badge shows `warning`. TEV-03 and TEV-04 check rows show `warning`. Warning notice is shown.

**TC-TEV-06: No execute button or token input**

- Preconditions: Restore page open with a target empty result (any status).
- Steps:
  1. Inspect the Target Empty Verification section for buttons and input fields.
- Expected result: No "Execute", "Run Restore", or similar button. No password or token input field.

---

## Destructive Operation Policy (Gate 4)

**TC-DOP-01: Writes-disabled notice always visible**

- Preconditions: Restore page open.
- Steps:
  1. Scroll to the "Destructive Operation Policy (Gate 4)" section.
- Expected result: A writes-disabled notice is shown. No execute button is shown. No token field is shown.

**TC-DOP-02: Empty operations list returns compliant**

- Preconditions: Restore page open. No operations declared (default state).
- Steps:
  1. Click "Verify operation policy".
- Expected result: Status badge shows `compliant`. A compliant notice is shown. "Restore writes remain disabled" is visible in the compliant notice.

**TC-DOP-03: Delete operation returns blocked**

- Preconditions: Restore page open. A delete operation is present in the declared operations list.
- Steps:
  1. Click "Verify operation policy".
- Expected result: Status badge shows `blocked`. DOP-02 check row shows `failed`. Blocked operations list shows the operation label.

**TC-DOP-04: Attachment upload returns blocked**

- Preconditions: Restore page open. An attachment upload operation is present in the declared operations list.
- Steps:
  1. Click "Verify operation policy".
- Expected result: Status badge shows `blocked`. DOP-04 check row shows `failed`.

**TC-DOP-05: Create-only operations return compliant**

- Preconditions: Restore page open. Only create-only operations (createTable, createField, createRecord, etc.) declared.
- Steps:
  1. Click "Verify operation policy".
- Expected result: Status badge shows `compliant`. All check rows show `passed`. DOP-05 row shows `passed`.

**TC-DOP-06: No execute button or token input**

- Preconditions: Restore page open with a destructive operation policy result (any status).
- Steps:
  1. Inspect the Destructive Operation Policy section for buttons and input fields.
- Expected result: No "Execute", "Run Restore", or similar button. No password or token input field.

---

## Attachment Upload Policy (Gate 5)

**TC-AUP-01: Writes-disabled notice always visible**

- Preconditions: Restore page open.
- Steps:
  1. Scroll to the "Attachment Upload Policy (Gate 5)" section.
- Expected result: A writes-disabled notice is shown. No execute button is shown. No token field is shown.

**TC-AUP-02: Empty fields list returns compliant**

- Preconditions: Restore page open. No attachment fields declared (default state).
- Steps:
  1. Click "Verify attachment policy".
- Expected result: Status badge shows `compliant`. A compliant notice is shown. "Restore writes remain disabled" is visible in the compliant notice.

**TC-AUP-03: Upload-requested field returns blocked**

- Preconditions: Restore page open. An attachment field with `uploadRequested` intent is declared.
- Steps:
  1. Click "Verify attachment policy".
- Expected result: Status badge shows `blocked`. AUP-02 check row shows `failed`. Blocked fields list shows the field name.

**TC-AUP-04: Download-requested field returns warning**

- Preconditions: Restore page open. An attachment field with `downloadRequested` intent is declared.
- Steps:
  1. Click "Verify attachment policy".
- Expected result: Status badge shows `warning` (not `blocked`). AUP-03 check row shows `warning`.

**TC-AUP-05: All metadata-only fields return compliant**

- Preconditions: Restore page open. Only `metadataOnly` attachment fields declared.
- Steps:
  1. Click "Verify attachment policy".
- Expected result: Status badge shows `compliant`. All check rows show `passed`. AUP-05 row shows `passed`.

**TC-AUP-06: No execute button or token input**

- Preconditions: Restore page open with an attachment upload policy result (any status).
- Steps:
  1. Inspect the Attachment Upload Policy section for buttons and input fields.
- Expected result: No "Execute", "Run Restore", or similar button. No password or token input field.

---

## Schema Record Order Policy (Gate 6)

**TC-SRO-01: Writes-disabled notice always visible**

- Preconditions: Restore page open.
- Steps:
  1. Scroll to the "Schema Record Order Policy (Gate 6)" section.
- Expected result: A writes-disabled notice is shown. No execute button is shown. No token field is shown.

**TC-SRO-02: Empty phase list returns warning**

- Preconditions: Restore page open. No phases declared (default state).
- Steps:
  1. Click "Verify phase ordering".
- Expected result: Status badge shows `warning`. A warning notice is shown.

**TC-SRO-03: Valid phase order returns compliant**

- Preconditions: Restore page open. Phases declared in order: schema → records → linkedRecords → attachments → validation.
- Steps:
  1. Click "Verify phase ordering".
- Expected result: Status badge shows `compliant`. A compliant notice is shown. "Restore writes remain disabled" is visible in the compliant notice. All 5 check rows show `passed`.

**TC-SRO-04: Records before schema returns blocked**

- Preconditions: Restore page open. Phases declared in order: records → schema.
- Steps:
  1. Click "Verify phase ordering".
- Expected result: Status badge shows `blocked`. SRO-03 check row shows `failed`. Ordering violations list shows `records-before-schema`.

**TC-SRO-05: Missing schema with records returns blocked**

- Preconditions: Restore page open. Only a records phase declared (no schema phase).
- Steps:
  1. Click "Verify phase ordering".
- Expected result: Status badge shows `blocked`. SRO-02 check row shows `failed`. Ordering violations list shows `missing-schema-with-records`.

**TC-SRO-06: No execute button or token input**

- Preconditions: Restore page open with a schema record order policy result (any status).
- Steps:
  1. Inspect the Schema Record Order Policy section for buttons and input fields.
- Expected result: No "Execute", "Run Restore", or similar button. No password or token input field.

---

## Sandbox Write Testing Policy (Gate 7)

**TC-SWT-01: Writes-disabled notice always visible**

- Preconditions: Restore page open.
- Steps:
  1. Scroll to the "Sandbox Write Testing Policy (Gate 7)" section.
- Expected result: A writes-disabled notice is shown. No execute button is shown. No token field is shown.

**TC-SWT-02: Production target returns blocked**

- Preconditions: Restore page open. Request has `targetClassification: "production"`.
- Steps:
  1. Click "Verify sandbox testing".
- Expected result: Status badge shows `blocked`. SWT-02 check row shows `failed`.

**TC-SWT-03: No evidence returns blocked**

- Preconditions: Restore page open. No evidence declared in request.
- Steps:
  1. Click "Verify sandbox testing".
- Expected result: Status badge shows `blocked`. SWT-04 check row shows `failed`.

**TC-SWT-04: Partial evidence returns warning**

- Preconditions: Restore page open. Evidence has some fields false (e.g., `schemaPlanReviewed: false`).
- Steps:
  1. Click "Verify sandbox testing".
- Expected result: Status badge shows `warning`. SWT-05 check row shows `warning`.

**TC-SWT-05: Complete evidence with sandbox target returns compliant**

- Preconditions: Restore page open. `targetClassification: "sandbox"`, `sandboxVerificationPassed: true`, all evidence fields true, filename is a basename.
- Steps:
  1. Click "Verify sandbox testing".
- Expected result: Status badge shows `compliant`. A compliant notice is shown. "Restore writes remain disabled" is visible in the compliant notice.

**TC-SWT-06: No execute button or token input**

- Preconditions: Restore page open with a sandbox write testing policy result (any status).
- Steps:
  1. Inspect the Sandbox Write Testing Policy section for buttons and input fields.
- Expected result: No "Execute", "Run Restore", or similar button. No password or token input field.

---

## Live Write Confirmation Policy (Gate 8)

**TC-LWC-01: Writes-disabled notice always visible**

- Preconditions: Restore page open.
- Steps:
  1. Scroll to the "Live Write Confirmation" section.
- Expected result: A notice reads "Live restore writes are disabled." The notice is visible before any verification has been run.

**TC-LWC-02: Required phrase shown before input**

- Preconditions: Restore page open with a target base name set.
- Steps:
  1. Scroll to the "Live Write Confirmation" section.
- Expected result: The required confirmation phrase (e.g., `LIVE RESTORE MY BASE — WRITES REMAIN DISABLED`) is shown in a code block before the input field.

**TC-LWC-03: Wrong text returns rejected**

- Preconditions: Restore page open; all prior gates completed without blocked status.
- Steps:
  1. Type "wrong text" in the confirmation input.
  2. Click "Verify".
- Expected result: Status badge shows `rejected`. LWC-04 check shows `failed`. A rejected notice is shown.

**TC-LWC-04: Correct phrase (exact, case-sensitive) returns confirmed**

- Preconditions: Restore page open; all prior gates completed without blocked status.
- Steps:
  1. Copy the required phrase shown in the panel.
  2. Paste it exactly into the confirmation input.
  3. Click "Verify".
- Expected result: Status badge shows `confirmed`. LWC-04 check shows `passed`. A confirmed notice is shown saying "writes remain disabled".

**TC-LWC-05: Lowercased phrase returns rejected**

- Preconditions: Restore page open; all prior gates completed without blocked status.
- Steps:
  1. Type the required phrase in lowercase.
  2. Click "Verify".
- Expected result: Status badge shows `rejected`.

**TC-LWC-06: Prior blocked gate returns blocked even with correct phrase**

- Preconditions: Restore page open; Gate 7 (sandbox write testing) has a `blocked` status.
- Steps:
  1. Type the correct required phrase.
  2. Click "Verify".
- Expected result: Status badge shows `blocked`. A blocked notice is shown.

**TC-LWC-07: No execute button or token input**

- Preconditions: Restore page open with a live write confirmation policy result (any status).
- Steps:
  1. Inspect the Live Write Confirmation Policy section for buttons and input fields.
- Expected result: No "Execute", "Run Restore", or similar button. No password or token input field (`type="password"` or `name="token"`).

---

## Rate-Limit and Backoff Policy (Gate 9)

**TC-RLB-01: Writes-disabled notice always visible**

- Preconditions: Restore page open.
- Steps:
  1. Scroll to the "Rate-Limit and Backoff Policy" section.
- Expected result: A notice reads "Live restore writes are disabled." The notice is visible before any verification has been run.

**TC-RLB-02: No plan declared returns blocked (2 checks)**

- Preconditions: Restore page open.
- Steps:
  1. Click "Verify rate-limit policy" without providing a plan.
- Expected result: Status badge shows `blocked`. Only 2 check rows are shown (RLB-01 and RLB-02). A blocked notice is shown saying "writes remain disabled".

**TC-RLB-03: Safe plan returns compliant (10 checks)**

- Preconditions: Restore page with a valid rate-limit plan (RPS ≤ 5, batch ≤ 10, 429 handled, retries bounded, backoff declared, stop declared, checkpoint full).
- Steps:
  1. Click "Verify rate-limit policy".
- Expected result: Status badge shows `compliant`. 10 check rows are shown. Plan summary is shown. A compliant notice says "writes remain disabled".

**TC-RLB-04: Checkpoint compatibility warning**

- Preconditions: Restore page with a plan where `checkpointCompatibility` is `"partial"`.
- Steps:
  1. Click "Verify rate-limit policy".
- Expected result: Status badge shows `warning` (not `blocked`). RLB-09 shows a warning status. A warning notice is shown saying "writes remain disabled".

**TC-RLB-05: No execute button or token input**

- Preconditions: Restore page open with a rate-limit policy result (any status).
- Steps:
  1. Inspect the Rate-Limit and Backoff Policy section for buttons and input fields.
- Expected result: No "Execute", "Run Restore", or similar button. No password or token input field (`type="password"` or `name="token"`).

**TC-RLB-06: Compliant result does not enable writes**

- Preconditions: Restore page; safe plan returns `compliant`.
- Steps:
  1. Observe the safety summary and compliant notice.
- Expected result: "Writes enabled: no" is shown. Compliant notice explicitly states "compliance does not start any write operation".

---

## Checkpoint Durability Policy (Gate 10)

**TC-CDP-01: Writes-disabled notice always visible**

- Preconditions: Restore page open.
- Steps:
  1. Scroll to the "Checkpoint Durability Policy" section.
- Expected result: A notice reads "Live restore writes are disabled." The notice is visible before any verification has been run.

**TC-CDP-02: No plan declared returns blocked (2 checks)**

- Preconditions: Restore page open.
- Steps:
  1. Click "Verify checkpoint durability policy" without providing a plan.
- Expected result: Status badge shows `blocked`. Only 2 check rows are shown (CDP-01 and CDP-02). A blocked notice is shown saying "writes remain disabled".

**TC-CDP-03: Complete plan returns compliant (9 checks)**

- Preconditions: Restore page with a complete checkpoint plan (all boolean fields true, remote backend).
- Steps:
  1. Click "Verify checkpoint durability policy".
- Expected result: Status badge shows `compliant`. 9 check rows are shown. Plan summary is shown. A compliant notice says "writes remain disabled".

**TC-CDP-04: Memory backend produces warning**

- Preconditions: Restore page with a plan where `durabilityBackend` is `"memory"` and all other fields are true.
- Steps:
  1. Click "Verify checkpoint durability policy".
- Expected result: Status badge shows `warning` (not `blocked`). CDP-08 shows a warning status. A warning notice is shown saying "writes remain disabled".

**TC-CDP-05: Linked updates without ID mapping blocked**

- Preconditions: Restore page with a plan where `hasLinkedUpdates: true` and `hasIdMappingCheckpoint: false`.
- Steps:
  1. Click "Verify checkpoint durability policy".
- Expected result: Status badge shows `blocked`. CDP-06 shows a failed status.

**TC-CDP-06: No linked updates without ID mapping — passes**

- Preconditions: Restore page with a plan where `hasLinkedUpdates: false` and `hasIdMappingCheckpoint: false`.
- Steps:
  1. Click "Verify checkpoint durability policy".
- Expected result: CDP-06 shows a passed status (ID mapping not required when no linked updates).

**TC-CDP-07: No execute button or token input**

- Preconditions: Restore page open with a checkpoint durability policy result (any status).
- Steps:
  1. Inspect the Checkpoint Durability Policy section for buttons and input fields.
- Expected result: No "Execute", "Run Restore", or similar button. No password or token input field (`type="password"` or `name="token"`).

**TC-CDP-08: Compliant result does not enable writes**

- Preconditions: Restore page; complete plan returns `compliant`.
- Steps:
  1. Observe the safety summary and compliant notice.
- Expected result: "Writes enabled: no" is shown. Compliant notice explicitly states "compliance does not start any write operation".

---

## Final Validation Policy (Gate 11)

**TC-FVP-01: Writes-disabled notice always visible**

- Preconditions: Restore page open.
- Steps:
  1. Scroll to the "Final Validation Policy" section.
- Expected result: A notice reads "Live restore writes are disabled." The notice is visible before any verification has been run.

**TC-FVP-02: No plan declared returns blocked (2 checks)**

- Preconditions: Restore page open.
- Steps:
  1. Click "Verify final validation policy" without providing a plan.
- Expected result: Status badge shows `blocked`. Only 2 check rows are shown (FVP-01 and FVP-02). A blocked notice is shown saying "writes remain disabled".

**TC-FVP-03: Complete plan returns compliant (12 checks)**

- Preconditions: Restore page with a complete final validation plan (all boolean fields true, attachmentValidationMetadataOnly false).
- Steps:
  1. Click "Verify final validation policy".
- Expected result: Status badge shows `compliant`. 12 check rows are shown. Plan summary is shown. A compliant notice says "writes remain disabled" and does not introduce a restore success state.

**TC-FVP-04: Metadata-only attachment validation produces warning**

- Preconditions: Restore page with a plan where `attachmentValidationMetadataOnly: true` and all other fields true.
- Steps:
  1. Click "Verify final validation policy".
- Expected result: Status badge shows `warning` (not `blocked`). FVP-09 shows a warning status. A warning notice is shown saying "writes remain disabled".

**TC-FVP-05: Missing required validation step blocked**

- Preconditions: Restore page with a plan where `hasRecordCountValidation: false`.
- Steps:
  1. Click "Verify final validation policy".
- Expected result: Status badge shows `blocked`. FVP-05 shows a failed status.

**TC-FVP-06: No execute button or token input**

- Preconditions: Restore page open with a final validation policy result (any status).
- Steps:
  1. Inspect the Final Validation Policy section for buttons and input fields.
- Expected result: No "Execute", "Run Restore", or similar button. No password or token input field (`type="password"` or `name="token"`).

**TC-FVP-07: Compliant result does not enable writes**

- Preconditions: Restore page; complete plan returns `compliant`.
- Steps:
  1. Observe the safety summary and compliant notice.
- Expected result: "Writes enabled: no" is shown. Compliant notice explicitly states "compliance does not start any write operation".

**TC-FVP-08: Compliant result does not introduce restore success state**

- Preconditions: Restore page; complete plan returns `compliant`.
- Steps:
  1. Inspect the Final Validation Policy panel and all visible text.
- Expected result: No text containing "Restore complete", "Restore succeeded", "succeeded", or "success" is visible anywhere in the final validation policy area.

---

## Write Phase Ordering Policy (Gate 12)

**TC-WPO-01: Writes-disabled notice always visible**

- Preconditions: Restore page open.
- Steps:
  1. Scroll to the "Gate 12 — Write Phase Ordering Policy" section.
- Expected result: A notice reading "Live restore writes are disabled" is shown. No execute button is present. No token field is present.

**TC-WPO-02: No phase list declared returns blocked (2 checks)**

- Preconditions: Restore page; invoke `verifyWritePhaseOrderingPolicy` with no `phases` field.
- Steps:
  1. Click verify; observe result.
- Expected result: Status is `blocked`. Exactly 2 checks shown (WPO-01, WPO-02). WPO-01 passes. WPO-02 fails with "No write phase list declared" message.

**TC-WPO-03: Canonical phase list returns compliant (10 checks)**

- Preconditions: Restore page; invoke with all 9 phases in canonical order, all `completed` or `planned`.
- Steps:
  1. Click verify; observe result.
- Expected result: Status is `compliant`. 10 checks shown. All check rows show `passed`. Phase summary shows 9 rows. Compliant notice says "writes remain disabled".

**TC-WPO-04: Attachment_metadata_verify skipped with metadata-only reason produces warning**

- Preconditions: Restore page; declare all 9 phases but set `attachmentMetadataVerify` to `skipped` with `skipReason: "metadata-only"`.
- Steps:
  1. Click verify; observe result.
- Expected result: Status is `warning`. WPO-09 shows `warning`. Warning notice is shown. Safety summary shows `writesEnabled: no`.

**TC-WPO-05: Unsafe ordering transition is blocked**

- Preconditions: Restore page; declare phase list with `recordCreate` as `ready` and `schemaVerify` as `planned` (not completed).
- Steps:
  1. Click verify; observe result.
- Expected result: Status is `blocked`. WPO-05 shows `failed` with message about schema_verify prerequisite. Blocked notice is shown.

**TC-WPO-06: No execute button or token input**

- Preconditions: Restore page; Gate 12 panel visible.
- Steps:
  1. Inspect all buttons and input fields in the panel.
- Expected result: Only a "Verify write phase ordering policy" button is present. No "Execute", "Start restore", or "Run" button. No password or token input field.

**TC-WPO-07: Compliant result does not enable writes**

- Preconditions: Restore page; canonical phases return `compliant`.
- Steps:
  1. Observe the safety summary and compliant notice.
- Expected result: "Writes enabled: no" is shown. Compliant notice explicitly states "compliance does not start any write operation".

**TC-WPO-08: Compliant result does not introduce restore success state**

- Preconditions: Restore page; canonical phases return `compliant`.
- Steps:
  1. Inspect the Write Phase Ordering Policy panel and all visible text.
- Expected result: No text containing "Restore complete", "Restore succeeded", "succeeded", or "success" is visible anywhere in the write phase ordering policy area.

---

## Failure Modes Policy (Gate 13)

**TC-FMP-01: Writes-disabled notice always visible**

- Preconditions: Restore page open.
- Steps:
  1. Scroll to the "Gate 13 — Failure Modes Policy" section.
- Expected result: A notice reading "Live restore writes are disabled" is shown. No execute button is present. No token field is present.

**TC-FMP-02: No handling plans declared returns blocked (2 checks)**

- Preconditions: Restore page; invoke `verifyFailureModesPolicy` with no `handlingPlans` field.
- Steps:
  1. Click verify; observe result.
- Expected result: Status is `blocked`. Exactly 2 checks shown (FMP-01, FMP-02). FMP-01 passes. FMP-02 fails with "No failure mode handling plans declared" message.

**TC-FMP-03: Complete safe handling plan returns compliant (11 checks)**

- Preconditions: Restore page; invoke with all 10 required failure modes declared, each with a safe stop behavior and `capturesDiagnosticContext: true`.
- Steps:
  1. Click verify; observe result.
- Expected result: Status is `compliant`. 11 checks shown. All check rows show `passed`. Handling summary table shows 10 rows. Compliant notice says "writes remain disabled".

**TC-FMP-04: Missing required failure mode returns blocked**

- Preconditions: Restore page; declare only 9 of the 10 required failure modes (omit e.g. `recordCreateFailure`).
- Steps:
  1. Click verify; observe result.
- Expected result: Status is `blocked`. FMP-03 shows `failed` with a message naming the missing mode. Blocked notice is shown.

**TC-FMP-05: Destructive rollback declared returns blocked**

- Preconditions: Restore page; declare all 10 modes but set `triggersDestructiveRollback: true` on any mode.
- Steps:
  1. Click verify; observe result.
- Expected result: Status is `blocked`. FMP-05 shows `failed`. Blocked notice is shown.

**TC-FMP-06: Mode without diagnostic context produces warning**

- Preconditions: Restore page; declare all 10 modes with safe stop behaviors, but set `capturesDiagnosticContext: false` on one mode.
- Steps:
  1. Click verify; observe result.
- Expected result: Status is `warning`. A warning check row with ID matching `FMP-W-{modeName}` is shown. Warning notice is shown. Safety summary shows `writesEnabled: no`.

**TC-FMP-07: Partial failure labeled success returns blocked**

- Preconditions: Restore page; declare all 10 modes but set `partialFailureLabeledSuccess: true` on any mode.
- Steps:
  1. Click verify; observe result.
- Expected result: Status is `blocked`. FMP-10 shows `failed`. Blocked notice is shown.

**TC-FMP-08: No execute button or token input**

- Preconditions: Restore page; Gate 13 panel visible.
- Steps:
  1. Inspect all buttons and input fields in the panel.
- Expected result: Only a "Verify failure modes policy" button is present. No "Execute", "Start restore", or "Run" button. No password or token input field.

**TC-FMP-09: Compliant result does not enable writes**

- Preconditions: Restore page; complete safe plan returns `compliant`.
- Steps:
  1. Observe the safety summary and compliant notice.
- Expected result: "Writes enabled: no" is shown. Compliant notice explicitly states "compliance does not start any write operation".

**TC-FMP-10: Compliant result does not introduce restore success state**

- Preconditions: Restore page; complete safe plan returns `compliant`.
- Steps:
  1. Inspect the Failure Modes Policy panel and all visible text.
- Expected result: No text containing "Restore complete", "Restore succeeded", "succeeded", or "success" is visible anywhere in the failure modes policy area.

---

## Restore Rollback Limitation Policy (Gate 14)

**TC-RLP-01: Writes-disabled notice always visible**

- Preconditions: Restore page open.
- Steps:
  1. Scroll to the "Gate 14 — Rollback Limitation Policy" section.
- Expected result: A notice is always visible stating that live restore writes are disabled, automatic rollback is not available, and manual cleanup requires a separate explicit future action. No execute button is shown. No cleanup/delete/revert button is shown. No token input is shown.

**TC-RLP-02: No plan declared returns blocked (2 checks)**

- Preconditions: Restore page open.
- Steps:
  1. Invoke `verifyRollbackLimitationPolicy` with no `plan` field.
- Expected result: Status badge shows `blocked`. Exactly 2 checks are shown. RLP-01 passed. RLP-02 failed with "No rollback limitation plan declared." No plan summary section. `writesEnabled` is `false`.

**TC-RLP-03: Safe plan returns compliant (12 checks)**

- Preconditions: Restore page open.
- Steps:
  1. Invoke `verifyRollbackLimitationPolicy` with a complete safe plan (`rollbackBehavior: noAutomaticRollback`, `partialRestoreIsNotSuccess: true`, `recoveryGuidance: checkpointBasedResume`, `userVisibleLimitationNotice: true`, `noticeIncludesLimitationDetails: true`, `manualCleanupRequiresSeparateAction: true`).
- Expected result: Status badge shows `compliant`. 12 checks shown, all passed. Plan summary shows `noAutomaticRollback`. Message says "writes remain disabled".

**TC-RLP-04: Automatic destructive rollback returns blocked**

- Preconditions: Restore page open.
- Steps:
  1. Invoke `verifyRollbackLimitationPolicy` with `rollbackBehavior: automaticDestructiveRollback`.
- Expected result: Status badge shows `blocked`. RLP-03 shows `failed`. Remediation text visible.

**TC-RLP-05: Automatic delete cleanup returns blocked**

- Preconditions: Restore page open.
- Steps:
  1. Invoke `verifyRollbackLimitationPolicy` with `rollbackBehavior: automaticDeleteCleanup`.
- Expected result: Status badge shows `blocked`. RLP-04 shows `failed`. Remediation text visible.

**TC-RLP-06: Missing recovery guidance returns warning**

- Preconditions: Restore page open.
- Steps:
  1. Invoke `verifyRollbackLimitationPolicy` with `recoveryGuidance: noneDeClared` (all other fields safe).
- Expected result: Status badge shows `warning`. RLP-07 shows `warning`. All blocking checks pass.

**TC-RLP-07: Missing user-visible notice returns warning**

- Preconditions: Restore page open.
- Steps:
  1. Invoke `verifyRollbackLimitationPolicy` with `userVisibleLimitationNotice: false` (all other fields safe).
- Expected result: Status badge shows `warning`. RLP-08 shows `warning`.

**TC-RLP-08: Manual cleanup without separate action returns blocked**

- Preconditions: Restore page open.
- Steps:
  1. Invoke `verifyRollbackLimitationPolicy` with `manualCleanupRequiresSeparateAction: false`.
- Expected result: Status badge shows `blocked`. RLP-09 shows `failed`.

**TC-RLP-09: No execute button or token input**

- Preconditions: Restore page open; `compliant` result shown.
- Steps:
  1. Inspect the Rollback Limitation Policy panel for interactive controls.
- Expected result: No execute, restore, cleanup, delete-all, or revert button is present. No token or password input field is present.

**TC-RLP-10: Compliant result does not enable writes or introduce restore success state**

- Preconditions: Restore page; safe plan returns `compliant`.
- Steps:
  1. Inspect the Rollback Limitation Policy panel and all visible text.
- Expected result: `writesEnabled` is `false`. "Writes disabled" tag is visible. No text containing "Restore complete", "Restore succeeded", "succeeded", or "success" is visible. The message explicitly says "writes remain disabled".

---

## Write Engine Skeleton

**TC-WE-01: Write engine disabled notice is always visible**

- Preconditions: Restore page open; no backup file selected.
- Steps:
  1. Scroll to the "Write Engine" section.
- Expected result: A notice reads "Restore write execution is not enabled in this version." No execute button is shown. No token field is shown.

**TC-WE-02: Write engine preview appears after schema plan**

- Preconditions: A backup file has been inspected, a dry-run plan created, and a schema creation plan generated.
- Steps:
  1. Wait for the schema creation plan to load.
  2. Observe the Write Engine section.
- Expected result: A "No Airtable changes were made." notice appears. Six phase rows appear, all with disabled status. Notes describe what each phase would do if enabled.

**TC-WE-03: No execute button and no token input in write engine panel**

- Preconditions: Restore page open with a schema plan loaded.
- Steps:
  1. Inspect the Write Engine section.
- Expected result: No button with text like "Execute", "Run", or "Start" is visible. No password or token input field is visible.

**TC-WE-04: No success message**

- Preconditions: Any state.
- Steps:
  1. Inspect the Write Engine section and all visible text on the Restore page.
- Expected result: No text containing "Restore complete", "Restore succeeded", "succeeded", or "success" is visible anywhere in the write engine area.

**TC-WE-05: Phase rows cover all six pipeline phases**

- Preconditions: Schema plan is loaded so the preview is visible.
- Steps:
  1. Count the phase rows in the Write Engine section.
- Expected result: Exactly six rows appear, one for each phase: validateInputs, schemaCreation, recordCreation, linkedRecordUpdates, attachmentHandling, finalValidation.

---

## Credential Storage (Settings Page)

**TC-CRED-01: Saved Credentials section is visible in Settings**

- Preconditions: Application open, Settings page navigated to.
- Steps:
  1. Navigate to Settings.
  2. Locate the "Saved Credentials" section.
- Expected result: The section is present. An explanatory notice states that saving is optional and that the token is not stored in files, history, or logs.

**TC-CRED-02: Token input is type password**

- Preconditions: Settings page open; keychain available.
- Steps:
  1. Locate the token input in the Saved Credentials section.
  2. Click into the input and type any value.
- Expected result: The typed characters are masked (dots or asterisks). The raw value is not visible as plain text anywhere on the page.

**TC-CRED-03: Save button is disabled with empty input**

- Preconditions: Settings page open; no token in the input field.
- Steps:
  1. Observe the Save button.
- Expected result: The Save button is disabled.

**TC-CRED-04: Save button becomes enabled after typing a token**

- Preconditions: Settings page open.
- Steps:
  1. Type a non-empty value in the token input.
  2. Observe the Save button.
- Expected result: The Save button becomes enabled.

**TC-CRED-05: Token input is removed from DOM after successful save**

- Preconditions: Settings page open; keychain available.
- Steps:
  1. Type a value in the token input.
  2. Click "Save to Keychain".
  3. Wait for the operation to complete.
- Expected result: The token input field is no longer visible. The status badge updates to "Saved token present". A Remove button appears.

**TC-CRED-06: Saved token value is never rendered**

- Preconditions: A token has been saved.
- Steps:
  1. Inspect all visible text on the Settings page.
  2. Search for the token value in the DOM.
- Expected result: The token value does not appear anywhere in the UI — not in the status badge, feedback message, or any other element.

**TC-CRED-07: Remove button removes the saved token**

- Preconditions: A token has been saved (TC-CRED-05 passed).
- Steps:
  1. Click "Remove Saved Token".
  2. Wait for the operation to complete.
- Expected result: The status badge updates to "No saved token". The token input reappears. The Remove button is hidden.

**TC-CRED-08: Keychain unavailable shows notice**

- Preconditions: The application is running on a system where the OS keychain is not available (e.g., headless Linux without a secret service daemon).
- Steps:
  1. Open Settings → Saved Credentials.
- Expected result: An unavailable notice is shown. The token input and Save button are hidden. No error message contains any token value.

**TC-CRED-09: No token in localStorage or sessionStorage**

- Preconditions: Any state.
- Steps:
  1. Open browser DevTools (if available in the Tauri webview).
  2. Inspect `localStorage` and `sessionStorage` for any key containing "token" or "credential".
- Expected result: No such key is present. No token value appears as a stored value.

**TC-CRED-10: Credential storage does not enable restore write execution**

- Preconditions: A token has been saved to the keychain.
- Steps:
  1. Navigate to the Restore page.
  2. Attempt a full restore gate sequence (package inspected, dry-run ready, confirmation entered).
  3. Click "Attempt Restore".
- Expected result: The result status is "Disabled". The write engine is not enabled. "No Airtable changes were made." is shown. Saving a token has no effect on the restore write gate.

---

## Record Write Engine Foundation

**TC-RWE-01: Record write plan — disabled result for ready import plan**

- Preconditions: A restore package is inspected, dry-run plan is ready, schema plan is ready, record import plan is ready.
- Steps:
  1. Navigate to the Restore page and complete the restore planning flow through the record import plan step.
  2. Observe any "Record Write Plan" section (or invoke via the mock service in a test context).
- Expected result: The result has `status: "disabled"`. No success or execution message is shown. "No Airtable changes were made." is visible.

**TC-RWE-02: Record write plan — blocked for blocked import plan**

- Preconditions: Record import plan status is `"blocked"`.
- Steps:
  1. Invoke `previewRecordWriteRequestPlan` (mock or Tauri) with `recordImportPlanStatus: "blocked"`.
- Expected result: The result has `status: "blocked"` and `blockedReason: "recordImportPlanNotReady"`. `noChangesMade` is `true`.

**TC-RWE-03: Record write plan — blocked for zero tables**

- Preconditions: `tableCount: 0`.
- Steps:
  1. Invoke `previewRecordWriteRequestPlan` with `tableCount: 0`.
- Expected result: The result has `status: "blocked"` and `blockedReason: "noTablesInPlan"`. `noChangesMade` is `true`.

**TC-RWE-04: No token in request or result**

- Preconditions: Any state.
- Steps:
  1. Invoke `previewRecordWriteRequestPlan` with a valid request.
  2. Inspect `JSON.stringify(request)` and `JSON.stringify(result)`.
- Expected result: Neither string contains `"token"`, `"apiKey"`, or any API credential. The request struct has no token field.

**TC-RWE-05: No raw record payloads in result**

- Preconditions: A ready plan with `totalFirstPassBatches > 0`.
- Steps:
  1. Invoke `previewRecordWriteRequestPlan` and stringify the result.
- Expected result: No `"records":` array, no `"payload":` field, and no `"newRecordId"` field in the JSON.

**TC-RWE-06: `noChangesMade` and `networkWritesAttempted` invariants**

- Preconditions: Any state (ready, blocked, or zero-tables).
- Steps:
  1. Invoke `previewRecordWriteRequestPlan` for each case.
- Expected result: `noChangesMade` is `true` and `networkWritesAttempted` is `false` in every result, regardless of status.

**TC-RWE-07: No execute button and no token input**

- Preconditions: Any state.
- Steps:
  1. Observe any UI surface that shows record write plan output.
- Expected result: No "Execute", "Run", "Start", or "Apply" button is rendered. No token input field is rendered.

**TC-RWE-08: Result status is never succeeded**

- Preconditions: Any state.
- Steps:
  1. Invoke `previewRecordWriteRequestPlan` and inspect `result.status`.
  2. Also check `JSON.stringify(result)` for the string `"succeeded"`.
- Expected result: `result.status` is `"disabled"` or `"blocked"`. `"succeeded"` does not appear anywhere in the serialized result.

**TC-RWE-09: Op counts are consistent**

- Preconditions: A ready request with known counts.
- Steps:
  1. Invoke `previewRecordWriteRequestPlan` with `tableCount: 2, totalFirstPassBatches: 4, totalSecondPassBatches: 2, attachmentFieldCount: 1, skippedFieldCount: 2`.
- Expected result: `createBatchOpCount = 4`, `linkedUpdateOpCount = 2`, `checkpointOpCount = 2`, `attachmentOpCount = 1`, `skippedFieldOpCount = 2`, `totalOpCount = 11`.

**TC-RWE-10: IPC fallback is safe**

- Preconditions: Tauri IPC is unavailable (test environment without Tauri runtime).
- Steps:
  1. Call `liveAirBridgeService.previewRecordWriteRequestPlan(request)` in a jsdom context.
- Expected result: Returns `status: "disabled"`, all op counts 0, `noChangesMade: true`, `networkWritesAttempted: false`. No token in the result.

---

## Live Write Safety Contract Non-Regression

**TC-LW-01: Write gate still disabled**

- Preconditions: Release build.
- Steps:
  1. Run `cargo test -- write_safety_contract` in the Rust test suite.
- Expected result: All 20 tests pass. `contract_01_write_gate_always_disabled` passes.

**TC-LW-02: No Succeeded status in serialized output**

- Preconditions: Any state.
- Steps:
  1. Run `cargo test -- contract_02` in the Rust test suite.
- Expected result: All three `contract_02_*` tests pass. No serialized status value contains `"succeeded"`.

**TC-LW-03: noChangesMade invariant across all write types**

- Preconditions: Any state.
- Steps:
  1. Run `cargo test -- contract_03` in the Rust test suite.
- Expected result: All three `contract_03_*` tests pass. Safety report, schema dry-run, and record dry-run all set `noChangesMade: true`.

**TC-LW-04: No token or full path in any write result**

- Preconditions: Any state.
- Steps:
  1. Run `cargo test -- contract_12 contract_15` in the Rust test suite.
- Expected result: All `contract_12_*` and `contract_15_*` tests pass. No serialized result contains `"token"`, `"apiKey"`, `/Users/`, `/home/`, or `/tmp/`.

**TC-LW-05: Attachment phase still produces zero ops with no attachment fields**

- Preconditions: Any state.
- Steps:
  1. Run `cargo test -- contract_16` in the Rust test suite.
- Expected result: Both `contract_16_*` tests pass. `writes_enabled` is `false`. Empty attachment field count produces `attachment_op_count: 0`.

**TC-LW-06: No write endpoint reachable from any restore planning path**

- Preconditions: Source code.
- Steps:
  1. `grep -r "create_records\|update_records\|delete_records\|create_table\|create_field" apps/desktop/src-tauri/src/restore/`
- Expected result: Zero matches. No Airtable write endpoint is called from any file in the restore module.
