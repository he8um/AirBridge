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

  it("mock service satisfies AirBridgeService interface", () => {
    const _: AirBridgeService = mockAirBridgeService;
    expect(_).toBeDefined();
  });
});
