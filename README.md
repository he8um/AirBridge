# AirBridge

**Local backup and restore for Airtable bases.**

AirBridge is a local-first, open-source desktop app for backing up, inspecting, validating, and restoring Airtable bases with best-effort schema reconstruction.

AirBridge is designed for teams that use Airtable as an operational database and need a practical way to create local backups, inspect what those backups contain, validate backup integrity, and restore a base into a new or empty destination base when needed.

## What AirBridge does

AirBridge focuses on four core workflows:

1. **Backup** — Export Airtable base metadata, tables, fields, records, linked-record references, select options, view metadata where available, attachment metadata, and reports into a portable `.airbridge` package.
2. **Inspect** — Open a backup package and review its contents before attempting any restore operation.
3. **Validate** — Check package structure, manifest compatibility, checksums, record files, schema consistency, and restore compatibility.
4. **Restore** — Recreate supported base structure and records into a new or empty base using a staged restore process and a clear restore report.

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

AirBridge is planned as an open-source project with an initial public alpha target.

Planned v0.1 scope:

- Airtable Personal Access Token connection
- Base selection
- Schema and record backup
- `.airbridge` package creation
- Backup inspector
- Local package validation
- Restore compatibility report
- Restore to new or empty base
- Basic linked-record remapping
- Dry-run restore planning
- Restore report
- Cross-platform release builds

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

AirBridge is preparing for a v0.1.0-alpha release. The current build supports:

- Personal access token connection checks and permission inspection.
- Base catalog and schema read.
- Backup planning and records export planning.
- Backup package creation (requires explicit file selection and confirmation text).
- Package inspection and validation.
- Restore dry-run planning (read-only, no token required).
- Restore schema creation planning — generates an ordered schema creation plan (table steps, field steps, deferred linked fields, dependency graph) from a dry-run result. Read-only; no token required; no Airtable tables or fields are created.
- Restore record import planning — generates a complete record import batch plan (batch counts at size 10, field import policies, linked record second-pass plans, attachment policies, checkpoint plans, retry policy) from a dry-run result and schema plan. Read-only; no token required; no Airtable records are created.
- Restore execution safety gate (all preconditions validated; write engine not yet enabled).
- Local activity history — recent operations are shown on the Reports page with safe summaries (no tokens, no full paths, no record payloads). History is in-memory and does not persist between sessions.

**Restore write execution is disabled in this version.** The safety gate validates all preconditions and returns a `readyButDisabled` result. No Airtable base, table, field, or record is created by restore operations.

**Token persistence is not implemented.** Tokens must be entered for each operation.

**v0.1.0-alpha preparation:** A `workflow_dispatch`-only GitHub Actions release workflow builds platform artifacts (macOS, Linux, Windows) and uploads them as workflow run artifacts. No release is published automatically. See the [release process](docs/release/release-process.md) for how to trigger and review a release build.

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
