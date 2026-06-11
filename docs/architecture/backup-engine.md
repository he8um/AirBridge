# Backup Engine

## Purpose

The backup engine fetches schema, exports records page by page, extracts linked-record references, writes table files, writes reports, computes checksums, and validates the package before marking it complete. Partial package files should not be presented as successful backups.

## Design requirements

- Keep responsibilities separated by domain.
- Keep user-facing workflows responsive.
- Prefer structured errors over string-only failures.
- Keep backup and restore behavior testable without UI.
- Avoid leaking credentials or sensitive record data to logs.
- Use clear boundaries between platform integration and product logic.

## Main components

```text
React UI
Tauri command boundary
Rust core engine
SQLite local state
OS credential store
.airbridge package files
Airtable Web API
```

## Risks

- Long-running jobs blocking the UI.
- Restore behavior spread across too many modules.
- API assumptions becoming stale.
- Logs accidentally containing sensitive data.
- Platform-specific packaging issues.

## Acceptance criteria

- The architecture can support backup, inspect, validate, and restore workflows.
- The UI can display progress for long-running jobs.
- Core behavior can be tested independently.
- Restore planning can run without modifying Airtable.
- Sensitive values are excluded from logs by default.
