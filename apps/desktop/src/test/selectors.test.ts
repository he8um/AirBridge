import { describe, it, expect } from "vitest";
import { MOCK_STATE } from "../state/mockState";
import {
  getConnectedProfiles,
  getRecentBackupPackages,
  getRecentReports,
  getActiveJobs,
  getPermissionSummary,
  getCompatibilitySummary,
  getLogsByLevel,
  getDashboardStats,
} from "../state/selectors";
import type { AppState } from "../state/appState";

describe("selectors", () => {
  describe("getConnectedProfiles", () => {
    it("returns only connections with status connected", () => {
      const profiles = getConnectedProfiles(MOCK_STATE);
      expect(profiles.length).toBe(1);
      expect(profiles[0].id).toBe("conn-002");
    });

    it("returns empty array when no connections are connected", () => {
      const state: AppState = { ...MOCK_STATE, connections: [] };
      expect(getConnectedProfiles(state)).toEqual([]);
    });
  });

  describe("getRecentBackupPackages", () => {
    it("returns packages sorted by createdAt descending", () => {
      const packages = getRecentBackupPackages(MOCK_STATE);
      expect(packages.length).toBe(3);
      expect(packages[0].id).toBe("pkg-001");
      expect(packages[1].id).toBe("pkg-002");
      expect(packages[2].id).toBe("pkg-003");
    });

    it("respects the limit parameter", () => {
      const packages = getRecentBackupPackages(MOCK_STATE, 2);
      expect(packages.length).toBe(2);
      expect(packages[0].id).toBe("pkg-001");
    });

    it("does not mutate the original state array", () => {
      const before = MOCK_STATE.backupPackages.map((p) => p.id);
      getRecentBackupPackages(MOCK_STATE);
      const after = MOCK_STATE.backupPackages.map((p) => p.id);
      expect(after).toEqual(before);
    });
  });

  describe("getRecentReports", () => {
    it("returns reports sorted by createdAt descending", () => {
      const reports = getRecentReports(MOCK_STATE);
      expect(reports.length).toBe(3);
      const timestamps = reports.map((r) => new Date(r.createdAt).getTime());
      expect(timestamps[0]).toBeGreaterThanOrEqual(timestamps[1]);
      expect(timestamps[1]).toBeGreaterThanOrEqual(timestamps[2]);
    });

    it("respects the limit parameter", () => {
      const reports = getRecentReports(MOCK_STATE, 1);
      expect(reports.length).toBe(1);
    });
  });

  describe("getActiveJobs", () => {
    it("returns empty array when no jobs are running or queued", () => {
      const jobs = getActiveJobs(MOCK_STATE);
      expect(jobs).toEqual([]);
    });

    it("returns running backup jobs as JobSummary with type backup", () => {
      const runningState: AppState = {
        ...MOCK_STATE,
        backupJobs: [
          {
            ...MOCK_STATE.backupJobs[0],
            status: "running",
          },
        ],
      };
      const jobs = getActiveJobs(runningState);
      expect(jobs.length).toBe(1);
      expect(jobs[0].type).toBe("backup");
      expect(jobs[0].status).toBe("running");
    });
  });

  describe("getPermissionSummary", () => {
    it("counts passed, failed, and unknown permissions across all connections", () => {
      const summary = getPermissionSummary(MOCK_STATE);
      expect(summary.passed).toBe(2);
      expect(summary.failed).toBe(2);
      expect(summary.unknown).toBe(4);
      expect(summary.total).toBe(8);
    });

    it("returns all zeros for empty connections", () => {
      const state: AppState = { ...MOCK_STATE, connections: [] };
      const summary = getPermissionSummary(state);
      expect(summary).toEqual({ passed: 0, failed: 0, unknown: 0, total: 0 });
    });
  });

  describe("getCompatibilitySummary", () => {
    it("counts compatibility rules by support level", () => {
      const summary = getCompatibilitySummary(MOCK_STATE);
      expect(summary.totalRules).toBe(8);
      expect(summary.bySupport.restorable).toBe(3);
      expect(summary.bySupport.partially_restorable).toBe(2);
      expect(summary.bySupport.metadata_only).toBe(2);
      expect(summary.bySupport.unsupported_for_restore).toBe(1);
      expect(summary.bySupport.manual_action_required).toBe(0);
    });
  });

  describe("getLogsByLevel", () => {
    it("returns only entries matching the given level", () => {
      const warnings = getLogsByLevel(MOCK_STATE, "warning");
      expect(warnings.length).toBe(1);
      expect(warnings[0].level).toBe("warning");
    });

    it("returns debug entries correctly", () => {
      const debugLogs = getLogsByLevel(MOCK_STATE, "debug");
      expect(debugLogs.length).toBe(1);
      expect(debugLogs[0].message).toContain("Initializing");
    });

    it("returns info entries correctly", () => {
      const infoLogs = getLogsByLevel(MOCK_STATE, "info");
      expect(infoLogs.length).toBe(4);
    });
  });

  describe("getDashboardStats", () => {
    it("returns correct stats based on mock state", () => {
      const stats = getDashboardStats(MOCK_STATE);
      expect(stats.connectedBases).toBe(2);
      expect(stats.recentBackups).toBe(2);
      expect(stats.restoreJobs).toBe(1);
    });

    it("returns zero connected bases when no connections are active", () => {
      const state: AppState = {
        ...MOCK_STATE,
        connections: MOCK_STATE.connections.map((c) => ({ ...c, status: "disconnected" as const })),
      };
      const stats = getDashboardStats(state);
      expect(stats.connectedBases).toBe(0);
    });
  });
});
