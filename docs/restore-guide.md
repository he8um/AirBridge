# Restore Guide

This guide describes the AirBridge restore workflow.

## Restore principle

Restore should be conservative, explicit, and report-driven. AirBridge should not silently overwrite or merge data in v0.1.

## Supported restore targets in v0.1

- New base.
- Empty existing base.

## Unsupported restore targets in v0.1

- Non-empty base.
- Merge into existing base.
- Overwrite existing base.
- Partial table restore.

## Restore wizard

The restore wizard should follow this flow:

1. Select `.airbridge` file.
2. Inspect package.
3. Validate package.
4. Review compatibility report.
5. Select destination workspace or empty base.
6. Run dry-run.
7. Confirm restore.
8. Run restore.
9. View restore report.

## Restore algorithm

The restore engine should run in staged phases:

```text
1. Read package
2. Validate manifest and checksums
3. Parse schema
4. Generate restore plan
5. Check destination permissions
6. Create destination base if needed
7. Create tables
8. Create supported non-linked fields
9. Create linked-record fields after all tables exist
10. Import records without linked-record values
11. Build old_record_id -> new_record_id mapping
12. Reconnect linked records
13. Restore attachment metadata where possible
14. Validate created counts
15. Generate restore report
```

## Why linked records require two phases

Airtable assigns new record IDs during restore. Linked-record values from the source package reference old record IDs. AirBridge must first import records and capture the mapping from old IDs to new IDs. After mapping exists, linked-record fields can be updated to point to the new records.

## Dry-run

Dry-run should show:

- Destination target.
- Tables to create.
- Fields to create.
- Records to import.
- Linked-record relationships to reconnect.
- Unsupported fields.
- Manual actions required.
- Estimated API calls.
- Permission issues.

Dry-run should not create or modify Airtable data.

## Restore report

The restore report should include:

- Created base ID.
- Created tables.
- Created fields.
- Created records.
- Linked records reconnected.
- Skipped fields.
- Partial fields.
- Failed records.
- Manual actions.
- API errors.
- Duration.
- Final status.

## Restore safety

AirBridge should require explicit confirmation before any write operation. Restore warnings must be shown before the user can start restore.
