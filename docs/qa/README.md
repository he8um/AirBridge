# AirBridge QA Documentation

This directory contains the quality assurance documentation for the AirBridge project.

## QA Philosophy

AirBridge is a local-first desktop application. No data leaves the user's device during normal operation. The QA process reflects this:

- **No real data in tests.** All test fixtures use synthetic, obviously fake records. Real Airtable tokens, base IDs, or record values must never appear in the test suite.
- **No live API calls in CI.** Automated tests run fully offline using fixture data. Network-dependent tests are manual-only and clearly labeled.
- **No telemetry.** The application ships with no analytics or error-reporting calls. QA includes explicit verification that no outbound network traffic is generated.
- **Reproducibility first.** A test failure on any supported platform must be reproducible from the fixture files alone, with no external dependencies.

## Documents in This Directory

| File | Description |
|------|-------------|
| `qa-strategy.md` | Overall QA goals, testing layers, coverage targets, and release gates |
| `manual-test-plan.md` | Step-by-step manual test cases for all major application flows |
| `release-qa-checklist.md` | Pre-release verification checklist covering builds, signing, and regression |
| `backup-qa-checklist.md` | Detailed checklist for the backup flow specifically |
| `restore-qa-checklist.md` | Detailed checklist for the restore flow specifically |
| `cross-platform-qa.md` | Platform-specific install and behavior checks for macOS, Windows, and Linux |
| `accessibility-qa.md` | Keyboard navigation, screen reader, contrast, and ARIA label checks |
| `security-privacy-qa.md` | Verification that no credentials, telemetry, or record data leave the device |
| `test-fixtures.md` | How to use and extend the fixture data in `fixtures/` |
| `regression-testing.md` | Regression baseline process, tiers, and recording procedures |
| `bug-report-review-checklist.md` | Checklist for triaging and reviewing incoming bug reports |

## Quick Reference: QA Gates

Before any release build is published, the following must be complete:

1. All automated tests pass on macOS, Windows, and Linux CI runners.
2. Manual test plan executed and all blocking issues resolved.
3. Release QA checklist signed off.
4. Backup and restore checklists verified against the release build.
5. Accessibility pass completed.
6. Security/privacy verification completed.
7. CHANGELOG updated with the new version's entries.
