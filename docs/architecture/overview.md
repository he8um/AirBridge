# Architecture Overview

## Purpose

AirBridge is split into a React/TypeScript frontend, a Tauri desktop shell, and a Rust core engine. The frontend owns presentation and user interaction. The Rust core owns Airtable API calls, package writing, validation, restore planning, restore execution, job progress, and secure local integration. This separation keeps business logic testable and avoids placing sensitive workflow behavior in the UI layer.

## Design requirements

- Keep responsibilities separated by domain.
- Keep user-facing workflows responsive.
- Prefer structured errors over string-only failures.
- Keep backup and restore behavior testable without UI.
- Avoid leaking credentials or sensitive record data to logs.
- Use clear boundaries between platform integration and product logic.

## Component documentation

- [Airtable API client](airtable-api-client.md)

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
