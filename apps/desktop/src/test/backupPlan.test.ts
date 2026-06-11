import { describe, it, expect } from "vitest";
import { mockAirBridgeService } from "../services/mockAirBridgeService";
import type { BackupPlanRequest } from "../backend/types";

// Fixture: a request with attachment, linked-record, and formula fields
const MIXED_REQUEST: BackupPlanRequest = {
  baseId: "appTest001",
  baseName: "Test Base",
  scope: "full",
  tables: [
    {
      id: "tblA",
      name: "Projects",
      fields: [
        { id: "fld1", name: "Name", fieldType: "singleLineText" },
        { id: "fld2", name: "Status", fieldType: "singleSelect" },
        { id: "fld3", name: "Formula", fieldType: "formula" },
        { id: "fld4", name: "Attachments", fieldType: "multipleAttachments" },
      ],
    },
    {
      id: "tblB",
      name: "Tasks",
      fields: [
        { id: "fld5", name: "Title", fieldType: "singleLineText" },
        { id: "fld6", name: "Linked", fieldType: "multipleRecordLinks" },
      ],
    },
  ],
};

describe("mockAirBridgeService.createBackupPlan", () => {
  it("returns a plan for a valid request", async () => {
    const plan = await mockAirBridgeService.createBackupPlan(MIXED_REQUEST);
    expect(plan).toBeDefined();
  });

  it("plan.dryRun is always true", async () => {
    const plan = await mockAirBridgeService.createBackupPlan(MIXED_REQUEST);
    expect(plan.dryRun).toBe(true);
  });

  it("plan.outputPackagePath is always absent", async () => {
    const plan = await mockAirBridgeService.createBackupPlan(MIXED_REQUEST);
    expect(plan.outputPackagePath).toBeUndefined();
  });

  it("plan.baseId and baseName match request", async () => {
    const plan = await mockAirBridgeService.createBackupPlan(MIXED_REQUEST);
    expect(plan.baseId).toBe("appTest001");
    expect(plan.baseName).toBe("Test Base");
  });

  it("plan.tableCount equals number of tables in request", async () => {
    const plan = await mockAirBridgeService.createBackupPlan(MIXED_REQUEST);
    expect(plan.tableCount).toBe(2);
  });

  it("plan.totalFieldCount equals sum of all fields", async () => {
    const plan = await mockAirBridgeService.createBackupPlan(MIXED_REQUEST);
    expect(plan.totalFieldCount).toBe(6);
  });

  it("all tables from request are included in plan", async () => {
    const plan = await mockAirBridgeService.createBackupPlan(MIXED_REQUEST);
    const ids = plan.tables.map((t) => t.id);
    expect(ids).toContain("tblA");
    expect(ids).toContain("tblB");
  });

  it("attachment field produces ATTACHMENT_METADATA_ONLY warning", async () => {
    const plan = await mockAirBridgeService.createBackupPlan(MIXED_REQUEST);
    const allWarnings = plan.warnings;
    const attachmentWarning = allWarnings.find((w) => w.code === "ATTACHMENT_METADATA_ONLY");
    expect(attachmentWarning).toBeDefined();
    expect(attachmentWarning?.severity).toBe("warning");
    expect(attachmentWarning?.tableName).toBe("Projects");
  });

  it("linked record field produces LINKED_RECORD_REMAPPING warning", async () => {
    const plan = await mockAirBridgeService.createBackupPlan(MIXED_REQUEST);
    const linkedWarning = plan.warnings.find((w) => w.code === "LINKED_RECORD_REMAPPING");
    expect(linkedWarning).toBeDefined();
    expect(linkedWarning?.severity).toBe("warning");
    expect(linkedWarning?.tableName).toBe("Tasks");
  });

  it("formula field produces COMPUTED_FIELD info notice", async () => {
    const plan = await mockAirBridgeService.createBackupPlan(MIXED_REQUEST);
    const computedNotice = plan.warnings.find((w) => w.code === "COMPUTED_FIELD");
    expect(computedNotice).toBeDefined();
    expect(computedNotice?.severity).toBe("info");
  });

  it("plan result does not contain token-like strings", async () => {
    const plan = await mockAirBridgeService.createBackupPlan(MIXED_REQUEST);
    const json = JSON.stringify(plan);
    expect(json).not.toMatch(/pat[A-Za-z0-9_]{10,}/);
    expect(json).not.toContain("Bearer ");
  });

  it("plan.estimate.recordReadPages is unknown when no record counts provided", async () => {
    const plan = await mockAirBridgeService.createBackupPlan(MIXED_REQUEST);
    expect(plan.estimate.recordReadPages.type).toBe("unknown");
  });

  it("restorable fields are counted correctly in compatibility summary", async () => {
    const plan = await mockAirBridgeService.createBackupPlan(MIXED_REQUEST);
    // singleLineText×2, singleSelect×1 → 3 restorable
    expect(plan.compatibility.restorableCount).toBe(3);
  });

  it("metadataOnly fields are counted correctly in compatibility summary", async () => {
    const plan = await mockAirBridgeService.createBackupPlan(MIXED_REQUEST);
    // formula×1 → 1 metadata-only
    expect(plan.compatibility.metadataOnlyCount).toBe(1);
  });

  it("unknown fields are counted correctly in compatibility summary", async () => {
    const plan = await mockAirBridgeService.createBackupPlan(MIXED_REQUEST);
    // multipleAttachments×1, multipleRecordLinks×1 → 2 unknown
    expect(plan.compatibility.unknownCount).toBe(2);
  });

  it("compatibility totalCount equals totalFieldCount", async () => {
    const plan = await mockAirBridgeService.createBackupPlan(MIXED_REQUEST);
    expect(plan.compatibility.totalCount).toBe(plan.totalFieldCount);
  });

  it("empty tables request returns zero-count plan with dryRun true", async () => {
    const emptyReq: BackupPlanRequest = {
      baseId: "appEmpty",
      baseName: "Empty Base",
      scope: "schemaOnly",
      tables: [],
    };
    const plan = await mockAirBridgeService.createBackupPlan(emptyReq);
    expect(plan.tableCount).toBe(0);
    expect(plan.totalFieldCount).toBe(0);
    expect(plan.dryRun).toBe(true);
    expect(plan.warnings).toHaveLength(0);
  });

  it("known record count propagates to table plan", async () => {
    const req: BackupPlanRequest = {
      baseId: "appKnown",
      baseName: "Known Base",
      scope: "full",
      tables: [
        {
          id: "tblK",
          name: "Items",
          fields: [{ id: "f1", name: "Name", fieldType: "singleLineText" }],
          recordCount: 250,
        },
      ],
    };
    const plan = await mockAirBridgeService.createBackupPlan(req);
    expect(plan.tables[0].recordCount).toBe(250);
  });

  it("plan.scope matches request scope", async () => {
    const plan = await mockAirBridgeService.createBackupPlan(MIXED_REQUEST);
    expect(plan.scope).toBe("full");
  });
});
