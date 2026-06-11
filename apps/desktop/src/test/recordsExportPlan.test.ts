import { describe, it, expect } from "vitest";
import { mockAirBridgeService } from "../services/mockAirBridgeService";
import type { BackupPlanRequest, RecordsExportPlanRequest } from "../backend/types";

// Build a deterministic backup plan first, then use it for export planning.
async function makePlanRequest(tables: BackupPlanRequest["tables"]): Promise<BackupPlanRequest> {
  return {
    baseId: "appExport001",
    baseName: "Export Test Base",
    scope: "full",
    tables,
  };
}

const MIXED_TABLES: BackupPlanRequest["tables"] = [
  {
    id: "tblAlpha",
    name: "Projects",
    fields: [
      { id: "f01", name: "Name", fieldType: "singleLineText" },
      { id: "f02", name: "Files", fieldType: "multipleAttachments" },
    ],
    recordCount: 250,
  },
  {
    id: "tblBeta",
    name: "Tasks",
    fields: [
      { id: "f03", name: "Title", fieldType: "singleLineText" },
      { id: "f04", name: "Linked", fieldType: "multipleRecordLinks" },
    ],
    // no record count — unknown
  },
];

async function makeMixedExportRequest(): Promise<RecordsExportPlanRequest> {
  const planReq = await makePlanRequest(MIXED_TABLES);
  const backupPlan = await mockAirBridgeService.createBackupPlan(planReq);
  return {
    baseId: "appExport001",
    baseName: "Export Test Base",
    backupPlan,
  };
}

describe("mockAirBridgeService.createRecordsExportPlan", () => {
  it("returns a plan for a valid request", async () => {
    const req = await makeMixedExportRequest();
    const plan = await mockAirBridgeService.createRecordsExportPlan(req);
    expect(plan).toBeDefined();
  });

  it("plan.plannedOnly is always true", async () => {
    const req = await makeMixedExportRequest();
    const plan = await mockAirBridgeService.createRecordsExportPlan(req);
    expect(plan.plannedOnly).toBe(true);
  });

  it("plan.outputPackagePath is always absent", async () => {
    const req = await makeMixedExportRequest();
    const plan = await mockAirBridgeService.createRecordsExportPlan(req);
    expect(plan.outputPackagePath).toBeUndefined();
  });

  it("plan.tableCount equals number of backup plan tables", async () => {
    const req = await makeMixedExportRequest();
    const plan = await mockAirBridgeService.createRecordsExportPlan(req);
    expect(plan.tableCount).toBe(2);
  });

  it("table with known record count shows correct state", async () => {
    const req = await makeMixedExportRequest();
    const plan = await mockAirBridgeService.createRecordsExportPlan(req);
    const alpha = plan.tables.find((t) => t.tableId === "tblAlpha");
    expect(alpha?.recordCount.type).toBe("known");
    if (alpha?.recordCount.type === "known") {
      expect(alpha.recordCount.count).toBe(250);
    }
  });

  it("table with unknown record count shows unknown state", async () => {
    const req = await makeMixedExportRequest();
    const plan = await mockAirBridgeService.createRecordsExportPlan(req);
    const beta = plan.tables.find((t) => t.tableId === "tblBeta");
    expect(beta?.recordCount.type).toBe("unknown");
  });

  it("known record count produces correct page estimate", async () => {
    const req = await makeMixedExportRequest();
    const plan = await mockAirBridgeService.createRecordsExportPlan(req);
    const alpha = plan.tables.find((t) => t.tableId === "tblAlpha");
    // 250 records ÷ 100 per page = 3 pages
    expect(alpha?.requestEstimate.type).toBe("known");
    if (alpha?.requestEstimate.type === "known") {
      expect(alpha.requestEstimate.pages).toBe(3);
    }
  });

  it("unknown record count produces unknown page estimate", async () => {
    const req = await makeMixedExportRequest();
    const plan = await mockAirBridgeService.createRecordsExportPlan(req);
    const beta = plan.tables.find((t) => t.tableId === "tblBeta");
    expect(beta?.requestEstimate.type).toBe("unknown");
  });

  it("table has JSONL output entry path", async () => {
    const req = await makeMixedExportRequest();
    const plan = await mockAirBridgeService.createRecordsExportPlan(req);
    const alpha = plan.tables.find((t) => t.tableId === "tblAlpha");
    expect(alpha?.jsonlOutput.entryPath).toContain("tblAlpha");
    expect(alpha?.jsonlOutput.entryPath).toContain("records.jsonl");
  });

  it("JSONL output entry path has no absolute path", async () => {
    const req = await makeMixedExportRequest();
    const plan = await mockAirBridgeService.createRecordsExportPlan(req);
    for (const t of plan.tables) {
      expect(t.jsonlOutput.entryPath).not.toMatch(/^\//);
      expect(t.jsonlOutput.entryPath).not.toContain("Users/");
    }
  });

  it("attachment field creates attachment metadata extraction plan", async () => {
    const req = await makeMixedExportRequest();
    const plan = await mockAirBridgeService.createRecordsExportPlan(req);
    const alpha = plan.tables.find((t) => t.tableId === "tblAlpha");
    expect(alpha?.attachmentPlans.length).toBeGreaterThan(0);
    expect(alpha?.attachmentPlans[0].policy).toBe("metadataOnly");
  });

  it("linked record field creates linked record extraction plan", async () => {
    const req = await makeMixedExportRequest();
    const plan = await mockAirBridgeService.createRecordsExportPlan(req);
    const beta = plan.tables.find((t) => t.tableId === "tblBeta");
    expect(beta?.linkedRecordPlans.length).toBeGreaterThan(0);
    expect(beta?.linkedRecordPlans[0].policy).toBe("remappingRequiredForRestore");
  });

  it("UNKNOWN_RECORD_COUNT warning generated for table without count", async () => {
    const req = await makeMixedExportRequest();
    const plan = await mockAirBridgeService.createRecordsExportPlan(req);
    const allCodes = plan.warnings.map((w) => w.code);
    expect(allCodes).toContain("UNKNOWN_RECORD_COUNT");
  });

  it("ATTACHMENT_METADATA_ONLY warning generated for attachment fields", async () => {
    const req = await makeMixedExportRequest();
    const plan = await mockAirBridgeService.createRecordsExportPlan(req);
    const allCodes = plan.warnings.map((w) => w.code);
    expect(allCodes).toContain("ATTACHMENT_METADATA_ONLY");
  });

  it("LINKED_RECORD_REMAPPING warning generated for linked fields", async () => {
    const req = await makeMixedExportRequest();
    const plan = await mockAirBridgeService.createRecordsExportPlan(req);
    const allCodes = plan.warnings.map((w) => w.code);
    expect(allCodes).toContain("LINKED_RECORD_REMAPPING");
  });

  it("plan does not contain token-like strings", async () => {
    const req = await makeMixedExportRequest();
    const plan = await mockAirBridgeService.createRecordsExportPlan(req);
    const json = JSON.stringify(plan);
    expect(json).not.toMatch(/pat[A-Za-z0-9_]{10,}/);
    expect(json).not.toContain("Bearer ");
  });

  it("plan baseId and baseName match request", async () => {
    const req = await makeMixedExportRequest();
    const plan = await mockAirBridgeService.createRecordsExportPlan(req);
    expect(plan.baseId).toBe("appExport001");
    expect(plan.baseName).toBe("Export Test Base");
  });

  it("page size is 100", async () => {
    const req = await makeMixedExportRequest();
    const plan = await mockAirBridgeService.createRecordsExportPlan(req);
    expect(plan.pageSize).toBe(100);
    for (const t of plan.tables) {
      expect(t.pageSize).toBe(100);
    }
  });

  it("JSONL output is marked as planned only", async () => {
    const req = await makeMixedExportRequest();
    const plan = await mockAirBridgeService.createRecordsExportPlan(req);
    for (const t of plan.tables) {
      expect(t.jsonlOutput.plannedOnly).toBe(true);
    }
  });

  it("empty backup plan yields zero-count export plan", async () => {
    const emptyPlan = await mockAirBridgeService.createBackupPlan({
      baseId: "appEmpty",
      baseName: "Empty",
      scope: "full",
      tables: [],
    });
    const req: RecordsExportPlanRequest = {
      baseId: "appEmpty",
      baseName: "Empty",
      backupPlan: emptyPlan,
    };
    const plan = await mockAirBridgeService.createRecordsExportPlan(req);
    expect(plan.tableCount).toBe(0);
    expect(plan.plannedOnly).toBe(true);
    expect(plan.warnings).toHaveLength(0);
  });
});
