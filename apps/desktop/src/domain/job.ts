export type JobId = string;

export type JobType = "backup" | "restore" | "validation" | "connection_check";

export type JobStatus = "idle" | "queued" | "running" | "succeeded" | "failed" | "cancelled";

export interface JobProgress {
  current: number;
  total: number;
  label?: string;
}

export interface JobSummary {
  id: JobId;
  type: JobType;
  status: JobStatus;
  startedAt?: string;
  completedAt?: string;
  progress?: JobProgress;
  errorMessage?: string;
}
