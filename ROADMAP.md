# Roadmap

This roadmap describes the intended direction of AirBridge. It is not a contract, and scope may change based on technical findings, user feedback, and maintainability.

## v0.1 — Public Alpha

Goal: prove the core local backup and best-effort restore workflow.

Planned capabilities:

- Desktop app for macOS, Windows, and Linux.
- Airtable Personal Access Token connection.
- Secure token storage where supported by the operating system.
- Base selection.
- Schema backup.
- Records backup.
- Linked-record reference backup.
- Attachment metadata backup.
- `.airbridge` package creation.
- Backup inspector.
- Local package validation.
- Restore compatibility report.
- Restore to new base where permissions allow.
- Restore to empty existing base.
- Basic linked-record remapping.
- Dry-run restore planning.
- Restore report.
- No telemetry.

## v0.2 — Safer Restore

Goal: improve reliability, reporting, and restore confidence.

Planned capabilities:

- Better field compatibility handling.
- Better linked-record validation.
- Better restore error recovery.
- Field and table exclusion UI.
- Redaction options.
- Improved job logs.
- Improved retry and backoff UX.
- CSV preview for backed-up tables.
- More detailed restore reports.

## v0.3 — Backup Hardening

Goal: improve backup resilience and portability.

Planned capabilities:

- Optional backup encryption.
- Optional attachment download.
- Experimental attachment restore.
- Resume failed backup and restore jobs where practical.
- Backup comparison.
- Schema diff.
- More comprehensive package validation.

## v1.0 — Stable Release

Goal: make AirBridge stable enough for routine use by technical and non-technical operators.

Planned capabilities:

- Stable `.airbridge` package specification.
- Mature compatibility matrix.
- Comprehensive test fixtures.
- Stable release process.
- Clear installation docs.
- Clear security policy.
- Signed installers if practical.
- Mature contributor workflow.

## Non-goals for early releases

The following are intentionally out of scope for early releases:

- Cloud backup storage.
- Restore into non-empty bases.
- Merge workflows.
- Destructive overwrite workflows.
- Airtable automation restore.
- Airtable interface restore.
- Permission cloning.
- Scheduled backup.
- Multi-user collaboration inside AirBridge.
