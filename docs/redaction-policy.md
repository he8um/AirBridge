# Redaction Policy

Redaction reduces sensitive data in backup packages. It does not guarantee complete anonymization.

## v0.1 redaction options

AirBridge should support:

- Exclude selected tables.
- Exclude selected fields.
- Exclude attachment URLs.
- Redact email-like values.
- Redact collaborator/user fields.

## Exclusion vs redaction

Exclusion removes data from the backup. Redaction keeps the field or record structure but replaces sensitive values.

Example redacted value:

```json
{"Email":"[REDACTED_EMAIL]"}
```

## Redaction report

The backup report should include:

- Redaction modes enabled.
- Tables excluded.
- Fields excluded.
- Number of values redacted where practical.
- Whether attachment URLs were included.

## Limitations

Pattern-based redaction can miss sensitive data embedded in free text. Users are responsible for reviewing backup contents before sharing.

## Restore impact

Redacted backups restore redacted values. AirBridge should make this clear in the restore plan.
