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
  RunBackupCommandRequest,
  RunBackupCommandResponse,
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

export const mockCheckConnection = checkConnectionImpl;

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
};
