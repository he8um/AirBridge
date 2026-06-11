# Privacy

AirBridge is designed as a local-first desktop application.

## Data processed by AirBridge

AirBridge may process:

- Airtable base metadata.
- Table names.
- Field names and field configuration.
- Record values.
- Linked-record references.
- Attachment metadata.
- User-selected export paths.
- Local job logs.
- Restore reports.

## Local-first behavior

In v0.1:

- AirBridge does not provide cloud backup.
- AirBridge does not send telemetry.
- AirBridge does not upload backup packages to a remote service.
- Backup files are written to locations selected by the user.

## Credentials

AirBridge uses Airtable Personal Access Tokens in v0.1.

Credential handling goals:

- Tokens are not stored inside `.airbridge` backup files.
- Tokens are not written to logs.
- Tokens are stored using operating system credential storage where available.
- If secure storage is unavailable, AirBridge should support session-only token use.

## Backup files

`.airbridge` files may contain sensitive data. Users are responsible for storing, sharing, encrypting, and deleting backup packages appropriately.

## Redaction

AirBridge includes redaction and exclusion options to reduce sensitive data in backup packages, including field exclusion, table exclusion, attachment URL exclusion, and value redaction patterns.

## Third-party services

AirBridge communicates with Airtable APIs only when the user connects a token and performs backup or restore operations. AirBridge is not affiliated with Airtable.
