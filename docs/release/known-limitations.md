# Known Limitations

This document lists known limitations of the current AirBridge release. Each limitation is described with its scope, the reason it exists, and the expected resolution path.

---

## Restore Write Engine Not Yet Enabled

**Scope:** Restore functionality  
**Status:** Safety gate, schema creation planning, record import planning, and write engine skeleton complete; write engine execution disabled

The restore execution safety gate validates all preconditions (package inspection, dry-run plan, target mode, token, confirmation text) and returns `readyButDisabled`. No Airtable data is modified.

The **schema creation planner** produces a full ordered plan (table steps, field steps, deferred linked fields, dependency graph) without making any Airtable API calls. No Airtable base, table, or field is created.

The **record import planner** produces a complete import batch plan (per-table batch counts at batch size 10, field import policies, linked record second-pass update plans, attachment policies, old-to-new record ID mapping strategy, checkpoint plans, retry policy) without making any Airtable API calls. No Airtable records are created.

The **write engine skeleton** (`preview_restore_write_engine`) produces a six-phase preview of what the write pipeline would execute — schema creation, record import, linked record updates, attachment handling, and validation — using counts from the existing planning outputs. No token is required. No Airtable calls are made. All phases are disabled. `noChangesMade` is always true.

The **schema write engine foundation** (`preview_schema_write_request_plan`) builds a sequenced list of schema write operations from a schema plan summary and passes them through the dry-run executor. Operations are produced in four ordered phases (CreateTable → CreateField → DeferLinkedField → ManualAction). The request plan builder is complete; however, live execution remains disabled — the write gate always returns `Disabled/DisabledByProductPolicy`. No token is accepted. No Airtable API calls are made. No Airtable base, table, or field is created. `noChangesMade` is always `true`. `networkWritesAttempted` is always `false`.

The **record write engine foundation** (`preview_record_write_request_plan`) builds a sequenced list of record write operations from a record import plan summary and passes them through the dry-run executor. Operations are produced in five ordered phases (CreateRecordBatch → UpdateLinkedRecordBatch → Checkpoint → PreserveMetadataOnlyAttachment → SkipComputedField). The request plan builder is complete; however, live execution remains disabled — the write gate always returns `Disabled/DisabledByProductPolicy`. No token is accepted. No Airtable API calls are made. No records are created, updated, or deleted. `noChangesMade` is always `true`. `networkWritesAttempted` is always `false`. No raw record payloads are present in the result.

**Old-to-new record ID mapping** — The import plan describes the `MapSourceRecordIdToCreatedRecordId` strategy. Actual new record IDs are only available after first-pass record creation, which requires the write engine. ID mapping cannot be resolved at planning time.

**Linked record second-pass updates** — The import plan identifies which fields require a second update pass (after all records are created and ID mapping is resolved). The second pass itself requires the write engine and is not executed in this version.

Restore write execution will be enabled in a future release once the write engine, linked record remapping, post-restore verification, and schema/record execution flows are complete and tested.

---

## Credential Storage — Saved Token Not Used Automatically

**Scope:** Token handling  
**Status:** OS keychain storage implemented; auto-fill not yet wired

Users can optionally save their Airtable Personal Access Token to the OS keychain via Settings → Saved Credentials. The token is stored in the OS keychain only — never in files, SQLite, `localStorage`, history items, or logs. Saving is never required.

In this version, the saved token is not automatically retrieved for connection checks or backup operations. Users who save a token will still need to paste it into the relevant field when initiating an operation.

Automatic token retrieval for connection checks and backup operations is deferred to a future release.

If the OS keychain is not available (e.g., headless Linux without a secret service daemon), the Saved Credentials panel shows a notice and the save/remove controls are hidden.

---

## Attachment Files Not Downloaded or Uploaded

**Scope:** Backup content and restore  
**Status:** Metadata only

Attachment metadata (filename, MIME type, size, and the attachment URL at time of backup) is included in the backup package under `attachments/metadata.jsonl`. Attachment file bytes are not downloaded or stored during backup.

During restore planning, all attachment fields receive the `MetadataOnly` policy — the record import planner will not schedule attachment uploads. Attachments must be manually re-attached to restored records after a restore completes.

Attachment download URLs returned by the Airtable API may expire. The backed-up URL captures the state at the time of backup only.

Full attachment file download and re-upload is deferred to a future release.

---

## No Streaming Progress

**Scope:** Backup execution progress  
**Status:** Polling only

Backup progress is polled from the frontend. Events are not streamed. Progress updates may appear coarser-grained than expected for large bases.

Streaming progress via Tauri events is planned for a future release.

---

## No Automatic Retry on Rate Limit

**Scope:** Backup execution  
**Status:** Manual only

If the Airtable API returns a 429 rate-limit response during backup, the operation surfaces an error. Automatic retry with back-off is not implemented.

Users who hit the rate limit should wait and retry the backup manually.

Automatic retry with exponential back-off is planned for a future release.

---

## Job History Does Not Persist Between Sessions

**Scope:** Backup/restore operations  
**Status:** Memory-only

AirBridge maintains a local activity history on the Reports page showing recent connection checks, backup and restore operations. History items contain only safe summaries: no tokens, no full paths, no record payloads, no attachment URLs.

In v0.1.0-alpha this history is stored in memory only. It does not persist between application restarts — history is cleared each time the application is launched.

SQLite-backed persistent job history is planned for a future release.

---

## Restore Into Non-Empty Bases Not Supported

**Scope:** Restore targeting  
**Status:** Blocked by design

Restore targets must be a new base or an empty existing base. Restoring into a non-empty base is not supported and will not be attempted. This restriction prevents destructive overwrites, unexpected duplicates, and unsafe merge behavior.

Non-destructive merge restore is not planned for v0.1.

---

## Computed and Advanced Field Types

**Scope:** Field backup and restore  
**Status:** Metadata-only for some types

Computed field types (formula, rollup, lookup, count) are captured in the schema backup but cannot be restored via the Airtable API. These fields are classified in the compatibility report and are listed as `MetadataOnly` or `UnsupportedForRestore`. They must be recreated manually after restore.

Automations, interfaces, and sharing settings are not captured in the backup.

---

## Write Permission Not Pre-Verified for Restore Targets

**Scope:** Restore execution (when enabled)  
**Status:** Verified at write time

When the restore write engine is enabled, write permissions to the target base are verified at the time of the first write operation, not before execution begins. A permission error mid-restore will surface in the restore report.

Pre-flight permission verification is planned for the restore write engine release.

---

## `greet` Scaffold Command Present

**Scope:** Tauri command registration  
**Status:** Harmless; unused in the UI

A scaffold command named `greet` from the Tauri project template remains registered. It is not accessible from the AirBridge UI and poses no security risk. It will be removed before a stable release.

---

## macOS Notarization and Code Signing Not Verified in CI

**Scope:** Release builds  
**Status:** Not yet tested end-to-end

Cross-platform release build artifacts (`.dmg`, `.msi`, `.AppImage`) and the macOS notarization pipeline have not been verified in a full CI run. Release build QA is required before any public distribution.
