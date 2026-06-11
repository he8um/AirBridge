import type { AirtableConnectionProfile } from "../domain/connection";
import type { AirtableWorkspace, AirtableBaseSummary } from "../domain/airtable";
import type { BackupPackageSummary, BackupJobSummary } from "../domain/backup";
import type { RestorePlanSummary, RestoreJobSummary } from "../domain/restore";
import type { ReportSummary } from "../domain/report";
import type { JobLogEntry } from "../domain/log";
import type { FieldCompatibilityRule } from "../domain/compatibility";

export interface AppState {
  connections: AirtableConnectionProfile[];
  workspaces: AirtableWorkspace[];
  bases: AirtableBaseSummary[];
  backupPackages: BackupPackageSummary[];
  backupJobs: BackupJobSummary[];
  restorePlans: RestorePlanSummary[];
  restoreJobs: RestoreJobSummary[];
  reports: ReportSummary[];
  logs: JobLogEntry[];
  compatibilityRules: FieldCompatibilityRule[];
  selectedConnectionId: string | null;
  selectedBaseId: string | null;
}
