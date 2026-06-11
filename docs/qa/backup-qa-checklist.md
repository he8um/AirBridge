# Backup QA Checklist

Use this checklist when verifying the backup functionality against a release build. Mark each item pass (P), fail (F), or not applicable (N/A). Record the build version and test date at the top.

**Build version:** ___________
**Test date:** ___________
**Tested by:** ___________
**Platform:** ___________

---

## Pre-Backup: Token and Permission Validation

- [ ] **Token validation before backup starts.** If the token is invalid or expired, an error is shown before any API call to list or read records is made. The error message is actionable (e.g., "Your token appears to be invalid — check it in Settings").
- [ ] **Schema read permission check.** If the token lacks permission to read schema metadata, a specific error is shown identifying the permission gap, not a generic failure.
- [ ] **Records read permission check.** If the token can read schema but not records, the backup reports a permission error for record access. The schema-only fallback is offered if appropriate.
- [ ] **No token logged.** After a validation failure, confirm the token value does not appear in the application log file.

---

## During Backup: API Interaction

- [ ] **All tables are discovered.** The backup includes all tables present in the base, including tables added since the connection was first used.
- [ ] **All fields are captured per table.** Each field in each table appears in the backup schema with its correct type and options.
- [ ] **All records are fetched.** For a base with a known record count, verify the backup record count matches. Pay attention to bases with more than 100 records (pagination boundary).
- [ ] **Field type metadata is preserved.** Select fields include their option choices. Date fields include their format settings. Number fields include precision. Linked record fields include the linked table ID.
- [ ] **Attachment URL handling.** If the base contains attachment fields, the backup package includes the attachment URLs as provided by the API. Verify that the application does not attempt to download attachment content unless that feature is explicitly enabled.
- [ ] **Rate limit behavior.** If the API returns a 429 Too Many Requests response, the backup pauses, retries after the indicated delay, and continues to completion. The user sees a progress indicator that communicates the delay, not a frozen UI.
- [ ] **Partial failure handling.** If a single table's record fetch fails mid-backup (simulate by revoking permissions on one table), the error is recorded in the manifest, the remaining tables are still attempted, and the final report identifies which tables failed.

---

## Output Package Validation

- [ ] **Package is written to the chosen location.** After a successful backup, a file or directory exists at the path the user selected.
- [ ] **Manifest correctness.** Open `manifest.json` and verify:
  - `backupId` is a non-empty unique string.
  - `baseId` matches the source base.
  - `baseName` matches the source base's display name.
  - `createdAt` is a valid ISO 8601 timestamp.
  - `airBridgeVersion` matches the application version.
  - `backupType` is `"full"` (or `"schema-only"` when appropriate).
  - `tableCount` matches the actual number of tables in `schema.json`.
  - `recordCount` matches the actual number of lines in `records.jsonl`.
  - `schemaOnly` is `false` for a full backup and `true` for a schema-only backup.
- [ ] **Schema completeness.** Open `schema.json` and confirm:
  - The `tables` array contains one entry per table.
  - Each table entry has `id`, `name`, and `fields`.
  - Each field entry has `id`, `name`, `type`, and, where applicable, `options`.
- [ ] **Record count accuracy.** Count the lines in `records.jsonl`. The line count must equal `recordCount` in the manifest. Empty lines at the end of the file must not be counted.
- [ ] **Field type preservation.** For a base that includes singleSelect, date, number, and checkbox fields, verify that the backed-up field values in `records.jsonl` match the values visible in the Airtable UI.
- [ ] **JSONL format validity.** Every line in `records.jsonl` must be a valid JSON object. Verify using a JSONL linter or by parsing with `cargo test` against the file.

---

## Integrity and Safety

- [ ] **File size sanity.** The backup package size is plausible for the record count. A backup of 1,000 records should not be 0 bytes or 1 GB.
- [ ] **Checksum / integrity marker.** If AirBridge writes a checksum file (e.g., `SHA256SUMS`) into the package, verify that re-computing the checksum of each file matches the recorded value.
- [ ] **Overwrite behavior.** If a backup is started with an output path where a file already exists:
  - The user is warned before any write occurs.
  - If the user cancels, the existing file is untouched.
  - If the user confirms, the existing file is replaced by the new backup.
  - In no case is the existing file partially overwritten and left in a corrupt state.
- [ ] **Redaction of sensitive fields.** If the user has configured any fields as redacted, verify that those field values appear as `null` or a placeholder in `records.jsonl`, not their actual values.
- [ ] **No token or credentials in the package.** Open all files in the backup package and confirm that no Airtable token, user email, or other credential appears anywhere.
- [ ] **File permissions on output.** On macOS and Linux, the backup files are not world-writable (permissions should be `0644` or more restrictive for files, `0755` or more restrictive for the directory).

---

## Schema-Only Backup

- [ ] Backup with schema-only enabled completes without error.
- [ ] Output contains `manifest.json` with `schemaOnly: true`.
- [ ] Output contains `schema.json` with the full table and field definitions.
- [ ] Output does not contain `records.jsonl`, or the file is present but empty.
- [ ] `recordCount` in the manifest is `0`.

---

## Notes and Failures

Record any failures here with a brief description and steps to reproduce.

| Item | Status | Notes |
|------|--------|-------|
| | | |

---

## Backup Planning (Dry-Run)

These items cover the backup planning flow added in v0.1. A "plan" is a dry-run summary — no backup file is written.

- [ ] **Base selector shows accessible bases.** On the Backups page, the "Backup Planning" card lists the same bases as the Connection page reported.
- [ ] **"Load Schema" button requires a base selection.** The button is disabled when no base is selected.
- [ ] **Schema loads and is summarised correctly.** After clicking "Load Schema", the card shows table count, restorable field count, metadata-only count, and unknown count. Values match what the Connection page schema view reports.
- [ ] **"Generate Backup Plan" button is disabled until schema is loaded.** Clicking before a schema is available does not trigger a plan call.
- [ ] **Plan result shows correct table count and field count.** After generating a plan, the displayed counts match the schema summary.
- [ ] **Attachment fields produce a warning.** If any table contains a `multipleAttachments` field, a warning with code `ATTACHMENT_METADATA_ONLY` appears in the plan.
- [ ] **Linked record fields produce a warning.** If any table contains a `multipleRecordLinks` field, a warning with code `LINKED_RECORD_REMAPPING` appears.
- [ ] **Formula/rollup/count fields produce an info notice.** Tables with computed fields show a `COMPUTED_FIELD` info entry.
- [ ] **"No backup file has been created yet" copy is visible.** The dry-run notice is present and clearly labelled.
- [ ] **`dryRun` is true in the Tauri command response.** Inspect the JSON returned by `create_backup_plan` and confirm `dryRun: true`.
- [ ] **`outputPackagePath` is absent in the response.** The plan JSON does not contain an `outputPackagePath` key.
- [ ] **No token appears in plan output.** Inspect the plan JSON and confirm no token string is present.
- [ ] **Plan estimate shows "unknown" when record counts are unavailable.** The record read pages field reads "unknown (no record counts available)".
- [ ] **Plan scope matches the request scope.** If "Full backup" is the scope, the plan `scope` field is `full`.

---

## Package Format (Writer / Reader / Validator)

These items cover the package format foundation added in V0.1. The writer and validator are tested with synthetic data only; live backup export is not yet wired to the UI.

- [ ] **"Package Format" section is visible on the Backups page.** The section explains that the package writer is available but live export is not yet enabled.
- [ ] **No backup file is created from the UI.** Confirm there is no file picker and no "create package" button that writes to disk.
- [ ] **Writer produces a valid ZIP.** Open a test-generated `.airbridge` file with a standard ZIP tool and confirm it opens without errors.
- [ ] **manifest.json is present and parseable.** Extract `manifest.json` from a test package and confirm it contains `format`, `formatVersion`, `appVersion`, `createdAt`, `source`, `contents`, `security`, and `package` fields.
- [ ] **checksums/sha256.json is present and lists all entries.** Confirm the checksums file exists and each key is a relative archive path with no leading `/`.
- [ ] **SHA-256 hashes validate correctly.** For each entry in `checksums/sha256.json`, manually compute `sha256(<entry content>)` and confirm it matches.
- [ ] **No token appears in any package entry.** Search all text entries in the test package for `pat`, `Bearer`, or any known token string.
- [ ] **No local filesystem paths in archive entries.** Confirm no archive entry name starts with `/`, `Users/`, or `home/`.
- [ ] **Validator returns `valid` for a correctly written package.** Run `validate_package` (test path) and confirm `status == "valid"` and `errors` is empty.
- [ ] **Validator returns `invalid` for a package with a missing required entry.** Remove `manifest.json` or `checksums/sha256.json` from a copy of the test package and confirm the validator reports `MISSING_REQUIRED_ENTRY`.
- [ ] **Validator returns `invalid` for a tampered entry.** Modify any text entry after write and confirm the validator reports `CHECKSUM_MISMATCH`.
- [ ] **Validator returns `invalid` for an unsupported format version.** Change `formatVersion` in manifest to `"99.0.0"` and confirm the validator reports `UNSUPPORTED_FORMAT_VERSION`.
- [ ] **JSONL records are preserved line-by-line.** Open `tables/<id>/records.jsonl` and confirm each line is a valid JSON object with an `id` field.
- [ ] **Attachment metadata-only policy is documented.** `security.containsAttachmentUrls` is `false` in a V0.1 package; attachment file content is not present.


---

## Records Export Planning (Dry-Run)

These items cover the records export planning layer. No live records are fetched and no package files are written.

- [ ] **"Records Export Plan" section is visible on the Backups page.** The section appears below the Backup Planning section.
- [ ] **"Generate Records Export Plan" button appears.** The button is present and disabled until a backup plan has been generated.
- [ ] **Button becomes enabled after a backup plan is generated.** Once the Backup Planning section produces a plan, the export plan button is enabled.
- [ ] **Generated plan shows table count.** The plan result shows the number of tables included.
- [ ] **Known record count is displayed.** If a table has a known record count, it is shown in the table export plan row.
- [ ] **Unknown record count is shown as "unknown".** Tables without a known record count display "unknown" for the record count.
- [ ] **Estimated page count is shown for known counts.** For a table with 250 records and page size 100, the estimate shows "~3 pages".
- [ ] **Unknown page estimate shown for unknown counts.** Tables without record counts show "unknown" for estimated pages.
- [ ] **JSONL output entry path is shown.** Each table shows the target archive path (e.g. `tables/tblAbc01/records.jsonl`).
- [ ] **JSONL entry path contains no absolute filesystem path.** The path does not start with `/` or contain `Users/` or `home/`.
- [ ] **Linked record extraction policy is shown.** Tables with linked record fields show the `remappingRequiredForRestore` policy.
- [ ] **Attachment metadata policy is shown.** Tables with attachment fields show the `metadataOnly` policy.
- [ ] **UNKNOWN_RECORD_COUNT warning appears.** Tables without known record counts show a warning with code `UNKNOWN_RECORD_COUNT`.
- [ ] **ATTACHMENT_METADATA_ONLY warning appears.** Tables with attachment fields show a warning with code `ATTACHMENT_METADATA_ONLY`.
- [ ] **LINKED_RECORD_REMAPPING warning appears.** Tables with linked record fields show a warning with code `LINKED_RECORD_REMAPPING`.
- [ ] **"No records have been fetched" notice is visible.** The notice is clearly displayed in the plan result.
- [ ] **"No backup file has been written" notice is visible.** The planning-only status is explicitly communicated.
- [ ] **`plannedOnly` is true in the Tauri command response.** Inspect the JSON returned by `create_records_export_plan` and confirm `plannedOnly: true`.
- [ ] **`outputPackagePath` is absent in the response.** The plan JSON does not contain an `outputPackagePath` key.
- [ ] **No token appears in export plan output.** Inspect the plan JSON and confirm no token string is present.

---

## Paginated Record Export Engine (Internal)

These items cover the engine layer. All are verified by automated tests — no live API calls should be made during testing.

- [ ] **Unit tests: all pass.** Run `cargo test --lib`. All 317+ unit tests pass with no failures.
- [ ] **Integration tests: all pass.** Run `cargo test --test export_engine_integration`. All 3 integration tests pass.
- [ ] **Two-page pagination accumulates all records.** The engine correctly follows the `offset` cursor across pages and accumulates all records from both pages.
- [ ] **Linked record references are extracted.** For a table with a `multipleRecordLinks` field, linked record IDs appear in the `linked-records.jsonl` bytes.
- [ ] **Attachment metadata is extracted without URLs.** For a table with a `multipleAttachments` field, `filename` and `urlPresent` appear in the metadata, but no `https://` URL is stored.
- [ ] **`urlPresent: true` recorded when API returned a URL.** Even though the URL is discarded, the flag is set so future phases know which records had downloadable files.
- [ ] **Integration test package validates as `Valid`.** The package written by the integration test passes `validate_package()` with `status: Valid`.
- [ ] **Package is written to tempdir only.** No `.airbridge` file is written outside `tempfile::tempdir()` in tests.
- [ ] **No token sentinel in any output.** JSONL lines, linked-records bytes, and attachment-metadata bytes do not contain the test token sentinel.
- [ ] **No absolute filesystem paths in JSONL.** Record JSONL lines do not contain `/Users/` or any absolute path component.
- [ ] **401 maps to `InvalidToken` error.** A mock 401 response causes `ExportEngineError::InvalidToken`.
- [ ] **429 maps to `RateLimited` error.** A mock 429 response causes `ExportEngineError::RateLimited`.
- [ ] **No live export button in the UI.** The Backups page has no button that triggers a live Airtable export in V0.1.
- [ ] **UI notice describes engine status.** The Package Format section states that the paginated record export engine is available but live backup creation is not enabled in the UI.

---

## Backup Job Orchestration (Internal)

These items cover the job orchestration layer. All are verified by automated tests — no live API calls should be made during testing.

- [ ] **Unit tests: all pass.** Run `cargo test --lib`. All unit tests pass with no failures.
- [ ] **Integration tests: all pass.** Run `cargo test --test export_engine_integration`. All 3 integration tests pass.
- [ ] **Frontend tests: all pass.** Run `npm --prefix apps/desktop run test`. All tests pass including `backupJobOrchestration.test.tsx`.
- [ ] **Successful orchestration returns `succeeded` status.** A single-table mock run produces `BackupJobResult.status == "succeeded"`.
- [ ] **Event order is correct.** Events emitted in order: `jobStarted` → `phaseStarted(planning)` → `phaseStarted(recordsExport)` → `tableExportStarted` → `tableExportCompleted` → `phaseStarted(packageBuild)` → `packageWriteStarted` → `packageWriteCompleted` → `phaseStarted(validation)` → `validationStarted` → `validationCompleted` → `phaseStarted(completed)` → `jobSucceeded`.
- [ ] **All events carry `jobId`.** No event is missing the `jobId` field.
- [ ] **No token in any event or result.** Search all serialised events and the result for any token sentinel string — none should appear.
- [ ] **No absolute path in any event or result.** No event or result field contains `/Users/`, `/home/`, or any absolute path component.
- [ ] **No attachment URL in any event or result.** No `https://` URL appears in any event or result.
- [ ] **401 maps to `AUTH_FAILED`.** A mock 401 response produces `BackupJobResult` with `errors[0].code == "AUTH_FAILED"` and `recoverable == false`.
- [ ] **403 maps to `PERMISSION_DENIED`.** A mock 403 response produces `errors[0].code == "PERMISSION_DENIED"`.
- [ ] **429 maps to `RATE_LIMITED`.** A mock 429 response produces `errors[0].code == "RATE_LIMITED"` and `recoverable == true`.
- [ ] **Cancellation before export emits `jobCancelled`.** Setting the cancellation token before `run()` produces a `JobCancelled` event with `atPhase: "planning"` and `BackupJobResult.status == "cancelled"`.
- [ ] **Cancelled result has no `packageSummary`.** `packageSummary` and `validationSummary` are absent on a cancelled result.
- [ ] **Package summary has `encrypted: false` for V0.1.** `BackupJobPackageSummary.encrypted` is always `false`.
- [ ] **Package summary has `attachmentPolicy: "metadataOnly"`.** `BackupJobPackageSummary.attachmentPolicy` is `"metadataOnly"`.
- [ ] **UI section "Backup Job Pipeline" is visible.** The Backups page shows the pipeline readiness notice.
- [ ] **UI states live backup creation is not enabled.** The section contains copy matching "live backup creation … not enabled yet".
- [ ] **UI states no file is created from the screen.** The section contains copy matching "no file is created from this screen".
- [ ] **No enabled production backup-trigger button.** There is no enabled button matching "start backup", "run backup", or "create backup" on the Backups page.
