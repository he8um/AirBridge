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

## Restore Final Validation Enforcement Policy (Gate 15)

**TC-FVE-01: Writes-disabled notice always visible**

- Preconditions: Restore page open.
- Steps:
  1. Scroll to the "Gate 15 — Final Validation Enforcement Policy" section.
- Expected result: A notice is always visible stating that live restore writes are disabled and no result may be labeled complete or successful without final validation explicitly passing. No execute button is shown. No token input is shown.

**TC-FVE-02: No plan declared returns blocked (2 checks)**

- Preconditions: Restore page open.
- Steps:
  1. Invoke `verifyFinalValidationEnforcementPolicy` with no `plan` field.
- Expected result: Status badge shows `blocked`. Exactly 2 checks are shown. FVE-01 passed. FVE-02 failed with "No final validation enforcement plan declared." No enforcement summary section. `writesEnabled` is `false`.

**TC-FVE-03: Complete safe plan returns compliant (15 checks)**

- Preconditions: Restore page open.
- Steps:
  1. Invoke `verifyFinalValidationEnforcementPolicy` with a complete safe plan (all required states `passed`, full completion guard with all three invariants `true`).
- Expected result: Status badge shows `compliant`. 15 checks shown, all passed. Enforcement summary visible. Message says "writes remain disabled".

**TC-FVE-04: Missing completion guard returns blocked**

- Preconditions: Restore page open.
- Steps:
  1. Invoke `verifyFinalValidationEnforcementPolicy` with a plan that has no `completionGuard`.
- Expected result: Status badge shows `blocked`. FVE-03 shows `failed`. Remediation text visible.

**TC-FVE-05: Incomplete completion guard returns blocked**

- Preconditions: Restore page open.
- Steps:
  1. Invoke `verifyFinalValidationEnforcementPolicy` with a plan where `completionGuard.blocksPartialValidationAsCompletion` is `false`.
- Expected result: Status badge shows `blocked`. FVE-03 shows `failed`.

**TC-FVE-06: Failed schema validation returns blocked**

- Preconditions: Restore page open.
- Steps:
  1. Invoke `verifyFinalValidationEnforcementPolicy` with `schemaValidationState: failed`.
- Expected result: Status badge shows `blocked`. FVE-04 shows `failed`.

**TC-FVE-07: Skipped validation state returns blocked**

- Preconditions: Restore page open.
- Steps:
  1. Invoke `verifyFinalValidationEnforcementPolicy` with any validation state set to `skipped`.
- Expected result: Status badge shows `blocked`. FVE-12 shows `failed`.

**TC-FVE-08: Attachment metadata-only returns warning**

- Preconditions: Restore page open.
- Steps:
  1. Invoke `verifyFinalValidationEnforcementPolicy` with `attachmentValidationMetadataOnly: true`.
- Expected result: Status badge shows `warning`. FVE-08 shows `warning`. No blocking failure.

**TC-FVE-09: NotRequired with reason returns warning**

- Preconditions: Restore page open.
- Steps:
  1. Invoke `verifyFinalValidationEnforcementPolicy` with `schemaValidationState: notRequired` and a `schemaValidationNonRequiredReason` provided.
- Expected result: Status badge shows `warning`. FVE-04 shows `warning`. Message does not say blocked.

**TC-FVE-10: NotRequired without reason returns blocked**

- Preconditions: Restore page open.
- Steps:
  1. Invoke `verifyFinalValidationEnforcementPolicy` with `recordCountValidationState: notRequired` and no reason.
- Expected result: Status badge shows `blocked`. FVE-05 shows `failed`.

**TC-FVE-11: No manifest skips manifest check**

- Preconditions: Restore page open.
- Steps:
  1. Invoke `verifyFinalValidationEnforcementPolicy` with `packageManifestPresent: false`.
- Expected result: FVE-09 shows `passed` with "not applicable" message. No blocking from manifest check.

**TC-FVE-12: No execute button or token input**

- Preconditions: Restore page open; `compliant` result shown.
- Steps:
  1. Inspect the Final Validation Enforcement Policy panel for interactive controls.
- Expected result: No execute or restore button is present. No token or password input field is present.

**TC-FVE-13: Compliant result does not enable writes or introduce restore success state**

- Preconditions: Restore page; complete safe plan returns `compliant`.
- Steps:
  1. Inspect the Final Validation Enforcement Policy panel and all visible text.
- Expected result: `writesEnabled` is `false`. "Writes disabled" tag is visible. No text containing "Restore complete", "Restore succeeded", "succeeded", or "success" is visible. The message explicitly says "writes remain disabled".

---

## Restore Sensitive Data Safety Policy (Gate 16)

**TC-SDS-01: Writes-disabled notice always visible**

- Preconditions: Restore page open.
- Steps:
  1. Scroll to the "Gate 16 — Sensitive Data Safety Policy" section.
- Expected result: A notice is always visible stating that live restore writes are disabled and sensitive material must never be exposed through any restore write surface. No execute button is shown. No token input is shown.

**TC-SDS-02: No plan declared returns blocked (2 checks)**

- Preconditions: Restore page open.
- Steps:
  1. Invoke `verifySensitiveDataSafetyPolicy` with no `plan` field.
- Expected result: Status badge shows `blocked`. Exactly 2 checks are shown. SDS-01 passed. SDS-02 failed with "No sensitive data safety plan declared." No safety summary section. `writesEnabled` is `false`.

**TC-SDS-03: Complete safe plan returns compliant (15 checks)**

- Preconditions: Restore page open.
- Steps:
  1. Invoke `verifySensitiveDataSafetyPolicy` with a complete safe plan (all 10 surfaces covered, all 8 boolean flags `true`, all rules named).
- Expected result: Status badge shows `compliant`. 15 checks shown, all passed. Safety summary visible. Message says "writes remain disabled".

**TC-SDS-04: Missing exposure surface returns blocked**

- Preconditions: Restore page open.
- Steps:
  1. Invoke `verifySensitiveDataSafetyPolicy` with `redactionCoverage` missing one required surface (e.g. `uiPanel`).
- Expected result: Status badge shows `blocked`. SDS-03 shows `failed`. Remediation text visible.

**TC-SDS-05: noTokenInResults false returns blocked**

- Preconditions: Restore page open.
- Steps:
  1. Invoke `verifySensitiveDataSafetyPolicy` with `noTokenInResults: false`.
- Expected result: Status badge shows `blocked`. SDS-04 shows `failed`.

**TC-SDS-06: noFullPathInResults false returns blocked**

- Preconditions: Restore page open.
- Steps:
  1. Invoke `verifySensitiveDataSafetyPolicy` with `noFullPathInResults: false`.
- Expected result: Status badge shows `blocked`. SDS-05 shows `failed`.

**TC-SDS-07: packageReferencesFilenameOnly false returns blocked**

- Preconditions: Restore page open.
- Steps:
  1. Invoke `verifySensitiveDataSafetyPolicy` with `packageReferencesFilenameOnly: false`.
- Expected result: Status badge shows `blocked`. SDS-06 shows `failed`.

**TC-SDS-08: Unnamed redaction rules return warning only**

- Preconditions: Restore page open.
- Steps:
  1. Invoke `verifySensitiveDataSafetyPolicy` with all surfaces covered, all flags `true`, but one or more redaction rules with an empty `redactionRule` string.
- Expected result: Status badge shows `warning`. SDS-12 shows `warning`. No blocking failure. Message does not say "blocked".

**TC-SDS-09: No execute button or token input**

- Preconditions: Restore page open; `compliant` result shown.
- Steps:
  1. Inspect the Sensitive Data Safety Policy panel for interactive controls.
- Expected result: No execute or restore button is present. No token or password input field is present.

**TC-SDS-10: Compliant result does not enable writes or introduce restore success state**

- Preconditions: Restore page; complete safe plan returns `compliant`.
- Steps:
  1. Inspect the Sensitive Data Safety Policy panel and all visible text.
- Expected result: `writesEnabled` is `false`. "Writes disabled" tag is visible. No text containing "Restore complete", "Restore succeeded", "succeeded", or "success" is visible. The message explicitly says "writes remain disabled".

---

## Restore Attachment Phase Disabled Policy (Gate 17)

**TC-APD-01: Writes-disabled notice always visible**

- Preconditions: Restore page open.
- Steps:
  1. Scroll to the "Gate 17 — Attachment Phase Disabled Policy" section.
- Expected result: A writes-disabled notice is visible, mentioning that binary attachment download, upload, fetch, and transfer are not permitted.

**TC-APD-02: Metadata-only notice always visible**

- Preconditions: Restore page open.
- Steps:
  1. Observe the metadata-only notice in the attachment phase disabled policy section.
- Expected result: A notice states that attachment handling is metadata-only. No binary content is downloaded, uploaded, fetched, or transferred.

**TC-APD-03: No plan declared returns blocked (2 checks)**

- Preconditions: Restore page; mock service configured to return no plan.
- Steps:
  1. Click "Verify attachment phase disabled policy".
- Expected result: Status badge shows `blocked`. Exactly 2 checks are shown. APD-01 passed. APD-02 failed with "No attachment metadata plan declared." No phase summary section. `writesEnabled` is `false`.

**TC-APD-04: Complete plan returns compliant (16 checks)**

- Preconditions: Restore page; complete `AttachmentMetadataOnlyPlan` with all flags set correctly.
- Steps:
  1. Click "Verify attachment phase disabled policy".
- Expected result: Status badge shows `compliant`. 16 check rows appear. Phase summary section visible. All 8 boolean flags shown. "Writes disabled" and "Metadata only" tags visible. Message says "writes remain disabled".

**TC-APD-05: binaryHandlingDisabled false returns blocked**

- Preconditions: Plan with `binaryHandlingDisabled: false`.
- Steps:
  1. Click "Verify attachment phase disabled policy".
- Expected result: Status badge shows `blocked`. APD-05 (or later binary check) shows `failed`. Remediation text visible.

**TC-APD-06: Metadata verification disabled without reason returns blocked**

- Preconditions: Plan with `metadataVerificationEnabled: false` and no skip reason.
- Steps:
  1. Click "Verify attachment phase disabled policy".
- Expected result: Status badge shows `blocked`. APD-04 shows `failed`.

**TC-APD-07: Metadata verification disabled with reason returns warning only**

- Preconditions: Plan with `metadataVerificationEnabled: false` and a non-empty skip reason.
- Steps:
  1. Click "Verify attachment phase disabled policy".
- Expected result: Status badge shows `warning`. APD-04 shows `warning`. Message does not say "blocked".

**TC-APD-08: Operation class table shows permitted/blocked correctly**

- Preconditions: Restore page; any result present.
- Steps:
  1. Observe the "Attachment Operation Classes" table.
- Expected result: `metadataInspect` and `metadataVerify` rows show "permitted". All other 8 rows show "blocked".

**TC-APD-09: No execute button or token input**

- Preconditions: Restore page with compliant result.
- Steps:
  1. Inspect all buttons and input fields in the attachment phase disabled policy panel.
- Expected result: No button labeled "Execute", "Start restore", or similar. No `type="password"` or `name="token"` input field. No binary download or upload button.

**TC-APD-10: Compliant result does not enable writes or introduce restore success state**

- Preconditions: Restore page; complete plan returns `compliant`.
- Steps:
  1. Inspect the Attachment Phase Disabled Policy panel and all visible text.
- Expected result: `writesEnabled` is `false`. "Writes disabled" and "Metadata only" tags are visible. No text containing "Restore complete", "Restore succeeded", "succeeded", or "success" is visible. The message explicitly says "writes remain disabled".

---

## Restore Live Write Readiness Policy (Gate 18)

**TC-LWR-01: Writes-disabled notice always visible**

- Preconditions: Restore page open.
- Steps:
  1. Scroll to the "Gate 18 — Live Write Readiness Policy" section.
- Expected result: A writes-disabled notice is visible, stating that verifying this policy does not enable writes or start any restore operation.

**TC-LWR-02: Advisory-only notice always visible**

- Preconditions: Restore page open.
- Steps:
  1. Observe the advisory-only notice in the live write readiness policy section.
- Expected result: A notice states that this is an advisory readiness check only. A Ready result does not enable write execution. Restore completion remains unavailable.

**TC-LWR-03: No gates declared returns blocked (2 checks)**

- Preconditions: Restore page; mock service configured to return no gates.
- Steps:
  1. Click "Verify live write readiness".
- Expected result: Status badge shows `blocked`. Exactly 2 checks are shown. LWR-01 passed. LWR-02 failed with message about no gates declared. No gate summary section. `writesEnabled` is `false`.

**TC-LWR-04: All 17 gates passed returns ready (advisory only)**

- Preconditions: Restore page; all 17 required gates declared with `passed` status.
- Steps:
  1. Click "Verify live write readiness".
- Expected result: Status badge shows "Ready (advisory only)". 10 check rows appear. Gate summary section visible. Total gates = 17. Passed = 17. "Writes disabled" and "Advisory only" tags visible. Message says "writes remain disabled" and "advisory only".

**TC-LWR-05: Failed required gate returns blocked**

- Preconditions: One required gate (e.g. `sandboxEnvironment`) has `failed` status.
- Steps:
  1. Click "Verify live write readiness".
- Expected result: Status badge shows `blocked`. LWR-03 shows `failed`. Remediation text visible.

**TC-LWR-06: Warning gate returns warning (not blocked)**

- Preconditions: One required gate has `warning` status, all others `passed`.
- Steps:
  1. Click "Verify live write readiness".
- Expected result: Status badge shows `warning`. LWR-04 shows `warning`. Message says "writes remain disabled".

**TC-LWR-07: notEvaluated gate returns blocked**

- Preconditions: One required gate has `notEvaluated` status.
- Steps:
  1. Click "Verify live write readiness".
- Expected result: Status badge shows `blocked`. LWR-08 shows `failed`.

**TC-LWR-08: No execute, enable, or token field**

- Preconditions: Restore page with ready result.
- Steps:
  1. Inspect all buttons and input fields in the live write readiness panel.
- Expected result: No button labeled "Execute", "Enable writes", "Start restore", or similar. No `type="password"` or `name="token"` input field.

**TC-LWR-09: Ready result does not enable writes or introduce restore success state**

- Preconditions: Restore page; all 17 gates passed returns `ready`.
- Steps:
  1. Inspect the Live Write Readiness Policy panel and all visible text.
- Expected result: `writesEnabled` is `false`. "Writes disabled" and "Advisory only" tags are visible. No text containing "Restore complete", "Restore succeeded", "succeeded", or "success" is visible. The message explicitly says "writes remain disabled" and "advisory only". The Ready badge text includes "advisory only".

**TC-LWR-10: Gate summary shows correct counts**

- Preconditions: All 17 required gates present with mixed statuses.
- Steps:
  1. Click "Verify live write readiness" with a mix of passed, warning, and not-evaluated gates.
  2. Observe the gate summary section.
- Expected result: Passed, warning, failed, and not-evaluated counts match the declared gate statuses. Total gates = 17. `liveExecutionAvailable` shows `No`.

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

---

## Schema Write Execution Preview Test Cases

**TC-SWEP-01: Blocked when prerequisites missing**

- Preconditions: Application running in mock mode.
- Steps:
  1. Open the Restore page.
  2. Click "Preview schema write execution" without having run any prerequisite gates.
- Expected result: Panel shows `Blocked` badge. "Live schema writes remain disabled" is visible. No execute button is shown.

**TC-SWEP-02: DryRunReady for safe dry-run plan**

- Preconditions: Application running in mock mode.
- Steps:
  1. Open the Restore page.
  2. Ensure all prerequisite gates have been verified (sandbox, target empty, schema plan ready, all policies safe, live write readiness satisfied).
  3. Click "Preview schema write execution".
- Expected result: Panel shows `Dry-run ready` badge. Ordered steps are displayed. "Live schema writes remain disabled" notice is visible. No execute button is shown. `writesEnabled` is `false` in the result.

**TC-SWEP-03: Step ordering — tables before fields**

- Preconditions: `dryRunReady` result with tables and fields.
- Steps:
  1. Obtain a `dryRunReady` preview result with ≥1 table and ≥1 direct field.
  2. Inspect the rendered step list.
- Expected result: All `SWEP-STEP-TBL-*` steps appear before `SWEP-STEP-FLD-DIRECT`. Step indices are sequential starting from 0.

**TC-SWEP-04: Writes remain disabled after preview**

- Preconditions: Application running.
- Steps:
  1. Run `cargo test -- write_gate` in the Rust test suite.
  2. Run `cargo test -- schema_write_execution_preview::tests::write_gate_not_bypassed_by_preview`.
- Expected result: `evaluate_write_gate()` returns `Disabled/DisabledByProductPolicy` before and after the preview call. No live schema write is attempted.

**TC-SWEP-05: No token/path/payload/raw HTTP in result**

- Preconditions: Any state.
- Steps:
  1. Run `cargo test -- schema_write_execution_preview::tests::no_token_in_dry_run_ready_serialization schema_write_execution_preview::tests::no_absolute_path_in_dry_run_ready_serialization schema_write_execution_preview::tests::no_record_payload_in_serialization schema_write_execution_preview::tests::no_attachment_url_in_serialization`.
- Expected result: All four tests pass. No token, path, record payload, or attachment URL appears in the serialized result.

**TC-SWEP-06: Safety invariants in all result states**

- Preconditions: Any state.
- Steps:
  1. Run `cargo test -- schema_write_execution_preview::tests`.
- Expected result: All 38 Rust tests in the `schema_write_execution_preview` module pass. `writesEnabled` is always `false`. `noChangesMade` is always `true`. `networkWritesAttempted` is always `false`.

---

## Record Write Execution Preview Test Cases

**TC-RWEP-01: Blocked when prerequisites missing**

- Preconditions: No prerequisites satisfied.
- Steps:
  1. Call `previewRecordWriteExecution({})` with no fields set.
- Expected result: `status` is `"blocked"`. `mode` is `"liveBlocked"`. `blockedReason` contains `RWEP-PRE-02`. `writesEnabled` is `false`. `noChangesMade` is `true`. `networkWritesAttempted` is `false`. Result has exactly one batch with `status: "blocked"`.

**TC-RWEP-02: Blocked when batch size exceeds 10**

- Preconditions: All other prerequisites satisfied; `batchSize: 11`.
- Steps:
  1. Call `previewRecordWriteExecution({ ...allSafe, batchSize: 11 })`.
- Expected result: `status` is `"blocked"`. `blockedReason` contains `RWEP-PRE-07`. `writesEnabled` is `false`.

**TC-RWEP-03: DryRunReady for safe dry-run plan**

- Preconditions: All 13 prerequisites satisfied.
- Steps:
  1. Call `previewRecordWriteExecution({ ...allSafe, tableCount: 2, totalFirstPassBatches: 4, totalSecondPassBatches: 2, totalRecordCount: 35, batchSize: 10 })`.
- Expected result: `status` is `"dryRunReady"`. `mode` is `"dryRunOnly"`. `firstPassBatchCount` is `4`. `secondPassBatchCount` is `2`. `totalRecordCount` is `35`. `batchSize` is `10`. Message contains "disabled" and "does not start any restore execution".

**TC-RWEP-04: Batch ordering — first-pass before second-pass**

- Preconditions: All prerequisites satisfied.
- Steps:
  1. Call `previewRecordWriteExecution({ ...allSafe, tableCount: 2, totalFirstPassBatches: 4, totalSecondPassBatches: 2 })`.
  2. Inspect `batches` array.
- Expected result: All `first-pass-create` batches appear before any `second-pass-linked-update` batch. `batchIndex` values are sequential from 0.

**TC-RWEP-05: Writes remain disabled after preview**

- Preconditions: All prerequisites satisfied.
- Steps:
  1. Call `preview_record_write_execution_gate` with all safe inputs.
  2. Check `evaluate_write_gate()` return value before and after.
- Expected result: `evaluate_write_gate()` returns `Disabled` before and after. `writesEnabled` in the preview result is `false`.

**TC-RWEP-06: Safety invariants in all result states**

- Preconditions: Any state.
- Steps:
  1. Run `cargo test -- record_write_execution_preview::tests`.
- Expected result: All Rust tests in the `record_write_execution_preview` module pass. `writesEnabled` is always `false`. `noChangesMade` is always `true`. `networkWritesAttempted` is always `false`. No token, absolute path, attachment URL, raw record payload, or `"succeeded"` appears in any serialized result.

---

**TC-MCEP-01: Blocked when prerequisites missing**

- Preconditions: Empty/default request (no prerequisites satisfied).
- Steps:
  1. Invoke `preview_mapping_checkpoint_execution_gate` with an empty request `{}`.
  2. Inspect the result.
- Expected result: `status = "blocked"`. `blockedReason` contains `MCEP-PRE-01`. `writesEnabled = false`. `noChangesMade = true`. `networkWritesAttempted = false`.

**TC-MCEP-02: Blocked when record write preview not ready**

- Preconditions: All prerequisites set to safe except `recordWritePreviewReady = false`.
- Steps:
  1. Invoke `preview_mapping_checkpoint_execution_gate` with `recordWritePreviewReady: false` and all other prerequisites satisfied.
  2. Inspect the result.
- Expected result: `status = "blocked"`. `blockedReason` contains `MCEP-PRE-02`. `writesEnabled = false`.

**TC-MCEP-03: DryRunReady for safe dry-run plan**

- Preconditions: All 8 prerequisites satisfied.
- Steps:
  1. Invoke `preview_mapping_checkpoint_execution_gate` with a fully safe request (all prerequisites `true`, sensible batch counts).
  2. Inspect the result.
- Expected result: `status = "dryRunReady"`. `mode = "dryRunOnly"`. `steps[0].stepId = "MCEP-CHK-SCHEMA"`. Last step is `"MCEP-CHK-PRE-FV"`. `writesEnabled = false`. `noChangesMade = true`. `networkWritesAttempted = false`.

**TC-MCEP-04: Step ordering — mapping before pre-linked-update**

- Preconditions: All prerequisites satisfied; `firstPassBatchCount ≥ 1`.
- Steps:
  1. Invoke `preview_mapping_checkpoint_execution_gate` with `firstPassBatchCount: 3`.
  2. Find the maximum `stepIndex` of all `MCEP-MAP-REC-B{n}` steps.
  3. Find the `stepIndex` of `MCEP-CHK-PRE-LINK`.
- Expected result: All mapping step indices are less than the `MCEP-CHK-PRE-LINK` step index.

**TC-MCEP-05: Writes remain disabled after preview**

- Preconditions: All prerequisites satisfied.
- Steps:
  1. Invoke `preview_mapping_checkpoint_execution_gate` with all safe inputs.
  2. Check `evaluate_write_gate()` return value before and after.
- Expected result: `evaluate_write_gate()` returns `Disabled` before and after. `writesEnabled` in the preview result is `false`.

**TC-MCEP-06: Safety invariants in all result states**

- Preconditions: Any state.
- Steps:
  1. Run `cargo test -- mapping_checkpoint_execution_preview::tests`.
- Expected result: All Rust tests in the `mapping_checkpoint_execution_preview` module pass. `writesEnabled` is always `false`. `noChangesMade` is always `true`. `networkWritesAttempted` is always `false`. No token, absolute path, attachment URL, record ID, field value, or `"succeeded"` appears in any serialized result.


---

**TC-LSEP-01: Blocked when prerequisites missing**

- Preconditions: Empty/default request (no prerequisites satisfied).
- Steps:
  1. Invoke `preview_linked_second_pass_execution_gate` with an empty request `{}`.
  2. Inspect the result.
- Expected result: `status = "blocked"`. `blockedReason` contains `LSEP-PRE-02`. `writesEnabled = false`. `noChangesMade = true`. `networkWritesAttempted = false`.

**TC-LSEP-02: Blocked when mapping/checkpoint preview not ready**

- Preconditions: All prerequisites set to safe except `mappingCheckpointPreviewReady = false`.
- Steps:
  1. Invoke `preview_linked_second_pass_execution_gate` with `mappingCheckpointPreviewReady: false` and all other prerequisites satisfied.
  2. Inspect the result.
- Expected result: `status = "blocked"`. `blockedReason` contains `LSEP-PRE-03`. `writesEnabled = false`.

**TC-LSEP-03: DryRunReady for safe dry-run plan**

- Preconditions: All 8 prerequisites satisfied; sensible field summaries provided.
- Steps:
  1. Invoke `preview_linked_second_pass_execution_gate` with a fully safe request.
  2. Inspect the result.
- Expected result: `status = "dryRunReady"`. `mode = "dryRunOnly"`. `writesEnabled = false`. `noChangesMade = true`. `networkWritesAttempted = false`. Batches have ordered indices.

**TC-LSEP-04: Unresolved links produce warning, not blocked**

- Preconditions: All prerequisites satisfied; field summaries contain `unresolvedLinkCount > 0`.
- Steps:
  1. Invoke `preview_linked_second_pass_execution_gate` with `fieldSummaries[].unresolvedLinkCount > 0`.
  2. Inspect result status and `mappingSummary.unresolvedLinkCount`.
- Expected result: `status = "dryRunReady"`. `mappingSummary.unresolvedLinkCount > 0`. Message contains "unresolved".

**TC-LSEP-05: Writes remain disabled after preview**

- Preconditions: All prerequisites satisfied.
- Steps:
  1. Invoke `preview_linked_second_pass_execution_gate` with all safe inputs.
  2. Check `evaluate_write_gate()` return value before and after.
- Expected result: `evaluate_write_gate()` returns `Disabled` before and after. `writesEnabled` in the preview result is `false`.

**TC-LSEP-06: Safety invariants in all result states**

- Preconditions: Any state.
- Steps:
  1. Run `cargo test -- linked_second_pass_execution_preview::tests`.
- Expected result: All Rust tests in the `linked_second_pass_execution_preview` module pass. `writesEnabled` is always `false`. `noChangesMade` is always `true`. `networkWritesAttempted` is always `false`. No token, absolute path, attachment URL, old/new record ID, field value, or `"succeeded"` appears in any serialized result.

---

**TC-FVEP-01: Blocked when prerequisites missing**

- Preconditions: Empty/default request (no prerequisites satisfied).
- Steps:
  1. Invoke `preview_final_validation_execution_gate` with an empty request `{}`.
  2. Inspect the result.
- Expected result: `status = "blocked"`. `blockedReason` contains `FVEP-PRE-02`. `writesEnabled = false`. `noChangesMade = true`. `networkWritesAttempted = false`.

**TC-FVEP-02: Blocked when linked second-pass preview not ready**

- Preconditions: All prerequisites set to safe except `linkedSecondPassPreviewReady = false`.
- Steps:
  1. Invoke `preview_final_validation_execution_gate` with `linkedSecondPassPreviewReady: false` and all other prerequisites satisfied.
  2. Inspect the result.
- Expected result: `status = "blocked"`. `blockedReason` contains `FVEP-PRE-05`. `writesEnabled = false`.

**TC-FVEP-03: DryRunReady for safe dry-run plan**

- Preconditions: All 10 prerequisites satisfied; sensible count fields provided.
- Steps:
  1. Invoke `preview_final_validation_execution_gate` with a fully safe request.
  2. Inspect the result.
- Expected result: `status = "dryRunReady"`. `mode = "dryRunOnly"`. `writesEnabled = false`. `noChangesMade = true`. `networkWritesAttempted = false`. Result contains exactly 8 checks in order.

**TC-FVEP-04: Manifest check skipped when no manifest**

- Preconditions: All prerequisites satisfied; `manifestPresent = false`.
- Steps:
  1. Invoke `preview_final_validation_execution_gate` with `manifestPresent: false`.
  2. Inspect the checks list.
- Expected result: `status = "dryRunReady"`. Check `FVEP-CHK-MANIFEST` has `status = "skipped"` and `expectedCount = 0`.

**TC-FVEP-05: Writes remain disabled after preview**

- Preconditions: All prerequisites satisfied.
- Steps:
  1. Invoke `preview_final_validation_execution_gate` with all safe inputs.
  2. Check `evaluate_write_gate()` return value before and after.
- Expected result: `evaluate_write_gate()` returns `Disabled` before and after. `writesEnabled` in the preview result is `false`.

**TC-FVEP-06: Safety invariants in all result states**

- Preconditions: Any state.
- Steps:
  1. Run `cargo test -- final_validation_execution_preview::tests`.
- Expected result: All Rust tests in the `final_validation_execution_preview` module pass. `writesEnabled` is always `false`. `noChangesMade` is always `true`. `networkWritesAttempted` is always `false`. No token, absolute path, attachment URL, old/new record ID, field value, or `"succeeded"` appears in any serialized check or summary.

### Checkpoint Metadata Store Test Cases

**TC-RCPS-01: Blocked when prerequisite missing**

- Preconditions: Restore page loaded.
- Steps:
  1. Invoke `store_restore_checkpoint_metadata` with an empty request `{}`.
- Expected result: `status = "blocked"`. `blockedReason` contains `RCPS-PRE-02`. `writesEnabled = false`. `noChangesMade = true`. `networkWritesAttempted = false`. No file written.

**TC-RCPS-02: Blocked when final validation preview not ready**

- Preconditions: All other prerequisites satisfied.
- Steps:
  1. Invoke `store_restore_checkpoint_metadata` with `finalValidationPreviewReady: false` and all other prerequisites satisfied.
- Expected result: `status = "blocked"`. `blockedReason` contains `RCPS-PRE-05`. `writesEnabled = false`. No file written.

**TC-RCPS-03: Stored when all prerequisites satisfied**

- Preconditions: All 5 prerequisites satisfied.
- Steps:
  1. Invoke `store_restore_checkpoint_metadata` with a fully safe request including phases and boundaries.
- Expected result: `status = "stored"`. `summary` present with correct counts. `writesEnabled = false`. `noChangesMade = false`. `networkWritesAttempted = false`. `summary.safeFilename` starts with `rcps-` and ends with `.json`. No path separator in `safeFilename`.

**TC-RCPS-04: Stored checkpoint file is sanitized**

- Preconditions: TC-RCPS-03 completed successfully.
- Steps:
  1. Inspect the stored JSON file in `<os-temp>/airbridge-checkpoints/`.
- Expected result: File declares `"restoreExecutionNotTriggered": true` and `"noSensitiveData": true`. No token, no absolute path, no record IDs, no record field values, no attachment URL, no raw HTTP body.

**TC-RCPS-05: Write gate remains disabled after store**

- Preconditions: TC-RCPS-03 completed.
- Steps:
  1. Invoke `preview_restore_write_engine` after storing checkpoint metadata.
- Expected result: Write engine still returns `status = "Disabled"`. Storing checkpoint metadata does not affect write gate state.

**TC-RCPS-06: Label sanitization for path traversal input**

- Preconditions: Any state.
- Steps:
  1. Invoke `store_restore_checkpoint_metadata` with `checkpointLabel: "../../../etc/passwd"` and all prerequisites satisfied.
- Expected result: `summary.safeFilename` contains no `/`, no `.`, and the string `"passwd"` does not appear. The label is sanitized to alphanumeric, hyphens, and underscores only.

**TC-RCPS-07: Safety invariants in all result states**

- Preconditions: Any state.
- Steps:
  1. Run `cargo test -- checkpoint_store::tests`.
- Expected result: All Rust tests in the `checkpoint_store` module pass. `writesEnabled` is always `false`. `networkWritesAttempted` is always `false`. No token, absolute path, attachment URL, old/new record ID, field value, or `"succeeded"` appears in any serialized result or stored file.

### Schema Write Executor Foundation Test Cases (internal — Rust only)

**TC-SWEX-01: Blocked when mode is disabled**

- Preconditions: Any state (internal Rust test only).
- Steps:
  1. Call `build_schema_write_executor_plan` with `mode: disabled` and a valid request plan.
- Expected result: `status = "blocked"`. `blockedReason` contains `SWEX-PRE-02`. `writesEnabled = false`. `noChangesMade = true`. `networkWritesAttempted = false`.

**TC-SWEX-02: Blocked when explicit internal write flag not set**

- Preconditions: Any state.
- Steps:
  1. Call `build_schema_write_executor_plan` with `mode: sandboxOnly` and `explicitInternalSchemaWriteRequested: false`.
- Expected result: `status = "blocked"`. `blockedReason` contains `SWEX-PRE-03`.

**TC-SWEX-03: Blocked when sandbox not verified**

- Preconditions: Any state.
- Steps:
  1. Call `build_schema_write_executor_plan` with `mode: sandboxOnly`, explicit flag set, but `sandboxVerified: false`.
- Expected result: `status = "blocked"`. `blockedReason` contains `SWEX-PRE-04`.

**TC-SWEX-04: Blocked when target not empty**

- Preconditions: Any state.
- Steps:
  1. Call `build_schema_write_executor_plan` with `targetEmptyVerified: false`.
- Expected result: `status = "blocked"`. `blockedReason` contains `SWEX-PRE-05`.

**TC-SWEX-05: NotExecuted when all prerequisites satisfied**

- Preconditions: Any state.
- Steps:
  1. Call `build_schema_write_executor_plan` with all prerequisites satisfied and a ready request plan.
- Expected result: `status = "notExecuted"`. `mode = "disabled"`. `writesEnabled = false`. `noChangesMade = true`. `networkWritesAttempted = false`. Steps list is non-empty with all steps `pending`.

**TC-SWEX-06: Write gate remains disabled**

- Preconditions: Any state.
- Steps:
  1. Call `build_schema_write_executor_plan` with all prerequisites satisfied.
  2. Call `evaluate_write_gate()`.
- Expected result: Write gate still returns `status = "Disabled"`. The executor call does not affect write gate state.

**TC-SWEX-07: Safety invariants in all result states**

- Preconditions: Any state.
- Steps:
  1. Run `cargo test -- schema_write_executor::tests`.
- Expected result: All Rust tests pass. `writesEnabled` is always `false`. `noChangesMade` is always `true`. `networkWritesAttempted` is always `false`. No token, absolute path, attachment URL, old/new record ID, or `"succeeded"` appears in any serialized result. Steps are ordered tables-first.

### Record Write Executor Foundation Test Cases (internal — Rust only)

**TC-RWEX-01: Blocked when mode is disabled**

- Preconditions: Any state (internal Rust test only).
- Steps:
  1. Call `build_record_write_executor_plan` with `mode: disabled` and a valid request plan.
- Expected result: `status = "blocked"`. `blockedReason` contains `RWEX-PRE-02`. `writesEnabled = false`. `noChangesMade = true`. `networkWritesAttempted = false`.

**TC-RWEX-02: Blocked when explicit internal write flag not set**

- Preconditions: Any state.
- Steps:
  1. Call `build_record_write_executor_plan` with `mode: sandboxOnly` and `explicitInternalRecordWriteRequested: false`.
- Expected result: `status = "blocked"`. `blockedReason` contains `RWEX-PRE-03`.

**TC-RWEX-03: Blocked when sandbox not verified**

- Preconditions: Any state.
- Steps:
  1. Call `build_record_write_executor_plan` with `mode: sandboxOnly`, explicit flag set, but `sandboxVerified: false`.
- Expected result: `status = "blocked"`. `blockedReason` contains `RWEX-PRE-04`.

**TC-RWEX-04: Blocked when target not empty**

- Preconditions: Any state.
- Steps:
  1. Call `build_record_write_executor_plan` with `targetEmptyVerified: false`.
- Expected result: `status = "blocked"`. `blockedReason` contains `RWEX-PRE-05`.

**TC-RWEX-05: Blocked when schema executor not safe**

- Preconditions: Any state.
- Steps:
  1. Call `build_record_write_executor_plan` with `schemaExecutorSafe: false`.
- Expected result: `status = "blocked"`. `blockedReason` contains `RWEX-PRE-06`.

**TC-RWEX-06: Blocked when rate-limit policy not safe**

- Preconditions: Any state.
- Steps:
  1. Call `build_record_write_executor_plan` with `rateLimitBackoffSafe: false`.
- Expected result: `status = "blocked"`. `blockedReason` contains `RWEX-PRE-07`.

**TC-RWEX-07: NotExecuted when all prerequisites satisfied**

- Preconditions: Any state.
- Steps:
  1. Call `build_record_write_executor_plan` with all prerequisites satisfied and a ready request plan.
- Expected result: `status = "notExecuted"`. `mode = "disabled"`. `writesEnabled = false`. `noChangesMade = true`. `networkWritesAttempted = false`. Batches list is non-empty. First-pass create batches precede second-pass linked-update batches.

**TC-RWEX-08: Write gate remains disabled**

- Preconditions: Any state.
- Steps:
  1. Call `build_record_write_executor_plan` with all prerequisites satisfied.
  2. Call `evaluate_write_gate()`.
- Expected result: Write gate still returns `status = "Disabled"`. The executor call does not affect write gate state.

**TC-RWEX-09: Safety invariants in all result states**

- Preconditions: Any state.
- Steps:
  1. Run `cargo test -- record_write_executor::foundation_tests`.
- Expected result: All Rust tests pass. `writesEnabled` is always `false`. `noChangesMade` is always `true`. `networkWritesAttempted` is always `false`. No token, absolute path, record payload, attachment URL, old/new record ID, or `"succeeded"` appears in any serialized result. Batches are ordered first-pass before second-pass. Batch indices are sequential.

### Linked Second-Pass Executor Foundation Test Cases (internal — Rust only)

**TC-LSEX-01: Blocked when mode is disabled**

- Preconditions: Any state (internal Rust test only).
- Steps:
  1. Call `build_linked_second_pass_executor_plan` with `mode: disabled` and valid prerequisites.
- Expected result: `status = "blocked"`. `blockedReason` contains `LSEX-PRE-02`. `writesEnabled = false`. `noChangesMade = true`. `networkWritesAttempted = false`.

**TC-LSEX-02: Blocked when explicit internal flag not set**

- Preconditions: Any state.
- Steps:
  1. Call `build_linked_second_pass_executor_plan` with `mode: sandboxOnly` and `explicitInternalLinkedSecondPassRequested: false`.
- Expected result: `status = "blocked"`. `blockedReason` contains `LSEX-PRE-03`.

**TC-LSEX-03: Blocked when record executor not safe**

- Preconditions: Any state.
- Steps:
  1. Call `build_linked_second_pass_executor_plan` with `recordExecutorSafe: false`.
- Expected result: `status = "blocked"`. `blockedReason` contains `LSEX-PRE-06`.

**TC-LSEX-04: Blocked when linked second-pass preview blocked**

- Preconditions: Any state.
- Steps:
  1. Call `build_linked_second_pass_executor_plan` with `linkedSecondPassPreviewStatus: blocked`.
- Expected result: `status = "blocked"`. `blockedReason` contains `LSEX-PRE-07`.

**TC-LSEX-05: Blocked when batch size exceeds maximum**

- Preconditions: Any state.
- Steps:
  1. Call `build_linked_second_pass_executor_plan` with `batchSize: 11`.
- Expected result: `status = "blocked"`. `blockedReason` contains `LSEX-BATCH-SIZE`.

**TC-LSEX-06: Unresolved optional links are warning-safe**

- Preconditions: Any state.
- Steps:
  1. Call `build_linked_second_pass_executor_plan` with all prerequisites satisfied and field_summaries containing `unresolved_link_count > 0`.
- Expected result: `status = "notExecuted"` (not blocked). Unresolved links do not block the executor when the preview returned DryRunReady.

**TC-LSEX-07: NotExecuted when all prerequisites satisfied**

- Preconditions: Any state.
- Steps:
  1. Call `build_linked_second_pass_executor_plan` with all prerequisites satisfied and valid field summaries.
- Expected result: `status = "notExecuted"`. `mode = "disabled"`. `writesEnabled = false`. `noChangesMade = true`. `networkWritesAttempted = false`. Batches list is non-empty. Field ordering is preserved. Batch indices are sequential.

**TC-LSEX-08: Write gate remains disabled**

- Preconditions: Any state.
- Steps:
  1. Call `build_linked_second_pass_executor_plan` with all prerequisites satisfied.
  2. Call `evaluate_write_gate()`.
- Expected result: Write gate still returns `status = "Disabled"`. The executor call does not affect write gate state.

**TC-LSEX-09: Safety invariants in all result states**

- Preconditions: Any state.
- Steps:
  1. Run `cargo test -- linked_second_pass_executor::tests`.
- Expected result: All Rust tests pass. `writesEnabled` is always `false`. `noChangesMade` is always `true`. `networkWritesAttempted` is always `false`. No token, absolute path, record payload, attachment URL, old/new record ID, or `"succeeded"` appears in any serialized result. Batch indices are sequential. Field ordering from field_summaries is preserved.

---

## Final Validation Reader Foundation (internal module — TC-FVRD-*)

**TC-FVRD-01: Blocked when mode is disabled**

- Preconditions: Internal module available.
- Steps:
  1. Call `build_final_validation_reader_plan` with `mode: Disabled` and all other prerequisites satisfied.
- Expected result: Result status is `Blocked`. `blocked_reason` contains `FVRD-PRE-02`. `readsEnabled` is `false`. `writesEnabled` is `false`. `noChangesMade` is `true`. `networkReadsAttempted` is `false`. `networkWritesAttempted` is `false`.

**TC-FVRD-02: Blocked when explicit internal flag is not set**

- Preconditions: Internal module available.
- Steps:
  1. Call `build_final_validation_reader_plan` with `explicit_internal_final_validation_read_requested: false` and all other prerequisites satisfied.
- Expected result: Result status is `Blocked`. `blocked_reason` contains `FVRD-PRE-03`. No Airtable reads are attempted.

**TC-FVRD-03: Blocked when sandbox not verified**

- Preconditions: Internal module available.
- Steps:
  1. Call `build_final_validation_reader_plan` with `sandbox_verified: false` and all other prerequisites satisfied.
- Expected result: Result status is `Blocked`. `blocked_reason` contains `FVRD-PRE-04`. No Airtable reads are attempted.

**TC-FVRD-04: Blocked when schema executor not safe**

- Preconditions: Internal module available.
- Steps:
  1. Call `build_final_validation_reader_plan` with `schema_executor_safe: false` and all other prerequisites satisfied.
- Expected result: Result status is `Blocked`. `blocked_reason` contains `FVRD-PRE-05`. No Airtable reads are attempted.

**TC-FVRD-05: Blocked when record executor not safe**

- Preconditions: Internal module available.
- Steps:
  1. Call `build_final_validation_reader_plan` with `record_executor_safe: false` and all other prerequisites satisfied.
- Expected result: Result status is `Blocked`. `blocked_reason` contains `FVRD-PRE-06`. No Airtable reads are attempted.

**TC-FVRD-06: Blocked when linked executor not safe**

- Preconditions: Internal module available.
- Steps:
  1. Call `build_final_validation_reader_plan` with `linked_executor_safe: false` and all other prerequisites satisfied.
- Expected result: Result status is `Blocked`. `blocked_reason` contains `FVRD-PRE-07`. No Airtable reads are attempted.

**TC-FVRD-07: Blocked when final validation preview not ready**

- Preconditions: Internal module available.
- Steps:
  1. Call `build_final_validation_reader_plan` with `final_validation_preview_ready: false` and all other prerequisites satisfied.
- Expected result: Result status is `Blocked`. `blocked_reason` contains `FVRD-PRE-08`. No Airtable reads are attempted.

**TC-FVRD-08: Blocked when enforcement policy not safe**

- Preconditions: Internal module available.
- Steps:
  1. Call `build_final_validation_reader_plan` with `final_validation_enforcement_safe: false` and all other prerequisites satisfied.
- Expected result: Result status is `Blocked`. `blocked_reason` contains `FVRD-PRE-09`. No Airtable reads are attempted.

**TC-FVRD-09: Blocked when sensitive data not safe**

- Preconditions: Internal module available.
- Steps:
  1. Call `build_final_validation_reader_plan` with `sensitive_data_safe: false` and all other prerequisites satisfied.
- Expected result: Result status is `Blocked`. `blocked_reason` contains `FVRD-PRE-10`. No Airtable reads are attempted.

**TC-FVRD-10: Blocked when attachment phase not safe**

- Preconditions: Internal module available.
- Steps:
  1. Call `build_final_validation_reader_plan` with `attachment_phase_disabled_safe: false` and all other prerequisites satisfied.
- Expected result: Result status is `Blocked`. `blocked_reason` contains `FVRD-PRE-11`. No Airtable reads are attempted.

**TC-FVRD-11: NotExecuted when all prerequisites satisfied**

- Preconditions: Internal module available.
- Steps:
  1. Call `build_final_validation_reader_plan` with all prerequisites satisfied.
- Expected result: Result status is `NotExecuted`. `blocked_reason` is `None`. `readsEnabled` is `false`. `writesEnabled` is `false`. `noChangesMade` is `true`. `networkReadsAttempted` is `false`. `networkWritesAttempted` is `false`. `safety_snapshot.read_gate_disabled` is `true`.

**TC-FVRD-12: Manifest check skipped when not present**

- Preconditions: Internal module available; all prerequisites satisfied.
- Steps:
  1. Call `build_final_validation_reader_plan` with `manifest_present: false`.
- Expected result: `FVRD-CHK-MANIFEST` check has status `Skipped`. All other checks have status `Pending`.

**TC-FVRD-13: Check order is deterministic**

- Preconditions: Internal module available; all prerequisites satisfied.
- Steps:
  1. Call `build_final_validation_reader_plan` twice with identical inputs.
- Expected result: Both results have identical check ID ordering. First check is `FVRD-CHK-SCHEMA`. Last check is `FVRD-CHK-GUARD`.

**TC-FVRD-14: Safety invariants in all result states**

- Preconditions: Any state.
- Steps:
  1. Run `cargo test -- final_validation_reader::tests`.
- Expected result: All Rust tests pass. `readsEnabled` is always `false`. `writesEnabled` is always `false`. `noChangesMade` is always `true`. `networkReadsAttempted` is always `false`. `networkWritesAttempted` is always `false`. No token, absolute path, record payload, attachment URL, old/new record ID, or `"succeeded"` appears in any serialized result. Check ordering is deterministic. Attach check note does not mention binary retrieval.

---

## Restore Orchestrator Foundation (internal module — TC-ORCH-*)

**TC-ORCH-01: Blocked when mode is disabled**

- Preconditions: Internal module available.
- Steps:
  1. Call `build_restore_orchestrator_plan` with `mode: Disabled` and all other prerequisites satisfied.
- Expected result: Result status is `Blocked`. `blocked_reason` contains `ORCH-PRE-02`. `writesEnabled` is `false`. `readsEnabled` is `false`. `noChangesMade` is `true`. `networkReadsAttempted` is `false`. `networkWritesAttempted` is `false`.

**TC-ORCH-02: Blocked when sandbox not verified**

- Preconditions: Internal module available.
- Steps:
  1. Call `build_restore_orchestrator_plan` with `sandbox_verified: false` and all other prerequisites satisfied.
- Expected result: Result status is `Blocked`. `blocked_reason` contains `ORCH-PRE-03`. No Airtable calls attempted.

**TC-ORCH-03: Blocked when target not empty**

- Preconditions: Internal module available.
- Steps:
  1. Call `build_restore_orchestrator_plan` with `target_empty_verified: false` and all other prerequisites satisfied.
- Expected result: Result status is `Blocked`. `blocked_reason` contains `ORCH-PRE-04`.

**TC-ORCH-04: Blocked when write phase ordering unsafe**

- Preconditions: Internal module available.
- Steps:
  1. Call `build_restore_orchestrator_plan` with `write_phase_ordering_safe: false` and all other prerequisites satisfied.
- Expected result: Result status is `Blocked`. `blocked_reason` contains `ORCH-PRE-05`.

**TC-ORCH-05: Blocked when failure modes unsafe**

- Preconditions: Internal module available.
- Steps:
  1. Call `build_restore_orchestrator_plan` with `failure_modes_safe: false` and all other prerequisites satisfied.
- Expected result: Result status is `Blocked`. `blocked_reason` contains `ORCH-PRE-06`.

**TC-ORCH-06: Blocked when rollback limitation unsafe**

- Preconditions: Internal module available.
- Steps:
  1. Call `build_restore_orchestrator_plan` with `rollback_limitation_safe: false` and all other prerequisites satisfied.
- Expected result: Result status is `Blocked`. `blocked_reason` contains `ORCH-PRE-07`.

**TC-ORCH-07: Blocked when live write readiness not safe**

- Preconditions: Internal module available.
- Steps:
  1. Call `build_restore_orchestrator_plan` with `live_write_readiness_safe: false` and all other prerequisites satisfied.
- Expected result: Result status is `Blocked`. `blocked_reason` contains `ORCH-PRE-08`.

**TC-ORCH-08: Blocked when schema executor not safe**

- Preconditions: Internal module available.
- Steps:
  1. Call `build_restore_orchestrator_plan` with `schema_executor_safe: false` and all other prerequisites satisfied.
- Expected result: Result status is `Blocked`. `blocked_reason` contains `ORCH-PRE-09`.

**TC-ORCH-09: Blocked when record executor not safe**

- Preconditions: Internal module available.
- Steps:
  1. Call `build_restore_orchestrator_plan` with `record_executor_safe: false` and all other prerequisites satisfied.
- Expected result: Result status is `Blocked`. `blocked_reason` contains `ORCH-PRE-10`.

**TC-ORCH-10: Blocked when linked executor not safe**

- Preconditions: Internal module available.
- Steps:
  1. Call `build_restore_orchestrator_plan` with `linked_executor_safe: false` and all other prerequisites satisfied.
- Expected result: Result status is `Blocked`. `blocked_reason` contains `ORCH-PRE-11`.

**TC-ORCH-11: Blocked when final validation reader not safe**

- Preconditions: Internal module available.
- Steps:
  1. Call `build_restore_orchestrator_plan` with `final_validation_reader_safe: false` and all other prerequisites satisfied.
- Expected result: Result status is `Blocked`. `blocked_reason` contains `ORCH-PRE-12`.

**TC-ORCH-12: NotExecuted when all prerequisites satisfied**

- Preconditions: Internal module available.
- Steps:
  1. Call `build_restore_orchestrator_plan` with all prerequisites satisfied.
- Expected result: Result status is `NotExecuted`. `blocked_reason` is `None`. `writesEnabled` is `false`. `readsEnabled` is `false`. `noChangesMade` is `true`. `networkReadsAttempted` is `false`. `networkWritesAttempted` is `false`. `safety_snapshot.write_gate_disabled` is `true`. `total_phase_count` is 8.

**TC-ORCH-13: Deterministic phase ordering**

- Preconditions: Internal module available; all prerequisites satisfied.
- Steps:
  1. Call `build_restore_orchestrator_plan` twice with identical inputs.
- Expected result: Both results have identical phase ID ordering. First phase is `ORCH-PH-01` (schema executor). Last phase is `ORCH-PH-08` (final guard). Checkpoint boundaries follow their respective executors.

**TC-ORCH-14: Safety invariants in all result states**

- Preconditions: Any state.
- Steps:
  1. Run `cargo test -- restore_orchestrator::tests`.
- Expected result: All Rust tests pass. `writesEnabled` is always `false`. `readsEnabled` is always `false`. `noChangesMade` is always `true`. `networkReadsAttempted` is always `false`. `networkWritesAttempted` is always `false`. No token, absolute path, record payload, attachment URL, old/new record ID, `"succeeded"`, `"completed"`, or `"restoreSuccess"` appears in any serialized result. Phase ordering is deterministic. No production mode exists.

---

## Sandbox Gate Contract Foundation Tests

**TC-SGC-01: Disabled mode returns Disabled immediately**

- Preconditions: Internal module available.
- Steps:
  1. Call `evaluate_sandbox_gate_contract` with `mode: disabled` and all other fields `false`.
- Expected result: Result status is `disabled`. `prerequisites` is empty. `writesEnabled` is `false`. `readsEnabled` is `false`. `noChangesMade` is `true`. `networkReadsAttempted` is `false`. `networkWritesAttempted` is `false`. `blocked_reason` is absent.

**TC-SGC-02: Blocked when sandbox not safe**

- Preconditions: Internal module available.
- Steps:
  1. Call `evaluate_sandbox_gate_contract` with `mode: sandboxOnlyCandidate`, `sandbox_verification_safe: false`, all other prerequisite fields `true`.
- Expected result: Result status is `blocked`. `blocked_reason` contains `SGC-PRE-01`.

**TC-SGC-03: Blocked when target not empty**

- Preconditions: Internal module available.
- Steps:
  1. Call `evaluate_sandbox_gate_contract` with `mode: sandboxOnlyCandidate`, `target_empty_safe: false`, all other prerequisite fields `true`.
- Expected result: Result status is `blocked`. `blocked_reason` contains `SGC-PRE-02`.

**TC-SGC-04: Blocked when confirmation gate not declared**

- Preconditions: Internal module available.
- Steps:
  1. Call `evaluate_sandbox_gate_contract` with `mode: sandboxOnlyCandidate`, `confirmation_gate_declared: false`, all other prerequisite fields `true`.
- Expected result: Result status is `blocked`. `blocked_reason` contains `SGC-PRE-03`.

**TC-SGC-05: Blocked when destructive operation policy unsafe**

- Preconditions: Internal module available.
- Steps:
  1. Call `evaluate_sandbox_gate_contract` with `mode: sandboxOnlyCandidate`, `destructive_operation_policy_safe: false`, all other prerequisite fields `true`.
- Expected result: Result status is `blocked`. `blocked_reason` contains `SGC-PRE-04`.

**TC-SGC-06: Blocked when attachment phase unsafe**

- Preconditions: Internal module available.
- Steps:
  1. Call `evaluate_sandbox_gate_contract` with `mode: sandboxOnlyCandidate`, `attachment_phase_disabled_safe: false`, all other prerequisite fields `true`.
- Expected result: Result status is `blocked`. `blocked_reason` contains `SGC-PRE-05`.

**TC-SGC-07: Blocked when live write readiness not safe**

- Preconditions: Internal module available.
- Steps:
  1. Call `evaluate_sandbox_gate_contract` with `mode: sandboxOnlyCandidate`, `live_write_readiness_safe: false`, all other prerequisite fields `true`.
- Expected result: Result status is `blocked`. `blocked_reason` contains `SGC-PRE-06`.

**TC-SGC-08: Blocked when restore orchestrator not present**

- Preconditions: Internal module available.
- Steps:
  1. Call `evaluate_sandbox_gate_contract` with `mode: sandboxOnlyCandidate`, `restore_orchestrator_present: false`, all other prerequisite fields `true`.
- Expected result: Result status is `blocked`. `blocked_reason` contains `SGC-PRE-07`.

**TC-SGC-09: Blocked when schema executor not present**

- Preconditions: Internal module available.
- Steps:
  1. Call `evaluate_sandbox_gate_contract` with `mode: sandboxOnlyCandidate`, `schema_executor_present: false`, all other prerequisite fields `true`.
- Expected result: Result status is `blocked`. `blocked_reason` contains `SGC-PRE-08`.

**TC-SGC-10: Blocked when record executor not present**

- Preconditions: Internal module available.
- Steps:
  1. Call `evaluate_sandbox_gate_contract` with `mode: sandboxOnlyCandidate`, `record_executor_present: false`, all other prerequisite fields `true`.
- Expected result: Result status is `blocked`. `blocked_reason` contains `SGC-PRE-09`.

**TC-SGC-11: Blocked when linked executor not present**

- Preconditions: Internal module available.
- Steps:
  1. Call `evaluate_sandbox_gate_contract` with `mode: sandboxOnlyCandidate`, `linked_executor_present: false`, all other prerequisite fields `true`.
- Expected result: Result status is `blocked`. `blocked_reason` contains `SGC-PRE-10`.

**TC-SGC-12: Blocked when final validation reader not present**

- Preconditions: Internal module available.
- Steps:
  1. Call `evaluate_sandbox_gate_contract` with `mode: sandboxOnlyCandidate`, `final_validation_reader_present: false`, all other prerequisite fields `true`.
- Expected result: Result status is `blocked`. `blocked_reason` contains `SGC-PRE-11`.

**TC-SGC-13: EligibleButNotArmed when all prerequisites satisfied**

- Preconditions: Internal module available.
- Steps:
  1. Call `evaluate_sandbox_gate_contract` with `mode: sandboxOnlyCandidate` and all prerequisite fields `true`.
- Expected result: Result status is `eligibleButNotArmed`. `blocked_reason` is absent. `writesEnabled` is `false`. `readsEnabled` is `false`. `noChangesMade` is `true`. `networkReadsAttempted` is `false`. `networkWritesAttempted` is `false`. `safety_snapshot.write_gate_disabled` is `true`. `total_prereq_count` is 12. Message contains "NOT armed" and "NOT enabled".

**TC-SGC-14: Safety invariants in all result states**

- Preconditions: Any state.
- Steps:
  1. Run `cargo test -- sandbox_gate_contract::tests`.
- Expected result: All Rust tests pass. `writesEnabled` is always `false`. `readsEnabled` is always `false`. `noChangesMade` is always `true`. `networkReadsAttempted` is always `false`. `networkWritesAttempted` is always `false`. No token, absolute path, record payload, attachment URL, old/new record ID, `"armed"`, `"enabled"`, `"succeeded"`, or `"restoreSuccess"` appears in any serialized result. Prerequisite ordering is deterministic (SGC-PRE-01 first, SGC-PRE-12 last). No production mode exists. `evaluate_write_gate()` is never modified.

---

## Sandbox Restore Harness Foundation Tests

**TC-SRH-01: Disabled mode returns NotExecuted immediately**

- Preconditions: Internal module available.
- Steps:
  1. Call `build_sandbox_restore_harness_plan` with `mode: disabled` and all other fields `false`.
- Expected result: Result status is `notExecuted`. `gate_armed` is `false`. `writesEnabled` is `false`. `readsEnabled` is `false`. `noChangesMade` is `true`. `networkReadsAttempted` is `false`. `networkWritesAttempted` is `false`. `blocked_reason` is absent.

**TC-SRH-02: Blocked when sandbox not safe**

- Preconditions: Internal module available.
- Steps:
  1. Call `build_sandbox_restore_harness_plan` with `mode: sandboxOnlyDryHarness`, `sandbox_verification_safe: false`, all other prerequisite fields `true`.
- Expected result: Result status is `blocked`. `gate_armed` is `false`.

**TC-SRH-03: Blocked when target not empty**

- Preconditions: Internal module available.
- Steps:
  1. Call `build_sandbox_restore_harness_plan` with `mode: sandboxOnlyDryHarness`, `target_empty_safe: false`, all other prerequisite fields `true`.
- Expected result: Result status is `blocked`.

**TC-SRH-04: Blocked when write phase ordering unsafe**

- Preconditions: Internal module available.
- Steps:
  1. Call `build_sandbox_restore_harness_plan` with `mode: sandboxOnlyDryHarness`, `write_phase_ordering_safe: false`, all other prerequisite fields `true`.
- Expected result: Result status is `blocked`. `blocked_reason` contains "Orchestrator".

**TC-SRH-05: Blocked when failure modes unsafe**

- Preconditions: Internal module available.
- Steps:
  1. Call `build_sandbox_restore_harness_plan` with `mode: sandboxOnlyDryHarness`, `failure_modes_safe: false`, all other prerequisite fields `true`.
- Expected result: Result status is `blocked`.

**TC-SRH-06: Blocked when rollback limitation unsafe**

- Preconditions: Internal module available.
- Steps:
  1. Call `build_sandbox_restore_harness_plan` with `mode: sandboxOnlyDryHarness`, `rollback_limitation_safe: false`, all other prerequisite fields `true`.
- Expected result: Result status is `blocked`.

**TC-SRH-07: ReadyNotExecuted when all prerequisites satisfied**

- Preconditions: Internal module available.
- Steps:
  1. Call `build_sandbox_restore_harness_plan` with `mode: sandboxOnlyDryHarness` and all prerequisite fields `true`.
- Expected result: Result status is `readyNotExecuted`. `gate_armed` is `false`. `writesEnabled` is `false`. `readsEnabled` is `false`. `noChangesMade` is `true`. `networkReadsAttempted` is `false`. `networkWritesAttempted` is `false`. `safety_snapshot.write_gate_disabled` is `true`. `safety_snapshot.gate_armed` is `false`. `total_phase_count` is 8. Message contains "NOT armed", "NOT enabled", and "remains pending".

**TC-SRH-08: Deterministic phase ordering**

- Preconditions: Internal module available; all prerequisites satisfied.
- Steps:
  1. Call `build_sandbox_restore_harness_plan` twice with identical inputs.
- Expected result: Both results have identical phase ID ordering. First phase is `SRH-PH-01` (gate contract evaluation). Last phase is `SRH-PH-08` (final guard). All phase IDs use the `SRH-PH-` prefix.

**TC-SRH-09: Safety invariants in all result states**

- Preconditions: Any state.
- Steps:
  1. Run `cargo test -- sandbox_restore_harness::tests`.
- Expected result: All Rust tests pass. `gate_armed` is always `false`. `writesEnabled` is always `false`. `readsEnabled` is always `false`. `noChangesMade` is always `true`. `networkReadsAttempted` is always `false`. `networkWritesAttempted` is always `false`. No token, absolute path, record payload, attachment URL, old/new record ID, `"armed"`, `"enabled"`, `"succeeded"`, or `"restoreSuccess"` appears in any serialized result. Phase ordering is deterministic. No production mode exists. `evaluate_write_gate()` is never modified. Live sandbox E2E restore execution remains pending.

## Sandbox Enablement Readiness Report Tests

**TC-SERN-01: ReadyButDisabled when all prerequisites satisfied**

- Preconditions: Internal module available.
- Steps:
  1. Call `build_sandbox_enablement_readiness_report` with all boolean fields `true`.
- Expected result: Status is `readyButDisabled`. `gate_armed` is `false`. `writesEnabled` is `false`. `readsEnabled` is `false`. `noChangesMade` is `true`. `networkReadsAttempted` is `false`. `networkWritesAttempted` is `false`. `blocked_reason` is absent. `total_item_count` is 13. `ready_item_count` is 13. Message contains "NOT armed", "NOT enabled", and "remains separate pending work".

**TC-SERN-02: NotReady when sandbox verification not safe**

- Preconditions: Internal module available.
- Steps:
  1. Call `build_sandbox_enablement_readiness_report` with all fields `true` except `sandbox_verification_safe: false`.
- Expected result: Status is `notReady`. `blocked_reason` is set. `gate_armed` is `false`.

**TC-SERN-03: NotReady when target not empty**

- Preconditions: Internal module available.
- Steps:
  1. Call `build_sandbox_enablement_readiness_report` with all fields `true` except `target_empty_safe: false`.
- Expected result: Status is `notReady`.

**TC-SERN-04: NotReady when confirmation gate not declared**

- Preconditions: Internal module available.
- Steps:
  1. Call `build_sandbox_enablement_readiness_report` with all fields `true` except `confirmation_gate_declared: false`.
- Expected result: Status is `notReady`.

**TC-SERN-05: NotReady when write phase ordering unsafe**

- Preconditions: Internal module available.
- Steps:
  1. Call `build_sandbox_enablement_readiness_report` with all fields `true` except `write_phase_ordering_safe: false`.
- Expected result: Status is `notReady`.

**TC-SERN-06: NotReady when failure modes unsafe**

- Preconditions: Internal module available.
- Steps:
  1. Call `build_sandbox_enablement_readiness_report` with all fields `true` except `failure_modes_safe: false`.
- Expected result: Status is `notReady`.

**TC-SERN-07: NotReady when rollback limitation unsafe**

- Preconditions: Internal module available.
- Steps:
  1. Call `build_sandbox_enablement_readiness_report` with all fields `true` except `rollback_limitation_safe: false`.
- Expected result: Status is `notReady`.

**TC-SERN-08: Deterministic item ordering**

- Preconditions: Internal module available; all prerequisites satisfied.
- Steps:
  1. Call `build_sandbox_enablement_readiness_report` twice with identical inputs.
- Expected result: Both results have identical item ID ordering. First item is `SERN-01` (write gate default). Last item is `SERN-13` (no sensitive data exposure). All item IDs use the `SERN-` prefix.

**TC-SERN-09: Safety invariants in all result states**

- Preconditions: Any state.
- Steps:
  1. Run `cargo test -- sandbox_enablement_readiness::tests`.
- Expected result: All Rust tests pass. `gate_armed` is always `false`. `writesEnabled` is always `false`. `readsEnabled` is always `false`. `noChangesMade` is always `true`. `networkReadsAttempted` is always `false`. `networkWritesAttempted` is always `false`. No token, absolute path, record payload, attachment URL, old/new record ID, `"armed"`, `"enabled"`, `"succeeded"`, or `"restoreSuccess"` appears in any serialized result. Item ordering is deterministic. `evaluate_write_gate()` is never modified. Future sandbox-only gate enablement remains separate pending work.

## Sandbox Gate Arming Model Tests

**TC-SGA-01: Blocked when mode is disabled**

- Preconditions: Internal module available.
- Steps:
  1. Call `build_sandbox_gate_arming_decision` with `mode: disabled` and all other fields true.
- Expected result: Status is `blocked`. `gate_armed` is `false`. `executionEnabled` is `false`. `writesEnabled` is `false`. `readsEnabled` is `false`. `noChangesMade` is `true`. `networkReadsAttempted` is `false`. `networkWritesAttempted` is `false`. `blocked_reason` is set.

**TC-SGA-02: Blocked when explicit arming flag missing**

- Preconditions: Internal module available.
- Steps:
  1. Call `build_sandbox_gate_arming_decision` with `mode: sandboxOnlyInternal`, `explicit_internal_sandbox_arming_requested: false`, all other fields true.
- Expected result: Status is `blocked`. `blocked_reason` contains `SGA-CHK-02`. `gate_armed` is `false`.

**TC-SGA-03: Blocked when sandbox verification missing**

- Preconditions: Internal module available.
- Steps:
  1. Call `build_sandbox_gate_arming_decision` with all fields true except `sandbox_verification_safe: false`.
- Expected result: Status is `blocked`.

**TC-SGA-04: Blocked when readiness not readyButDisabled**

- Preconditions: Internal module available.
- Steps:
  1. Call `build_sandbox_gate_arming_decision` with `sandbox_verification_safe: false`, `target_empty_safe: false`, `confirmation_gate_declared: false`, all other fields true.
- Expected result: Status is `blocked`. `blocked_reason` contains `SGA-CHK-04`.

**TC-SGA-05: ArmedNotExecutable when all prerequisites satisfied**

- Preconditions: Internal module available.
- Steps:
  1. Call `build_sandbox_gate_arming_decision` with `mode: sandboxOnlyInternal`, `explicit_internal_sandbox_arming_requested: true`, all other prerequisite fields true.
- Expected result: Status is `armedNotExecutable`. `gate_armed` is `true`. `executionEnabled` is `false`. `writesEnabled` is `false`. `readsEnabled` is `false`. `noChangesMade` is `true`. `networkReadsAttempted` is `false`. `networkWritesAttempted` is `false`. `safety_snapshot.write_gate_disabled` is `true`. `safety_snapshot.execution_enabled` is `false`. Message contains "NOT enabled", "not stored globally", "remains separate pending work".

**TC-SGA-06: evaluate_write_gate unchanged after armedNotExecutable**

- Preconditions: Internal module available.
- Steps:
  1. Call `build_sandbox_gate_arming_decision` with all prerequisites true — get `armedNotExecutable`.
  2. Call `evaluate_write_gate()`.
- Expected result: `evaluate_write_gate()` returns `Disabled/DisabledByProductPolicy`. The arming decision did not modify runtime gate behavior.

**TC-SGA-07: Decision is not persisted globally**

- Preconditions: Internal module available.
- Steps:
  1. Call `build_sandbox_gate_arming_decision` with all prerequisites true — get `armedNotExecutable`.
  2. Call `build_sandbox_gate_arming_decision` with `mode: disabled`.
- Expected result: Second call returns `blocked`. No state from the first call persists. Both calls are independent.

**TC-SGA-08: Safety invariants in all result states**

- Preconditions: Any state.
- Steps:
  1. Run `cargo test -- sandbox_gate_arming::tests`.
- Expected result: All Rust tests pass. `executionEnabled` is always `false`. `writesEnabled` is always `false`. `readsEnabled` is always `false`. `noChangesMade` is always `true`. `networkReadsAttempted` is always `false`. `networkWritesAttempted` is always `false`. `safety_snapshot.execution_enabled` is always `false`. No token, absolute path, record payload, attachment URL, old/new record ID, `"enabled"`, `"succeeded"`, `"executionReady"`, or `"restoreSuccess"` appears in any serialized result. No production mode exists. `evaluate_write_gate()` is never modified. The decision is not persisted globally. Live sandbox E2E restore execution remains separate pending work.

## Sandbox Restore Simulator Tests

**TC-SRS-01: Blocked when mode is disabled**

- Preconditions: Internal module available.
- Steps:
  1. Call `run_sandbox_restore_simulator` with `mode: disabled` and all other fields true.
- Expected result: Status is `blocked`. `gateArmed` is `false`. `executionEnabled` is `false`. `writesEnabled` is `false`. `readsEnabled` is `false`. `noChangesMade` is `true`. `airtableClientCalled` is `false`. `checkpointFileWritten` is `false`. `blocked_reason` contains `SRS-CHK-01`.

**TC-SRS-02: Blocked when explicit simulation flag missing**

- Preconditions: Internal module available.
- Steps:
  1. Call `run_sandbox_restore_simulator` with `mode: sandboxOnlyInternalSimulation`, `explicit_internal_simulation_requested: false`, all other fields true.
- Expected result: Status is `blocked`. `blocked_reason` contains `SRS-CHK-02`. `gateArmed` is `false`.

**TC-SRS-03: Blocked when arming decision is blocked**

- Preconditions: Internal module available.
- Steps:
  1. Call `run_sandbox_restore_simulator` with all fields true except `sandbox_verification_safe: false`.
- Expected result: Status is `blocked`. `blocked_reason` contains `SRS-CHK-04`.

**TC-SRS-04: SimulatedNotExecuted when all prerequisites satisfied**

- Preconditions: Internal module available.
- Steps:
  1. Call `run_sandbox_restore_simulator` with `mode: sandboxOnlyInternalSimulation`, `explicit_internal_simulation_requested: true`, all other prerequisite fields true.
- Expected result: Status is `simulatedNotExecuted`. `gateArmed` is `false`. `ephemeral_armed_decision_seen` is `true`. `executionEnabled` is `false`. `writesEnabled` is `false`. `readsEnabled` is `false`. `airtableClientCalled` is `false`. `checkpointFileWritten` is `false`. `noChangesMade` is `true`. `total_phase_count` is 8. Message contains "NOT armed", "NOT enabled", "No Airtable calls were made", "remains separate pending work".

**TC-SRS-05: All 8 phases represented with correct statuses**

- Preconditions: Internal module available; all prerequisites satisfied.
- Steps:
  1. Call `run_sandbox_restore_simulator` with all prerequisites true; inspect `phases`.
- Expected result: `phases` has exactly 8 entries. SRS-PH-01 is `simulated`. SRS-PH-02 is `skipped`. SRS-PH-03 is `simulated`. SRS-PH-04 is `skipped`. SRS-PH-05 is `simulated`. SRS-PH-06 is `skipped`. SRS-PH-07 is `simulated`. SRS-PH-08 is `simulated`. No phase has `succeeded`, `complete`, or `done` status.

**TC-SRS-06: evaluate_write_gate unchanged after simulation**

- Preconditions: Internal module available.
- Steps:
  1. Call `run_sandbox_restore_simulator` with all prerequisites true — get `simulatedNotExecuted`.
  2. Call `evaluate_write_gate()`.
- Expected result: `evaluate_write_gate()` returns `Disabled/DisabledByProductPolicy`. The simulation did not modify runtime gate behavior.

**TC-SRS-07: Simulator result is not persisted globally**

- Preconditions: Internal module available.
- Steps:
  1. Call `run_sandbox_restore_simulator` with all prerequisites true — get `simulatedNotExecuted`.
  2. Call `run_sandbox_restore_simulator` with `mode: disabled`.
- Expected result: Second call returns `blocked`. No state from the first call persists. Both calls are independent.

**TC-SRS-08: Safety invariants in all result states**

- Preconditions: Any state.
- Steps:
  1. Run `cargo test -- sandbox_restore_simulator::tests`.
- Expected result: All Rust tests pass. `gateArmed` (runtime/global) is always `false`. `executionEnabled` is always `false`. `writesEnabled` is always `false`. `readsEnabled` is always `false`. `noChangesMade` is always `true`. `networkReadsAttempted` is always `false`. `networkWritesAttempted` is always `false`. `airtableClientCalled` is always `false`. `checkpointFileWritten` is always `false`. Phase ordering is deterministic. No token, absolute path, record payload, attachment URL, old/new record ID, `"succeeded"`, `"enabled"`, `"executionReady"`, or `"restoreSuccess"` appears in any serialized result. No production mode exists. `evaluate_write_gate()` is never modified. The result is not persisted globally. Live sandbox E2E restore execution remains separate pending work.

---

## Sandbox Schema Write Adapter Test Cases (TC-SSWA-01 through TC-SSWA-08)

> Scope: `restore/sandbox_schema_write_adapter.rs` — internal Rust module only. No Tauri command, no TypeScript, no UI surface.

**TC-SRWA-01: Default disabled mode returns notExecuted**

- Preconditions: Internal module available (Rust unit tests).
- Steps:
  1. Call `build_sandbox_record_write_adapter` with `mode: disabled`, all flags false.
- Expected result: Status `notExecuted`. Mode `disabled`. `runtimeExecutionEnabled: false`. `appRuntimeWritesEnabled: false`. `appRuntimeReadsEnabled: false`. `networkWritesAttempted: false`. `noChangesMade: true`. `operations` is empty.

**TC-SRWA-02: Missing explicit internal record sandbox flag returns blocked**

- Preconditions: Internal module available.
- Steps:
  1. Call with `mode: sandboxOnlyInternal`, `explicit_internal_record_sandbox_call_requested: false`, all other prereqs true.
- Expected result: Status `blocked`. Blocked reason contains `SRWA-CHK-02`. All safety invariants hold.

**TC-SRWA-03: Prerequisite chain propagates correctly**

- Preconditions: Internal module available.
- Steps:
  1. Call with arming prereqs failing (e.g. `sandbox_verified: false`). Observe blocked at SRWA-CHK-04.
  2. Call with record executor plan blocked. Observe blocked at SRWA-CHK-06.
  3. Call with schema adapter plan blocked. Observe blocked at SRWA-CHK-07.
- Expected result: Each failure is blocked at the earliest failed check. Blocked reason identifies the check. All safety invariants hold in all cases.

**TC-SRWA-04: readyForSandboxCall returned when all prerequisites satisfied**

- Preconditions: Internal module available. Simple record plan (1 table, 10 records) and simple schema plan (1 table, 1 field).
- Steps:
  1. Call with all prereqs true, explicit flag true.
- Expected result: Status `readyForSandboxCall`. Operations contain only `createRecordBatchDescriptor`. No other operation kinds present. `runtimeExecutionEnabled: false`. `appRuntimeWritesEnabled: false`. `appRuntimeReadsEnabled: false`. `networkWritesAttempted: false`. `noChangesMade: true`. `safety_snapshot.write_gate_disabled: true`.

**TC-SRWA-05: Only first-pass create operations appear — linked update, schema, attachment excluded**

- Preconditions: Internal module available. Record plan with mixed operation kinds.
- Steps:
  1. Call with a plan that contains CreateRecordBatch operations.
- Expected result: `operations` list contains only `createRecordBatchDescriptor`. No `createTable`, `createField`, `updateLinkedRecord`, `preserveMetadata`, `skipComputedField`, or attachment operation kinds appear. Serialized JSON contains no `"attachment"`, `"linkedUpdate"`, `"createTable"`, `"createField"`, or `"fields":{` keys.

**TC-SRWA-06: Operation ordering is deterministic**

- Preconditions: Internal module available.
- Steps:
  1. Call twice with identical inputs.
  2. Compare operation ID sequences.
- Expected result: Sequences are identical. `SRWA-OP-NNN` prefix used consistently.

**TC-SRWA-07: No token/path/payload/raw HTTP/record ID in serialized result**

- Preconditions: Internal module available.
- Steps:
  1. Call with all prereqs true and serialize the result to JSON.
- Expected result: JSON contains no `"token"`, `"apiKey"`, `"pat_"`, `/Users/`, `/home/`, `"fields":{`, `"records":[`, `"body":{`, `"headers":{`, `"oldRecordId"`, `"newRecordId"`, `cdn.airtable.com`, or `"attachmentUrl"`. No `"succeeded"`, `"restoreComplete"`, or `"restoreSuccess"` in JSON.

**TC-SRWA-08: Safety invariants in all result states**

- Preconditions: Any state.
- Steps:
  1. Run `cargo test -- sandbox_record_write_adapter::tests`.
- Expected result: All Rust tests pass. `runtimeExecutionEnabled` is always `false`. `appRuntimeWritesEnabled` is always `false`. `appRuntimeReadsEnabled` is always `false`. `networkWritesAttempted` is always `false`. `noChangesMade` is always `true`. `safety_snapshot.write_gate_disabled` is always `true`. Operation ordering is deterministic. No token, absolute path, record payload, raw HTTP, old/new record ID, attachment URL, `"succeeded"`, `"enabled"`, `"executionReady"`, or `"restoreSuccess"` appears in any serialized result. No Tauri command added. No TypeScript/UI surface added. No production adapter wired. `evaluate_write_gate()` is never modified. The result is not persisted globally. Linked record updates, schema writes, final validation reads, attachment handling, and live end-to-end restore execution remain pending separate work.

---

**TC-SSWA-01: Default disabled mode returns notExecuted**

- Preconditions: Internal module available.
- Steps:
  1. Call `build_sandbox_schema_write_adapter` with `mode: disabled`, all prereqs false.
- Expected result: Status `notExecuted`. Mode `disabled`. `runtimeExecutionEnabled: false`. `appRuntimeWritesEnabled: false`. `appRuntimeReadsEnabled: false`. `networkWritesAttempted: false`. `noChangesMade: true`. No operations. Blocked reason is absent.

**TC-SSWA-02: Missing explicit flag returns blocked**

- Preconditions: Internal module available.
- Steps:
  1. Call with `mode: sandboxOnlyInternal`, `explicit_internal_schema_sandbox_call_requested: false`, all other prereqs true.
- Expected result: Status `blocked`. Blocked reason contains `SSWA-CHK-02`. All safety invariants hold.

**TC-SSWA-03: Prerequisite chain propagates correctly**

- Preconditions: Internal module available.
- Steps:
  1. Call with arming prereqs failing (e.g. `sandbox_verified: false`). Observe blocked at SSWA-CHK-04.
  2. Call with executor plan blocked. Observe blocked at SSWA-CHK-06.
- Expected result: Each failure is blocked at the earliest failed check. Blocked reason identifies the check. All safety invariants hold in all cases.

**TC-SSWA-04: readyForSandboxCall returned when all prerequisites satisfied**

- Preconditions: Internal module available. Simple schema plan with table + field.
- Steps:
  1. Call with all prereqs true, explicit flag true, sandbox plan with 1 table and 1 direct field.
- Expected result: Status `readyForSandboxCall`. Operations contain `createTableDescriptor` and `createFieldDescriptor`. No other operation kinds present. `runtimeExecutionEnabled: false`. `appRuntimeWritesEnabled: false`. `appRuntimeReadsEnabled: false`. `networkWritesAttempted: false`. `noChangesMade: true`. `safety_snapshot.write_gate_disabled: true`.

**TC-SSWA-05: Only schema operations appear — record, linked, attachment excluded**

- Preconditions: Internal module available. Schema plan containing deferred-linked and manual-action operations.
- Steps:
  1. Call with a plan that includes deferred linked fields and manual-action fields.
- Expected result: `operations` list contains only `createTableDescriptor` and `createFieldDescriptor`. No `deferLinkedField`, `manualAction`, `createRecord`, `updateRecord`, `linkedUpdate`, or attachment operation kinds appear. Serialized JSON contains no `"attachment"`, `"linkedUpdate"`, `"createRecord"`, or `"records"` keys.

**TC-SSWA-06: Operation ordering is deterministic**

- Preconditions: Internal module available.
- Steps:
  1. Call twice with identical inputs.
  2. Compare operation ID sequences.
- Expected result: Sequences are identical. Table descriptors precede field descriptors in both calls. `SSWA-OP-NNN` prefix used consistently.

**TC-SSWA-07: No token/path/payload/raw HTTP/record ID in serialized result**

- Preconditions: Internal module available.
- Steps:
  1. Call with all prereqs true and serialize the result to JSON.
- Expected result: JSON contains no `"token"`, `"apiKey"`, `"pat_"`, `/Users/`, `/home/`, `"fields":{`, `"records":[`, `"body":{`, `"headers":{`, `"oldRecordId"`, `"newRecordId"`, `"rec`, `cdn.airtable.com`, or `"attachmentUrl"`. No `"succeeded"`, `"restoreComplete"`, or `"restoreSuccess"` in JSON.

**TC-SSWA-08: Safety invariants in all result states**

- Preconditions: Any state.
- Steps:
  1. Run `cargo test -- sandbox_schema_write_adapter::tests`.
- Expected result: All Rust tests pass. `runtimeExecutionEnabled` is always `false`. `appRuntimeWritesEnabled` is always `false`. `appRuntimeReadsEnabled` is always `false`. `networkWritesAttempted` is always `false`. `noChangesMade` is always `true`. `safety_snapshot.write_gate_disabled` is always `true`. Operation ordering is deterministic. No token, absolute path, record payload, raw HTTP, old/new record ID, attachment URL, `"succeeded"`, `"enabled"`, `"executionReady"`, or `"restoreSuccess"` appears in any serialized result. No Tauri command added. No TypeScript/UI surface added. No production adapter wired. `evaluate_write_gate()` is never modified. The result is not persisted globally. Record writes, linked record updates, final validation reads, attachment handling, and live end-to-end restore execution remain pending separate work.
