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
