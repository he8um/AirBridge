// AppHealth returned by get_app_health command
export interface AppHealthResponse {
  appName: string;
  version: string;
  status: string;
  backend: string;
}

// ConnectionCheckResult from check_connection command
export interface ConnectionCheckResult {
  connectionId: string;
  status: "disconnected" | "checking" | "connected" | "failed";
  permissions: Array<{
    key: string;
    label: string;
    status: "unknown" | "checking" | "passed" | "failed";
    detail?: string;
  }>;
  /** Bases visible to the token, populated on successful connection check. */
  accessibleBases?: Array<{
    id: string;
    name: string;
  }>;
}

// AirBridgeError structure returned on command failure
export interface AirBridgeCommandError {
  code: string;
  message: string;
}

// Catalog and schema summary types from list_accessible_bases / get_base_schema commands

export interface AccessibleBaseSummary {
  id: string;
  name: string;
}

export interface FieldTypeCount {
  fieldType: string;
  count: number;
}

export interface SchemaCompatibilitySummary {
  restorableCount: number;
  metadataOnlyCount: number;
  unknownCount: number;
  totalCount: number;
}

export interface TableSchemaSummary {
  id: string;
  name: string;
  fieldCount: number;
  fieldTypeCounts: FieldTypeCount[];
  compatibility: SchemaCompatibilitySummary;
}

export interface BaseSchemaSummary {
  baseId: string;
  tableCount: number;
  tables: TableSchemaSummary[];
  compatibility: SchemaCompatibilitySummary;
}
