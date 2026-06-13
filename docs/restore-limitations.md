# Restore Limitations

AirBridge restore is best-effort and intentionally conservative.

## v0.1 restore targets

Supported:

- New base.
- Empty existing base.

Not supported:

- Non-empty base.
- Merge into existing base.
- Overwrite existing base.
- Partial table restore.

## Features not restored in v0.1

AirBridge v0.1 does not restore:

- Airtable automations.
- Airtable interfaces.
- Base permissions.
- Sharing settings.
- External integrations.
- Webhooks.
- Full attachment files.
- Exact system field values.

## System field limitations

System fields such as created time, last modified time, created by, last modified by, and autonumber are controlled by Airtable. AirBridge may back up their metadata or observed values, but cannot guarantee exact preservation during restore.

## Computed field limitations

Formula, lookup, rollup, and count fields may depend on Airtable-specific configuration and linked fields. AirBridge may back up these field definitions and values, but v0.1 does not guarantee automatic recreation.

## Attachment limitations

In v0.1, AirBridge backs up attachment metadata. Full file download and re-upload are planned for later exploration. The record import planner assigns `MetadataOnly` policy to all attachment fields; files must be manually re-attached after restore.

## Collaborator limitations

Collaborator fields depend on users available in the destination workspace. AirBridge v0.1 does not provide full user mapping.

## Linked-record limitations

Linked records are restored using old-to-new record ID mapping. Broken links may occur if referenced records were excluded from backup or if restore fails before mapping completes.

## Reporting

All restore limitations should be visible in:

- Restore plan.
- Compatibility report.
- Restore report.
