# Logging Guidelines

## Purpose

Logs should include job ID, phase, resource IDs, status, retry count, and sanitized error detail. Logs must not include tokens, authorization headers, or full record payloads by default.

## Required practices

- Keep changes small enough to review.
- Prefer explicit behavior over hidden magic.
- Update documentation when behavior changes.
- Add tests for backup, restore, compatibility, and validation logic where practical.
- Never commit tokens, real backup packages, private logs, or sensitive data.
- Prefer safe defaults for write operations.

## Review checklist

- Does the change preserve local-first behavior?
- Does it keep restore behavior conservative?
- Does it handle errors clearly?
- Does it avoid logging sensitive values?
- Does it work across supported platforms or document platform limits?
- Does it have tests or a clear manual test note?

## Anti-patterns

- Large unrelated refactors.
- Silent restore skips.
- String-only errors with no code.
- API calls scattered across UI components.
- Loading large backups fully into memory.
- Assuming every Airtable feature can be recreated automatically.
