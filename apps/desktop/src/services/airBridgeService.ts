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
  OutputPathValidationResult,
  RecordsExportPlan,
  RecordsExportPlanRequest,
  RestoreDryRunPlan,
  RestoreDryRunRequest,
  RestoreExecutionRequest,
  RestoreExecutionResult,
  RestoreSchemaPlan,
  RestoreSchemaPlanRequest,
  RunBackupCommandRequest,
  RunBackupCommandResponse,
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
}
