# QA Strategy

## Goals and Principles

The AirBridge QA strategy aims to ensure the application:

- Correctly reads and writes the AirBridge backup package format without data loss or corruption.
- Handles a wide variety of Airtable schema structures, including edge cases such as linked records, many field types, and missing optional fields.
- Behaves consistently on all supported platforms without platform-specific regressions.
- Fails safely and informatively — errors are surfaced to the user with clear messaging, not silent data loss.
- Never transmits data outside the local machine during any operation covered by the application.

### Core Principles

- **Test with fixtures, not live data.** No test in the automated suite should require a real Airtable token or an active internet connection.
- **Test behavior, not implementation.** Unit tests focus on outputs given known inputs, not on internal implementation details that are likely to change.
- **Prefer explicit over implicit.** Test cases state their preconditions and expected results clearly. Ambiguous test names or vague assertions are treated as defects.
- **Fail loud.** A flaky test must be investigated and fixed or explicitly quarantined, not silently ignored.

---

## Testing Layers

### Unit Tests (Rust — `cargo test`)

The Rust backend contains the core logic for parsing backup packages, validating manifests and schemas, mapping field types, and writing backup files. Unit tests cover:

- Parsing valid and invalid `manifest.json` structures.
- Parsing valid and invalid `schema.json` structures, including all supported field types.
- JSONL record parsing, including malformed lines and empty files.
- Field type mapping and serialization round-trips.
- Checksum computation and verification.
- Error types and their human-readable messages.

**Coverage target:** 80% line coverage on the `airbridge-core` crate.

### Component Tests (React / Vitest)

The desktop frontend is tested with Vitest and React Testing Library. Component tests cover:

- Rendering of all major UI views: connections list, backup progress, restore progress, report viewer, settings.
- User interactions: button clicks, form submissions, file picker invocations (mocked).
- State transitions: idle → in-progress → complete, idle → in-progress → error.
- Error boundary behavior when the Tauri backend returns an error response.
- Correct display of fixture-derived data passed as props or via mocked Tauri commands.

**Coverage target:** 70% line coverage on `src/` frontend code.

### Integration Tests

Integration tests exercise the Tauri command layer by invoking Rust-backed commands with controlled inputs (loaded from fixture files) and verifying the responses. They run in a headless environment without a display server.

Covered scenarios:
- Opening a valid backup package returns the expected manifest and schema summary.
- Opening a corrupted backup package returns a structured error, not a panic.
- A backup round-trip on fixture data produces output that re-parses identically to the input.
- Restore dry-run on a fixture produces a correct diff report with no writes performed.

### Manual Tests

Manual tests are documented in `manual-test-plan.md`. They cover flows that are impractical to automate:

- Visual layout and typography on each supported platform.
- File dialog interaction (system-native dialog behavior is not mockable in CI).
- Installer experience (download, install, first launch).
- Real Airtable API behavior when a valid token is present (performed in a controlled test environment, never in CI).
- Accessibility with live screen reader software.

### Accessibility Tests

Documented in `accessibility-qa.md`. Covers keyboard navigation, focus management, screen reader output, color contrast, and reduced-motion behavior.

### Cross-Platform Tests

Documented in `cross-platform-qa.md`. Covers install behavior, path handling, font rendering, and known platform-specific issues on macOS (Intel and Apple Silicon), Windows 10/11, and Ubuntu/Debian Linux.

---

## QA Gates Per Release

The following gates must be passed before a release is published. Each gate is documented in the corresponding checklist file.

| Gate | File | Required for |
|------|------|--------------|
| All automated tests pass | CI status | Every release |
| Manual test plan executed | `manual-test-plan.md` | Every release |
| Release QA checklist complete | `release-qa-checklist.md` | Every release |
| Backup checklist complete | `backup-qa-checklist.md` | Every release |
| Restore checklist complete | `restore-qa-checklist.md` | Every release |
| Accessibility pass | `accessibility-qa.md` | Every minor+ release |
| Cross-platform pass | `cross-platform-qa.md` | Every minor+ release |
| Security/privacy verification | `security-privacy-qa.md` | Every release |
| CHANGELOG updated | — | Every release |

---

## Test Coverage Goals

| Layer | Target | Measured by |
|-------|--------|-------------|
| Rust core logic | 80% line coverage | `cargo tarpaulin` |
| React components | 70% line coverage | Vitest coverage report |
| Integration (Tauri commands) | All happy paths + known error paths | Manual audit |

Coverage targets are goals, not hard gates. A release is not blocked solely because coverage is below target if the gap is in trivially simple or unreachable code. Coverage gaps in core parsing and validation logic are treated as blocking.

---

## What Is Explicitly Not Tested in CI

- **Live Airtable API calls.** CI has no API credentials. Tests that require real API responses use pre-recorded fixture data instead.
- **Real attachment download/upload.** Attachment URLs in fixtures point to `example.com`. Actual attachment transfer is tested manually.
- **App auto-update flow.** The update mechanism is tested manually against a staging update server.
- **Operating system–specific dialogs.** Native file pickers and system notifications cannot be automated reliably across platforms.
- **Installer behavior.** Installers are smoke-tested manually on each platform after a build.

---

## Fixture-Based Testing Rationale

Using local fixture files rather than live API calls provides several benefits:

- **Speed.** Fixture-based tests complete in milliseconds rather than seconds.
- **Determinism.** A fixture always returns the same data, so test failures are reproducible.
- **No rate limits.** Tests can run as many times as needed without hitting API quotas.
- **No credential management.** CI runners need no secrets to execute the test suite.
- **Privacy.** Real user data never enters the test environment.

The tradeoff is that fixtures must be kept accurate with respect to the real Airtable API response format. When Airtable changes their API, affected fixtures should be updated as part of the same change that updates the parsing code.
