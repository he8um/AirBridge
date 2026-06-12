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
  RunBackupCommandRequest,
  RunBackupCommandResponse,
} from "../backend/types";
import type { AirBridgeService } from "./airBridgeService";
import * as commands from "../backend/commands";

async function listConnections(): Promise<AirtableConnectionProfile[]> {
  return [];
}

async function listWorkspaces(): Promise<AirtableWorkspace[]> {
  return (await commands.listWorkspaces()) ?? [];
}

async function listBases(): Promise<AirtableBaseSummary[]> {
  return (await commands.listBases()) ?? [];
}

async function listBackupPackages(): Promise<BackupPackageSummary[]> {
  return (await commands.listBackupPackages()) ?? [];
}

async function listRestorePlans(): Promise<RestorePlanSummary[]> {
  return (await commands.listRestorePlans()) ?? [];
}

async function listReports(): Promise<ReportSummary[]> {
  return (await commands.listReports()) ?? [];
}

async function listLogs(): Promise<JobLogEntry[]> {
  return (await commands.listLogs()) ?? [];
}

async function listCompatibilityRules(): Promise<FieldCompatibilityRule[]> {
  return (await commands.listCompatibilityRules()) ?? [];
}

async function checkConnection(input: { token: string }): Promise<ConnectionCheckResult> {
  // Token is forwarded to the Rust command only. It is never stored here.
  const result = await commands.checkConnection(input.token);

  if (result === null) {
    // Tauri IPC unavailable — return a safe fallback failure.
    return {
      connectionId: "conn-unavailable",
      status: "failed",
      permissions: [
        { key: "schema.bases:read", label: "Schema read", status: "unknown" },
        { key: "data.records:read", label: "Records read", status: "unknown" },
        {
          key: "schema.bases:write",
          label: "Schema write",
          status: "unknown",
          detail: "Write access not verified",
        },
        {
          key: "data.records:write",
          label: "Records write",
          status: "unknown",
          detail: "Write access not verified",
        },
      ],
    };
  }

  return result;
}

async function listAccessibleBases(input: { token: string }): Promise<AccessibleBaseSummary[]> {
  // Token is forwarded to the Rust command only. It is never stored here.
  return (await commands.listAccessibleBases(input.token)) ?? [];
}

async function getBaseSchema(input: { token: string; baseId: string }): Promise<BaseSchemaSummary> {
  // Token is forwarded to the Rust command only. It is never stored here.
  const result = await commands.getBaseSchema(input.token, input.baseId);
  if (result === null) {
    return {
      baseId: input.baseId,
      tableCount: 0,
      tables: [],
      compatibility: {
        restorableCount: 0,
        metadataOnlyCount: 0,
        unknownCount: 0,
        totalCount: 0,
      },
    };
  }
  return result;
}

async function createRecordsExportPlan(
  request: RecordsExportPlanRequest,
): Promise<RecordsExportPlan> {
  const result = await commands.createRecordsExportPlan(request);
  if (result === null) {
    return {
      baseId: request.baseId,
      baseName: request.baseName,
      tableCount: 0,
      pageSize: 100,
      tables: [],
      warnings: [],
      plannedOnly: true,
    };
  }
  return result;
}

async function createBackupPlan(request: BackupPlanRequest): Promise<BackupPlan> {
  const result = await commands.createBackupPlan(request);
  if (result === null) {
    // Tauri IPC unavailable — return a safe empty plan.
    return {
      baseId: request.baseId,
      baseName: request.baseName,
      scope: request.scope,
      tableCount: 0,
      totalFieldCount: 0,
      tables: [],
      compatibility: { restorableCount: 0, metadataOnlyCount: 0, unknownCount: 0, totalCount: 0 },
      warnings: [],
      estimate: {
        schemaRequests: 1,
        recordReadPages: { type: "unknown" },
        note: "Plan unavailable — Tauri runtime not detected.",
      },
      dryRun: true,
    };
  }
  return result;
}

async function validateBackupOutputPath(path: string): Promise<OutputPathValidationResult> {
  const result = await commands.validateBackupOutputPath(path);
  if (result === null) {
    return { valid: false, errorCode: "IPC_UNAVAILABLE", errorMessage: "Tauri IPC unavailable" };
  }
  return result;
}

async function runBackupJob(request: RunBackupCommandRequest): Promise<RunBackupCommandResponse> {
  // Token is forwarded to the Rust command only; not stored or logged here.
  const result = await commands.runBackupJob(request);
  if (result === null) {
    return {
      success: false,
      safetyErrors: [{ code: "IPC_UNAVAILABLE", message: "Tauri IPC unavailable" }],
      pathValidation: { valid: false, errorCode: "IPC_UNAVAILABLE" },
    };
  }
  return result;
}

async function cancelBackupJob(jobId: string): Promise<BackupJobCancellationResult> {
  const result = await commands.cancelBackupJob(jobId);
  if (result === null) {
    return { jobId, wasRunning: false, statusAtCancellation: "not_running" };
  }
  return result;
}

async function getBackupJobStatus(jobId: string): Promise<BackupJobProgressSnapshot | null> {
  return commands.getBackupJobStatus(jobId);
}

async function inspectBackupPackage(path: string): Promise<BackupPackageInspectionResult> {
  const result = await commands.inspectBackupPackage(path);
  if (result === null) {
    return {
      filename: "",
      validationStatus: "invalid",
      entryCount: 0,
      warnings: [],
      errors: [{ code: "IPC_UNAVAILABLE", message: "Tauri IPC unavailable" }],
    };
  }
  return result;
}

async function createRestoreDryRunPlan(request: RestoreDryRunRequest): Promise<RestoreDryRunPlan> {
  const result = await commands.createRestoreDryRunPlan(request);
  if (result === null) {
    return {
      filename: "",
      status: "blocked",
      targetMode: request.targetMode,
      tables: [],
      warnings: [],
      errors: [{ code: "IPC_UNAVAILABLE", message: "Tauri IPC unavailable" }],
      noChangesMade: true,
    };
  }
  return result;
}

async function runRestoreExecution(
  request: RestoreExecutionRequest,
): Promise<RestoreExecutionResult> {
  // Token is forwarded to the Rust command only; not stored or logged here.
  const result = await commands.runRestoreExecution(request);
  if (result === null) {
    return {
      filename: request.packageFilename,
      status: "blocked",
      blockReason: "missingPackageInspection",
      message: "Tauri IPC unavailable. No Airtable changes were made.",
      warnings: [],
      errors: [{ code: "IPC_UNAVAILABLE", message: "Tauri IPC unavailable" }],
      noChangesMade: true,
    };
  }
  return result;
}

export const liveAirBridgeService: AirBridgeService = {
  listConnections,
  listWorkspaces,
  listBases,
  listBackupPackages,
  listRestorePlans,
  listReports,
  listLogs,
  listCompatibilityRules,
  checkConnection,
  listAccessibleBases,
  getBaseSchema,
  createBackupPlan,
  createRecordsExportPlan,
  validateBackupOutputPath,
  runBackupJob,
  cancelBackupJob,
  getBackupJobStatus,
  inspectBackupPackage,
  createRestoreDryRunPlan,
  runRestoreExecution,
};
