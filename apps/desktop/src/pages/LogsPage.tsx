import { useState } from "react";
import { SectionHeader } from "../components/SectionHeader";

type LogFilter = "all" | "warnings" | "errors";

const FILTERS: { id: LogFilter; label: string }[] = [
  { id: "all", label: "All" },
  { id: "warnings", label: "Warnings" },
  { id: "errors", label: "Errors" },
];

// Console / terminal icon path
const LOGS_ICON =
  "M9 12h6m-6 4h6m2 5H7a2 2 0 01-2-2V5a2 2 0 012-2h5.586a1 1 0 01.707.293l5.414 5.414a1 1 0 01.293.707V19a2 2 0 01-2 2z";

export function LogsPage() {
  const [activeFilter, setActiveFilter] = useState<LogFilter>("all");

  return (
    <div className="page">
      <div className="page-content">
        <section aria-labelledby="logs-heading">
          <SectionHeader
            title="Job Logs"
            action={{
              label: "Clear Logs",
              onClick: () => {},
              disabled: true,
            }}
          />

          {/* Filter row */}
          <div
            className="filter-row"
            role="group"
            aria-label="Filter logs by severity"
            style={{ marginBottom: "var(--space-4)" }}
          >
            {FILTERS.map((f) => (
              <button
                key={f.id}
                type="button"
                className={`filter-btn${activeFilter === f.id ? " active" : ""}`}
                onClick={() => setActiveFilter(f.id)}
                aria-pressed={activeFilter === f.id}
              >
                {f.label}
              </button>
            ))}
          </div>

          {/* Log content area */}
          <div
            className="card"
            style={{
              padding: 0,
              overflow: "hidden",
              minHeight: 360,
              display: "flex",
              alignItems: "center",
              justifyContent: "center",
            }}
            role="log"
            aria-live="polite"
            aria-label="Job log output"
          >
            <div
              style={{
                display: "flex",
                flexDirection: "column",
                alignItems: "center",
                justifyContent: "center",
                gap: "var(--space-4)",
                padding: "var(--space-12) var(--space-8)",
                textAlign: "center",
                width: "100%",
              }}
            >
              <div
                style={{
                  width: 48,
                  height: 48,
                  borderRadius: "var(--radius-lg)",
                  backgroundColor: "var(--color-bg)",
                  border: "1px solid var(--color-border)",
                  display: "flex",
                  alignItems: "center",
                  justifyContent: "center",
                  flexShrink: 0,
                }}
                aria-hidden="true"
              >
                <svg
                  width="24"
                  height="24"
                  viewBox="0 0 24 24"
                  fill="none"
                  stroke="var(--color-text-muted)"
                  strokeWidth="1.5"
                  strokeLinecap="round"
                  strokeLinejoin="round"
                >
                  <path d={LOGS_ICON} />
                </svg>
              </div>
              <div style={{ display: "flex", flexDirection: "column", gap: "var(--space-2)" }}>
                <h3 style={{ fontSize: "var(--text-base)", fontWeight: 600 }}>No logs</h3>
                <p
                  style={{
                    fontSize: "var(--text-sm)",
                    color: "var(--color-text-muted)",
                    maxWidth: 320,
                  }}
                >
                  Logs will appear here during and after backup or restore jobs.
                </p>
              </div>
            </div>
          </div>
        </section>
      </div>
    </div>
  );
}
