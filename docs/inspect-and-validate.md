# Inspect and Validate

AirBridge should make backup packages understandable before restore.

## Inspect mode

Inspect mode opens a `.airbridge` file and displays package contents without modifying Airtable.

Inspect should show:

- Backup creation time.
- Source base name and ID.
- Package format version.
- App version used to create the backup.
- Table count.
- Field count.
- Record count.
- Attachment metadata count.
- Linked-record relationship count.
- Redactions applied.
- Exclusions applied.
- Compatibility warnings.

## Local validation

Local validation checks package integrity and structure.

Validation should verify:

- Required files exist.
- `manifest.json` is valid.
- `schema.json` is valid.
- Table folders are present.
- `records.jsonl` files are parseable.
- Checksums match.
- Linked-record references point to known backed-up records where possible.
- Reports are consistent with package contents.

## Restore compatibility validation

Compatibility validation estimates restore fidelity.

It should answer:

- Which fields can be recreated automatically?
- Which fields can be backed up but not restored automatically?
- Which values cannot be preserved exactly?
- Which linked records can be reconnected?
- Which attachments are metadata-only?
- Which manual actions are required?

## Validation statuses

Suggested statuses:

```text
valid
valid_with_warnings
invalid
unsupported_version
corrupt
incomplete
```

## Output

Validation results should be visible in the UI and saved as:

```text
reports/validation-report.json
```
