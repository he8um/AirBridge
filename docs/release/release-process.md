# Release Process

## Overview

AirBridge releases are prepared manually. No release is published automatically — every release requires a human to review artifacts, approve quality gates, and publish the GitHub release.

The current release workflow (`release.yml`) is triggered exclusively via `workflow_dispatch`. It builds platform artifacts and uploads them as workflow artifacts only. A GitHub release must be created manually after reviewing the artifacts.

---

## Pre-Release Checks

Before triggering the release workflow, verify:

1. **All CI checks pass on `main`.** The CI workflow must be green before a release build is started.
2. **Version is consistent** across:
   - `apps/desktop/package.json` → `"version"`
   - `apps/desktop/src-tauri/Cargo.toml` → `version`
   - `apps/desktop/src-tauri/tauri.conf.json` → `"version"`
3. **Alpha readiness document is current.** See `docs/release/v0.1.0-alpha-readiness.md`.
4. **Known limitations are documented.** See `docs/release/known-limitations.md`.
5. **Release notes are reviewed.** See `docs/release/v0.1.0-alpha-release-notes.md`.
   Use `docs/release/v0.1.0-alpha-publishing-checklist.md` for the complete pre-publish checklist.
6. **No prohibited content in repository.** Run the prohibited-terms scan before tagging.
7. **Restore write execution remains disabled.** No Airtable writes should occur from restore operations.
8. **Token persistence is not implemented.** Tokens must be entered per-operation.

Run the full local quality gate to confirm:

```bash
npm run release:check
```

This is equivalent to `npm --prefix apps/desktop run check` and runs typecheck, lint, format check, frontend tests, Rust checks, and a build verification.

---

## Triggering the Release Workflow

The release workflow is triggered manually:

1. Go to **Actions → Release** on GitHub.
2. Click **Run workflow**.
3. Set `version` to the release tag (e.g. `v0.1.0-alpha`).
4. Click **Run workflow**.

The workflow will:

- Run on macOS, Linux, and Windows.
- Execute all quality gates (`npm --prefix apps/desktop run check`).
- Build the Tauri application for each platform.
- Upload artifacts named `airbridge-<version>-<OS>` as workflow run artifacts.

**The workflow does not create a GitHub release automatically.** It produces workflow run artifacts only. A GitHub release must be created manually after reviewing the artifacts.

---

## Reviewing Artifacts

After the workflow completes:

1. Open the workflow run in the **Actions** tab.
2. Download the artifact for each platform.
3. Verify artifact names match the convention in `artifact-naming.md`.
4. Smoke-test the installer on each target platform (see `docs/qa/release-qa-checklist.md`).
5. Verify no token, full path, or record payload content appears in any installer UI.

---

## Creating the GitHub Release Manually

After artifacts are reviewed and approved:

1. Go to **Releases → Draft a new release**.
2. Set the tag to the release version (e.g. `v0.1.0-alpha`). Create the tag at this point — do not tag the repository locally before this step.
3. Set the title to `AirBridge v0.1.0-alpha`.
4. Copy the release body from `docs/release/github-release-draft-template.md`.
5. Upload the reviewed platform artifacts.
6. Mark as **pre-release** for alpha/beta releases.
7. Keep as **draft** until final review is complete.
8. Publish when ready.

---

## Cancellation and Rollback

- If a workflow run needs to be cancelled, cancel it in the Actions tab before artifacts are published.
- If a GitHub release was published in error, it can be converted back to draft or deleted from the GitHub Releases page.
- Local repository state is not affected by triggering or cancelling a workflow run.
- No local tagging or pushing is required to trigger the workflow — `workflow_dispatch` does not require a tag.

---

## Dependency Caching

Both CI and release workflows cache npm and Cargo dependencies:

- **npm** — `actions/setup-node` caches the npm module cache keyed to `apps/desktop/package-lock.json`. A cache hit skips most of the `npm ci` download time.
- **Cargo** — `Swatinem/rust-cache@v2` caches the Cargo registry and compiled dependency artifacts keyed per OS and `Cargo.lock`. A cache hit significantly reduces Rust compilation time.
- **`cargo fetch`** — runs before checks and builds with one automatic retry (`cargo fetch || sleep 10 && cargo fetch`). This reduces failures caused by transient crates.io registry or network resets. It does not mask real compile failures.

If a CI run fails on a dependency download step (e.g. a registry connection reset), re-run the job once before investigating the code.

---

## Current Limitations

- Code signing and notarization are not configured. macOS `.dmg` artifacts will not be notarized. Windows `.msi` artifacts will not be code-signed. Both are required before distributing to end users outside the development team.
- Checksums are not automatically generated and uploaded. They must be created manually from the downloaded artifacts (see `docs/release/checksums.md`).
- The release workflow does not run on self-hosted runners or ARM targets.
- Linux `.deb` packaging is not verified end-to-end.
