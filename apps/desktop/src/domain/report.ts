export type ReportId = string;

export type ReportType = "backup" | "restore" | "validation" | "compatibility";

export type ReportSeverity = "info" | "warning" | "error";

export interface ReportItem {
  id: string;
  severity: ReportSeverity;
  title: string;
  detail?: string;
  fieldName?: string;
  tableName?: string;
}

export interface ReportSummary {
  id: ReportId;
  type: ReportType;
  title: string;
  createdAt: string; // ISO
  severity: ReportSeverity;
  itemCount: number;
  items: ReportItem[];
  relatedJobId?: string;
  relatedBaseId?: string;
  relatedBaseName?: string;
}
