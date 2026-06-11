# Security and Privacy

AirBridge handles user-controlled Airtable exports. Security and privacy are core product requirements.

## Design goals

- Local-first behavior.
- No telemetry in v0.1.
- No cloud sync in v0.1.
- No token inside backup packages.
- No secrets in logs.
- Clear warning when backups may contain sensitive data.
- Redaction and exclusion options.
- Explicit confirmation before restore writes.

## Credential storage

AirBridge should use operating system credential storage where available:

| Platform | Storage target |
| --- | --- |
| macOS | Keychain |
| Windows | Credential Manager |
| Linux | Secret Service or keyring |

If secure storage is not available, AirBridge should support session-only token usage.

## Backup sensitivity

`.airbridge` files may contain:

- Record values.
- Table and field names.
- Attachment metadata.
- Attachment URLs unless excluded.
- User or collaborator references unless redacted.

Treat backup files like database exports.

## Logging policy

Logs must not include:

- Tokens.
- Authorization headers.
- Full record payloads by default.
- Private attachment URLs unless explicitly included in debug exports.

## Restore safety

Before restore, AirBridge should show:

- Destination target.
- Write operations to be performed.
- Unsupported fields.
- Manual actions.
- Warnings about partial restore.

## Future encryption

Optional backup encryption is planned for a later release after the backup format stabilizes.
