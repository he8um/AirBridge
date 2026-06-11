import type { AirtableBaseId } from "./airtable";
import type { BackupPackageId } from "./backup";
import type { ConnectionId } from "./connection";

export type RestorePlanId = string;
export type RestoreJobId = string;

export type RestoreMode = "new_base" | "empty_existing_base";

export type RestorePlanStatus = "draft" | "validated" | "incompatible" | "ready";

export type RestoreJobStatus =
  | "pending"
  | "running"
  | "dry_run_complete"
  | "succeeded"
  | "failed"
  | "cancelled";

export interface RestoreCompatibilityWarning {
  fieldId: string;
  fieldName: string;
  fieldType: string;
  message: string;
  severity: "info" | "warning" | "error";
}

export interface RestorePlanSummary {
  id: RestorePlanId;
  packageId: BackupPackageId;
  connectionId: ConnectionId;
  targetBaseId?: AirtableBaseId;
  mode: RestoreMode;
  status: RestorePlanStatus;
  warnings: RestoreCompatibilityWarning[];
  createdAt: string;
}

export interface RestoreJobSummary {
  id: RestoreJobId;
  planId: RestorePlanId;
  connectionId: ConnectionId;
  isDryRun: boolean;
  status: RestoreJobStatus;
  startedAt?: string;
  completedAt?: string;
  tablesRestored: number;
  totalTables: number;
  recordsRestored: number;
  skippedFields: string[];
  errorMessage?: string;
}
