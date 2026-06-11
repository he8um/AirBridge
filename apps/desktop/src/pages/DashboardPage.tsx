import { StatCard } from "../components/StatCard";
import { SectionHeader } from "../components/SectionHeader";

export function DashboardPage() {
  function handleNewBackup() {
    console.log("New Backup clicked");
  }
  function handleOpenBackup() {
    console.log("Open Backup File clicked");
  }
  function handleRestoreBackup() {
    console.log("Restore Backup clicked");
  }

  return (
    <div className="page">
      <div className="page-content">
        {/* Welcome block */}
        <section aria-labelledby="welcome-heading">
          <div style={{ marginBottom: "var(--space-4)" }}>
            <h1
              id="welcome-heading"
              style={{ fontSize: "var(--text-2xl)", marginBottom: "var(--space-2)" }}
            >
              Welcome to AirBridge
            </h1>
            <p style={{ color: "var(--color-text-muted)", fontSize: "var(--text-base)" }}>
              Local backup, inspection, and restore for Airtable bases.
            </p>
          </div>

          {/* Privacy notice */}
          <div className="notice notice-info" role="note" aria-label="Privacy notice">
            {/* Lock icon */}
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
              <rect x="3" y="11" width="18" height="11" rx="2" ry="2" />
              <path d="M7 11V7a5 5 0 0110 0v4" />
            </svg>
            <span>
              <strong>No data leaves your device.</strong> All operations run locally. Nothing is
              sent to external servers.
            </span>
          </div>
        </section>

        {/* Stats */}
        <section aria-labelledby="stats-heading">
          <h2
            id="stats-heading"
            style={{
              fontSize: "var(--text-sm)",
              fontWeight: 600,
              color: "var(--color-text-muted)",
              textTransform: "uppercase",
              letterSpacing: "0.06em",
              marginBottom: "var(--space-3)",
            }}
          >
            Activity Summary
          </h2>
          <div className="stat-grid">
            <StatCard label="Recent Backups" value={0} note="Last 30 days" />
            <StatCard label="Restore Jobs" value={0} note="Last 30 days" />
            <StatCard label="Connected Bases" value={0} note="Active connections" />
          </div>
        </section>

        {/* Quick Actions */}
        <section aria-labelledby="actions-heading">
          <SectionHeader title="Quick Actions" />
          <div className="quick-actions">
            <button
              type="button"
              className="btn btn-primary"
              onClick={handleNewBackup}
              aria-label="Create a new backup"
            >
              {/* Download / save icon */}
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
              New Backup
            </button>

            <button
              type="button"
              className="btn btn-secondary"
              onClick={handleOpenBackup}
              aria-label="Open an existing backup file"
            >
              {/* Folder open icon */}
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
                <path d="M5 19a2 2 0 01-2-2V7a2 2 0 012-2h4l2 2h4a2 2 0 012 2v1M5 19h14a2 2 0 002-2v-5a2 2 0 00-2-2H9a2 2 0 00-2 2v5a2 2 0 01-2 2z" />
              </svg>
              Open Backup File
            </button>

            <button
              type="button"
              className="btn btn-secondary"
              onClick={handleRestoreBackup}
              aria-label="Restore from a backup"
            >
              {/* Restore / refresh icon */}
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
              Restore Backup
            </button>
          </div>
        </section>
      </div>
    </div>
  );
}
