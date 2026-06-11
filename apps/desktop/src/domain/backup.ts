import type { AirtableBaseId, AirtableWorkspaceId } from "./airtable";
import type { ConnectionId } from "./connection";

export type BackupPackageId = string;
export type BackupJobId = string;

export type BackupScope = "full" | "schema_only" | "records_only";

export type BackupStatus = "pending" | "running" | "succeeded" | "failed" | "cancelled";

export interface BackupPackageSummary {
  id: BackupPackageId;
  connectionId: ConnectionId;
  baseId: AirtableBaseId;
  workspaceId: AirtableWorkspaceId;
  baseName: string;
  scope: BackupScope;
  status: BackupStatus;
  tableCount: number;
  recordCount: number;
  fileSizeBytes: number;
  createdAt: string; // ISO
  outputPath: string;
}

export interface BackupJobSummary {
  id: BackupJobId;
  connectionId: ConnectionId;
  baseId: AirtableBaseId;
  baseName: string;
  scope: BackupScope;
  status: BackupStatus;
  startedAt?: string;
  completedAt?: string;
  packageId?: BackupPackageId;
  errorMessage?: string;
  tablesProcessed: number;
  totalTables: number;
  recordsProcessed: number;
}
