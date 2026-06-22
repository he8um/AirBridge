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

**Restore checkpoint metadata store** is implemented via the `store_restore_checkpoint_metadata` Tauri command. It writes a sanitized JSON checkpoint manifest to an app-controlled local directory (`<os-temp>/airbridge-checkpoints/`). No token, full filesystem path, record payload, old or new record IDs, raw HTTP body, or attachment URL is accepted, stored, or returned. The stored manifest explicitly declares `restoreExecutionNotTriggered: true` and `noSensitiveData: true`. Only a safe filename (no directory component), boundary count, phase count, and item count are returned to the UI. The command checks 5 prerequisites (RCPS-PRE-01 through RCPS-PRE-05): write gate disabled, checkpoint durability policy safe, sensitive data safety policy satisfied, mapping/checkpoint preview `DryRunReady`, and final validation preview `DryRunReady`. If any prerequisite is missing or unsafe, the result is `Blocked` and no file is written. `Stored` does NOT enable live restore execution, does NOT introduce a restore success state, does NOT call any Airtable endpoint, does NOT accept any user-supplied output path, and does NOT write any token, path, record ID, record payload, raw HTTP data, or attachment URL to disk. `writesEnabled` is always `false`, `networkWritesAttempted` is always `false`. `noChangesMade` is `false` only when a local checkpoint metadata file was actually written; it is `true` when blocked. End-to-end restore execution remains pending.

**Schema write executor foundation** (`build_schema_write_executor_plan` in `restore/schema_write_executor.rs`) is an internal Rust module with no Tauri command and no UI surface. It builds an ordered internal step list (tables first, then direct fields, then deferred linked fields, then manual actions) from an existing `SchemaWriteRequestPlan`. It checks 7 prerequisites (SWEX-PRE-01 through SWEX-PRE-07): write gate disabled, mode must be `sandboxOnly`, explicit internal write flag set, sandbox environment verified, target empty verified, live-write readiness satisfied, and request plan not blocked. Since `evaluate_write_gate()` always returns `Disabled`, the result is always `NotExecuted` or `Blocked` — never `DryRunOnly`. No Airtable API calls are made. No token, full path, record payload, raw HTTP body, old/new record IDs, or attachment URL is accepted or returned. `writesEnabled` is always `false`, `noChangesMade` is always `true`, `networkWritesAttempted` is always `false`. No UI execute button exists for this module. Record writes, linked updates, and live final validation reads remain pending.

**Record write executor foundation** (`build_record_write_executor_plan` in `restore/record_write_executor.rs`) is an internal Rust module with no Tauri command and no UI surface. It builds an ordered internal batch plan (first-pass create batches before second-pass linked-update batches, preserving the ordering from the `RecordWriteRequestPlan`) from an existing `RecordWriteRequestPlan`. It checks 9 prerequisites (RWEX-PRE-01 through RWEX-PRE-09): write gate disabled, mode must be `sandboxOnly`, explicit internal write flag set, sandbox environment verified, target empty verified, schema write executor foundation safe, rate-limit/backoff policy compliant, checkpoint metadata store safe, and live-write readiness satisfied. Batch sizes are validated against a maximum of 10. Since `evaluate_write_gate()` always returns `Disabled`, the result is always `NotExecuted` or `Blocked` — never `DryRunOnly`. No Airtable API calls are made. No token, full path, record payload, raw HTTP body, old/new record IDs, or attachment URL is accepted or returned. `writesEnabled` is always `false`, `noChangesMade` is always `true`, `networkWritesAttempted` is always `false`. No UI execute button exists for this module. Linked record ID mapping, live final validation reads, and end-to-end restore execution remain pending.

**Linked second-pass executor foundation** (`build_linked_second_pass_executor_plan` in `restore/linked_second_pass_executor.rs`) is an internal Rust module with no Tauri command and no UI surface. It builds an ordered internal batch plan from per-field summaries (field ordering preserved, batches of at most 10) covering second-pass linked record ID remapping updates. It checks 10 prerequisites (LSEX-PRE-01 through LSEX-PRE-10): write gate disabled, mode must be `sandboxOnly`, explicit internal flag set, sandbox environment verified, target empty verified, record write executor foundation safe, linked second-pass preview `DryRunReady`, mapping/checkpoint preview `DryRunReady`, sensitive data safety satisfied, and live-write readiness satisfied. Unresolved optional links are warning-safe when the linked second-pass preview returned `DryRunReady`. Batch sizes are validated against a maximum of 10. Since `evaluate_write_gate()` always returns `Disabled`, the result is always `NotExecuted` or `Blocked` — never `DryRunOnly`. No Airtable API calls are made. No token, full path, record payload, raw HTTP body, old/new record IDs, or attachment URL is accepted or returned. `writesEnabled` is always `false`, `noChangesMade` is always `true`, `networkWritesAttempted` is always `false`. No UI execute button exists for this module. Live final validation reads and end-to-end restore execution remain pending.

**Final validation reader foundation** (`build_final_validation_reader_plan` in `restore/final_validation_reader.rs`) is an internal Rust module with no Tauri command and no UI surface. It builds an ordered internal check plan of eight typed validation read descriptors (schema/table count, field count, record count, ID mapping coverage, linked field coverage, attachment metadata-only, manifest/checksum reference — skipped when no manifest present, and final completion guard) from declared safe counts in the request. No Airtable API calls are made at any point. It checks 11 prerequisites (FVRD-PRE-01 through FVRD-PRE-11): validation read gate disabled (backed by `evaluate_write_gate()`), mode must be `sandboxOnly`, explicit internal validation read flag set, sandbox environment verified, schema write executor foundation safe, record write executor foundation safe, linked second-pass executor foundation safe, final validation execution preview `DryRunReady`, final validation enforcement policy safe, sensitive data safety satisfied, and attachment phase disabled policy safe. Since `evaluate_write_gate()` always returns `Disabled`, the validation read gate is always disabled, and the result is always `NotExecuted` or `Blocked` — never `DryRunOnly`. The attachment check (`FVRD-CHK-ATTACH`) is metadata-only — no binary retrieval, no attachment URL returned. No raw record IDs appear in any check descriptor. No token, full path, record payload, raw HTTP body, old/new record IDs, or attachment URL is accepted or returned. `readsEnabled` is always `false`, `writesEnabled` is always `false`, `noChangesMade` is always `true`, `networkReadsAttempted` is always `false`, `networkWritesAttempted` is always `false`. No UI execute button exists for this module. Live validation reads and end-to-end restore execution remain pending.

**Restore orchestrator foundation** (`build_restore_orchestrator_plan` in `restore/restore_orchestrator.rs`) is an internal Rust module with no Tauri command and no UI surface. It builds a deterministic eight-phase orchestration plan that sequences all existing executor foundations in order: (1) schema write executor, (2) schema checkpoint boundary, (3) record write executor, (4) record checkpoint boundary, (5) linked second-pass executor, (6) linked phase checkpoint boundary, (7) final validation reader, (8) final guard. No Airtable API calls are made at any point. No checkpoint files are written. It checks 12 prerequisites (ORCH-PRE-01 through ORCH-PRE-12): write gate disabled, mode must be `sandboxOnly`, sandbox environment verified, target empty verified, write phase ordering policy safe, failure modes policy safe, rollback limitation policy safe, live write readiness safe, schema write executor foundation safe, record write executor foundation safe, linked second-pass executor foundation safe, and final validation reader foundation safe. Since `evaluate_write_gate()` always returns `Disabled`, the result is always `NotExecuted` or `Blocked`. No token, full path, record payload, raw HTTP body, old/new record IDs, or attachment URL is accepted or returned. `writesEnabled` is always `false`, `readsEnabled` is always `false`, `noChangesMade` is always `true`, `networkReadsAttempted` is always `false`, `networkWritesAttempted` is always `false`. No UI execute button exists for this module. Future sandbox-only gate enablement, live restore execution, and end-to-end restore remain separate pending work.

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

## 17. Sandbox Gate Contract Foundation

The sandbox gate contract (`evaluate_sandbox_gate_contract` in `restore/sandbox_gate_contract.rs`) evaluates whether all prerequisites for a future sandbox-only gate enablement are present. It enforces the following invariants:

- `evaluate_write_gate()` is called internally (SGC-PRE-12) and always returns `Disabled/DisabledByProductPolicy`. The function is never modified by this module.
- The gate contract never arms the gate and never enables execution. `EligibleButNotArmed` is a diagnostic status only — it does not unlock any execution path.
- No Airtable API calls are made under any status.
- No restore execution, network read, or network write becomes reachable through this module.
- `writesEnabled` is always `false`. `readsEnabled` is always `false`. `noChangesMade` is always `true`. `networkReadsAttempted` is always `false`. `networkWritesAttempted` is always `false`.
- No `Armed`, `Enabled`, `Succeeded`, `Complete`, or `Done` status variant exists.
- No `production` mode variant exists — only `disabled` and `sandboxOnlyCandidate`.
- The result contains no token, no full path, no old/new record IDs, no raw record payload, no raw HTTP body, and no attachment URL.
- Mode `disabled` (default) returns `Disabled` immediately — no prerequisites are evaluated.
- Prerequisites are evaluated in the order SGC-PRE-01 through SGC-PRE-12; the first failing prerequisite blocks the result with its ID in `blocked_reason`.

---

## 18. Sandbox Restore Harness Foundation

The sandbox restore harness (`build_sandbox_restore_harness_plan` in `restore/sandbox_restore_harness.rs`) assembles the gate contract and orchestrator into a dry harness plan. It enforces the following invariants:

- The harness never arms the gate and never enables execution. `ReadyNotExecuted` is a diagnostic status only — it does not unlock any execution path.
- `evaluate_write_gate()` is called indirectly (via the gate contract and orchestrator) and always returns `Disabled/DisabledByProductPolicy`. The function is never modified by this module.
- No Airtable API calls are made under any status.
- No restore execution, network read, or network write becomes reachable through this module.
- `gate_armed` is always `false`. `writesEnabled` is always `false`. `readsEnabled` is always `false`. `noChangesMade` is always `true`. `networkReadsAttempted` is always `false`. `networkWritesAttempted` is always `false`.
- No `Armed`, `Enabled`, `Succeeded`, `Complete`, or `Done` status variant exists.
- No `production` mode variant exists — only `disabled` and `sandboxOnlyDryHarness`.
- The result contains no token, no full path, no old/new record IDs, no raw record payload, no raw HTTP body, and no attachment URL.
- Mode `disabled` (default) returns `NotExecuted` immediately — no evaluation is performed.
- `ReadyNotExecuted` requires: gate contract `eligibleButNotArmed`, orchestrator `notExecuted`, and all executor and checkpoint phases represented. Even then, the gate is NOT armed.
- Live sandbox E2E restore execution remains pending as separate future work.

---

## 19. Sandbox Enablement Readiness Report

The sandbox enablement readiness report (`build_sandbox_enablement_readiness_report` in `restore/sandbox_enablement_readiness.rs`) composes all existing restore foundation modules to produce a deterministic read-only diagnostic. It enforces the following invariants:

- The report never arms the gate and never enables execution. `ReadyButDisabled` is a diagnostic status only — it does not unlock any execution path.
- `evaluate_write_gate()` is called as the first action and must return `Disabled/DisabledByProductPolicy`. If it does not, the report returns `Blocked` immediately.
- No Airtable API calls are made under any status.
- No restore execution, network read, or network write becomes reachable through this module.
- `gate_armed` is always `false`. `writesEnabled` is always `false`. `readsEnabled` is always `false`. `noChangesMade` is always `true`. `networkReadsAttempted` is always `false`. `networkWritesAttempted` is always `false`.
- No `Armed`, `Enabled`, `Succeeded`, `Complete`, or `Done` status variant exists.
- The result contains no token, no full path, no old/new record IDs, no raw record payload, no raw HTTP body, and no attachment URL.
- 13 readiness items (SERN-01 through SERN-13) are evaluated: each foundation module is probed with a minimal disabled-mode request and the result is recorded in the item list and safety snapshot.
- Safety invariant items (SERN-01, SERN-10 through SERN-13) are declared by the report itself; foundation probe items (SERN-02 through SERN-09) require the caller to declare all prerequisite booleans true.
- `ReadyButDisabled` requires all 13 items to be `Ready` or `Warning`. Future sandbox-only gate enablement remains separate pending work.
- The report is not exposed as a Tauri command and has no UI surface.

---

## 20. Sandbox Gate Arming Model

The sandbox gate arming model (`build_sandbox_gate_arming_decision` in `restore/sandbox_gate_arming.rs`) provides an internal Rust-unit-test-only path for building an ephemeral arming decision. It enforces the following invariants:

- The arming decision is ephemeral — it is not stored globally, not persisted between calls, and does not affect runtime behavior.
- The arming decision is not exposed as a Tauri command and has no UI surface. It is only reachable from Rust unit tests.
- `evaluate_write_gate()` is called as a prerequisite check and must return `Disabled/DisabledByProductPolicy`. If it does not, the result is `Blocked` immediately.
- `ArmedNotExecutable` does NOT change `evaluate_write_gate()` behavior — it continues to return `Disabled/DisabledByProductPolicy` after any arming call.
- `ArmedNotExecutable` does NOT unlock any executor, network read path, or network write path.
- `executionEnabled` is always `false`. `writesEnabled` is always `false`. `readsEnabled` is always `false`. `noChangesMade` is always `true`. `networkReadsAttempted` is always `false`. `networkWritesAttempted` is always `false`.
- `gate_armed: true` is present in the returned result only — it describes the arming decision object, not a global armed state.
- No `Enabled`, `Succeeded`, `Complete`, `ExecutionReady`, or `Done` status variant exists.
- The result contains no token, no full path, no old/new record IDs, no raw record payload, no raw HTTP body, and no attachment URL.
- Prerequisites: `explicit_internal_sandbox_arming_requested: true`, mode `sandboxOnlyInternal`, all readiness/contract/harness probes pass.
- Live sandbox E2E restore execution remains separate pending work.

---

## 21. Sandbox Restore Simulator

The sandbox restore simulator (`run_sandbox_restore_simulator` in `restore/sandbox_restore_simulator.rs`) exercises the 8-phase restore sequence in memory only. It enforces the following invariants:

- The simulator never calls the real Airtable client (reads or writes).
- The simulator never writes checkpoint files to disk.
- The simulator never arms the gate globally — `gate_armed` (runtime/global) is always `false`.
- The simulator never enables execution, writes, or reads.
- The simulator never changes `evaluate_write_gate()` behavior.
- The result is not stored globally, not persisted, and not reachable from UI, TypeScript, or any Tauri command.
- `executionEnabled` is always `false`. `writesEnabled` is always `false`. `readsEnabled` is always `false`. `noChangesMade` is always `true`. `networkReadsAttempted` is always `false`. `networkWritesAttempted` is always `false`. `airtableClientCalled` is always `false`. `checkpointFileWritten` is always `false`.
- All 8 phases are represented as in-memory descriptors: schema executor (SRS-PH-01), schema checkpoint (SRS-PH-02, skipped), record executor (SRS-PH-03), record checkpoint (SRS-PH-04, skipped), linked second-pass executor (SRS-PH-05), linked checkpoint (SRS-PH-06, skipped), final validation reader (SRS-PH-07), final guard (SRS-PH-08).
- Phases use `simulated` or `skipped` status — no `succeeded`, `completed`, or `done` status variant exists.
- Prerequisites: `explicit_internal_simulation_requested: true`, mode `sandboxOnlyInternalSimulation`, arming decision `armedNotExecutable`, harness `readyNotExecuted`, orchestrator `notExecuted`.
- The result contains no token, no full path, no old/new record IDs, no raw record payload, no raw HTTP body, and no attachment URL.
- Live sandbox E2E restore execution remains separate pending work.

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

## Section 25 — Sandbox Final Validation Adapter Boundary

The sandbox final validation adapter boundary (`build_sandbox_final_validation_adapter` in `restore/sandbox_final_validation_adapter.rs`) is an internal Rust module — it is not exposed as a Tauri command and has no UI surface. It builds adapter-boundary read descriptors for final validation operations (`schemaCountReadDescriptor`, `fieldCountReadDescriptor`, `recordCountReadDescriptor`, `linkedFieldCoverageReadDescriptor`, `attachmentMetadataReadDescriptor`, `manifestChecksumReadDescriptor` when a manifest is present, and `finalGuardDescriptor`) without calling the real Airtable client, enabling runtime writes, reads, or execution, or persisting any state globally. No Airtable network calls are made. No schema, first-pass record create, linked update, attachment endpoint, or checkpoint operations appear in the adapter output.

The adapter requires: mode is `sandboxOnlyInternal`, `explicit_internal_validation_sandbox_call_requested` is `true`, the write gate returns `Disabled/DisabledByProductPolicy`, the sandbox gate arming decision returns `armedNotExecutable`, the sandbox restore simulator returns `simulatedNotExecuted`, the final validation reader plan returns `notExecuted`, the schema write adapter returns `readyForSandboxCall`, the record write adapter returns `readyForSandboxCall`, the linked second-pass adapter returns `readyForSandboxCall`, `final_validation_enforcement_safe` is true, and `sandbox_verified` is true. First failure in this ordered chain blocks immediately.

`readyForSandboxCall` does NOT execute any Airtable network call. `runtimeExecutionEnabled` is always `false`. `appRuntimeWritesEnabled` is always `false`. `appRuntimeReadsEnabled` is always `false`. `networkReadsAttempted` is always `false`. `networkWritesAttempted` is always `false`. `noChangesMade` is always `true`. `evaluate_write_gate()` continues to return `Disabled/DisabledByProductPolicy` — the adapter boundary does not change this. No token, full path, record payload, raw HTTP body, old/new record IDs, or attachment URL appears in any result field. No `succeeded`, `complete`, `executionReady`, `enabled`, or `done` status exists. Attachment operations are metadata-only descriptors (filename, MIME type, size) — no binary retrieval and no attachment URL is returned.

The adapter provides a `FinalValidationReadAdapter` trait with two test-only implementations: `NoOpFinalValidationReadAdapter` (always zero) and `MockFinalValidationReadAdapter` (configurable count). No production adapter path is implemented. No real Airtable client is wired into any runtime or app flow.

The adapter has three statuses: `notExecuted` (mode is `disabled` — default), `blocked` (any prerequisite missing or flag not set), and `readyForSandboxCall` (all prerequisites pass, internal flag is true). Live end-to-end restore execution remains pending separate work.

---

## Section 24 — Sandbox Linked Second-Pass Adapter Boundary

The sandbox linked second-pass adapter boundary (`build_sandbox_linked_second_pass_adapter` in `restore/sandbox_linked_second_pass_adapter.rs`) is an internal Rust module — it is not exposed as a Tauri command and has no UI surface. It builds adapter-boundary operation descriptors for linked second-pass update batches (`linkedUpdateBatchDescriptor`) only, without calling the real Airtable client, enabling runtime writes or reads, or persisting any state globally. No Airtable network calls are made. No schema, first-pass record create, checkpoint, attachment, or skipped-field operations are accepted.

The adapter requires the sandbox gate arming decision to return `armedNotExecutable`, the sandbox restore simulator to return `simulatedNotExecuted`, the linked second-pass executor plan to return `notExecuted`, the schema write adapter to return `readyForSandboxCall`, and the record write adapter to return `readyForSandboxCall`. Mapping coverage must also be declared sufficient (without exposing record IDs). When all prerequisites pass and the explicit internal flag is set, it returns `readyForSandboxCall` with batch descriptors covering the declared field summaries only.

`readyForSandboxCall` does NOT execute any Airtable network call. `runtimeExecutionEnabled` is always `false`. `appRuntimeWritesEnabled` is always `false`. `appRuntimeReadsEnabled` is always `false`. `networkWritesAttempted` is always `false`. `noChangesMade` is always `true`. `evaluate_write_gate()` continues to return `Disabled/DisabledByProductPolicy` — the adapter boundary does not change this. No token, full path, record payload, raw HTTP body, old/new record IDs, or attachment URL appears in any result field. No `succeeded`, `complete`, `executionReady`, `enabled`, or `done` status exists.

The adapter provides a `LinkedSecondPassAdapter` trait with two test-only implementations: `NoOpLinkedSecondPassAdapter` (always zero) and `MockLinkedSecondPassAdapter` (configurable count). No production adapter path is implemented. No real Airtable client is wired into any runtime or app flow.

The adapter has three statuses: `notExecuted` (mode is `disabled` — default), `blocked` (any prerequisite missing or flag not set), and `readyForSandboxCall` (all prerequisites pass, internal flag is true). Final validation reads, attachment handling, and live end-to-end restore execution remain pending separate work.

---

## Section 23 — Sandbox Record Write Adapter Boundary

The sandbox record write adapter boundary (`build_sandbox_record_write_adapter` in `restore/sandbox_record_write_adapter.rs`) is an internal Rust module — it is not exposed as a Tauri command and has no UI surface. It builds adapter-boundary operation descriptors for first-pass record create operations (createRecordBatchDescriptor) only, without calling the real Airtable client, enabling runtime writes or reads, or persisting any state globally. No Airtable network calls are made. No linked update, schema, checkpoint, attachment, or skipped-field operations are accepted.

The adapter requires the sandbox gate arming decision to return `armedNotExecutable`, the sandbox restore simulator to return `simulatedNotExecuted`, the record write executor plan to return `notExecuted`, and the schema write adapter to return `readyForSandboxCall`. When all prerequisites pass and the explicit internal flag is set, it returns `readyForSandboxCall` with operation descriptors scoped to first-pass record create batches only.

`readyForSandboxCall` does NOT execute any Airtable network call. `runtimeExecutionEnabled` is always `false`. `appRuntimeWritesEnabled` is always `false`. `appRuntimeReadsEnabled` is always `false`. `networkWritesAttempted` is always `false`. `noChangesMade` is always `true`. `evaluate_write_gate()` continues to return `Disabled/DisabledByProductPolicy` — the adapter boundary does not change this. No token, full path, record payload, raw HTTP body, old/new record IDs, or attachment URL appears in any result field. No `succeeded`, `complete`, `executionReady`, `enabled`, or `done` status exists.

The adapter provides a `RecordWriteAdapter` trait with two test-only implementations: `NoOpRecordWriteAdapter` (always zero) and `MockRecordWriteAdapter` (configurable count). No production adapter path is implemented. No real Airtable client is wired into any runtime or app flow.

The adapter has three statuses: `notExecuted` (mode is `disabled` — default), `blocked` (any prerequisite missing or flag not set), and `readyForSandboxCall` (all prerequisites pass, internal flag is true). Linked record updates, schema writes, final validation reads, attachment handling, and live end-to-end restore execution remain pending separate work.

---

## Section 22 — Sandbox Schema Write Adapter Boundary

The sandbox schema write adapter boundary (`build_sandbox_schema_write_adapter` in `restore/sandbox_schema_write_adapter.rs`) is an internal Rust module — it is not exposed as a Tauri command and has no UI surface. It builds adapter-boundary operation descriptors for schema write operations (create table, create field) only, without calling the real Airtable client, enabling runtime writes or reads, or persisting any state globally. No Airtable network calls are made. No record, linked update, or attachment operations are accepted.

The adapter requires the sandbox gate arming decision to return `armedNotExecutable`, the sandbox restore simulator to return `simulatedNotExecuted`, and the schema write executor plan to return `notExecuted`. When all prerequisites pass and the explicit internal flag is set, it returns `readyForSandboxCall` with operation descriptors scoped to schema operations only.

`readyForSandboxCall` does NOT execute any Airtable network call. `runtimeExecutionEnabled` is always `false`. `appRuntimeWritesEnabled` is always `false`. `appRuntimeReadsEnabled` is always `false`. `networkWritesAttempted` is always `false`. `noChangesMade` is always `true`. `evaluate_write_gate()` continues to return `Disabled/DisabledByProductPolicy` — the adapter boundary does not change this. No token, full path, record payload, raw HTTP body, old/new record IDs, or attachment URL appears in any result field. No `succeeded`, `complete`, `executionReady`, `enabled`, or `done` status exists.

The adapter provides a `SchemaWriteAdapter` trait with two test-only implementations: `NoOpSchemaWriteAdapter` (always zero) and `MockSchemaWriteAdapter` (configurable count). No production adapter path is implemented. No real Airtable client is wired into any runtime or app flow.

The adapter has three statuses: `notExecuted` (mode is `disabled` — default), `blocked` (any prerequisite missing or flag not set), and `readyForSandboxCall` (all prerequisites pass, internal flag is true). Record writes, linked record updates, final validation reads, attachment handling, and live end-to-end restore execution remain pending separate work.

---

## Section 26 — Sandbox Adapter Chain Runner (internal, mock/no-op only)

The sandbox adapter chain runner (`run_sandbox_adapter_chain` in `restore/sandbox_adapter_chain_runner.rs`) is an internal Rust module — it is not exposed as a Tauri command and has no UI surface. It composes all four sandbox adapter boundaries in strict order (schema → record → linked → final validation) using mock/no-op adapters only, without calling the real Airtable client, enabling runtime writes, reads, or execution, or persisting any state globally. No Airtable network calls are made. No checkpoint files are written.

The chain runner requires: mode is `mockInternalOnly`, `explicit_internal_mock_chain_requested` is `true`, the write gate returns `Disabled/DisabledByProductPolicy`, the sandbox restore simulator returns `simulatedNotExecuted`, the schema write adapter returns `readyForSandboxCall`, the record write adapter returns `readyForSandboxCall`, the linked second-pass adapter returns `readyForSandboxCall`, and the final validation adapter returns `readyForSandboxCall`. The first failing prerequisite blocks immediately with its check ID (SACR-CHK-01 through SACR-CHK-08).

When all eight prerequisites pass, the runner returns `mockRunNotExecuted` with four phase entries (SACR-PH-01 through SACR-PH-04), each with status `mockObserved` and a safe operation count. The runner reports only safe operation counts per adapter — no raw operation payloads, no record IDs, no token, no path, no raw HTTP body, no attachment URL.

`mockRunNotExecuted` does NOT execute any Airtable network call. `runtimeExecutionEnabled` is always `false`. `appRuntimeWritesEnabled` is always `false`. `appRuntimeReadsEnabled` is always `false`. `networkReadsAttempted` is always `false`. `networkWritesAttempted` is always `false`. `noChangesMade` is always `true`. `airtableClientCalled` is always `false`. `evaluate_write_gate()` continues to return `Disabled/DisabledByProductPolicy` — the chain runner does not change this. No `succeeded`, `complete`, `executionReady`, `enabled`, or `done` status exists. The result is not stored globally and is not reachable from UI, TypeScript, or any Tauri command.

The chain runner has two statuses: `blocked` (default — any prerequisite missing, flag not set, or mode `disabled`) and `mockRunNotExecuted` (all prerequisites pass, internal flag is true). Mode variants are `disabled` (default) and `mockInternalOnly` — no `production` mode exists. Live end-to-end sandbox restore execution remains separate pending work.

---

## Section 27 — Live Schema Write Test Contract (internal, contract-only)

The live schema write test contract (`evaluate_live_schema_write_test_contract` in `restore/live_schema_write_test_contract.rs`) is an internal Rust module — it is not exposed as a Tauri command and has no UI surface. It evaluates whether a future live schema write integration test could be attempted, without performing any Airtable network call, without accepting or persisting any token, and without enabling any runtime execution, writes, or reads. No Airtable API calls are made at any point. No checkpoint files are written. The result is not stored globally.

The contract requires: mode is `sandboxIntegrationCandidate`, `explicit_internal_live_schema_test_contract_requested` is `true`, `evaluate_write_gate()` returns `Disabled/DisabledByProductPolicy`, the sandbox schema write adapter returns `readyForSandboxCall`, the sandbox adapter chain runner returns `mockRunNotExecuted`, the sandbox gate arming decision returns `armedNotExecutable`, the sandbox restore simulator returns `simulatedNotExecuted`, and the sandbox enablement readiness report returns `readyButDisabled`. The first failing prerequisite in this ordered chain (LSWTC-PRE-01 through LSWTC-PRE-08) blocks immediately.

`eligibleButNotExecuted` does NOT perform any Airtable network call. `contract_only` is always `true`. `appRuntimeExecutionEnabled` is always `false`. `appRuntimeWritesEnabled` is always `false`. `appRuntimeReadsEnabled` is always `false`. `networkReadsAttempted` is always `false`. `networkWritesAttempted` is always `false`. `noChangesMade` is always `true`. `airtableClientCalled` is always `false`. `evaluate_write_gate()` continues to return `Disabled/DisabledByProductPolicy` — the contract does not change this. No token, full path, record payload, raw HTTP body, old/new record IDs, or attachment URL appears in any result field. No `succeeded`, `complete`, `executionReady`, `enabled`, or `done` status exists.

The contract reports required future-live conditions without executing them: sandbox-only base required, target base must be empty, explicit test-only credentials required in future task, no UI execution path allowed, only schema operations allowed in the first live phase, record writes remain disabled, linked record updates remain disabled, and final validation reads remain disabled.

The contract has two statuses: `blocked` (default — any prerequisite missing, flag not set, or mode `disabled`) and `eligibleButNotExecuted` (all prerequisites pass, internal flag is true). Mode variants are `disabled` (default) and `sandboxIntegrationCandidate` — no `production` mode exists. The live schema write integration test itself remains separate pending work. Record writes, linked record updates, final validation reads, and live end-to-end restore execution remain pending separate work.

---

## Section 28 — Sandbox Schema Write Integration Harness (test-only, ignored by default)

The sandbox schema write integration harness (`tests/live_schema_write_sandbox.rs`) is a Rust integration test file in the `apps/desktop/src-tauri/tests/` directory. It is `#[ignore]` by default — normal `cargo test` will not run it. It requires explicit opt-in via environment variables and the `--ignored` flag.

**Opt-in environment variables:**

| Variable | Requirement |
|----------|-------------|
| `AIRBRIDGE_ENABLE_LIVE_SCHEMA_WRITE_TEST` | Must be exactly `"true"` |
| `AIRBRIDGE_SANDBOX_AIRTABLE_TOKEN` | Must be non-empty; used for Authorization header only; never printed |
| `AIRBRIDGE_SANDBOX_TARGET_BASE_ID` | Must be non-empty; must be a disposable sandbox base |
| `AIRBRIDGE_SANDBOX_TEST_PREFIX` | Optional; defaults to `"airbridge_sandbox_test"` |

**Safety invariants enforced by the harness:**

- If any required env var is absent, the test exits immediately without a network call.
- `evaluate_write_gate()` is verified to return `Disabled/DisabledByProductPolicy` before and after the live call.
- The live schema write test contract must return `eligibleButNotExecuted` before the live call.
- The sandbox schema write adapter must return `readyForSandboxCall` before the live call.
- Only a single `createTable` Metadata API call is made — schema-only.
- No record endpoints, linked update endpoints, attachment endpoints, or final validation read endpoints are called.
- No records are created, updated, or deleted.
- The token is consumed by the HTTP transport and never returned, printed, or asserted on.
- The base ID is consumed by the API call and never printed or asserted on by value.
- App runtime execution, reads, and writes remain disabled throughout the test.
- No Tauri command is introduced. No TypeScript/UI surface is introduced.
- No raw HTTP body, record ID, raw field values, or attachment URL appears in any assertion.

**Cleanup note:** The harness may leave a test table (`{prefix}_schema_write`) in the sandbox base. Remove it manually after the run. No automatic cleanup path exists. The harness must only be run against a disposable sandbox base.

**What remains pending after this harness:**

- Record writes remain disabled.
- Linked record updates remain disabled.
- Final validation reads remain disabled.
- Attachment handling remains disabled.
- App runtime restore execution remains disabled.
- Live end-to-end restore execution remains pending separate work.
- `evaluate_write_gate()` behavior is unchanged — still returns `Disabled/DisabledByProductPolicy`.

---

## Section 29: Internal Live Record Write Test Contract

**Module:** `restore/live_record_write_test_contract.rs`
**Function:** `evaluate_live_record_write_test_contract`
**Status:** Contract-only — no Airtable network call, no token, no execution

This module is a contract-only readiness layer that evaluates whether a future live record write integration test could be attempted. It does not perform any record write, schema write, network read, or network write.

**10 prerequisites (LRWTC-PRE-01 through LRWTC-PRE-10):**

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

**Safety invariants (always enforced):**

- Does not call the Airtable API (reads or writes).
- Does not accept, store, or return a token.
- Does not enable execution, writes, or reads.
- Does not change `evaluate_write_gate()` behavior.
- Does not write checkpoint files to disk.
- Does not store any state globally.
- Is not reachable from UI, TypeScript, or any Tauri command.
- `contract_only` is always `true`.
- `app_runtime_execution_enabled` is always `false`.
- `app_runtime_writes_enabled` is always `false`.
- `app_runtime_reads_enabled` is always `false`.
- `network_reads_attempted` is always `false`.
- `network_writes_attempted` is always `false`.
- `airtable_client_called` is always `false`.
- `no_changes_made` is always `true`.

**Required future-live conditions (reported, not executed):**

- Disposable sandbox-only base required — no production base may be used.
- Schema phase must already be test-created or safely represented before record writes.
- Explicit test-only credentials required in future task — no token accepted by this contract.
- No UI execution path allowed — live call must be a separate Rust-internal task.
- Only first-pass record create operations allowed — no linked updates.
- Linked record updates remain disabled.
- Final validation reads remain disabled.
- Attachment handling remains disabled.

The live record write integration test itself remains separate pending work.

---

## Section 30: Sandbox Record Write Integration Test Harness

**File:** `apps/desktop/src-tauri/tests/live_record_write_sandbox.rs`
**Status:** Test-only, `#[ignore]` by default — no live call in default `cargo test`

This integration test creates a single test record in an explicitly provided disposable sandbox Airtable table using the Records API. It is not connected to app runtime, UI, TypeScript, or any Tauri command.

**Required environment variables:**

- `AIRBRIDGE_ENABLE_LIVE_RECORD_WRITE_TEST=true`
- `AIRBRIDGE_SANDBOX_AIRTABLE_TOKEN` — personal access token (never printed or asserted on)
- `AIRBRIDGE_SANDBOX_TARGET_BASE_ID` — sandbox base ID (never printed or asserted on)
- `AIRBRIDGE_SANDBOX_TARGET_TABLE_ID_OR_NAME` — sandbox table ID or name (never printed or asserted on)

Optional: `AIRBRIDGE_SANDBOX_TEST_PREFIX`

**Pre-call contract checks (verified before any live call):**

1. `evaluate_write_gate()` returns `Disabled/DisabledByProductPolicy`.
2. `evaluate_live_record_write_test_contract()` returns `EligibleButNotExecuted`.
3. `build_sandbox_record_write_adapter()` returns `ReadyForSandboxCall`.
4. `run_sandbox_adapter_chain()` returns `MockRunNotExecuted`.

**Live call behavior:**

- Calls `create_single_sandbox_record` (Records API POST) with a minimal `Name` string field.
- No linked fields, no attachment endpoints, no final validation reads, no update operations.
- Outcome is sanitized (`CreateSandboxRecordOutcome`): `record_created` boolean, `record_count`, `table_name` — no record ID exposed.

**Post-call invariants:**

- `evaluate_write_gate()` still returns `Disabled` after the live call.
- App runtime execution, reads, and writes remain disabled.
- No state is persisted globally.

**Safety invariants (always enforced):**

- Token, base ID, table ID/name are never printed, asserted on value, or included in test output.
- No record ID is exposed in the sanitized outcome.
- No attachment endpoint is called.
- No linked record update is performed.
- No final validation reads are performed.
- No Tauri command exists for this test.
- No TypeScript/UI surface exists for this test.
- No restore success state is introduced.

**Cleanup note:** The harness may leave a test record in the sandbox table. Delete it manually after the run. No automatic cleanup path exists. The harness must only be run against a disposable sandbox base and table.

**What remains pending after this harness:**

- Linked record updates remain disabled.
- Final validation reads remain disabled.
- Attachment handling remains disabled.
- App runtime restore execution remains disabled.
- Live end-to-end restore execution remains pending separate work.
- `evaluate_write_gate()` behavior is unchanged — still returns `Disabled/DisabledByProductPolicy`.

---

## Section 31: Live Linked Update Test Contract

**File:** `apps/desktop/src-tauri/src/restore/live_linked_update_test_contract.rs`
**Status:** Contract-only — no live call, no network access, no Airtable API call

This module evaluates whether a future live linked record update integration test could be attempted, without performing any live call. It is not connected to app runtime, UI, TypeScript, or any Tauri command.

**Prerequisites (LLUTC-PRE-01 through LLUTC-PRE-11):**

| ID | Probe |
|----|-------|
| LLUTC-PRE-01 | Mode is `SandboxIntegrationCandidate` |
| LLUTC-PRE-02 | `explicit_internal_live_linked_update_test_contract_requested` is `true` |
| LLUTC-PRE-03 | `evaluate_write_gate()` returns `Disabled/DisabledByProductPolicy` |
| LLUTC-PRE-04 | `evaluate_live_record_write_test_contract()` returns `EligibleButNotExecuted` |
| LLUTC-PRE-05 | `build_sandbox_linked_second_pass_adapter()` returns `ReadyForSandboxCall` |
| LLUTC-PRE-06 | `build_sandbox_record_write_adapter()` returns `ReadyForSandboxCall` |
| LLUTC-PRE-07 | `build_sandbox_schema_write_adapter()` returns `ReadyForSandboxCall` |
| LLUTC-PRE-08 | `run_sandbox_adapter_chain()` returns `MockRunNotExecuted` |
| LLUTC-PRE-09 | `build_sandbox_gate_arming_decision()` returns `ArmedNotExecutable` |
| LLUTC-PRE-10 | `run_sandbox_restore_simulator()` returns `SimulatedNotExecuted` |
| LLUTC-PRE-11 | `build_sandbox_enablement_readiness_report()` returns `ReadyButDisabled` |

**Safety invariants (always enforced):**

- `contract_only` is always `true`.
- `app_runtime_execution_enabled` is always `false`.
- `app_runtime_writes_enabled` is always `false`.
- `app_runtime_reads_enabled` is always `false`.
- `network_reads_attempted` is always `false`.
- `network_writes_attempted` is always `false`.
- `airtable_client_called` is always `false`.
- `no_changes_made` is always `true`.
- `evaluate_write_gate()` is never modified — always returns `Disabled/DisabledByProductPolicy`.
- No token field, no absolute path, no record payload, no raw HTTP body, no old/new record IDs, no attachment URL in the result.
- No `succeeded`, `complete`, `enabled`, `done`, or `executionReady` status variant exists.
- No UI path, no Tauri command, no TypeScript surface exists.
- The live linked update integration test itself remains separate pending work.

**Required future-live conditions (reported, not executed):**

- Disposable sandbox-only base required — no production base may be used.
- Live schema and live record harnesses must have prepared sandbox-only records before linked updates.
- Explicit test-only credentials required in future task — no token accepted by this contract.
- No UI execution path allowed — live call must be a separate Rust-internal task.
- Only linked update operations allowed — no schema or first-pass record create operations.
- Attachment handling remains disabled — must not be enabled in this task.
- Final validation reads remain disabled — must not be enabled in this task.

**What remains pending after this contract:**

- Live linked record update integration test remains separate pending work.
- Final validation reads remain disabled.
- Attachment handling remains disabled.
- App runtime restore execution remains disabled.
- Live end-to-end restore execution remains pending separate work.
- `evaluate_write_gate()` behavior is unchanged — still returns `Disabled/DisabledByProductPolicy`.

---

## Section 32: Live Final Validation Test Contract

**File:** `apps/desktop/src-tauri/src/restore/live_final_validation_test_contract.rs`
**Status:** Contract-only — no live call, no network access, no Airtable API call

This module evaluates whether a future live final validation read integration test could be attempted, without performing any live call. It is not connected to app runtime, UI, TypeScript, or any Tauri command.

**Prerequisites (LFVTC-PRE-01 through LFVTC-PRE-12):**

| ID | Probe |
|----|-------|
| LFVTC-PRE-01 | Mode is `SandboxIntegrationCandidate` |
| LFVTC-PRE-02 | `explicit_internal_live_final_validation_test_contract_requested` is `true` |
| LFVTC-PRE-03 | `evaluate_write_gate()` returns `Disabled/DisabledByProductPolicy` |
| LFVTC-PRE-04 | `evaluate_live_linked_update_test_contract()` returns `EligibleButNotExecuted` |
| LFVTC-PRE-05 | `build_sandbox_final_validation_adapter()` returns `ReadyForSandboxCall` |
| LFVTC-PRE-06 | `build_sandbox_linked_second_pass_adapter()` returns `ReadyForSandboxCall` |
| LFVTC-PRE-07 | `build_sandbox_record_write_adapter()` returns `ReadyForSandboxCall` |
| LFVTC-PRE-08 | `build_sandbox_schema_write_adapter()` returns `ReadyForSandboxCall` |
| LFVTC-PRE-09 | `run_sandbox_adapter_chain()` returns `MockRunNotExecuted` |
| LFVTC-PRE-10 | `build_sandbox_gate_arming_decision()` returns `ArmedNotExecutable` |
| LFVTC-PRE-11 | `run_sandbox_restore_simulator()` returns `SimulatedNotExecuted` |
| LFVTC-PRE-12 | `build_sandbox_enablement_readiness_report()` returns `ReadyButDisabled` |

**Safety invariants (always enforced):**

- `contract_only` is always `true`.
- `app_runtime_execution_enabled` is always `false`.
- `app_runtime_writes_enabled` is always `false`.
- `app_runtime_reads_enabled` is always `false`.
- `network_reads_attempted` is always `false`.
- `network_writes_attempted` is always `false`.
- `airtable_client_called` is always `false`.
- `no_changes_made` is always `true`.
- `evaluate_write_gate()` is never modified — always returns `Disabled/DisabledByProductPolicy`.
- No token field, no absolute path, no record payload, no raw HTTP body, no old/new record IDs, no attachment URL in the result.
- No `succeeded`, `complete`, `enabled`, `done`, or `executionReady` status variant exists.
- No UI path, no Tauri command, no TypeScript surface exists.
- The live final validation read integration test itself remains separate pending work.

**Required future-live conditions (reported, not executed):**

- Disposable sandbox-only base required — no production base may be used.
- Schema, record, and linked update test harnesses must have prepared sandbox-only state before final validation reads.
- Explicit test-only credentials required in future task — no token accepted by this contract.
- No UI execution path allowed — live call must be a separate Rust-internal task.
- Only validation read operations allowed — no schema write, first-pass record create, linked update, or attachment binary operations.
- Attachment binary handling remains disabled — must not be enabled in this task.
- App runtime restore execution remains disabled — must not be enabled in this task.

**What remains pending after this contract:**

- Live final validation read integration test remains separate pending work.
- Attachment binary handling remains disabled.
- App runtime restore execution remains disabled.
- Live end-to-end restore execution remains pending separate work.
- `evaluate_write_gate()` behavior is unchanged — still returns `Disabled/DisabledByProductPolicy`.

---

## Section 33: Sandbox Linked Update Integration Test Harness

**File:** `apps/desktop/src-tauri/tests/live_linked_update_sandbox.rs`
**Type:** Opt-in Rust integration test (`#[ignore]` by default)
**Added:** task `test: add live linked update sandbox harness`

### Purpose

Provides a live integration test for the two-step linked record update flow using a disposable sandbox Airtable base. The harness is fully blocked from running in standard `cargo test` and requires explicit opt-in via six environment variables.

### Required environment variables

| Env Var | Required | Purpose |
|---------|----------|---------|
| `AIRBRIDGE_ENABLE_LIVE_LINKED_UPDATE_TEST` | Yes (`true`) | Master opt-in flag |
| `AIRBRIDGE_SANDBOX_AIRTABLE_TOKEN` | Yes | PAT for sandbox base |
| `AIRBRIDGE_SANDBOX_TARGET_BASE_ID` | Yes | Sandbox base ID |
| `AIRBRIDGE_SANDBOX_LINK_SOURCE_TABLE_ID_OR_NAME` | Yes | Source table (has linked field) |
| `AIRBRIDGE_SANDBOX_LINK_TARGET_TABLE_ID_OR_NAME` | Yes | Target table (linked-to) |
| `AIRBRIDGE_SANDBOX_LINK_FIELD_NAME` | Yes | Linked field name in source table |
| `AIRBRIDGE_SANDBOX_TEST_PREFIX` | No | Prefix for test field values |

### Pre-call contract gating

Before any live Airtable call is made, the `#[ignore]` test verifies:

1. `evaluate_write_gate()` returns `Disabled`.
2. `evaluate_live_linked_update_test_contract()` returns `EligibleButNotExecuted`.
3. `build_sandbox_linked_second_pass_adapter()` returns `ReadyForSandboxCall`.
4. `build_sandbox_record_write_adapter()` returns `ReadyForSandboxCall`.
5. `build_sandbox_schema_write_adapter()` returns `ReadyForSandboxCall`.
6. `run_sandbox_adapter_chain()` returns `MockRunNotExecuted`.

### Live calls (only when all env vars set + `--ignored` flag)

1. `POST` target table — create one minimal record; ID extracted as local opaque handle.
2. `POST` source table — create one minimal record; ID extracted as local opaque handle.
3. `PATCH` source table — set linked field to point at target record ID.

The IDs are used only as opaque handles for the PATCH call. They are never printed, asserted on by value, serialized into outcome structs, or included in post-call assertions.

### Post-call assertions

- `outcome.record_updated` is `true`.
- `outcome.record_count == 1`.
- `outcome.linked_target_count == 1`.
- `outcome.source_table_name` is non-empty.
- Serialized outcome JSON contains no `pat_` (token) or record ID patterns.
- `evaluate_write_gate()` still returns `Disabled`.

### Safety invariants

- Token, base ID, table IDs, field name, and record IDs are never printed or included in any outcome.
- `UpdateLinkedSandboxRecordOutcome` exposes only boolean/count/name fields.
- `evaluate_write_gate()` returns `Disabled` before and after live calls — unchanged.
- App runtime execution, reads, and writes remain disabled.
- No schema writes, no attachment endpoints, no final validation reads, no record deletes.
- No Tauri command added. No TypeScript/UI surface.
- Test may leave sandbox-only records; must only be run against disposable base/tables.
- No automatic cleanup path.

### New models (added to `src/airtable/models.rs`)

**`UpdateLinkedSandboxRecordRequest`** — sandbox-only PATCH request. Contains `source_record_id` (opaque), `linked_field_name`, `target_record_ids`. No token field.

**`UpdateLinkedSandboxRecordOutcome`** — sanitized result. Contains `record_updated: bool`, `record_count: usize`, `source_table_name: String`, `linked_field_name: String`, `linked_target_count: usize`. No record IDs, no token, no raw HTTP.

### New client method (added to `src/airtable/client.rs`)

**`update_single_linked_sandbox_record(base_id, source_table, source_table_name, request)`** — issues one PATCH to the Records API. Constructs the `[{"id": "recXXX"}]` linked field wire format internally. Returns `UpdateLinkedSandboxRecordOutcome`. Covered by 5 unit tests.

---

## Section 34: Sandbox Final Validation Read Integration Test Harness

**File:** `apps/desktop/src-tauri/tests/live_final_validation_sandbox.rs`
**Type:** Opt-in Rust integration test (`#[ignore]` by default)
**Added:** task `test: add live final validation sandbox harness`

### Purpose

Provides a live integration test for the final validation read flow using an accessible sandbox Airtable table. The harness is fully blocked from running in standard `cargo test` and requires explicit opt-in via four environment variables.

### Required environment variables

| Env Var | Required | Purpose |
|---------|----------|---------|
| `AIRBRIDGE_ENABLE_LIVE_FINAL_VALIDATION_TEST` | Yes (`true`) | Master opt-in flag |
| `AIRBRIDGE_SANDBOX_AIRTABLE_TOKEN` | Yes | PAT for sandbox base |
| `AIRBRIDGE_SANDBOX_TARGET_BASE_ID` | Yes | Sandbox base ID |
| `AIRBRIDGE_SANDBOX_VALIDATION_TABLE_ID_OR_NAME` | Yes | Table to read from |
| `AIRBRIDGE_SANDBOX_EXPECTED_MIN_RECORD_COUNT` | No | Minimum record count to assert |

### Pre-call contract gating

Before any live Airtable call is made, the `#[ignore]` test verifies:

1. `evaluate_write_gate()` returns `Disabled`.
2. `evaluate_live_final_validation_test_contract()` returns `EligibleButNotExecuted`.
3. `build_final_validation_reader_plan()` returns `NotExecuted`.
4. `build_sandbox_final_validation_adapter()` returns `ReadyForSandboxCall`.
5. `build_sandbox_linked_second_pass_adapter()` returns `ReadyForSandboxCall`.
6. `build_sandbox_record_write_adapter()` returns `ReadyForSandboxCall`.
7. `build_sandbox_schema_write_adapter()` returns `ReadyForSandboxCall`.
8. `run_sandbox_adapter_chain()` returns `MockRunNotExecuted`.

### Live call (only when all env vars set + `--ignored` flag)

1. `GET` records endpoint for the validation table — first page only (read-only).

No records are created, updated, or deleted. No schema writes. No linked updates. No attachment endpoints.

### Post-call assertions

- `outcome.table_reachable` is `true`.
- If `AIRBRIDGE_SANDBOX_EXPECTED_MIN_RECORD_COUNT` is set: `outcome.min_count_satisfied` is `true`.
- Serialized outcome JSON contains no `pat_` (token) or `rec` (record ID) patterns.
- `evaluate_write_gate()` still returns `Disabled`.
- `build_final_validation_reader_plan()` still returns `NotExecuted` — gate unchanged.

### Safety invariants

- Token, base ID, table ID, and record IDs are never printed or included in any outcome.
- `SandboxValidationReadOutcome` exposes only boolean/count fields — no record IDs, no raw field values.
- `evaluate_write_gate()` returns `Disabled` before and after — unchanged.
- The validation reader plan remains `NotExecuted` — read gate is unchanged.
- App runtime execution, reads, and writes remain disabled.
- No records created, updated, or deleted.
- No schema writes, no linked updates, no attachment endpoints.
- No Tauri command added. No TypeScript/UI surface.
- Safe against any accessible sandbox table (read-only).

### New model (added to `src/airtable/models.rs`)

**`SandboxValidationReadOutcome`** — sanitized result. Contains `table_reachable: bool`, `observed_record_count: usize`, `min_count_satisfied: bool`, `has_records: bool`. No record IDs, no raw field values, no token, no raw HTTP.

### New client method (added to `src/airtable/client.rs`)

**`list_sandbox_records_for_validation(base_id, table_id_or_name, expected_min_count)`** — issues one GET to the Records API (first page, minimal page size). Returns `SandboxValidationReadOutcome`. Covered by 6 unit tests.

---

## Section 35: Live E2E Restore Test Contract

**Module:** `apps/desktop/src-tauri/src/restore/live_e2e_restore_test_contract.rs`  
**Public function:** `evaluate_live_e2e_restore_test_contract(request, schema_plan, record_plan)`  
**Status:** Contract-only. No Airtable call. No live execution.

### Purpose

Evaluates whether a future live E2E sandbox restore integration harness could be attempted — without performing any live call. This is the top-level contract in the safety chain: it verifies all sub-contracts (schema write, record write, linked update, final validation) and all supporting probes (adapter chain, gate arming, simulator, enablement readiness, restore harness) before reporting `EligibleButNotExecuted`.

### Prerequisites (9 total)

| ID | Prerequisite |
|----|-------------|
| LE2ERTC-PRE-01 | Mode is `sandboxIntegrationCandidate` |
| LE2ERTC-PRE-02 | `explicit_internal_live_e2e_restore_test_contract_requested` is `true` |
| LE2ERTC-PRE-03 | `evaluate_write_gate()` returns `Disabled/DisabledByProductPolicy` |
| LE2ERTC-PRE-04 | Live final validation test contract returns `EligibleButNotExecuted` |
| LE2ERTC-PRE-05 | Sandbox adapter chain runner returns `MockRunNotExecuted` |
| LE2ERTC-PRE-06 | Sandbox gate arming decision returns `ArmedNotExecutable` |
| LE2ERTC-PRE-07 | Sandbox restore simulator returns `SimulatedNotExecuted` |
| LE2ERTC-PRE-08 | Sandbox enablement readiness returns `ReadyButDisabled` |
| LE2ERTC-PRE-09 | Sandbox restore harness returns `ReadyNotExecuted` |

### Planned E2E phases (reported, not executed)

| Phase ID | Label |
|----------|-------|
| LE2ERTC-PHASE-01 | Schema write (sandbox) |
| LE2ERTC-PHASE-02 | Record write (sandbox) |
| LE2ERTC-PHASE-03 | Linked field update (sandbox) |
| LE2ERTC-PHASE-04 | Final validation read (sandbox) |
| LE2ERTC-PHASE-05 | Final non-success guard |

### Safety invariants

- `contract_only` is always `true`.
- `airtable_client_called` is always `false`.
- `network_reads_attempted` is always `false`.
- `network_writes_attempted` is always `false`.
- `no_changes_made` is always `true`.
- `app_runtime_execution_enabled` is always `false`.
- `app_runtime_writes_enabled` is always `false`.
- `app_runtime_reads_enabled` is always `false`.
- `evaluate_write_gate()` behavior is never modified.
- No token field, no path field, no record payload, no raw HTTP body, no record IDs, no attachment URL.
- No `Succeeded`, `Complete`, `Enabled`, `Done`, or `ExecutionReady` status exists.
- Not reachable from UI, TypeScript, or any Tauri command.
### Sub-contract chain verified

`LiveE2ERestoreTestContract` → verifies `LiveFinalValidationTestContract` → verifies `LiveLinkedUpdateTestContract` → verifies `LiveRecordWriteTestContract` → verifies `LiveSchemaWriteTestContract`.

---

## Section 36: Live E2E Restore Sandbox Integration Harness (test-only, ignored by default)

**File:** `apps/desktop/src-tauri/tests/live_e2e_restore_sandbox.rs`  
**Status:** Test-only. `#[ignore]` by default. No app runtime path. No Tauri command.

### Purpose

Sequences all five restore phases in a single opt-in integration test against a disposable sandbox base. Combines the four independently-verified phase harnesses with a final non-runtime guard phase.

### Required environment variables (9)

| Env Var | Purpose |
|---------|---------|
| `AIRBRIDGE_ENABLE_LIVE_E2E_RESTORE_TEST` | Must be exactly `true` |
| `AIRBRIDGE_SANDBOX_AIRTABLE_TOKEN` | Personal access token |
| `AIRBRIDGE_SANDBOX_TARGET_BASE_ID` | Sandbox base ID |
| `AIRBRIDGE_SANDBOX_SCHEMA_TABLE_NAME` | Name for the new table (phase 1) |
| `AIRBRIDGE_SANDBOX_RECORD_TABLE_ID_OR_NAME` | Table for record writes (phase 2) |
| `AIRBRIDGE_SANDBOX_LINK_SOURCE_TABLE_ID_OR_NAME` | Source table for linked update (phase 3) |
| `AIRBRIDGE_SANDBOX_LINK_TARGET_TABLE_ID_OR_NAME` | Target table for linked update (phase 3) |
| `AIRBRIDGE_SANDBOX_LINK_FIELD_NAME` | Linked field name in source table (phase 3) |
| `AIRBRIDGE_SANDBOX_VALIDATION_TABLE_ID_OR_NAME` | Table for validation read (phase 4) |

Optional: `AIRBRIDGE_SANDBOX_EXPECTED_MIN_RECORD_COUNT`, `AIRBRIDGE_SANDBOX_TEST_PREFIX`.

### Phase sequence

| Phase | Operation | API Call |
|-------|-----------|----------|
| 1 — Schema write | Create table in sandbox base | `POST` createTable |
| 2 — Record write | Create single record | `POST` records |
| 3 — Linked update | Create target + source records; PATCH linked field | `POST` records (×2), `PATCH` records |
| 4 — Final validation read | List records (read-only, first page) | `GET` records |
| 5 — Final non-runtime guard | Verify write gate + app runtime state | No network call |

### Pre-call gating (before each live call)

- Top-level E2E contract verified: `EligibleButNotExecuted`.
- Phase-specific contract verified: `EligibleButNotExecuted`.
- `evaluate_write_gate()` verified: `Disabled`.

### Post-call gating (after each phase)

- `evaluate_write_gate()` verified still `Disabled`.

### Safety invariants

- Token, base ID, table IDs/names, field names, and record IDs are never printed, asserted on by value, or included in any serialized result.
- Record IDs used in phase 3 PATCH are held as local `String` variables only, used once, then dropped.
- All outcome structs contain only boolean/count/name fields.
- App runtime execution, reads, and writes remain disabled throughout.
- No attachment endpoints. No record deletes. No schema deletes.
- Must only be run against a disposable sandbox base. No automatic cleanup.
- Not reachable from UI, TypeScript, or any Tauri command.
- `evaluate_write_gate()` always returns `Disabled` — never modified.

### Test count

- 19 default non-ignored tests (run in standard `cargo test`)
- 1 `#[ignore]` live test (requires all 9 env vars + `--ignored` flag)

---

## Section 37: Post-E2E Restore Readiness Audit (Rust-internal, no network calls)

**Module:** `apps/desktop/src-tauri/src/restore/post_e2e_restore_readiness_audit.rs`
**Public function:** `audit_post_e2e_restore_readiness(request, schema_plan, record_plan)`
**Status:** Rust-internal only. No Tauri command. No network call. No UI surface.

### Purpose

Produces a sanitized internal readiness report by inspecting and composing existing contract and harness readiness concepts. Does not execute any harness. Does not call Airtable. Does not accept credential values. Intended for maintainer use and Rust unit tests only.

### Required inputs (audit gates)

All of the following must be `true` for `SandboxHarnessesReadyRuntimeDisabled` to be returned:

| Gate | Input |
|------|-------|
| Explicit audit flag | `explicit_internal_post_e2e_audit_requested == true` |
| Write gate | `evaluate_write_gate()` returns `Disabled` |
| E2E contract | Returns `EligibleButNotExecuted` |
| Schema harness | `schema_harness_ignored_by_default == true` |
| Record harness | `record_harness_ignored_by_default == true` |
| Linked update harness | `linked_update_harness_ignored_by_default == true` |
| Final validation harness | `final_validation_harness_ignored_by_default == true` |
| E2E restore harness | `e2e_restore_harness_ignored_by_default == true` |
| Restore command | `restore_command_does_not_expose_live_execution == true` |
| No Tauri command | `no_tauri_command_exposes_live_execution == true` |
| No TS/UI path | `no_typescript_ui_path_exposes_live_execution == true` |

### Audit items (12 total)

| Item ID | Label |
|---------|-------|
| PERRA-ITEM-01 | Schema write sandbox harness is `#[ignore]` by default |
| PERRA-ITEM-02 | Record write sandbox harness is `#[ignore]` by default |
| PERRA-ITEM-03 | Linked update sandbox harness is `#[ignore]` by default |
| PERRA-ITEM-04 | Final validation sandbox harness is `#[ignore]` by default |
| PERRA-ITEM-05 | E2E restore sandbox harness is `#[ignore]` by default |
| PERRA-ITEM-06 | E2E restore test contract returns `EligibleButNotExecuted` |
| PERRA-ITEM-07 | `evaluate_write_gate()` returns `Disabled/DisabledByProductPolicy` |
| PERRA-ITEM-08 | `commands/restore.rs` does not expose live restore execution |
| PERRA-ITEM-09 | No Tauri command exposes live restore execution |
| PERRA-ITEM-10 | No TypeScript/UI path exposes live restore execution |
| PERRA-ITEM-11 | Attachment binary handling remains disabled |
| PERRA-ITEM-12 | App runtime execution/writes/reads remain disabled |

### Pending work items (7 total)

| Pending ID | Label |
|------------|-------|
| PERRA-PENDING-01 | Product decision for runtime restore enablement |
| PERRA-PENDING-02 | Runtime restore command contract (if ever approved) |
| PERRA-PENDING-03 | UI review/confirmation design (if ever approved) |
| PERRA-PENDING-04 | Credential handling review for any future runtime path |
| PERRA-PENDING-05 | Attachment binary handling (remains disabled) |
| PERRA-PENDING-06 | Cleanup strategy for sandbox-created tables/records |
| PERRA-PENDING-07 | Security review before any user-facing restore execution |

### Safety invariants

- `app_runtime_execution_enabled`, `app_runtime_writes_enabled`, `app_runtime_reads_enabled` always `false`.
- `live_harnesses_ignored_by_default`, `no_changes_made` always `true`.
- `network_reads_attempted`, `network_writes_attempted`, `airtable_client_called` always `false`.
- `tauri_command_exposes_live_restore`, `typescript_ui_path_exposes_live_restore` always `false`.
- No token, base ID, table/field/record values accepted.
- `evaluate_write_gate()` never modified.
- No restore success/completed/enabled product state exists.
- Not reachable from UI, TypeScript, or any Tauri command.
- **`SandboxHarnessesReadyRuntimeDisabled` does NOT mean product-level restore is complete or approved.**

### Test count

- 28 unit tests in `#[cfg(test)]` block

---

## Section 38: Restore Release Readiness Snapshot (Rust-internal, no network calls)

**Module:** `apps/desktop/src-tauri/src/restore/restore_release_readiness_snapshot.rs`
**Public function:** `build_restore_release_readiness_snapshot(request, schema_plan, record_plan)`
**Status:** Rust-internal only. No Tauri command. No network call. No UI surface.

### Purpose

Produces a sanitized, structured release-readiness snapshot for maintainers by composing the post-E2E restore readiness audit and write gate results. It reports 12 distinct readiness areas, 7 pending work items, and 3 maintainer recommendations. It does not execute any harness, does not call Airtable, and does not accept credential values.

### Return status

`AlphaReadyRestoreRuntimeDisabled` is returned only when ALL of the following are true:

| Condition | Input |
|-----------|-------|
| Explicit snapshot flag | `explicit_internal_restore_release_snapshot_requested == true` |
| Write gate | `evaluate_write_gate()` returns `Disabled/DisabledByProductPolicy` |
| No Tauri live execution | `no_tauri_command_exposes_live_execution == true` |
| No TS/UI live execution | `no_typescript_ui_path_exposes_live_execution == true` |
| Restore command safe | `restore_command_does_not_expose_live_execution == true` |
| Post-E2E audit | `audit_post_e2e_restore_readiness()` returns `SandboxHarnessesReadyRuntimeDisabled` |

`AlphaReadyRestoreRuntimeDisabled` does **NOT** mean user-facing restore execution is complete or approved for production use.

### 12 readiness areas

| Area ID | Name | Default status |
|---------|------|----------------|
| RRRS-AREA-01 | Backup package creation | `Ready` when `backup_package_creation_ready` |
| RRRS-AREA-02 | Package inspection | `Ready` when `package_inspection_ready` |
| RRRS-AREA-03 | Restore dry-run planning | `Ready` when `restore_dry_run_planning_ready` |
| RRRS-AREA-04 | Restore execution preview | `Ready` when `restore_execution_preview_ready` |
| RRRS-AREA-05 | Sandbox schema write harness | `Ready` when `schema_harness_ignored_by_default` |
| RRRS-AREA-06 | Sandbox record write harness | `Ready` when `record_harness_ignored_by_default` |
| RRRS-AREA-07 | Sandbox linked update harness | `Ready` when `linked_update_harness_ignored_by_default` |
| RRRS-AREA-08 | Sandbox final validation read harness | `Ready` when `final_validation_harness_ignored_by_default` |
| RRRS-AREA-09 | Sandbox E2E restore harness | `Ready` when `e2e_restore_harness_ignored_by_default` |
| RRRS-AREA-10 | Runtime restore execution | Always `Disabled` |
| RRRS-AREA-11 | Attachment handling | Always `Disabled` |
| RRRS-AREA-12 | Product/security approval | Always `PendingApproval` |

### 7 pending work items

| Pending ID | Label |
|------------|-------|
| RRRS-PENDING-01 | Product/security decision for runtime restore enablement |
| RRRS-PENDING-02 | Runtime restore command contract (if ever approved) |
| RRRS-PENDING-03 | UI confirmation and failure-state design (if ever approved) |
| RRRS-PENDING-04 | Credential handling review for future runtime restore |
| RRRS-PENDING-05 | Cleanup strategy for sandbox-created tables and records |
| RRRS-PENDING-06 | Attachment binary handling (remains disabled) |
| RRRS-PENDING-07 | User-facing restore documentation |

### 3 recommendations

| Rec ID | Label |
|--------|-------|
| RRRS-REC-01 | Do not ship user-facing restore execution without product/security approval |
| RRRS-REC-02 | Run sandbox E2E harness against a disposable base before any approval review |
| RRRS-REC-03 | Address all pending work items before requesting approval |

### Safety invariants

- `app_runtime_execution_enabled`, `app_runtime_writes_enabled`, `app_runtime_reads_enabled` always `false`.
- `live_harnesses_ignored_by_default`, `no_changes_made` always `true`.
- `network_reads_attempted`, `network_writes_attempted`, `airtable_client_called` always `false`.
- `tauri_command_exposes_live_restore`, `typescript_ui_path_exposes_live_restore` always `false`.
- No token, base ID, table/field/record values accepted.
- `evaluate_write_gate()` never modified.
- No restore success/completed/enabled product state exists.
- Not reachable from UI, TypeScript, or any Tauri command.
- `AlphaReadyRestoreRuntimeDisabled` does NOT mean product-level restore is complete or approved.

### Test count

- 24 unit tests in `#[cfg(test)]` block

---

## Related Documents

- [Restore Write Engine Skeleton](./restore-write-engine-skeleton.md)
- [Schema Write Engine Foundation](./schema-write-engine-foundation.md)
- [Record Write Engine Foundation](./record-write-engine-foundation.md)
- [Restore Execution Command Contract](./restore-execution-command-contract.md)
- [Live Restore Write Safety Checklist](../qa/live-restore-write-safety-checklist.md)
- [Known Limitations](../release/known-limitations.md)
- [Security Architecture](./security-architecture.md)
