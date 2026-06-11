import type { AirtableConnectionProfile } from "../domain/connection";
import type { AirtableWorkspace, AirtableBaseSummary } from "../domain/airtable";
import type { BackupPackageSummary } from "../domain/backup";
import type { RestorePlanSummary } from "../domain/restore";
import type { ReportSummary } from "../domain/report";
import type { JobLogEntry } from "../domain/log";
import type { FieldCompatibilityRule } from "../domain/compatibility";
import type { ConnectionCheckResult } from "../backend/types";

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
}
