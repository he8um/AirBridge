import { useState } from "react";
import { SectionHeader } from "../components/SectionHeader";
import { useAppState } from "../state/useAppState";
import type { LogLevel } from "../domain/log";

type LogFilter = "all" | "warnings" | "errors";

const FILTERS: { id: LogFilter; label: string }[] = [
  { id: "all", label: "All" },
  { id: "warnings", label: "Warnings" },
  { id: "errors", label: "Errors" },
];

const LOGS_ICON =
  "M9 12h6m-6 4h6m2 5H7a2 2 0 01-2-2V5a2 2 0 012-2h5.586a1 1 0 01.707.293l5.414 5.414a1 1 0 01.293.707V19a2 2 0 01-2 2z";

const LEVEL_COLORS: Record<LogLevel, string> = {
  debug: "var(--color-text-muted)",
  info: "var(--color-text)",
  warning: "var(--color-warning, #d97706)",
  error: "var(--color-danger)",
};

const LEVEL_LABELS: Record<LogLevel, string> = {
  debug: "DBG",
  info: "INF",
  warning: "WRN",
  error: "ERR",
};

function filterToLevel(filter: LogFilter): LogLevel | null {
  if (filter === "warnings") return "warning";
  if (filter === "errors") return "error";
  return null;
}

export function LogsPage() {
  const [activeFilter, setActiveFilter] = useState<LogFilter>("all");
  const { state, getLogsByLevel } = useAppState();

  const level = filterToLevel(activeFilter);
  const entries = level !== null ? getLogsByLevel(level) : state.logs;

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
            }}
            role="log"
            aria-live="polite"
            aria-label="Job log output"
          >
            {entries.length === 0 ? (
              <div
                style={{
                  display: "flex",
                  flexDirection: "column",
                  alignItems: "center",
                  justifyContent: "center",
                  gap: "var(--space-4)",
                  padding: "var(--space-12) var(--space-8)",
                  textAlign: "center",
                  height: "100%",
                  minHeight: 360,
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
            ) : (
              <ul
                style={{
                  listStyle: "none",
                  margin: 0,
                  padding: "var(--space-2) 0",
                  fontFamily: "monospace",
                }}
                aria-label="Log entries"
              >
                {entries.map((entry) => (
                  <li
                    key={entry.id}
                    style={{
                      display: "flex",
                      gap: "var(--space-3)",
                      padding: "var(--space-2) var(--space-5)",
                      fontSize: "var(--text-xs)",
                      lineHeight: 1.5,
                      borderBottom: "1px solid var(--color-border)",
                    }}
                  >
                    <span
                      style={{ color: "var(--color-text-muted)", flexShrink: 0, minWidth: 160 }}
                    >
                      {new Date(entry.timestamp).toLocaleTimeString()}
                    </span>
                    <span
                      style={{
                        color: LEVEL_COLORS[entry.level],
                        fontWeight: 600,
                        flexShrink: 0,
                        minWidth: 32,
                      }}
                    >
                      {LEVEL_LABELS[entry.level]}
                    </span>
                    <span style={{ color: "var(--color-text)" }}>
                      {entry.message}
                      {entry.detail && (
                        <span
                          style={{ color: "var(--color-text-muted)", marginLeft: "var(--space-2)" }}
                        >
                          — {entry.detail}
                        </span>
                      )}
                    </span>
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
