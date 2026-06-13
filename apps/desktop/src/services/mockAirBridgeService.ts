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
};
