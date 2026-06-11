export type AirtableWorkspaceId = string;
export type AirtableBaseId = string;
export type AirtableTableId = string;
export type AirtableFieldId = string;

export type AirtableFieldType =
  | "singleLineText"
  | "multilineText"
  | "number"
  | "currency"
  | "percent"
  | "checkbox"
  | "singleSelect"
  | "multipleSelects"
  | "date"
  | "dateTime"
  | "email"
  | "url"
  | "phoneNumber"
  | "rating"
  | "duration"
  | "multipleRecordLinks"
  | "formula"
  | "rollup"
  | "count"
  | "lookup"
  | "createdTime"
  | "lastModifiedTime"
  | "createdBy"
  | "lastModifiedBy"
  | "multipleAttachments"
  | "barcode"
  | "externalSyncSource"
  | "autoNumber"
  | "button"
  | "aiText"; // note: unsupported for restore in most scenarios

export interface AirtableFieldSummary {
  id: AirtableFieldId;
  name: string;
  type: AirtableFieldType;
  primary?: boolean;
}

export interface AirtableTableSummary {
  id: AirtableTableId;
  name: string;
  fieldCount: number;
  recordCount?: number;
  fields?: AirtableFieldSummary[];
}

export interface AirtableBaseSummary {
  id: AirtableBaseId;
  workspaceId: AirtableWorkspaceId;
  name: string;
  tableCount: number;
  tables?: AirtableTableSummary[];
}

export interface AirtableWorkspace {
  id: AirtableWorkspaceId;
  name: string;
  bases: AirtableBaseSummary[];
}
