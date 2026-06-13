# Live Restore Write Safety Checklist

Use this checklist to verify that all safety gates defined in the live restore write safety contract are satisfied before any live Airtable write path is enabled. Every item must be marked Pass (P) before `evaluate_write_gate()` may return an enabled decision.

**Review date:** ___________  
**Reviewer:** ___________  
**Build version:** ___________  
**Target scope:** ___________  

---

## Gate 1 — Sandbox Environment Verification

**Status: Implemented (local checks only). Live Airtable metadata check deferred.**

The `verify_restore_sandbox_environment` Tauri command runs 10 local safety checks (CHK-01 through CHK-10). No Airtable API calls are made. No token is required. No files are written. `noChangesMade` is always `true`. `writesEnabled` is always `false`.

- [x] **Sandbox verification command implemented.** `verify_restore_sandbox_environment` runs CHK-01 through CHK-10. Accessible from the Restore page.
- [x] **CHK-01 target mode.** `NewBase` and `EmptyExistingBase` pass; `EmptyExistingBase` without identifier is a warning.
- [x] **CHK-02 empty target expectation.** Blocked if `expectsEmptyTarget` is false.
- [x] **CHK-03 write gate.** Calls `evaluate_write_gate()` — must return `Disabled`. Passes if disabled; blocked if unexpectedly enabled.
- [x] **CHK-04 destructive operations.** Blocked if `allowDestructiveOperations` is true.
- [x] **CHK-05 attachment upload.** Warning (not blocked) if `allowAttachmentUpload` is true.
- [x] **CHK-06 plan status.** Blocked if schema or record import plan status is `"blocked"`.
- [x] **CHK-07 filename safety.** Warns if source package filename contains suspicious patterns.
- [x] **CHK-08 token safety.** Always passes — no token accepted or returned by this command.
- [x] **CHK-09 network safety.** Always passes — no Airtable API calls are made.
- [x] **CHK-10 live metadata check.** Always skipped — future implementation required.
- [x] **Safety summary fields always correct.** `writesEnabled: false`, `networkWritesAttempted: false`, `noChangesMade: true`, `writeGateStatus: "disabled"`, `liveMetadataCheckPerformed: false`.
- [x] **22 Rust unit tests pass** for sandbox verification module.
- [x] **25 frontend tests pass** for sandbox verification service contract and panel rendering.
- [ ] **Live metadata check implemented.** A future release must implement CHK-10 with a real Airtable read call to verify the target base is empty and accessible before enabling writes.

### Pre-write current state checks

- [ ] **Write gate still disabled.** `evaluate_write_gate()` returns `Disabled/DisabledByProductPolicy`. No branch exists that returns an enabled decision.
- [ ] **No `Succeeded` status.** `RestoreWriteEngineStatus` has no `Succeeded` variant. Confirm by grepping the Rust source: `grep -r "Succeeded" apps/desktop/src-tauri/src/restore/` returns no results in `write_result.rs`.
- [ ] **`noChangesMade` always true.** All write engine result types set `no_changes_made: true`. Rust tests confirm this for every code path.
- [ ] **`networkWritesAttempted` always false.** Schema and record write foundation result types set `network_writes_attempted: false`. Rust tests confirm this.
- [ ] **933+ Rust unit tests pass.** `cargo test --lib` exits 0 with no failures.
- [ ] **567+ frontend tests pass.** `vitest run` exits 0 with no failures.

---

## Gate 2 — Sandbox Testing

- [ ] **Sandbox base designated.** A dedicated empty Airtable base exists for write testing. Its base ID is in test configuration only, not in release code.
- [ ] **Schema creation phase tested in sandbox.** CreateTable, CreateField, and DeferLinkedField operations complete successfully against the sandbox base.
- [ ] **Record creation phase tested in sandbox.** CreateRecordBatch operations complete with correct field values. Record count in target matches expected count.
- [ ] **Linked second-pass tested in sandbox.** UpdateLinkedRecordBatch operations complete after first-pass records exist. Linked field values point to new record IDs, not source IDs.
- [ ] **Final validation phase tested in sandbox.** Record and table counts verified; `Succeeded` status only set after validation passes.
- [ ] **Sandbox base deleted after test.** No production data affected.

---

## Gate 2 — Explicit User Confirmation

**Status: Implemented (local validation only). Live writes still disabled.**

The `validate_restore_confirmation_gate` Tauri command validates the user's confirmation text. No Airtable API calls are made. No token is required. No files are written. `noChangesMade` is always `true`. `writesEnabled` is always `false`. A `Confirmed` result does NOT enable restore writes.

- [x] **Confirmation command implemented.** `validate_restore_confirmation_gate` runs CHK-C01 through CHK-C05. Accessible from the Restore page.
- [x] **CHK-C01 write gate check.** Calls `evaluate_write_gate()` — must return `Disabled`. Always passes in this version.
- [x] **CHK-C02 sandbox prerequisite.** Blocked if sandbox status is `"blocked"`. Skipped if not yet run. Passes if `"verified"` or `"warning"`.
- [x] **CHK-C03 exact text match.** Requires exact trim-then-compare match, case-sensitive.
- [x] **CHK-C04 no token in text.** Blocked if entered text resembles an Airtable PAT format.
- [x] **CHK-C05 writes remain disabled.** Always passes — confirmation does not enable writes.
- [x] **Required text is deterministic.** Built from target label or package filename; falls back to `"RESTORE BACKUP"`. Never contains path or token.
- [x] **Wrong case rejected.** `"restore to my base"` returns `rejected`.
- [x] **Partial match rejected.** `"RESTORE"` alone returns `rejected`.
- [x] **Extra words rejected.** `"RESTORE TO MY BASE NOW"` returns `rejected`.
- [x] **Blocked sandbox propagates.** If sandbox status is `"blocked"`, confirmation result is `"blocked"` regardless of entered text.
- [x] **30 Rust unit tests pass** for confirmation module.
- [x] **39 frontend tests pass** for confirmation service contract and panel rendering.
- [ ] **Confirmation enables live write path.** When the write engine is enabled in a future release, a valid `Confirmed` result (with all prerequisites met) must be checked before any write begins.

## Gate 3 — Target Empty Verification

**Status: Implemented (local and reported counts only). Live Airtable metadata check deferred.**

The `verify_restore_target_empty` Tauri command verifies that the target base is empty before any live writes begin. No Airtable write API calls are made. No token is required. No files are written. `noChangesMade` is always `true`. `writesEnabled` is always `false`. A `Verified` result does NOT enable restore writes.

- [x] **Target empty verification command implemented.** `verify_restore_target_empty` runs TEV-01 through TEV-05. Accessible from the Restore page.
- [x] **TEV-01 write gate check.** Calls `evaluate_write_gate()` — must return `Disabled`. Always passes in this version.
- [x] **TEV-02 target mode allowlist.** Only `newBase` and `emptyExistingBase` are supported. Any other value is blocked.
- [x] **TEV-03 table count check.** `newBase` always passes. `emptyExistingBase` with 0 tables passes; non-zero is blocked; unknown is warning.
- [x] **TEV-04 record count check.** `newBase` always passes. `emptyExistingBase` with 0 records passes; non-zero is blocked; unknown is warning.
- [x] **TEV-05 no-writes-enabled check.** Always passes — verification does not enable writes.
- [x] **Unknown counts produce warning, not blocked.** When counts are unavailable for `emptyExistingBase`, result is `warning` not `blocked` — allowing the user to proceed with awareness.
- [x] **Non-zero table count is blocked.** `targetTableCount > 0` for `emptyExistingBase` produces `blocked`.
- [x] **Non-zero record count is blocked.** `targetRecordCount > 0` for `emptyExistingBase` produces `blocked`.
- [x] **29 Rust unit tests pass** for target empty verification module.
- [x] **47 frontend tests pass** for target empty verification service contract and panel rendering.
- [ ] **Live metadata check implemented.** A future release must implement a live Airtable read call to `GET /v0/meta/bases/{baseId}/tables` to count tables and records before enabling writes.

## Gate 4 — Destructive Operation Policy

**Status: Implemented (declared-operation check only). Live writes still disabled.**

The `verify_destructive_operation_policy_gate` Tauri command verifies that no destructive operations are declared in the restore plan. No Airtable API calls are made. No token is required. No files are written. `noChangesMade` is always `true`. `writesEnabled` is always `false`. A `Compliant` result does NOT enable restore writes.

- [x] **Destructive operation policy command implemented.** `verify_destructive_operation_policy_gate` runs DOP-01 through DOP-05. Accessible from the Restore page.
- [x] **DOP-01 write gate check.** Calls `evaluate_write_gate()` — must return `Disabled`. Always passes in this version.
- [x] **DOP-02 no delete operations.** `deleteBase`, `deleteTable`, `deleteField`, `deleteRecord` are all blocked.
- [x] **DOP-03 no update/overwrite operations.** `updateExistingRecord`, `overwriteField`, `overwriteTable` are all blocked.
- [x] **DOP-04 no attachment upload.** `attachmentUpload` is blocked in this phase. Only attachment metadata preservation is allowed.
- [x] **DOP-05 create-only classification.** All remaining declared operations must be create-only or safe; unknown operation kinds produce a `Warning`.
- [x] **Compliant result does not enable writes.** `writesEnabled: false` always.
- [x] **Blocked result names the offending operations.** `blockedOperations` list contains the label of every blocked operation.
- [x] **Empty operation list is compliant.** A plan with no declared operations returns `compliant`.
- [x] **29 Rust unit tests pass** for destructive operation policy module.
- [x] **46 frontend tests pass** for destructive operation policy service contract and panel rendering.
- [ ] **Policy wired to live plan.** A future release must pass the actual planned operations from the schema and record write foundations to this command before any write is attempted.

---

## Gate 5 — Attachment Upload Policy

- [x] **`AttachmentUploadPolicyRequest` / `AttachmentUploadPolicyResult` types defined.** Rust types use `#[serde(rename_all = "camelCase")]`. TypeScript types are in `backend/types.ts`.
- [x] **`verify_attachment_upload_policy()` implemented.** Runs 5 checks: AUP-01 through AUP-05.
- [x] **AUP-01 always passes.** Write gate check always returns `Passed`.
- [x] **AUP-02 blocks upload-requested.** Any `UploadRequested` field intent causes status `Blocked` and a non-empty `blockedFieldNames` list.
- [x] **AUP-03 warns on download-requested.** `DownloadRequested` field intent causes status `Warning` only — not `Blocked`.
- [x] **AUP-04 warns on unknown intents.** `Unknown` field intent causes status `Warning` only.
- [x] **AUP-05 confirms metadata-only.** All `MetadataOnly` fields cause `Passed`. Any non-metadata field causes `Warning` or `Failed`.
- [x] **Empty field list is compliant.** A plan with no declared attachment fields returns `compliant`.
- [x] **`noChangesMade` always true.** All 3 safety invariants are always set in every result.
- [x] **`writesEnabled` always false.** Compliant result does NOT enable restore writes.
- [x] **No full attachment URL in any result field.** `dl.airtable.com` and `airtableusercontent.com` never appear in any serialized output.
- [x] **36 Rust unit tests pass** for attachment upload policy module.
- [x] **Sufficient frontend tests pass** for attachment upload policy service contract and panel rendering.
- [ ] **Policy wired to live plan.** A future release must pass the actual declared attachment fields from the dry-run plan to this command before any write is attempted.

---

## Gate 6 — Sandbox Write Testing

- [ ] **Sandbox base designated.** A dedicated empty Airtable base exists for write testing. Its base ID is in test configuration only, not in release code.
- [ ] **Schema creation phase tested in sandbox.** CreateTable, CreateField, and DeferLinkedField operations complete successfully against the sandbox base.
- [ ] **Record creation phase tested in sandbox.** CreateRecordBatch operations complete with correct field values. Record count in target matches expected count.
- [ ] **Linked second-pass tested in sandbox.** UpdateLinkedRecordBatch operations complete after first-pass records exist. Linked field values point to new record IDs, not source IDs.
- [ ] **Final validation phase tested in sandbox.** Record and table counts verified; `Succeeded` status only set after validation passes.
- [ ] **Sandbox base deleted after test.** No production data affected.

## Gate 7 — User Confirmation for Live Writes

- [ ] **Confirmation phrase enforced at Rust level.** The Rust command rejects any confirmation that is not exactly the required phrase (see Gate 2).
- [ ] **Partial match rejected.** Lowercase, partial, and extra-word inputs all fail the check.
- [ ] **Confirmation phrase shown in UI.** The user sees the phrase they must type before the input field is rendered.
- [ ] **No auto-fill.** The confirmation input is never pre-filled programmatically.
- [ ] **Confirmation constant shared.** A single Rust constant and a single TypeScript constant hold the fixed fallback phrase. Tests use these constants, not inline strings.

---

## Gate 8 — Target Base Safety

- [ ] **Empty-base check implemented.** Before the first write, the Airtable schema API is called to verify the target base has zero tables.
- [ ] **Non-empty base rejected.** If the target base contains any tables, the write engine stops with a `BlockedByTargetSafety` reason before any write is attempted.
- [ ] **New-base creation verified.** If target mode is "new base", the created base ID is confirmed before the schema phase begins.
- [ ] **Target base ID not in public result.** The target base ID does not appear in any result, log line, or job history item.

---

## Gate 9 — No Destructive Operations

- [ ] **No delete record calls.** Grep confirms no `DELETE /v0/` record endpoint is called anywhere in the write path.
- [ ] **No delete table calls.** Grep confirms no table deletion API call exists.
- [ ] **No delete field calls.** Grep confirms no field deletion API call exists.
- [ ] **No PATCH/PUT on existing records outside second pass.** The only PATCH calls are in the `UpdateLinkedRecordBatch` phase.
- [ ] **No record creation in non-target base.** The base ID used for every write call matches the confirmed target base ID.

---

## Gate 10 — Write Phase Ordering

- [ ] **Tables created before fields.** `CreateTable` operations all complete before any `CreateField` operation begins.
- [ ] **Fields created before records.** `CreateField` and `DeferLinkedField` operations complete before any `CreateRecordBatch` begins.
- [ ] **Records created before linked second pass.** All `CreateRecordBatch` first-pass operations complete before any `UpdateLinkedRecordBatch` begins.
- [ ] **ID mapping complete before second pass.** The old-to-new record ID mapping is fully populated for all tables with linked fields before `UpdateLinkedRecordBatch` operations issue any PATCH call.
- [ ] **Phase ordering enforced in code.** The write engine does not proceed to the next phase if the current phase contains a non-retryable failure.

---

## Gate 11 — Checkpoint Safety

- [ ] **Checkpoint before each batch.** A durable checkpoint is written before each `CreateRecordBatch` operation begins.
- [ ] **Checkpoint before second pass.** A durable checkpoint records first-pass completion before `UpdateLinkedRecordBatch` begins.
- [ ] **Resumption tested.** Simulate an interrupted write at batch 2 of 5. Confirm that re-running the write engine resumes from batch 2, not from batch 1.
- [ ] **No duplicate records on resume.** Records created before the interruption are not re-created on resume.

---

## Gate 12 — Rate Limit and Backoff

- [ ] **429 triggers backoff.** A 429 response from the Airtable API pauses the write engine and waits before retrying.
- [ ] **Initial backoff ≥ 1 000 ms.** The first retry waits at least one second.
- [ ] **Backoff multiplier ≥ 2×.** Subsequent retries wait progressively longer.
- [ ] **Maximum retries ≥ 5.** After 5 retries on the same batch, the operation fails permanently.
- [ ] **Retry policy from record import planner.** The `RestoreRetryPolicy` values from the record import plan are used — not new hardcoded values.
- [ ] **Rate-limit event in restore report.** When a 429 is encountered, a rate-limit event appears in the restore report.
- [ ] **No write during backoff.** The Airtable API is not called during the backoff wait period.

---

## Gate 13 — Failure Modes

- [ ] **401 stops execution.** An authentication error stops all further writes. "Token invalid" message shown. No further API calls.
- [ ] **403 stops execution.** A permission error stops all further writes. "Permission denied" message shown.
- [ ] **Base/table 404 stops phase.** A 404 on the target base or table stops the affected phase. Other phases are not attempted.
- [ ] **Schema conflict (422) handled.** A 422 on field creation adds the field to the manual-action list and continues to the next field.
- [ ] **5xx retried, then failed permanently.** After max retries on a 5xx, the operation is marked failed. Partial failure is reported.
- [ ] **Checkpoint failure stops execution.** If a checkpoint cannot be written (disk full, permission denied), the write engine stops and does not attempt the next batch.
- [ ] **Partial failure report is accurate.** The restore report identifies every table, batch, and field that succeeded or failed.

---

## Gate 14 — Rollback Limitation Notice

- [ ] **Notice shown in UI before confirmation.** Before the user types `"RESTORE BACKUP"`, a notice reads: "Restore cannot be automatically rolled back. If execution fails partway through, the partially-created base must be cleaned up manually."
- [ ] **Notice not dismissable.** The notice is always visible when the confirmation input is shown — it is not collapsible or behind a toggle.
- [ ] **Report identifies partial state.** After a partial failure, the restore report shows exactly which tables and record batches were written before the failure.

---

## Gate 15 — Final Validation

- [ ] **`FinalValidation` phase implemented.** A `FinalValidation` phase exists in the write engine and runs after all record writes complete.
- [ ] **Record count check.** `FinalValidation` verifies the record count in the target base matches the count in the backup manifest.
- [ ] **Table presence check.** `FinalValidation` verifies all tables from the manifest are present in the target base.
- [ ] **`Succeeded` only after validation passes.** The write engine result is only allowed to carry `Succeeded` status when `FinalValidation` completes without error.
- [ ] **`Succeeded` status added only when this gate is complete.** The `RestoreWriteEngineStatus::Succeeded` variant is not added until `FinalValidation` is implemented and this checklist item is marked Pass.

---

## Gate 16 — Token Safety During Live Writes

- [ ] **Token not in any write engine result.** `RestoreWriteEngineResult` has no token field. `JSON.stringify(result)` does not contain the token string.
- [ ] **Token not in any job history item.** After a live restore, `list_job_history` response does not contain the token.
- [ ] **Token not in any log line.** Log file does not contain the token string after a live restore.
- [ ] **Token cleared after execution.** In-memory token state in the UI is cleared after the write completes (success, failure, or partial).
- [ ] **`RestoreExecutionRequest` still non-serializable.** The struct derives `Deserialize` only. `serde_json::to_string(&request)` must fail to compile.

---

## Gate 17 — Path Safety During Live Writes

- [ ] **Full path not in any result.** `RestoreWriteEngineResult.filename` contains only the basename.
- [ ] **`Path::file_name()` applied.** Confirmed by code review that all result filenames are derived via `Path::file_name()`.
- [ ] **Full path not in any event.** `RestoreWriteEvent.message` does not contain absolute path components.
- [ ] **Full path not in any log line.** Log file does not contain `/Users/`, `/home/`, or `:\\` after a live restore.

---

## Gate 18 — Attachment Phase Disabled

- [ ] **`AttachmentHandling` phase remains disabled.** The phase produces a disabled-status summary with zero operations.
- [ ] **No attachment upload API call.** Grep confirms no attachment upload endpoint (`/v0/{baseId}/{tableId}/{recordId}/files` or similar) is called.
- [ ] **All attachment fields use `MetadataOnly` policy.** The record import plan continues to assign `MetadataOnly` to all attachment fields.
- [ ] **Restore report notes manual re-attachment.** The report explicitly states that attachment fields require manual re-attachment after restore.

---

## Gate 19 — No Prohibited Terms in Public Files

- [ ] **Full prohibited terms scan passes.** `grep -RniE 'claude|anthropic|chatgpt|openai|ai-generated|ai-assisted|agent|llm|co-authored|generated with|generated by'` returns no hits in source, docs, or config files.

---

## Summary

| Gate | Description | Status |
|------|-------------|--------|
| 1 | Sandbox environment verification | ☐ |
| 2 | Explicit user confirmation | ☐ |
| 3 | Target empty verification | ☐ |
| 4 | Destructive operation policy | ☐ |
| 5 | Attachment upload policy | ☐ |
| 6 | Sandbox write testing | ☐ |
| 7 | User confirmation for live writes | ☐ |
| 8 | Target base safety | ☐ |
| 9 | No destructive operations | ☐ |
| 10 | Write phase ordering | ☐ |
| 11 | Checkpoint safety | ☐ |
| 12 | Rate limit and backoff | ☐ |
| 13 | Failure modes | ☐ |
| 14 | Rollback limitation notice | ☐ |
| 15 | Final validation | ☐ |
| 16 | Token safety | ☐ |
| 17 | Path safety | ☐ |
| 18 | Attachment phase disabled | ☐ |
| 19 | No prohibited terms | ☐ |

**Release decision:** Do not enable live writes until all 19 gates are marked Pass.

---

## Related Documents

- [Live Restore Write Safety Contract](../architecture/live-restore-write-safety-contract.md)
- [Restore Write Engine Skeleton](../architecture/restore-write-engine-skeleton.md)
- [Restore QA Checklist](./restore-qa-checklist.md)
- [Security and Privacy QA](./security-privacy-qa.md)
- [Known Limitations](../release/known-limitations.md)
