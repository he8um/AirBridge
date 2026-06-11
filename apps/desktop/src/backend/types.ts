// AppHealth returned by get_app_health command
export interface AppHealthResponse {
  appName: string;
  version: string;
  status: string;
  backend: string;
}

// ConnectionCheckResult from check_connection command
export interface ConnectionCheckResult {
  connectionId: string;
  status: "disconnected" | "checking" | "connected" | "failed";
  permissions: Array<{
    key: string;
    label: string;
    status: "unknown" | "checking" | "passed" | "failed";
    detail?: string;
  }>;
  /** Bases visible to the token, populated on successful connection check. */
  accessibleBases?: Array<{
    id: string;
    name: string;
  }>;
}

// AirBridgeError structure returned on command failure
export interface AirBridgeCommandError {
  code: string;
  message: string;
}

// Catalog and schema summary types from list_accessible_bases / get_base_schema commands

export interface AccessibleBaseSummary {
  id: string;
  name: string;
}

export interface FieldTypeCount {
  fieldType: string;
  count: number;
}

export interface SchemaCompatibilitySummary {
  restorableCount: number;
  metadataOnlyCount: number;
  unknownCount: number;
  totalCount: number;
}

export interface TableSchemaSummary {
  id: string;
  name: string;
  fieldCount: number;
  fieldTypeCounts: FieldTypeCount[];
  compatibility: SchemaCompatibilitySummary;
}

export interface BaseSchemaSummary {
  baseId: string;
  tableCount: number;
  tables: TableSchemaSummary[];
  compatibility: SchemaCompatibilitySummary;
}

// Backup planning types from create_backup_plan command

export type BackupScope = "full" | "schemaOnly" | "recordsOnly";

export type WarningSeverity = "info" | "warning" | "error";

export type AttachmentPolicy = "metadataOnly";

export type LinkedRecordPolicy = "referencesCaptured" | "remappingRequiredForRestore";

export type RecordReadEstimate = { type: "known"; value: number } | { type: "unknown" };

export interface BackupPlanCompatibilitySummary {
  restorableCount: number;
  metadataOnlyCount: number;
  unknownCount: number;
  totalCount: number;
}

export interface BackupPlanWarning {
  severity: WarningSeverity;
  code: string;
  message: string;
  tableName?: string;
  fieldName?: string;
}

export interface BackupPlanEstimate {
  schemaRequests: number;
  recordReadPages: RecordReadEstimate;
  note: string;
}

export interface BackupPlanField {
  id: string;
  name: string;
  fieldType: string;
  compatibility: "restorable" | "metadataOnly" | "unknown";
  attachmentPolicy?: AttachmentPolicy;
  linkedRecordPolicy?: LinkedRecordPolicy;
}

export interface BackupPlanTable {
  id: string;
  name: string;
  fieldCount: number;
  recordCount?: number;
  fields: BackupPlanField[];
  warnings: BackupPlanWarning[];
  compatibility: BackupPlanCompatibilitySummary;
}

export interface BackupPlanFieldInput {
  id: string;
  name: string;
  fieldType: string;
}

export interface BackupPlanTableInput {
  id: string;
  name: string;
  fields: BackupPlanFieldInput[];
  recordCount?: number;
}

export interface BackupPlanRequest {
  baseId: string;
  baseName: string;
  scope: BackupScope;
  tables: BackupPlanTableInput[];
}

export interface BackupPlan {
  baseId: string;
  baseName: string;
  scope: BackupScope;
  tableCount: number;
  totalFieldCount: number;
  tables: BackupPlanTable[];
  compatibility: BackupPlanCompatibilitySummary;
  warnings: BackupPlanWarning[];
  estimate: BackupPlanEstimate;
  /** Always true in this phase — no backup file is created. */
  dryRun: boolean;
  /** Always absent in this phase — no output file is written. */
  outputPackagePath?: string;
}

// Package format validation types (mirrors Rust backup::validation)

export type PackageValidationStatus = "valid" | "invalid" | "warning";

export interface PackageValidationIssue {
  code: string;
  message: string;
}

export interface PackageManifestSummary {
  format: string;
  formatVersion: string;
  appVersion: string;
  createdAt: string;
  baseId: string;
  baseName: string;
  tableCount: number;
  recordCount: number;
}

export interface PackageValidationReport {
  status: PackageValidationStatus;
  errors: PackageValidationIssue[];
  warnings: PackageValidationIssue[];
  entryCount: number;
  manifestSummary?: PackageManifestSummary;
}
