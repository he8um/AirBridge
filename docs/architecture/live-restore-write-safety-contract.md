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

**Gate 6 — schema record order policy** is implemented via the `verify_schema_record_order_policy_gate` Tauri command. It runs 5 checks (SRO-01 through SRO-05): write gate state, schema phase presence and plannedness, schema phase ordering before records, record-create ordering before linked-record updates, and record-create ordering before attachment handling. Missing or blocked schema phases cause `Blocked`. A record phase declared before a schema phase causes `Blocked` with a `records-before-schema` ordering violation. Linked-record or attachment phases declared before record-create cause `Blocked`. Warning conditions (unplanned schema, linked/attachment without records, no phases declared) produce `Warning`. No Airtable API calls are made. No token is required. No record payload appears in any result field. A `Compliant` result does NOT enable restore writes.

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
