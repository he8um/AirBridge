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
};
