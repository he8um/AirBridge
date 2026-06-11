# Backup Guide

This guide describes the AirBridge backup workflow.

## Backup goal

A backup should create a local `.airbridge` package containing the Airtable base structure and record data needed for inspection, validation, and best-effort restore.

## Backup contents

AirBridge v0.1 should back up:

- Base metadata.
- Tables.
- Fields.
- Field options.
- View metadata where available.
- Records.
- Linked-record references.
- Select options.
- Attachment metadata.
- Backup report.
- Compatibility report.
- Checksums.

## Backup wizard

The backup wizard should follow this flow:

1. Select connection.
2. Select base.
3. Choose backup scope.
4. Configure redaction and exclusions.
5. Review backup plan.
6. Run backup.
7. View backup report.

## Backup scope options

Recommended options:

- Include schema.
- Include records.
- Include view metadata.
- Include linked-record references.
- Include attachment metadata.
- Exclude selected tables.
- Exclude selected fields.
- Exclude attachment URLs.
- Redact email-like values.
- Redact collaborator/user fields.

## Backup plan

Before running, AirBridge should show:

- Source base name.
- Source base ID.
- Table count estimate.
- Field count estimate.
- Record count estimate where available.
- Selected exclusions.
- Redaction choices.
- Output path.
- Expected package extension.

## Progress states

The backup progress UI should show meaningful stages:

```text
Connecting
Fetching base schema
Exporting table 1/8
Exporting records 400/2400
Writing package
Generating checksums
Validating package
Done
```

## Failure behavior

If backup fails, AirBridge should:

- Stop safely.
- Preserve partial logs.
- Avoid writing a package that appears complete.
- Mark partial packages as invalid or incomplete.
- Show the failing table, field, or request where possible.
- Suggest retry when the failure is transient.

## Backup report

The backup report should include:

- Started time.
- Finished time.
- Source base.
- Table count.
- Field count.
- Record count.
- Attachment metadata count.
- Exclusions.
- Redactions.
- Warnings.
- Errors.
- Package path.
- Package checksum status.
