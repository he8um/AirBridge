# Known Limitations

This document lists known limitations of the current AirBridge release. Each limitation is described with its scope, the reason it exists, and the expected resolution path.

---

## Restore Write Engine Not Yet Enabled

**Scope:** Restore functionality  
**Status:** Safety gate, schema creation planning, and record import planning complete; write engine disabled

The restore execution safety gate validates all preconditions (package inspection, dry-run plan, target mode, token, confirmation text) and returns `readyButDisabled`. No Airtable data is modified.

The **schema creation planner** produces a full ordered plan (table steps, field steps, deferred linked fields, dependency graph) without making any Airtable API calls. No Airtable base, table, or field is created.

The **record import planner** produces a complete import batch plan (per-table batch counts at batch size 10, field import policies, linked record second-pass update plans, attachment policies, old-to-new record ID mapping strategy, checkpoint plans, retry policy) without making any Airtable API calls. No Airtable records are created.

**Old-to-new record ID mapping** — The import plan describes the `MapSourceRecordIdToCreatedRecordId` strategy. Actual new record IDs are only available after first-pass record creation, which requires the write engine. ID mapping cannot be resolved at planning time.

**Linked record second-pass updates** — The import plan identifies which fields require a second update pass (after all records are created and ID mapping is resolved). The second pass itself requires the write engine and is not executed in this version.

Restore write execution will be enabled in a future release once the write engine, linked record remapping, post-restore verification, and schema/record execution flows are complete and tested.

---

## No Credential Storage

**Scope:** Token handling  
**Status:** Not implemented

Tokens must be entered for each operation. AirBridge does not persist tokens between sessions. There is no OS credential store integration in v0.1.0-alpha.

Token persistence via the OS keychain is planned for a future release.

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

## No Job History

**Scope:** Backup/restore operations  
**Status:** Not implemented

Completed backup jobs are not recorded in a persistent history. The backup report is written into the `.airbridge` package at completion, but there is no in-app job log that persists between sessions beyond the current session's log entries.

A persistent job history registry is planned for a future release.

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
