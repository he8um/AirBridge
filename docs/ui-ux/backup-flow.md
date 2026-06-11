# Backup Flow

## Purpose

The backup flow should guide the user from connection to base selection, scope, redaction, plan review, progress, and report. The user should understand what data is included.

## UX requirements

- Make data safety visible.
- Use clear wording for destructive or write operations.
- Require confirmation before restore.
- Show warnings before action, not only after failure.
- Provide details without overwhelming the default view.
- Keep navigation predictable.

## UI states to define

- Empty.
- Loading.
- Valid.
- Valid with warnings.
- Failed.
- Partial success.
- Cancelled.
- Needs user action.

## Copy guidelines

Use direct language:

```text
This backup contains 8 tables and 24,392 records.
6 fields require manual action during restore.
Attachment files will not be restored in v0.1.
```

Avoid vague language:

```text
Something went wrong.
Some items may not work.
```

## Acceptance criteria

- A first-time user can complete a backup without reading developer docs.
- Restore warnings are visible before writes start.
- Reports are understandable by non-developers.
- Advanced technical details remain available for debugging.
