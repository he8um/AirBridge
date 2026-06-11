import { SectionHeader } from "../components/SectionHeader";
import { EmptyState } from "../components/EmptyState";
import { StatusBadge } from "../components/StatusBadge";
import { useAppState } from "../state/useAppState";
import type { BackupStatus } from "../domain/backup";
import type { AirtableBaseSummary } from "../domain/airtable";

const SCOPE_OPTIONS = [
  { value: "full", label: "Full backup", description: "Schema and all records" },
  { value: "schema", label: "Schema only", description: "Table structure, no records" },
  { value: "records", label: "Records only", description: "Records without schema" },
] as const;

const EMPTY_ICON =
  "M20 13V6a2 2 0 00-2-2H6a2 2 0 00-2 2v7m16 0v5a2 2 0 01-2 2H6a2 2 0 01-2-2v-5m16 0h-2.586a1 1 0 00-.707.293l-2.414 2.414a1 1 0 01-.707.293h-3.172a1 1 0 01-.707-.293l-2.414-2.414A1 1 0 006.586 13H4";

function backupStatusBadge(status: BackupStatus): "connected" | "error" | "warning" | "idle" {
  switch (status) {
    case "succeeded":
      return "connected";
    case "failed":
      return "error";
    case "running":
    case "pending":
      return "warning";
    default:
      return "idle";
  }
}

function formatBytes(bytes: number): string {
  if (bytes === 0) return "0 B";
  if (bytes < 1024) return `${bytes} B`;
  if (bytes < 1024 * 1024) return `${(bytes / 1024).toFixed(1)} KB`;
  return `${(bytes / (1024 * 1024)).toFixed(1)} MB`;
}

function BaseCatalogCard({ bases }: { bases: AirtableBaseSummary[] }) {
  if (bases.length === 0) {
    return (
      <div className="card notice-neutral" style={{ maxWidth: 560 }}>
        <p style={{ fontSize: "var(--text-sm)", color: "var(--color-text-muted)" }}>
          No bases loaded. Connect an Airtable account to see available bases.
        </p>
      </div>
    );
  }

  return (
    <div className="card" style={{ maxWidth: 560, padding: 0, overflow: "hidden" }}>
      <ul style={{ listStyle: "none", margin: 0, padding: 0 }} aria-label="Available bases catalog">
        {bases.map((base, idx) => {
          const totalFields = base.tables?.reduce((sum, t) => sum + t.fieldCount, 0) ?? 0;
          return (
            <li
              key={base.id}
              style={{
                display: "flex",
                flexDirection: "column",
                gap: "var(--space-1)",
                padding: "var(--space-4) var(--space-5)",
                borderBottom: idx < bases.length - 1 ? "1px solid var(--color-border)" : "none",
              }}
            >
              <span style={{ fontSize: "var(--text-sm)", fontWeight: 500 }}>{base.name}</span>
              <span style={{ fontSize: "var(--text-xs)", color: "var(--color-text-muted)" }}>
                {base.tableCount} {base.tableCount === 1 ? "table" : "tables"}
                {totalFields > 0 && ` · ${totalFields} fields`}
              </span>
              <span
                style={{
                  fontSize: "var(--text-xs)",
                  color: "var(--color-text-muted)",
                  fontFamily: "var(--font-mono)",
                }}
              >
                {base.id}
              </span>
            </li>
          );
        })}
      </ul>
    </div>
  );
}

export function BackupsPage() {
  const { recentBackups, state } = useAppState();

  const bases = state.bases;

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
                  disabled={bases.length === 0}
                  aria-label="Select Airtable base to back up"
                >
                  {bases.length === 0 ? (
                    <option value="">No bases connected</option>
                  ) : (
                    <>
                      <option value="">Choose a base…</option>
                      {bases.map((base) => (
                        <option key={base.id} value={base.id}>
                          {base.name}
                          {base.tableCount > 0
                            ? ` (${base.tableCount} ${base.tableCount === 1 ? "table" : "tables"})`
                            : ""}
                        </option>
                      ))}
                    </>
                  )}
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

        {/* Base Catalog section */}
        <section aria-labelledby="base-catalog-heading">
          <SectionHeader title="Available Bases" />
          <BaseCatalogCard bases={bases} />
        </section>

        {/* Recent Backups section */}
        <section aria-labelledby="recent-backups-heading">
          <SectionHeader title="Recent Backups" />

          <div className="card" style={{ padding: 0, overflow: "hidden" }}>
            {recentBackups.length === 0 ? (
              <EmptyState
                icon={EMPTY_ICON}
                title="No backups yet"
                description="Run your first backup to see results here."
              />
            ) : (
              <ul style={{ listStyle: "none", margin: 0, padding: 0 }} aria-label="Recent backups">
                {recentBackups.map((pkg, idx) => (
                  <li
                    key={pkg.id}
                    style={{
                      display: "flex",
                      alignItems: "center",
                      justifyContent: "space-between",
                      padding: "var(--space-4) var(--space-5)",
                      borderBottom:
                        idx < recentBackups.length - 1 ? "1px solid var(--color-border)" : "none",
                      gap: "var(--space-4)",
                    }}
                  >
                    <div
                      style={{
                        display: "flex",
                        flexDirection: "column",
                        gap: "var(--space-1)",
                        minWidth: 0,
                      }}
                    >
                      <span
                        style={{
                          fontSize: "var(--text-sm)",
                          fontWeight: 500,
                          overflow: "hidden",
                          textOverflow: "ellipsis",
                          whiteSpace: "nowrap",
                        }}
                      >
                        {pkg.baseName}
                      </span>
                      <span
                        style={{ fontSize: "var(--text-xs)", color: "var(--color-text-muted)" }}
                      >
                        {pkg.scope === "full"
                          ? "Full"
                          : pkg.scope === "schema_only"
                            ? "Schema only"
                            : "Records only"}
                        {" · "}
                        {pkg.tableCount} {pkg.tableCount === 1 ? "table" : "tables"}
                        {pkg.scope !== "schema_only" && ` · ${pkg.recordCount} records`}
                        {pkg.fileSizeBytes > 0 && ` · ${formatBytes(pkg.fileSizeBytes)}`}
                      </span>
                      <span
                        style={{ fontSize: "var(--text-xs)", color: "var(--color-text-muted)" }}
                      >
                        {new Date(pkg.createdAt).toLocaleDateString(undefined, {
                          year: "numeric",
                          month: "short",
                          day: "numeric",
                        })}
                      </span>
                    </div>
                    <StatusBadge
                      status={backupStatusBadge(pkg.status)}
                      label={pkg.status.charAt(0).toUpperCase() + pkg.status.slice(1)}
                    />
                  </li>
                ))}
              </ul>
            )}
          </div>
        </section>
      </div>
    </div>
  );
}
