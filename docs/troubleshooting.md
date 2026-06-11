# Troubleshooting

## Token is rejected

Possible causes:

- Token was copied incorrectly.
- Token was deleted or regenerated.
- Token does not include required scopes.
- Token does not have access to the selected base.

Suggested action:

- Create or update the token.
- Confirm resource access.
- Reconnect in AirBridge.

## Cannot read base schema

Possible causes:

- Missing `schema.bases:read` scope.
- Token resource access does not include the base.
- User lacks permission on the base.

## Cannot restore to new base

Possible causes:

- Missing `schema.bases:write` scope.
- User lacks permission in destination workspace.
- Destination workspace restrictions.

Suggested action:

- Use restore to empty existing base if create-base permission is unavailable.
- Confirm token scope and workspace permission.

## Backup stops after some records

Possible causes:

- Network interruption.
- API rate limit.
- Airtable service error.
- Local disk write error.

Suggested action:

- Retry backup.
- Check available disk space.
- Review logs.

## Validation fails checksum

Possible causes:

- Package was modified after creation.
- Package is incomplete.
- File corruption occurred during transfer.

Suggested action:

- Recreate the backup.
- Do not restore from a corrupted package.

## Linked records are not fully restored

Possible causes:

- Referenced records were excluded.
- Record import failed before mapping completed.
- Linked field was unsupported or partially created.

Suggested action:

- Review restore report.
- Restore from a full-base backup.
- Avoid excluding linked tables when restore fidelity matters.
