import { SectionHeader } from "../components/SectionHeader";
import { StatusBadge } from "../components/StatusBadge";

const PERMISSIONS = ["Schema read", "Records read", "Schema write", "Records write"] as const;

export function ConnectionsPage() {
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
                <StatusBadge status="idle" label="Not connected" />
              </div>

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
                  {PERMISSIONS.map((perm) => (
                    <div key={perm} className="check-list-row" role="listitem">
                      <span className="check-list-label">{perm}</span>
                      <span className="check-list-status" aria-label={`${perm}: not verified`}>
                        —
                      </span>
                    </div>
                  ))}
                </div>
              </div>
            </div>
          </div>
        </section>
      </div>
    </div>
  );
}
