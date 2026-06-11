# Empty States

## Purpose

Empty states should teach the next action: create a connection, start a backup, open a package, or review documentation. Avoid generic blank screens.

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
