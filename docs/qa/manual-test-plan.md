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
