import type { AirtableConnectionProfile } from "../domain/connection";
import type { AirtableWorkspace, AirtableBaseSummary } from "../domain/airtable";
import type { BackupPackageSummary } from "../domain/backup";
import type { RestorePlanSummary } from "../domain/restore";
import type { ReportSummary } from "../domain/report";
import type { JobLogEntry } from "../domain/log";
import type { FieldCompatibilityRule } from "../domain/compatibility";
import type {
  AccessibleBaseSummary,
  BackupJobCancellationResult,
  BackupJobProgressSnapshot,
  BackupPackageInspectionResult,
  BackupPlan,
  BackupPlanRequest,
  BaseSchemaSummary,
  ConnectionCheckResult,
  CredentialRemoveRequest,
  CredentialRemoveResult,
  CredentialSaveRequest,
  CredentialSaveResult,
  CredentialStatusRequest,
  CredentialStatusResult,
  JobHistoryFilter,
  JobHistoryListResult,
  OutputPathValidationResult,
  RecordWriteRequestPlanRequest,
  RecordWriteRequestPlanResult,
  RecordsExportPlan,
  RecordsExportPlanRequest,
  RestoreDryRunPlan,
  RestoreDryRunRequest,
  RestoreExecutionRequest,
  RestoreExecutionResult,
  RestoreRecordImportPlan,
  RestoreRecordImportPlanRequest,
  RestoreSchemaPlan,
  RestoreSchemaPlanRequest,
  RestoreConfirmationRequest,
  RestoreConfirmationResult,
  TargetEmptyVerificationRequest,
  TargetEmptyVerificationResult,
  DestructiveOperationPolicyRequest,
  DestructiveOperationPolicyResult,
  AttachmentUploadPolicyRequest,
  AttachmentUploadPolicyResult,
  SchemaRecordOrderPolicyRequest,
  SchemaRecordOrderPolicyResult,
  SandboxWriteTestingPolicyRequest,
  SandboxWriteTestingPolicyResult,
  LiveWriteConfirmationPolicyRequest,
  LiveWriteConfirmationPolicyResult,
  RateLimitBackoffPolicyRequest,
  RateLimitBackoffPolicyResult,
  CheckpointDurabilityPolicyRequest,
  CheckpointDurabilityPolicyResult,
  FinalValidationPolicyRequest,
  FinalValidationPolicyResult,
  WritePhaseOrderingPolicyRequest,
  WritePhaseOrderingPolicyResult,
  FailureModesPolicyRequest,
  FailureModesPolicyResult,
  RollbackLimitationPolicyRequest,
  RollbackLimitationPolicyResult,
  RestoreWriteEngineRequest,
  RestoreWriteEngineResult,
  RunBackupCommandRequest,
  RunBackupCommandResponse,
  SandboxVerificationRequest,
  SandboxVerificationResult,
  SchemaWriteRequestPlanRequest,
  SchemaWriteRequestPlanResult,
} from "../backend/types";

export interface AirBridgeService {
  listConnections(): Promise<AirtableConnectionProfile[]>;
  listWorkspaces(): Promise<AirtableWorkspace[]>;
  listBases(): Promise<AirtableBaseSummary[]>;
  listBackupPackages(): Promise<BackupPackageSummary[]>;
  listRestorePlans(): Promise<RestorePlanSummary[]>;
  listReports(): Promise<ReportSummary[]>;
  listLogs(): Promise<JobLogEntry[]>;
  listCompatibilityRules(): Promise<FieldCompatibilityRule[]>;
  checkConnection(input: { token: string }): Promise<ConnectionCheckResult>;
  listAccessibleBases(input: { token: string }): Promise<AccessibleBaseSummary[]>;
  getBaseSchema(input: { token: string; baseId: string }): Promise<BaseSchemaSummary>;
  createBackupPlan(request: BackupPlanRequest): Promise<BackupPlan>;
  createRecordsExportPlan(request: RecordsExportPlanRequest): Promise<RecordsExportPlan>;
  validateBackupOutputPath(path: string): Promise<OutputPathValidationResult>;
  runBackupJob(request: RunBackupCommandRequest): Promise<RunBackupCommandResponse>;
  /**
   * Signal cancellation for a running backup job.
   *
   * V0.1: always returns `wasRunning: false` — no background registry exists yet.
   */
  cancelBackupJob(jobId: string): Promise<BackupJobCancellationResult>;
  /**
   * Get a progress snapshot for a job by ID.
   *
   * V0.1: always returns null — jobs run synchronously and complete before this
   * can be called. Future: returns a live snapshot from the background registry.
   */
  getBackupJobStatus(jobId: string): Promise<BackupJobProgressSnapshot | null>;
  /**
   * Inspect an existing `.airbridge` package.
   *
   * - No files are extracted.
   * - No writes of any kind.
   * - The result contains filename only — the full path is never included.
   */
  inspectBackupPackage(path: string): Promise<BackupPackageInspectionResult>;
  /**
   * Creates a restore dry-run plan from an existing `.airbridge` package.
   *
   * - No Airtable API calls.
   * - No token required.
   * - No files extracted.
   * - No write operations.
   * - The result contains filename only — the full path is never included.
   */
  createRestoreDryRunPlan(request: RestoreDryRunRequest): Promise<RestoreDryRunPlan>;
  /**
   * Validates the restore execution safety gate.
   *
   * - Token is forwarded to the command only; never stored.
   * - No Airtable API calls.
   * - No files extracted.
   * - No write operations.
   * - Returns filename only — the full path is never included.
   * - Always returns noChangesMade: true.
   * - In this version always returns blocked/readyButDisabled — write engine not enabled.
   */
  runRestoreExecution(request: RestoreExecutionRequest): Promise<RestoreExecutionResult>;
  /**
   * Creates a schema creation plan from a dry-run result.
   *
   * - No Airtable API calls.
   * - No token required.
   * - No files extracted.
   * - No write operations.
   * - Returns filename only — the full path is never included.
   * - Always returns noChangesMade: true.
   */
  createRestoreSchemaPlan(request: RestoreSchemaPlanRequest): Promise<RestoreSchemaPlan>;
  /**
   * Creates a record import plan from a dry-run result and schema plan.
   *
   * - No Airtable API calls.
   * - No token required.
   * - No files written or extracted.
   * - Returns filename only — the full path is never included.
   * - Always returns noChangesMade: true.
   */
  createRestoreRecordImportPlan(
    request: RestoreRecordImportPlanRequest,
  ): Promise<RestoreRecordImportPlan>;
  /**
   * List recent job history items.
   *
   * - No token in request or response.
   * - No full paths in response.
   * - In V0.1 returns deterministic in-memory data.
   */
  listJobHistory(filter?: JobHistoryFilter): Promise<JobHistoryListResult>;
  /**
   * Clear job history.
   *
   * In V0.1 this is a no-op (no persistent store). Returns count cleared (0).
   */
  clearJobHistory(): Promise<number>;
  /**
   * Produces a write engine skeleton preview from existing planning outputs.
   *
   * - No token field — no Airtable access required.
   * - No Airtable API calls.
   * - No file writes.
   * - Returns disabled status — never succeeded.
   * - noChangesMade is always true.
   */
  previewRestoreWriteEngine(request: RestoreWriteEngineRequest): Promise<RestoreWriteEngineResult>;
  /**
   * Returns the OS keychain storage status for the Airtable token.
   *
   * - Never returns the token value.
   * - Returns safe display string only.
   */
  getCredentialStorageStatus(request: CredentialStatusRequest): Promise<CredentialStatusResult>;
  /**
   * Saves an Airtable token to the OS keychain.
   *
   * - Token is forwarded to the Rust command only; never returned or stored here.
   * - Returns success status and safe display string only.
   * - No Airtable API calls.
   */
  saveAirtableTokenToKeychain(request: CredentialSaveRequest): Promise<CredentialSaveResult>;
  /**
   * Removes a saved Airtable token from the OS keychain.
   *
   * - Never returns the token.
   * - Returns success status only.
   */
  removeAirtableTokenFromKeychain(
    request: CredentialRemoveRequest,
  ): Promise<CredentialRemoveResult>;

  /**
   * Previews a schema write request plan built from a schema plan summary.
   *
   * - No token accepted or returned.
   * - No Airtable API calls are made. No schema is written.
   * - All operations are disabled — the write gate blocks execution.
   * - noChangesMade is always true. networkWritesAttempted is always false.
   */
  previewSchemaWriteRequestPlan(
    request: SchemaWriteRequestPlanRequest,
  ): Promise<SchemaWriteRequestPlanResult>;

  /**
   * Previews a record write request plan built from a record import plan summary.
   *
   * - No token accepted or returned.
   * - No Airtable API calls are made. No records are created, updated, or deleted.
   * - All operations are disabled — the write gate blocks execution.
   * - noChangesMade is always true. networkWritesAttempted is always false.
   * - No raw record payloads in result.
   * - Old-to-new record ID mapping is execution-time only — not resolved here.
   */
  previewRecordWriteRequestPlan(
    request: RecordWriteRequestPlanRequest,
  ): Promise<RecordWriteRequestPlanResult>;
  /**
   * Read-only sandbox environment verification (Gate 1).
   *
   * - No Airtable writes. No token. No full path. No network writes.
   * - writesEnabled is always false. noChangesMade is always true.
   * - Returns blocked for unsafe target configurations.
   */
  verifyRestoreSandboxEnvironment(
    request: SandboxVerificationRequest,
  ): Promise<SandboxVerificationResult>;
  /**
   * Validates restore confirmation text (Gate 2).
   *
   * - No Airtable writes. No token. No full path. No network writes.
   * - writesEnabled is always false. noChangesMade is always true.
   * - Confirmed status does NOT enable restore writes.
   */
  validateRestoreConfirmationGate(
    request: RestoreConfirmationRequest,
  ): Promise<RestoreConfirmationResult>;
  /**
   * Verifies that the restore target base is empty (Gate 3).
   *
   * - No Airtable write API calls. No token. No full path. No network writes.
   * - writesEnabled is always false. noChangesMade is always true.
   * - Verified status does NOT enable restore writes.
   */
  verifyRestoreTargetEmpty(
    request: TargetEmptyVerificationRequest,
  ): Promise<TargetEmptyVerificationResult>;
  /**
   * Verifies that no destructive operations exist in the declared restore plan (Gate 4).
   *
   * - No Airtable write API calls. No token. No full path. No network writes.
   * - writesEnabled is always false. noChangesMade is always true.
   * - Compliant status does NOT enable restore writes.
   */
  verifyDestructiveOperationPolicy(
    request: DestructiveOperationPolicyRequest,
  ): Promise<DestructiveOperationPolicyResult>;
  /**
   * Verifies the attachment upload policy for all declared attachment fields (Gate 5).
   *
   * - No Airtable write API calls. No token. No full path. No full attachment URL. No network writes.
   * - writesEnabled is always false. noChangesMade is always true.
   * - Compliant status does NOT enable restore writes.
   * - Attachment file bytes are never uploaded.
   */
  verifyAttachmentUploadPolicy(
    request: AttachmentUploadPolicyRequest,
  ): Promise<AttachmentUploadPolicyResult>;
  /**
   * Verifies that write phases observe schema-before-record ordering (Gate 6).
   *
   * - No Airtable write API calls. No token. No full path. No record payload. No network writes.
   * - writesEnabled is always false. noChangesMade is always true.
   * - Compliant status does NOT enable restore writes.
   */
  verifySchemaRecordOrderPolicy(
    request: SchemaRecordOrderPolicyRequest,
  ): Promise<SchemaRecordOrderPolicyResult>;
  /**
   * Verifies that sandbox write testing has been completed with required evidence (Gate 7).
   *
   * - No Airtable write API calls. No token. No full path. No record payload. No network writes.
   * - writesEnabled is always false. noChangesMade is always true.
   * - Compliant status does NOT enable restore writes.
   */
  verifySandboxWriteTestingPolicy(
    request: SandboxWriteTestingPolicyRequest,
  ): Promise<SandboxWriteTestingPolicyResult>;
  /**
   * Gate 8: Live-write-specific user confirmation policy.
   * - No Airtable API calls.
   * - No token required.
   * - No record payload.
   * - writesEnabled is always false.
   * - Confirmed does NOT enable restore writes.
   */
  verifyLiveWriteConfirmationPolicy(
    request: LiveWriteConfirmationPolicyRequest,
  ): Promise<LiveWriteConfirmationPolicyResult>;
  /**
   * Gate 9: Rate-limit and backoff policy.
   * - No Airtable API calls.
   * - No token required.
   * - No record payload.
   * - writesEnabled is always false.
   * - Compliant does NOT enable restore writes.
   */
  verifyRateLimitBackoffPolicy(
    request: RateLimitBackoffPolicyRequest,
  ): Promise<RateLimitBackoffPolicyResult>;
  /**
   * Gate 10: Checkpoint durability policy.
   * - No Airtable API calls.
   * - No token required.
   * - No record payload.
   * - writesEnabled is always false.
   * - Compliant does NOT enable restore writes.
   */
  verifyCheckpointDurabilityPolicy(
    request: CheckpointDurabilityPolicyRequest,
  ): Promise<CheckpointDurabilityPolicyResult>;
  /**
   * Verifies the final validation policy (Gate 11).
   * - No Airtable API calls.
   * - No token accepted or returned.
   * - No filesystem path accepted or returned.
   * - No record payload accepted or returned.
   * - noChangesMade is always true.
   * - networkWritesAttempted is always false.
   * - writesEnabled is always false.
   * - Compliant does NOT enable restore writes.
   * - Compliant does NOT introduce a restore success state.
   */
  verifyFinalValidationPolicy(
    request: FinalValidationPolicyRequest,
  ): Promise<FinalValidationPolicyResult>;
  /**
   * Verify write phase ordering policy (Gate 12).
   * - No Airtable API calls.
   * - No token accepted or returned.
   * - No record payload accepted or returned.
   * - writesEnabled is always false.
   * - noChangesMade is always true.
   * - networkWritesAttempted is always false.
   * - Compliant does NOT enable restore writes.
   * - Compliant does NOT introduce a restore success state.
   */
  verifyWritePhaseOrderingPolicy(
    request: WritePhaseOrderingPolicyRequest,
  ): Promise<WritePhaseOrderingPolicyResult>;
  /**
   * Gate 13 — Failure Modes Policy.
   * Safety invariants:
   * - No token field.
   * - No filesystem path field.
   * - No record payload field.
   * - writesEnabled is always false.
   * - noChangesMade is always true.
   * - networkWritesAttempted is always false.
   * - Compliant does NOT enable restore writes.
   * - Compliant does NOT introduce a restore success state.
   */
  verifyFailureModesPolicy(request: FailureModesPolicyRequest): Promise<FailureModesPolicyResult>;
  /**
   * Verifies the rollback limitation policy for a planned restore write operation.
   *
   * Safety invariants:
   * - No Airtable API calls are made.
   * - No token accepted or returned.
   * - No filesystem path accepted or returned.
   * - No record payload accepted or returned.
   * - writesEnabled is always false.
   * - noChangesMade is always true.
   * - networkWritesAttempted is always false.
   * - Compliant does NOT enable restore writes.
   * - Compliant does NOT introduce a restore success state.
   * - No automatic destructive rollback, delete, or update cleanup exists.
   */
  verifyRollbackLimitationPolicy(
    request: RollbackLimitationPolicyRequest,
  ): Promise<RollbackLimitationPolicyResult>;
}
