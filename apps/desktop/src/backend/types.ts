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

// ─── Restore Write Engine Skeleton ───────────────────────────────────────────

/**
 * Status of a write engine skeleton preview.
 * Note: "succeeded" is intentionally absent — the write engine is not enabled.
 */
export type RestoreWriteEngineStatus = "disabled" | "blocked" | "notStarted";

/** Pipeline phase identifiers for the write engine. */
export type RestoreWritePhase =
  | "validateInputs"
  | "schemaCreation"
  | "recordCreation"
  | "linkedRecordUpdates"
  | "attachmentHandling"
  | "finalValidation";

/** Why restore writes are disabled or blocked. */
export type RestoreWriteDisabledReason =
  | "disabledByProductPolicy"
  | "blockedByInvalidPlan"
  | "blockedByMissingConfirmation"
  | "blockedByTargetSafety"
  | "notAvailable";

/** An event emitted during write engine skeleton evaluation. */
export interface RestoreWriteEvent {
  phase: RestoreWritePhase;
  code: string;
  message: string;
}

/** Per-phase status summary for one write engine phase. */
export interface RestoreWritePhaseSummary {
  phase: RestoreWritePhase;
  status: RestoreWriteEngineStatus;
  /** Always true — no changes made. */
  noChangesMade: boolean;
  note: string;
}

/**
 * Input for the write engine skeleton preview command.
 *
 * - No token field — no Airtable access required.
 * - Counts are derived from existing planning outputs.
 */
export interface RestoreWriteEngineRequest {
  /** Filename-only identifier from the most recent package inspection. */
  packageFilename: string;
  /** Full path — used to derive filename; never echoed in the result. */
  packagePath: string;
  schemaTableCount?: number;
  schemaDirectFieldCount?: number;
  schemaDeferredFieldCount?: number;
  schemaManualActionCount?: number;
  schemaUnsupportedCount?: number;
  estimatedFirstPassBatches?: number;
  estimatedSecondPassBatches?: number;
  linkedRecordUpdateCount?: number;
}

/**
 * Result of the write engine skeleton preview.
 *
 * - No token.
 * - No absolute path — filename only.
 * - noChangesMade is always true.
 * - status is always disabled or blocked — never succeeded.
 */
export interface RestoreWriteEngineResult {
  /** Filename only — never the full path. */
  filename: string;
  status: RestoreWriteEngineStatus;
  disabledReason: RestoreWriteDisabledReason;
  message: string;
  phaseSummaries: RestoreWritePhaseSummary[];
  events: RestoreWriteEvent[];
  /** Always true — no Airtable changes were made. */
  noChangesMade: boolean;
}

// ─── Credential Storage ───────────────────────────────────────────────────────

/** The kind of credential being stored. */
export type CredentialKind = "airtablePersonalAccessToken";

/** Whether the OS keychain backend is available on this system. */
export type CredentialStorageAvailability = "available" | "unavailable";

/** The current storage status of a credential. */
export type CredentialStorageStatus = "saved" | "notSaved" | "unavailable" | "failed";

/** Request to check the storage status of a credential. */
export interface CredentialStatusRequest {
  kind: CredentialKind;
}

/**
 * Result of a credential status check.
 * Never contains the token value.
 */
export interface CredentialStatusResult {
  kind: CredentialKind;
  status: CredentialStorageStatus;
  availability: CredentialStorageAvailability;
  hasSavedToken: boolean;
  /** Safe display string — never the token value. */
  display: string;
}

/**
 * Request to save a token to the OS keychain.
 * The token is sent to the Rust command only and is never returned.
 */
export interface CredentialSaveRequest {
  kind: CredentialKind;
  /** Forwarded to the Rust keychain command only. Never returned. */
  token: string;
}

/**
 * Result of saving a credential.
 * Never contains the token.
 */
export interface CredentialSaveResult {
  kind: CredentialKind;
  success: boolean;
  hasSavedToken: boolean;
  display: string;
  errorMessage: string | null;
}

/** Request to remove a saved credential. */
export interface CredentialRemoveRequest {
  kind: CredentialKind;
}

/**
 * Result of removing a credential.
 * Never contains the token.
 */
export interface CredentialRemoveResult {
  kind: CredentialKind;
  success: boolean;
  hasSavedToken: boolean;
  display: string;
  errorMessage: string | null;
}

/**
 * A safe redacted summary of a stored credential for display purposes.
 * Never contains the token value.
 */
export interface RedactedCredentialSummary {
  kind: CredentialKind;
  hasSavedToken: boolean;
  display: string;
}

// ── Schema write engine foundation ─────────────────────────────────────────────

/**
 * What kind of schema write operation is planned.
 * Operations are never executed in this version.
 */
export type SchemaWriteOperationKind =
  | "createBase"
  | "createTable"
  | "createField"
  | "deferLinkedField"
  | "manualAction";

/**
 * Planning-time status for a schema write operation.
 * "succeeded" is intentionally absent — no operations are executed.
 */
export type SchemaWriteOperationStatus = "planned" | "blocked" | "disabled";

/** Why a schema write plan is blocked or disabled. */
export type SchemaWriteBlockedReason =
  | "disabledByProductPolicy"
  | "schemaPlanNotReady"
  | "noTablesInPlan";

/**
 * A single planned schema write operation.
 *
 * Safety properties:
 * - No token field.
 * - source_table_id / source_field_id come from the backup package, not from
 *   a live Airtable response.
 * - status is never "succeeded".
 * - no_changes_made is always true.
 */
export interface SchemaWriteOperation {
  index: number;
  kind: SchemaWriteOperationKind;
  status: SchemaWriteOperationStatus;
  sourceTableId: string;
  tableName: string;
  sourceFieldId?: string;
  fieldName?: string;
  fieldType?: string;
  linkedSourceTableId?: string;
  note: string;
  noChangesMade: boolean;
}

/**
 * A full schema write request plan.
 *
 * Safety properties:
 * - No token field.
 * - filename is basename only.
 * - noChangesMade is always true.
 * - networkWritesAttempted is always false.
 * - status is never "succeeded".
 */
export interface SchemaWriteRequestPlan {
  filename: string;
  status: SchemaWriteOperationStatus;
  blockedReason?: SchemaWriteBlockedReason;
  operations: SchemaWriteOperation[];
  tableOpCount: number;
  fieldOpCount: number;
  deferredOpCount: number;
  manualActionCount: number;
  totalOpCount: number;
  warnings: string[];
  noChangesMade: boolean;
  networkWritesAttempted: boolean;
}

/**
 * Request for the schema write request plan preview command.
 * No token field.
 */
export interface SchemaWriteRequestPlanRequest {
  packageFilename: string;
  schemaPlanStatus: string;
  tableCount: number;
  directFieldCount: number;
  deferredFieldCount: number;
  manualActionCount: number;
}

/**
 * Result of the schema write request plan preview command.
 *
 * Safety properties:
 * - No token field.
 * - filename is basename only.
 * - noChangesMade is always true.
 * - networkWritesAttempted is always false.
 * - status is never "succeeded".
 */
export interface SchemaWriteRequestPlanResult {
  filename: string;
  status: SchemaWriteOperationStatus;
  blockedReason?: SchemaWriteBlockedReason;
  disabledReason?: string;
  message: string;
  tableOpCount: number;
  fieldOpCount: number;
  deferredOpCount: number;
  manualActionCount: number;
  totalOpCount: number;
  warnings: string[];
  noChangesMade: boolean;
  networkWritesAttempted: boolean;
}

// ── Record Write Engine Foundation ─────────────────────────────────────────────

export type RecordWriteOperationKind =
  | "createRecordBatch"
  | "updateLinkedRecordBatch"
  | "checkpoint"
  | "preserveMetadataOnlyAttachment"
  | "skipComputedField"
  | "manualAction";

/** Planning-time status. success/succeeded/completed/executed intentionally absent. */
export type RecordWriteOperationStatus = "planned" | "blocked" | "disabled";

export type RecordWriteBlockedReason =
  | "disabledByProductPolicy"
  | "recordImportPlanNotReady"
  | "noTablesInPlan";

export interface RecordWriteOperation {
  index: number;
  kind: RecordWriteOperationKind;
  status: RecordWriteOperationStatus;
  tableId: string;
  tableName: string;
  batchIndex?: number;
  /** Number of records planned for this batch. Absent if record count is unknown. */
  plannedRecordCount?: number;
  linkedFieldCount?: number;
  attachmentPolicy?: string;
  skippedFieldName?: string;
  skippedFieldType?: string;
  note: string;
  noChangesMade: boolean;
}

export interface RecordWriteRequestPlan {
  filename: string;
  status: RecordWriteOperationStatus;
  blockedReason?: RecordWriteBlockedReason;
  operations: RecordWriteOperation[];
  createBatchOpCount: number;
  linkedUpdateOpCount: number;
  checkpointOpCount: number;
  attachmentOpCount: number;
  skippedFieldOpCount: number;
  totalOpCount: number;
  totalFirstPassBatches: number;
  totalSecondPassBatches: number;
  warnings: string[];
  noChangesMade: boolean;
  networkWritesAttempted: boolean;
}

/** No token field — record write planning requires no Airtable access. */
export interface RecordWriteRequestPlanRequest {
  packageFilename: string;
  recordImportPlanStatus: string;
  tableCount: number;
  totalFirstPassBatches: number;
  totalSecondPassBatches: number;
  attachmentFieldCount: number;
  skippedFieldCount: number;
}

export interface RecordWriteRequestPlanResult {
  filename: string;
  status: RecordWriteOperationStatus;
  blockedReason?: RecordWriteBlockedReason;
  disabledReason?: string;
  message: string;
  createBatchOpCount: number;
  linkedUpdateOpCount: number;
  checkpointOpCount: number;
  attachmentOpCount: number;
  skippedFieldOpCount: number;
  totalOpCount: number;
  totalFirstPassBatches: number;
  totalSecondPassBatches: number;
  warnings: string[];
  noChangesMade: boolean;
  networkWritesAttempted: boolean;
}

// ── Sandbox verification types (mirrors Rust restore::sandbox_verification) ──

export type SandboxVerificationStatus = "verified" | "warning" | "blocked";

export type SandboxVerificationCheckStatus = "passed" | "warning" | "failed" | "skipped";

export type SandboxVerificationFailureReason =
  | "targetModeNotAllowed"
  | "targetNotEmpty"
  | "missingTargetIdentifier"
  | "missingTargetName"
  | "writeGateDisabled"
  | "writeGateUnexpectedState"
  | "destructiveOperationRequested"
  | "attachmentUploadRequested"
  | "tokenReturnForbidden"
  | "fullPathReturnForbidden"
  | "liveMetadataCheckUnavailable"
  | "invalidRequest"
  | "unsupportedTarget";

export interface SandboxVerificationCheck {
  checkId: string;
  label: string;
  status: SandboxVerificationCheckStatus;
  message: string;
  remediation?: string;
}

export interface SandboxVerificationTarget {
  targetMode: RestoreTargetMode;
  targetBaseId?: string;
  targetBaseName?: string;
}

export interface SandboxVerificationSafetySummary {
  writesEnabled: boolean;
  networkWritesAttempted: boolean;
  noChangesMade: boolean;
  writeGateStatus: string;
  liveMetadataCheckPerformed: boolean;
}

export interface SandboxVerificationRequest {
  targetMode: RestoreTargetMode;
  targetBaseId?: string;
  targetBaseName?: string;
  targetTableCount?: number;
  targetRecordCount?: number;
  expectsEmptyTarget: boolean;
  allowAttachmentUpload: boolean;
  allowDestructiveOperations: boolean;
  sourcePackageFilename?: string;
  schemaPlanStatus?: string;
  recordImportPlanStatus?: string;
}
// NOTE: No token field. No full path field.

export interface SandboxVerificationResult {
  status: SandboxVerificationStatus;
  checks: SandboxVerificationCheck[];
  safetySummary: SandboxVerificationSafetySummary;
  message: string;
  noChangesMade: boolean;
  networkWritesAttempted: boolean;
  writesEnabled: boolean;
}

// ── Restore confirmation types (mirrors Rust restore::restore_confirmation) ──

export type RestoreConfirmationStatus = "confirmed" | "rejected" | "blocked";

export type RestoreConfirmationCheckStatus = "passed" | "failed" | "skipped";

export interface RestoreConfirmationRequirement {
  requirementId: string;
  label: string;
  satisfied: boolean;
  note: string;
}

export interface RestoreConfirmationCheck {
  checkId: string;
  label: string;
  status: RestoreConfirmationCheckStatus;
  message: string;
}

/**
 * Request for Gate 2 confirmation validation.
 * - No token field.
 * - No filesystem path field.
 */
export interface RestoreConfirmationRequest {
  enteredText: string;
  sourcePackageLabel?: string;
  targetLabel?: string;
  sandboxVerificationStatus?: string;
}

/**
 * Result from validate_restore_confirmation_gate.
 * - No token field.
 * - writesEnabled is always false.
 * - noChangesMade is always true.
 * - networkWritesAttempted is always false.
 * - Confirmed status does NOT enable restore writes.
 */
export interface RestoreConfirmationResult {
  status: RestoreConfirmationStatus;
  checks: RestoreConfirmationCheck[];
  requirements: RestoreConfirmationRequirement[];
  requiredText: string;
  message: string;
  noChangesMade: boolean;
  networkWritesAttempted: boolean;
  writesEnabled: boolean;
}

// ── Target empty verification types (mirrors Rust restore::target_empty_verification) ──

export type TargetEmptyVerificationStatus = "verified" | "warning" | "blocked";

export type TargetEmptyVerificationCheckStatus = "passed" | "warning" | "failed" | "skipped";

export interface TargetEmptyVerificationCheck {
  checkId: string;
  label: string;
  status: TargetEmptyVerificationCheckStatus;
  message: string;
  remediation?: string;
}

/**
 * Request for Gate 3 target empty verification.
 * - No token field.
 * - No filesystem path field.
 */
export interface TargetEmptyVerificationRequest {
  targetMode: string;
  targetTableCount?: number;
  targetRecordCount?: number;
  targetDisplayName?: string;
  liveCheckPerformed: boolean;
}

/**
 * Result from verify_restore_target_empty.
 * - No token field.
 * - writesEnabled is always false.
 * - noChangesMade is always true.
 * - networkWritesAttempted is always false.
 * - Verified status does NOT enable restore writes.
 */
export interface TargetEmptyVerificationResult {
  status: TargetEmptyVerificationStatus;
  checks: TargetEmptyVerificationCheck[];
  message: string;
  noChangesMade: boolean;
  networkWritesAttempted: boolean;
  writesEnabled: boolean;
}

// ── Destructive operation policy types (mirrors Rust restore::destructive_operation_policy) ──

export type DestructiveOperationPolicyStatus = "compliant" | "warning" | "blocked";

export type DestructiveOperationCheckStatus = "passed" | "warning" | "failed";

export type DestructiveOperationKind =
  | "deleteBase"
  | "deleteTable"
  | "deleteField"
  | "deleteRecord"
  | "updateExistingRecord"
  | "overwriteField"
  | "overwriteTable"
  | "attachmentUpload"
  | "createBase"
  | "createTable"
  | "createField"
  | "createRecord"
  | "updateLinkedRecordReference"
  | "preserveAttachmentMetadata"
  | "checkpoint"
  | "skipField"
  | "manualAction"
  | "deferLinkedField";

export interface DeclaredOperation {
  kind: DestructiveOperationKind;
  label: string;
}

export interface DestructiveOperationCheck {
  checkId: string;
  label: string;
  status: DestructiveOperationCheckStatus;
  message: string;
  remediation?: string;
}

/**
 * Request for Gate 4 destructive-operation policy verification.
 * - No token field.
 * - No filesystem path field.
 */
export interface DestructiveOperationPolicyRequest {
  declaredOperations: DeclaredOperation[];
  targetDisplayName?: string;
}

/**
 * Result from verify_destructive_operation_policy_gate.
 * - No token field.
 * - writesEnabled is always false.
 * - noChangesMade is always true.
 * - networkWritesAttempted is always false.
 * - Compliant status does NOT enable restore writes.
 */
export interface DestructiveOperationPolicyResult {
  status: DestructiveOperationPolicyStatus;
  checks: DestructiveOperationCheck[];
  message: string;
  blockedOperations: string[];
  noChangesMade: boolean;
  networkWritesAttempted: boolean;
  writesEnabled: boolean;
}

// ── Attachment upload policy types (mirrors Rust restore::attachment_upload_policy) ──

export type AttachmentUploadPolicyStatus = "compliant" | "warning" | "blocked";

export type AttachmentUploadPolicyCheckStatus = "passed" | "warning" | "failed";

export type AttachmentUploadIntent =
  | "metadataOnly"
  | "uploadRequested"
  | "downloadRequested"
  | "unknown";

export interface DeclaredAttachmentField {
  fieldName: string;
  tableName: string;
  intent: AttachmentUploadIntent;
}

export interface AttachmentUploadPolicyCheck {
  checkId: string;
  label: string;
  status: AttachmentUploadPolicyCheckStatus;
  message: string;
  remediation?: string;
}

/**
 * Request for Gate 5 attachment upload policy verification.
 * - No token field.
 * - No filesystem path field.
 * - No full attachment URL field.
 */
export interface AttachmentUploadPolicyRequest {
  declaredAttachmentFields: DeclaredAttachmentField[];
  targetDisplayName?: string;
}

/**
 * Result from verify_attachment_upload_policy_gate.
 * - No token field.
 * - No filesystem path field.
 * - No full attachment URL field.
 * - writesEnabled is always false.
 * - noChangesMade is always true.
 * - networkWritesAttempted is always false.
 * - Compliant status does NOT enable restore writes.
 * - Attachment file bytes are never uploaded.
 */
export interface AttachmentUploadPolicyResult {
  status: AttachmentUploadPolicyStatus;
  checks: AttachmentUploadPolicyCheck[];
  message: string;
  blockedFieldNames: string[];
  metadataOnlyFieldCount: number;
  noChangesMade: boolean;
  networkWritesAttempted: boolean;
  writesEnabled: boolean;
}

export type SchemaRecordOrderPolicyStatus = "compliant" | "warning" | "blocked";
export type SchemaRecordOrderCheckStatus = "passed" | "warning" | "failed";
export type RestoreWritePhaseKind =
  | "schema"
  | "records"
  | "linkedRecords"
  | "attachments"
  | "validation";

export interface DeclaredWritePhase {
  phase: RestoreWritePhaseKind;
  isPlanned: boolean;
  isBlocked: boolean;
}

export interface SchemaRecordOrderCheck {
  checkId: string;
  label: string;
  status: SchemaRecordOrderCheckStatus;
  message: string;
  remediation?: string;
}

export interface SchemaRecordOrderPolicyRequest {
  declaredPhases: DeclaredWritePhase[];
  targetDisplayName?: string;
}

export interface SchemaRecordOrderPolicyResult {
  status: SchemaRecordOrderPolicyStatus;
  checks: SchemaRecordOrderCheck[];
  message: string;
  orderingViolations: string[];
  noChangesMade: boolean;
  networkWritesAttempted: boolean;
  writesEnabled: boolean;
}

export type SandboxWriteTestingPolicyStatus = "compliant" | "warning" | "blocked";
export type SandboxWriteTestingCheckStatus = "passed" | "warning" | "failed";
export type SandboxTargetClassification = "sandbox" | "production" | "unknown";

/**
 * Evidence that sandbox write testing has been performed.
 * - No token field.
 * - No filesystem path field (testPackageFilename is basename only).
 * - No raw record payload.
 */
export interface SandboxWriteTestEvidence {
  sandboxBaseVerified: boolean;
  testPackageFilename?: string;
  dryRunCompleted: boolean;
  schemaPlanReviewed: boolean;
  recordPlanReviewed: boolean;
  reviewerLabel?: string;
  evidenceTimestamp?: string;
}

export interface SandboxWriteTestingCheck {
  checkId: string;
  label: string;
  status: SandboxWriteTestingCheckStatus;
  message: string;
  remediation?: string;
}

export interface SandboxWriteTestingPolicyRequest {
  targetClassification: SandboxTargetClassification;
  sandboxVerificationPassed: boolean;
  evidence?: SandboxWriteTestEvidence;
  targetDisplayName?: string;
}

/**
 * Result from verify_sandbox_write_testing_policy_gate.
 * - No token field.
 * - No filesystem path field.
 * - No record payload field.
 * - writesEnabled is always false.
 * - noChangesMade is always true.
 * - networkWritesAttempted is always false.
 * - Compliant status does NOT enable restore writes.
 */
export interface SandboxWriteTestingPolicyResult {
  status: SandboxWriteTestingPolicyStatus;
  checks: SandboxWriteTestingCheck[];
  message: string;
  noChangesMade: boolean;
  networkWritesAttempted: boolean;
  writesEnabled: boolean;
}

// ── Gate 8: Live write confirmation policy ────────────────────────────────────

export type LiveWriteConfirmationPolicyStatus = "confirmed" | "warning" | "blocked" | "rejected";
export type LiveWriteConfirmationCheckStatus = "passed" | "warning" | "failed";

/**
 * Prior gate status summary for the live-write confirmation policy check.
 * All fields are safe display strings — no token, no path, no record payload.
 */
export interface PriorGateStatuses {
  sandboxVerificationStatus?: string;
  destructiveOperationPolicyStatus?: string;
  attachmentUploadPolicyStatus?: string;
  schemaRecordOrderPolicyStatus?: string;
  sandboxWriteTestingPolicyStatus?: string;
}

/**
 * Input to the live-write confirmation policy gate.
 * - No token field.
 * - No filesystem path field.
 * - No record payload field.
 */
export interface LiveWriteConfirmationPolicyRequest {
  enteredText: string;
  targetLabel?: string;
  priorGateStatuses?: PriorGateStatuses;
}

export interface LiveWriteConfirmationCheck {
  checkId: string;
  label: string;
  status: LiveWriteConfirmationCheckStatus;
  message: string;
  remediation?: string;
}

/**
 * Result from verify_live_write_confirmation_policy_gate.
 * - No token field.
 * - No filesystem path field.
 * - No record payload field.
 * - writesEnabled is always false.
 * - noChangesMade is always true.
 * - networkWritesAttempted is always false.
 * - Confirmed status does NOT enable restore writes.
 */
export interface LiveWriteConfirmationPolicyResult {
  status: LiveWriteConfirmationPolicyStatus;
  checks: LiveWriteConfirmationCheck[];
  requiredText: string;
  message: string;
  noChangesMade: boolean;
  networkWritesAttempted: boolean;
  writesEnabled: boolean;
}
