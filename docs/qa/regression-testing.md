# Regression Testing

## What Counts as a Regression

A regression is a defect in a new build that was not present in a previously verified build. For AirBridge, regressions are classified in three scenarios:

1. **Behavior that previously worked correctly now produces an incorrect result.** For example: a backup that previously captured 50 records now captures 49.
2. **An error that was previously handled gracefully now causes a crash or silent failure.** For example: opening a backup with a missing schema field now panics instead of showing an error message.
3. **A property that was previously verified (e.g., no token in logs) is no longer true.** For example: after a refactor of the logging module, the API token starts appearing in log output.

Bug fixes that also change previously documented behavior are not regressions but must be noted in the CHANGELOG.

---

## Regression Priority Tiers

| Tier | Definition | Action |
|------|-----------|--------|
| Critical | Data loss, silent corruption, security property violated, or application crash | Block release immediately. Must be fixed before the release proceeds. |
| High | Core feature broken (backup fails, restore fails, backup package unreadable), or a previously resolved bug has returned | Must be fixed or explicitly risk-accepted before release. |
| Low | Visual regression, minor behavioral change, cosmetic issue, or edge-case failure in a non-critical path | Filed as an issue and tracked for a future release. Does not block the current release. |

---

## Regression Test Baseline Process

1. **After each verified release,** the state of the release build is the new baseline.
2. The build number or version tag is recorded in `docs/qa/regression-baseline.md` (maintained separately) along with the date and the person who verified it.
3. Any manual test case that passed in the baseline is considered a regression candidate for subsequent builds.
4. When a new release candidate is built, run the regression checks described below and compare results against the baseline.

---

## Automated Regression Checks (Vitest and cargo test)

The automated test suite is the first line of regression detection. These checks run on every commit via CI.

### What Vitest Covers

- Component rendering regressions: a previously passing snapshot test that now fails indicates a visual or structural regression in a component.
- State machine regressions: tests that verify transitions (idle → in-progress → complete) catch regressions in progress-tracking logic.
- Error boundary regressions: tests that mock backend errors and verify the UI response catch regressions in error handling.

### What cargo test Covers

- Parsing regressions: fixture-based parse tests catch regressions in the Rust parsing logic for manifest, schema, and records files.
- Serialization regressions: round-trip tests catch regressions where a backup package written by the new build cannot be re-read correctly.
- Field type mapping regressions: per-field-type unit tests catch regressions when a field type that previously mapped correctly now maps to an incorrect internal representation.
- Checksum regressions: integrity checks catch regressions where the checksum computation produces a different result for the same input.

### Running Automated Checks

```bash
# Rust tests
cargo test --workspace

# Frontend component tests
pnpm run test

# With coverage (for coverage regression tracking)
cargo tarpaulin --out Lcov
pnpm run test --coverage
```

Compare coverage reports between the baseline and the release candidate. A significant drop in coverage (more than 5%) should be investigated.

---

## Manual Regression Checks Before Release

The following manual checks are performed against the release candidate build, not the dev build. These supplement the automated suite for areas that are not fully automatable.

### Backup Regression Checks

- [ ] Backup a known test base and verify the record count matches the previous baseline count for that base.
- [ ] Open the produced backup package and confirm `manifest.json`, `schema.json`, and `records.jsonl` are all present and valid JSON/JSONL.
- [ ] Confirm that a backup package produced by the previous release can still be opened by the current build (forward compatibility).

### Restore Regression Checks

- [ ] Use the `simple-base` fixture to run a dry-run restore and confirm the report output matches the expected format documented in the baseline.
- [ ] Use the `linked-records-base` fixture to run a dry-run restore and confirm linked record relationships are correctly identified in the report.

### UI Regression Checks

- [ ] Launch the application and navigate through all primary views: Home, Connections, Backup, Restore, Reports, Settings.
- [ ] Confirm that no view is blank, has layout breakage, or shows a JavaScript error overlay.
- [ ] Confirm that the version string in Settings matches the release tag.

### Security Regression Checks

- [ ] Confirm that no token appears in the log after a backup (see `security-privacy-qa.md`).
- [ ] Confirm that no outbound connections are made to non-Airtable hosts during a backup.

---

## How to Record a Regression

When a regression is found, open an issue in the project repository with the following information:

1. **Title:** Start with `[Regression]` to distinguish from new bugs. Example: `[Regression] Backup silently drops records over pagination boundary`.
2. **Affected version:** The build or version in which the regression was found.
3. **Last known good version:** The most recent version where the behavior was correct.
4. **Steps to reproduce:** Minimal steps that demonstrate the regression.
5. **Expected result:** What the baseline behavior was.
6. **Actual result:** What the broken behavior is.
7. **Priority tier:** Critical, High, or Low (see table above).
8. **Linked test case:** Reference the manual test case (e.g., `TC-BACK-01`) or automated test file that covers this behavior.

---

## Maintaining the Regression Test Suite

- When a regression is fixed, a corresponding automated test must be added (or an existing test must be tightened) so that the same regression cannot recur silently.
- The test added for a regression fix should include a comment referencing the issue number: `// Regression test for issue #42`.
- If a regression is decided to be acceptable (e.g., a deliberate behavior change), the test baseline must be updated and the change noted in the CHANGELOG.
