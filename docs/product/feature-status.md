# Feature Status Matrix

**Version:** 0.1.0-alpha  
**Updated:** 2026-06-15

This matrix describes the current status of each feature area. Status values:

- **Complete** — implemented, tested, and available in the current build.
- **Partial** — core functionality implemented; some capabilities deferred.
- **Disabled** — contract and gate implemented; execution blocked.
- **Planned** — not yet implemented.

---

| Feature | Status | User-Facing Availability | Safety Notes | Next Work |
|---------|--------|--------------------------|-------------|-----------|
| Connection check | Complete | Available — Connections page | Token validated locally before API call; never stored; never in result | None |
| Base catalog / schema read | Complete | Available — Connections, Backups pages | Token consumed per call; never stored | None |
| Backup plan | Complete | Available — Backups page | No token required; no writes; read-only planning | None |
| Records export plan | Complete | Available — Backups page | No token required; no writes; read-only planning | None |
| Backup execution | Complete | Available — Backups page | Requires file picker + `CREATE BACKUP` confirmation; token consumed; path validated; filename-only in result | None |
| Backup progress / cancellation | Partial | Available — polling only | No destructive behavior on cancel | Streaming progress deferred |
| Package inspection | Complete | Available — Restore page | Read-only; no token; full path never in result | None |
| Restore dry-run | Complete | Available — Restore page | No token; no API calls; no writes; full path never in result | None |
| Restore execution gate | Complete | Available — Restore page | All preconditions validated; token checked for presence only; write engine disabled; `noChangesMade` always true | Enable write engine |
| Restore schema creation plan | Complete | Available — Restore page | No token; no Airtable calls; no writes; table-first ordering; field classification; dependency graph; `noChangesMade` always true | None |
| Restore record import plan | Complete | Available — Restore page | No token; no Airtable calls; no writes; batch planning (size 10); field import policies; linked record second-pass; attachment metadata; checkpoint; retry policy; `noChangesMade` always true | None |
| Restore sandbox verification (Gate 1) | Partial | Available — Restore page | No Airtable API calls; no token; no writes; CHK-01 through CHK-09 run locally; CHK-10 (live metadata) always skipped; `noChangesMade` always true; `writesEnabled` always false | Implement live CHK-10 metadata check |
| Restore confirmation gate (Gate 2) | Partial | Available — Restore page | No Airtable API calls; no token; no writes; exact text match validated; sandbox prerequisite checked; `noChangesMade` always true; `writesEnabled` always false; `Confirmed` does not enable writes | Wire confirmed result to future live write path |
| Restore target empty verification (Gate 3) | Partial | Available — Restore page | No Airtable write API calls; no token; no writes; target mode allowlisted; table and record counts checked; `noChangesMade` always true; `writesEnabled` always false; `Verified` does not enable writes | Implement live metadata check for count retrieval |
| Restore destructive operation policy (Gate 4) | Partial | Available — Restore page | No Airtable API calls; no token; no writes; delete/update/overwrite/attachment-upload operations blocked; create-only operations validated; `noChangesMade` always true; `writesEnabled` always false; `Compliant` does not enable writes | Wire to actual planned operations from schema and record write foundations |
| Restore attachment upload policy (Gate 5) | Partial | Available — Restore page | No Airtable API calls; no token; no full attachment URL; attachment file bytes never uploaded; `UploadRequested` fields blocked; `DownloadRequested`/`Unknown` fields produce warning; `noChangesMade` always true; `writesEnabled` always false; `Compliant` does not enable writes | Wire to actual declared attachment fields from dry-run plan |
| Restore schema record order policy (Gate 6) | Partial | Available — Restore page | No Airtable API calls; no token; no record payload; missing/blocked schema phase blocked; records-before-schema ordering blocked; linked/attachment-before-records ordering blocked; unplanned/undeclared conditions produce warning; `noChangesMade` always true; `writesEnabled` always false; `Compliant` does not enable writes | Wire to actual planned phase declarations from write engine |
| Restore sandbox write testing policy (Gate 7) | Partial | Available — Restore page | No Airtable API calls; no token; no record payload; production/unknown targets blocked; missing evidence blocked; incomplete evidence produces warning; filename must be basename only; `noChangesMade` always true; `writesEnabled` always false; `Compliant` does not enable writes | Provide real sandbox test evidence before live write testing |
| Restore live write confirmation policy (Gate 8) | Partial | Available — Restore page | No Airtable API calls; no token; no record payload; exact case-sensitive phrase match required; prior blocked gates prevent confirmation; confirmed result does not enable writes; `noChangesMade` always true; `writesEnabled` always false; `Confirmed` does not enable writes | Wire confirmed result to future live write path |
| Restore rate-limit and backoff policy (Gate 9) | Partial | Available — Restore page | No Airtable API calls; no token; no record payload; plan must declare max RPS ≤ 5, batch size ≤ 10, 429 handling, bounded retries, backoff strategy, stop condition; partial/none/unknown checkpoint compatibility produces warning; no plan causes immediate blocked; `noChangesMade` always true; `writesEnabled` always false; `Compliant` does not enable writes | Wire declared plan to actual restore write path |
| Restore checkpoint durability policy (Gate 10) | Partial | Available — Restore page | No Airtable API calls; no token; no record payload; plan must declare table/batch checkpoints, phase markers, resume-safe stop condition; linked updates require ID mapping checkpoint; memory/unknown backend produces warning; no plan causes immediate blocked; `noChangesMade` always true; `writesEnabled` always false; `Compliant` does not enable writes | Wire declared plan to actual restore write path |
| Restore final validation policy (Gate 11) | Partial | Available — Restore page | No Airtable API calls; no token; no record payload; plan must declare schema/table/record count validation, ID mapping validation, linked record validation, attachment metadata validation, manifest checksum validation, and success-blocked-without-validation; metadata-only attachment validation produces warning; no plan causes immediate blocked; `noChangesMade` always true; `writesEnabled` always false; `Compliant` does not enable writes or introduce restore success state | Wire declared plan to actual restore write path |
| Restore write phase ordering policy (Gate 12) | Partial | Available — Restore page | No Airtable API calls; no token; no record payload; phases must be declared in canonical order (preflight → schema_create → schema_verify → record_create → record_verify → linked_record_update → linked_record_verify → attachment_metadata_verify → final_validation); prerequisite phase violations and unsafe transitions blocked; attachment upload/binary language blocked; attachment_metadata_verify skip produces warning; no phase list causes immediate blocked; `noChangesMade` always true; `writesEnabled` always false; `Compliant` does not enable writes or introduce restore success state | Wire declared phase list to actual restore write path |
| Restore failure modes policy (Gate 13) | Partial | Available — Restore page | No Airtable API calls; no token; no record payload; all 10 required failure modes must be declared with explicit stop behaviors; all four stop behavior variants unconditionally stop writes; destructive rollback blocked; partial failure labeled as success blocked; modes without diagnostic context produce per-mode warnings; no handling plans causes immediate blocked; `noChangesMade` always true; `writesEnabled` always false; `Compliant` does not enable writes or introduce restore success state | Wire declared handling plans to actual restore write path |
| Restore rollback limitation policy (Gate 14) | Partial | Available — Restore page | No Airtable API calls; no token; no record payload; automatic destructive rollback blocked; automatic delete cleanup blocked; automatic update/revert cleanup blocked; partial restore labeled as success blocked; manual cleanup requires separate explicit future action; missing recovery guidance produces warning; missing user-visible notice produces warning; notice without limitation details produces warning; no plan causes immediate blocked; no automatic rollback or cleanup path exists in implementation; `noChangesMade` always true; `writesEnabled` always false; `Compliant` does not enable writes or introduce restore success state | Wire declared rollback limitation plan to actual restore write path; implement cleanup UI as separate explicit future flow |
| Restore write engine | Disabled (skeleton) | Skeleton preview available — Restore page | No Airtable writes; no token required; all phases disabled; `noChangesMade` always true | Satisfy all gates in live restore write safety contract before enabling |
| Schema write engine foundation | Disabled | Request plan builder available — internal only | No Airtable writes; no token required; request builders exist; write gate always disabled; `noChangesMade` always true; `networkWritesAttempted` always false | Enable live schema writes when write engine is ready |
| Record write engine foundation | Disabled | Request plan builder available — internal only | No Airtable writes; no token required; request builders exist; write gate always disabled; `noChangesMade` always true; `networkWritesAttempted` always false; no raw record payloads; old-to-new ID mapping deferred to execution | Enable live record writes when write engine is ready |
| Credential / token storage | Partial | Optional — Settings page | Token stored in OS keychain only; never in files, history, SQLite, or localStorage; keychain unavailable state handled safely | Wire saved token to connection check |
| Local job history | Complete | Available — Reports page | Memory-only; no tokens; no full paths; no record payloads; summary-level only | SQLite persistence deferred |
| Streaming progress events | Planned | Not available | — | Tauri event stream |
| Attachment file download | Planned | Not available — metadata only | Attachment URLs captured at backup time only; may expire | File download and storage engine |
| Attachment URL preservation | Partial | Metadata and URL in package | URL valid at backup time only | Link freshness / re-fetch strategy |
| Formula / rollup field restore | Not applicable | — | These fields cannot be created via the Airtable API; listed in compatibility report as unsupported | API limitation; manual recreation required |
| Automation / interface backup | Not applicable | — | Not part of v0.1 scope | Out of scope |
| Restore into non-empty base | Blocked by design | Not available | Prevents destructive overwrites | Not planned for v0.1 |
| Merge / overwrite restore | Blocked by design | Not available | Prevents data loss | Not planned for v0.1 |
| System field restore (created time, etc.) | Not applicable | — | Airtable API does not support writing system fields | API limitation |
| Cross-platform release builds | Planned | Not yet verified | — | CI pipeline verification |

---

## Detailed Notes

### Backup Execution

The backup execution contract (`run_backup_job`) is the only command that writes a file. It requires:

1. Explicit output path selection via the OS file picker.
2. Exact confirmation text `CREATE BACKUP` from the user.
3. Output path validation (extension, parent directory, no traversal, not an existing directory).

The token is consumed to build the Airtable HTTP client and discarded. It never appears in the result. The full output path never appears in the result — only the filename.

### Restore Execution Gate

The restore execution gate (`run_restore_execution`) validates seven preconditions in order:

1. Package inspected (non-empty filename).
2. Package validation status is `valid` or `warning`.
3. Dry-run plan exists (non-empty status).
4. Dry-run status is `ready` or `readyWithWarnings`.
5. Package path non-empty (target mode implicitly set).
6. Token non-empty.
7. Confirmation text equals `RESTORE BACKUP` exactly.

When all seven pass, the command returns `readyButDisabled` with `restoreWriteEngineNotEnabled`. No Airtable API call is made. `noChangesMade` is always `true`.

### Package Inspection

Package inspection (`inspect_backup_package`) is fully read-only. No token is required. The package is opened in memory, validated, and closed. No entries are extracted to disk. The result contains only the filename — the full path is stripped using `Path::file_name()` before the result is returned.

### Restore Dry-Run

Restore dry-run planning (`create_restore_dry_run_plan`) reads the inspected package, applies the compatibility engine, and returns a plan preview. No Airtable API calls are made. No token is required. The full path is not included in the result.

### Schema Write Engine Foundation

The schema write engine foundation (`preview_schema_write_request_plan`) builds a sequenced list of schema write operations from a schema plan summary and passes them through the dry-run executor skeleton. No token is accepted or returned. No Airtable API calls are made. No Airtable base, table, or field is created.

The request plan builder produces operations in four ordered phases: `CreateTable`, `CreateField` (directly creatable fields only), `DeferLinkedField`, and `ManualAction`. The write gate always returns `Disabled/DisabledByProductPolicy` — there is no enabled branch.

`noChangesMade` is always `true`. `networkWritesAttempted` is always `false`. The result status is never `"succeeded"`.

### Record Write Engine Foundation

The record write engine foundation (`preview_record_write_request_plan`) builds a sequenced list of record write operations from a record import plan summary and passes them through the dry-run executor skeleton. No token is accepted or returned. No Airtable API calls are made. No Airtable records are created, updated, or deleted.

The request plan builder produces operations in five ordered phases: `CreateRecordBatch` (first-pass batch creation), `UpdateLinkedRecordBatch` (second-pass linked field updates), `Checkpoint` (per-table), `PreserveMetadataOnlyAttachment`, and `SkipComputedField`. The write gate always returns `Disabled/DisabledByProductPolicy` — there is no enabled branch.

Old-to-new record ID mapping is `UnavailableUntilExecution` — linked record update operations note this explicitly. No actual record IDs are present in the plan.

`noChangesMade` is always `true`. `networkWritesAttempted` is always `false`. The result contains no raw record payloads. The result status is never `"succeeded"`.

---

## Related Documents

- [Tauri Command Inventory](../architecture/tauri-command-inventory.md)
- [Known Limitations](../release/known-limitations.md)
- [v0.1.0-alpha Readiness](../release/v0.1.0-alpha-readiness.md)
- [Restore Execution Command Contract](../architecture/restore-execution-command-contract.md)
- [Safe Backup Command Contract](../architecture/safe-backup-command-contract.md)
