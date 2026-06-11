import type { AppHealthResponse, ConnectionCheckResult } from "./types";
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
