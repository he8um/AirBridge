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
  RestoreWriteEngineRequest,
  RestoreWriteEngineResult,
  RunBackupCommandRequest,
  RunBackupCommandResponse,
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
}
