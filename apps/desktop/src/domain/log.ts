export type LogLevel = "debug" | "info" | "warning" | "error";

export interface JobLogEntry {
  id: string;
  timestamp: string; // ISO
  level: LogLevel;
  jobId?: string;
  jobType?: string;
  message: string;
  detail?: string;
}
