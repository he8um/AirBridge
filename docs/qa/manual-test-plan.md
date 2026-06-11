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
