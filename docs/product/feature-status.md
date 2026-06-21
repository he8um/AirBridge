# Feature Status Matrix

**Version:** 0.1.0-alpha  
**Updated:** 2026-06-21 (6)

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
| Restore final validation enforcement policy (Gate 15) | Partial | Available — Restore page | No Airtable API calls; no token; no record payload; completion guard must declare all three invariants true; schema/record count/ID mapping/linked record/attachment/manifest validation states enforced; `Skipped`/`Partial`/`NotDeclared` states always block; `NotRequired` without reason blocks; metadata-only attachment produces warning; manifest check skipped when no manifest present; no plan or no guard causes immediate blocked; no result may be labeled complete without final validation passing; `noChangesMade` always true; `writesEnabled` always false; `Compliant` does not enable writes or introduce restore success state | Wire declared validation enforcement plan to actual restore write path |
| Restore sensitive data safety policy (Gate 16) | Partial | Available — Restore page | No Airtable API calls; no token; no full path; no package path; no record payload; no attachment URL; no raw HTTP data; 10 exposure surfaces must all have redaction coverage; 10 sensitive pattern classes enforced; unnamed redaction rules produce warning only; no plan causes immediate blocked; `noChangesMade` always true; `writesEnabled` always false; `Compliant` does not enable writes or introduce restore success state | Wire declared safety plan to actual restore write enforcement |
| Restore attachment phase disabled policy (Gate 17) | Partial | Available — Restore page | No Airtable API calls; no token; no attachment binary download, upload, URL fetch, or transfer; no attachment URL exposed; no field mutation; attachment handling metadata-only; binary handling, URL exposure, field mutation, and phase-required-for-completion all must be disabled; metadata verification disabled with reason produces warning only; no plan causes immediate blocked; `noChangesMade` always true; `writesEnabled` always false; `Compliant` does not enable writes, does not download attachments, and does not introduce restore success state | Wire declared attachment phase plan to actual restore write enforcement; implement binary attachment restore as a separate future feature |
| Restore live write readiness policy (Gate 18) | Partial | Available — Restore page | Advisory only; no Airtable API calls; no token; aggregates all 17 required safety gates; `Ready` does NOT enable writes and does NOT introduce a restore success state; `Ready` badge explicitly labeled advisory only; `liveExecutionAvailable: true` causes blocked; success-equivalent wording in gate notes causes blocked; sensitive data in gate notes causes blocked; warning gates produce warning only without blocking; no gates declared causes immediate blocked; `noChangesMade` always true; `writesEnabled` always false | This is the final pre-execution safety aggregator; wire to actual execution only after all gates are live and a full write-path review is complete |
| Restore write engine | Disabled (skeleton) | Skeleton preview available — Restore page | No Airtable writes; no token required; all phases disabled; `noChangesMade` always true | Satisfy all gates in live restore write safety contract before enabling |
| Schema write execution preview | Dry-run/Blocked only | Available — Restore page | No Airtable writes; no token; no base/table/field created; shows ordered dry-run steps (validate → create tables → direct fields → deferred fields → manual actions → post-check); all prerequisites must be satisfied for `DryRunReady`; `DryRunReady` does NOT enable live writes; `writesEnabled` always false; `noChangesMade` always true; `networkWritesAttempted` always false; live schema writes remain disabled | Enable live schema write execution only after full write-path review and all gates satisfied |
| Record write execution preview | Dry-run/Blocked only | Available — Restore page | No Airtable writes; no token; no record created/updated/deleted; no raw field values; no raw HTTP; no attachment URL; shows ordered dry-run batches (first-pass create → second-pass linked-update); 13 prerequisites must be satisfied for `DryRunReady`; batch size enforced ≤ 10; `DryRunReady` does NOT enable live record writes; `writesEnabled` always false; `noChangesMade` always true; `networkWritesAttempted` always false; live record writes remain disabled | Enable live record write execution only after full write-path review and all gates satisfied |
| Live schema write test contract | Disabled | Internal only — no UI surface, no Tauri command, no token, no network call; contract-only readiness layer; `eligibleButNotExecuted` does NOT perform any live call; `contractOnly` always true; live schema write integration test remains separate pending work |
| Sandbox schema write harness (integration test) | Test-only, ignored by default | Internal only — no UI surface, no Tauri command; `#[ignore]` by default; requires `AIRBRIDGE_ENABLE_LIVE_SCHEMA_WRITE_TEST=true`, `AIRBRIDGE_SANDBOX_AIRTABLE_TOKEN`, `AIRBRIDGE_SANDBOX_TARGET_BASE_ID`; schema-only (`createTable` + primary field); no records, linked updates, attachment, or final validation; `evaluate_write_gate()` remains Disabled before and after; test may leave a sandbox-only table — must only run against disposable sandbox base; no cleanup path | Record writes, linked record updates, final validation reads, attachment handling, and live E2E restore remain pending |
| Live record write test contract | Disabled | Internal only — no UI surface, no Tauri command, no token, no network call; contract-only readiness layer; `eligibleButNotExecuted` does NOT perform any live call; `contractOnly` always true; 10 prerequisites (LRWTC-PRE-01 through LRWTC-PRE-10); live record write integration test remains separate pending work |
| Sandbox adapter chain runner | Disabled | Internal only — no UI surface, no Tauri command | No Airtable calls; no token; no network reads or writes; no checkpoint files written; mock/no-op adapters only; write gate always disabled; `noChangesMade` always true; `airtableClientCalled` always false; `mockRunNotExecuted` does NOT enable any live execution path | Live end-to-end sandbox restore execution remains separate pending work |
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

### Mapping and Checkpoint Execution Preview Foundation

The mapping and checkpoint execution preview (`preview_mapping_checkpoint_execution_gate`) converts declared safety prerequisites and batch counts into a deterministic ordered step list showing checkpoint boundaries across the restore pipeline. No token is accepted or returned. No Airtable API calls are made. No checkpoint files are written to disk.

The step builder produces steps in six ordered phases: `MCEP-CHK-SCHEMA` (schema checkpoint boundary), `MCEP-CHK-PRE-REC` (pre-record-create boundary), `MCEP-MAP-REC-B{n}` (per-first-pass-batch ID mapping capture, one per batch), `MCEP-CHK-PRE-LINK` (pre-linked-update boundary, only when first-pass batches > 0), `MCEP-CHK-LINK-B{n}` (per-second-pass-batch linked-update checkpoint, one per batch), and `MCEP-CHK-PRE-FV` (pre-final-validation boundary).

The preview checks eight prerequisites (MCEP-PRE-01 through MCEP-PRE-08): write gate disabled, record write preview `DryRunReady`, checkpoint durability safe, failure modes safe, rollback limitation safe, final validation enforcement present, sensitive data safe, and live write readiness satisfied. If any prerequisite is missing, the result is `Blocked`. If all prerequisites are satisfied, the result is `DryRunReady` with `mode = DryRunOnly`.

`DryRunReady` does NOT enable live mapping capture, does NOT persist checkpoint files, does NOT start any restore execution. `writesEnabled` is always `false`, `noChangesMade` is always `true`, `networkWritesAttempted` is always `false`. The result status is never `"succeeded"`. No record IDs, field values, attachment URLs, or raw HTTP data appear in any result field.

### Linked Second-Pass Execution Preview Foundation

The linked second-pass execution preview (`preview_linked_second_pass_execution_gate`) converts declared safety prerequisites and per-field summaries into a deterministic batch list showing which linked fields require second-pass updates, how many batches they require, and how many unresolved links exist. No token is accepted or returned. No Airtable API calls are made. No checkpoint files are written.

The batch builder groups records per linked-field into batches of at most 10, ordered by field and batch index. Each batch carries only a table label, field label, update count, mapping coverage count, and unresolved-link count — no raw record IDs, field values, or HTTP data.

The preview checks eight prerequisites (LSEP-PRE-01 through LSEP-PRE-08): write gate disabled, record write preview `DryRunReady`, mapping/checkpoint preview `DryRunReady`, write phase ordering safe, checkpoint durability safe, sensitive data safe, final validation enforcement present, and live write readiness satisfied. If any prerequisite is missing, the result is `Blocked`. If all prerequisites are satisfied, the result is `DryRunReady` with `mode = DryRunOnly`.

`DryRunReady` does NOT enable live linked record updates, does NOT persist checkpoint files, does NOT call any Airtable endpoint, and does NOT start any restore execution. Unresolved links produce a warning count in the summary — they do not cause `Blocked`. `writesEnabled` is always `false`, `noChangesMade` is always `true`, `networkWritesAttempted` is always `false`. The result status is never `"succeeded"`. Live checkpoint persistence, final validation execution, and end-to-end restore execution remain pending.

### Final Validation Execution Preview Foundation

The final validation execution preview (`preview_final_validation_execution_gate`) provides a deterministic dry-run preview of the eight ordered final validation checks that would run after all write phases complete. No token is accepted or returned. No Airtable API calls are made. No checkpoint files are written. No record IDs appear in any result field.

The preview checks ten prerequisites (FVEP-PRE-01 through FVEP-PRE-10): write gate disabled, schema write preview `DryRunReady`, record write preview `DryRunReady`, mapping/checkpoint preview `DryRunReady`, linked second-pass preview `DryRunReady`, final validation policy safe, final validation enforcement policy safe, sensitive data safe, attachment phase disabled safe, and live write readiness satisfied. If any prerequisite is missing, the result is `Blocked`. If all prerequisites are satisfied, the result is `DryRunReady` with `mode = DryRunOnly`.

When `DryRunReady`, the result contains eight ordered checks: `FVEP-CHK-SCHEMA` (table count), `FVEP-CHK-FIELDS` (field count), `FVEP-CHK-RECORDS` (record count), `FVEP-CHK-MAPPING` (ID mapping coverage), `FVEP-CHK-LINKED` (linked record coverage), `FVEP-CHK-ATTACH` (attachment metadata only), `FVEP-CHK-MANIFEST` (manifest/checksum reference — skipped if no manifest), and `FVEP-CHK-GUARD` (final completion guard). Each check carries only safe counts, labels, and notes — no raw IDs, field values, or HTTP data.

`DryRunReady` does NOT enable live final validation execution, does NOT persist checkpoint files, does NOT call any Airtable endpoint, and does NOT start any restore execution. `writesEnabled` is always `false`, `noChangesMade` is always `true`, `networkWritesAttempted` is always `false`. The result status is never `"succeeded"`. Live final validation execution and end-to-end restore execution remain pending.

### Restore Checkpoint Metadata Store

The checkpoint metadata store (`store_restore_checkpoint_metadata`) writes a sanitized JSON checkpoint manifest to an app-controlled local directory (`<os-temp>/airbridge-checkpoints/`). No token, full filesystem path, record payload, old or new record IDs, raw HTTP body, or attachment URL is accepted, stored, or returned. The stored file explicitly declares `restoreExecutionNotTriggered: true` and `noSensitiveData: true`. The command returns only a safe filename (no directory component), boundary count, phase count, and item count to the UI.

The command checks five prerequisites (RCPS-PRE-01 through RCPS-PRE-05): write gate disabled, checkpoint durability policy safe, sensitive data safety policy satisfied, mapping/checkpoint preview `DryRunReady`, and final validation preview `DryRunReady`. If any prerequisite is missing, the result is `Blocked` and no file is written.

`Stored` does NOT enable live restore execution, does NOT introduce a restore success state, does NOT call any Airtable endpoint, and does NOT accept any user-supplied output path. `writesEnabled` is always `false`, `networkWritesAttempted` is always `false`. `noChangesMade` is `false` only when a local checkpoint metadata file was actually written; it is `true` when blocked. The result status is never `"succeeded"`. End-to-end restore execution remains pending.

### Schema Write Executor Foundation (internal)

The schema write executor foundation (`build_schema_write_executor_plan` in `restore/schema_write_executor.rs`) is an internal Rust module — it is not exposed as a Tauri command and has no UI surface. It builds an ordered step list of typed internal schema operation descriptors (tables first, then direct fields, then deferred linked fields, then manual actions) from an existing `SchemaWriteRequestPlan`. No Airtable API calls are made at any point.

The executor checks seven prerequisites (SWEX-PRE-01 through SWEX-PRE-07): write gate disabled, mode must be `sandboxOnly`, explicit internal write flag set, sandbox environment verified, target empty verified, live-write readiness satisfied, and request plan not blocked. If any prerequisite is missing, the result is `Blocked`. If all prerequisites are satisfied, the result is `NotExecuted` — because `evaluate_write_gate()` currently always returns `Disabled`. The `DryRunOnly` status is defined for future use but is currently unreachable.

The executor never makes network calls, never returns a token, full path, record payload, raw HTTP body, old/new record IDs, or attachment URL. `writesEnabled` is always `false`, `noChangesMade` is always `true`, `networkWritesAttempted` is always `false`. No production-target mode exists. No UI execute button is provided. Record writes, linked record updates, and live final validation reads remain pending.

### Record Write Executor Foundation (internal)

The record write executor foundation (`build_record_write_executor_plan` in `restore/record_write_executor.rs`) is an internal Rust module — it is not exposed as a Tauri command and has no UI surface. It builds an ordered batch plan of typed internal record operation descriptors (first-pass create batches before second-pass linked-update batches) from an existing `RecordWriteRequestPlan`. No Airtable API calls are made at any point.

The executor checks nine prerequisites (RWEX-PRE-01 through RWEX-PRE-09): write gate disabled, mode must be `sandboxOnly`, explicit internal write flag set, sandbox environment verified, target empty verified, schema write executor foundation safe, rate-limit/backoff policy compliant, checkpoint metadata store safe, and live-write readiness satisfied. If any prerequisite is missing, the result is `Blocked`. If all prerequisites are satisfied and no oversized batches are found, the result is `NotExecuted` — because `evaluate_write_gate()` currently always returns `Disabled`. The `DryRunOnly` status is defined for future use but is currently unreachable.

The executor never makes network calls, never returns a token, full path, record payload, raw HTTP body, old/new record IDs, or attachment URL. `writesEnabled` is always `false`, `noChangesMade` is always `true`, `networkWritesAttempted` is always `false`. Batch size is validated against a maximum of 10 records. No production-target mode exists. No UI execute button is provided. Linked record ID mapping, live final validation reads, and end-to-end restore execution remain pending.

### Linked Second-Pass Executor Foundation (internal)

The linked second-pass executor foundation (`build_linked_second_pass_executor_plan` in `restore/linked_second_pass_executor.rs`) is an internal Rust module — it is not exposed as a Tauri command and has no UI surface. It builds an ordered internal batch plan from per-field summaries (preserving field ordering, splitting into batches of at most 10), covering second-pass linked record update operations. No Airtable API calls are made at any point.

The executor checks ten prerequisites (LSEX-PRE-01 through LSEX-PRE-10): write gate disabled, mode must be `sandboxOnly`, explicit internal flag set, sandbox environment verified, target empty verified, record write executor foundation safe, linked second-pass preview `DryRunReady`, mapping/checkpoint preview `DryRunReady`, sensitive data safety satisfied, and live-write readiness satisfied. Batch sizes are validated against a maximum of 10. Unresolved optional links are warning-safe when the linked second-pass preview already returned `DryRunReady`. If any prerequisite is missing, the result is `Blocked`. If all prerequisites are satisfied, the result is `NotExecuted` — because `evaluate_write_gate()` currently always returns `Disabled`. The `DryRunOnly` status is defined for future use but is currently unreachable.

The executor never makes network calls, never returns a token, full path, record payload, raw HTTP body, old/new record IDs, or attachment URL. `writesEnabled` is always `false`, `noChangesMade` is always `true`, `networkWritesAttempted` is always `false`. No production-target mode exists. No UI execute button is provided. Live final validation reads and end-to-end restore execution remain pending.

### Final Validation Reader Foundation (internal)

The final validation reader foundation (`build_final_validation_reader_plan` in `restore/final_validation_reader.rs`) is an internal Rust module — it is not exposed as a Tauri command and has no UI surface. It builds an ordered internal check plan of eight typed validation read descriptors (schema/table count, field count, record count, ID mapping coverage, linked field coverage, attachment metadata-only, manifest/checksum, final completion guard) from declared safe counts in the request. No Airtable API calls are made at any point.

The reader checks eleven prerequisites (FVRD-PRE-01 through FVRD-PRE-11): validation read gate disabled, mode must be `sandboxOnly`, explicit internal validation read flag set, sandbox environment verified, schema write executor foundation safe, record write executor foundation safe, linked second-pass executor foundation safe, final validation execution preview `DryRunReady`, final validation enforcement policy safe, sensitive data safety satisfied, and attachment phase disabled policy safe. If any prerequisite is missing, the result is `Blocked`. If all prerequisites are satisfied, the result is `NotExecuted` — because the validation read gate (backed by `evaluate_write_gate()`) currently always returns `Disabled`. The `DryRunOnly` status is defined for future use but is currently unreachable.

The manifest/checksum check (`FVRD-CHK-MANIFEST`) is `Skipped` when no manifest is present. The attachment check (`FVRD-CHK-ATTACH`) is metadata-only — no binary retrieval, no attachment URL is returned. No raw record IDs appear in any check descriptor. The reader never makes network calls, never returns a token, full path, record payload, raw HTTP body, old/new record IDs, or attachment URL. `readsEnabled` is always `false`, `writesEnabled` is always `false`, `noChangesMade` is always `true`, `networkReadsAttempted` is always `false`, `networkWritesAttempted` is always `false`. No production-target mode exists. No UI execute button is provided. Live validation reads and end-to-end restore execution remain pending.

### Restore Orchestrator Foundation (internal)

The restore orchestrator foundation (`build_restore_orchestrator_plan` in `restore/restore_orchestrator.rs`) is an internal Rust module — it is not exposed as a Tauri command and has no UI surface. It builds a deterministic eight-phase orchestration plan sequencing all existing executor foundations: (1) schema write executor, (2) schema checkpoint boundary, (3) record write executor, (4) record checkpoint boundary, (5) linked second-pass executor, (6) linked phase checkpoint boundary, (7) final validation reader, (8) final guard. No Airtable API calls are made at any point. No checkpoint files are written.

The orchestrator checks twelve prerequisites (ORCH-PRE-01 through ORCH-PRE-12): write gate disabled, mode must be `sandboxOnly`, sandbox environment verified, target empty verified, write phase ordering policy safe, failure modes policy safe, rollback limitation policy safe, live write readiness safe, schema write executor foundation safe, record write executor foundation safe, linked second-pass executor foundation safe, and final validation reader foundation safe. If any prerequisite is missing, the result is `Blocked`. If all prerequisites are satisfied, the result is `NotExecuted` — because `evaluate_write_gate()` currently always returns `Disabled`.

The orchestrator never makes network calls, never returns a token, full path, record payload, raw HTTP body, old/new record IDs, or attachment URL. `writesEnabled` is always `false`, `readsEnabled` is always `false`, `noChangesMade` is always `true`, `networkReadsAttempted` is always `false`, `networkWritesAttempted` is always `false`. No production-target mode exists. No UI execute button is provided. Future sandbox-only gate enablement, live restore execution, and end-to-end restore remain separate pending work.

### Sandbox Schema Write Integration Harness (test-only, ignored by default)

The sandbox schema write integration harness (`tests/live_schema_write_sandbox.rs`) is a Rust integration test file. It is `#[ignore]` by default — normal `cargo test` will not run it.

**What it does (when all env vars are set and `--ignored` is passed):**

1. Verifies `evaluate_write_gate()` returns `Disabled` before the live call.
2. Verifies the live schema write test contract returns `eligibleButNotExecuted`.
3. Verifies the sandbox schema write adapter returns `readyForSandboxCall`.
4. Calls `createTable` via the Airtable Metadata API against the declared sandbox base — creating one table with one `singleLineText` field.
5. Verifies `evaluate_write_gate()` still returns `Disabled` after the live call.
6. Asserts the table name matches and the outcome contains no token.

**What it does NOT do:**

- Does not create records.
- Does not perform linked record updates.
- Does not call attachment or record endpoints.
- Does not perform final validation reads.
- Does not enable app runtime execution, writes, or reads.
- Does not modify `evaluate_write_gate()` behavior.
- Does not introduce any UI, Tauri command, or TypeScript path.
- Does not persist the token.
- Does not print or assert on the token or base ID value.

**Cleanup:** The test may leave a sandbox-only test table (`airbridge_sandbox_test_schema_write` by default) in the target base. Remove it manually after the run. No automatic cleanup path exists.

**Required environment variables (opt-in only):**

| Variable | Description |
|----------|-------------|
| `AIRBRIDGE_ENABLE_LIVE_SCHEMA_WRITE_TEST` | Must be exactly `true` to opt in |
| `AIRBRIDGE_SANDBOX_AIRTABLE_TOKEN` | Personal access token for the sandbox base |
| `AIRBRIDGE_SANDBOX_TARGET_BASE_ID` | Airtable base ID of a disposable sandbox base |

**Optional:**

| Variable | Description |
|----------|-------------|
| `AIRBRIDGE_SANDBOX_TEST_PREFIX` | Prefix for the test table name (default: `airbridge_sandbox_test`) |

**To run manually:**

```
AIRBRIDGE_ENABLE_LIVE_SCHEMA_WRITE_TEST=true \
AIRBRIDGE_SANDBOX_AIRTABLE_TOKEN=your_pat \
AIRBRIDGE_SANDBOX_TARGET_BASE_ID=appYourSandboxBase \
cargo test --manifest-path apps/desktop/src-tauri/Cargo.toml \
  --test live_schema_write_sandbox \
  -- sandbox_schema_write_creates_table_and_verifies_contract --ignored
```

### Live Schema Write Test Contract (internal, contract-only)

The live schema write test contract (`evaluate_live_schema_write_test_contract` in `restore/live_schema_write_test_contract.rs`) is an internal Rust module — it is not exposed as a Tauri command and has no UI surface. It evaluates whether a future live schema write integration test could be attempted, without performing any Airtable network call, without accepting or persisting any token, and without enabling any runtime execution, writes, or reads. No Airtable API calls are made. No checkpoint files are written. The result is not stored globally.

The contract composes all existing sandbox prerequisite modules (schema write adapter, adapter chain runner, gate arming, simulator, enablement readiness) into an ordered 8-prerequisite chain (LSWTC-PRE-01 through LSWTC-PRE-08). It returns `eligibleButNotExecuted` only when all prerequisites pass and the explicit internal flag is set. It also reports required future-live conditions without executing them.

`contract_only` is always `true`. `appRuntimeExecutionEnabled` is always `false`. `appRuntimeWritesEnabled` is always `false`. `appRuntimeReadsEnabled` is always `false`. `noChangesMade` is always `true`. `networkReadsAttempted` is always `false`. `networkWritesAttempted` is always `false`. `airtableClientCalled` is always `false`. The result status is never `succeeded`, `enabled`, or `executionReady`. Mode variants are `disabled` (default) and `sandboxIntegrationCandidate` — no `production` mode exists. The live schema write integration test, record writes, linked record updates, final validation reads, and live end-to-end restore execution all remain pending separate work.

### Sandbox Adapter Chain Runner (internal, mock/no-op only)

The sandbox adapter chain runner (`run_sandbox_adapter_chain` in `restore/sandbox_adapter_chain_runner.rs`) is an internal Rust module — it is not exposed as a Tauri command and has no UI surface. It composes all four sandbox adapter boundaries in strict order (schema → record → linked → final validation) using mock/no-op adapters only. No Airtable API calls are made. No checkpoint files are written. The result is not stored globally.

The chain runner returns `mockRunNotExecuted` only when all eight prerequisites are satisfied and `explicit_internal_mock_chain_requested` is `true`. Four phase entries (SACR-PH-01 through SACR-PH-04) are reported, each with status `mockObserved` and a safe operation count. No raw operation payloads, record IDs, tokens, paths, raw HTTP bodies, or attachment URLs appear in any result field.

`runtimeExecutionEnabled` is always `false`. `appRuntimeWritesEnabled` is always `false`. `appRuntimeReadsEnabled` is always `false`. `noChangesMade` is always `true`. `networkReadsAttempted` is always `false`. `networkWritesAttempted` is always `false`. `airtableClientCalled` is always `false`. The result status is never `succeeded`, `enabled`, or `executionReady`. Mode variants are `disabled` (default) and `mockInternalOnly` — no `production` mode exists. Live end-to-end sandbox restore execution remains separate pending work.

### Sandbox Gate Contract Foundation (internal)

The sandbox gate contract foundation (`evaluate_sandbox_gate_contract` in `restore/sandbox_gate_contract.rs`) is an internal Rust module — it is not exposed as a Tauri command and has no UI surface. It evaluates whether all prerequisites for a future sandbox-only gate enablement are present, without enabling anything. No Airtable API calls are made. No gate is armed. No restore execution is started.

The contract checks twelve prerequisites (SGC-PRE-01 through SGC-PRE-12): sandbox verification safe, target empty safe, explicit confirmation gate declared, destructive operation policy safe, attachment phase disabled policy safe, live write readiness safe, restore orchestrator present and default-blocked, schema executor present and default-blocked, record executor present and default-blocked, linked second-pass executor present and default-blocked, final validation reader present and default-blocked, and write gate default remains `Disabled/DisabledByProductPolicy`. The prerequisites are evaluated in order; the first missing or unsafe prerequisite blocks the result.

The contract has three possible statuses: `disabled` (default — mode is `disabled`, no prerequisites evaluated), `blocked` (one or more prerequisites missing or unsafe), and `eligibleButNotArmed` (all prerequisites satisfied — this does NOT arm the gate or enable execution). The mode variants are `disabled` and `sandboxOnlyCandidate` — no `production` mode exists.

`evaluate_write_gate()` is called internally and its result reported as SGC-PRE-12. `evaluate_write_gate()` always returns `Disabled/DisabledByProductPolicy` and is never modified by this module. `writesEnabled` is always `false`, `readsEnabled` is always `false`, `noChangesMade` is always `true`, `networkReadsAttempted` is always `false`, `networkWritesAttempted` is always `false`. `eligibleButNotArmed` is not equivalent to `enabled` — it is a forward-looking diagnostic status only. No arming, armed flag, success state, or enabled state exists in this module.

### Sandbox Restore Harness Foundation (internal)

The sandbox restore harness foundation (`build_sandbox_restore_harness_plan` in `restore/sandbox_restore_harness.rs`) is an internal Rust module — it is not exposed as a Tauri command and has no UI surface. It assembles the sandbox gate contract and restore orchestrator foundations into a single eight-phase dry harness plan for future sandbox end-to-end restore testing. No Airtable API calls are made. The gate is not armed. No restore execution is started. Live sandbox E2E restore execution remains pending.

The harness evaluates the gate contract (expecting `eligibleButNotArmed`), then the restore orchestrator (expecting `notExecuted` due to the disabled write gate), then verifies that schema executor, record executor, linked second-pass executor, final validation reader, and checkpoint boundary phases are each represented in the orchestrator plan. If any prerequisite is missing or unsafe, the result is `blocked`. If all prerequisites are satisfied, the result is `readyNotExecuted` — this does NOT arm the gate or enable execution.

The harness has three statuses: `notExecuted` (default — mode is `disabled`), `blocked` (prerequisite missing or unsafe), and `readyNotExecuted` (all prerequisites satisfied — not armed, not enabled). Mode variants are `disabled` and `sandboxOnlyDryHarness` — no `production` mode exists. `gate_armed` is always `false`. `writesEnabled` is always `false`. `readsEnabled` is always `false`. `noChangesMade` is always `true`. `networkReadsAttempted` is always `false`. `networkWritesAttempted` is always `false`. No armed, enabled, success, or completed state exists in this module.

### Sandbox Enablement Readiness Report (internal)

The sandbox enablement readiness report (`build_sandbox_enablement_readiness_report` in `restore/sandbox_enablement_readiness.rs`) is an internal Rust module — it is not exposed as a Tauri command and has no UI surface. It composes all existing restore foundation modules (sandbox gate contract, sandbox restore harness, restore orchestrator, schema write executor, record write executor, linked second-pass executor, final validation reader, checkpoint metadata store) to produce a single deterministic read-only diagnostic report. No Airtable API calls are made. No gate is armed. No restore execution is started.

The report evaluates 13 readiness items (SERN-01 through SERN-13) across nine categories: `safetyInvariant`, `gateContract`, `restoreHarness`, `orchestrator`, `schemaExecutor`, `recordExecutor`, `linkedExecutor`, `finalValidationReader`, and `checkpointStore`. Foundation modules are probed with minimal disabled-mode requests. Safety invariant items are declared by the report itself without code calls.

The report has three statuses: `blocked` (write gate not disabled — critical safety violation), `notReady` (one or more items missing or unsafe), and `readyButDisabled` (all 13 items ready — gate NOT armed, NOT enabled). `gate_armed` is always `false`. `writesEnabled` is always `false`. `readsEnabled` is always `false`. `noChangesMade` is always `true`. `networkReadsAttempted` is always `false`. `networkWritesAttempted` is always `false`. No armed, enabled, success, or completed state exists. `readyButDisabled` does NOT arm the gate or enable execution — it is a forward-looking diagnostic status only. Future sandbox-only gate enablement remains separate pending work.

### Sandbox Gate Arming Model (internal, not persisted)

The sandbox gate arming model (`build_sandbox_gate_arming_decision` in `restore/sandbox_gate_arming.rs`) is an internal Rust module — it is not exposed as a Tauri command and has no UI surface. It builds an ephemeral arming decision by verifying that the sandbox enablement readiness report returns `readyButDisabled`, the gate contract returns `eligibleButNotArmed`, and the restore harness returns `readyNotExecuted`. No Airtable API calls are made. The decision is not stored globally and does not affect runtime behavior.

The arming decision returns `armedNotExecutable` only when all prerequisites are satisfied and `explicit_internal_sandbox_arming_requested` is `true`. `armedNotExecutable` does NOT enable execution, writes, reads, or network calls. `gate_armed: true` describes the returned decision object only — it does not change `evaluate_write_gate()`, which continues to return `Disabled/DisabledByProductPolicy`. The decision is ephemeral: calling the function again produces a fresh independent result; no state is stored between calls.

The model has two statuses: `blocked` (default — any prerequisite missing, flag not set, or mode disabled) and `armedNotExecutable` (all prerequisites pass, internal flag is true). `executionEnabled` is always `false`. `writesEnabled` is always `false`. `readsEnabled` is always `false`. `noChangesMade` is always `true`. `networkReadsAttempted` is always `false`. `networkWritesAttempted` is always `false`. No `enabled`, `succeeded`, `complete`, `executionReady`, or `done` status exists. Live sandbox E2E restore execution remains separate pending work.

### Sandbox Restore Simulator (internal, in-memory only)

The sandbox restore simulator (`run_sandbox_restore_simulator` in `restore/sandbox_restore_simulator.rs`) is an internal Rust module — it is not exposed as a Tauri command and has no UI surface. It exercises the 8-phase restore sequence entirely in memory using mock/descriptor phases, without calling the real Airtable client, writing checkpoint files to disk, or enabling any live execution path. No Airtable API calls are made. No files are written. The result is not stored globally.

The simulator requires the sandbox gate arming decision to return `armedNotExecutable`, the harness to return `readyNotExecuted`, and the orchestrator to return `notExecuted`. When all prerequisites pass, it returns `simulatedNotExecuted` with all 8 phases marked `simulated` or `skipped` (checkpoint boundaries). The ephemeral arming decision seen during the run is noted in `ephemeral_armed_decision_seen` — this does NOT reflect a global armed state.

The simulator has two statuses: `blocked` (any prerequisite missing, flag not set, or mode disabled) and `simulatedNotExecuted` (all prerequisites pass). `gate_armed` (runtime/global) is always `false`. `executionEnabled` is always `false`. `writesEnabled` is always `false`. `readsEnabled` is always `false`. `noChangesMade` is always `true`. `networkReadsAttempted` is always `false`. `networkWritesAttempted` is always `false`. `airtableClientCalled` is always `false`. `checkpointFileWritten` is always `false`. No `succeeded`, `complete`, `executionReady`, `enabled`, or `done` status exists. Live sandbox E2E restore execution remains separate pending work.

### Sandbox Final Validation Adapter Boundary (internal, no network call)

The sandbox final validation adapter boundary (`build_sandbox_final_validation_adapter` in `restore/sandbox_final_validation_adapter.rs`) is an internal Rust module — it is not exposed as a Tauri command and has no UI surface. It describes final validation read operations as adapter-boundary read descriptors (`schemaCountReadDescriptor`, `fieldCountReadDescriptor`, `recordCountReadDescriptor`, `linkedFieldCoverageReadDescriptor`, `attachmentMetadataReadDescriptor`, `manifestChecksumReadDescriptor` when a manifest is present, and `finalGuardDescriptor`), without calling the real Airtable client, enabling runtime writes, reads, or execution, or persisting any state globally. No Airtable network calls are made. No schema, first-pass record create, linked update, attachment endpoint, or checkpoint operations are included.

The adapter requires the sandbox gate arming decision to return `armedNotExecutable`, the sandbox restore simulator to return `simulatedNotExecuted`, the final validation reader plan to return `notExecuted`, the schema write adapter to return `readyForSandboxCall`, the record write adapter to return `readyForSandboxCall`, the linked second-pass adapter to return `readyForSandboxCall`, `final_validation_enforcement_safe` to be true, and `sandbox_verified` to be true. When all prerequisites pass and the explicit internal flag is set, it returns `readyForSandboxCall` with read descriptors for all applicable validation checks.

`readyForSandboxCall` does NOT execute any Airtable network call. `runtimeExecutionEnabled` is always `false`. `appRuntimeWritesEnabled` is always `false`. `appRuntimeReadsEnabled` is always `false`. `networkReadsAttempted` is always `false`. `networkWritesAttempted` is always `false`. `noChangesMade` is always `true`. `evaluate_write_gate()` continues to return `Disabled/DisabledByProductPolicy`. No token, full path, record payload, raw HTTP body, old/new record IDs, or attachment URL appears in any result field. Attachment descriptors are metadata-only (filename, MIME type, size) — no binary retrieval and no CDN URL. No production adapter is implemented. The adapter provides a `FinalValidationReadAdapter` trait with `NoOpFinalValidationReadAdapter` and `MockFinalValidationReadAdapter` for test-only use.

The adapter has three statuses: `notExecuted` (default, mode disabled), `blocked` (prerequisite missing or flag not set), and `readyForSandboxCall` (all prerequisites pass). Live end-to-end restore execution remains pending separate work.

### Sandbox Linked Second-Pass Adapter Boundary (internal, no network call)

The sandbox linked second-pass adapter boundary (`build_sandbox_linked_second_pass_adapter` in `restore/sandbox_linked_second_pass_adapter.rs`) is an internal Rust module — it is not exposed as a Tauri command and has no UI surface. It describes linked second-pass update batches (`linkedUpdateBatchDescriptor`) as adapter-boundary operation objects, without calling the real Airtable client, enabling runtime writes or reads, or persisting any state globally. No Airtable network calls are made. No schema, first-pass record create, checkpoint, attachment, or skipped-field operations are included.

The adapter requires the sandbox gate arming decision to return `armedNotExecutable`, the sandbox restore simulator to return `simulatedNotExecuted`, the linked second-pass executor plan to return `notExecuted`, the schema write adapter to return `readyForSandboxCall`, the record write adapter to return `readyForSandboxCall`, and mapping coverage to be declared sufficient (without exposing record IDs). When all prerequisites pass and the explicit internal flag is set, it returns `readyForSandboxCall` with batch descriptors covering the declared field summaries.

`readyForSandboxCall` does NOT execute any Airtable network call. `runtimeExecutionEnabled` is always `false`. `appRuntimeWritesEnabled` is always `false`. `appRuntimeReadsEnabled` is always `false`. `networkWritesAttempted` is always `false`. `noChangesMade` is always `true`. `evaluate_write_gate()` continues to return `Disabled/DisabledByProductPolicy`. No token, full path, record payload, raw HTTP body, old/new record IDs, or attachment URL appears in any result field. No production adapter is implemented. The adapter provides a `LinkedSecondPassAdapter` trait with `NoOpLinkedSecondPassAdapter` and `MockLinkedSecondPassAdapter` for test-only use.

The adapter has three statuses: `notExecuted` (default, mode disabled), `blocked` (prerequisite missing or flag not set), and `readyForSandboxCall` (all prerequisites pass). Final validation reads, attachment handling, and live end-to-end restore execution remain pending separate work.

### Sandbox Record Write Adapter Boundary (internal, no network call)

The sandbox record write adapter boundary (`build_sandbox_record_write_adapter` in `restore/sandbox_record_write_adapter.rs`) is an internal Rust module — it is not exposed as a Tauri command and has no UI surface. It describes first-pass record create operations (createRecordBatchDescriptor) as adapter-boundary operation objects, without calling the real Airtable client, enabling runtime writes or reads, or persisting any state globally. No Airtable network calls are made. No linked update, schema, checkpoint, attachment, or skipped-field operations are included.

The adapter requires the sandbox gate arming decision to return `armedNotExecutable`, the sandbox restore simulator to return `simulatedNotExecuted`, the record write executor plan to return `notExecuted`, and the schema write adapter to return `readyForSandboxCall`. When all prerequisites pass and the explicit internal flag is set, it returns `readyForSandboxCall` with operation descriptors for first-pass record create batches only.

`readyForSandboxCall` does NOT execute any Airtable network call. `runtimeExecutionEnabled` is always `false`. `appRuntimeWritesEnabled` is always `false`. `appRuntimeReadsEnabled` is always `false`. `networkWritesAttempted` is always `false`. `noChangesMade` is always `true`. `evaluate_write_gate()` continues to return `Disabled/DisabledByProductPolicy`. No token, full path, record payload, raw HTTP body, old/new record IDs, or attachment URL appears in any result field. No production adapter is implemented. The adapter provides a `RecordWriteAdapter` trait with `NoOpRecordWriteAdapter` and `MockRecordWriteAdapter` for test-only use.

The adapter has three statuses: `notExecuted` (default, mode disabled), `blocked` (prerequisite missing or flag not set), and `readyForSandboxCall` (all prerequisites pass). Linked record updates, final validation reads, attachment handling, and live end-to-end restore execution remain pending separate work.

### Sandbox Schema Write Adapter Boundary (internal, no network call)

The sandbox schema write adapter boundary (`build_sandbox_schema_write_adapter` in `restore/sandbox_schema_write_adapter.rs`) is an internal Rust module — it is not exposed as a Tauri command and has no UI surface. It describes schema write operations (create table descriptor, create field descriptor) as adapter-boundary operation objects, without calling the real Airtable client, enabling runtime writes or reads, or persisting any state globally.

The adapter requires the sandbox gate arming decision to return `armedNotExecutable`, the sandbox restore simulator to return `simulatedNotExecuted`, and the schema write executor plan to return `notExecuted`. When all prerequisites pass and the explicit internal flag is set, it returns `readyForSandboxCall` with operation descriptors scoped to schema operations (createTable and createField) only. Record, linked update, deferred-field, manual-action, and attachment operations are excluded from the adapter boundary.

`readyForSandboxCall` does NOT execute any Airtable network call. `runtimeExecutionEnabled` is always `false`. `appRuntimeWritesEnabled` is always `false`. `appRuntimeReadsEnabled` is always `false`. `networkWritesAttempted` is always `false`. `noChangesMade` is always `true`. `evaluate_write_gate()` continues to return `Disabled/DisabledByProductPolicy`. No token, full path, record payload, raw HTTP body, old/new record IDs, or attachment URL appears in any result field. No production adapter is implemented. The adapter provides a `SchemaWriteAdapter` trait with `NoOpSchemaWriteAdapter` and `MockSchemaWriteAdapter` for test-only use.

The adapter has three statuses: `notExecuted` (default, mode disabled), `blocked` (prerequisite missing or flag not set), and `readyForSandboxCall` (all prerequisites pass). Record writes, linked record updates, final validation reads, attachment handling, and live end-to-end restore execution remain pending separate work.

### Live Record Write Test Contract (internal, no network call)

The live record write test contract (`evaluate_live_record_write_test_contract` in `restore/live_record_write_test_contract.rs`) is an internal Rust module — it is not exposed as a Tauri command and has no UI surface. It is a contract-only readiness layer that evaluates whether a future live record write integration test could be attempted, without performing any live call.

The contract evaluates 10 prerequisites (LRWTC-PRE-01 through LRWTC-PRE-10):

| ID | Prerequisite |
|----|-------------|
| LRWTC-PRE-01 | Mode is `sandboxIntegrationCandidate` |
| LRWTC-PRE-02 | `explicit_internal_live_record_test_contract_requested` is true |
| LRWTC-PRE-03 | `evaluate_write_gate()` returns `Disabled/DisabledByProductPolicy` |
| LRWTC-PRE-04 | Live schema write test contract returns `EligibleButNotExecuted` |
| LRWTC-PRE-05 | Sandbox record write adapter returns `ReadyForSandboxCall` |
| LRWTC-PRE-06 | Sandbox schema write adapter returns `ReadyForSandboxCall` |
| LRWTC-PRE-07 | Sandbox adapter chain runner returns `MockRunNotExecuted` |
| LRWTC-PRE-08 | Sandbox gate arming decision returns `ArmedNotExecutable` |
| LRWTC-PRE-09 | Sandbox restore simulator returns `SimulatedNotExecuted` |
| LRWTC-PRE-10 | Sandbox enablement readiness returns `ReadyButDisabled` |

`eligibleButNotExecuted` does NOT perform any Airtable network call. `contractOnly` is always `true`. `appRuntimeExecutionEnabled` is always `false`. `appRuntimeWritesEnabled` is always `false`. `appRuntimeReadsEnabled` is always `false`. `networkReadsAttempted` is always `false`. `networkWritesAttempted` is always `false`. `airtableClientCalled` is always `false`. `noChangesMade` is always `true`. No token is accepted or stored. `evaluate_write_gate()` remains `Disabled/DisabledByProductPolicy`. No UI surface, no Tauri command, no TypeScript path exists.

The contract reports the following required future-live conditions (not executed):
- Disposable sandbox-only base required.
- Schema phase must already be test-created or safely represented before record writes.
- Explicit test-only credentials required in future task.
- No UI execution path allowed.
- Only first-pass record create operations allowed.
- Linked record updates remain disabled.
- Final validation reads remain disabled.
- Attachment handling remains disabled.

The live record write integration test itself remains separate pending work.

---

## Related Documents

- [Tauri Command Inventory](../architecture/tauri-command-inventory.md)
- [Known Limitations](../release/known-limitations.md)
- [v0.1.0-alpha Readiness](../release/v0.1.0-alpha-readiness.md)
- [Restore Execution Command Contract](../architecture/restore-execution-command-contract.md)
- [Safe Backup Command Contract](../architecture/safe-backup-command-contract.md)
