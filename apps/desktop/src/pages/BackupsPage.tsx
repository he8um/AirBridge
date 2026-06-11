import { SectionHeader } from "../components/SectionHeader";
import { EmptyState } from "../components/EmptyState";

const SCOPE_OPTIONS = [
  { value: "full", label: "Full backup", description: "Schema and all records" },
  { value: "schema", label: "Schema only", description: "Table structure, no records" },
  { value: "records", label: "Records only", description: "Records without schema" },
] as const;

// Empty inbox icon path
const EMPTY_ICON =
  "M20 13V6a2 2 0 00-2-2H6a2 2 0 00-2 2v7m16 0v5a2 2 0 01-2 2H6a2 2 0 01-2-2v-5m16 0h-2.586a1 1 0 00-.707.293l-2.414 2.414a1 1 0 01-.707.293h-3.172a1 1 0 01-.707-.293l-2.414-2.414A1 1 0 006.586 13H4";

export function BackupsPage() {
  return (
    <div className="page">
      <div className="page-content">
        {/* New Backup section */}
        <section aria-labelledby="new-backup-heading">
          <SectionHeader title="New Backup" />

          <div className="card" style={{ maxWidth: 560 }}>
            <div style={{ display: "flex", flexDirection: "column", gap: "var(--space-5)" }}>
              {/* Select Base */}
              <div className="form-field">
                <label htmlFor="base-select" className="form-label">
                  Select Base
                </label>
                <select
                  id="base-select"
                  className="form-input"
                  disabled
                  aria-label="Select Airtable base to back up"
                >
                  <option value="">No bases connected</option>
                </select>
              </div>

              {/* Backup Scope */}
              <fieldset style={{ border: "none", padding: 0 }}>
                <legend className="form-label" style={{ marginBottom: "var(--space-3)" }}>
                  Backup Scope
                </legend>
                <div style={{ display: "flex", flexDirection: "column", gap: "var(--space-2)" }}>
                  {SCOPE_OPTIONS.map((opt, idx) => (
                    <label
                      key={opt.value}
                      style={{
                        display: "flex",
                        alignItems: "center",
                        gap: "var(--space-3)",
                        padding: "var(--space-3)",
                        borderRadius: "var(--radius-md)",
                        border: `1px solid ${idx === 0 ? "var(--color-accent)" : "var(--color-border)"}`,
                        backgroundColor:
                          idx === 0 ? "var(--color-accent-light)" : "var(--color-bg)",
                        cursor: "not-allowed",
                        opacity: 0.7,
                      }}
                    >
                      <input
                        type="radio"
                        name="backup-scope"
                        value={opt.value}
                        defaultChecked={idx === 0}
                        disabled
                        aria-label={opt.label}
                      />
                      <div>
                        <div style={{ fontSize: "var(--text-sm)", fontWeight: 500 }}>
                          {opt.label}
                        </div>
                        <div
                          style={{ fontSize: "var(--text-xs)", color: "var(--color-text-muted)" }}
                        >
                          {opt.description}
                        </div>
                      </div>
                    </label>
                  ))}
                </div>
              </fieldset>

              {/* Output location */}
              <div className="form-field">
                <label htmlFor="output-path" className="form-label">
                  Output Location
                </label>
                <div style={{ display: "flex", gap: "var(--space-2)" }}>
                  <input
                    id="output-path"
                    type="text"
                    className="form-input"
                    placeholder="~/Documents/AirBridge/backups/"
                    disabled
                    aria-label="Output directory for backup files"
                  />
                  <button
                    type="button"
                    className="btn btn-secondary"
                    disabled
                    style={{ flexShrink: 0 }}
                  >
                    Browse
                  </button>
                </div>
              </div>

              {/* Actions */}
              <div style={{ paddingTop: "var(--space-2)" }}>
                <button
                  type="button"
                  className="btn btn-primary"
                  disabled
                  aria-label="Start backup job"
                >
                  {/* Download icon */}
                  <svg
                    width="14"
                    height="14"
                    viewBox="0 0 24 24"
                    fill="none"
                    stroke="currentColor"
                    strokeWidth="2"
                    strokeLinecap="round"
                    strokeLinejoin="round"
                    aria-hidden="true"
                  >
                    <path d="M8 7H5a2 2 0 00-2 2v9a2 2 0 002 2h14a2 2 0 002-2V9a2 2 0 00-2-2h-3m-1 4l-3 3m0 0l-3-3m3 3V4" />
                  </svg>
                  Start Backup
                </button>
              </div>
            </div>
          </div>
        </section>

        {/* Recent Backups section */}
        <section aria-labelledby="recent-backups-heading">
          <SectionHeader title="Recent Backups" />

          <div className="card" style={{ padding: 0, overflow: "hidden" }}>
            <EmptyState
              icon={EMPTY_ICON}
              title="No backups yet"
              description="Run your first backup to see results here."
            />
          </div>
        </section>
      </div>
    </div>
  );
}
