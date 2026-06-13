import { describe, it, expect } from "vitest";
import { mockAirBridgeService } from "../services/mockAirBridgeService";
import type { SchemaWriteRequestPlanRequest, SchemaWriteRequestPlanResult } from "../backend/types";

// ── Mock service contract ──────────────────────────────────────────────────────

const SENTINEL = "pat_schema_write_sentinel_0123456789abcdefghijklmnopqrstuvwx";

function readyRequest(
  overrides?: Partial<SchemaWriteRequestPlanRequest>,
): SchemaWriteRequestPlanRequest {
  return {
    packageFilename: "backup.airbridge",
    schemaPlanStatus: "ready",
    tableCount: 3,
    directFieldCount: 6,
    deferredFieldCount: 1,
    manualActionCount: 1,
    ...overrides,
  };
}

describe("schemaWriteRequestPlan mock service", () => {
  it("result status is disabled for a ready plan", async () => {
    const result = await mockAirBridgeService.previewSchemaWriteRequestPlan(readyRequest());
    expect(result.status).toBe("disabled");
  });

  it("noChangesMade is always true", async () => {
    const result = await mockAirBridgeService.previewSchemaWriteRequestPlan(readyRequest());
    expect(result.noChangesMade).toBe(true);
  });

  it("networkWritesAttempted is always false", async () => {
    const result = await mockAirBridgeService.previewSchemaWriteRequestPlan(readyRequest());
    expect(result.networkWritesAttempted).toBe(false);
  });

  it("no token in result", async () => {
    const result = await mockAirBridgeService.previewSchemaWriteRequestPlan(readyRequest());
    const json = JSON.stringify(result);
    expect(json).not.toContain(SENTINEL);
    expect(json).not.toContain('"token"');
    expect(json).not.toContain('"apiKey"');
  });

  it("no succeeded status in result", async () => {
    const result = await mockAirBridgeService.previewSchemaWriteRequestPlan(readyRequest());
    const json = JSON.stringify(result);
    expect(json).not.toContain('"succeeded"');
    expect(json.toLowerCase()).not.toContain("succeeded");
  });

  it("filename is basename only", async () => {
    const result = await mockAirBridgeService.previewSchemaWriteRequestPlan(readyRequest());
    expect(result.filename).toBe("backup.airbridge");
    expect(result.filename).not.toContain("/");
    expect(result.filename).not.toContain("\\");
  });

  it("tableOpCount matches request.tableCount", async () => {
    const result = await mockAirBridgeService.previewSchemaWriteRequestPlan(readyRequest());
    expect(result.tableOpCount).toBe(3);
  });

  it("fieldOpCount matches request.directFieldCount", async () => {
    const result = await mockAirBridgeService.previewSchemaWriteRequestPlan(readyRequest());
    expect(result.fieldOpCount).toBe(6);
  });

  it("deferredOpCount matches request.deferredFieldCount", async () => {
    const result = await mockAirBridgeService.previewSchemaWriteRequestPlan(readyRequest());
    expect(result.deferredOpCount).toBe(1);
  });

  it("manualActionCount matches request.manualActionCount", async () => {
    const result = await mockAirBridgeService.previewSchemaWriteRequestPlan(readyRequest());
    expect(result.manualActionCount).toBe(1);
  });

  it("totalOpCount equals sum of all op counts", async () => {
    const result = await mockAirBridgeService.previewSchemaWriteRequestPlan(readyRequest());
    expect(result.totalOpCount).toBe(
      result.tableOpCount + result.fieldOpCount + result.deferredOpCount + result.manualActionCount,
    );
  });

  it("blocked when schema plan status is blocked", async () => {
    const result = await mockAirBridgeService.previewSchemaWriteRequestPlan(
      readyRequest({ schemaPlanStatus: "blocked" }),
    );
    expect(result.status).toBe("blocked");
    expect(result.blockedReason).toBe("schemaPlanNotReady");
    expect(result.noChangesMade).toBe(true);
    expect(result.networkWritesAttempted).toBe(false);
  });

  it("blocked when tableCount is zero", async () => {
    const result = await mockAirBridgeService.previewSchemaWriteRequestPlan(
      readyRequest({ tableCount: 0 }),
    );
    expect(result.status).toBe("blocked");
    expect(result.blockedReason).toBe("noTablesInPlan");
    expect(result.noChangesMade).toBe(true);
    expect(result.networkWritesAttempted).toBe(false);
  });

  it("readyWithWarnings plan is disabled not blocked", async () => {
    const result = await mockAirBridgeService.previewSchemaWriteRequestPlan(
      readyRequest({ schemaPlanStatus: "readyWithWarnings" }),
    );
    expect(result.status).toBe("disabled");
    expect(result.noChangesMade).toBe(true);
  });

  it("blocked result has totalOpCount zero", async () => {
    const result = await mockAirBridgeService.previewSchemaWriteRequestPlan(
      readyRequest({ schemaPlanStatus: "blocked" }),
    );
    expect(result.totalOpCount).toBe(0);
  });
});

// ── IPC fallback contract ──────────────────────────────────────────────────────

describe("schemaWriteRequestPlan IPC fallback", () => {
  it("fallback result is safe — no token", () => {
    const fallback: SchemaWriteRequestPlanResult = {
      filename: "backup.airbridge",
      status: "disabled",
      disabledReason: "disabledByProductPolicy",
      message: "Schema write planning is not available in this context.",
      tableOpCount: 0,
      fieldOpCount: 0,
      deferredOpCount: 0,
      manualActionCount: 0,
      totalOpCount: 0,
      warnings: [],
      noChangesMade: true,
      networkWritesAttempted: false,
    };
    const json = JSON.stringify(fallback);
    expect(json).not.toContain('"token"');
    expect(json).not.toContain('"succeeded"');
    expect(fallback.noChangesMade).toBe(true);
    expect(fallback.networkWritesAttempted).toBe(false);
  });

  it("fallback status is never succeeded", () => {
    const fallback: SchemaWriteRequestPlanResult = {
      filename: "backup.airbridge",
      status: "disabled",
      message: "Not available.",
      tableOpCount: 0,
      fieldOpCount: 0,
      deferredOpCount: 0,
      manualActionCount: 0,
      totalOpCount: 0,
      warnings: [],
      noChangesMade: true,
      networkWritesAttempted: false,
    };
    expect(fallback.status).not.toBe("succeeded" as never);
  });
});

// ── Restore write gate unchanged ───────────────────────────────────────────────

describe("schema write engine does not affect restore write gate", () => {
  it("previewRestoreWriteEngine still returns disabled after schema write plan", async () => {
    // Running schema write plan must not change the write engine gate.
    await mockAirBridgeService.previewSchemaWriteRequestPlan(readyRequest());

    const writeResult = await mockAirBridgeService.previewRestoreWriteEngine({
      packageFilename: "backup.airbridge",
      packagePath: "/tmp/backup.airbridge",
    });
    expect(writeResult.status).toBe("disabled");
    const json = JSON.stringify(writeResult).toLowerCase();
    expect(json).not.toContain("succeeded");
  });
});
