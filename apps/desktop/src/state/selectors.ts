import type { AppState } from "./appState";
import type { AirtableConnectionProfile } from "../domain/connection";
import type { BackupPackageSummary } from "../domain/backup";
import type { ReportSummary } from "../domain/report";
import type { JobLogEntry, LogLevel } from "../domain/log";
import type { FieldRestoreSupport } from "../domain/compatibility";
import type { JobSummary } from "../domain/job";

export interface DashboardStats {
  connectedBases: number;
  recentBackups: number;
  restoreJobs: number;
}

export interface PermissionSummary {
  passed: number;
  failed: number;
  unknown: number;
  total: number;
}

export interface CompatibilitySummaryResult {
  bySupport: Record<FieldRestoreSupport, number>;
  totalRules: number;
}

export function getConnectedProfiles(state: AppState): AirtableConnectionProfile[] {
  return state.connections.filter((c) => c.status === "connected");
}

export function getRecentBackupPackages(state: AppState, limit = 10): BackupPackageSummary[] {
  return [...state.backupPackages]
    .sort((a, b) => new Date(b.createdAt).getTime() - new Date(a.createdAt).getTime())
    .slice(0, limit);
}

export function getRecentReports(state: AppState, limit = 10): ReportSummary[] {
  return [...state.reports]
    .sort((a, b) => new Date(b.createdAt).getTime() - new Date(a.createdAt).getTime())
    .slice(0, limit);
}

export function getActiveJobs(state: AppState): JobSummary[] {
  const backupMapped: JobSummary[] = state.backupJobs
    .filter((j) => j.status === "running" || j.status === "pending")
    .map((j) => ({
      id: j.id,
      type: "backup" as const,
      status: j.status === "pending" ? ("queued" as const) : (j.status as "running"),
      startedAt: j.startedAt,
      completedAt: j.completedAt,
      errorMessage: j.errorMessage,
      progress:
        j.totalTables > 0
          ? {
              current: j.tablesProcessed,
              total: j.totalTables,
              label: `${j.tablesProcessed} of ${j.totalTables} tables`,
            }
          : undefined,
    }));

  const restoreMapped: JobSummary[] = state.restoreJobs
    .filter((j) => j.status === "running" || j.status === "pending")
    .map((j) => ({
      id: j.id,
      type: "restore" as const,
      status: j.status === "pending" ? ("queued" as const) : (j.status as "running"),
      startedAt: j.startedAt,
      completedAt: j.completedAt,
      errorMessage: j.errorMessage,
      progress:
        j.totalTables > 0
          ? {
              current: j.tablesRestored,
              total: j.totalTables,
              label: `${j.tablesRestored} of ${j.totalTables} tables`,
            }
          : undefined,
    }));

  return [...backupMapped, ...restoreMapped];
}

export function getPermissionSummary(state: AppState): PermissionSummary {
  let passed = 0;
  let failed = 0;
  let unknown = 0;

  for (const connection of state.connections) {
    for (const perm of connection.permissions) {
      if (perm.status === "passed") {
        passed += 1;
      } else if (perm.status === "failed") {
        failed += 1;
      } else {
        // covers "unknown" and "checking"
        unknown += 1;
      }
    }
  }

  return { passed, failed, unknown, total: passed + failed + unknown };
}

export function getCompatibilitySummary(state: AppState): CompatibilitySummaryResult {
  const bySupport: Record<FieldRestoreSupport, number> = {
    restorable: 0,
    partially_restorable: 0,
    metadata_only: 0,
    unsupported_for_restore: 0,
    manual_action_required: 0,
  };

  for (const rule of state.compatibilityRules) {
    bySupport[rule.support] = (bySupport[rule.support] ?? 0) + 1;
  }

  return { bySupport, totalRules: state.compatibilityRules.length };
}

export function getLogsByLevel(state: AppState, level: LogLevel): JobLogEntry[] {
  return state.logs.filter((entry) => entry.level === level);
}

export function getDashboardStats(state: AppState): DashboardStats {
  const hasConnected = getConnectedProfiles(state).length > 0;
  const connectedBases = hasConnected ? state.bases.length : 0;
  const recentBackups = state.backupPackages.filter((p) => p.status === "succeeded").length;
  const restoreJobs = state.restoreJobs.length;

  return { connectedBases, recentBackups, restoreJobs };
}
