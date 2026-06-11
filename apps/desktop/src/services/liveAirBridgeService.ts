import type { AirtableConnectionProfile } from "../domain/connection";
import type { AirtableWorkspace, AirtableBaseSummary } from "../domain/airtable";
import type { BackupPackageSummary } from "../domain/backup";
import type { RestorePlanSummary } from "../domain/restore";
import type { ReportSummary } from "../domain/report";
import type { JobLogEntry } from "../domain/log";
import type { FieldCompatibilityRule } from "../domain/compatibility";
import type {
  AccessibleBaseSummary,
  BaseSchemaSummary,
  ConnectionCheckResult,
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
};
