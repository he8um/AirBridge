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

// Records export planning types (mirrors Rust backup::export_plan)

export type RecordCountState = { type: "known"; count: number } | { type: "unknown" };

export type RequestEstimate = { type: "known"; pages: number } | { type: "unknown" };

export interface JsonlOutputPlan {
  entryPath: string;
  plannedOnly: boolean;
}

export interface LinkedRecordExtractionPlan {
  fieldId: string;
  fieldName: string;
  policy: LinkedRecordPolicy;
  restoreNote: string;
}

export interface AttachmentMetadataExtractionPlan {
  fieldId: string;
  fieldName: string;
  policy: AttachmentPolicy;
  contentNote: string;
}

export interface FieldExtractionPlan {
  fieldId: string;
  fieldName: string;
  fieldType: string;
  compatibility: "restorable" | "metadataOnly" | "unknown";
  linkedRecordPlan?: LinkedRecordExtractionPlan;
  attachmentPlan?: AttachmentMetadataExtractionPlan;
}

export interface TableExportPlan {
  tableId: string;
  tableName: string;
  recordCount: RecordCountState;
  requestEstimate: RequestEstimate;
  pageSize: number;
  jsonlOutput: JsonlOutputPlan;
  tableMetadataPath: string;
  fieldsMetadataPath: string;
  fields: FieldExtractionPlan[];
  linkedRecordPlans: LinkedRecordExtractionPlan[];
  attachmentPlans: AttachmentMetadataExtractionPlan[];
  warnings: BackupPlanWarning[];
}

export interface RecordsExportPlan {
  baseId: string;
  baseName: string;
  tableCount: number;
  pageSize: number;
  tables: TableExportPlan[];
  warnings: BackupPlanWarning[];
  /** Always true — no records have been fetched and no file has been written. */
  plannedOnly: boolean;
  /** Always absent — no output file is written at planning time. */
  outputPackagePath?: string;
}

export interface RecordsExportPlanRequest {
  baseId: string;
  baseName: string;
  backupPlan: BackupPlan;
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

// Backup package inspection types (mirrors Rust backup::inspection)

export interface BackupPackageInspectionIssue {
  code: string;
  message: string;
}

export interface BackupPackageInspectionManifestSummary {
  format: string;
  formatVersion: string;
  appVersion: string;
  createdAt: string;
  provider: string;
  baseId: string;
  baseName: string;
}

export interface BackupPackageInspectionContentsSummary {
  tableCount: number;
  fieldCount: number;
  recordCount: number;
  linkedRecordRelationshipCount: number;
  attachmentCount: number;
}

export interface BackupPackageInspectionSecuritySummary {
  encrypted: boolean;
  containsRecordData: boolean;
  containsAttachmentUrls: boolean;
  redactionsApplied: string[];
}

export interface BackupPackageInspectionChecksumSummary {
  checksumCount: number;
  allValid: boolean;
}

export interface BackupPackageInspectionResult {
  /** Filename only — never includes directory path. */
  filename: string;
  validationStatus: "valid" | "invalid" | "warning";
  manifest?: BackupPackageInspectionManifestSummary;
  contents?: BackupPackageInspectionContentsSummary;
  security?: BackupPackageInspectionSecuritySummary;
  checksums?: BackupPackageInspectionChecksumSummary;
  entryCount: number;
  warnings: BackupPackageInspectionIssue[];
  errors: BackupPackageInspectionIssue[];
}

// ── Backup Job Orchestration ───────────────────────────────────────────────

export type BackupJobStatus = "queued" | "running" | "succeeded" | "failed" | "cancelled";

export type BackupJobPhase =
  | "planning"
  | "schema"
  | "recordsExport"
  | "packageBuild"
  | "validation"
  | "completed";

export interface BackupJobWarning {
  code: string;
  message: string;
  tableId?: string;
}

export interface BackupJobError {
  code: string;
  message: string;
  recoverable: boolean;
}

export interface BackupJobTableResult {
  tableId: string;
  tableName: string;
  recordCount: number;
  pagesFetched: number;
}

export interface BackupJobPackageSummary {
  packageId: string;
  formatVersion: string;
  tableCount: number;
  recordCount: number;
  entryCount: number;
  checksumCount: number;
  /** Always false in V0.1. */
  encrypted: boolean;
  attachmentPolicy: string;
}

export interface BackupJobValidationSummary {
  status: PackageValidationStatus;
  errorCount: number;
  warningCount: number;
  entryCount: number;
}

/**
 * A single event in the backup job timeline.
 *
 * The `kind` discriminant is one of: jobStarted, phaseStarted,
 * tableExportStarted, tableExportCompleted, packageWriteStarted,
 * packageWriteCompleted, validationStarted, validationCompleted,
 * jobSucceeded, jobFailed, jobCancelled.
 *
 * No token, no absolute paths, no attachment URLs.
 */
export interface BackupJobEvent {
  kind: string;
  jobId: string;
  phase?: BackupJobPhase;
  tableId?: string;
  tableName?: string;
  recordCount?: number;
  pagesFetched?: number;
  entryCount?: number;
  status?: string;
  errorCount?: number;
  warningCount?: number;
  totalRecords?: number;
  tableCount?: number;
  errorCode?: string;
  message?: string;
  atPhase?: BackupJobPhase;
  baseId?: string;
  baseName?: string;
}

/** Read-only snapshot of a backup job's progress at a point in time. */
export interface BackupJobProgressSnapshot {
  jobId: string;
  phase: BackupJobPhase;
  status: BackupJobStatus;
  completedTables: number;
  totalTables?: number;
  unknownTotal: boolean;
  currentTableId?: string;
  currentTableName?: string;
  warningCount: number;
  errorCount: number;
}

/** Request to cancel a running backup job. */
export interface BackupJobCancellationRequest {
  jobId: string;
}

/**
 * Result of a cancellation attempt.
 *
 * In V0.1 `wasRunning` is always false — no background job registry exists yet.
 * `statusAtCancellation` is `"not_running"` in V0.1.
 */
export interface BackupJobCancellationResult {
  jobId: string;
  wasRunning: boolean;
  statusAtCancellation: string;
}

export interface BackupJobResult {
  jobId: string;
  status: BackupJobStatus;
  baseId: string;
  baseName: string;
  tables: BackupJobTableResult[];
  warnings: BackupJobWarning[];
  errors: BackupJobError[];
  packageSummary?: BackupJobPackageSummary;
  validationSummary?: BackupJobValidationSummary;
  /** Ordered event timeline. Empty for jobs that ran before this field was added. */
  events?: BackupJobEvent[];
}

// ── Safe Backup Command Contract ───────────────────────────────────────────

/** Table spec sent to the run_backup_job command. */
export interface RunBackupTableSpec {
  tableId: string;
  tableName: string;
  linkedFieldNames?: string[];
  attachmentFieldNames?: string[];
}

/**
 * Request to run a backup job via the Tauri command bridge.
 *
 * - `token` is forwarded to the Rust command only; never stored.
 * - `confirmation` must equal "CREATE BACKUP" exactly.
 * - `outputPath` must pass output path validation (`.airbridge` extension,
 *   parent directory exists, no traversal, no null bytes).
 * - No token appears in the response.
 * - No absolute output path appears in the response.
 */
export interface RunBackupCommandRequest {
  token: string;
  outputPath: string;
  /** Must equal "CREATE BACKUP". */
  confirmation: string;
  baseId: string;
  baseName: string;
  baseJson: number[];
  schemaJson: number[];
  tableSpecs: RunBackupTableSpec[];
  pageSize?: number;
  jobId?: string;
}

/** Result of validating a proposed backup output path. */
export interface OutputPathValidationResult {
  valid: boolean;
  errorCode?: string;
  errorMessage?: string;
}

/** A pre-run safety error (confirmation missing, path invalid, etc.). */
export interface BackupCommandSafetyError {
  code: string;
  message: string;
}

/**
 * Response returned by the run_backup_job command.
 *
 * - No token.
 * - No absolute output path — only the package filename is returned.
 */
export interface RunBackupCommandResponse {
  success: boolean;
  /** Filename-only portion of the output path (no directory). */
  packageFilename?: string;
  safetyErrors?: BackupCommandSafetyError[];
  jobResult?: BackupJobResult;
  pathValidation: OutputPathValidationResult;
}

// ── Restore dry-run planning types (mirrors Rust restore::plan) ────────────

export type RestoreTargetMode = "newBase" | "emptyExistingBase";

export type RestorePlanStatus = "ready" | "readyWithWarnings" | "blocked";

export type RestoreFieldCompatibility =
  | "supported"
  | "partiallySupported"
  | "metadataOnly"
  | "unsupported"
  | "manualActionRequired";

export interface RestorePackageSummary {
  /** Filename only — never the full path. */
  filename: string;
  format: string;
  formatVersion: string;
  appVersion: string;
  createdAt: string;
  provider: string;
  baseId: string;
  baseName: string;
  tableCount: number;
  fieldCount: number;
  recordCount: number;
  containsRecordData: boolean;
  containsAttachmentUrls: boolean;
  encrypted: boolean;
}

export interface RestoreFieldPlan {
  fieldId: string;
  fieldName: string;
  fieldType: string;
  compatibility: RestoreFieldCompatibility;
  note: string;
}

export interface RestoreLinkedRecordPlan {
  fieldId: string;
  fieldName: string;
  linkedTableId: string;
  remappingRequired: boolean;
  note: string;
}

export interface RestoreAttachmentPlan {
  fieldId: string;
  fieldName: string;
  /** Always true in V0.1 — file content is not re-uploaded. */
  metadataOnly: boolean;
  note: string;
}

export interface RestoreTablePlan {
  tableId: string;
  tableName: string;
  fieldCount: number;
  recordCount: number;
  fields: RestoreFieldPlan[];
  linkedRecordPlans: RestoreLinkedRecordPlan[];
  attachmentPlans: RestoreAttachmentPlan[];
  restorableFieldCount: number;
  partialFieldCount: number;
  unsupportedFieldCount: number;
}

export interface RestoreRecordOrderingPlan {
  createTablesFirst: boolean;
  createFieldsAfterTables: boolean;
  importRecordsWithoutLinks: boolean;
  applyLinksAfterRecords: boolean;
  note: string;
}

export interface RestoreDryRunWarning {
  code: string;
  message: string;
  tableName?: string;
  fieldName?: string;
}

export interface RestoreDryRunError {
  code: string;
  message: string;
}

export interface RestoreDryRunRequest {
  /** Absolute path to the package. Never echoed in the result. */
  path: string;
  targetMode: RestoreTargetMode;
  targetBaseName?: string;
}

export interface RestoreDryRunPlan {
  /** Filename only — never the full path. */
  filename: string;
  status: RestorePlanStatus;
  targetMode: RestoreTargetMode;
  targetBaseName?: string;
  packageSummary?: RestorePackageSummary;
  tables: RestoreTablePlan[];
  ordering?: RestoreRecordOrderingPlan;
  warnings: RestoreDryRunWarning[];
  errors: RestoreDryRunError[];
  /** Always true — states that no Airtable changes were made. */
  noChangesMade: boolean;
}

// ── Restore execution command contract (mirrors Rust restore::execution) ───

/**
 * Status of the restore execution attempt.
 * Note: "succeeded" is intentionally absent — the write engine is not enabled.
 */
export type RestoreExecutionStatus = "blocked" | "readyButDisabled" | "failed";

/** The reason a restore execution was blocked. */
export type RestoreExecutionBlockReason =
  | "missingPackageInspection"
  | "invalidPackage"
  | "missingDryRunPlan"
  | "dryRunBlocked"
  | "missingTargetMode"
  | "missingToken"
  | "missingConfirmation"
  | "restoreWriteEngineNotEnabled";

export interface RestoreExecutionWarning {
  code: string;
  message: string;
}

export interface RestoreExecutionError {
  code: string;
  message: string;
}

/**
 * Input for the restore execution command.
 * - token is forwarded to the Rust command only; never stored here.
 * - packagePath is the absolute path used to locate the file; never echoed in the result.
 * - confirmation must equal "RESTORE BACKUP" exactly.
 */
export interface RestoreExecutionRequest {
  /** Filename-only identifier from the most recent inspection result. */
  packageFilename: string;
  /** Absolute path to the package. Never echoed in the result. */
  packagePath: string;
  /** Validation status from the most recent inspection ("valid" | "warning"). */
  packageValidationStatus: string;
  /** Status from the most recent dry-run plan ("ready" | "readyWithWarnings"). */
  dryRunStatus: string;
  targetMode: RestoreTargetMode;
  targetBaseName?: string;
  /** Airtable personal access token. Forwarded to Rust only; never stored. */
  token: string;
  /** Must equal "RESTORE BACKUP" exactly. */
  confirmation: string;
}

/**
 * Result of the restore execution command.
 * - No token.
 * - No absolute path — filename only.
 * - noChangesMade is always true.
 */
export interface RestoreExecutionResult {
  /** Filename only — never the full path. */
  filename: string;
  status: RestoreExecutionStatus;
  blockReason?: RestoreExecutionBlockReason;
  message: string;
  warnings: RestoreExecutionWarning[];
  errors: RestoreExecutionError[];
  /** Always true — no Airtable changes were made. */
  noChangesMade: boolean;
}

// ── Restore schema creation planning types (mirrors Rust restore::schema_plan) ──

export type RestoreSchemaPlanStatus = "ready" | "readyWithWarnings" | "blocked";

export type RestoreFieldCreateClassification =
  | "createDirectly"
  | "createWithAdjustment"
  | "deferUntilTablesExist"
  | "metadataOnly"
  | "manualActionRequired"
  | "unsupported";

export interface RestoreTableCreationStep {
  tableId: string;
  tableName: string;
  stepIndex: number;
  fieldCount: number;
  directFieldCount: number;
  deferredFieldCount: number;
  manualActionCount: number;
  unsupportedCount: number;
  note: string;
}

export interface RestoreFieldCreationStep {
  fieldId: string;
  fieldName: string;
  fieldType: string;
  tableId: string;
  tableName: string;
  classification: RestoreFieldCreateClassification;
  note: string;
}

export interface RestoreDeferredFieldStep {
  fieldId: string;
  fieldName: string;
  fieldType: string;
  tableId: string;
  tableName: string;
  reason: string;
  linkedTableId?: string;
}

export interface RestoreManualActionField {
  fieldId: string;
  fieldName: string;
  fieldType: string;
  tableId: string;
  tableName: string;
  actionDescription: string;
}

export interface RestoreLinkedDependencyStep {
  fieldId: string;
  fieldName: string;
  sourceTableId: string;
  sourceTableName: string;
  targetTableId: string;
  targetTableName: string;
  remappingRequired: boolean;
  note: string;
}

export interface RestoreSchemaDependencyGraph {
  edges: RestoreLinkedDependencyStep[];
  hasCircularDependency: boolean;
  resolutionNote: string;
}

export interface RestoreSchemaWarning {
  code: string;
  message: string;
  tableName?: string;
  fieldName?: string;
}

export interface RestoreSchemaError {
  code: string;
  message: string;
}

/** Input table for the schema creation plan command (derived from dry-run plan). */
export interface SchemaPlanTableInput {
  tableId: string;
  tableName: string;
  fields: SchemaPlanFieldInput[];
}

/** Input field for the schema creation plan command (derived from dry-run plan). */
export interface SchemaPlanFieldInput {
  fieldId: string;
  fieldName: string;
  fieldType: string;
  linkedTableId?: string;
}

/**
 * Input for the schema creation plan command.
 * - No token — schema planning requires no Airtable access.
 * - packageFilename is the filename only; never a full path.
 */
export interface RestoreSchemaPlanRequest {
  /** Filename from the most recent package inspection or dry-run. Never a path. */
  packageFilename: string;
  /** Serialised dry-run plan status for gate-check ("ready" | "readyWithWarnings" | "blocked"). */
  dryRunStatus: string;
  targetMode: RestoreTargetMode;
  targetBaseName?: string;
  /** Tables extracted from the dry-run plan for planning purposes. */
  tables: SchemaPlanTableInput[];
}

/**
 * Full schema creation plan.
 * - No Airtable calls.
 * - No writes.
 * - No token in the result.
 * - noChangesMade is always true.
 */
export interface RestoreSchemaPlan {
  /** Filename only — never the full path. */
  filename: string;
  status: RestoreSchemaPlanStatus;
  targetMode: RestoreTargetMode;
  targetBaseName?: string;
  /** Ordered steps for table creation (tables planned before fields). */
  tableSteps: RestoreTableCreationStep[];
  /** Ordered steps for field creation (only directly-creatable fields). */
  fieldSteps: RestoreFieldCreationStep[];
  /** Fields deferred until tables and records exist. */
  deferredSteps: RestoreDeferredFieldStep[];
  /** Fields that require manual action outside the restore process. */
  manualActionFields: RestoreManualActionField[];
  /** Linked record dependency graph. */
  dependencyGraph: RestoreSchemaDependencyGraph;
  warnings: RestoreSchemaWarning[];
  errors: RestoreSchemaError[];
  /** Always true — no Airtable changes were made. */
  noChangesMade: boolean;
}

// ─── Restore Record Import Plan ───────────────────────────────────────────────

export type RestoreRecordImportPlanStatus = "ready" | "readyWithWarnings" | "blocked";

export type RestoreRecordBatchPhase =
  | "createRecords"
  | "updateLinkedRecords"
  | "skippedFields"
  | "validation";

export type RestoreRecordMappingStrategy =
  | "mapSourceRecordIdToCreatedRecordId"
  | "preserveSourceIdInMetadata"
  | "unavailableUntilExecution";

export type RestoreAttachmentRestorePolicy =
  | "metadataOnly"
  | "downloadNotSupported"
  | "uploadNotSupported"
  | "manualActionRequired";

export type RestoreRecordFieldImportPolicy =
  | "include"
  | "deferToLinkedRecordPass"
  | "skip"
  | "metadataOnly";

export interface RestoreRecordBatchPlan {
  batchIndex: number;
  phase: RestoreRecordBatchPhase;
  recordCount?: number;
  note: string;
}

export interface RestoreRecordMappingPlan {
  tableId: string;
  tableName: string;
  strategy: RestoreRecordMappingStrategy;
  remappingRequired: boolean;
  note: string;
}

export interface RestoreLinkedRecordUpdatePlan {
  tableId: string;
  tableName: string;
  fieldId: string;
  fieldName: string;
  linkedTableId: string;
  linkedTableName: string;
  updateBatchCount?: number;
  note: string;
}

export interface RestoreAttachmentImportPolicy {
  tableId: string;
  tableName: string;
  fieldId: string;
  fieldName: string;
  policy: RestoreAttachmentRestorePolicy;
  note: string;
}

export interface RestoreRecordFieldPolicy {
  fieldId: string;
  fieldName: string;
  fieldType: string;
  policy: RestoreRecordFieldImportPolicy;
  note: string;
}

export interface RestoreRecordImportCheckpointPlan {
  tableId: string;
  tableName: string;
  checkpointBatchIndex: number;
  sourceRecordIdOffsetPlaceholder: string;
  completedPhase: RestoreRecordBatchPhase;
  note: string;
}

export interface RestoreTableImportPlan {
  tableId: string;
  tableName: string;
  importOrder: number;
  recordCount?: number;
  recordCountKnown: boolean;
  batchSize: number;
  createBatchCount?: number;
  updateBatchCount?: number;
  firstPassBatches: RestoreRecordBatchPlan[];
  secondPassBatches: RestoreRecordBatchPlan[];
  fieldPolicies: RestoreRecordFieldPolicy[];
  attachmentPolicies: RestoreAttachmentImportPolicy[];
  mappingPlan: RestoreRecordMappingPlan;
  checkpointPlan: RestoreRecordImportCheckpointPlan;
  linkedRecordUpdates: RestoreLinkedRecordUpdatePlan[];
}

export interface RestoreRetryPolicy {
  maxRetriesOnRateLimit: number;
  initialBackoffMs: number;
  backoffMultiplier: number;
  note: string;
}

export interface RestoreRecordImportWarning {
  code: string;
  message: string;
  tableName?: string;
  fieldName?: string;
}

export interface RestoreRecordImportError {
  code: string;
  message: string;
}

export interface RecordImportFieldInput {
  fieldId: string;
  fieldName: string;
  fieldType: string;
  linkedTableId?: string;
}

export interface RecordImportTableInput {
  tableId: string;
  tableName: string;
  recordCount?: number;
  fields: RecordImportFieldInput[];
}

export interface RestoreRecordImportPlanRequest {
  packageFilename: string;
  dryRunStatus: string;
  schemaPlanStatus: string;
  targetMode: RestoreTargetMode;
  targetBaseName?: string;
  tables: RecordImportTableInput[];
}

export interface RestoreRecordImportPlan {
  filename: string;
  status: RestoreRecordImportPlanStatus;
  targetMode: RestoreTargetMode;
  targetBaseName?: string;
  tablePlans: RestoreTableImportPlan[];
  linkedRecordUpdatePlans: RestoreLinkedRecordUpdatePlan[];
  retryPolicy: RestoreRetryPolicy;
  warnings: RestoreRecordImportWarning[];
  errors: RestoreRecordImportError[];
  /** Always true — no Airtable changes were made. */
  noChangesMade: boolean;
}

// ─── Job History ──────────────────────────────────────────────────────────────

export type JobHistoryKind =
  | "connectionCheck"
  | "backupPlan"
  | "recordsExportPlan"
  | "backupExecution"
  | "packageInspection"
  | "restoreDryRun"
  | "restoreSchemaplan"
  | "restoreRecordImportPlan"
  | "restoreExecutionAttempt";

export type JobHistoryStatus =
  | "planned"
  | "running"
  | "succeeded"
  | "succeededWithWarnings"
  | "blocked"
  | "failed"
  | "cancelled";

export type JobHistorySource = "backupPage" | "restorePage" | "connectionsPage" | "system";

export interface JobHistoryWarning {
  code: string;
  message: string;
}

export interface JobHistoryError {
  code: string;
  message: string;
}

export interface JobHistorySummary {
  title: string;
  detail?: string;
  /** Filename only — never a full path. */
  packageFilename?: string;
  baseName?: string;
  warningCount: number;
  errorCount: number;
  validationStatus?: string;
}

export interface JobHistoryItem {
  id: { 0: string };
  kind: JobHistoryKind;
  status: JobHistoryStatus;
  source: JobHistorySource;
  /** ISO-8601 UTC timestamp string. */
  startedAt?: string;
  finishedAt?: string;
  summary: JobHistorySummary;
  warnings: JobHistoryWarning[];
  errors: JobHistoryError[];
  /** Always true for planning/inspection operations. */
  noChangesMade: boolean;
}

export interface JobHistoryFilter {
  kind?: JobHistoryKind;
  status?: JobHistoryStatus;
  limit?: number;
}

export interface JobHistoryListResult {
  items: JobHistoryItem[];
  totalCount: number;
  filtered: boolean;
}
