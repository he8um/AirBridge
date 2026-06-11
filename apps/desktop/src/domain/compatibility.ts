import type { AirtableFieldType } from "./airtable";

export type FieldRestoreSupport =
  | "restorable"
  | "partially_restorable"
  | "metadata_only"
  | "unsupported_for_restore"
  | "manual_action_required";

export interface FieldCompatibilityRule {
  fieldType: AirtableFieldType;
  support: FieldRestoreSupport;
  note: string;
  backupSupport: "full" | "partial" | "metadata_only" | "none";
}

export interface CompatibilitySummary {
  totalFieldTypes: number;
  bySupport: Record<FieldRestoreSupport, number>;
}
