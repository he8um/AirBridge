import { describe, it, expect } from "vitest";
import { mockAirBridgeService } from "../services/mockAirBridgeService";
import type { AirBridgeService } from "../services/airBridgeService";

describe("mockAirBridgeService", () => {
  it("listConnections returns connection profiles", async () => {
    const result = await mockAirBridgeService.listConnections();
    expect(Array.isArray(result)).toBe(true);
    expect(result.length).toBeGreaterThanOrEqual(1);
  });

  it("listWorkspaces returns workspaces", async () => {
    const result = await mockAirBridgeService.listWorkspaces();
    expect(Array.isArray(result)).toBe(true);
    expect(result.length).toBeGreaterThanOrEqual(1);
  });

  it("listBases returns base summaries", async () => {
    const result = await mockAirBridgeService.listBases();
    expect(Array.isArray(result)).toBe(true);
    expect(result.length).toBeGreaterThanOrEqual(1);
  });

  it("listBackupPackages returns packages", async () => {
    const result = await mockAirBridgeService.listBackupPackages();
    expect(Array.isArray(result)).toBe(true);
    expect(result.length).toBeGreaterThanOrEqual(1);
  });

  it("listRestorePlans returns plans", async () => {
    const result = await mockAirBridgeService.listRestorePlans();
    expect(Array.isArray(result)).toBe(true);
    expect(result.length).toBeGreaterThanOrEqual(1);
    expect(result[0].id).toBe("plan-001");
  });

  it("listReports returns reports", async () => {
    const result = await mockAirBridgeService.listReports();
    expect(Array.isArray(result)).toBe(true);
    expect(result.length).toBeGreaterThanOrEqual(1);
  });

  it("listLogs returns log entries", async () => {
    const result = await mockAirBridgeService.listLogs();
    expect(Array.isArray(result)).toBe(true);
    expect(result.length).toBeGreaterThanOrEqual(1);
  });

  it("listCompatibilityRules returns rules", async () => {
    const result = await mockAirBridgeService.listCompatibilityRules();
    expect(Array.isArray(result)).toBe(true);
    expect(result.length).toBeGreaterThanOrEqual(1);
  });

  it("listCompatibilityRules includes a restorable rule", async () => {
    const result = await mockAirBridgeService.listCompatibilityRules();
    const hasRestorable = result.some((rule) => rule.support === "restorable");
    expect(hasRestorable).toBe(true);
  });

  it("checkConnection exists on mock service and returns a result", async () => {
    const result = await mockAirBridgeService.checkConnection({
      token: "example_valid_token_abcdefgh12345",
    });
    expect(result).toBeDefined();
    expect(result.connectionId).toBeDefined();
    expect(result.status).toBeDefined();
    expect(JSON.stringify(result)).not.toContain("example_valid_token_abcdefgh12345");
  });

  it("checkConnection write permissions are not marked passed on success", async () => {
    const result = await mockAirBridgeService.checkConnection({
      token: "example_valid_token_abcdefgh12345",
    });
    const writePerms = result.permissions.filter(
      (p) => p.key === "schema.bases:write" || p.key === "data.records:write",
    );
    expect(writePerms.length).toBe(2);
    for (const p of writePerms) {
      expect(p.status).not.toBe("passed");
    }
  });

  it("mock service satisfies AirBridgeService interface", () => {
    const _: AirBridgeService = mockAirBridgeService;
    expect(_).toBeDefined();
  });

  // ── listAccessibleBases ─────────────────────────────────────────────────

  it("listAccessibleBases returns an array of base summaries", async () => {
    const result = await mockAirBridgeService.listAccessibleBases({
      token: "example_valid_token_abcdefgh12345",
    });
    expect(Array.isArray(result)).toBe(true);
    expect(result.length).toBeGreaterThanOrEqual(1);
  });

  it("listAccessibleBases result has id and name on each entry", async () => {
    const result = await mockAirBridgeService.listAccessibleBases({
      token: "example_valid_token_abcdefgh12345",
    });
    for (const base of result) {
      expect(typeof base.id).toBe("string");
      expect(typeof base.name).toBe("string");
      expect(base.id.length).toBeGreaterThan(0);
    }
  });

  it("listAccessibleBases result does not contain token", async () => {
    const token = "example_valid_token_abcdefgh12345";
    const result = await mockAirBridgeService.listAccessibleBases({ token });
    expect(JSON.stringify(result)).not.toContain(token);
  });

  // ── getBaseSchema ────────────────────────────────────────────────────────

  it("getBaseSchema returns a BaseSchemaSummary", async () => {
    const result = await mockAirBridgeService.getBaseSchema({
      token: "example_valid_token_abcdefgh12345",
      baseId: "appExampleBase01",
    });
    expect(result).toBeDefined();
    expect(typeof result.baseId).toBe("string");
    expect(typeof result.tableCount).toBe("number");
    expect(Array.isArray(result.tables)).toBe(true);
    expect(result.compatibility).toBeDefined();
  });

  it("getBaseSchema result has correct base id", async () => {
    const result = await mockAirBridgeService.getBaseSchema({
      token: "example_valid_token_abcdefgh12345",
      baseId: "appExampleBase01",
    });
    expect(result.baseId).toBe("appExampleBase01");
  });

  it("getBaseSchema compatibility counts are non-negative", async () => {
    const result = await mockAirBridgeService.getBaseSchema({
      token: "example_valid_token_abcdefgh12345",
      baseId: "appExampleBase01",
    });
    expect(result.compatibility.restorableCount).toBeGreaterThanOrEqual(0);
    expect(result.compatibility.metadataOnlyCount).toBeGreaterThanOrEqual(0);
    expect(result.compatibility.unknownCount).toBeGreaterThanOrEqual(0);
    expect(result.compatibility.totalCount).toBeGreaterThanOrEqual(0);
  });

  it("getBaseSchema totalCount equals sum of all table field counts", async () => {
    const result = await mockAirBridgeService.getBaseSchema({
      token: "example_valid_token_abcdefgh12345",
      baseId: "appExampleBase01",
    });
    const sumFromTables = result.tables.reduce((sum, t) => sum + t.fieldCount, 0);
    expect(result.compatibility.totalCount).toBe(sumFromTables);
  });

  it("getBaseSchema result does not contain token", async () => {
    const token = "example_valid_token_abcdefgh12345";
    const result = await mockAirBridgeService.getBaseSchema({
      token,
      baseId: "appExampleBase01",
    });
    expect(JSON.stringify(result)).not.toContain(token);
  });

  it("checkConnection returns accessibleBases on success", async () => {
    const result = await mockAirBridgeService.checkConnection({
      token: "example_valid_token_abcdefgh12345",
    });
    expect(result.status).toBe("connected");
    expect(Array.isArray(result.accessibleBases)).toBe(true);
    expect((result.accessibleBases ?? []).length).toBeGreaterThanOrEqual(1);
  });

  it("checkConnection accessibleBases does not contain token", async () => {
    const token = "example_valid_token_abcdefgh12345";
    const result = await mockAirBridgeService.checkConnection({ token });
    expect(JSON.stringify(result.accessibleBases)).not.toContain(token);
  });
});
