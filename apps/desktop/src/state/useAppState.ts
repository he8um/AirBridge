import { useState } from "react";
import type { AppState } from "./appState";
import { MOCK_STATE } from "./mockState";
import type { LogLevel } from "../domain/log";
import {
  getConnectedProfiles,
  getRecentBackupPackages,
  getRecentReports,
  getActiveJobs,
  getPermissionSummary,
  getCompatibilitySummary,
  getLogsByLevel,
  getDashboardStats,
} from "./selectors";

export function useAppState() {
  const [state, setState] = useState<AppState>(MOCK_STATE);

  return {
    state,
    setState,
    connectedProfiles: getConnectedProfiles(state),
    recentBackups: getRecentBackupPackages(state),
    recentReports: getRecentReports(state),
    activeJobs: getActiveJobs(state),
    permissionSummary: getPermissionSummary(state),
    compatibilitySummary: getCompatibilitySummary(state),
    dashboardStats: getDashboardStats(state),
    getLogsByLevel: (level: LogLevel) => getLogsByLevel(state, level),
  };
}
