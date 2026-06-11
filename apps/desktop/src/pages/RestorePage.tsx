import { SectionHeader } from "../components/SectionHeader";
import { useAppState } from "../state/useAppState";

export function RestorePage() {
  const { state, compatibilitySummary } = useAppState();
  const plan = state.restorePlans[0];
  const bases = state.bases;

  return (
    <div className="page">
      <div className="page-content">
        {/* Restore form section */}
        <section aria-labelledby="restore-heading">
          <SectionHeader title="Restore from Backup" />

          <div
            className="card"
            style={{
              maxWidth: 600,
              display: "flex",
              flexDirection: "column",
              gap: "var(--space-6)",
            }}
          >
            {/* Backup file selector */}
            <div className="form-field">
              <label className="form-label">Backup File</label>
              <div style={{ display: "flex", gap: "var(--space-2)", alignItems: "center" }}>
                <input
                  type="text"
                  className="form-input"
                  placeholder="No file selected"
                  disabled
                  readOnly
                  aria-label="Selected backup file path"
                />
                <button
                  type="button"
                  className="btn btn-secondary"
                  disabled
                  aria-label="Choose .airbridge backup file"
                  style={{ flexShrink: 0, whiteSpace: "nowrap" }}
                >
                  <svg
                    width="13"
                    height="13"
                    viewBox="0 0 24 24"
                    fill="none"
                    stroke="currentColor"
                    strokeWidth="2"
                    strokeLinecap="round"
                    strokeLinejoin="round"
                    aria-hidden="true"
                  >
                    <path d="M14 2H6a2 2 0 00-2 2v16a2 2 0 002 2h12a2 2 0 002-2V8z" />
                    <polyline points="14 2 14 8 20 8" />
                  </svg>
                  Choose .airbridge file
                </button>
              </div>
            </div>

            <div className="divider" style={{ margin: 0 }} />

            {/* Inspection placeholder */}
            <div>
              <p
                style={{
                  fontSize: "var(--text-xs)",
                  fontWeight: 600,
                  color: "var(--color-text-muted)",
                  textTransform: "uppercase",
                  letterSpacing: "0.06em",
                  marginBottom: "var(--space-3)",
                }}
              >
                Inspection
              </p>
              <div
                className="notice notice-neutral"
                style={{ justifyContent: "center", textAlign: "center" }}
                role="status"
                aria-label="Backup inspection placeholder"
              >
                <span>Select a backup file to inspect its contents.</span>
              </div>
            </div>

            <div className="divider" style={{ margin: 0 }} />

            {/* Restore options */}
            <div>
              <p
                style={{
                  fontSize: "var(--text-xs)",
                  fontWeight: 600,
                  color: "var(--color-text-muted)",
                  textTransform: "uppercase",
                  letterSpacing: "0.06em",
                  marginBottom: "var(--space-4)",
                }}
              >
                Restore Options
              </p>

              <div style={{ display: "flex", flexDirection: "column", gap: "var(--space-4)" }}>
                {/* Dry-run toggle */}
                <label
                  style={{
                    display: "flex",
                    alignItems: "center",
                    gap: "var(--space-3)",
                    cursor: "not-allowed",
                    opacity: 0.7,
                  }}
                >
                  <input
                    type="checkbox"
                    disabled
                    style={{ width: 16, height: 16, flexShrink: 0 }}
                    aria-label="Enable dry-run mode"
                  />
                  <div>
                    <div style={{ fontSize: "var(--text-sm)", fontWeight: 500 }}>Dry-run mode</div>
                    <div style={{ fontSize: "var(--text-xs)", color: "var(--color-text-muted)" }}>
                      Simulate the restore without writing any changes
                    </div>
                  </div>
                </label>

                {/* Target base */}
                <div className="form-field">
                  <label htmlFor="target-base-select" className="form-label">
                    Target Base
                  </label>
                  <select
                    id="target-base-select"
                    className="form-input"
                    disabled
                    aria-label="Select target Airtable base for restore"
                  >
                    {bases.length === 0 ? (
                      <option value="">No bases connected</option>
                    ) : (
                      bases.map((base) => (
                        <option key={base.id} value={base.id}>
                          {base.name}
                        </option>
                      ))
                    )}
                  </select>
                </div>

                {/* Start restore */}
                <div style={{ paddingTop: "var(--space-2)" }}>
                  <button
                    type="button"
                    className="btn btn-primary"
                    disabled
                    aria-label="Start restore job"
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
                      <path d="M4 4v5h.582m15.356 2A8.001 8.001 0 004.582 9m0 0H9m11 11v-5h-.581m0 0a8.003 8.003 0 01-15.357-2m15.357 2H15" />
                    </svg>
                    Start Restore
                  </button>
                </div>
              </div>
            </div>
          </div>
        </section>

        {/* Compatibility section */}
        <section aria-labelledby="compatibility-heading">
          <SectionHeader title="Compatibility" />

          <div
            className="card"
            style={{
              maxWidth: 600,
              display: "flex",
              flexDirection: "column",
              gap: "var(--space-4)",
            }}
          >
            {/* Summary row */}
            <div style={{ display: "flex", gap: "var(--space-6)", flexWrap: "wrap" }}>
              <div style={{ display: "flex", flexDirection: "column", gap: "var(--space-1)" }}>
                <span
                  style={{
                    fontSize: "var(--text-xs)",
                    color: "var(--color-text-muted)",
                    textTransform: "uppercase",
                    letterSpacing: "0.06em",
                  }}
                >
                  Restorable
                </span>
                <span style={{ fontSize: "var(--text-lg)", fontWeight: 600 }}>
                  {compatibilitySummary.bySupport.restorable}
                </span>
              </div>
              <div style={{ display: "flex", flexDirection: "column", gap: "var(--space-1)" }}>
                <span
                  style={{
                    fontSize: "var(--text-xs)",
                    color: "var(--color-text-muted)",
                    textTransform: "uppercase",
                    letterSpacing: "0.06em",
                  }}
                >
                  Partial
                </span>
                <span style={{ fontSize: "var(--text-lg)", fontWeight: 600 }}>
                  {compatibilitySummary.bySupport.partially_restorable}
                </span>
              </div>
              <div style={{ display: "flex", flexDirection: "column", gap: "var(--space-1)" }}>
                <span
                  style={{
                    fontSize: "var(--text-xs)",
                    color: "var(--color-text-muted)",
                    textTransform: "uppercase",
                    letterSpacing: "0.06em",
                  }}
                >
                  Unsupported
                </span>
                <span style={{ fontSize: "var(--text-lg)", fontWeight: 600 }}>
                  {compatibilitySummary.bySupport.unsupported_for_restore}
                </span>
              </div>
            </div>

            {/* Plan warnings */}
            {plan && plan.warnings.length > 0 && (
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
                  Warnings for selected plan
                </p>
                <ul
                  style={{
                    listStyle: "none",
                    margin: 0,
                    padding: 0,
                    display: "flex",
                    flexDirection: "column",
                    gap: "var(--space-2)",
                  }}
                >
                  {plan.warnings.map((w) => (
                    <li
                      key={w.fieldId}
                      className={`notice notice-${w.severity === "warning" ? "warning" : w.severity === "error" ? "danger" : "info"}`}
                    >
                      <span>
                        <strong>{w.fieldName}</strong> ({w.fieldType}): {w.message}
                      </span>
                    </li>
                  ))}
                </ul>
              </div>
            )}

            {!plan && (
              <p style={{ fontSize: "var(--text-sm)", color: "var(--color-text-muted)" }}>
                Field compatibility and warnings will appear here after selecting a backup file.
              </p>
            )}
          </div>
        </section>
      </div>
    </div>
  );
}
