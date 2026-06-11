# Corrupted Backup Fixture

This directory intentionally contains no valid backup files.

## Purpose

It is used to test AirBridge's error-handling behavior when the application attempts to open a corrupted, incomplete, or otherwise invalid backup package.

## Expected Test Behavior

Tests that reference this fixture should expect one or more of the following outcomes:

- An error state is shown in the UI (e.g., "This backup package could not be read")
- Validation failures are reported with actionable messages
- The application degrades gracefully and does not crash or leave the UI in an inconsistent state
- No partial data is displayed from a package that could not be fully validated
- The user is given a clear path back to a functional state (e.g., close and open a different backup)

## What Is Not Present

- No `manifest.json` — tests can verify the missing-manifest error path
- No `schema.json` — tests can verify the missing-schema error path
- No `records.jsonl` — tests can verify the missing-records error path

## Adding Intentionally Malformed Files

If a specific test requires a malformed file (e.g., a `manifest.json` with invalid JSON), create it in a subdirectory of `corrupted-backup/` named after the scenario, and document it here. Do not place malformed files directly in this directory, as they may confuse tooling that scans for JSON syntax errors.
