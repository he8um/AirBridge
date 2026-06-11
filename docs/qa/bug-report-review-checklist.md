# Bug Report Review Checklist

Use this checklist when triaging an incoming bug report. Work through each section in order. If a section's requirements are not met, request the missing information from the reporter before assigning a priority.

---

## Step 1: Reproducibility

- [ ] **Reproduced independently.** A maintainer or contributor other than the original reporter has been able to reproduce the issue using the reported steps.
- [ ] **Reproduction is deterministic.** The issue occurs every time the steps are followed, or its intermittent nature has been documented (e.g., "occurs ~1 in 5 runs").
- [ ] **Reproduction environment is clean.** The issue has been reproduced on a machine that is not the reporter's, or on a fresh install of AirBridge, to rule out environment-specific causes.

If the issue cannot be reproduced: label it `needs-reproduction` and ask the reporter for more detail before proceeding.

---

## Step 2: Version and Platform Identification

- [ ] **AirBridge version identified.** The exact version string (e.g., `0.1.0`) from the application's Settings view is recorded in the report. "Latest" is not sufficient.
- [ ] **Operating system and version identified.** For example: `macOS 14.3 (Apple Silicon)`, `Windows 11 23H2`, `Ubuntu 22.04 LTS`.
- [ ] **Architecture identified** (if relevant): Intel x86_64 vs. Apple Silicon arm64 on macOS.
- [ ] **Install source identified:** Was the application installed from a release artifact, a nightly build, or built from source?

---

## Step 3: Severity Assessment

Assign one of the following severity levels:

| Severity | Definition |
|---------|-----------|
| Critical | Data loss, silent data corruption, security property violated (e.g., token appears in log), or application crash on a common action |
| High | Core feature completely broken (backup fails every time, restore cannot complete), significant UI failure preventing key flows |
| Medium | Feature partially broken, workaround exists, or issue affects an edge case in an otherwise working flow |
| Low | Cosmetic issue, minor UX inconvenience, or issue in a rarely used feature with a clear workaround |

- [ ] Severity level assigned and recorded on the issue.
- [ ] If Critical: escalate immediately and notify maintainers.

---

## Step 4: Steps to Reproduce

- [ ] Steps are numbered and specific — each step describes a single action.
- [ ] Steps include the exact input values used (e.g., which fixture file, what field type, what token scope).
- [ ] Steps do not require access to a private Airtable base or real credentials to reproduce. If they do, a synthetic reproduction case using fixtures is requested from the reporter.
- [ ] Steps begin from a defined starting state (e.g., "Fresh install, no connections configured" or "Backup of `simple-base` fixture loaded").

---

## Step 5: Expected vs. Actual Behavior

- [ ] **Expected behavior** is clearly stated: what should happen when the steps are followed.
- [ ] **Actual behavior** is clearly stated: what actually happens, with specific details (error message text, incorrect count, visual glitch description).
- [ ] Expected and actual behaviors are distinct and non-overlapping. A report that says "it should work" without defining "work" must be clarified.

---

## Step 6: Logs and Supporting Evidence

- [ ] **Log file attached or excerpted.** For any backend error (backup failure, restore failure, crash), the relevant section of the application log file is included.
- [ ] **Screenshot or screen recording attached** for visual/UI bugs. For layout bugs, screenshots on both the affected platform and a working platform are helpful.
- [ ] **Log file has been reviewed** to confirm it does not contain real API tokens, real record data, or real personal information before being attached to a public issue. If it does, request that the reporter redact sensitive values before attaching.
- [ ] If no log is attached and the issue involves a backend error: request the log before closing or assigning.

---

## Step 7: Area Linkage

Identify which area of the application the bug affects and confirm it is labeled accordingly:

- [ ] Label `area/backup` if the issue occurs during the backup flow.
- [ ] Label `area/restore` if the issue occurs during the restore flow.
- [ ] Label `area/ui` if the issue is a UI rendering or navigation problem.
- [ ] Label `area/settings` if the issue is in the Settings or connection management flow.
- [ ] Label `area/reports` if the issue is in the backup summary or diff report views.
- [ ] Label `area/packaging` if the issue involves the backup package format itself (manifest, schema, records).
- [ ] Label `area/platform` and the relevant OS tag if the issue is platform-specific.

---

## Step 8: Duplicate Check

- [ ] The issue tracker has been searched for existing open and closed issues that describe the same behavior.
- [ ] If a duplicate is found: close the new issue as a duplicate and link to the original. Add any new reproduction information from the duplicate to the original issue.
- [ ] If a previously closed issue has been reopened by this report: re-open the original issue and note the regression, then follow the regression recording steps in `regression-testing.md`.

---

## Step 9: Triage Label

Apply one of the following triage labels:

| Label | Meaning |
|-------|--------|
| `triage/confirmed` | Reproduced and fully triaged; ready for prioritization |
| `triage/needs-reproduction` | Cannot be reproduced yet; awaiting more information |
| `triage/needs-info` | Missing version, platform, steps, or logs |
| `triage/duplicate` | Closed as a duplicate of another issue |
| `triage/by-design` | The reported behavior is intentional; close with explanation |
| `triage/wont-fix` | The issue is valid but will not be addressed; close with explanation |

- [ ] Exactly one triage label is applied.

---

## Step 10: Priority Assignment

Once the report is confirmed (`triage/confirmed`), assign a priority:

| Priority | Criteria |
|---------|---------|
| `priority/critical` | Critical severity; blocks a release |
| `priority/high` | High severity; should be fixed in the current or next release |
| `priority/medium` | Medium severity; target for the next release cycle |
| `priority/low` | Low severity; fix when time permits |

- [ ] Priority label applied.
- [ ] If `priority/critical`: notify maintainers and update the release status if a release is in progress.
- [ ] If `priority/high` and a release is in progress: add to the release's blocking issues list.

---

## Summary

A fully triaged bug report has:

1. Confirmed reproducibility
2. Exact version and platform
3. Severity and priority labels
4. Area label
5. Clear steps, expected result, and actual result
6. Log file or screenshot (where applicable)
7. Triage label set to `triage/confirmed`
8. No duplicate in the tracker
