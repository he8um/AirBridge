# Frontend Architecture

## Purpose

The frontend should use feature-based organization. Pages orchestrate user flows; feature modules own state and API command wrappers; shared components provide tables, cards, banners, forms, progress views, and report viewers. Frontend state should not duplicate source-of-truth job state beyond display caching.

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
