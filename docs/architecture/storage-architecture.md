# Storage Architecture

## Purpose

SQLite stores local app state such as connection profiles without raw tokens, recent backup paths, job summaries, restore history, and settings. Backup content belongs in `.airbridge` files, not the app database.

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
