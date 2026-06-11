export type ConnectionId = string;

export type ConnectionStatus = "disconnected" | "checking" | "connected" | "failed";

export type PermissionCheckStatus = "unknown" | "checking" | "passed" | "failed";

export interface PermissionCheck {
  key: string;
  label: string;
  status: PermissionCheckStatus;
  detail?: string;
}

export interface AirtableConnectionProfile {
  id: ConnectionId;
  label: string;
  status: ConnectionStatus;
  connectedAt?: string; // ISO date string
  failureMessage?: string;
  permissions: PermissionCheck[];
}
