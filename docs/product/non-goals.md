# Non-goals

## Summary

Non-goals include cloud backup, scheduled jobs, multi-user collaboration, full Airtable clone fidelity, automation restore, interface restore, and merge restore in early releases.

## Requirements

- The product must solve a real local backup and restore problem.
- The user must understand what is included in a backup.
- The user must understand what restore can and cannot recreate.
- Restore must avoid destructive behavior in v0.1.
- Reports must be useful for troubleshooting and manual follow-up.

## v0.1 acceptance notes

A v0.1 workflow is acceptable when:

- A user can connect a token.
- A user can select a base.
- A user can produce a `.airbridge` package.
- A user can inspect the package.
- A user can see compatibility warnings.
- A user can run dry-run.
- A user can restore to a new or empty base within supported field constraints.
- A user receives a clear report.
