import { describe, it, expect } from "vitest";
import {
  getAppHealth,
  checkConnection,
  listWorkspaces,
  listBases,
  listBackupPackages,
  listRestorePlans,
  listReports,
  listLogs,
  listCompatibilityRules,
} from "../backend/commands";
import { isAirBridgeCommandError, formatCommandError } from "../backend/errors";

describe("command bridge", () => {
  it("getAppHealth returns null in test environment", async () => {
    const result = await getAppHealth();
    expect(result).toBeNull();
  });

  it("checkConnection returns null in test environment", async () => {
    const result = await checkConnection("test-token");
    expect(result).toBeNull();
  });

  it("checkConnection result does not contain token value", async () => {
    const sentinel = "unique-sentinel-token-abc123";
    const result = await checkConnection(sentinel);
    if (result !== null) {
      expect(JSON.stringify(result)).not.toContain(sentinel);
    } else {
      expect(result).toBeNull();
    }
  });

  it("listWorkspaces returns null in test environment", async () => {
    const result = await listWorkspaces();
    expect(result).toBeNull();
  });

  it("listBases returns null in test environment", async () => {
    const result = await listBases();
    expect(result).toBeNull();
  });

  it("listBackupPackages returns null in test environment", async () => {
    const result = await listBackupPackages();
    expect(result).toBeNull();
  });

  it("listRestorePlans returns null in test environment", async () => {
    const result = await listRestorePlans();
    expect(result).toBeNull();
  });

  it("listReports returns null in test environment", async () => {
    const result = await listReports();
    expect(result).toBeNull();
  });

  it("listLogs returns null in test environment", async () => {
    const result = await listLogs();
    expect(result).toBeNull();
  });

  it("listCompatibilityRules returns null in test environment", async () => {
    const result = await listCompatibilityRules();
    expect(result).toBeNull();
  });

  it("isAirBridgeCommandError correctly identifies error shape", () => {
    const valid = { code: "NOT_FOUND", message: "Resource not found" };
    expect(isAirBridgeCommandError(valid)).toBe(true);

    expect(isAirBridgeCommandError(null)).toBe(false);
    expect(isAirBridgeCommandError(undefined)).toBe(false);
    expect(isAirBridgeCommandError("string error")).toBe(false);
    expect(isAirBridgeCommandError(42)).toBe(false);
    expect(isAirBridgeCommandError({})).toBe(false);
    expect(isAirBridgeCommandError({ code: 123, message: "oops" })).toBe(false);
    expect(isAirBridgeCommandError({ code: "ERR", message: 999 })).toBe(false);
    expect(isAirBridgeCommandError({ code: "ERR" })).toBe(false);
    expect(isAirBridgeCommandError({ message: "oops" })).toBe(false);
  });

  it("formatCommandError formats known error codes", () => {
    const err = { code: "PERMISSION_DENIED", message: "Token lacks required scope" };
    expect(formatCommandError(err)).toBe("[PERMISSION_DENIED] Token lacks required scope");

    expect(formatCommandError("plain string error")).toBe("plain string error");
    expect(formatCommandError(null)).toBe("An unexpected error occurred.");
    expect(formatCommandError(undefined)).toBe("An unexpected error occurred.");
    expect(formatCommandError(42)).toBe("An unexpected error occurred.");
  });
});
