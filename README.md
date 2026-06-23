# AirBridge

**Local backup and restore for Airtable bases.**

AirBridge is a local-first, open-source desktop app for backing up, inspecting, validating, and restoring Airtable bases with best-effort schema reconstruction.

AirBridge is designed for teams that use Airtable as an operational database and need a practical way to create local backups, inspect what those backups contain, validate backup integrity, and restore a base into a new or empty destination base when needed.

## What AirBridge does

AirBridge focuses on four core workflows:

1. **Backup** — Export Airtable base metadata, tables, fields, records, linked-record references, select options, view metadata where available, attachment metadata, and reports into a portable `.airbridge` package.
2. **Inspect** — Open a backup package and review its contents before attempting any restore operation.
3. **Validate** — Check package structure, manifest compatibility, checksums, record files, schema consistency, and restore compatibility.
4. **Restore** — Plan and preview a restore into a new or empty base using a staged dry-run process with a clear readiness report. Runtime restore execution is not yet available in v0.1.0-alpha — live writes to Airtable are disabled by policy pending product and security approval.

## What AirBridge does not promise

AirBridge does not claim to be a full-fidelity Airtable clone tool. The goal is reliable local backup and transparent best-effort restore, not perfect replication of every Airtable feature.

AirBridge v0.1 does not restore:

- Airtable automations
- Airtable interfaces
- Base permissions and sharing settings
- Exact system field values such as created time, modified time, created by, or last modified by
- Full attachment file re-upload
- Restore into non-empty bases
- Merge or overwrite workflows

Some computed or advanced field types are backed up and reported, but may require manual recreation depending on Airtable API support and account permissions.

## Platform targets

AirBridge is planned as a cross-platform desktop application:

- macOS
- Windows
- Linux

## Technology stack

```text
Desktop framework: Tauri
Frontend: React + TypeScript
Core engine: Rust
Local state: SQLite
Credentials: OS credential store where available
Backup package: .airbridge ZIP package
Distribution: GitHub Releases
```

## Backup package format

AirBridge uses a portable `.airbridge` package. The file is a standard ZIP archive with a documented internal structure.

```text
example.airbridge
├── manifest.json
├── base.json
├── schema.json
├── tables/
│   └── tbl_xxx/
│       ├── table.json
│       ├── fields.json
│       ├── records.jsonl
│       └── records.csv
├── links/
│   └── linked-records.jsonl
├── attachments/
│   └── metadata.jsonl
├── reports/
│   ├── backup-report.json
│   ├── compatibility-report.json
│   └── validation-report.json
└── checksums/
    └── sha256.json
```

`records.jsonl` is the restore source of truth. `records.csv` is provided for human-readable inspection.

## Restore philosophy

Restore is intentionally conservative. In v0.1 AirBridge supports:

- Restore to a new base, when the token and workspace permissions allow it
- Restore to an empty existing base

AirBridge does not restore into non-empty bases in v0.1. This avoids destructive operations, unexpected overwrites, duplicate conflicts, and unsafe merge behavior.

## Security and privacy

AirBridge is local-first by design:

- No telemetry in v0.1
- No cloud sync
- No token stored inside backup packages
- Token storage uses the operating system credential store where available
- Backup files remain under user control
- Redaction and exclusion options are part of the backup flow

Backup files can contain sensitive Airtable data. Treat `.airbridge` files like database exports.

## Project status

AirBridge is in public alpha (v0.1.0-alpha).

**Available in v0.1.0-alpha:**

- Airtable Personal Access Token connection and permission inspection
- Base catalog and schema read
- Schema and record backup with `.airbridge` package creation
- Backup package inspection and validation
- Restore compatibility report
- Restore dry-run planning (read-only, no token required)
- Restore schema creation planning and record import planning (read-only)
- Restore safety gate evaluation (all preconditions validated; write engine disabled)
- Optional OS keychain token storage
- Local job history (in-memory)
- Cross-platform release build workflow (macOS, Linux, Windows)

**Not yet available:**

- Runtime restore execution (live writes to Airtable are disabled; requires product and security approval)
- Attachment binary backup/restore (metadata only)
- Credential auto-fill from saved token
- Streaming backup progress
- Job history persistence between sessions

## Documentation

Start with:

- [Overview](docs/overview.md)
- [Getting Started](docs/getting-started.md)
- [Backup Guide](docs/backup-guide.md)
- [Restore Guide](docs/restore-guide.md)
- [Backup Format](docs/backup-format.md)
- [Field Compatibility](docs/field-compatibility.md)
- [Restore Limitations](docs/restore-limitations.md)
- [Security and Privacy](docs/security-and-privacy.md)
- [Architecture Overview](docs/architecture/overview.md)
- [Development Setup](docs/development/development-setup.md)

## Current Status (v0.1.0-alpha)

AirBridge is in public alpha. The current build supports:

- Personal access token connection checks and permission inspection.
- Base catalog and schema read.
- Backup planning and records export planning.
- Backup package creation (requires explicit file selection and confirmation text `CREATE BACKUP`).
- Package inspection and validation.
- Restore dry-run planning (read-only, no token required, no Airtable calls).
- Restore schema creation planning (read-only, no token required, no Airtable calls).
- Restore record import planning (read-only, no token required, no Airtable calls).
- Restore execution safety gate — validates all preconditions. When all pass, returns a `readyButDisabled` result. No Airtable base, table, field, or record is created.
- Optional OS keychain token storage (Settings → Saved Credentials). Token is never stored in files, logs, or results.
- Local activity history — recent operations shown on the Reports page with safe summaries (no tokens, no full paths). History is in-memory and does not persist between sessions.
- Cross-platform release builds via a `workflow_dispatch`-only GitHub Actions workflow. No release is published automatically.

**Restore write execution is disabled.** This is a deliberate product policy, not a gap in the safety contract. Live Airtable writes require explicit product and security approval. The dry-run and planning flows are complete; no restore execution UI or Tauri command exists.

**Attachment handling is metadata-only.** Backup packages capture attachment metadata and URLs. File bytes are not downloaded or re-uploaded. Attachment URLs from backup time may expire.

**Token persistence is manual.** Tokens saved to the OS keychain are not yet automatically retrieved for operations — users must still paste the token manually when initiating a backup or connection check.

See the full details in:

- [Feature Status](docs/product/feature-status.md)
- [Known Limitations](docs/release/known-limitations.md)
- [v0.1.0-alpha Readiness](docs/release/v0.1.0-alpha-readiness.md)
- [Release Process](docs/release/release-process.md)

---

## Contributing

Contributions are welcome through issues and pull requests. Please read:

- [Contributing](CONTRIBUTING.md)
- [Code of Conduct](CODE_OF_CONDUCT.md)
- [Security Policy](SECURITY.md)
- [Roadmap](ROADMAP.md)

## License

AirBridge is licensed under the Apache License, Version 2.0. See [LICENSE](LICENSE).
