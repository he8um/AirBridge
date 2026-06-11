import { SectionHeader } from "../components/SectionHeader";
import { StatusBadge } from "../components/StatusBadge";
import { useAppState } from "../state/useAppState";
import { ConnectionForm, PermissionCheckList } from "../features/connections";
import type { PermissionCheckStatus } from "../domain/connection";

function connectionStatusBadge(status: string): "connected" | "error" | "warning" | "idle" {
  switch (status) {
    case "connected":
      return "connected";
    case "failed":
      return "error";
    case "checking":
      return "warning";
    default:
      return "idle";
  }
}

function connectionStatusLabel(status: string): string {
  switch (status) {
    case "connected":
      return "Connected";
    case "failed":
      return "Failed";
    case "checking":
      return "Checking…";
    default:
      return "Not connected";
  }
}

function permissionChecksFromProfile(
  permissions: Array<{
    key: string;
    label: string;
    status: PermissionCheckStatus;
    detail?: string;
  }>,
) {
  return permissions;
}

export function ConnectionsPage() {
  const { state } = useAppState();

  return (
    <div className="page">
      <div className="page-content">
        {/* New connection form */}
        <section aria-labelledby="new-connection-heading">
          <SectionHeader title="Add Connection" />
          <div className="card" style={{ maxWidth: 560 }}>
            <ConnectionForm />
          </div>
        </section>

        {/* Saved connections from state */}
        <section aria-labelledby="saved-connections-heading">
          <SectionHeader title="Saved Connections" />

          {state.connections.length === 0 ? (
            <div className="card notice-neutral" style={{ maxWidth: 560 }}>
              <p style={{ fontSize: "var(--text-sm)", color: "var(--color-text-muted)" }}>
                No saved connections.
              </p>
            </div>
          ) : (
            <div style={{ display: "flex", flexDirection: "column", gap: "var(--space-4)" }}>
              {state.connections.map((conn) => (
                <div key={conn.id} className="card" style={{ maxWidth: 560 }}>
                  <div style={{ display: "flex", flexDirection: "column", gap: "var(--space-4)" }}>
                    <div
                      style={{
                        display: "flex",
                        alignItems: "center",
                        justifyContent: "space-between",
                      }}
                    >
                      <span style={{ fontSize: "var(--text-sm)", fontWeight: 600 }}>
                        {conn.label}
                      </span>
                      <StatusBadge
                        status={connectionStatusBadge(conn.status)}
                        label={connectionStatusLabel(conn.status)}
                      />
                    </div>

                    {conn.connectedAt && (
                      <p
                        style={{
                          fontSize: "var(--text-xs)",
                          color: "var(--color-text-muted)",
                          margin: 0,
                        }}
                      >
                        Connected since{" "}
                        {new Date(conn.connectedAt).toLocaleDateString(undefined, {
                          year: "numeric",
                          month: "short",
                          day: "numeric",
                        })}
                      </p>
                    )}

                    {conn.permissions.length > 0 && (
                      <>
                        <div className="divider" style={{ margin: 0 }} />
                        <PermissionCheckList
                          checks={permissionChecksFromProfile(conn.permissions)}
                          title="Required Permissions"
                        />
                      </>
                    )}
                  </div>
                </div>
              ))}
            </div>
          )}
        </section>
      </div>
    </div>
  );
}
