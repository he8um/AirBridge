import { SectionHeader } from "../components/SectionHeader";

export function SettingsPage() {
  return (
    <div className="page">
      <div className="page-content">
        {/* Local Storage */}
        <section aria-labelledby="storage-heading">
          <SectionHeader title="Local Storage" />

          <div
            className="card"
            style={{
              maxWidth: 560,
              display: "flex",
              flexDirection: "column",
              gap: "var(--space-5)",
            }}
          >
            <div className="form-field">
              <label htmlFor="backup-dir" className="form-label">
                Backup directory
              </label>
              <div style={{ display: "flex", gap: "var(--space-2)" }}>
                <input
                  id="backup-dir"
                  type="text"
                  className="form-input"
                  placeholder="~/Documents/AirBridge/backups/"
                  disabled
                  aria-label="Backup storage directory"
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

            <div className="form-field">
              <label htmlFor="max-history" className="form-label">
                Max backup history
              </label>
              <input
                id="max-history"
                type="number"
                className="form-input"
                defaultValue={10}
                disabled
                aria-label="Maximum number of backups to retain"
                style={{ maxWidth: 120 }}
              />
              <p style={{ fontSize: "var(--text-xs)", color: "var(--color-text-muted)" }}>
                Older backups beyond this limit will be archived.
              </p>
            </div>
          </div>
        </section>

        {/* Privacy */}
        <section aria-labelledby="privacy-heading">
          <SectionHeader title="Privacy" />

          <div className="card notice-info notice" style={{ maxWidth: 560 }}>
            {/* Shield icon */}
            <svg
              width="16"
              height="16"
              viewBox="0 0 24 24"
              fill="none"
              stroke="currentColor"
              strokeWidth="2"
              strokeLinecap="round"
              strokeLinejoin="round"
              aria-hidden="true"
              style={{ flexShrink: 0, marginTop: 1 }}
            >
              <path d="M12 22s8-4 8-10V5l-8-3-8 3v7c0 6 8 10 8 10z" />
            </svg>
            <p style={{ fontSize: "var(--text-sm)", lineHeight: "var(--line-height-base)" }}>
              AirBridge does not collect telemetry, usage data, or crash reports. All operations are
              local.
            </p>
          </div>
        </section>

        {/* Redaction Defaults */}
        <section aria-labelledby="redaction-heading">
          <SectionHeader title="Redaction Defaults" />

          <div
            className="card"
            style={{
              maxWidth: 560,
              display: "flex",
              flexDirection: "column",
              gap: "var(--space-4)",
            }}
          >
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
                aria-label="Redact attachment URLs in backups"
              />
              <div>
                <div style={{ fontSize: "var(--text-sm)", fontWeight: 500 }}>
                  Redact attachment URLs
                </div>
                <div style={{ fontSize: "var(--text-xs)", color: "var(--color-text-muted)" }}>
                  Replace attachment URLs with placeholder values in backup files
                </div>
              </div>
            </label>

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
                aria-label="Redact formula field values in backups"
              />
              <div>
                <div style={{ fontSize: "var(--text-sm)", fontWeight: 500 }}>
                  Redact formula fields
                </div>
                <div style={{ fontSize: "var(--text-xs)", color: "var(--color-text-muted)" }}>
                  Omit computed formula values from backup files
                </div>
              </div>
            </label>
          </div>
        </section>

        {/* Appearance */}
        <section aria-labelledby="appearance-heading">
          <SectionHeader title="Appearance" />

          <div className="card" style={{ maxWidth: 560 }}>
            <div className="form-field">
              <label className="form-label">Theme</label>
              <p
                style={{
                  fontSize: "var(--text-sm)",
                  color: "var(--color-text-muted)",
                  padding: "var(--space-2) 0",
                }}
              >
                System default (light/dark theme support coming soon)
              </p>
            </div>
          </div>
        </section>
      </div>
    </div>
  );
}
