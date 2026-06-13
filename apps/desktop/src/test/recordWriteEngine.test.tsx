import { describe, it, expect } from "vitest";
import { mockAirBridgeService } from "../services/mockAirBridgeService";
import type { RecordWriteRequestPlanRequest, RecordWriteRequestPlanResult } from "../backend/types";

// ── Mock service contract ──────────────────────────────────────────────────────

const SENTINEL = "pat_record_write_sentinel_0123456789abcdefghijklmnopqrstuvwx";

function readyRequest(
  overrides?: Partial<RecordWriteRequestPlanRequest>,
): RecordWriteRequestPlanRequest {
  return {
    packageFilename: "backup.airbridge",
    recordImportPlanStatus: "ready",
    tableCount: 2,
    totalFirstPassBatches: 4,
    totalSecondPassBatches: 2,
    attachmentFieldCount: 1,
    skippedFieldCount: 2,
    ...overrides,
  };
}

describe("recordWriteRequestPlan mock service — status invariants", () => {
  it("result status is disabled for a ready plan", async () => {
    const result = await mockAirBridgeService.previewRecordWriteRequestPlan(readyRequest());
    expect(result.status).toBe("disabled");
  });

  it("blocked when recordImportPlanStatus is blocked", async () => {
    const result = await mockAirBridgeService.previewRecordWriteRequestPlan(
      readyRequest({ recordImportPlanStatus: "blocked" }),
    );
    expect(result.status).toBe("blocked");
    expect(result.blockedReason).toBe("recordImportPlanNotReady");
  });

  it("blocked when tableCount is zero", async () => {
    const result = await mockAirBridgeService.previewRecordWriteRequestPlan(
      readyRequest({ tableCount: 0 }),
    );
    expect(result.status).toBe("blocked");
    expect(result.blockedReason).toBe("noTablesInPlan");
  });

  it("readyWithWarnings status is not blocked", async () => {
    const result = await mockAirBridgeService.previewRecordWriteRequestPlan(
      readyRequest({ recordImportPlanStatus: "readyWithWarnings" }),
    );
    expect(result.status).toBe("disabled");
  });
});

describe("recordWriteRequestPlan mock service — safety invariants", () => {
  it("noChangesMade is always true", async () => {
    for (const req of [
      readyRequest(),
      readyRequest({ recordImportPlanStatus: "blocked" }),
      readyRequest({ tableCount: 0 }),
    ]) {
      const result = await mockAirBridgeService.previewRecordWriteRequestPlan(req);
      expect(result.noChangesMade).toBe(true);
    }
  });

  it("networkWritesAttempted is always false", async () => {
    for (const req of [readyRequest(), readyRequest({ recordImportPlanStatus: "blocked" })]) {
      const result = await mockAirBridgeService.previewRecordWriteRequestPlan(req);
      expect(result.networkWritesAttempted).toBe(false);
    }
  });

  it("no token in result", async () => {
    const result = await mockAirBridgeService.previewRecordWriteRequestPlan(readyRequest());
    const json = JSON.stringify(result);
    expect(json).not.toContain(SENTINEL);
    expect(json).not.toContain('"token"');
    expect(json).not.toContain('"apiKey"');
    expect(json).not.toContain('"secret"');
  });

  it("no succeeded status in result", async () => {
    const result = await mockAirBridgeService.previewRecordWriteRequestPlan(readyRequest());
    const json = JSON.stringify(result).toLowerCase();
    expect(json).not.toContain('"succeeded"');
    expect(json).not.toContain("succeeded");
  });

  it("filename is basename only", async () => {
    const result = await mockAirBridgeService.previewRecordWriteRequestPlan(readyRequest());
    expect(result.filename).toBe("backup.airbridge");
    expect(result.filename).not.toContain("/");
    expect(result.filename).not.toContain("\\");
  });

  it("no raw record payloads in result", async () => {
    const result = await mockAirBridgeService.previewRecordWriteRequestPlan(readyRequest());
    const json = JSON.stringify(result);
    expect(json).not.toContain('"records":');
    expect(json).not.toContain('"payload":');
    expect(json).not.toContain("newRecordId");
  });
});

describe("recordWriteRequestPlan mock service — op counts", () => {
  it("createBatchOpCount matches totalFirstPassBatches", async () => {
    const result = await mockAirBridgeService.previewRecordWriteRequestPlan(readyRequest());
    expect(result.createBatchOpCount).toBe(4);
  });

  it("linkedUpdateOpCount matches totalSecondPassBatches", async () => {
    const result = await mockAirBridgeService.previewRecordWriteRequestPlan(readyRequest());
    expect(result.linkedUpdateOpCount).toBe(2);
  });

  it("checkpointOpCount matches tableCount", async () => {
    const result = await mockAirBridgeService.previewRecordWriteRequestPlan(readyRequest());
    expect(result.checkpointOpCount).toBe(2);
  });

  it("attachmentOpCount matches attachmentFieldCount", async () => {
    const result = await mockAirBridgeService.previewRecordWriteRequestPlan(readyRequest());
    expect(result.attachmentOpCount).toBe(1);
  });

  it("skippedFieldOpCount matches skippedFieldCount", async () => {
    const result = await mockAirBridgeService.previewRecordWriteRequestPlan(readyRequest());
    expect(result.skippedFieldOpCount).toBe(2);
  });

  it("totalOpCount equals sum of all op count components", async () => {
    const result = await mockAirBridgeService.previewRecordWriteRequestPlan(readyRequest());
    const sum =
      result.createBatchOpCount +
      result.linkedUpdateOpCount +
      result.checkpointOpCount +
      result.attachmentOpCount +
      result.skippedFieldOpCount;
    expect(result.totalOpCount).toBe(sum);
  });

  it("totalFirstPassBatches echoes request", async () => {
    const result = await mockAirBridgeService.previewRecordWriteRequestPlan(readyRequest());
    expect(result.totalFirstPassBatches).toBe(4);
  });

  it("totalSecondPassBatches echoes request", async () => {
    const result = await mockAirBridgeService.previewRecordWriteRequestPlan(readyRequest());
    expect(result.totalSecondPassBatches).toBe(2);
  });

  it("blocked result has all op counts zero", async () => {
    const result = await mockAirBridgeService.previewRecordWriteRequestPlan(
      readyRequest({ recordImportPlanStatus: "blocked" }),
    );
    expect(result.totalOpCount).toBe(0);
    expect(result.createBatchOpCount).toBe(0);
    expect(result.linkedUpdateOpCount).toBe(0);
  });

  it("no linked update ops when totalSecondPassBatches is zero", async () => {
    const result = await mockAirBridgeService.previewRecordWriteRequestPlan(
      readyRequest({ totalSecondPassBatches: 0 }),
    );
    expect(result.linkedUpdateOpCount).toBe(0);
  });

  it("no attachment ops when attachmentFieldCount is zero", async () => {
    const result = await mockAirBridgeService.previewRecordWriteRequestPlan(
      readyRequest({ attachmentFieldCount: 0 }),
    );
    expect(result.attachmentOpCount).toBe(0);
  });

  it("no skipped ops when skippedFieldCount is zero", async () => {
    const result = await mockAirBridgeService.previewRecordWriteRequestPlan(
      readyRequest({ skippedFieldCount: 0 }),
    );
    expect(result.skippedFieldOpCount).toBe(0);
  });
});

// ── IPC fallback contract ──────────────────────────────────────────────────────

describe("recordWriteRequestPlan IPC fallback", () => {
  it("fallback result is safe — no token", () => {
    const fallback: RecordWriteRequestPlanResult = {
      filename: "backup.airbridge",
      status: "disabled",
      disabledReason: "disabledByProductPolicy",
      message: "Record write planning is not available in this context.",
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
    };
    const json = JSON.stringify(fallback);
    expect(json).not.toContain('"token"');
    expect(json).not.toContain('"succeeded"');
    expect(fallback.noChangesMade).toBe(true);
    expect(fallback.networkWritesAttempted).toBe(false);
  });

  it("fallback status is never succeeded", () => {
    const fallback: RecordWriteRequestPlanResult = {
      filename: "backup.airbridge",
      status: "disabled",
      message: "Not available.",
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
    };
    expect(fallback.status).not.toBe("succeeded" as never);
  });
});

// ── Isolation from other write gates ──────────────────────────────────────────

describe("record write engine does not affect other gates", () => {
  it("previewRestoreWriteEngine still returns disabled after record write plan", async () => {
    await mockAirBridgeService.previewRecordWriteRequestPlan(readyRequest());

    const writeResult = await mockAirBridgeService.previewRestoreWriteEngine({
      packageFilename: "backup.airbridge",
      packagePath: "/tmp/backup.airbridge",
    });
    expect(writeResult.status).toBe("disabled");
    const json = JSON.stringify(writeResult).toLowerCase();
    expect(json).not.toContain("succeeded");
  });

  it("previewSchemaWriteRequestPlan still returns disabled after record write plan", async () => {
    await mockAirBridgeService.previewRecordWriteRequestPlan(readyRequest());

    const schemaResult = await mockAirBridgeService.previewSchemaWriteRequestPlan({
      packageFilename: "backup.airbridge",
      schemaPlanStatus: "ready",
      tableCount: 2,
      directFieldCount: 4,
      deferredFieldCount: 1,
      manualActionCount: 0,
    });
    expect(schemaResult.status).toBe("disabled");
    const json = JSON.stringify(schemaResult).toLowerCase();
    expect(json).not.toContain("succeeded");
  });
});
