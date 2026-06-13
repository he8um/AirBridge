# Release QA Checklist

Complete this checklist before publishing any release. Each item must be checked and initialed by the person who verified it. Open issues must be triaged before the release proceeds.

**Release version:** ___________
**Release date:** ___________
**Verified by:** ___________

---

## Release Workflow (workflow_dispatch)

- [ ] The release workflow was triggered via **Actions → Release → Run workflow** with `version` set to the correct release tag.
- [ ] The workflow completed successfully on all three matrix runners (macOS, Linux, Windows) — no matrix leg failed.
- [ ] Each matrix leg ran the quality gates step (`npm --prefix apps/desktop run check`) without error.
- [ ] Workflow artifacts are present: `airbridge-<version>-macOS`, `airbridge-<version>-Linux`, `airbridge-<version>-Windows`.
- [ ] Artifact names match the convention in `docs/release/artifact-naming.md`.
- [ ] The workflow did not create a GitHub release automatically — only workflow run artifacts are present.

---

## Build Artifacts

- [ ] macOS (Intel x86_64) `.dmg` artifact is present in the workflow run artifacts.
- [ ] macOS (Apple Silicon arm64) `.dmg` artifact is present in the workflow run artifacts.
- [ ] Windows (x64) `.msi` or `.exe` installer is present.
- [ ] Linux (x86_64) `.AppImage` or `.deb` artifact is present.
- [ ] All artifact file sizes are within expected range (not suspiciously small or unexpectedly large).
- [ ] SHA-256 checksums have been computed and recorded for all artifacts.
- [ ] Artifact filenames include the correct version number (e.g., `airbridge-v0.1.0-alpha-macOS`).

---

## Version String

- [ ] The version string displayed in the application's About/Settings view matches the release tag.
- [ ] The version in `tauri.conf.json` (or equivalent) matches the release tag.
- [ ] The version in `Cargo.toml` matches the release tag.
- [ ] The version in `package.json` matches the release tag.
- [ ] No "-dev", "-alpha", "-debug", or similar pre-release suffix is present in a production release build.

---

## No Debug or Dev Artifacts Shipped

- [ ] Developer tools (DevTools / inspector) are not accessible in the production build (keyboard shortcut or menu entry is absent or disabled).
- [ ] No `.map` source map files are bundled into the production frontend assets (or, if present, they are intentionally included and this is noted).
- [ ] No debug logging is enabled by default (log level is `info` or `warn`, not `debug` or `trace`).
- [ ] No hardcoded fixture paths, local file paths, or developer machine paths appear in the build.
- [ ] The build was produced from a clean checkout of the tagged commit, not from a dirty working tree.

---

## Code Signing and Notarization (macOS)

- [ ] Both macOS artifacts are code-signed with the official Developer ID certificate.
- [ ] Both macOS artifacts have been notarized by Apple and have received a notarization ticket.
- [ ] Notarization stapling has been applied to both `.dmg` files.
- [ ] Gatekeeper passes on a clean macOS system (launching the app does not show "unidentified developer" warning).

---

## Windows Signing

- [ ] The Windows installer is signed with the official code-signing certificate.
- [ ] SmartScreen does not show an "unrecognized publisher" warning when running the installer.

---

## Installer Smoke Tests

### macOS

- [ ] Mount and install from `.dmg` on macOS Sonoma (Intel or Apple Silicon as appropriate).
- [ ] Application launches without crash after install.
- [ ] Application appears in `/Applications` and launches from Spotlight.
- [ ] Uninstall by dragging to Trash leaves no persistent background processes.

### Windows

- [ ] Install from `.msi` or `.exe` on Windows 11.
- [ ] Application launches from the Start Menu shortcut.
- [ ] Uninstall via Add/Remove Programs leaves no leftover files in Program Files.

### Linux

- [ ] `.AppImage` is executable and launches on Ubuntu 22.04 LTS without additional dependencies.
- [ ] (If `.deb` is provided) `dpkg -i` installs cleanly; application launches from the application menu.

---

## Functionality Smoke Tests

Perform a quick pass of the core flows on the release build, not the dev build:

- [ ] Add a connection with a valid token — succeeds.
- [ ] List bases for the connection — list is shown.
- [ ] Run a backup of a small test base — completes without error, package written.
- [ ] Open the written backup package — manifest and schema summary shown correctly.
- [ ] Run a dry-run restore from the package — report produced, no writes made.
- [ ] Settings view loads and shows correct version and app data path.
- [ ] Log file is written to the expected location after an operation.

---

## Regression Pass

- [ ] All automated tests pass on the release commit (CI green).
- [ ] All items in `regression-testing.md` manual regression list have been checked.
- [ ] No known high-priority regressions are open against this release.

---

## Accessibility Pass

- [ ] Keyboard navigation works through all primary flows (no mouse required).
- [ ] Focus ring is visible on all interactive elements.
- [ ] Color contrast meets WCAG AA on primary UI elements.
- [ ] See `accessibility-qa.md` for the full checklist — confirm it has been completed.

---

## Cross-Platform Pass

- [ ] The application has been smoke-tested on at least one macOS, one Windows, and one Linux machine.
- [ ] No platform-specific layout breaks or missing fonts observed.
- [ ] See `cross-platform-qa.md` for the full checklist — confirm it has been completed.

---

## Alpha Release Gate (v0.1.0-alpha specific)

Complete this section for alpha releases only. These items are prerequisites for any alpha distribution.

### Release Workflow Checks

- [ ] Release workflow triggered via `workflow_dispatch` only — no push or tag auto-trigger fired.
- [ ] All three matrix legs (macOS, Linux, Windows) passed quality gates before the build step.
- [ ] Artifacts are uploaded as workflow run artifacts only — no GitHub release was created automatically.
- [ ] Artifact names match `airbridge-<version>-<OS>` convention per `docs/release/artifact-naming.md`.

### Safety Gates

- [ ] `run_restore_execution` returns `readyButDisabled` — confirmed no Airtable writes occur.
- [ ] `run_restore_execution` response does not contain the token value.
- [ ] `run_restore_execution` response does not contain the full package path.
- [ ] `run_backup_job` requires the exact confirmation text `CREATE BACKUP` — partial or mis-cased text is rejected.
- [ ] `inspect_backup_package` is read-only — no files extracted, no token, no API calls.
- [ ] `create_restore_dry_run_plan` is read-only — no token, no API calls, no writes.
- [ ] `create_restore_schema_creation_plan` is read-only — no token, no API calls, no tables or fields created.
- [ ] `create_restore_record_import_plan` is read-only — no token, no API calls, no records created.
- [ ] `list_job_history` response contains no token values, no full paths, no record payload content.

### Installer Smoke Tests (per platform)

- [ ] macOS installer launches and the application opens without a crash.
- [ ] Linux AppImage is executable and opens without additional dependencies.
- [ ] Windows installer completes and the application launches from the Start Menu.
- [ ] No full path, token value, or record payload content is visible in any installer dialog or first-run screen.

### Known Limitations Documented

- [ ] Restore write engine disabled — documented in `docs/release/known-limitations.md`.
- [ ] Token persistence not implemented — documented.
- [ ] Attachment files not downloaded — documented.
- [ ] Job history does not persist between sessions — documented.
- [ ] Release notes reviewed against `docs/release/v0.1.0-alpha-release-notes-draft.md`.
- [ ] Known limitations document is accurate for the release commit.

### Test Counts Verified

- [ ] Rust unit tests: 680 pass (or more — confirm actual count from `cargo test` output).
- [ ] Rust integration tests: 3 pass (or more).
- [ ] Frontend tests: 444 pass (or more — confirm actual count from `vitest run` output).

### Prohibited Terms

- [ ] No references to private development workflow tools in any public file.
- [ ] No absolute user-local paths embedded in docs, source, or fixtures.
- [ ] No token-like values in docs or fixtures.
- [ ] No generated `.airbridge` binary packages committed.
- [ ] Docs do not imply restore write execution is enabled.
- [ ] Docs do not imply credential storage is implemented.
- [ ] No token or full path leakage in any UI visible in installer smoke tests.

---

## Documentation and Changelog

- [ ] `CHANGELOG.md` has an entry for this release with a meaningful summary of changes.
- [ ] The release date in `CHANGELOG.md` matches the actual publish date.
- [ ] `README.md` version references (if any) have been updated.
- [ ] Any new or changed features have corresponding documentation updates.

---

## Final Sign-Off

| Area | Checked by | Date |
|------|-----------|------|
| Build artifacts | | |
| Version strings | | |
| macOS signing | | |
| Windows signing | | |
| Installer smoke | | |
| Functionality smoke | | |
| Regression pass | | |
| Accessibility | | |
| Cross-platform | | |
| Documentation | | |

**Release approved:** Yes / No

**Notes:**
