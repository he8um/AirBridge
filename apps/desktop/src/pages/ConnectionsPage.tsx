import { SectionHeader } from "../components/SectionHeader";
import { StatusBadge } from "../components/StatusBadge";
import { useAppState } from "../state/useAppState";
import type { PermissionCheckStatus } from "../domain/connection";

function permissionStatusLabel(status: PermissionCheckStatus): string {
  switch (status) {
    case "passed":
      return "Passed";
    case "failed":
      return "Failed";
    case "checking":
      return "Checking…";
    default:
      return "—";
  }
}

function permissionStatusBadge(
  status: PermissionCheckStatus,
): "connected" | "error" | "warning" | "idle" {
  switch (status) {
    case "passed":
      return "connected";
    case "failed":
      return "error";
    case "checking":
      return "warning";
    default:
      return "idle";
  }
}

export function ConnectionsPage() {
  const { state } = useAppState();
  const selected =
    state.connections.find((c) => c.id === state.selectedConnectionId) ?? state.connections[0];

  return (
    <div className="page">
      <div className="page-content">
        {/* Connection card */}
        <section aria-labelledby="connection-heading">
          <SectionHeader title="Airtable Connection" />

          <div className="card" style={{ maxWidth: 560 }}>
            <div style={{ display: "flex", flexDirection: "column", gap: "var(--space-5)" }}>
              {/* Token field */}
              <div className="form-field">
                <label htmlFor="pat-input" className="form-label">
                  Personal Access Token
                </label>
                <div style={{ display: "flex", gap: "var(--space-2)" }}>
                  <input
                    id="pat-input"
                    type="password"
                    className="form-input"
                    placeholder="pat_xxxxxxxxxxxxxxxxxxxx"
                    disabled
                    aria-label="Personal access token"
                    autoComplete="off"
                  />
                  <button
                    type="button"
                    className="btn btn-primary"
                    disabled
                    aria-label="Connect with provided token"
                    style={{ flexShrink: 0 }}
                  >
                    Connect
                  </button>
                </div>
                <p style={{ fontSize: "var(--text-xs)", color: "var(--color-text-muted)" }}>
                  Token is stored locally and never transmitted.
                </p>
              </div>

              {/* Connection status */}
              <div style={{ display: "flex", alignItems: "center", gap: "var(--space-3)" }}>
                <span style={{ fontSize: "var(--text-sm)", color: "var(--color-text-muted)" }}>
                  Status:
                </span>
                {selected ? (
                  <StatusBadge
                    status={
                      selected.status === "connected"
                        ? "connected"
                        : selected.status === "failed"
                          ? "error"
                          : selected.status === "checking"
                            ? "warning"
                            : "idle"
                    }
                    label={
                      selected.status === "connected"
                        ? "Connected"
                        : selected.status === "failed"
                          ? "Failed"
                          : selected.status === "checking"
                            ? "Checking…"
                            : "Not connected"
                    }
                  />
                ) : (
                  <StatusBadge status="idle" label="Not connected" />
                )}
              </div>

              {selected?.connectedAt && (
                <p
                  style={{
                    fontSize: "var(--text-xs)",
                    color: "var(--color-text-muted)",
                    margin: 0,
                  }}
                >
                  Connected since{" "}
                  {new Date(selected.connectedAt).toLocaleDateString(undefined, {
                    year: "numeric",
                    month: "short",
                    day: "numeric",
                  })}
                </p>
              )}

              <div className="divider" style={{ margin: 0 }} />

              {/* Permission list */}
              <div>
                <p
                  style={{
                    fontSize: "var(--text-xs)",
                    fontWeight: 600,
                    color: "var(--color-text-muted)",
                    textTransform: "uppercase",
                    letterSpacing: "0.06em",
                    marginBottom: "var(--space-2)",
                  }}
                >
                  Required Permissions
                </p>
                <div className="check-list" role="list" aria-label="Required permissions">
                  {selected?.permissions.map((perm) => (
                    <div key={perm.key} className="check-list-row" role="listitem">
                      <span className="check-list-label">{perm.label}</span>
                      <StatusBadge
                        status={permissionStatusBadge(perm.status)}
                        label={permissionStatusLabel(perm.status)}
                      />
                    </div>
                  )) ?? (
                    <>
                      {(
                        ["Schema read", "Records read", "Schema write", "Records write"] as const
                      ).map((label) => (
                        <div key={label} className="check-list-row" role="listitem">
                          <span className="check-list-label">{label}</span>
                          <span className="check-list-status" aria-label={`${label}: not verified`}>
                            —
                          </span>
                        </div>
                      ))}
                    </>
                  )}
                </div>
              </div>
            </div>
          </div>
        </section>
      </div>
    </div>
  );
}
