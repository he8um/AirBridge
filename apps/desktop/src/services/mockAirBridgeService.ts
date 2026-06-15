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
  JobHistoryFilter,
  JobHistoryItem,
  JobHistoryKind,
  JobHistoryListResult,
  JobHistorySource,
  JobHistoryStatus,
  OutputPathValidationResult,
  RecordImportFieldInput,
  RecordImportTableInput,
  RecordsExportPlan,
  RecordsExportPlanRequest,
  RestoreDryRunPlan,
  RestoreDryRunRequest,
  RestoreExecutionRequest,
  RestoreExecutionResult,
  RestoreRecordImportPlan,
  RestoreRecordImportPlanRequest,
  RestoreSchemaPlan,
  CredentialKind,
  CredentialRemoveRequest,
  CredentialRemoveResult,
  CredentialSaveRequest,
  CredentialSaveResult,
  CredentialStatusRequest,
  CredentialStatusResult,
  RestoreSchemaPlanRequest,
  RestoreWriteEngineRequest,
  RestoreWriteEngineResult,
  RunBackupCommandRequest,
  RestoreConfirmationRequest,
  RestoreConfirmationResult,
  TargetEmptyVerificationRequest,
  TargetEmptyVerificationResult,
  TargetEmptyVerificationStatus,
  DestructiveOperationPolicyRequest,
  DestructiveOperationPolicyResult,
  DestructiveOperationPolicyStatus,
  AttachmentUploadPolicyRequest,
  AttachmentUploadPolicyResult,
  AttachmentUploadPolicyStatus,
  SchemaRecordOrderPolicyRequest,
  SchemaRecordOrderPolicyResult,
  SchemaRecordOrderPolicyStatus,
  SandboxWriteTestingPolicyRequest,
  SandboxWriteTestingPolicyResult,
  SandboxWriteTestingPolicyStatus,
  LiveWriteConfirmationPolicyRequest,
  LiveWriteConfirmationPolicyResult,
  LiveWriteConfirmationPolicyStatus,
  RateLimitBackoffPolicyRequest,
  RateLimitBackoffPolicyResult,
  RateLimitBackoffPolicyStatus,
  CheckpointDurabilityPolicyRequest,
  CheckpointDurabilityPolicyResult,
  CheckpointDurabilityPolicyStatus,
  FinalValidationPolicyRequest,
  FinalValidationPolicyResult,
  FinalValidationPolicyStatus,
  WritePhaseOrderingPolicyRequest,
  WritePhaseOrderingPolicyResult,
  WritePhaseOrderingPolicyStatus,
  FailureModesPolicyRequest,
  FailureModesPolicyResult,
  FailureModesPolicyStatus,
  RollbackLimitationPolicyRequest,
  RollbackLimitationPolicyResult,
  RollbackLimitationPolicyStatus,
  RunBackupCommandResponse,
  RecordWriteRequestPlanRequest,
  RecordWriteRequestPlanResult,
  SandboxVerificationRequest,
  SandboxVerificationResult,
  SchemaWriteRequestPlanRequest,
  SchemaWriteRequestPlanResult,
  TableExportPlan,
} from "../backend/types";
import type { AirBridgeService } from "./airBridgeService";
import { MOCK_STATE } from "../state/mockState";

function listConnections(): Promise<AirtableConnectionProfile[]> {
  return Promise.resolve(MOCK_STATE.connections);
}

function listWorkspaces(): Promise<AirtableWorkspace[]> {
  return Promise.resolve(MOCK_STATE.workspaces);
}

function listBases(): Promise<AirtableBaseSummary[]> {
  return Promise.resolve(MOCK_STATE.bases);
}

function listBackupPackages(): Promise<BackupPackageSummary[]> {
  return Promise.resolve(MOCK_STATE.backupPackages);
}

function listRestorePlans(): Promise<RestorePlanSummary[]> {
  return Promise.resolve(MOCK_STATE.restorePlans);
}

function listReports(): Promise<ReportSummary[]> {
  return Promise.resolve(MOCK_STATE.reports);
}

function listLogs(): Promise<JobLogEntry[]> {
  return Promise.resolve(MOCK_STATE.logs);
}

function listCompatibilityRules(): Promise<FieldCompatibilityRule[]> {
  return Promise.resolve(MOCK_STATE.compatibilityRules);
}

function checkConnectionImpl(input: { token: string }): Promise<ConnectionCheckResult> {
  const tokenLength = input.token.length;
  const tokenHasFail = input.token.includes("fail");
  // Drop the token — it must not appear in the returned result
  void input.token;

  if (tokenLength >= 20 && !tokenHasFail) {
    return Promise.resolve({
      connectionId: "conn-preview",
      status: "connected",
      permissions: [
        { key: "schema.bases:read", label: "Schema read", status: "passed" },
        { key: "data.records:read", label: "Records read", status: "passed" },
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
      accessibleBases: [
        { id: "appExampleBase01", name: "Example Projects & Tasks" },
        { id: "appExampleBase02", name: "Example Contacts" },
      ],
    });
  }

  return Promise.resolve({
    connectionId: "conn-preview",
    status: "failed",
    permissions: [
      {
        key: "schema.bases:read",
        label: "Schema read",
        status: "failed",
        detail: "Invalid or expired token",
      },
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
  });
}

function listAccessibleBasesImpl(input: { token: string }): Promise<AccessibleBaseSummary[]> {
  // Token is not stored — length check guards mock branching only.
  void input.token;
  return Promise.resolve([
    { id: "appExampleBase01", name: "Example Projects & Tasks" },
    { id: "appExampleBase02", name: "Example Contacts" },
  ]);
}

function getBaseSchemaImpl(input: { token: string; baseId: string }): Promise<BaseSchemaSummary> {
  // Token is not stored. Mock schema keyed on baseId.
  const isContacts = input.baseId === "appExampleBase02";
  if (isContacts) {
    return Promise.resolve({
      baseId: input.baseId,
      tableCount: 1,
      tables: [
        {
          id: "tblContacts01",
          name: "Contacts",
          fieldCount: 4,
          fieldTypeCounts: [
            { fieldType: "email", count: 1 },
            { fieldType: "multilineText", count: 1 },
            { fieldType: "phoneNumber", count: 1 },
            { fieldType: "singleLineText", count: 1 },
          ],
          compatibility: {
            restorableCount: 4,
            metadataOnlyCount: 0,
            unknownCount: 0,
            totalCount: 4,
          },
        },
      ],
      compatibility: {
        restorableCount: 4,
        metadataOnlyCount: 0,
        unknownCount: 0,
        totalCount: 4,
      },
    });
  }

  return Promise.resolve({
    baseId: input.baseId,
    tableCount: 2,
    tables: [
      {
        id: "tblProjects01",
        name: "Projects",
        fieldCount: 5,
        fieldTypeCounts: [
          { fieldType: "date", count: 1 },
          { fieldType: "formula", count: 1 },
          { fieldType: "singleLineText", count: 2 },
          { fieldType: "singleSelect", count: 1 },
        ],
        compatibility: {
          restorableCount: 4,
          metadataOnlyCount: 1,
          unknownCount: 0,
          totalCount: 5,
        },
      },
      {
        id: "tblTasks01",
        name: "Tasks",
        fieldCount: 4,
        fieldTypeCounts: [
          { fieldType: "checkbox", count: 1 },
          { fieldType: "multipleRecordLinks", count: 1 },
          { fieldType: "rollup", count: 1 },
          { fieldType: "singleLineText", count: 1 },
        ],
        compatibility: {
          restorableCount: 2,
          metadataOnlyCount: 1,
          unknownCount: 1,
          totalCount: 4,
        },
      },
    ],
    compatibility: {
      restorableCount: 6,
      metadataOnlyCount: 2,
      unknownCount: 1,
      totalCount: 9,
    },
  });
}

function createBackupPlanImpl(request: BackupPlanRequest): Promise<BackupPlan> {
  // Build a deterministic mock plan from the supplied request.
  // Token is never present in BackupPlanRequest — no token handling needed here.
  const tablePlans = request.tables.map((t) => ({
    id: t.id,
    name: t.name,
    fieldCount: t.fields.length,
    recordCount: t.recordCount,
    fields: t.fields.map((f) => ({
      id: f.id,
      name: f.name,
      fieldType: f.fieldType,
      compatibility:
        f.fieldType === "formula" || f.fieldType === "rollup" || f.fieldType === "count"
          ? ("metadataOnly" as const)
          : f.fieldType === "multipleRecordLinks" || f.fieldType === "multipleAttachments"
            ? ("unknown" as const)
            : ("restorable" as const),
      attachmentPolicy:
        f.fieldType === "multipleAttachments" ? ("metadataOnly" as const) : undefined,
      linkedRecordPolicy:
        f.fieldType === "multipleRecordLinks"
          ? ("remappingRequiredForRestore" as const)
          : undefined,
    })),
    warnings: [
      ...(t.fields.some((f) => f.fieldType === "multipleAttachments")
        ? [
            {
              severity: "warning" as const,
              code: "ATTACHMENT_METADATA_ONLY",
              message:
                "Attachment field detected — attachment metadata only; file content is not exported.",
              tableName: t.name,
            },
          ]
        : []),
      ...(t.fields.some((f) => f.fieldType === "multipleRecordLinks")
        ? [
            {
              severity: "warning" as const,
              code: "LINKED_RECORD_REMAPPING",
              message:
                "Linked record field detected — record ID references captured; restore will require remapping.",
              tableName: t.name,
            },
          ]
        : []),
      ...(t.fields.some((f) => ["formula", "rollup", "count", "lookup"].includes(f.fieldType))
        ? [
            {
              severity: "info" as const,
              code: "COMPUTED_FIELD",
              message: "Computed field detected — schema captured, value cannot be restored.",
              tableName: t.name,
            },
          ]
        : []),
    ],
    compatibility: {
      restorableCount: t.fields.filter(
        (f) =>
          ![
            "formula",
            "rollup",
            "count",
            "lookup",
            "multipleRecordLinks",
            "multipleAttachments",
          ].includes(f.fieldType),
      ).length,
      metadataOnlyCount: t.fields.filter((f) =>
        ["formula", "rollup", "count", "lookup"].includes(f.fieldType),
      ).length,
      unknownCount: t.fields.filter((f) =>
        ["multipleRecordLinks", "multipleAttachments"].includes(f.fieldType),
      ).length,
      totalCount: t.fields.length,
    },
  }));

  const totalFields = tablePlans.reduce((s, t) => s + t.fieldCount, 0);
  const allWarnings = tablePlans.flatMap((t) => t.warnings);

  return Promise.resolve({
    baseId: request.baseId,
    baseName: request.baseName,
    scope: request.scope,
    tableCount: tablePlans.length,
    totalFieldCount: totalFields,
    tables: tablePlans,
    compatibility: {
      restorableCount: tablePlans.reduce((s, t) => s + t.compatibility.restorableCount, 0),
      metadataOnlyCount: tablePlans.reduce((s, t) => s + t.compatibility.metadataOnlyCount, 0),
      unknownCount: tablePlans.reduce((s, t) => s + t.compatibility.unknownCount, 0),
      totalCount: totalFields,
    },
    warnings: allWarnings,
    estimate: {
      schemaRequests: 1,
      recordReadPages: { type: "unknown" },
      note: "Record counts are unknown until records are fetched. Page estimates are approximate.",
    },
    dryRun: true,
  });
}

function createRecordsExportPlanImpl(
  request: RecordsExportPlanRequest,
): Promise<RecordsExportPlan> {
  const PAGE_SIZE = 100;

  const tablePlans: TableExportPlan[] = request.backupPlan.tables.map((t) => {
    const recordCount =
      t.recordCount != null
        ? { type: "known" as const, count: t.recordCount }
        : { type: "unknown" as const };

    const requestEstimate =
      recordCount.type === "known"
        ? {
            type: "known" as const,
            pages: recordCount.count === 0 ? 1 : Math.ceil(recordCount.count / PAGE_SIZE),
          }
        : { type: "unknown" as const };

    const linkedRecordPlans = t.fields
      .filter((f) => f.fieldType === "multipleRecordLinks")
      .map((f) => ({
        fieldId: f.id,
        fieldName: f.name,
        policy: "remappingRequiredForRestore" as const,
        restoreNote: "Record ID references are captured. Restore requires ID remapping.",
      }));

    const attachmentPlans = t.fields
      .filter((f) => f.fieldType === "multipleAttachments")
      .map((f) => ({
        fieldId: f.id,
        fieldName: f.name,
        policy: "metadataOnly" as const,
        contentNote:
          "Attachment file content is not exported. Only metadata (filename, URL, size) is captured.",
      }));

    const warnings: RecordsExportPlan["warnings"] = [];
    if (recordCount.type === "unknown") {
      warnings.push({
        severity: "warning",
        code: "UNKNOWN_RECORD_COUNT",
        message: "Record count is unknown. Actual pages will be determined at export time.",
        tableName: t.name,
      });
    }
    if (attachmentPlans.length > 0) {
      warnings.push({
        severity: "warning",
        code: "ATTACHMENT_METADATA_ONLY",
        message: "Attachment fields detected — only metadata will be exported, not file content.",
        tableName: t.name,
      });
    }
    if (linkedRecordPlans.length > 0) {
      warnings.push({
        severity: "warning",
        code: "LINKED_RECORD_REMAPPING",
        message: "Linked record references are captured. Restore will require ID remapping.",
        tableName: t.name,
      });
    }

    return {
      tableId: t.id,
      tableName: t.name,
      recordCount,
      requestEstimate,
      pageSize: PAGE_SIZE,
      jsonlOutput: {
        entryPath: `tables/${t.id}/records.jsonl`,
        plannedOnly: true,
      },
      tableMetadataPath: `tables/${t.id}/table.json`,
      fieldsMetadataPath: `tables/${t.id}/fields.json`,
      fields: t.fields.map((f) => ({
        fieldId: f.id,
        fieldName: f.name,
        fieldType: f.fieldType,
        compatibility:
          f.fieldType === "formula" || f.fieldType === "rollup" || f.fieldType === "count"
            ? ("metadataOnly" as const)
            : f.fieldType === "multipleRecordLinks" || f.fieldType === "multipleAttachments"
              ? ("unknown" as const)
              : ("restorable" as const),
        linkedRecordPlan:
          f.fieldType === "multipleRecordLinks"
            ? {
                fieldId: f.id,
                fieldName: f.name,
                policy: "remappingRequiredForRestore" as const,
                restoreNote: "Record ID references are captured. Restore requires ID remapping.",
              }
            : undefined,
        attachmentPlan:
          f.fieldType === "multipleAttachments"
            ? {
                fieldId: f.id,
                fieldName: f.name,
                policy: "metadataOnly" as const,
                contentNote:
                  "Attachment file content is not exported. Only metadata (filename, URL, size) is captured.",
              }
            : undefined,
      })),
      linkedRecordPlans,
      attachmentPlans,
      warnings,
    };
  });

  const allWarnings = tablePlans.flatMap((t) => t.warnings);

  return Promise.resolve({
    baseId: request.baseId,
    baseName: request.baseName,
    tableCount: tablePlans.length,
    pageSize: PAGE_SIZE,
    tables: tablePlans,
    warnings: allWarnings,
    plannedOnly: true,
  });
}

function validateBackupOutputPathImpl(path: string): Promise<OutputPathValidationResult> {
  if (!path || path.length === 0) {
    return Promise.resolve({
      valid: false,
      errorCode: "EMPTY_PATH",
      errorMessage: "output path must not be empty",
    });
  }
  if (!path.endsWith(".airbridge")) {
    return Promise.resolve({
      valid: false,
      errorCode: "WRONG_EXTENSION",
      errorMessage: "output path must have a .airbridge extension",
    });
  }
  return Promise.resolve({ valid: true });
}

function runBackupJobImpl(request: RunBackupCommandRequest): Promise<RunBackupCommandResponse> {
  // Mock service: validates confirmation and path, but does NOT write any file.
  void request.token; // token is not stored

  if (request.confirmation !== "CREATE BACKUP") {
    return Promise.resolve({
      success: false,
      safetyErrors: [
        {
          code: "CONFIRMATION_REQUIRED",
          message: 'confirmation must be the exact phrase "CREATE BACKUP"',
        },
      ],
      pathValidation: { valid: true },
    });
  }

  if (!request.outputPath.endsWith(".airbridge")) {
    return Promise.resolve({
      success: false,
      safetyErrors: [{ code: "INVALID_OUTPUT_PATH", message: "output path validation failed" }],
      pathValidation: {
        valid: false,
        errorCode: "WRONG_EXTENSION",
        errorMessage: "output path must have a .airbridge extension",
      },
    });
  }

  // Return a safe mock succeeded response — no file is written.
  return Promise.resolve({
    success: true,
    packageFilename: request.outputPath.split("/").pop() ?? "mock-backup.airbridge",
    safetyErrors: [],
    jobResult: {
      jobId: request.jobId ?? "mock-job-001",
      status: "succeeded",
      baseId: request.baseId,
      baseName: request.baseName,
      tables: [],
      warnings: [],
      errors: [],
    },
    pathValidation: { valid: true },
  });
}

function cancelBackupJobImpl(jobId: string): Promise<BackupJobCancellationResult> {
  // Mock: no background registry — always returns not_running.
  return Promise.resolve({
    jobId,
    wasRunning: false,
    statusAtCancellation: "not_running",
  });
}

function getBackupJobStatusImpl(jobId: string): Promise<BackupJobProgressSnapshot | null> {
  // Mock: synchronous execution — no live snapshot available.
  void jobId;
  return Promise.resolve(null);
}

function inspectBackupPackageImpl(path: string): Promise<BackupPackageInspectionResult> {
  // Deterministic mock variants: "invalid" in path returns an invalid result;
  // otherwise returns a valid inspection result. No absolute paths appear in output.
  const filename = path.split("/").pop()?.split("\\").pop() ?? "unknown.airbridge";

  if (path.includes("invalid") || path.includes("corrupt")) {
    return Promise.resolve({
      filename,
      validationStatus: "invalid",
      entryCount: 0,
      warnings: [],
      errors: [{ code: "CANNOT_OPEN", message: "package could not be opened" }],
    });
  }

  return Promise.resolve({
    filename,
    validationStatus: "valid",
    manifest: {
      format: "airbridge",
      formatVersion: "0.1.0",
      appVersion: "0.1.0",
      createdAt: "2026-06-11T00:00:00Z",
      provider: "airtable",
      baseId: "appExampleBase01",
      baseName: "Example Projects & Tasks",
    },
    contents: {
      tableCount: 2,
      fieldCount: 9,
      recordCount: 47,
      linkedRecordRelationshipCount: 1,
      attachmentCount: 0,
    },
    security: {
      encrypted: false,
      containsRecordData: true,
      containsAttachmentUrls: false,
      redactionsApplied: [],
    },
    checksums: {
      checksumCount: 5,
      allValid: true,
    },
    entryCount: 8,
    warnings: [],
    errors: [],
  });
}

function createRestoreDryRunPlanImpl(request: RestoreDryRunRequest): Promise<RestoreDryRunPlan> {
  // Deterministic mock — no absolute paths appear in the result.
  // Filename is derived from path safely via basename extraction.
  const filename = request.path.split("/").pop()?.split("\\").pop() ?? "mock-backup.airbridge";

  if (request.path.includes("invalid") || request.path.includes("corrupt")) {
    return Promise.resolve({
      filename,
      status: "blocked",
      targetMode: request.targetMode,
      targetBaseName: request.targetBaseName,
      tables: [],
      warnings: [],
      errors: [{ code: "CANNOT_OPEN", message: "Package could not be opened or validated." }],
      noChangesMade: true,
    });
  }

  const plan: RestoreDryRunPlan = {
    filename,
    status: "readyWithWarnings",
    targetMode: request.targetMode,
    targetBaseName: request.targetBaseName ?? "Example Projects & Tasks (Restored)",
    packageSummary: {
      filename,
      format: "airbridge",
      formatVersion: "0.1.0",
      appVersion: "0.1.0",
      createdAt: "2026-06-11T00:00:00Z",
      provider: "airtable",
      baseId: "appExampleBase01",
      baseName: "Example Projects & Tasks",
      tableCount: 2,
      fieldCount: 9,
      recordCount: 47,
      containsRecordData: true,
      containsAttachmentUrls: false,
      encrypted: false,
    },
    tables: [
      {
        tableId: "tblProjects01",
        tableName: "Projects",
        fieldCount: 5,
        recordCount: 0,
        fields: [
          {
            fieldId: "fldProjName",
            fieldName: "Name",
            fieldType: "singleLineText",
            compatibility: "supported",
            note: "Fully restorable.",
          },
          {
            fieldId: "fldProjStatus",
            fieldName: "Status",
            fieldType: "singleSelect",
            compatibility: "supported",
            note: "Fully restorable.",
          },
          {
            fieldId: "fldProjDue",
            fieldName: "Due Date",
            fieldType: "date",
            compatibility: "supported",
            note: "Fully restorable.",
          },
          {
            fieldId: "fldProjTasks",
            fieldName: "Tasks",
            fieldType: "multipleRecordLinks",
            compatibility: "partiallySupported",
            note: "Linked record references require remapping during restore.",
          },
          {
            fieldId: "fldProjCalc",
            fieldName: "Calculated Progress",
            fieldType: "formula",
            compatibility: "unsupported",
            note: "Formula fields cannot be restored. The formula definition is captured but the field must be manually recreated.",
          },
        ],
        linkedRecordPlans: [
          {
            fieldId: "fldProjTasks",
            fieldName: "Tasks",
            linkedTableId: "tblTasks01",
            remappingRequired: true,
            note: "Links to Tasks table. Record IDs must be remapped after all records are imported.",
          },
        ],
        attachmentPlans: [],
        restorableFieldCount: 3,
        partialFieldCount: 1,
        unsupportedFieldCount: 1,
      },
      {
        tableId: "tblTasks01",
        tableName: "Tasks",
        fieldCount: 4,
        recordCount: 0,
        fields: [
          {
            fieldId: "fldTaskName",
            fieldName: "Task Name",
            fieldType: "singleLineText",
            compatibility: "supported",
            note: "Fully restorable.",
          },
          {
            fieldId: "fldTaskDone",
            fieldName: "Done",
            fieldType: "checkbox",
            compatibility: "supported",
            note: "Fully restorable.",
          },
          {
            fieldId: "fldTaskFiles",
            fieldName: "Attachments",
            fieldType: "multipleAttachments",
            compatibility: "metadataOnly",
            note: "Attachment metadata is captured. File content was not exported and cannot be restored.",
          },
          {
            fieldId: "fldTaskRollup",
            fieldName: "Completion Rollup",
            fieldType: "rollup",
            compatibility: "metadataOnly",
            note: "Computed field. Schema is captured; values are not restored.",
          },
        ],
        linkedRecordPlans: [],
        attachmentPlans: [
          {
            fieldId: "fldTaskFiles",
            fieldName: "Attachments",
            metadataOnly: true,
            note: "Attachment metadata is captured. File content was not exported and cannot be re-uploaded.",
          },
        ],
        restorableFieldCount: 2,
        partialFieldCount: 0,
        unsupportedFieldCount: 2,
      },
    ],
    ordering: {
      createTablesFirst: true,
      createFieldsAfterTables: true,
      importRecordsWithoutLinks: true,
      applyLinksAfterRecords: true,
      note: "Tables and fields are created first. Records are imported without linked references. Linked references are applied in a second pass after all record IDs are remapped.",
    },
    warnings: [
      {
        code: "LINKED_RECORD_REMAPPING_REQUIRED",
        message: "Linked record field requires ID remapping during restore.",
        tableName: "Projects",
        fieldName: "Tasks",
      },
      {
        code: "ATTACHMENT_METADATA_ONLY",
        message:
          "Attachment field detected. Only metadata is available — file content was not exported.",
        tableName: "Tasks",
        fieldName: "Attachments",
      },
      {
        code: "COMPUTED_FIELD_NOT_RESTORED",
        message: "Rollup field values cannot be restored. Schema only.",
        tableName: "Tasks",
        fieldName: "Completion Rollup",
      },
      {
        code: "UNSUPPORTED_FIELD_MANUAL_RECREATION",
        message: "Formula field cannot be restored. Recreate manually after import.",
        tableName: "Projects",
        fieldName: "Calculated Progress",
      },
    ],
    errors: [],
    noChangesMade: true,
  };

  return Promise.resolve(plan);
}

export const MOCK_RESTORE_CONFIRMATION = "RESTORE BACKUP";

function runRestoreExecutionImpl(
  request: RestoreExecutionRequest,
): Promise<RestoreExecutionResult> {
  // Token is not stored. Checked for presence only.
  const filename =
    request.packageFilename ||
    request.packagePath.split("/").pop()?.split("\\").pop() ||
    "unknown.airbridge";

  // Missing package inspection
  if (!request.packageFilename) {
    return Promise.resolve({
      filename,
      status: "blocked",
      blockReason: "missingPackageInspection",
      message: "No package has been inspected. Select and inspect a backup package first.",
      warnings: [],
      errors: [{ code: "GATE_BLOCKED", message: "Missing package inspection." }],
      noChangesMade: true,
    });
  }

  // Invalid package
  if (
    request.packageValidationStatus !== "valid" &&
    request.packageValidationStatus !== "warning"
  ) {
    return Promise.resolve({
      filename,
      status: "blocked",
      blockReason: "invalidPackage",
      message: "The selected package is invalid or has not been inspected.",
      warnings: [],
      errors: [{ code: "GATE_BLOCKED", message: "Invalid package." }],
      noChangesMade: true,
    });
  }

  // Missing dry-run
  if (!request.dryRunStatus) {
    return Promise.resolve({
      filename,
      status: "blocked",
      blockReason: "missingDryRunPlan",
      message:
        "A restore plan preview must be generated before restore execution can be attempted.",
      warnings: [],
      errors: [{ code: "GATE_BLOCKED", message: "Missing dry-run plan." }],
      noChangesMade: true,
    });
  }

  // Blocked dry-run
  if (request.dryRunStatus !== "ready" && request.dryRunStatus !== "readyWithWarnings") {
    return Promise.resolve({
      filename,
      status: "blocked",
      blockReason: "dryRunBlocked",
      message: "The restore plan is blocked. Resolve all errors in the plan before proceeding.",
      warnings: [],
      errors: [{ code: "GATE_BLOCKED", message: "Dry-run plan is blocked." }],
      noChangesMade: true,
    });
  }

  // Missing token
  if (!request.token) {
    return Promise.resolve({
      filename,
      status: "blocked",
      blockReason: "missingToken",
      message:
        "An Airtable personal access token is required. The token is used only for this operation and is not stored.",
      warnings: [],
      errors: [{ code: "GATE_BLOCKED", message: "Missing token." }],
      noChangesMade: true,
    });
  }

  // Wrong confirmation
  if (request.confirmation !== MOCK_RESTORE_CONFIRMATION) {
    return Promise.resolve({
      filename,
      status: "blocked",
      blockReason: "missingConfirmation",
      message: "Confirmation text does not match. Type the exact phrase to proceed.",
      warnings: [],
      errors: [{ code: "GATE_BLOCKED", message: "Missing or incorrect confirmation." }],
      noChangesMade: true,
    });
  }

  // All gates pass — write engine not enabled.
  return Promise.resolve({
    filename,
    status: "readyButDisabled",
    blockReason: "restoreWriteEngineNotEnabled",
    message:
      "Restore execution contract is ready, but the write engine is not enabled in this version. No Airtable changes were made.",
    warnings: [
      {
        code: "WRITE_ENGINE_DISABLED",
        message:
          "The restore write engine is not enabled. Schema creation and record import will be available in a future version.",
      },
    ],
    errors: [],
    noChangesMade: true,
  });
}

function createRestoreSchemaPlanImpl(
  request: RestoreSchemaPlanRequest,
): Promise<RestoreSchemaPlan> {
  const filename = request.packageFilename;

  if (request.dryRunStatus === "blocked" || !request.dryRunStatus) {
    return Promise.resolve({
      filename,
      status: "blocked",
      targetMode: request.targetMode,
      tableSteps: [],
      fieldSteps: [],
      deferredSteps: [],
      manualActionFields: [],
      dependencyGraph: { edges: [], hasCircularDependency: false, resolutionNote: "" },
      warnings: [],
      errors: [{ code: "DRY_RUN_BLOCKED", message: "Dry-run plan is blocked." }],
      noChangesMade: true,
    });
  }

  return Promise.resolve({
    filename,
    status: "readyWithWarnings",
    targetMode: request.targetMode,
    targetBaseName: request.targetBaseName,
    tableSteps: [
      {
        tableId: "tblMockProjects",
        tableName: "Projects",
        stepIndex: 0,
        fieldCount: 5,
        directFieldCount: 2,
        deferredFieldCount: 1,
        manualActionCount: 1,
        unsupportedCount: 1,
        note: "Create table 'Projects': 2 direct, 1 deferred, 1 manual, 1 unsupported.",
      },
      {
        tableId: "tblMockTasks",
        tableName: "Tasks",
        stepIndex: 1,
        fieldCount: 2,
        directFieldCount: 2,
        deferredFieldCount: 0,
        manualActionCount: 0,
        unsupportedCount: 0,
        note: "Create table 'Tasks': 2 direct, 0 deferred, 0 manual, 0 unsupported.",
      },
    ],
    fieldSteps: [
      {
        fieldId: "fldMockName",
        fieldName: "Name",
        fieldType: "singleLineText",
        tableId: "tblMockProjects",
        tableName: "Projects",
        classification: "createDirectly",
        note: "'singleLineText' can be created directly via the Airtable API.",
      },
      {
        fieldId: "fldMockStatus",
        fieldName: "Status",
        fieldType: "singleSelect",
        tableId: "tblMockProjects",
        tableName: "Projects",
        classification: "createDirectly",
        note: "'singleSelect' can be created directly via the Airtable API.",
      },
      {
        fieldId: "fldMockFiles",
        fieldName: "Files",
        fieldType: "multipleAttachments",
        tableId: "tblMockProjects",
        tableName: "Projects",
        classification: "metadataOnly",
        note: "Attachment metadata is captured. File content is not re-uploaded.",
      },
    ],
    deferredSteps: [
      {
        fieldId: "fldMockTasks",
        fieldName: "Tasks",
        fieldType: "multipleRecordLinks",
        tableId: "tblMockProjects",
        tableName: "Projects",
        reason: "Linked record field — deferred until all tables and records exist.",
        linkedTableId: "tblMockTasks",
      },
    ],
    manualActionFields: [
      {
        fieldId: "fldMockCalc",
        fieldName: "Calculated Total",
        fieldType: "formula",
        tableId: "tblMockProjects",
        tableName: "Projects",
        actionDescription:
          "'formula' must be recreated manually — cannot be set via the Airtable API.",
      },
    ],
    dependencyGraph: {
      edges: [
        {
          fieldId: "fldMockTasks",
          fieldName: "Tasks",
          sourceTableId: "tblMockProjects",
          sourceTableName: "Projects",
          targetTableId: "tblMockTasks",
          targetTableName: "Tasks",
          remappingRequired: true,
          note: "Field 'Tasks' in 'Projects' links to 'Tasks'. Record IDs must be remapped after import.",
        },
      ],
      hasCircularDependency: false,
      resolutionNote:
        "Linked record fields are deferred and applied after all tables and records are imported.",
    },
    warnings: [
      {
        code: "ATTACHMENT_METADATA_ONLY",
        message:
          "Attachment fields are present. File content is not re-uploaded; only metadata is captured.",
        tableName: "Projects",
      },
      {
        code: "LINKED_FIELDS_DEFERRED",
        message: "1 linked record field(s) in 'Projects' will be deferred until all tables exist.",
        tableName: "Projects",
      },
      {
        code: "UNSUPPORTED_FIELDS_REQUIRE_MANUAL_RECREATION",
        message:
          "1 field(s) in 'Projects' cannot be created via the API and must be recreated manually.",
        tableName: "Projects",
      },
    ],
    errors: [],
    noChangesMade: true,
  });
}

export const mockCheckConnection = checkConnectionImpl;

function makeLinkedField(id: string, name: string, linkedTableId: string): RecordImportFieldInput {
  return { fieldId: id, fieldName: name, fieldType: "multipleRecordLinks", linkedTableId };
}

function makePrimitiveField(id: string, name: string, fieldType: string): RecordImportFieldInput {
  return { fieldId: id, fieldName: name, fieldType, linkedTableId: undefined };
}

function makeAttachmentField(id: string, name: string): RecordImportFieldInput {
  return {
    fieldId: id,
    fieldName: name,
    fieldType: "multipleAttachments",
    linkedTableId: undefined,
  };
}

function makeComputedField(id: string, name: string): RecordImportFieldInput {
  return { fieldId: id, fieldName: name, fieldType: "formula", linkedTableId: undefined };
}

function createRestoreRecordImportPlanImpl(
  request: RestoreRecordImportPlanRequest,
): Promise<RestoreRecordImportPlan> {
  const batchSize = 10;

  const knownTable: RecordImportTableInput = {
    tableId: "tblMock01",
    tableName: "Projects",
    recordCount: 25,
    fields: [
      makePrimitiveField("fld001", "Name", "singleLineText"),
      makePrimitiveField("fld002", "Status", "singleSelect"),
      makeLinkedField("fld003", "Tasks", "tblMock02"),
      makeAttachmentField("fld004", "Files"),
      makeComputedField("fld005", "Summary"),
    ],
  };

  const unknownTable: RecordImportTableInput = {
    tableId: "tblMock02",
    tableName: "Tasks",
    recordCount: undefined,
    fields: [
      makePrimitiveField("fld006", "Title", "singleLineText"),
      makePrimitiveField("fld007", "Done", "checkbox"),
    ],
  };

  const BATCH_SIZE = batchSize;
  const knownCount = knownTable.recordCount as number;
  const knownBatchCount = Math.ceil(knownCount / BATCH_SIZE);

  const plan: RestoreRecordImportPlan = {
    filename: request.packageFilename,
    status: "readyWithWarnings",
    targetMode: request.targetMode,
    targetBaseName: request.targetBaseName,
    tablePlans: [
      {
        tableId: knownTable.tableId,
        tableName: knownTable.tableName,
        importOrder: 0,
        recordCount: knownCount,
        recordCountKnown: true,
        batchSize: BATCH_SIZE,
        createBatchCount: knownBatchCount,
        updateBatchCount: knownBatchCount,
        firstPassBatches: Array.from({ length: knownBatchCount }, (_, i) => ({
          batchIndex: i,
          phase: "createRecords" as const,
          recordCount: i < knownBatchCount - 1 ? BATCH_SIZE : knownCount % BATCH_SIZE || BATCH_SIZE,
          note: `Batch ${i + 1} of ${knownBatchCount}`,
        })),
        secondPassBatches: Array.from({ length: knownBatchCount }, (_, i) => ({
          batchIndex: i,
          phase: "updateLinkedRecords" as const,
          recordCount: i < knownBatchCount - 1 ? BATCH_SIZE : knownCount % BATCH_SIZE || BATCH_SIZE,
          note: `Linked record update batch ${i + 1} of ${knownBatchCount}`,
        })),
        fieldPolicies: [
          {
            fieldId: "fld001",
            fieldName: "Name",
            fieldType: "singleLineText",
            policy: "include",
            note: "",
          },
          {
            fieldId: "fld002",
            fieldName: "Status",
            fieldType: "singleSelect",
            policy: "include",
            note: "",
          },
          {
            fieldId: "fld003",
            fieldName: "Tasks",
            fieldType: "multipleRecordLinks",
            policy: "deferToLinkedRecordPass",
            note: "",
          },
          {
            fieldId: "fld004",
            fieldName: "Files",
            fieldType: "multipleAttachments",
            policy: "metadataOnly",
            note: "",
          },
          {
            fieldId: "fld005",
            fieldName: "Summary",
            fieldType: "formula",
            policy: "skip",
            note: "",
          },
        ],
        attachmentPolicies: [
          {
            tableId: knownTable.tableId,
            tableName: knownTable.tableName,
            fieldId: "fld004",
            fieldName: "Files",
            policy: "metadataOnly",
            note: "Attachment metadata captured; files must be manually re-attached.",
          },
        ],
        mappingPlan: {
          tableId: knownTable.tableId,
          tableName: knownTable.tableName,
          strategy: "mapSourceRecordIdToCreatedRecordId",
          remappingRequired: true,
          note: "Linked record fields require ID remapping after first pass.",
        },
        checkpointPlan: {
          tableId: knownTable.tableId,
          tableName: knownTable.tableName,
          checkpointBatchIndex: 0,
          sourceRecordIdOffsetPlaceholder: "<source_record_id_at_checkpoint>",
          completedPhase: "createRecords",
          note: "",
        },
        linkedRecordUpdates: [
          {
            tableId: knownTable.tableId,
            tableName: knownTable.tableName,
            fieldId: "fld003",
            fieldName: "Tasks",
            linkedTableId: "tblMock02",
            linkedTableName: "Tasks",
            updateBatchCount: knownBatchCount,
            note: "Second pass update after ID mapping.",
          },
        ],
      },
      {
        tableId: unknownTable.tableId,
        tableName: unknownTable.tableName,
        importOrder: 1,
        recordCount: undefined,
        recordCountKnown: false,
        batchSize: BATCH_SIZE,
        createBatchCount: undefined,
        updateBatchCount: undefined,
        firstPassBatches: [],
        secondPassBatches: [],
        fieldPolicies: [
          {
            fieldId: "fld006",
            fieldName: "Title",
            fieldType: "singleLineText",
            policy: "include",
            note: "",
          },
          {
            fieldId: "fld007",
            fieldName: "Done",
            fieldType: "checkbox",
            policy: "include",
            note: "",
          },
        ],
        attachmentPolicies: [],
        mappingPlan: {
          tableId: unknownTable.tableId,
          tableName: unknownTable.tableName,
          strategy: "mapSourceRecordIdToCreatedRecordId",
          remappingRequired: false,
          note: "No linked record fields — no remapping required.",
        },
        checkpointPlan: {
          tableId: unknownTable.tableId,
          tableName: unknownTable.tableName,
          checkpointBatchIndex: 0,
          sourceRecordIdOffsetPlaceholder: "<source_record_id_at_checkpoint>",
          completedPhase: "createRecords",
          note: "Batch count unknown until import begins.",
        },
        linkedRecordUpdates: [],
      },
    ],
    linkedRecordUpdatePlans: [
      {
        tableId: knownTable.tableId,
        tableName: knownTable.tableName,
        fieldId: "fld003",
        fieldName: "Tasks",
        linkedTableId: "tblMock02",
        linkedTableName: "Tasks",
        updateBatchCount: knownBatchCount,
        note: "Second pass update after all records created.",
      },
    ],
    retryPolicy: {
      maxRetriesOnRateLimit: 5,
      initialBackoffMs: 1000,
      backoffMultiplier: 2,
      note: "Exponential backoff on rate-limit responses.",
    },
    warnings: [
      {
        code: "RECORD_COUNT_UNKNOWN",
        message: "Record count for 'Tasks' is not available.",
        tableName: "Tasks",
      },
      {
        code: "ATTACHMENT_METADATA_ONLY",
        message: "'Projects' contains attachment fields. Files must be manually re-attached.",
        tableName: "Projects",
      },
      {
        code: "COMPUTED_FIELDS_SKIPPED",
        message: "1 field(s) in 'Projects' will be skipped.",
        tableName: "Projects",
      },
      {
        code: "LINKED_RECORD_SECOND_PASS_REQUIRED",
        message: "'Projects' has 1 linked record field(s) requiring a second pass.",
        tableName: "Projects",
      },
    ],
    errors: [],
    noChangesMade: true,
  };

  void knownTable;
  void unknownTable;
  return Promise.resolve(plan);
}

function makeMockHistoryItem(
  id: string,
  kind: JobHistoryKind,
  status: JobHistoryStatus,
  source: JobHistorySource,
  title: string,
  packageFilename: string | undefined,
  baseName: string | undefined,
  warningCount: number,
  errorCount: number,
  validationStatus: string | undefined,
  finishedAt: string,
  noChangesMade: boolean,
): JobHistoryItem {
  return {
    id: { 0: id },
    kind,
    status,
    source,
    startedAt: undefined,
    finishedAt,
    summary: {
      title,
      detail: undefined,
      packageFilename,
      baseName,
      warningCount,
      errorCount,
      validationStatus,
    },
    warnings: [],
    errors: errorCount > 0 ? [{ code: "MOCK_ERROR", message: "Mock error" }] : [],
    noChangesMade,
  };
}

const MOCK_HISTORY_ITEMS: JobHistoryItem[] = [
  makeMockHistoryItem(
    "hist-001",
    "backupExecution",
    "succeeded",
    "backupPage",
    "Backup execution",
    "my-base-2026-06-10.airbridge",
    "My Base",
    0,
    0,
    undefined,
    "2026-06-10T09:01:12Z",
    false,
  ),
  makeMockHistoryItem(
    "hist-002",
    "packageInspection",
    "succeeded",
    "restorePage",
    "Package inspection",
    "my-base-2026-06-10.airbridge",
    undefined,
    0,
    0,
    "valid",
    "2026-06-10T09:05:00Z",
    true,
  ),
  makeMockHistoryItem(
    "hist-003",
    "restoreDryRun",
    "succeededWithWarnings",
    "restorePage",
    "Restore dry-run plan",
    "my-base-2026-06-10.airbridge",
    undefined,
    2,
    0,
    undefined,
    "2026-06-10T09:06:00Z",
    true,
  ),
  makeMockHistoryItem(
    "hist-004",
    "restoreSchemaplan",
    "succeeded",
    "restorePage",
    "Restore schema creation plan",
    "my-base-2026-06-10.airbridge",
    undefined,
    0,
    0,
    undefined,
    "2026-06-10T09:07:00Z",
    true,
  ),
  makeMockHistoryItem(
    "hist-005",
    "restoreRecordImportPlan",
    "succeededWithWarnings",
    "restorePage",
    "Restore record import plan",
    "my-base-2026-06-10.airbridge",
    undefined,
    3,
    0,
    undefined,
    "2026-06-10T09:08:00Z",
    true,
  ),
  makeMockHistoryItem(
    "hist-006",
    "restoreExecutionAttempt",
    "blocked",
    "restorePage",
    "Restore execution attempt (blocked)",
    "my-base-2026-06-10.airbridge",
    undefined,
    0,
    1,
    undefined,
    "2026-06-10T09:09:00Z",
    true,
  ),
];

function listJobHistoryImpl(filter?: JobHistoryFilter): Promise<JobHistoryListResult> {
  let items = [...MOCK_HISTORY_ITEMS].reverse();
  const filtered = !!(filter?.kind || filter?.status);
  if (filter?.kind) {
    items = items.filter((i) => i.kind === filter.kind);
  }
  if (filter?.status) {
    items = items.filter((i) => i.status === filter.status);
  }
  const totalCount = items.length;
  if (filter?.limit) {
    items = items.slice(0, filter.limit);
  }
  return Promise.resolve({ items, totalCount, filtered });
}

function clearJobHistoryImpl(): Promise<number> {
  return Promise.resolve(0);
}

function previewRestoreWriteEngineImpl(
  request: RestoreWriteEngineRequest,
): Promise<RestoreWriteEngineResult> {
  return Promise.resolve({
    filename: request.packageFilename,
    status: "disabled" as const,
    disabledReason: "disabledByProductPolicy" as const,
    message:
      "Restore write execution is not enabled in this version. No Airtable changes are made.",
    phaseSummaries: [
      {
        phase: "validateInputs" as const,
        status: "disabled" as const,
        noChangesMade: true,
        note: "Input validation completed. Write engine is disabled.",
      },
      {
        phase: "schemaCreation" as const,
        status: "disabled" as const,
        noChangesMade: true,
        note: `Schema creation disabled. Would create ${request.schemaTableCount ?? 0} table(s).`,
      },
      {
        phase: "recordCreation" as const,
        status: "disabled" as const,
        noChangesMade: true,
        note: `Record import disabled. ${request.estimatedFirstPassBatches ?? 0} first-pass batch(es) planned.`,
      },
      {
        phase: "linkedRecordUpdates" as const,
        status: "disabled" as const,
        noChangesMade: true,
        note: "Linked record updates disabled.",
      },
      {
        phase: "attachmentHandling" as const,
        status: "disabled" as const,
        noChangesMade: true,
        note: "Attachment handling disabled. Policy: MetadataOnly.",
      },
      {
        phase: "finalValidation" as const,
        status: "disabled" as const,
        noChangesMade: true,
        note: "Final validation not executed — write engine is disabled.",
      },
    ],
    events: [
      {
        phase: "validateInputs" as const,
        code: "WRITE_ENGINE_DISABLED",
        message: "Write engine is disabled by product policy.",
      },
    ],
    noChangesMade: true,
  });
}

// In-memory mock state for credential storage — test-only, never persisted
const _mockCredentialStore: Map<CredentialKind, boolean> = new Map();

function getCredentialStorageStatusImpl(
  request: CredentialStatusRequest,
): Promise<CredentialStatusResult> {
  const hasSaved = _mockCredentialStore.get(request.kind) === true;
  return Promise.resolve({
    kind: request.kind,
    status: hasSaved ? ("saved" as const) : ("notSaved" as const),
    availability: "available" as const,
    hasSavedToken: hasSaved,
    display: hasSaved ? "Saved token present" : "No saved token",
  });
}

function saveAirtableTokenToKeychainImpl(
  request: CredentialSaveRequest,
): Promise<CredentialSaveResult> {
  // Token is accepted but never stored in plaintext — only presence is recorded.
  // In a real implementation the token goes to the OS keychain.
  if (!request.token || request.token.trim().length === 0) {
    return Promise.resolve({
      kind: request.kind,
      success: false,
      hasSavedToken: false,
      display: "Token must not be empty.",
      errorMessage: "Token must not be empty.",
    });
  }
  _mockCredentialStore.set(request.kind, true);
  return Promise.resolve({
    kind: request.kind,
    success: true,
    hasSavedToken: true,
    display: "Saved token present",
    errorMessage: null,
  });
}

function removeAirtableTokenFromKeychainImpl(
  request: CredentialRemoveRequest,
): Promise<CredentialRemoveResult> {
  _mockCredentialStore.set(request.kind, false);
  return Promise.resolve({
    kind: request.kind,
    success: true,
    hasSavedToken: false,
    display: "No saved token",
    errorMessage: null,
  });
}

function previewSchemaWriteRequestPlanImpl(
  request: SchemaWriteRequestPlanRequest,
): Promise<SchemaWriteRequestPlanResult> {
  const filename = request.packageFilename;

  if (request.schemaPlanStatus === "blocked") {
    return Promise.resolve({
      filename,
      status: "blocked" as const,
      blockedReason: "schemaPlanNotReady" as const,
      message: "Schema plan is not ready — cannot build write request plan.",
      tableOpCount: 0,
      fieldOpCount: 0,
      deferredOpCount: 0,
      manualActionCount: 0,
      totalOpCount: 0,
      warnings: [],
      noChangesMade: true,
      networkWritesAttempted: false,
    });
  }

  if (request.tableCount === 0) {
    return Promise.resolve({
      filename,
      status: "blocked" as const,
      blockedReason: "noTablesInPlan" as const,
      message: "No tables in schema plan — nothing to write.",
      tableOpCount: 0,
      fieldOpCount: 0,
      deferredOpCount: 0,
      manualActionCount: 0,
      totalOpCount: 0,
      warnings: [],
      noChangesMade: true,
      networkWritesAttempted: false,
    });
  }

  const tableOpCount = request.tableCount;
  const fieldOpCount = request.directFieldCount;
  const deferredOpCount = request.deferredFieldCount;
  const manualActionCount = request.manualActionCount;
  const totalOpCount = tableOpCount + fieldOpCount + deferredOpCount + manualActionCount;

  return Promise.resolve({
    filename,
    status: "disabled" as const,
    disabledReason: "disabledByProductPolicy",
    message:
      "Restore write execution is not enabled in this version. Schema creation and record import are planning-only operations. No Airtable changes are made.",
    tableOpCount,
    fieldOpCount,
    deferredOpCount,
    manualActionCount,
    totalOpCount,
    warnings: [],
    noChangesMade: true,
    networkWritesAttempted: false,
  });
}

function previewRecordWriteRequestPlanImpl(
  request: RecordWriteRequestPlanRequest,
): Promise<RecordWriteRequestPlanResult> {
  const filename = request.packageFilename;

  if (request.recordImportPlanStatus === "blocked") {
    return Promise.resolve({
      filename,
      status: "blocked" as const,
      blockedReason: "recordImportPlanNotReady" as const,
      message: "Record import plan is not ready — cannot build record write request plan.",
      createBatchOpCount: 0,
      linkedUpdateOpCount: 0,
      checkpointOpCount: 0,
      attachmentOpCount: 0,
      skippedFieldOpCount: 0,
      totalOpCount: 0,
      totalFirstPassBatches: 0,
      totalSecondPassBatches: 0,
      warnings: [],
      noChangesMade: true,
      networkWritesAttempted: false,
    });
  }

  if (request.tableCount === 0) {
    return Promise.resolve({
      filename,
      status: "blocked" as const,
      blockedReason: "noTablesInPlan" as const,
      message: "No tables in record import plan — nothing to write.",
      createBatchOpCount: 0,
      linkedUpdateOpCount: 0,
      checkpointOpCount: 0,
      attachmentOpCount: 0,
      skippedFieldOpCount: 0,
      totalOpCount: 0,
      totalFirstPassBatches: 0,
      totalSecondPassBatches: 0,
      warnings: [],
      noChangesMade: true,
      networkWritesAttempted: false,
    });
  }

  const createBatchOpCount = request.totalFirstPassBatches;
  const linkedUpdateOpCount = request.totalSecondPassBatches;
  const checkpointOpCount = request.tableCount;
  const attachmentOpCount = request.attachmentFieldCount;
  const skippedFieldOpCount = request.skippedFieldCount;
  const totalOpCount =
    createBatchOpCount +
    linkedUpdateOpCount +
    checkpointOpCount +
    attachmentOpCount +
    skippedFieldOpCount;

  return Promise.resolve({
    filename,
    status: "disabled" as const,
    disabledReason: "disabledByProductPolicy",
    message:
      "Restore write execution is not enabled in this version. Record import is a planning-only operation. No Airtable changes are made.",
    createBatchOpCount,
    linkedUpdateOpCount,
    checkpointOpCount,
    attachmentOpCount,
    skippedFieldOpCount,
    totalOpCount,
    totalFirstPassBatches: request.totalFirstPassBatches,
    totalSecondPassBatches: request.totalSecondPassBatches,
    warnings: [],
    noChangesMade: true,
    networkWritesAttempted: false,
  });
}

function verifyRestoreSandboxEnvironmentImpl(
  request: SandboxVerificationRequest,
): Promise<SandboxVerificationResult> {
  const isUnsafe = request.allowDestructiveOperations || !request.expectsEmptyTarget;

  if (isUnsafe) {
    return Promise.resolve({
      status: "blocked" as const,
      checks: [
        {
          checkId: "CHK-04",
          label: "No Destructive Operations",
          status: "failed" as const,
          message: "Destructive operations are not permitted in sandbox mode.",
          remediation: "Set allowDestructiveOperations to false.",
        },
      ],
      safetySummary: {
        writesEnabled: false,
        networkWritesAttempted: false,
        noChangesMade: true,
        writeGateStatus: "disabled",
        liveMetadataCheckPerformed: false,
      },
      message:
        "Sandbox verification blocked: unsafe target configuration. writesEnabled is always false.",
      noChangesMade: true,
      networkWritesAttempted: false,
      writesEnabled: false,
    });
  }

  return Promise.resolve({
    status: "warning" as const,
    checks: [
      {
        checkId: "CHK-01",
        label: "Target Mode",
        status: "passed" as const,
        message: "Target mode is allowed for sandbox restore.",
      },
      {
        checkId: "CHK-10",
        label: "Live Metadata Check",
        status: "skipped" as const,
        message: "Live Airtable metadata check is not performed in this environment.",
        remediation: "Full verification requires a live Airtable connection.",
      },
    ],
    safetySummary: {
      writesEnabled: false,
      networkWritesAttempted: false,
      noChangesMade: true,
      writeGateStatus: "disabled",
      liveMetadataCheckPerformed: false,
    },
    message:
      "Sandbox verification passed with warnings. Live metadata check was skipped. writesEnabled is always false.",
    noChangesMade: true,
    networkWritesAttempted: false,
    writesEnabled: false,
  });
}

function validateRestoreConfirmationGateImpl(
  request: RestoreConfirmationRequest,
): Promise<RestoreConfirmationResult> {
  // Build required text using the same logic as the Rust implementation.
  const target = request.targetLabel?.trim().toUpperCase();
  const pkg = request.sourcePackageLabel?.trim().toUpperCase();
  const requiredText = target ? `RESTORE TO ${target}` : pkg ? `RESTORE ${pkg}` : "RESTORE BACKUP";

  const sandboxStatus = request.sandboxVerificationStatus ?? "unknown";
  const sandboxOk = sandboxStatus === "verified" || sandboxStatus === "warning";
  const sandboxBlocked = sandboxStatus === "blocked";
  const entered = request.enteredText.trim();
  const textMatches = entered === requiredText;
  const hasTokenPattern =
    entered.startsWith("pat") && entered.length > 20 && /^[a-zA-Z0-9]+$/.test(entered.slice(3));

  const checks: RestoreConfirmationResult["checks"] = [
    {
      checkId: "CHK-C01",
      label: "Write gate disabled",
      status: "passed",
      message: "Write gate is disabled — no writes will occur.",
    },
    {
      checkId: "CHK-C02",
      label: "Sandbox verification not blocked",
      status: sandboxOk ? "passed" : sandboxStatus === "unknown" ? "skipped" : "failed",
      message: sandboxOk
        ? `Sandbox verification status is '${sandboxStatus}'.`
        : sandboxStatus === "unknown"
          ? "Sandbox verification has not been run."
          : "Sandbox verification is blocked. Resolve Gate 1 before confirming.",
    },
    {
      checkId: "CHK-C03",
      label: "Confirmation text exact match",
      status: textMatches ? "passed" : "failed",
      message: textMatches
        ? "Confirmation text matches exactly."
        : entered === ""
          ? "No confirmation text entered."
          : "Confirmation text does not match.",
    },
    {
      checkId: "CHK-C04",
      label: "No token in confirmation text",
      status: hasTokenPattern ? "failed" : "passed",
      message: hasTokenPattern
        ? "Confirmation text resembles an API token."
        : "Confirmation text does not resemble an API token.",
    },
    {
      checkId: "CHK-C05",
      label: "Restore writes remain disabled",
      status: "passed",
      message:
        "Restore write execution is not enabled. Confirmation is recorded but triggers no writes.",
    },
  ];

  const anyHardFail = hasTokenPattern;
  const status: RestoreConfirmationResult["status"] =
    anyHardFail || sandboxBlocked ? "blocked" : textMatches && sandboxOk ? "confirmed" : "rejected";

  const message =
    status === "confirmed"
      ? "Confirmation accepted. Restore writes remain disabled — no Airtable changes will be made."
      : status === "blocked"
        ? sandboxBlocked
          ? "Sandbox verification is blocked. Resolve Gate 1 first."
          : "Confirmation text must not be an API token."
        : entered === ""
          ? "No confirmation text entered. Type the exact required text."
          : sandboxStatus === "unknown"
            ? "Run sandbox verification (Gate 1) before confirming."
            : "Confirmation text does not match. Type the exact required text (case-sensitive).";

  return Promise.resolve({
    status,
    checks,
    requirements: [
      {
        requirementId: "REQ-C01",
        label: "Write gate disabled",
        satisfied: true,
        note: "evaluate_write_gate() returns Disabled.",
      },
      {
        requirementId: "REQ-C02",
        label: "Sandbox verification not blocked",
        satisfied: sandboxOk,
        note: "Gate 1 must not be blocked.",
      },
      {
        requirementId: "REQ-C03",
        label: "Confirmation text exact match",
        satisfied: textMatches,
        note: `Required: "${requiredText}"`,
      },
    ],
    requiredText,
    message,
    noChangesMade: true,
    networkWritesAttempted: false,
    writesEnabled: false,
  });
}

function verifyRestoreTargetEmptyImpl(
  request: TargetEmptyVerificationRequest,
): Promise<TargetEmptyVerificationResult> {
  const mode = request.targetMode;
  const modeSafe = mode === "newBase" || mode === "emptyExistingBase";

  const checks: TargetEmptyVerificationResult["checks"] = [];

  // TEV-01: write gate
  checks.push({
    checkId: "TEV-01",
    label: "write-gate",
    status: "passed",
    message: "Write gate is disabled — no writes can be executed.",
  });

  // TEV-02: target mode
  if (modeSafe) {
    checks.push({
      checkId: "TEV-02",
      label: "target-mode",
      status: "passed",
      message: `Target mode '${mode}' is supported.`,
    });
  } else {
    checks.push({
      checkId: "TEV-02",
      label: "target-mode",
      status: "failed",
      message: `Target mode '${mode}' is not supported. Only 'newBase' and 'emptyExistingBase' are allowed.`,
      remediation: "Set targetMode to 'newBase' or 'emptyExistingBase'.",
    });
  }

  // TEV-03: table count
  let tableCountOk = false;
  if (mode === "newBase" && request.targetTableCount === undefined) {
    checks.push({
      checkId: "TEV-03",
      label: "table-count",
      status: "passed",
      message: "New base target — no existing tables expected.",
    });
    tableCountOk = true;
  } else if (request.targetTableCount === undefined) {
    checks.push({
      checkId: "TEV-03",
      label: "table-count",
      status: "warning",
      message: "Table count is not known. Live metadata check was not performed.",
      remediation:
        "Perform a live metadata check to verify the target base is empty before enabling writes.",
    });
  } else if (request.targetTableCount === 0) {
    checks.push({
      checkId: "TEV-03",
      label: "table-count",
      status: "passed",
      message: "Target base has zero tables — safe to restore.",
    });
    tableCountOk = true;
  } else {
    checks.push({
      checkId: "TEV-03",
      label: "table-count",
      status: "failed",
      message: `Target base has ${request.targetTableCount} table(s). Restoring into a non-empty base is not safe.`,
      remediation: "Choose an empty base or create a new base as the restore target.",
    });
  }

  // TEV-04: record count
  let recordCountOk = false;
  if (mode === "newBase" && request.targetRecordCount === undefined) {
    checks.push({
      checkId: "TEV-04",
      label: "record-count",
      status: "passed",
      message: "New base target — no existing records expected.",
    });
    recordCountOk = true;
  } else if (request.targetRecordCount === undefined) {
    checks.push({
      checkId: "TEV-04",
      label: "record-count",
      status: "warning",
      message: "Record count is not known. Live metadata check was not performed.",
      remediation:
        "Perform a live metadata check to verify the target base is empty before enabling writes.",
    });
  } else if (request.targetRecordCount === 0) {
    checks.push({
      checkId: "TEV-04",
      label: "record-count",
      status: "passed",
      message: "Target base has zero records — safe to restore.",
    });
    recordCountOk = true;
  } else {
    checks.push({
      checkId: "TEV-04",
      label: "record-count",
      status: "failed",
      message: `Target base has ${request.targetRecordCount} record(s). Restoring into a non-empty base is not safe.`,
      remediation: "Choose an empty base or delete all records before restoring.",
    });
  }

  // TEV-05: no writes enabled
  checks.push({
    checkId: "TEV-05",
    label: "no-writes-enabled",
    status: "passed",
    message: "Restore writes are not enabled. This check always passes in this version.",
  });

  const anyHardFail = checks.some((c) => c.status === "failed");
  const anyWarningOnly = !anyHardFail && (!tableCountOk || !recordCountOk);

  const status: TargetEmptyVerificationStatus =
    !modeSafe || anyHardFail ? "blocked" : anyWarningOnly ? "warning" : "verified";

  const targetName = request.targetDisplayName ?? "the target base";

  const message =
    status === "verified"
      ? mode === "newBase"
        ? "New base target — no existing data to conflict with. Restore is safe to proceed when writes are enabled."
        : `${targetName} is confirmed empty (0 tables, 0 records). Restore is safe to proceed when writes are enabled.`
      : status === "warning"
        ? `Target base emptiness could not be confirmed for ${targetName}. Live metadata check was not performed. Resolve this before enabling live writes.`
        : !modeSafe
          ? `Target mode '${mode}' is not supported. Only 'newBase' and 'emptyExistingBase' are allowed.`
          : `${targetName} is not empty. Restoring into a non-empty base is blocked to prevent data loss.`;

  return Promise.resolve({
    status,
    checks,
    message,
    noChangesMade: true,
    networkWritesAttempted: false,
    writesEnabled: false,
  });
}

function verifyDestructiveOperationPolicyImpl(
  request: DestructiveOperationPolicyRequest,
): Promise<DestructiveOperationPolicyResult> {
  const checks: DestructiveOperationPolicyResult["checks"] = [];

  const BLOCKED_KINDS = new Set([
    "deleteBase",
    "deleteTable",
    "deleteField",
    "deleteRecord",
    "updateExistingRecord",
    "overwriteField",
    "overwriteTable",
    "attachmentUpload",
  ]);

  const CREATE_ONLY_KINDS = new Set([
    "createBase",
    "createTable",
    "createField",
    "createRecord",
    "updateLinkedRecordReference",
    "preserveAttachmentMetadata",
    "checkpoint",
    "skipField",
    "manualAction",
    "deferLinkedField",
  ]);

  // DOP-01: write gate always disabled
  checks.push({
    checkId: "DOP-01",
    label: "write-gate-disabled",
    status: "passed",
    message:
      "Write gate is disabled — no writes can be executed. Restore write execution is not enabled in this version.",
  });

  const deleteOps = request.declaredOperations.filter((op) =>
    ["deleteBase", "deleteTable", "deleteField", "deleteRecord"].includes(op.kind),
  );
  const updateOverwriteOps = request.declaredOperations.filter((op) =>
    ["updateExistingRecord", "overwriteField", "overwriteTable"].includes(op.kind),
  );
  const attachmentUploadOps = request.declaredOperations.filter(
    (op) => op.kind === "attachmentUpload",
  );

  const blockedOperations = [...deleteOps, ...updateOverwriteOps, ...attachmentUploadOps].map(
    (op) => op.label,
  );

  // DOP-02: no delete operations
  if (deleteOps.length > 0) {
    checks.push({
      checkId: "DOP-02",
      label: "no-delete-operations",
      status: "failed",
      message: `Delete operations are not permitted during restore: ${deleteOps.map((op) => op.label).join(", ")}.`,
      remediation: "Remove all delete operations from the restore plan.",
    });
  } else {
    checks.push({
      checkId: "DOP-02",
      label: "no-delete-operations",
      status: "passed",
      message: "No delete operations declared.",
    });
  }

  // DOP-03: no update/overwrite operations
  if (updateOverwriteOps.length > 0) {
    checks.push({
      checkId: "DOP-03",
      label: "no-update-overwrite-operations",
      status: "failed",
      message: `Update and overwrite operations are not permitted during restore: ${updateOverwriteOps.map((op) => op.label).join(", ")}.`,
      remediation: "Remove all update and overwrite operations from the restore plan.",
    });
  } else {
    checks.push({
      checkId: "DOP-03",
      label: "no-update-overwrite-operations",
      status: "passed",
      message: "No update or overwrite operations declared.",
    });
  }

  // DOP-04: no attachment upload operations
  if (attachmentUploadOps.length > 0) {
    checks.push({
      checkId: "DOP-04",
      label: "no-attachment-upload",
      status: "failed",
      message: `Attachment upload operations are not permitted in this phase: ${attachmentUploadOps.map((op) => op.label).join(", ")}.`,
      remediation:
        "Attachment bytes cannot be uploaded during restore in this version. Only attachment metadata is preserved.",
    });
  } else {
    checks.push({
      checkId: "DOP-04",
      label: "no-attachment-upload",
      status: "passed",
      message: "No attachment upload operations declared.",
    });
  }

  // DOP-05: all remaining ops are create-only or safe
  const unknownOps = request.declaredOperations.filter(
    (op) => !BLOCKED_KINDS.has(op.kind) && !CREATE_ONLY_KINDS.has(op.kind),
  );
  if (unknownOps.length > 0) {
    checks.push({
      checkId: "DOP-05",
      label: "create-only-operations",
      status: "warning",
      message: `Some operations could not be classified as create-only: ${unknownOps.map((op) => op.label).join(", ")}.`,
      remediation: "Review unclassified operations before enabling live writes.",
    });
  } else {
    checks.push({
      checkId: "DOP-05",
      label: "create-only-operations",
      status: "passed",
      message: "All declared operations are create-only or safe.",
    });
  }

  const anyHardFail =
    deleteOps.length > 0 || updateOverwriteOps.length > 0 || attachmentUploadOps.length > 0;
  const anyUnknown = unknownOps.length > 0;

  const status: DestructiveOperationPolicyStatus = anyHardFail
    ? "blocked"
    : anyUnknown
      ? "warning"
      : "compliant";

  const targetName = request.targetDisplayName ?? "the target base";

  const message =
    status === "compliant"
      ? `All declared operations for ${targetName} are create-only. No destructive operations detected. Restore writes remain disabled.`
      : status === "warning"
        ? `Some operations for ${targetName} could not be classified. Manual review is required before enabling live writes.`
        : `Blocked operations detected for ${targetName}: ${blockedOperations.join(", ")}. Remove all destructive operations before proceeding.`;

  return Promise.resolve({
    status,
    checks,
    message,
    blockedOperations,
    noChangesMade: true,
    networkWritesAttempted: false,
    writesEnabled: false,
  });
}

function verifyAttachmentUploadPolicyImpl(
  request: AttachmentUploadPolicyRequest,
): Promise<AttachmentUploadPolicyResult> {
  const checks: AttachmentUploadPolicyResult["checks"] = [];

  const uploadFields = request.declaredAttachmentFields.filter(
    (f) => f.intent === "uploadRequested",
  );
  const downloadFields = request.declaredAttachmentFields.filter(
    (f) => f.intent === "downloadRequested",
  );
  const unknownFields = request.declaredAttachmentFields.filter((f) => f.intent === "unknown");
  const metadataOnlyCount = request.declaredAttachmentFields.filter(
    (f) => f.intent === "metadataOnly",
  ).length;

  const blockedFieldNames = uploadFields.map((f) => `${f.tableName}.${f.fieldName}`);

  // AUP-01: write gate always disabled
  checks.push({
    checkId: "AUP-01",
    label: "write-gate-disabled",
    status: "passed",
    message:
      "Write gate is disabled — no writes can be executed. Restore write execution is not enabled in this version.",
  });

  // AUP-02: no upload-requested intents
  if (uploadFields.length > 0) {
    checks.push({
      checkId: "AUP-02",
      label: "no-upload-requested",
      status: "failed",
      message: `Attachment upload is not permitted in this version: ${uploadFields.map((f) => `${f.tableName}.${f.fieldName}`).join(", ")}.`,
      remediation:
        "Change all attachment fields to MetadataOnly intent. File bytes cannot be uploaded during restore.",
    });
  } else {
    checks.push({
      checkId: "AUP-02",
      label: "no-upload-requested",
      status: "passed",
      message: "No attachment upload operations requested.",
    });
  }

  // AUP-03: no download-requested intents (warning)
  if (downloadFields.length > 0) {
    checks.push({
      checkId: "AUP-03",
      label: "no-download-requested",
      status: "warning",
      message: `Attachment download is deferred — file bytes will not be fetched: ${downloadFields.map((f) => `${f.tableName}.${f.fieldName}`).join(", ")}.`,
      remediation:
        "Attachment download is not implemented in this version. Only metadata is preserved.",
    });
  } else {
    checks.push({
      checkId: "AUP-03",
      label: "no-download-requested",
      status: "passed",
      message: "No attachment download operations requested.",
    });
  }

  // AUP-04: no unknown intents (warning)
  if (unknownFields.length > 0) {
    checks.push({
      checkId: "AUP-04",
      label: "no-unknown-intents",
      status: "warning",
      message: `Some attachment fields have unknown intent: ${unknownFields.map((f) => `${f.tableName}.${f.fieldName}`).join(", ")}.`,
      remediation: "Classify all attachment fields before enabling live writes.",
    });
  } else {
    checks.push({
      checkId: "AUP-04",
      label: "no-unknown-intents",
      status: "passed",
      message: "All declared attachment fields have a known intent.",
    });
  }

  // AUP-05: metadata-only confirmation
  const allMetadataOnly =
    uploadFields.length === 0 && downloadFields.length === 0 && unknownFields.length === 0;
  if (allMetadataOnly) {
    checks.push({
      checkId: "AUP-05",
      label: "metadata-only-confirmed",
      status: "passed",
      message: `All ${metadataOnlyCount} declared attachment field(s) use metadata-only handling. File bytes are not uploaded or downloaded.`,
    });
  } else {
    const totalNonMetadata = uploadFields.length + downloadFields.length + unknownFields.length;
    checks.push({
      checkId: "AUP-05",
      label: "metadata-only-confirmed",
      status: uploadFields.length > 0 ? "failed" : "warning",
      message: `${totalNonMetadata} attachment field(s) are not confirmed as metadata-only.`,
      remediation: "Set all attachment fields to MetadataOnly intent before enabling live writes.",
    });
  }

  const hasBlocked = uploadFields.length > 0;
  const hasWarning = downloadFields.length > 0 || unknownFields.length > 0;

  const status: AttachmentUploadPolicyStatus = hasBlocked
    ? "blocked"
    : hasWarning
      ? "warning"
      : "compliant";

  const targetName = request.targetDisplayName ?? "the target base";

  const message =
    status === "compliant"
      ? `All ${metadataOnlyCount} declared attachment field(s) for ${targetName} use metadata-only handling. Attachment file bytes are not uploaded or downloaded. Restore writes remain disabled.`
      : status === "warning"
        ? `Some attachment fields for ${targetName} have deferred or unknown intent. Attachment file bytes will not be uploaded or downloaded in this version.`
        : `Attachment upload is not permitted for ${targetName}: ${blockedFieldNames.join(", ")}. Change all attachment fields to MetadataOnly intent.`;

  return Promise.resolve({
    status,
    checks,
    message,
    blockedFieldNames,
    metadataOnlyFieldCount: metadataOnlyCount,
    noChangesMade: true,
    networkWritesAttempted: false,
    writesEnabled: false,
  });
}

function verifySchemaRecordOrderPolicyImpl(
  request: SchemaRecordOrderPolicyRequest,
): Promise<SchemaRecordOrderPolicyResult> {
  const checks: SchemaRecordOrderPolicyResult["checks"] = [];
  const violations: string[] = [];

  const phases = request.declaredPhases;

  const schemaIdx = phases.findIndex((p) => p.phase === "schema");
  const recordsIdx = phases.findIndex((p) => p.phase === "records");
  const linkedIdx = phases.findIndex((p) => p.phase === "linkedRecords");
  const attachIdx = phases.findIndex((p) => p.phase === "attachments");

  const schemaPhase = schemaIdx >= 0 ? phases[schemaIdx] : null;
  const recordsPhase = recordsIdx >= 0 ? phases[recordsIdx] : null;

  // SRO-01: write gate always disabled
  checks.push({
    checkId: "SRO-01",
    label: "write-gate-disabled",
    status: "passed",
    message:
      "Write gate is disabled — no writes can be executed. Restore write execution is not enabled in this version.",
  });

  // SRO-02: schema phase present and not blocked
  if (schemaPhase === null) {
    if (recordsPhase !== null) {
      checks.push({
        checkId: "SRO-02",
        label: "schema-phase-present",
        status: "failed",
        message:
          "Schema phase is missing but a record phase is declared. Records cannot be created without a schema.",
        remediation: "Add a schema creation phase before the record creation phase.",
      });
      violations.push("missing-schema-with-records");
    } else {
      checks.push({
        checkId: "SRO-02",
        label: "schema-phase-present",
        status: "warning",
        message: "No phases declared. Phase ordering cannot be verified.",
      });
    }
  } else if (schemaPhase.isBlocked) {
    checks.push({
      checkId: "SRO-02",
      label: "schema-phase-present",
      status: "failed",
      message: "Schema phase is blocked. Record phases must not proceed when schema is blocked.",
      remediation: "Resolve schema plan issues before attempting record creation.",
    });
    violations.push("schema-phase-blocked");
  } else if (!schemaPhase.isPlanned) {
    checks.push({
      checkId: "SRO-02",
      label: "schema-phase-present",
      status: "warning",
      message:
        "Schema phase is declared but not yet planned. Phase ordering cannot be fully verified.",
    });
  } else {
    checks.push({
      checkId: "SRO-02",
      label: "schema-phase-present",
      status: "passed",
      message: "Schema phase is present and planned.",
    });
  }

  // SRO-03: schema before records
  if (schemaIdx < 0 && recordsIdx >= 0) {
    checks.push({
      checkId: "SRO-03",
      label: "schema-before-records",
      status: "failed",
      message: "Record phase declared but no schema phase present.",
      remediation: "Add a schema phase before record creation.",
    });
    if (!violations.includes("missing-schema-with-records")) {
      violations.push("missing-schema-with-records");
    }
  } else if (schemaIdx >= 0 && recordsIdx >= 0 && recordsIdx <= schemaIdx) {
    checks.push({
      checkId: "SRO-03",
      label: "schema-before-records",
      status: "failed",
      message:
        "Record-create phase appears at or before schema phase. Schema must precede all record operations.",
      remediation: "Move schema phase before the record-create phase.",
    });
    violations.push("records-before-schema");
  } else {
    checks.push({
      checkId: "SRO-03",
      label: "schema-before-records",
      status: "passed",
      message:
        schemaIdx >= 0 && recordsIdx >= 0
          ? "Schema phase precedes record-create phase."
          : "No record phase declared; schema ordering constraint is satisfied.",
    });
  }

  // SRO-04: records before linked updates
  if (linkedIdx >= 0 && recordsIdx < 0) {
    checks.push({
      checkId: "SRO-04",
      label: "records-before-linked-updates",
      status: "warning",
      message:
        "Linked-record update phase declared without a record-create phase. Phase data may be incomplete.",
      remediation: "Ensure record-create phase is declared before linked-record updates.",
    });
  } else if (linkedIdx >= 0 && recordsIdx >= 0 && linkedIdx <= recordsIdx) {
    checks.push({
      checkId: "SRO-04",
      label: "records-before-linked-updates",
      status: "failed",
      message:
        "Linked-record update phase appears at or before record-create phase. Linked updates require first-pass records to exist.",
      remediation: "Move linked-record update phase after record-create phase.",
    });
    violations.push("linked-before-record-create");
  } else {
    checks.push({
      checkId: "SRO-04",
      label: "records-before-linked-updates",
      status: "passed",
      message:
        "Record-create phase precedes linked-record update phase, or no linked-record phase declared.",
    });
  }

  // SRO-05: records before attachments
  if (attachIdx >= 0 && recordsIdx < 0) {
    checks.push({
      checkId: "SRO-05",
      label: "records-before-attachments",
      status: "warning",
      message:
        "Attachment phase declared without a record-create phase. Phase data may be incomplete.",
      remediation: "Ensure record-create phase is declared before attachment handling.",
    });
  } else if (attachIdx >= 0 && recordsIdx >= 0 && attachIdx <= recordsIdx) {
    checks.push({
      checkId: "SRO-05",
      label: "records-before-attachments",
      status: "failed",
      message:
        "Attachment-handling phase appears at or before record-create phase. Attachment metadata is associated with records and requires record IDs.",
      remediation: "Move attachment-handling phase after record-create phase.",
    });
    violations.push("attachment-before-record-create");
  } else {
    checks.push({
      checkId: "SRO-05",
      label: "records-before-attachments",
      status: "passed",
      message:
        "Record-create phase precedes attachment-handling phase, or no attachment phase declared.",
    });
  }

  const hasBlocked = checks.some((c) => c.status === "failed");
  const hasWarning = checks.some((c) => c.status === "warning");

  const status: SchemaRecordOrderPolicyStatus = hasBlocked
    ? "blocked"
    : hasWarning
      ? "warning"
      : "compliant";

  const targetName = request.targetDisplayName ?? "the restore target";

  const message =
    status === "compliant"
      ? `Phase ordering for ${targetName} is valid. Schema precedes all record phases. Restore writes remain disabled.`
      : status === "warning"
        ? `Phase ordering for ${targetName} could not be fully verified. Some phase data is incomplete.`
        : `Phase ordering violation detected for ${targetName}: ${violations.join(", ")}. Resolve ordering issues before enabling live writes.`;

  return Promise.resolve({
    status,
    checks,
    message,
    orderingViolations: violations,
    noChangesMade: true,
    networkWritesAttempted: false,
    writesEnabled: false,
  });
}

function verifySandboxWriteTestingPolicyImpl(
  request: SandboxWriteTestingPolicyRequest,
): Promise<SandboxWriteTestingPolicyResult> {
  const checks = [];

  // SWT-01: write gate always disabled
  checks.push({
    checkId: "SWT-01",
    label: "write-gate-disabled",
    status: "passed" as const,
    message: "Restore write gate is disabled. No live writes are possible.",
  });

  // SWT-02: target classification
  if (request.targetClassification === "sandbox") {
    checks.push({
      checkId: "SWT-02",
      label: "sandbox-target-classification",
      status: "passed" as const,
      message: "Target is classified as a sandbox base.",
    });
  } else {
    checks.push({
      checkId: "SWT-02",
      label: "sandbox-target-classification",
      status: "failed" as const,
      message:
        request.targetClassification === "production"
          ? "Target is classified as a production base. Write testing must not be performed against production."
          : "Target classification is unknown. Cannot confirm this is a safe sandbox base.",
      remediation: "Use a dedicated sandbox or test base for write testing.",
    });
  }

  // SWT-03: sandbox verification passed
  if (request.sandboxVerificationPassed) {
    checks.push({
      checkId: "SWT-03",
      label: "sandbox-verification-passed",
      status: "passed" as const,
      message: "Sandbox environment verification (Gate 1) has passed for this session.",
    });
  } else {
    checks.push({
      checkId: "SWT-03",
      label: "sandbox-verification-passed",
      status: "failed" as const,
      message: "Sandbox environment verification (Gate 1) has not passed. Run Gate 1 checks first.",
      remediation: "Complete sandbox environment verification before sandbox write testing.",
    });
  }

  // SWT-04: evidence declared
  if (!request.evidence) {
    checks.push({
      checkId: "SWT-04",
      label: "sandbox-test-evidence-present",
      status: "failed" as const,
      message: "No sandbox test evidence declared. Sandbox write testing has not been recorded.",
      remediation:
        "Provide SandboxWriteTestEvidence with the results of a completed sandbox test run.",
    });
    checks.push({
      checkId: "SWT-05",
      label: "sandbox-evidence-complete",
      status: "failed" as const,
      message: "Evidence completeness check skipped — no evidence declared.",
      remediation: "Declare sandbox test evidence to enable this check.",
    });
  } else {
    checks.push({
      checkId: "SWT-04",
      label: "sandbox-test-evidence-present",
      status: "passed" as const,
      message: "Sandbox test evidence is declared.",
    });

    // SWT-05: evidence completeness
    const ev = request.evidence;
    const allRequired =
      ev.sandboxBaseVerified &&
      ev.dryRunCompleted &&
      ev.schemaPlanReviewed &&
      ev.recordPlanReviewed;
    const filenameOk =
      typeof ev.testPackageFilename === "string" &&
      ev.testPackageFilename.length > 0 &&
      !ev.testPackageFilename.includes("/") &&
      !ev.testPackageFilename.includes("\\");

    if (allRequired && filenameOk) {
      checks.push({
        checkId: "SWT-05",
        label: "sandbox-evidence-complete",
        status: "passed" as const,
        message: "All required evidence fields are present and confirmed.",
      });
    } else {
      const missing: string[] = [];
      if (!ev.sandboxBaseVerified) missing.push("sandboxBaseVerified");
      if (!ev.dryRunCompleted) missing.push("dryRunCompleted");
      if (!ev.schemaPlanReviewed) missing.push("schemaPlanReviewed");
      if (!ev.recordPlanReviewed) missing.push("recordPlanReviewed");
      if (!filenameOk) missing.push("testPackageFilename");
      checks.push({
        checkId: "SWT-05",
        label: "sandbox-evidence-complete",
        status: "warning" as const,
        message: `Evidence is incomplete. Missing or false: ${missing.join(", ")}.`,
        remediation: "Complete all required evidence fields before live write testing.",
      });
    }
  }

  const hasBlocked = checks.some((c) => c.status === "failed");
  const hasWarning = checks.some((c) => c.status === "warning");
  const status: SandboxWriteTestingPolicyStatus = hasBlocked
    ? "blocked"
    : hasWarning
      ? "warning"
      : "compliant";

  const targetName = request.targetDisplayName ?? "the restore target";
  const message =
    status === "compliant"
      ? `Sandbox write testing policy for ${targetName} is satisfied. All required evidence is present. Restore writes remain disabled.`
      : status === "warning"
        ? `Sandbox write testing policy for ${targetName} has warnings. Evidence is incomplete or partial. Restore writes remain disabled.`
        : `Sandbox write testing policy for ${targetName} is blocked. Target is not a sandbox base or required evidence is missing. Restore writes remain disabled.`;

  return Promise.resolve({
    status,
    checks,
    message,
    noChangesMade: true,
    networkWritesAttempted: false,
    writesEnabled: false,
  });
}

function buildLiveWriteConfirmationText(targetLabel: string | undefined): string {
  const safeTarget = targetLabel
    ? targetLabel
        .replace(/[^a-zA-Z0-9\-_. ]/g, "")
        .trim()
        .slice(0, 64)
        .toUpperCase() || "TARGET"
    : "TARGET";
  return `LIVE RESTORE ${safeTarget} — WRITES REMAIN DISABLED`;
}

function verifyLiveWriteConfirmationPolicyImpl(
  request: LiveWriteConfirmationPolicyRequest,
): Promise<LiveWriteConfirmationPolicyResult> {
  const requiredText = buildLiveWriteConfirmationText(request.targetLabel);
  const checks: LiveWriteConfirmationPolicyResult["checks"] = [];

  // LWC-01: Write gate disabled (always passes in mock)
  checks.push({
    checkId: "LWC-01",
    label: "write-gate-disabled",
    status: "passed",
    message: "Restore write gate is disabled. No live writes are possible.",
  });

  // LWC-02: Prior safety gates not blocked
  const gates = request.priorGateStatuses;
  const gateFlag = (s: string | undefined): { blocked: boolean; warning: boolean } => ({
    blocked: s === "blocked",
    warning: s === "warning",
  });
  const sandboxF = gateFlag(gates?.sandboxVerificationStatus);
  const dopF = gateFlag(gates?.destructiveOperationPolicyStatus);
  const aupF = gateFlag(gates?.attachmentUploadPolicyStatus);
  const sroF = gateFlag(gates?.schemaRecordOrderPolicyStatus);

  const anyPriorBlocked = sandboxF.blocked || dopF.blocked || aupF.blocked || sroF.blocked;
  const anyPriorWarn = sandboxF.warning || dopF.warning || aupF.warning || sroF.warning;

  if (anyPriorBlocked) {
    const blockedNames: string[] = [];
    if (sandboxF.blocked) blockedNames.push("sandbox-verification");
    if (dopF.blocked) blockedNames.push("destructive-operation-policy");
    if (aupF.blocked) blockedNames.push("attachment-upload-policy");
    if (sroF.blocked) blockedNames.push("schema-record-order-policy");
    checks.push({
      checkId: "LWC-02",
      label: "prior-gates-not-blocked",
      status: "failed",
      message: `Prior safety gate(s) are blocked: ${blockedNames.join(", ")}. Resolve before confirming.`,
      remediation: "Resolve all blocked prior gates before attempting live-write confirmation.",
    });
  } else if (anyPriorWarn) {
    checks.push({
      checkId: "LWC-02",
      label: "prior-gates-not-blocked",
      status: "warning",
      message: "Prior safety gates have warnings. Review before proceeding.",
      remediation: "Resolve prior-gate warnings where possible before confirming live writes.",
    });
  } else {
    checks.push({
      checkId: "LWC-02",
      label: "prior-gates-not-blocked",
      status: "passed",
      message: "Prior safety gates are not blocked.",
    });
  }

  // LWC-03: Sandbox write testing gate not blocked
  const swtF = gateFlag(gates?.sandboxWriteTestingPolicyStatus);
  if (swtF.blocked) {
    checks.push({
      checkId: "LWC-03",
      label: "sandbox-write-testing-not-blocked",
      status: "failed",
      message: "Sandbox write testing policy (Gate 7) is blocked. Resolve before confirming.",
      remediation:
        "Complete sandbox write testing with required evidence before confirming live writes.",
    });
  } else if (swtF.warning) {
    checks.push({
      checkId: "LWC-03",
      label: "sandbox-write-testing-not-blocked",
      status: "warning",
      message: "Sandbox write testing policy (Gate 7) has warnings. Review evidence completeness.",
      remediation:
        "Complete all required sandbox test evidence fields before confirming live writes.",
    });
  } else {
    checks.push({
      checkId: "LWC-03",
      label: "sandbox-write-testing-not-blocked",
      status: "passed",
      message: "Sandbox write testing policy (Gate 7) is not blocked.",
    });
  }

  // LWC-04: Confirmation text match (case-sensitive, trim only)
  const entered = request.enteredText.trim();
  const textMatches = entered === requiredText;
  if (textMatches) {
    checks.push({
      checkId: "LWC-04",
      label: "confirmation-text-match",
      status: "passed",
      message: "Confirmation text matches the required phrase.",
    });
  } else {
    checks.push({
      checkId: "LWC-04",
      label: "confirmation-text-match",
      status: "failed",
      message:
        "Confirmation text does not match the required phrase. Text must match exactly, case-sensitively.",
      remediation: `Type exactly: ${requiredText}`,
    });
  }

  // LWC-05: Writes remain disabled (always passes)
  checks.push({
    checkId: "LWC-05",
    label: "writes-remain-disabled",
    status: "passed",
    message:
      "Restore write execution is not enabled. Confirming this phrase does not start any write operation.",
  });

  const hasBlocked = checks.some((c) => c.status === "failed");
  const hasWarning = checks.some((c) => c.status === "warning");

  let status: LiveWriteConfirmationPolicyStatus;
  if (hasBlocked) {
    status = textMatches ? "blocked" : "rejected";
  } else if (!textMatches) {
    status = "rejected";
  } else if (hasWarning) {
    status = "warning";
  } else {
    status = "confirmed";
  }

  const targetName = request.targetLabel ?? "the restore target";
  const message =
    status === "confirmed"
      ? `Live-write confirmation for ${targetName} accepted. Required phrase matched. Restore writes remain disabled — confirmation does not enable live writes.`
      : status === "warning"
        ? `Live-write confirmation for ${targetName} accepted with warnings. Required phrase matched, but prior gates have unresolved warnings. Restore writes remain disabled.`
        : status === "blocked"
          ? `Live-write confirmation for ${targetName} is blocked. One or more prior safety gates are blocked. Restore writes remain disabled.`
          : `Live-write confirmation for ${targetName} rejected. Confirmation text did not match the required phrase. Restore writes remain disabled.`;

  return Promise.resolve({
    status,
    checks,
    requiredText,
    message,
    noChangesMade: true,
    networkWritesAttempted: false,
    writesEnabled: false,
  });
}

const SAFE_MAX_RPS = 5;
const SAFE_MAX_BATCH_SIZE = 10;
const DEFAULT_MAX_RETRIES = 3;

function verifyRateLimitBackoffPolicyImpl(
  request: RateLimitBackoffPolicyRequest,
): Promise<RateLimitBackoffPolicyResult> {
  const checks: RateLimitBackoffPolicyResult["checks"] = [];

  // RLB-01: Write gate always disabled
  checks.push({
    checkId: "RLB-01",
    label: "write-gate-disabled",
    status: "passed",
    message: "Restore write gate is disabled. No live writes are possible.",
  });

  const plan = request.plan;
  if (!plan) {
    checks.push({
      checkId: "RLB-02",
      label: "plan-declared",
      status: "failed",
      message:
        "No rate-limit/backoff plan declared. A plan is required before any live write path is considered.",
      remediation: "Declare a RateLimitBackoffPlan with all required fields.",
    });
    return Promise.resolve({
      status: "blocked" as RateLimitBackoffPolicyStatus,
      checks,
      message: `Rate-limit and backoff policy for ${request.targetLabel ?? "the restore target"} is blocked. One or more required fields are missing or exceed safe thresholds. Restore writes remain disabled.`,
      noChangesMade: true,
      networkWritesAttempted: false,
      writesEnabled: false,
    });
  }

  checks.push({
    checkId: "RLB-02",
    label: "plan-declared",
    status: "passed",
    message: "Rate-limit/backoff plan is declared.",
  });

  // RLB-03: RPS within threshold
  if (plan.maxRequestsPerSecond > SAFE_MAX_RPS) {
    checks.push({
      checkId: "RLB-03",
      label: "requests-per-second-safe",
      status: "failed",
      message: `Declared max requests per second (${plan.maxRequestsPerSecond}) exceeds the safe threshold (${SAFE_MAX_RPS}).`,
      remediation: `Reduce maxRequestsPerSecond to ${SAFE_MAX_RPS} or below.`,
    });
  } else {
    checks.push({
      checkId: "RLB-03",
      label: "requests-per-second-safe",
      status: "passed",
      message: `Declared max requests per second (${plan.maxRequestsPerSecond}) is within the safe threshold (${SAFE_MAX_RPS}).`,
    });
  }

  // RLB-04: Batch size within limit
  if (plan.batchSize > SAFE_MAX_BATCH_SIZE) {
    checks.push({
      checkId: "RLB-04",
      label: "batch-size-safe",
      status: "failed",
      message: `Declared batch size (${plan.batchSize}) exceeds the safe maximum (${SAFE_MAX_BATCH_SIZE}).`,
      remediation: `Reduce batchSize to ${SAFE_MAX_BATCH_SIZE} or below.`,
    });
  } else {
    checks.push({
      checkId: "RLB-04",
      label: "batch-size-safe",
      status: "passed",
      message: `Declared batch size (${plan.batchSize}) is within the safe maximum (${SAFE_MAX_BATCH_SIZE}).`,
    });
  }

  // RLB-05: 429 handling
  if (!plan.handles429) {
    checks.push({
      checkId: "RLB-05",
      label: "handles-429",
      status: "failed",
      message: "No 429 (rate-limit) response handling strategy is declared.",
      remediation: "Declare a 429 handling strategy (backoff and retry) before any live write.",
    });
  } else {
    checks.push({
      checkId: "RLB-05",
      label: "handles-429",
      status: "passed",
      message: "A 429 response handling strategy is declared.",
    });
  }

  // RLB-06: Retry limit bounded
  if (plan.maxRetries === undefined || plan.maxRetries === null) {
    checks.push({
      checkId: "RLB-06",
      label: "retry-limit-bounded",
      status: "failed",
      message: "No maximum retry limit declared. Unbounded retries are unsafe.",
      remediation: `Declare a bounded maxRetries value (recommended: ${DEFAULT_MAX_RETRIES}).`,
    });
  } else {
    checks.push({
      checkId: "RLB-06",
      label: "retry-limit-bounded",
      status: "passed",
      message: `Maximum retry limit is declared (${plan.maxRetries}).`,
    });
  }

  // RLB-07: Backoff strategy
  if (!plan.hasBackoffStrategy) {
    checks.push({
      checkId: "RLB-07",
      label: "backoff-strategy-declared",
      status: "failed",
      message: "No backoff delay strategy declared between retries.",
      remediation: "Declare an exponential or fixed backoff strategy before any live write.",
    });
  } else {
    checks.push({
      checkId: "RLB-07",
      label: "backoff-strategy-declared",
      status: "passed",
      message: "A backoff strategy between retries is declared.",
    });
  }

  // RLB-08: Stop condition
  if (!plan.hasStopCondition) {
    checks.push({
      checkId: "RLB-08",
      label: "stop-condition-declared",
      status: "failed",
      message: "No stop condition declared after repeated rate-limit failures.",
      remediation:
        "Declare a stop condition that aborts the operation after too many 429 failures.",
    });
  } else {
    checks.push({
      checkId: "RLB-08",
      label: "stop-condition-declared",
      status: "passed",
      message: "A stop condition for repeated rate-limit failures is declared.",
    });
  }

  // RLB-09: Checkpoint compatibility
  if (plan.checkpointCompatibility === "full") {
    checks.push({
      checkId: "RLB-09",
      label: "checkpoint-compatibility",
      status: "passed",
      message: "Full checkpoint/resume compatibility is declared.",
    });
  } else if (plan.checkpointCompatibility === "partial") {
    checks.push({
      checkId: "RLB-09",
      label: "checkpoint-compatibility",
      status: "warning",
      message:
        "Checkpoint/resume compatibility is partial. Full compatibility is recommended for long operations.",
      remediation: "Upgrade to full checkpoint/resume compatibility before enabling live writes.",
    });
  } else if (plan.checkpointCompatibility === "none") {
    checks.push({
      checkId: "RLB-09",
      label: "checkpoint-compatibility",
      status: "warning",
      message:
        "No checkpoint/resume compatibility declared. Long operations cannot be safely resumed after interruption.",
      remediation:
        "Implement checkpoint/resume support before enabling live writes for long operations.",
    });
  } else {
    checks.push({
      checkId: "RLB-09",
      label: "checkpoint-compatibility",
      status: "warning",
      message: "Checkpoint/resume compatibility is not declared or unknown.",
      remediation: 'Declare checkpointCompatibility as "full", "partial", or "none".',
    });
  }

  // RLB-10: Writes remain disabled (always passes)
  checks.push({
    checkId: "RLB-10",
    label: "writes-remain-disabled",
    status: "passed",
    message:
      "Restore write execution is not enabled. Verifying this policy does not start any write operation.",
  });

  const hasBlocked = checks.some((c) => c.status === "failed");
  const hasWarning = checks.some((c) => c.status === "warning");
  const status: RateLimitBackoffPolicyStatus = hasBlocked
    ? "blocked"
    : hasWarning
      ? "warning"
      : "compliant";

  const targetName = request.targetLabel ?? "the restore target";
  const message =
    status === "compliant"
      ? `Rate-limit and backoff policy for ${targetName} is compliant. All required fields are within safe bounds. Restore writes remain disabled.`
      : status === "warning"
        ? `Rate-limit and backoff policy for ${targetName} has warnings. No unsafe threshold is exceeded, but some fields are incomplete or partial. Restore writes remain disabled.`
        : `Rate-limit and backoff policy for ${targetName} is blocked. One or more required fields are missing or exceed safe thresholds. Restore writes remain disabled.`;

  return Promise.resolve({
    status,
    checks,
    message,
    planSummary: {
      maxRequestsPerSecond: plan.maxRequestsPerSecond,
      batchSize: plan.batchSize,
      handles429: plan.handles429,
      maxRetries: plan.maxRetries,
      hasBackoffStrategy: plan.hasBackoffStrategy,
      hasStopCondition: plan.hasStopCondition,
      checkpointCompatibility: plan.checkpointCompatibility,
    },
    noChangesMade: true,
    networkWritesAttempted: false,
    writesEnabled: false,
  });
}

function verifyCheckpointDurabilityPolicyImpl(
  request: CheckpointDurabilityPolicyRequest,
): Promise<CheckpointDurabilityPolicyResult> {
  const checks: CheckpointDurabilityPolicyResult["checks"] = [];

  // CDP-01: Write gate always disabled
  checks.push({
    checkId: "CDP-01",
    label: "write-gate-disabled",
    status: "passed",
    message: "Restore write gate is disabled. No live writes are possible.",
  });

  const plan = request.plan;
  if (!plan) {
    checks.push({
      checkId: "CDP-02",
      label: "plan-declared",
      status: "failed",
      message:
        "No checkpoint durability plan declared. A plan is required before any live write path is considered.",
      remediation: "Declare a CheckpointDurabilityPlan with all required fields.",
    });
    return Promise.resolve({
      status: "blocked" as CheckpointDurabilityPolicyStatus,
      checks,
      message: `Checkpoint durability policy for ${request.targetLabel ?? "the restore target"} is blocked. One or more required checkpoint fields are missing. Restore writes remain disabled.`,
      noChangesMade: true,
      networkWritesAttempted: false,
      writesEnabled: false,
    });
  }

  // CDP-02: Plan declared
  checks.push({
    checkId: "CDP-02",
    label: "plan-declared",
    status: "passed",
    message: "Checkpoint durability plan is declared.",
  });

  // CDP-03: Checkpoint after each table
  if (!plan.checkpointAfterEachTable) {
    checks.push({
      checkId: "CDP-03",
      label: "checkpoint-after-each-table",
      status: "failed",
      message:
        "No checkpoint after each table is declared. A durable checkpoint must be written after each table completes to allow safe resumption.",
      remediation: "Declare checkpointAfterEachTable: true in the durability plan.",
    });
  } else {
    checks.push({
      checkId: "CDP-03",
      label: "checkpoint-after-each-table",
      status: "passed",
      message: "Checkpoint after each table is declared.",
    });
  }

  // CDP-04: Checkpoint after each batch
  if (!plan.checkpointAfterEachBatch) {
    checks.push({
      checkId: "CDP-04",
      label: "checkpoint-after-each-batch",
      status: "failed",
      message:
        "No checkpoint after each record batch is declared. A durable checkpoint must be written after each batch to prevent duplicate writes on resume.",
      remediation: "Declare checkpointAfterEachBatch: true in the durability plan.",
    });
  } else {
    checks.push({
      checkId: "CDP-04",
      label: "checkpoint-after-each-batch",
      status: "passed",
      message: "Checkpoint after each record batch is declared.",
    });
  }

  // CDP-05: Phase markers
  if (!plan.hasPhaseMarkers) {
    checks.push({
      checkId: "CDP-05",
      label: "phase-markers-declared",
      status: "failed",
      message:
        "Phase markers are not declared. Phase markers for schema, record_create, linked_update, and final_validation are required.",
      remediation:
        "Declare phase markers for all four required phases: schema, record_create, linked_update, final_validation.",
    });
  } else {
    checks.push({
      checkId: "CDP-05",
      label: "phase-markers-declared",
      status: "passed",
      message: "Phase markers for all required phases are declared.",
    });
  }

  // CDP-06: ID mapping checkpoint
  if (plan.hasLinkedUpdates && !plan.hasIdMappingCheckpoint) {
    checks.push({
      checkId: "CDP-06",
      label: "id-mapping-checkpoint",
      status: "failed",
      message:
        "Linked record updates are declared but no old-to-new ID mapping checkpoint is planned. The mapping must be persisted before any linked update begins.",
      remediation:
        "Declare hasIdMappingCheckpoint: true to persist the record ID mapping before linked updates begin.",
    });
  } else if (!plan.hasLinkedUpdates && !plan.hasIdMappingCheckpoint) {
    checks.push({
      checkId: "CDP-06",
      label: "id-mapping-checkpoint",
      status: "passed",
      message: "No linked record updates declared — ID mapping checkpoint not required.",
    });
  } else {
    checks.push({
      checkId: "CDP-06",
      label: "id-mapping-checkpoint",
      status: "passed",
      message: "Old-to-new record ID mapping checkpoint is declared before linked updates.",
    });
  }

  // CDP-07: Resume-safe stop condition
  if (!plan.hasResumeSafeStopCondition) {
    checks.push({
      checkId: "CDP-07",
      label: "resume-safe-stop-condition",
      status: "failed",
      message:
        "No resume-safe stop condition is declared. The restore operation must be able to restart from the last checkpoint without duplicating writes.",
      remediation:
        "Declare a resume-safe stop condition that allows resumption from any checkpoint.",
    });
  } else {
    checks.push({
      checkId: "CDP-07",
      label: "resume-safe-stop-condition",
      status: "passed",
      message: "A resume-safe stop condition is declared.",
    });
  }

  // CDP-08: Durability backend
  if (plan.durabilityBackend === "memory") {
    checks.push({
      checkId: "CDP-08",
      label: "durability-backend",
      status: "warning",
      message:
        "Durability backend is memory-only. Checkpoints will be lost if the process exits — long operations cannot be safely resumed after a crash.",
      remediation: "Use a disk or remote durability backend for production restore operations.",
    });
  } else if (plan.durabilityBackend === "disk" || plan.durabilityBackend === "remote") {
    checks.push({
      checkId: "CDP-08",
      label: "durability-backend",
      status: "passed",
      message: `Durability backend is '${plan.durabilityBackend}' — checkpoints survive process restart.`,
    });
  } else {
    checks.push({
      checkId: "CDP-08",
      label: "durability-backend",
      status: "warning",
      message:
        "Durability backend is not declared or unknown. Checkpoint durability cannot be confirmed.",
      remediation: 'Declare durabilityBackend as "disk", "remote", or "memory".',
    });
  }

  // CDP-09: Writes remain disabled (always passes)
  checks.push({
    checkId: "CDP-09",
    label: "writes-remain-disabled",
    status: "passed",
    message:
      "Restore write execution is not enabled. Verifying this policy does not start any write operation.",
  });

  const hasBlocked = checks.some((c) => c.status === "failed");
  const hasWarning = checks.some((c) => c.status === "warning");
  const status: CheckpointDurabilityPolicyStatus = hasBlocked
    ? "blocked"
    : hasWarning
      ? "warning"
      : "compliant";

  const targetName = request.targetLabel ?? "the restore target";
  const message =
    status === "compliant"
      ? `Checkpoint durability policy for ${targetName} is compliant. All required checkpoint fields are declared. Restore writes remain disabled.`
      : status === "warning"
        ? `Checkpoint durability policy for ${targetName} has warnings. No required field is missing, but some durability conditions are incomplete. Restore writes remain disabled.`
        : `Checkpoint durability policy for ${targetName} is blocked. One or more required checkpoint fields are missing. Restore writes remain disabled.`;

  return Promise.resolve({
    status,
    checks,
    message,
    planSummary: {
      checkpointAfterEachTable: plan.checkpointAfterEachTable,
      checkpointAfterEachBatch: plan.checkpointAfterEachBatch,
      hasPhaseMarkers: plan.hasPhaseMarkers,
      hasIdMappingCheckpoint: plan.hasIdMappingCheckpoint,
      hasResumeSafeStopCondition: plan.hasResumeSafeStopCondition,
      hasLinkedUpdates: plan.hasLinkedUpdates,
      durabilityBackend: plan.durabilityBackend,
    },
    noChangesMade: true,
    networkWritesAttempted: false,
    writesEnabled: false,
  });
}

function verifyFinalValidationPolicyImpl(
  request: FinalValidationPolicyRequest,
): Promise<FinalValidationPolicyResult> {
  const checks: FinalValidationPolicyResult["checks"] = [];

  // FVP-01: Write gate always disabled
  checks.push({
    checkId: "FVP-01",
    label: "write-gate-disabled",
    status: "passed",
    message: "Restore write gate is disabled. No live writes are possible.",
  });

  const plan = request.plan;
  if (!plan) {
    checks.push({
      checkId: "FVP-02",
      label: "plan-declared",
      status: "failed",
      message:
        "No final validation plan declared. A plan is required before any live write path is considered.",
      remediation: "Declare a FinalValidationPlan with all required validation steps.",
    });
    return Promise.resolve({
      status: "blocked" as FinalValidationPolicyStatus,
      checks,
      message: `Final validation plan for ${request.targetLabel ?? "the restore target"} is blocked. A validation plan must be declared. Restore writes remain disabled.`,
      noChangesMade: true,
      networkWritesAttempted: false,
      writesEnabled: false,
    });
  }

  // FVP-02: Plan declared
  checks.push({
    checkId: "FVP-02",
    label: "plan-declared",
    status: "passed",
    message: "Final validation plan is declared.",
  });

  // FVP-03: Schema count validation
  if (!plan.hasSchemaCountValidation) {
    checks.push({
      checkId: "FVP-03",
      label: "schema-count-validation",
      status: "failed",
      message:
        "Schema count validation is not declared. Table and field counts must be verified against the backup manifest after restore.",
      remediation: "Declare hasSchemaCountValidation: true in the final validation plan.",
    });
  } else {
    checks.push({
      checkId: "FVP-03",
      label: "schema-count-validation",
      status: "passed",
      message: "Schema count validation is declared.",
    });
  }

  // FVP-04: Table/field validation
  if (!plan.hasTableFieldValidation) {
    checks.push({
      checkId: "FVP-04",
      label: "table-field-validation",
      status: "failed",
      message:
        "Table and field presence validation is not declared. Each table and field from the manifest must be confirmed present in the restored base.",
      remediation: "Declare hasTableFieldValidation: true in the final validation plan.",
    });
  } else {
    checks.push({
      checkId: "FVP-04",
      label: "table-field-validation",
      status: "passed",
      message: "Table and field presence validation is declared.",
    });
  }

  // FVP-05: Record count validation
  if (!plan.hasRecordCountValidation) {
    checks.push({
      checkId: "FVP-05",
      label: "record-count-validation",
      status: "failed",
      message:
        "Record count validation is not declared. Restored record counts must be verified against the backup manifest before restore is considered complete.",
      remediation: "Declare hasRecordCountValidation: true in the final validation plan.",
    });
  } else {
    checks.push({
      checkId: "FVP-05",
      label: "record-count-validation",
      status: "passed",
      message: "Record count validation is declared.",
    });
  }

  // FVP-06: ID mapping validation
  if (!plan.hasIdMappingValidation) {
    checks.push({
      checkId: "FVP-06",
      label: "id-mapping-validation",
      status: "failed",
      message:
        "Old-to-new record ID mapping validation is not declared. ID mapping completeness must be verified before linked records can be considered valid.",
      remediation: "Declare hasIdMappingValidation: true in the final validation plan.",
    });
  } else {
    checks.push({
      checkId: "FVP-06",
      label: "id-mapping-validation",
      status: "passed",
      message: "Old-to-new ID mapping validation is declared.",
    });
  }

  // FVP-07: Linked record validation
  if (!plan.hasLinkedRecordValidation) {
    checks.push({
      checkId: "FVP-07",
      label: "linked-record-validation",
      status: "failed",
      message:
        "Linked record second-pass validation is not declared. Linked field references must be verified after the second-pass write phase completes.",
      remediation: "Declare hasLinkedRecordValidation: true in the final validation plan.",
    });
  } else {
    checks.push({
      checkId: "FVP-07",
      label: "linked-record-validation",
      status: "passed",
      message: "Linked record second-pass validation is declared.",
    });
  }

  // FVP-08: Attachment metadata validation
  if (!plan.hasAttachmentMetadataValidation) {
    checks.push({
      checkId: "FVP-08",
      label: "attachment-metadata-validation",
      status: "failed",
      message:
        "Attachment metadata validation is not declared. Attachment field metadata must be validated against the backup manifest.",
      remediation: "Declare hasAttachmentMetadataValidation: true in the final validation plan.",
    });
  } else {
    checks.push({
      checkId: "FVP-08",
      label: "attachment-metadata-validation",
      status: "passed",
      message: "Attachment metadata validation is declared.",
    });
  }

  // FVP-09: Attachment validation scope
  if (plan.hasAttachmentMetadataValidation && plan.attachmentValidationMetadataOnly) {
    checks.push({
      checkId: "FVP-09",
      label: "attachment-validation-scope",
      status: "warning",
      message:
        "Attachment validation is metadata-only. Attachment file content integrity cannot be confirmed without a file download. Manual re-attachment is required after restore.",
      remediation:
        "Note that attachment file content is not validated. Plan for manual re-attachment of all attachment fields after restore.",
    });
  } else {
    checks.push({
      checkId: "FVP-09",
      label: "attachment-validation-scope",
      status: "passed",
      message: "Attachment validation scope is acceptable.",
    });
  }

  // FVP-10: Manifest checksum validation
  if (!plan.hasManifestChecksumValidation) {
    checks.push({
      checkId: "FVP-10",
      label: "manifest-checksum-validation",
      status: "failed",
      message:
        "Manifest checksum and reference validation is not declared. The final package checksum must be verified against the backup manifest when available.",
      remediation: "Declare hasManifestChecksumValidation: true in the final validation plan.",
    });
  } else {
    checks.push({
      checkId: "FVP-10",
      label: "manifest-checksum-validation",
      status: "passed",
      message: "Manifest checksum and reference validation is declared.",
    });
  }

  // FVP-11: Success blocked without validation
  if (!plan.blocksSuccessWithoutValidation) {
    checks.push({
      checkId: "FVP-11",
      label: "success-blocked-without-validation",
      status: "failed",
      message:
        "The restore result is not declared to block success status without validation passing. A restore must not be marked succeeded unless all final validation checks pass.",
      remediation: "Declare blocksSuccessWithoutValidation: true in the final validation plan.",
    });
  } else {
    checks.push({
      checkId: "FVP-11",
      label: "success-blocked-without-validation",
      status: "passed",
      message: "Restore success is blocked until all validation checks pass.",
    });
  }

  // FVP-12: Writes remain disabled (always passes)
  checks.push({
    checkId: "FVP-12",
    label: "writes-remain-disabled",
    status: "passed",
    message:
      "Restore write execution is not enabled. Verifying this policy does not start any write operation.",
  });

  const hasBlocked = checks.some((c) => c.status === "failed");
  const hasWarning = checks.some((c) => c.status === "warning");
  const status: FinalValidationPolicyStatus = hasBlocked
    ? "blocked"
    : hasWarning
      ? "warning"
      : "compliant";

  const targetName = request.targetLabel ?? "the restore target";
  const message =
    status === "compliant"
      ? `Final validation plan for ${targetName} is compliant. All required validation steps are declared. Restore writes remain disabled — compliance does not enable writes or introduce a restore success state.`
      : status === "warning"
        ? `Final validation plan for ${targetName} has warnings. Review incomplete validation steps before proceeding. Restore writes remain disabled.`
        : `Final validation plan for ${targetName} is blocked. One or more required validation steps are missing. Restore writes remain disabled.`;

  return Promise.resolve({
    status,
    checks,
    message,
    planSummary: {
      hasSchemaCountValidation: plan.hasSchemaCountValidation,
      hasTableFieldValidation: plan.hasTableFieldValidation,
      hasRecordCountValidation: plan.hasRecordCountValidation,
      hasIdMappingValidation: plan.hasIdMappingValidation,
      hasLinkedRecordValidation: plan.hasLinkedRecordValidation,
      hasAttachmentMetadataValidation: plan.hasAttachmentMetadataValidation,
      attachmentValidationMetadataOnly: plan.attachmentValidationMetadataOnly,
      hasManifestChecksumValidation: plan.hasManifestChecksumValidation,
      blocksSuccessWithoutValidation: plan.blocksSuccessWithoutValidation,
    },
    noChangesMade: true,
    networkWritesAttempted: false,
    writesEnabled: false,
  });
}

function verifyWritePhaseOrderingPolicyImpl(
  request: WritePhaseOrderingPolicyRequest,
): Promise<WritePhaseOrderingPolicyResult> {
  const checks: WritePhaseOrderingPolicyResult["checks"] = [];

  // WPO-01: Write gate disabled (always passes)
  checks.push({
    checkId: "WPO-01",
    label: "write-gate-disabled",
    status: "passed",
    message: "Restore write gate is disabled. No live writes are possible.",
  });

  // WPO-02: Phase list declared
  const phases = request.phases;
  if (!phases) {
    checks.push({
      checkId: "WPO-02",
      label: "phase-list-declared",
      status: "failed",
      message:
        "No write phase list declared. A phase list is required before any live write path is considered.",
      remediation: "Declare a list of WritePhaseOrderingPolicyRequest phases in canonical order.",
    });
    return Promise.resolve({
      status: "blocked" as WritePhaseOrderingPolicyStatus,
      checks,
      message:
        "Write phase ordering is blocked. Resolve all phase ordering violations before any live write is considered. Restore writes remain disabled.",
      noChangesMade: true,
      networkWritesAttempted: false,
      writesEnabled: false,
    });
  }

  checks.push({
    checkId: "WPO-02",
    label: "phase-list-declared",
    status: "passed",
    message: `Write phase list is declared with ${phases.length} phases.`,
  });

  const CANONICAL_ORDER = [
    "preflight",
    "schemaCreate",
    "schemaVerify",
    "recordCreate",
    "recordVerify",
    "linkedRecordUpdate",
    "linkedRecordVerify",
    "attachmentMetadataVerify",
    "finalValidation",
  ] as const;

  const canonicalPos = (kind: string): number =>
    CANONICAL_ORDER.indexOf(kind as (typeof CANONICAL_ORDER)[number]);

  const isActive = (status: string) => status === "ready" || status === "completed";
  const findStatus = (kind: string) => phases.find((p) => p.kind === kind)?.status;

  // WPO-03: Canonical ordering
  let orderingViolation: string | null = null;
  let lastPos = -1;
  for (const phase of phases) {
    const pos = canonicalPos(phase.kind);
    if (pos >= 0) {
      if (pos < lastPos) {
        orderingViolation = `Phase '${phase.kind}' (canonical position ${pos + 1}) appears after a later canonical phase, which violates the required order.`;
        break;
      }
      lastPos = pos;
    }
  }

  if (orderingViolation) {
    checks.push({
      checkId: "WPO-03",
      label: "canonical-order",
      status: "failed",
      message: orderingViolation,
      remediation:
        "Reorder the phase list to match the canonical sequence: preflight → schema_create → schema_verify → record_create → record_verify → linked_record_update → linked_record_verify → attachment_metadata_verify → final_validation.",
    });
  } else {
    checks.push({
      checkId: "WPO-03",
      label: "canonical-order",
      status: "passed",
      message: "Declared phases respect the canonical ordering.",
    });
  }

  // WPO-04: Prerequisite phases present
  const prereqPairs: Array<[string, string, string]> = [
    ["schemaCreate", "schemaVerify", "schema_verify requires schema_create"],
    ["schemaVerify", "recordCreate", "record_create requires schema_verify"],
    ["recordCreate", "recordVerify", "record_verify requires record_create"],
    ["recordVerify", "linkedRecordUpdate", "linked_record_update requires record_verify"],
    [
      "linkedRecordUpdate",
      "linkedRecordVerify",
      "linked_record_verify requires linked_record_update",
    ],
    ["linkedRecordVerify", "finalValidation", "final_validation requires linked_record_verify"],
  ];

  let prereqViolation: string | null = null;
  for (const [prereq, dependent, description] of prereqPairs) {
    const depStatus = findStatus(dependent);
    if (depStatus && isActive(depStatus) && !findStatus(prereq)) {
      prereqViolation = `Phase '${dependent}' is active but its prerequisite '${prereq}' is not declared. ${description}.`;
      break;
    }
  }

  if (prereqViolation) {
    checks.push({
      checkId: "WPO-04",
      label: "prerequisite-phases-present",
      status: "failed",
      message: prereqViolation,
      remediation: "Ensure all prerequisite phases are declared before their dependent phases.",
    });
  } else {
    checks.push({
      checkId: "WPO-04",
      label: "prerequisite-phases-present",
      status: "passed",
      message: "All active phases have their prerequisites declared.",
    });
  }

  // WPO-05: record_create not active before schema_verify completed
  const recordCreateActive = isActive(findStatus("recordCreate") ?? "");
  const schemaVerifyCompleted = findStatus("schemaVerify") === "completed";
  if (recordCreateActive && !schemaVerifyCompleted) {
    checks.push({
      checkId: "WPO-05",
      label: "record-create-after-schema-verify",
      status: "failed",
      message:
        "record_create is active but schema_verify is not completed. Records cannot be created before the schema is verified.",
      remediation:
        "Ensure schema_verify reaches Completed status before record_create is set to Ready or Completed.",
    });
  } else {
    checks.push({
      checkId: "WPO-05",
      label: "record-create-after-schema-verify",
      status: "passed",
      message: "record_create ordering relative to schema_verify is safe.",
    });
  }

  // WPO-06: linked_record_update not active before record_verify completed
  const linkedUpdateActive = isActive(findStatus("linkedRecordUpdate") ?? "");
  const recordVerifyCompleted = findStatus("recordVerify") === "completed";
  if (linkedUpdateActive && !recordVerifyCompleted) {
    checks.push({
      checkId: "WPO-06",
      label: "linked-update-after-record-verify",
      status: "failed",
      message:
        "linked_record_update is active but record_verify is not completed. Linked record updates cannot begin before first-pass records are verified.",
      remediation:
        "Ensure record_verify reaches Completed status before linked_record_update is set to Ready or Completed.",
    });
  } else {
    checks.push({
      checkId: "WPO-06",
      label: "linked-update-after-record-verify",
      status: "passed",
      message: "linked_record_update ordering relative to record_verify is safe.",
    });
  }

  // WPO-07: final_validation not active before linked_record_verify completed
  const finalValidationActive = isActive(findStatus("finalValidation") ?? "");
  const linkedVerifyCompleted = findStatus("linkedRecordVerify") === "completed";
  if (finalValidationActive && !linkedVerifyCompleted) {
    checks.push({
      checkId: "WPO-07",
      label: "final-validation-after-linked-verify",
      status: "failed",
      message:
        "final_validation is active but linked_record_verify is not completed. Final validation cannot run before all linked record updates are verified.",
      remediation:
        "Ensure linked_record_verify reaches Completed status before final_validation is set to Ready or Completed.",
    });
  } else {
    checks.push({
      checkId: "WPO-07",
      label: "final-validation-after-linked-verify",
      status: "passed",
      message: "final_validation ordering relative to linked_record_verify is safe.",
    });
  }

  // WPO-08: No attachment upload or binary handling phase.
  // For AttachmentMetadataVerify: only block on unsafe *required* language.
  // For all other phases: block on any upload/binary/download language.
  const unsafeMetadataLanguage = (r: string) => {
    const l = r.toLowerCase();
    return (
      l.includes("upload required") ||
      l.includes("binary upload") ||
      l.includes("download required") ||
      l.includes("binary download") ||
      l.includes("file transfer") ||
      l.includes("attachment body")
    );
  };
  const unsafeGeneralLanguage = (r: string) => {
    const l = r.toLowerCase();
    return l.includes("upload") || l.includes("binary") || l.includes("download");
  };
  const hasUploadLanguage = phases.some((p) => {
    const r = p.skipReason ?? "";
    if (p.kind === "attachmentMetadataVerify") {
      return unsafeMetadataLanguage(r);
    }
    return unsafeGeneralLanguage(r);
  });
  if (hasUploadLanguage) {
    checks.push({
      checkId: "WPO-08",
      label: "no-attachment-upload-phase",
      status: "failed",
      message:
        "A phase skip reason contains attachment upload, binary, or download language. Attachment file upload and binary handling are not permitted in the restore write pipeline.",
      remediation:
        "Remove any attachment upload or binary download phase. The restore pipeline supports only attachment metadata validation (no file download).",
    });
  } else {
    checks.push({
      checkId: "WPO-08",
      label: "no-attachment-upload-phase",
      status: "passed",
      message: "No attachment upload or binary handling phase is declared.",
    });
  }

  // WPO-09: attachment_metadata_verify skip warning
  const attachmentPhase = phases.find((p) => p.kind === "attachmentMetadataVerify");
  if (attachmentPhase?.status === "skipped") {
    const hasMetadataReason =
      attachmentPhase.skipReason?.toLowerCase().includes("metadata") ?? false;
    if (hasMetadataReason) {
      checks.push({
        checkId: "WPO-09",
        label: "attachment-metadata-verify-scope",
        status: "warning",
        message:
          "attachment_metadata_verify is skipped with a metadata-only reason. Attachment file content integrity cannot be confirmed. Manual re-attachment is required after restore.",
        remediation:
          "Note that attachment file content is not validated. Plan for manual re-attachment of all attachment fields after restore.",
      });
    } else {
      checks.push({
        checkId: "WPO-09",
        label: "attachment-metadata-verify-scope",
        status: "warning",
        message:
          "attachment_metadata_verify is skipped without a metadata-only explanation. Provide a skip reason to document why attachment verification was omitted.",
        remediation:
          "Add a skip_reason to the attachment_metadata_verify phase explaining why it was skipped.",
      });
    }
  } else {
    checks.push({
      checkId: "WPO-09",
      label: "attachment-metadata-verify-scope",
      status: "passed",
      message: "attachment_metadata_verify is declared and not skipped.",
    });
  }

  // WPO-10: Writes remain disabled (always passes)
  checks.push({
    checkId: "WPO-10",
    label: "writes-remain-disabled",
    status: "passed",
    message:
      "Restore write gate remains disabled. Write phase ordering policy compliance does not enable writes.",
  });

  const hasBlocked = checks.some((c) => c.status === "failed");
  const hasWarning = checks.some((c) => c.status === "warning");
  const status: WritePhaseOrderingPolicyStatus = hasBlocked
    ? "blocked"
    : hasWarning
      ? "warning"
      : "compliant";

  const targetSuffix = request.targetLabel ? ` for target "${request.targetLabel}"` : "";
  const message =
    status === "compliant"
      ? `Write phase ordering is compliant${targetSuffix}. All phases are declared in canonical order with no unsafe transitions. Restore writes remain disabled — compliance does not enable writes or introduce a restore success state.`
      : status === "warning"
        ? `Write phase ordering has warnings${targetSuffix}. Review skipped or incomplete phases before proceeding. Restore writes remain disabled.`
        : `Write phase ordering is blocked${targetSuffix}. Resolve all phase ordering violations before any live write is considered. Restore writes remain disabled.`;

  const phaseSummary = phases.map((p) => ({
    kind: p.kind,
    status: p.status,
    canonicalPosition: canonicalPos(p.kind) + 1,
    skipReason: p.skipReason,
  }));

  return Promise.resolve({
    status,
    checks,
    message,
    phaseSummary,
    noChangesMade: true,
    networkWritesAttempted: false,
    writesEnabled: false,
  });
}

// ── Gate 13 — Failure Modes Policy ───────────────────────────────────────────

const REQUIRED_FAILURE_MODES = [
  "schemaCreateFailure",
  "schemaVerifyFailure",
  "recordCreateFailure",
  "idMappingFailure",
  "linkedRecordUpdateFailure",
  "checkpointPersistenceFailure",
  "rateLimitExhaustion",
  "targetMutationDetected",
  "finalValidationFailure",
  "unknownFailure",
] as const;

function stopsWrites(behavior: string): boolean {
  return (
    behavior === "stopAndReport" ||
    behavior === "stopPreserveCheckpointAndReport" ||
    behavior === "stopAfterRetryLimit" ||
    behavior === "blockAndRequireManualReview"
  );
}

function verifyFailureModesPolicyImpl(
  request: FailureModesPolicyRequest,
): Promise<FailureModesPolicyResult> {
  const checks: FailureModesPolicyResult["checks"] = [];

  // FMP-01: Write gate disabled (always passes in mock)
  checks.push({
    checkId: "FMP-01",
    label: "write-gate-disabled",
    status: "passed",
    message: "Restore write gate is disabled. No live writes are possible.",
  });

  // FMP-02: Plans declared
  const plans = request.handlingPlans;
  if (!plans || plans.length === 0) {
    checks.push({
      checkId: "FMP-02",
      label: "plans-declared",
      status: "failed",
      message:
        "No failure handling plans declared. Plans for all required failure modes are required before any live write path is considered.",
      remediation: "Declare a RestoreFailureHandlingPlan for each required failure mode.",
    });
    return Promise.resolve({
      status: "blocked" as FailureModesPolicyStatus,
      checks,
      message:
        "Failure modes policy is blocked. One or more required failure modes are missing or declare unsafe behavior. Resolve all violations before any live write is considered. Restore writes remain disabled.",
      noChangesMade: true,
      networkWritesAttempted: false,
      writesEnabled: false,
    });
  }

  checks.push({
    checkId: "FMP-02",
    label: "plans-declared",
    status: "passed",
    message: `Failure handling plans are declared (${plans.length} mode(s) covered).`,
  });

  // FMP-03: All required modes covered
  const missing = REQUIRED_FAILURE_MODES.filter((mode) => !plans.some((p) => p.mode === mode));
  if (missing.length > 0) {
    checks.push({
      checkId: "FMP-03",
      label: "all-required-modes-covered",
      status: "failed",
      message: `Missing handling plans for required failure mode(s): ${missing.join(", ")}.`,
      remediation: "Declare a RestoreFailureHandlingPlan for each missing failure mode.",
    });
  } else {
    checks.push({
      checkId: "FMP-03",
      label: "all-required-modes-covered",
      status: "passed",
      message: "All required failure modes have handling plans.",
    });
  }

  // FMP-04: No mode allows writes to continue after failure
  const continuesWrites = plans.filter((p) => !stopsWrites(p.stopBehavior)).map((p) => p.mode);
  if (continuesWrites.length > 0) {
    checks.push({
      checkId: "FMP-04",
      label: "no-continue-after-failure",
      status: "failed",
      message: `The following failure mode(s) do not stop writes after failure: ${continuesWrites.join(", ")}.`,
      remediation: "All failure modes must declare a stop behavior that halts further writes.",
    });
  } else {
    checks.push({
      checkId: "FMP-04",
      label: "no-continue-after-failure",
      status: "passed",
      message: "All declared failure modes stop further writes on failure.",
    });
  }

  // FMP-05: No destructive rollback
  const destructive = plans.filter((p) => p.triggersDestructiveRollback).map((p) => p.mode);
  if (destructive.length > 0) {
    checks.push({
      checkId: "FMP-05",
      label: "no-destructive-rollback",
      status: "failed",
      message: `The following failure mode(s) declare automatic destructive rollback: ${destructive.join(", ")}.`,
      remediation:
        "Remove automatic destructive rollback. Instead, stop writes and report the partial state for manual review.",
    });
  } else {
    checks.push({
      checkId: "FMP-05",
      label: "no-destructive-rollback",
      status: "passed",
      message: "No failure mode declares automatic destructive rollback.",
    });
  }

  // FMP-06: Unknown failure stops writes
  const unknownPlan = plans.find((p) => p.mode === "unknownFailure");
  if (unknownPlan && !stopsWrites(unknownPlan.stopBehavior)) {
    checks.push({
      checkId: "FMP-06",
      label: "unknown-failure-stops-writes",
      status: "failed",
      message:
        "The unknownFailure mode does not stop all writes. Any unclassified failure must immediately halt further write operations.",
      remediation:
        "Set stopBehavior to stopAndReport or stopPreserveCheckpointAndReport for unknownFailure.",
    });
  } else if (unknownPlan) {
    checks.push({
      checkId: "FMP-06",
      label: "unknown-failure-stops-writes",
      status: "passed",
      message: "Unknown/unclassified failure is declared to stop all writes.",
    });
  }

  // FMP-07: Rate-limit stops after retry
  const ratePlan = plans.find((p) => p.mode === "rateLimitExhaustion");
  if (ratePlan) {
    if (stopsWrites(ratePlan.stopBehavior)) {
      checks.push({
        checkId: "FMP-07",
        label: "rate-limit-stops-after-retry",
        status: "passed",
        message: "Rate-limit exhaustion is declared to stop after the configured retry limit.",
      });
    } else {
      checks.push({
        checkId: "FMP-07",
        label: "rate-limit-stops-after-retry",
        status: "failed",
        message: "Rate-limit exhaustion does not stop after a configured retry limit.",
        remediation: "Declare stopBehavior as stopAfterRetryLimit for rateLimitExhaustion.",
      });
    }
  }

  // FMP-08: Final validation failure blocks success
  const fvPlan = plans.find((p) => p.mode === "finalValidationFailure");
  if (fvPlan) {
    if (fvPlan.partialFailureLabeledSuccess || !stopsWrites(fvPlan.stopBehavior)) {
      checks.push({
        checkId: "FMP-08",
        label: "final-validation-blocks-success",
        status: "failed",
        message:
          "finalValidationFailure is declared to allow partial failure to be labeled as success or does not stop writes.",
        remediation:
          "Set partialFailureLabeledSuccess: false and a stop behavior for finalValidationFailure.",
      });
    } else {
      checks.push({
        checkId: "FMP-08",
        label: "final-validation-blocks-success",
        status: "passed",
        message: "Final validation failure is declared to block success and stop writes.",
      });
    }
  }

  // FMP-09: Checkpoint persistence failure blocks continuation
  const cpPlan = plans.find((p) => p.mode === "checkpointPersistenceFailure");
  if (cpPlan) {
    if (!stopsWrites(cpPlan.stopBehavior)) {
      checks.push({
        checkId: "FMP-09",
        label: "checkpoint-failure-blocks-continuation",
        status: "failed",
        message:
          "checkpointPersistenceFailure does not stop further writes. If a checkpoint cannot be written, the engine must not continue.",
        remediation:
          "Set stopBehavior to stopAndReport or stopPreserveCheckpointAndReport for checkpointPersistenceFailure.",
      });
    } else {
      checks.push({
        checkId: "FMP-09",
        label: "checkpoint-failure-blocks-continuation",
        status: "passed",
        message: "Checkpoint persistence failure is declared to block continuation.",
      });
    }
  }

  // FMP-10: No partial failure labeled as success
  const partialSuccess = plans.filter((p) => p.partialFailureLabeledSuccess).map((p) => p.mode);
  if (partialSuccess.length > 0) {
    checks.push({
      checkId: "FMP-10",
      label: "no-partial-failure-as-success",
      status: "failed",
      message: `The following failure mode(s) declare that a partial failure may be labeled success: ${partialSuccess.join(", ")}.`,
      remediation: "Set partialFailureLabeledSuccess: false for all failure modes.",
    });
  } else {
    checks.push({
      checkId: "FMP-10",
      label: "no-partial-failure-as-success",
      status: "passed",
      message: "No failure mode allows partial failure to be labeled as success.",
    });
  }

  // FMP-11: Writes remain disabled
  checks.push({
    checkId: "FMP-11",
    label: "writes-remain-disabled",
    status: "passed",
    message: "Restore writes remain disabled. This policy gate does not enable live writes.",
  });

  // Diagnostic context warnings
  let hasWarning = false;
  for (const plan of plans) {
    if (!plan.capturesDiagnosticContext) {
      checks.push({
        checkId: `FMP-W-${plan.mode}`,
        label: `diagnostic-context-${plan.mode}`,
        status: "warning",
        message: `Failure mode '${plan.mode}' does not capture diagnostic context.`,
        remediation: `Ensure '${plan.mode}' captures error details for post-failure review.`,
      });
      hasWarning = true;
    }
  }

  const isBlocked =
    missing.length > 0 ||
    continuesWrites.length > 0 ||
    destructive.length > 0 ||
    partialSuccess.length > 0 ||
    (unknownPlan !== undefined && !stopsWrites(unknownPlan.stopBehavior)) ||
    (fvPlan !== undefined &&
      (fvPlan.partialFailureLabeledSuccess || !stopsWrites(fvPlan.stopBehavior))) ||
    (cpPlan !== undefined && !stopsWrites(cpPlan.stopBehavior));

  const status: FailureModesPolicyStatus = isBlocked
    ? "blocked"
    : hasWarning
      ? "warning"
      : "compliant";

  const targetNote = request.targetLabel ? ` for '${request.targetLabel}'` : "";
  const message =
    status === "compliant"
      ? `Failure modes policy is compliant${targetNote}. All required failure modes have explicit, safe stop-behavior declarations. Restore writes remain disabled — compliance does not start any write operation and does not introduce a restore success state.`
      : status === "warning"
        ? `Failure modes policy has warnings${targetNote}. All required modes have safe stop behavior, but one or more modes have incomplete diagnostic context. Restore writes remain disabled.`
        : `Failure modes policy is blocked${targetNote}. One or more required failure modes are missing or declare unsafe behavior. Resolve all violations before any live write is considered. Restore writes remain disabled.`;

  const handlingSummary = plans.map((p) => ({
    mode: p.mode,
    stopBehavior: p.stopBehavior,
    preservesCheckpoint: p.preservesCheckpoint,
    triggersDestructiveRollback: p.triggersDestructiveRollback,
    capturesDiagnosticContext: p.capturesDiagnosticContext,
  }));

  return Promise.resolve({
    status,
    checks,
    message,
    handlingSummary,
    noChangesMade: true,
    networkWritesAttempted: false,
    writesEnabled: false,
  });
}

function verifyRollbackLimitationPolicyImpl(
  request: RollbackLimitationPolicyRequest,
): Promise<RollbackLimitationPolicyResult> {
  const checks: RollbackLimitationPolicyResult["checks"] = [];

  // RLP-01: Write gate disabled (always passes in mock)
  checks.push({
    checkId: "RLP-01",
    label: "write-gate-disabled",
    status: "passed",
    message: "Write gate is disabled. No restore writes are attempted.",
  });

  // RLP-02: Plan declared — short-circuit if absent
  const plan = request.plan;
  if (!plan) {
    checks.push({
      checkId: "RLP-02",
      label: "plan-declared",
      status: "failed",
      message:
        "No rollback limitation plan declared. A plan declaring rollback behavior, recovery guidance, and user-visible notice is required before any live write path is considered.",
      remediation:
        "Declare a RollbackLimitationPlan with rollbackBehavior, recoveryGuidance, and userVisibleLimitationNotice.",
    });
    return Promise.resolve({
      status: "blocked" as RollbackLimitationPolicyStatus,
      checks,
      message:
        "Rollback limitation policy is blocked. Automatic destructive rollback or cleanup is allowed, or required declarations are missing. Resolve all violations before any live write is considered. Restore writes remain disabled.",
      noChangesMade: true,
      networkWritesAttempted: false,
      writesEnabled: false,
    });
  }

  checks.push({
    checkId: "RLP-02",
    label: "plan-declared",
    status: "passed",
    message: "Rollback limitation plan is declared.",
  });

  let blocked = false;
  let hasWarning = false;

  // RLP-03: No automatic destructive rollback
  if (plan.rollbackBehavior === "automaticDestructiveRollback") {
    checks.push({
      checkId: "RLP-03",
      label: "no-automatic-destructive-rollback",
      status: "failed",
      message:
        "Automatic destructive rollback is declared. The restore engine must not automatically delete all created objects on failure.",
      remediation:
        "Set rollbackBehavior to noAutomaticRollback. Destructive cleanup must require explicit separate user action.",
    });
    blocked = true;
  } else {
    checks.push({
      checkId: "RLP-03",
      label: "no-automatic-destructive-rollback",
      status: "passed",
      message: "Automatic destructive rollback is not declared.",
    });
  }

  // RLP-04: No automatic delete cleanup
  if (plan.rollbackBehavior === "automaticDeleteCleanup") {
    checks.push({
      checkId: "RLP-04",
      label: "no-automatic-delete-cleanup",
      status: "failed",
      message:
        "Automatic delete cleanup is declared. The restore engine must not automatically delete partially created records on failure.",
      remediation:
        "Set rollbackBehavior to noAutomaticRollback. Delete cleanup must require explicit separate user action.",
    });
    blocked = true;
  } else {
    checks.push({
      checkId: "RLP-04",
      label: "no-automatic-delete-cleanup",
      status: "passed",
      message: "Automatic delete cleanup is not declared.",
    });
  }

  // RLP-05: No automatic update/revert cleanup
  if (plan.rollbackBehavior === "automaticUpdateRevertCleanup") {
    checks.push({
      checkId: "RLP-05",
      label: "no-automatic-update-revert-cleanup",
      status: "failed",
      message:
        "Automatic update/revert cleanup is declared. The restore engine must not automatically revert or update partially linked records on failure.",
      remediation:
        "Set rollbackBehavior to noAutomaticRollback. Revert cleanup must require explicit separate user action.",
    });
    blocked = true;
  } else {
    checks.push({
      checkId: "RLP-05",
      label: "no-automatic-update-revert-cleanup",
      status: "passed",
      message: "Automatic update/revert cleanup is not declared.",
    });
  }

  // RLP-06: Partial restore is not success
  if (!plan.partialRestoreIsNotSuccess) {
    checks.push({
      checkId: "RLP-06",
      label: "partial-restore-is-not-success",
      status: "failed",
      message:
        "Plan does not declare that partial restore is not success. A partial restore that fails mid-way must never be reported as a successful completion.",
      remediation: "Set partialRestoreIsNotSuccess: true in the rollback limitation plan.",
    });
    blocked = true;
  } else {
    checks.push({
      checkId: "RLP-06",
      label: "partial-restore-is-not-success",
      status: "passed",
      message: "Partial restore is explicitly declared as not a success state.",
    });
  }

  // RLP-07: Checkpoint-based recovery guidance
  if (plan.recoveryGuidance === "noneDeClared") {
    checks.push({
      checkId: "RLP-07",
      label: "checkpoint-recovery-guidance",
      status: "warning",
      message:
        "No recovery guidance is declared. Users must be informed of how to recover from a partial restore failure.",
      remediation: "Set recoveryGuidance to checkpointBasedResume or manualCleanupRequired.",
    });
    hasWarning = true;
  } else if (plan.recoveryGuidance !== "checkpointBasedResume") {
    checks.push({
      checkId: "RLP-07",
      label: "checkpoint-recovery-guidance",
      status: "warning",
      message:
        "Recovery guidance is declared but does not include checkpoint-based resume. Checkpoint-based recovery is preferred to avoid duplicating already-created objects.",
      remediation: "Consider adding checkpointBasedResume as the primary recovery guidance.",
    });
    hasWarning = true;
  } else {
    checks.push({
      checkId: "RLP-07",
      label: "checkpoint-recovery-guidance",
      status: "passed",
      message: "Checkpoint-based recovery guidance is declared.",
    });
  }

  // RLP-08: User-visible limitation notice
  if (!plan.userVisibleLimitationNotice) {
    checks.push({
      checkId: "RLP-08",
      label: "user-visible-limitation-notice",
      status: "warning",
      message:
        "No user-visible rollback limitation notice is declared. Users must be informed that automatic rollback is not available before any restore operation begins.",
      remediation:
        "Set userVisibleLimitationNotice: true and show the notice in the UI before restore execution.",
    });
    hasWarning = true;
  } else if (!plan.noticeIncludesLimitationDetails) {
    checks.push({
      checkId: "RLP-08",
      label: "user-visible-limitation-notice",
      status: "warning",
      message:
        "User-visible notice is declared but does not include limitation details. Incomplete notice may leave users unable to understand the risk.",
      remediation:
        "Set noticeIncludesLimitationDetails: true and ensure the notice describes which objects may remain and what manual cleanup is required.",
    });
    hasWarning = true;
  } else {
    checks.push({
      checkId: "RLP-08",
      label: "user-visible-limitation-notice",
      status: "passed",
      message: "User-visible rollback limitation notice with details is declared.",
    });
  }

  // RLP-09: Manual cleanup requires separate explicit future action
  if (!plan.manualCleanupRequiresSeparateAction) {
    checks.push({
      checkId: "RLP-09",
      label: "manual-cleanup-separate-action",
      status: "failed",
      message:
        "Plan does not declare that manual cleanup requires a separate, explicit future user action. The restore engine must never trigger cleanup automatically.",
      remediation:
        "Set manualCleanupRequiresSeparateAction: true. Cleanup tooling must be a separate future feature requiring explicit user initiation.",
    });
    blocked = true;
  } else {
    checks.push({
      checkId: "RLP-09",
      label: "manual-cleanup-separate-action",
      status: "passed",
      message: "Manual cleanup is declared to require a separate, explicit future user action.",
    });
  }

  // RLP-10: No token/path/payload (always passes)
  checks.push({
    checkId: "RLP-10",
    label: "no-token-path-payload",
    status: "passed",
    message: "No token, filesystem path, or record payload is present in any result field.",
  });

  // RLP-11: No network writes (always passes)
  checks.push({
    checkId: "RLP-11",
    label: "no-network-writes",
    status: "passed",
    message: "No network writes have been attempted. networkWritesAttempted is always false.",
  });

  // RLP-12: Writes remain disabled (always passes)
  checks.push({
    checkId: "RLP-12",
    label: "writes-remain-disabled",
    status: "passed",
    message: "Restore writes remain disabled. Policy compliance does not enable write execution.",
  });

  const status: RollbackLimitationPolicyStatus = blocked
    ? "blocked"
    : hasWarning
      ? "warning"
      : "compliant";

  const planSummary = {
    rollbackBehavior: plan.rollbackBehavior,
    partialRestoreIsNotSuccess: plan.partialRestoreIsNotSuccess,
    recoveryGuidanceDeclared: plan.recoveryGuidance !== "noneDeClared",
    includesCheckpointGuidance: plan.recoveryGuidance === "checkpointBasedResume",
    userVisibleNotice: plan.userVisibleLimitationNotice,
    manualCleanupRequiresSeparateAction: plan.manualCleanupRequiresSeparateAction,
  };

  const label = request.targetLabel ? ` for '${request.targetLabel}'` : "";
  const message =
    status === "compliant"
      ? `Rollback limitation policy is compliant${label}. Automatic destructive rollback and cleanup are disabled. Partial restore is not success. Recovery guidance and user-visible limitation notice are declared. Restore writes remain disabled.`
      : status === "warning"
        ? `Rollback limitation policy has warnings${label}. Automatic rollback is safely disabled, but recovery guidance or the user-visible notice is incomplete. Restore writes remain disabled.`
        : `Rollback limitation policy is blocked${label}. Automatic destructive rollback or cleanup is allowed, or required declarations are missing. Resolve all violations before any live write is considered. Restore writes remain disabled.`;

  return Promise.resolve({
    status,
    checks,
    message,
    planSummary,
    noChangesMade: true,
    networkWritesAttempted: false,
    writesEnabled: false,
  });
}

export const mockAirBridgeService: AirBridgeService = {
  listConnections,
  listWorkspaces,
  listBases,
  listBackupPackages,
  listRestorePlans,
  listReports,
  listLogs,
  listCompatibilityRules,
  checkConnection: checkConnectionImpl,
  listAccessibleBases: listAccessibleBasesImpl,
  getBaseSchema: getBaseSchemaImpl,
  createBackupPlan: createBackupPlanImpl,
  createRecordsExportPlan: createRecordsExportPlanImpl,
  validateBackupOutputPath: validateBackupOutputPathImpl,
  runBackupJob: runBackupJobImpl,
  cancelBackupJob: cancelBackupJobImpl,
  getBackupJobStatus: getBackupJobStatusImpl,
  inspectBackupPackage: inspectBackupPackageImpl,
  createRestoreDryRunPlan: createRestoreDryRunPlanImpl,
  runRestoreExecution: runRestoreExecutionImpl,
  createRestoreSchemaPlan: createRestoreSchemaPlanImpl,
  createRestoreRecordImportPlan: createRestoreRecordImportPlanImpl,
  listJobHistory: listJobHistoryImpl,
  clearJobHistory: clearJobHistoryImpl,
  previewRestoreWriteEngine: previewRestoreWriteEngineImpl,
  getCredentialStorageStatus: getCredentialStorageStatusImpl,
  saveAirtableTokenToKeychain: saveAirtableTokenToKeychainImpl,
  removeAirtableTokenFromKeychain: removeAirtableTokenFromKeychainImpl,
  previewSchemaWriteRequestPlan: previewSchemaWriteRequestPlanImpl,
  previewRecordWriteRequestPlan: previewRecordWriteRequestPlanImpl,
  verifyRestoreSandboxEnvironment: verifyRestoreSandboxEnvironmentImpl,
  validateRestoreConfirmationGate: validateRestoreConfirmationGateImpl,
  verifyRestoreTargetEmpty: verifyRestoreTargetEmptyImpl,
  verifyDestructiveOperationPolicy: verifyDestructiveOperationPolicyImpl,
  verifyAttachmentUploadPolicy: verifyAttachmentUploadPolicyImpl,
  verifySchemaRecordOrderPolicy: verifySchemaRecordOrderPolicyImpl,
  verifySandboxWriteTestingPolicy: verifySandboxWriteTestingPolicyImpl,
  verifyLiveWriteConfirmationPolicy: verifyLiveWriteConfirmationPolicyImpl,
  verifyRateLimitBackoffPolicy: verifyRateLimitBackoffPolicyImpl,
  verifyCheckpointDurabilityPolicy: verifyCheckpointDurabilityPolicyImpl,
  verifyFinalValidationPolicy: verifyFinalValidationPolicyImpl,
  verifyWritePhaseOrderingPolicy: verifyWritePhaseOrderingPolicyImpl,
  verifyFailureModesPolicy: verifyFailureModesPolicyImpl,
  verifyRollbackLimitationPolicy: verifyRollbackLimitationPolicyImpl,
};
