# Live Restore Write Safety Contract

This document defines the safety requirements that must all be satisfied before any live Airtable write operation is enabled in the restore path. It is a forward-looking contract, not a description of currently active behavior. No restore writes are performed in the current version.

---

## Current State

All restore write paths are hard-disabled. `evaluate_write_gate()` returns `Disabled/DisabledByProductPolicy` unconditionally. `RestoreWriteEngineStatus::Succeeded` does not exist in the type system. `noChangesMade` is always `true`. `networkWritesAttempted` is always `false`.

**Gate 1 — sandbox environment verification** is implemented via the `verify_restore_sandbox_environment` Tauri command. It runs 10 local safety checks (CHK-01 through CHK-10) and returns a structured result with `writesEnabled: false` always. CHK-10 (live Airtable metadata check) is always `Skipped` — it must be implemented before Gate 1 can be fully satisfied. No Airtable API calls are made. No token is required.

**Gate 2 — explicit user confirmation** is implemented via the `validate_restore_confirmation_gate` Tauri command. It runs 5 checks (CHK-C01 through CHK-C05): write gate state, sandbox prerequisite, exact text match, no-token check, and a writes-remain-disabled assertion. Required confirmation text is built deterministically from target label or package filename. No Airtable API calls are made. No token is required. A `Confirmed` result does NOT enable restore writes — it only validates the confirmation contract.

**Gate 3 — target empty verification** is implemented via the `verify_restore_target_empty` Tauri command. It runs 5 checks (TEV-01 through TEV-05): write gate state, target mode allowlist, table count, record count, and a no-writes-enabled assertion. `newBase` mode is always verified (no counts needed). `emptyExistingBase` mode requires both table count and record count to be 0; if counts are unknown, status is `Warning`. No Airtable write API calls are made. No token is required. A `Verified` result does NOT enable restore writes.

**Gate 4 — destructive operation policy** is implemented via the `verify_destructive_operation_policy_gate` Tauri command. It runs 5 checks (DOP-01 through DOP-05): write gate state, no delete operations, no update/overwrite operations, no attachment upload operations, and create-only classification of all remaining operations. Any declared delete, update, overwrite, or attachment-upload operation kind causes status `Blocked`. Unknown (unclassified) operation kinds cause status `Warning`. No Airtable API calls are made. No token is required. A `Compliant` result does NOT enable restore writes.

**Gate 5 — attachment upload policy** is implemented via the `verify_attachment_upload_policy_gate` Tauri command. It runs 5 checks (AUP-01 through AUP-05): write gate state, no upload-requested intents, no download-requested intents (warning only), no unknown intents (warning only), and metadata-only confirmation. Any `UploadRequested` field intent causes status `Blocked`. `DownloadRequested` or `Unknown` intents cause status `Warning`. Attachment file bytes are never uploaded or downloaded. No Airtable API calls are made. No token is required. No full attachment URL appears in any result field. A `Compliant` result does NOT enable restore writes.

**Gate 6 — schema record order policy** is implemented via the `verify_schema_record_order_policy_gate` Tauri command. It runs 5 checks (SRO-01 through SRO-05): write gate state, schema phase presence, schema-before-records ordering, records-before-linked-updates ordering, and records-before-attachments ordering. Missing or blocked schema phases cause `Blocked`. Ordering violations cause `Blocked` with a named violation string. Warning conditions produce `Warning`. No Airtable API calls are made. No token is required. No record payload appears in any result field. A `Compliant` result does NOT enable restore writes.

**Gate 7 — sandbox write testing policy** is implemented via the `verify_sandbox_write_testing_policy_gate` Tauri command. It runs 5 checks (SWT-01 through SWT-05): write gate state, target classification (sandbox vs. production/unknown), Gate 1 sandbox verification prerequisite, sandbox test evidence presence, and evidence completeness. A `Production` or `Unknown` target classification causes `Blocked`. Missing evidence causes `Blocked`. Incomplete evidence (any required field absent or false, or a full path in the filename) causes `Warning`. No Airtable API calls are made. No token is required. No record payload appears in any result field. Evidence filenames are basenames only — no directory path is accepted. A `Compliant` result does NOT enable restore writes.

**Gate 8 — live-write-specific user confirmation policy** is implemented via the `verify_live_write_confirmation_policy_gate` Tauri command. It runs 5 checks (LWC-01 through LWC-05): write gate state, prior safety gates not blocked (Gates 1–6), sandbox write testing gate not blocked (Gate 7), exact confirmation text match, and writes-remain-disabled assertion. The required confirmation phrase is built from the safe target label and always ends with `"— WRITES REMAIN DISABLED"`. Text matching is case-sensitive with outer whitespace trimmed only. A blocked prior gate causes `Blocked` regardless of text match. A wrong text input causes `Rejected`. Prior gate warnings with a matching text produce `Warning`. All other conditions with a matching text produce `Confirmed`. No Airtable API calls are made. No token is required. No record payload appears in any result field. No token or path field exists anywhere in the result. A `Confirmed` result does NOT enable restore writes — `writesEnabled` is always `false`.

**Gate 9 — rate-limit and backoff policy** is implemented via the `verify_rate_limit_backoff_policy_gate` Tauri command. It runs 10 checks (RLB-01 through RLB-10): write gate state, rate-limit plan declared, max requests/sec ≤ 5 (matching `DEFAULT_PER_BASE_RPS`), batch size ≤ 10, 429 handling declared, bounded retry count (max retries must not be `None`), backoff strategy declared, stop condition declared, checkpoint/resume compatibility, and writes-remain-disabled assertion. If no plan is declared, the function short-circuits after RLB-02 and returns `Blocked` with 2 checks only. RLB-09 (checkpoint compatibility) produces `Warning` for `partial`, `none`, or `unknown` values — not `Blocked`. RLB-01 and RLB-10 always pass. No Airtable API calls are made. No token is required. No record payload appears in any result field. A `Compliant` result does NOT enable restore writes — `writesEnabled` is always `false`.

**Gate 10 — checkpoint durability policy** is implemented via the `verify_checkpoint_durability_policy_gate` Tauri command. It runs 9 checks (CDP-01 through CDP-09): write gate state, checkpoint plan declared, checkpoint after each table, checkpoint after each batch, phase markers for all required phases (schema, record_create, linked_update, final_validation), old-to-new ID mapping checkpoint before linked updates, resume-safe stop condition, durability backend not memory-only, and writes-remain-disabled assertion. If no plan is declared, the function short-circuits after CDP-02 and returns `Blocked` with 2 checks only. CDP-08 (durability backend) produces `Warning` for `memory` or unknown backends — not `Blocked`. CDP-06 (ID mapping checkpoint) is required only when linked updates are declared; if no linked updates are planned, the check passes unconditionally. CDP-01 and CDP-09 always pass. No Airtable API calls are made. No token is required. No record payload appears in any result field. A `Compliant` result does NOT enable restore writes — `writesEnabled` is always `false`.

**Gate 11 — final validation policy** is implemented via the `verify_final_validation_policy_gate` Tauri command. It runs 12 checks (FVP-01 through FVP-12): write gate state, final validation plan declared, schema count validation, table/field presence validation, record count validation, old-to-new ID mapping validation, linked record second-pass validation, attachment metadata validation, attachment validation scope (metadata-only produces Warning), manifest checksum/reference validation, success-blocked-without-validation assertion, and writes-remain-disabled assertion. If no plan is declared, the function short-circuits after FVP-02 and returns `Blocked` with 2 checks only. FVP-09 (attachment validation scope) produces `Warning` for metadata-only attachment validation — not `Blocked`. FVP-01 and FVP-12 always pass. No Airtable API calls are made. No token is required. No record payload appears in any result field. A `Compliant` result does NOT enable restore writes and does NOT introduce a restore success state — `writesEnabled` is always `false`.

**Gate 12 — write phase ordering policy** is implemented via the `verify_write_phase_ordering_policy_gate` Tauri command. It runs 10 checks (WPO-01 through WPO-10): write gate state, phase list declared, canonical ordering of all declared phases (preflight → schema_create → schema_verify → record_create → record_verify → linked_record_update → linked_record_verify → attachment_metadata_verify → final_validation), prerequisite phases present for all active phases, record_create not active before schema_verify completed, linked_record_update not active before record_verify completed, final_validation not active before linked_record_verify completed, no attachment upload or binary handling phase, attachment_metadata_verify skip with metadata-only reason (produces Warning), and writes-remain-disabled assertion. If no phase list is declared, the function short-circuits after WPO-02 and returns `Blocked` with 2 checks only. WPO-09 (attachment_metadata_verify skip) produces `Warning` — not `Blocked`. WPO-01 and WPO-10 always pass. No Airtable API calls are made. No token is required. No record payload appears in any result field. A `Compliant` result does NOT enable restore writes and does NOT introduce a restore success state — `writesEnabled` is always `false`.

**Gate 13 — failure modes policy** is implemented via the `verify_failure_modes_policy_gate` Tauri command. It runs 11 checks (FMP-01 through FMP-11): write gate state, failure mode handling plans declared, all 10 required failure modes covered (schemaCreateFailure, schemaVerifyFailure, recordCreateFailure, idMappingFailure, linkedRecordUpdateFailure, checkpointPersistenceFailure, rateLimitExhaustion, targetMutationDetected, finalValidationFailure, unknownFailure), no continue-after-failure (all declared stop behaviors must unconditionally stop writes), no destructive rollback, unknown failure stops all writes, rate-limit exhaustion stops after retry limit, final validation failure is never labeled success, checkpoint persistence failure stops writes, no partial failure labeled success, diagnostic context coverage (modes without `capturesDiagnosticContext` produce per-mode `FMP-W-{mode}` Warning checks), and writes-remain-disabled assertion. If no handling plans are declared, the function short-circuits after FMP-02 and returns `Blocked` with 2 checks only. FMP-11 (diagnostic context) produces `Warning` per mode — not `Blocked`. FMP-01 and the writes-remain-disabled check always pass. All four `FailureStopBehavior` variants (`StopAndReport`, `StopPreserveCheckpointAndReport`, `StopAfterRetryLimit`, `BlockAndRequireManualReview`) unconditionally stop writes. No Airtable API calls are made. No token is required. No record payload appears in any result field. A `Compliant` result does NOT enable restore writes and does NOT introduce a restore success state — `writesEnabled` is always `false`.

**Gate 14 — rollback limitation policy** is implemented via the `verify_rollback_limitation_policy_gate` Tauri command. It runs 12 checks (RLP-01 through RLP-12): write gate state, rollback limitation plan declared, no automatic destructive rollback (`rollbackBehavior` must not be `automaticDestructiveRollback`), no automatic delete cleanup (`rollbackBehavior` must not be `automaticDeleteCleanup`), no automatic update/revert cleanup (`rollbackBehavior` must not be `automaticUpdateRevertCleanup`), partial restore is not success (`partialRestoreIsNotSuccess: true`), checkpoint-based recovery guidance (absence or non-checkpoint guidance produces `Warning`), user-visible rollback limitation notice (absence produces `Warning`; notice without limitation details produces `Warning`), manual cleanup requires a separate explicit future action (`manualCleanupRequiresSeparateAction: true`), no token/path/payload exposure (safety invariant — always passes), no network writes attempted (safety invariant — always passes), and writes remain disabled (safety invariant — always passes). If no plan is declared, the function short-circuits after RLP-02 and returns `Blocked` with 2 checks only. RLP-07 and RLP-08 produce `Warning` — not `Blocked` — when guidance or notice is absent or incomplete. RLP-01, RLP-10, RLP-11, and RLP-12 always pass. No Airtable API calls are made. No token is required. No record payload appears in any result field. A `Compliant` result does NOT enable restore writes, does NOT introduce a restore success state, and does NOT trigger any automatic rollback, delete cleanup, or update/revert cleanup — `writesEnabled` is always `false`.

**Gate 15 — final validation enforcement policy** is implemented via the `verify_final_validation_enforcement_policy_gate` Tauri command. It runs 15 checks (FVE-01 through FVE-15): write gate state, plan declared, completion guard fully declared (all three `RestoreCompletionGuard` invariants must be `true`: `blocksCompletionWithoutFinalValidation`, `blocksPartialValidationAsCompletion`, `failedValidationBlocksCompletion`), schema validation state, record count validation state, ID mapping validation before linked record validation (ID mapping must be `Passed` when linked record validation is required), linked record validation state, attachment validation explicit state (metadata-only produces `Warning`; not-required without reason produces `Blocked`), manifest checksum validation if manifest present (skipped automatically when `packageManifestPresent: false`), no partial validation as completion (enforced by completion guard — always passes), failed validation blocks completion (enforced by completion guard — always passes), no unsafe skip (any `Skipped` state produces `Blocked`), no success state without validation (safety invariant — always passes), no token/path/payload exposure (safety invariant — always passes), and writes remain disabled (safety invariant — always passes). If no plan is declared, the function short-circuits after FVE-02 and returns `Blocked` with 2 checks only. A `ValidationCompletionState` of `NotRequired` is acceptable only when accompanied by a non-required reason; without a reason it produces `Blocked`. `Skipped`, `Partial`, and `NotDeclared` always produce `Blocked`. FVE-01, FVE-10, FVE-11, FVE-13, FVE-14, and FVE-15 always pass. No Airtable API calls are made. No token is required. No record payload appears in any result field. A `Compliant` result does NOT enable restore writes, does NOT introduce a restore success state, and does NOT allow any result to be labeled complete or successful before final validation explicitly passes — `writesEnabled` is always `false`.

**Gate 16 — sensitive data safety policy** is implemented via the `verify_sensitive_data_safety_policy_gate` Tauri command. It runs 15 checks (SDS-01 through SDS-15): write gate state, plan declared, all 10 exposure surfaces covered (CommandResult, UiPanel, DiagnosticMessage, CheckpointSummary, ValidationSummary, FailureSummary, LogMessage, ErrorMessage, PackageReference, RecordReference), no token in results, no full filesystem path in results, package references filename only, no record payload in results, no attachment URL in results, no raw HTTP request/response in results, error messages use safe summaries, summaries are payload-free, all redaction rules named (Warning only when unnamed — does not block), no success state introduced (safety invariant — always passes), no token/path/payload in result (safety invariant — always passes), and writes remain disabled (safety invariant — always passes). If no plan is declared, the function short-circuits after SDS-02 and returns `Blocked` with 2 checks only. If the write gate is unexpectedly enabled, SDS-01 returns immediately `Blocked` after 1 check. SDS-12 produces `Warning` only when redaction rules are unnamed — unnamed rules reduce auditability but do not violate safety. All 10 sensitive pattern classes are enforced: `AirtableToken`, `ApiKey`, `BearerToken`, `FullLocalPath`, `PackagePath`, `RecordPayload`, `FieldPayload`, `AttachmentUrl`, `RawHttpResponse`, `RawRequestBody`. SDS-01, SDS-13, SDS-14, and SDS-15 always pass. No Airtable API calls are made. No token is required. No full path, package path, record payload, attachment URL, or raw HTTP data appears in any result field. A `Compliant` result does NOT enable restore writes and does NOT introduce a restore success state — `writesEnabled` is always `false`.

**Gate 17 — attachment phase disabled policy** is implemented via the `verify_attachment_phase_disabled_policy_gate` Tauri command. It runs 16 checks (APD-01 through APD-16): write gate state, plan declared, metadata inspection enabled, metadata verification enabled (Warning when disabled with reason; Blocked when disabled without reason), binary handling disabled (covers BinaryDownload, BinaryUpload, UrlFetch, FileRead, FileWrite, and RawAttachmentTransfer), field mutation disabled, URL exposure disabled, phase not required for completion, final validation treats attachments as metadata-only, no binary attachment operations declared as planned, and no blocked operations required for completion. If no plan is declared, the function short-circuits after APD-02 and returns `Blocked` with 2 checks only. If the write gate is unexpectedly enabled, APD-01 returns immediately `Blocked` after 1 check. APD-04 produces `Warning` when metadata verification is disabled but a skip reason is provided — this reduces auditability but does not violate the binary-blocked invariant. The 10 attachment operation classes are: `MetadataInspect` and `MetadataVerify` (permitted); `BinaryDownload`, `BinaryUpload`, `UrlFetch`, `FileRead`, `FileWrite`, `RawAttachmentTransfer`, `AttachmentFieldMutation`, and `AttachmentUrlExposure` (all blocked). APD-01, APD-13, APD-14, APD-15, and APD-16 always pass when the plan is compliant. No Airtable API calls are made. No token is required. No attachment binary, attachment URL, record payload, or raw HTTP data appears in any result field. A `Compliant` result does NOT enable restore writes, does NOT introduce a restore success state, does NOT download attachments, does NOT upload attachments, and does NOT transfer attachment binaries — `writesEnabled` is always `false`.

**Gate 18 — live write readiness aggregate policy** is implemented via the `verify_live_write_readiness_policy_gate` Tauri command. It runs 10 checks (LWR-01 through LWR-10): write gate state, all 17 required gates declared, no failed required gate, warnings summarized (Warning only — does not block), live execution unavailable, no restore success state in gate notes, no sensitive data in gate notes, no unevaluated required gate, future implementation remains behind disabled gate (safety invariant — always passes), and readiness result is advisory only (safety invariant — always passes). The 17 required gates are: `sandboxEnvironment`, `restoreConfirmation`, `targetEmpty`, `destructiveOperationPolicy`, `attachmentUploadPolicy`, `schemaRecordOrder`, `sandboxWriteTesting`, `liveWriteConfirmation`, `rateLimitBackoff`, `checkpointDurability`, `finalValidationPlan`, `writePhaseOrdering`, `failureModes`, `rollbackLimitation`, `finalValidationEnforcement`, `sensitiveDataSafety`, and `attachmentPhaseDisabled`. If no gates are declared, the function short-circuits after LWR-02 and returns `Blocked` with 2 checks only. If the write gate is unexpectedly enabled, LWR-01 returns immediately `Blocked` after 1 check. LWR-04 produces `Warning` when any required gate has a warning status — warnings summarize risk without blocking. LWR-09 and LWR-10 always pass. No Airtable API calls are made. No token is required. No record payload, attachment URL, or raw HTTP data appears in any result field. A `Ready` result does NOT enable restore writes, does NOT start any restore operation, does NOT introduce a restore success state, and is explicitly labeled advisory only — `writesEnabled` is always `false`.

**Schema write execution preview** is implemented via the `preview_schema_write_execution_gate` Tauri command. It converts declared safety prerequisites into an ordered dry-run step list (validate inputs → create tables → create direct fields → defer linked fields → manual actions → post-schema verification). The preview checks nine prerequisites (SWEP-PRE-01 through SWEP-PRE-09): write gate disabled, sandbox flag present, target empty verified, schema plan ready, destructive policy safe, sensitive data safe, attachment phase disabled, final validation enforcement present, and live write readiness satisfied. If any prerequisite is missing or unsafe, the result is `Blocked` with a single `blocked` step. If all prerequisites are satisfied, the result is `DryRunReady` with ordered `pending` steps. `DryRunReady` does NOT enable live writes, does NOT create any base/table/field, and does NOT start any restore execution. No token is accepted. No Airtable API calls are made. No record payload, attachment URL, or raw HTTP data appears in any result field. `writesEnabled` is always `false`, `noChangesMade` is always `true`, `networkWritesAttempted` is always `false`. Record write execution, linked record second pass, checkpoint execution, final validation execution, and end-to-end restore execution remain pending.

**Record write execution preview** is implemented via the `preview_record_write_execution_gate` Tauri command. It converts declared safety prerequisites into an ordered dry-run batch list (first-pass create batches → second-pass linked-update batches). The preview checks 13 prerequisites (RWEP-PRE-01 through RWEP-PRE-13): write gate disabled, schema execution preview `DryRunReady`, sandbox flag present, target empty verified, record import plan ready, record write request plan ready, batch size ≤ 10, rate-limit/backoff policy safe, checkpoint durability policy safe, sensitive data safe, attachment phase disabled, final validation enforcement present, and live write readiness satisfied. If any prerequisite is missing or unsafe, the result is `Blocked` with a single `blocked` batch. If all prerequisites are satisfied, the result is `DryRunReady` with ordered `pending` batches showing only safe counts and labels — no raw field values, no raw HTTP request/response bodies, no record IDs. Batch size is enforced at ≤ 10 per batch. `DryRunReady` does NOT enable live record writes, does NOT create/update/delete any record, and does NOT start any restore execution. No token is accepted. No Airtable API calls are made. No attachment URL or raw record payload appears in any result field. `writesEnabled` is always `false`, `noChangesMade` is always `true`, `networkWritesAttempted` is always `false`. End-to-end restore execution, second-pass live linked-record updates, checkpoint execution, and final validation execution remain pending.

**Mapping and checkpoint execution preview** is implemented via the `preview_mapping_checkpoint_execution_gate` Tauri command. It converts declared safety prerequisites and batch counts into a deterministic ordered step list that shows checkpoint boundaries across the restore pipeline (schema checkpoint → pre-record-create → per-first-pass-batch ID mapping capture → pre-linked-update → per-second-pass-batch checkpoint → pre-final-validation). The preview checks 8 prerequisites (MCEP-PRE-01 through MCEP-PRE-08): write gate disabled, record write preview `DryRunReady`, checkpoint durability safe, failure modes safe, rollback limitation safe, final validation enforcement present, sensitive data safe, and live write readiness satisfied. If any prerequisite is missing or unsafe, the result is `Blocked`. If all prerequisites are satisfied, the result is `DryRunReady` with `mode = DryRunOnly`. `DryRunReady` does NOT enable live mapping capture, does NOT persist checkpoint files to disk, does NOT call any Airtable endpoint, and does NOT start any restore execution. No token is accepted. No record IDs, field values, attachment URLs, or raw HTTP data appear in any result field. `writesEnabled` is always `false`, `noChangesMade` is always `true`, `networkWritesAttempted` is always `false`. End-to-end restore execution, live checkpoint persistence, and final validation execution remain pending.

**Linked second-pass execution preview** is implemented via the `preview_linked_second_pass_execution_gate` Tauri command. It converts declared safety prerequisites and per-field summaries into a deterministic batch list showing which linked fields require second-pass updates and how many batches they require. The preview checks 8 prerequisites (LSEP-PRE-01 through LSEP-PRE-08): write gate disabled, record write preview `DryRunReady`, mapping/checkpoint preview `DryRunReady`, write phase ordering safe, checkpoint durability safe, sensitive data safe, final validation enforcement present, and live write readiness satisfied. Batch size is enforced at ≤ 10. If any prerequisite is missing or unsafe, the result is `Blocked`. If all prerequisites are satisfied, the result is `DryRunReady` with `mode = DryRunOnly`. Unresolved links are surfaced as a count in the mapping summary — they do not cause `Blocked`. `DryRunReady` does NOT enable live linked record updates, does NOT persist checkpoint files, does NOT call any Airtable endpoint, does NOT return old or new record IDs, and does NOT start any restore execution. No token is accepted. No record IDs, field values, attachment URLs, or raw HTTP data appear in any result field. `writesEnabled` is always `false`, `noChangesMade` is always `true`, `networkWritesAttempted` is always `false`. Live checkpoint persistence, final validation execution, and end-to-end restore execution remain pending.

**Final validation execution preview** is implemented via the `preview_final_validation_execution_gate` Tauri command. It provides a deterministic dry-run preview of the eight ordered final validation checks that would run after all write phases complete: schema/table count (`FVEP-CHK-SCHEMA`), field count (`FVEP-CHK-FIELDS`), record count (`FVEP-CHK-RECORDS`), ID mapping coverage (`FVEP-CHK-MAPPING`), linked record coverage (`FVEP-CHK-LINKED`), attachment metadata only (`FVEP-CHK-ATTACH`), manifest/checksum reference (`FVEP-CHK-MANIFEST` — skipped if no manifest), and final completion guard (`FVEP-CHK-GUARD`). The preview checks 10 prerequisites (FVEP-PRE-01 through FVEP-PRE-10): write gate disabled, schema write preview `DryRunReady`, record write preview `DryRunReady`, mapping/checkpoint preview `DryRunReady`, linked second-pass preview `DryRunReady`, final validation policy safe, final validation enforcement policy safe, sensitive data safe, attachment phase disabled safe, and live write readiness satisfied. If any prerequisite is missing or unsafe, the result is `Blocked`. If all prerequisites are satisfied, the result is `DryRunReady` with `mode = DryRunOnly`. `DryRunReady` does NOT enable live final validation execution, does NOT persist checkpoint files, does NOT call any Airtable endpoint, does NOT return record IDs or field values, and does NOT start any restore execution. No token is accepted. No record IDs, field values, attachment URLs, or raw HTTP data appear in any result field. `writesEnabled` is always `false`, `noChangesMade` is always `true`, `networkWritesAttempted` is always `false`. Live final validation execution and end-to-end restore execution remain pending.

This contract defines what must be true before `evaluate_write_gate()` is ever changed to return an enabled decision.

---

## 1. Sandbox-First Policy

Before any live write path is enabled in a release build, every write phase must be tested against a dedicated sandbox Airtable base that:

- Contains no production data.
- Is empty or newly created for each test run.
- Has its base ID allowlisted in test configuration only — never hardcoded in release code.
- Can be fully deleted after each test run without data loss.

A write operation that has not passed sandbox testing must not be enabled in a release build.

---

## 2. Explicit User Confirmation

Every live write operation requires explicit, unambiguous user confirmation before execution begins:

- The restore execution gate already enforces the phrase `"RESTORE BACKUP"` — this requirement extends to any future execution mode.
- The confirmation text must be displayed to the user in the UI before they type it.
- The confirmation must be checked at the Rust command level, not only in the frontend.
- Partial matches, case-insensitive matches, and trimmed matches must all be rejected.
- The confirmation phrase must be defined as a single constant shared between Rust and TypeScript tests.

---

## 3. Target Base Must Be New or Empty

The target base for any live restore must satisfy one of:

- **New base** — created by AirBridge as the first step of restore, containing no tables or records.
- **Empty existing base** — verified to contain zero tables before any write begins.

Restoring into a base that already contains tables, fields, or records is not supported and must be blocked before any write is attempted. This is enforced by checking the target base's table count via the Airtable schema API before the write engine proceeds.

Rationale: non-empty base restores risk data loss, duplicates, and destructive field-type conflicts that cannot be safely resolved without merge logic that is out of scope for v0.1.

---

## 4. No Destructive Write Operations

The following Airtable API operations are permanently out of scope for the restore write engine:

- **Delete record** — no record may be deleted from any base, table, or view.
- **Delete table** — no table may be deleted from any base.
- **Delete field** — no field may be deleted from any table.
- **Truncate / clear records** — no batch-delete or truncate operation.
- **Overwrite non-empty fields** — when writing to an existing record (which requires merge mode, which is not supported), field values must not be overwritten without explicit merge policy.

If any Airtable API call that deletes or modifies existing data is introduced, it must be gated behind a separate explicit confirmation distinct from the main restore confirmation.

---

## 5. No Overwrite of Existing Records

The restore write engine operates in create-only mode for the initial release:

- Records are created via `POST /v0/{baseId}/{tableId}` only.
- `PATCH` (update), `PUT` (replace), and `DELETE` are not called.
- The target base must be empty before records are created — this is guaranteed by requirement 3.
- The record write request plan builder (`CreateRecordBatch` operations) must never produce an update or delete operation.

---

## 6. Schema Writes Before Record Writes

The write engine must enforce this ordering invariant:

1. All tables created before any fields are created.
2. All directly-creatable fields created before deferred linked fields.
3. All deferred linked fields created before any records are imported.
4. All records created (first pass) before any linked record update pass begins.

Violation of this ordering causes field-not-found or table-not-found errors from the Airtable API that cannot be retried safely. The ordering invariant is already encoded in the schema write foundation phases (CreateTable → CreateField → DeferLinkedField → ManualAction) and the record write foundation phases (CreateRecordBatch → UpdateLinkedRecordBatch).

---

## 7. Record Writes Before Linked Second Pass

The linked record second-pass update pass must only begin after:

- All first-pass `CreateRecordBatch` operations complete successfully.
- The old-to-new record ID mapping is fully populated for every table that has linked record fields.
- Every new record ID in the mapping has been verified to exist in the Airtable base.

The `UpdateLinkedRecordBatch` operations in the record write foundation carry the note "ID mapping unavailable until execution" — this must be replaced with a concrete mapping before any PATCH call is issued.

---

## 8. Checkpoints Before Long Operations

Before any batch operation that would take more than one API call:

- A checkpoint record must be written (to local state, not Airtable) recording which batches completed successfully.
- If the write engine is interrupted, it must be able to resume from the last checkpoint rather than restarting from the beginning.
- Checkpoints are already planned in the record write foundation (`Checkpoint` operations) — their implementation must write durable state before the next batch begins.

---

## 9. Old-to-New Record ID Mapping Safety

When resolving old Airtable record IDs to new ones after first-pass creation:

- The mapping must be built exclusively from records returned by the Airtable create API — never from the backup package's source record IDs.
- The mapping must be complete (every source record ID has a corresponding new ID) before any linked field update begins.
- If a source record ID has no mapping entry (e.g., the record failed to create), the linked field referencing it must be left null or skipped with a warning — never populated with a stale source ID.
- The mapping must never be written to any result, log, or history item.

---

## 10. Rate Limit and Backoff Requirement

The write engine must implement exponential backoff for Airtable API 429 responses:

- Initial backoff: at least 1 000 ms.
- Backoff multiplier: at least 2×.
- Maximum retries: at least 5 before surfacing a permanent failure.
- The retry policy parameters are already defined in `RestoreRetryPolicy` (from the record import planner). The write engine must use them.
- During backoff, no write is attempted. The in-progress batch is held, not abandoned.
- A rate-limit event must be visible in the restore report.

---

## 11. Failure Modes and Stop Conditions

The write engine must stop immediately (not retry) on any of the following:

| Condition | Action |
|-----------|--------|
| Authentication error (401) | Stop; surface token-invalid message; no more writes |
| Permission error (403) | Stop; surface permission-denied message; no more writes |
| Base not found (404 on base) | Stop; surface base-not-found message |
| Table not found (404 on table) | Stop; surface table-not-found; mark table as failed |
| Schema conflict (422 on field create) | Stop field creation step; add to manual-action list |
| Unrecoverable API error (5xx, repeated) | Stop after max retries; surface partial-failure report |
| Local checkpoint write failure | Stop; do not proceed without checkpoint |

For retryable errors (429, transient 5xx), the engine retries with backoff up to the policy maximum. For non-retryable errors, the engine stops the affected phase and records the failure in the restore report. The engine does not attempt further writes in a phase after a non-retryable failure in that phase.

---

## 12. Rollback Limitations

AirBridge does not implement full rollback. The following must be explicitly communicated to the user before execution begins:

- If schema creation partially succeeds (some tables created, some not), the partially-created tables are **not** deleted automatically.
- If record import partially succeeds (some records created, some not), the partially-created records are **not** deleted automatically.
- The restore report identifies which tables, fields, and record batches succeeded and which failed.
- The user must manually clean up a partially-created base if they want to retry a fresh restore.

The UI must show a "Restore cannot be automatically rolled back" notice before the user provides confirmation.

Pre-execution cleanup tooling (detect and offer to delete a partially-created target base) is deferred to a future release.

---

## 13. No Restore Success Until Final Validation Exists

A restore operation must not be reported as succeeded until a `FinalValidation` phase completes. The `FinalValidation` phase must verify:

- Record count in the target base matches the expected count from the backup manifest.
- All tables listed in the backup manifest are present in the target base.
- No error was surfaced by any previous phase.

Until `FinalValidation` is implemented and passing, the write engine result status must remain `Disabled` or `Blocked`. The `Succeeded` status must not be added to `RestoreWriteEngineStatus` until this phase is complete and tested.

---

## 14. Token Safety During Live Writes

When the write engine is enabled, the Airtable token flows through the execution path. The following constraints apply regardless of whether writes are enabled:

- The token must not appear in any `RestoreWriteEngineResult` field.
- The token must not be included in any job history item, log line, or event payload.
- After the write operation completes (success, failure, or partial), the token must be cleared from all in-memory state.
- The `RestoreExecutionRequest` struct must continue to derive `Deserialize` only — it must never derive `Serialize`.

---

## 15. Path Safety During Live Writes

- The full package path must not appear in any result, event, or log entry.
- `Path::file_name()` must be applied before any path value is included in a result struct.
- The target base ID (from the Airtable API) must not be included in public log output at verbose levels.

---

## 16. No Attachment Writes in First Live Phase

Attachment upload is out of scope for the first live write phase. When the write engine is first enabled:

- The `AttachmentHandling` phase must remain disabled.
- All attachment fields in the restore plan must use the `MetadataOnly` policy.
- Attachment URLs from the backup package must not be re-uploaded or re-used.
- The restore report must note that attachment fields require manual re-attachment.

Attachment re-upload is a separate feature that requires its own safety contract and will be enabled in a later phase.

---

## Summary Checklist

Before `evaluate_write_gate()` may return an enabled decision:

- [ ] Sandbox base tested end-to-end for all write phases.
- [ ] User confirmation enforced at Rust level with exact phrase.
- [ ] Target base empty-check implemented before first write.
- [ ] No delete/truncate calls in any write path.
- [ ] Create-only record writes verified.
- [ ] Schema phase ordering enforced (table → field → deferred → record).
- [ ] Linked second pass gated behind complete ID mapping.
- [ ] Checkpoint implementation is durable before each batch.
- [ ] Exponential backoff on 429 implemented and tested.
- [ ] All non-retryable failure modes handled with stop-and-report.
- [ ] Rollback limitation notice shown in UI before confirmation.
- [ ] `FinalValidation` phase implemented and must pass before `Succeeded` status can be set.
- [ ] Token not in any result, log, history, or event during live write path.
- [ ] Path safety enforced throughout live write path.
- [ ] `AttachmentHandling` phase remains disabled in first live phase.

---

## Related Documents

- [Restore Write Engine Skeleton](./restore-write-engine-skeleton.md)
- [Schema Write Engine Foundation](./schema-write-engine-foundation.md)
- [Record Write Engine Foundation](./record-write-engine-foundation.md)
- [Restore Execution Command Contract](./restore-execution-command-contract.md)
- [Live Restore Write Safety Checklist](../qa/live-restore-write-safety-checklist.md)
- [Known Limitations](../release/known-limitations.md)
- [Security Architecture](./security-architecture.md)
