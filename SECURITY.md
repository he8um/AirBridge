# Security Policy

AirBridge handles local copies of Airtable data and uses credentials to access Airtable APIs. Security reports are taken seriously.

## Supported versions

During early development, only the latest released version is expected to receive security fixes.

| Version | Supported |
| --- | --- |
| Latest release | Yes |
| Older releases | Best effort |

## Reporting a vulnerability

If you discover a vulnerability, do not open a public issue if the report includes exploit details, tokens, private data, or a reproducible path to data exposure.

Use the private security reporting channel configured for the repository, or contact the maintainer through the support channel listed in `SUPPORT.md`.

Please include:

- AirBridge version.
- Operating system.
- Affected workflow: backup, inspect, validate, restore, packaging, or token storage.
- Steps to reproduce using sanitized data.
- Expected impact.
- Suggested fix, if known.

Do not include real Airtable tokens, real customer data, or unredacted backup packages.

## Security design goals

AirBridge aims to:

- Keep backup data local by default.
- Avoid telemetry in v0.1.
- Avoid storing tokens in backup packages.
- Use operating system credential storage where available.
- Avoid logging secrets.
- Provide redaction and exclusion options.
- Require explicit confirmation before restore write operations.
- Produce restore reports for partial or failed operations.

## Sensitive data warning

`.airbridge` files may contain Airtable record data. Treat them like database exports. Do not upload them to public issues unless they are synthetic or fully sanitized.
