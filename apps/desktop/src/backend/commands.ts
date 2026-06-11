import type {
  AccessibleBaseSummary,
  AppHealthResponse,
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
  RunBackupCommandRequest,
  RunBackupCommandResponse,
} from "./types";
import type { AirtableWorkspace, AirtableBaseSummary } from "../domain/airtable";
import type { BackupPackageSummary } from "../domain/backup";
import type { RestorePlanSummary } from "../domain/restore";
import type { ReportSummary } from "../domain/report";
import type { JobLogEntry } from "../domain/log";
import type { FieldCompatibilityRule } from "../domain/compatibility";

// Lazily calls the Tauri invoke function. Returns null if Tauri is unavailable
// (e.g. running in jsdom test environment or a plain browser without Tauri IPC).
async function safeInvoke<T>(command: string, args?: Record<string, unknown>): Promise<T | null> {
  try {
    const { invoke } = await import("@tauri-apps/api/core");
    return await invoke<T>(command, args);
  } catch {
    return null;
  }
}

export async function getAppHealth(): Promise<AppHealthResponse | null> {
  return safeInvoke<AppHealthResponse>("get_app_health");
}

export async function checkConnection(token: string): Promise<ConnectionCheckResult | null> {
  // Token is forwarded to the Rust command only. It is not stored or logged here.
  return safeInvoke<ConnectionCheckResult>("check_connection", { token });
}

export async function listWorkspaces(): Promise<AirtableWorkspace[] | null> {
  return safeInvoke<AirtableWorkspace[]>("list_workspaces");
}

export async function listBases(): Promise<AirtableBaseSummary[] | null> {
  return safeInvoke<AirtableBaseSummary[]>("list_bases");
}

export async function listBackupPackages(): Promise<BackupPackageSummary[] | null> {
  return safeInvoke<BackupPackageSummary[]>("list_backup_packages");
}

export async function listRestorePlans(): Promise<RestorePlanSummary[] | null> {
  return safeInvoke<RestorePlanSummary[]>("list_restore_plans");
}

export async function listReports(): Promise<ReportSummary[] | null> {
  return safeInvoke<ReportSummary[]>("list_reports");
}

export async function listLogs(): Promise<JobLogEntry[] | null> {
  return safeInvoke<JobLogEntry[]>("list_logs");
}

export async function listCompatibilityRules(): Promise<FieldCompatibilityRule[] | null> {
  return safeInvoke<FieldCompatibilityRule[]>("list_compatibility_rules");
}

export async function listAccessibleBases(token: string): Promise<AccessibleBaseSummary[] | null> {
  // Token is forwarded to the Rust command only. It is not stored or logged here.
  return safeInvoke<AccessibleBaseSummary[]>("list_accessible_bases", { token });
}

export async function getBaseSchema(
  token: string,
  baseId: string,
): Promise<BaseSchemaSummary | null> {
  // Token is forwarded to the Rust command only. It is not stored or logged here.
  return safeInvoke<BaseSchemaSummary>("get_base_schema", { token, baseId });
}

export async function createBackupPlan(request: BackupPlanRequest): Promise<BackupPlan | null> {
  return safeInvoke<BackupPlan>("create_backup_plan", { request });
}

export async function createRecordsExportPlan(
  request: RecordsExportPlanRequest,
): Promise<RecordsExportPlan | null> {
  return safeInvoke<RecordsExportPlan>("create_records_export_plan", { request });
}

/**
 * Validate a proposed backup output path without writing any file.
 * Safe to call from the UI at any time.
 * Returns null if Tauri IPC is unavailable (jsdom / browser without Tauri).
 */
export async function validateBackupOutputPath(
  path: string,
): Promise<OutputPathValidationResult | null> {
  return safeInvoke<OutputPathValidationResult>("validate_backup_output_path", { path });
}

/**
 * Run a backup job.
 *
 * - Token is forwarded to Rust only; never stored here.
 * - Confirmation must be "CREATE BACKUP".
 * - Returns null if Tauri IPC is unavailable.
 */
export async function runBackupJob(
  request: RunBackupCommandRequest,
): Promise<RunBackupCommandResponse | null> {
  // Token is forwarded to the Rust command only; not stored or logged here.
  return safeInvoke<RunBackupCommandResponse>("run_backup_job", { request });
}

/**
 * Signal cancellation for a running backup job.
 *
 * V0.1: always returns `wasRunning: false` — no background registry yet.
 * Returns null if Tauri IPC is unavailable.
 */
export async function cancelBackupJob(jobId: string): Promise<BackupJobCancellationResult | null> {
  return safeInvoke<BackupJobCancellationResult>("cancel_backup_job", { jobId });
}

/**
 * Get a progress snapshot for a running backup job.
 *
 * Not wired to a Tauri command in V0.1 — jobs run synchronously and complete
 * before the frontend could poll. Returns null always.
 */
export async function getBackupJobStatus(jobId: string): Promise<BackupJobProgressSnapshot | null> {
  void jobId;
  return null;
}

/**
 * Inspect an existing `.airbridge` package at the given absolute path.
 *
 * - No files are extracted.
 * - No writes of any kind.
 * - The result contains filename only — the full path is never included.
 * - Returns null if Tauri IPC is unavailable.
 */
export async function inspectBackupPackage(
  path: string,
): Promise<BackupPackageInspectionResult | null> {
  return safeInvoke<BackupPackageInspectionResult>("inspect_backup_package", { path });
}
