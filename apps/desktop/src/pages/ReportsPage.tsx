import { useState } from "react";
import { useAppState } from "../state/useAppState";
import type { ReportType } from "../domain/report";

type ReportTab = "backup" | "restore" | "validation";

interface Tab {
  id: ReportTab;
  label: string;
}

const TABS: Tab[] = [
  { id: "backup", label: "Backup Reports" },
  { id: "restore", label: "Restore Reports" },
  { id: "validation", label: "Validation Reports" },
];

const REPORT_ICON =
  "M9 17v-2m3 2v-4m3 4v-6m2 10H7a2 2 0 01-2-2V5a2 2 0 012-2h5.586a1 1 0 01.707.293l5.414 5.414a1 1 0 01.293.707V19a2 2 0 01-2 2z";

const TAB_DESCRIPTIONS: Record<ReportTab, string> = {
  backup: "Reports are generated automatically after each backup job.",
  restore: "Reports are generated automatically after each restore job.",
  validation: "Reports are generated automatically after each validation run.",
};

function tabToReportType(tab: ReportTab): ReportType {
  return tab === "validation" ? "validation" : tab;
}

function severityColor(severity: string): string {
  switch (severity) {
    case "error":
      return "var(--color-danger)";
    case "warning":
      return "var(--color-warning, #d97706)";
    default:
      return "var(--color-text-muted)";
  }
}

export function ReportsPage() {
  const [activeTab, setActiveTab] = useState<ReportTab>("backup");
  const { recentReports } = useAppState();

  const tabReports = recentReports.filter(
    (r) =>
      r.type === tabToReportType(activeTab) ||
      (activeTab === "validation" && r.type === "compatibility"),
  );

  return (
    <div className="page">
      <div className="page-content">
        <section aria-labelledby="reports-heading">
          <h2
            id="reports-heading"
            style={{
              fontSize: "var(--text-sm)",
              fontWeight: 600,
              color: "var(--color-text-muted)",
              textTransform: "uppercase",
              letterSpacing: "0.06em",
              marginBottom: "var(--space-4)",
            }}
          >
            Reports
          </h2>

          {/* Tab bar */}
          <div
            className="tab-bar"
            role="tablist"
            aria-label="Report categories"
            style={{ marginBottom: "var(--space-4)" }}
          >
            {TABS.map((tab) => (
              <button
                key={tab.id}
                type="button"
                role="tab"
                aria-selected={activeTab === tab.id}
                aria-controls={`tab-panel-${tab.id}`}
                id={`tab-${tab.id}`}
                className={`tab-btn${activeTab === tab.id ? " active" : ""}`}
                onClick={() => setActiveTab(tab.id)}
              >
                {tab.label}
              </button>
            ))}
          </div>

          {/* Tab panels */}
          {TABS.map((tab) => (
            <div
              key={tab.id}
              id={`tab-panel-${tab.id}`}
              role="tabpanel"
              aria-labelledby={`tab-${tab.id}`}
              hidden={activeTab !== tab.id}
            >
              {activeTab === tab.id && (
                <div className="card" style={{ padding: 0, overflow: "hidden" }}>
                  {tabReports.length === 0 ? (
                    <div
                      style={{
                        display: "flex",
                        flexDirection: "column",
                        alignItems: "center",
                        justifyContent: "center",
                        gap: "var(--space-4)",
                        padding: "var(--space-12) var(--space-8)",
                        textAlign: "center",
                      }}
                      role="status"
                      aria-label="No reports available"
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
                          <path d={REPORT_ICON} />
                        </svg>
                      </div>
                      <div
                        style={{ display: "flex", flexDirection: "column", gap: "var(--space-2)" }}
                      >
                        <h3 style={{ fontSize: "var(--text-base)", fontWeight: 600 }}>
                          No reports yet
                        </h3>
                        <p
                          style={{
                            fontSize: "var(--text-sm)",
                            color: "var(--color-text-muted)",
                            maxWidth: 320,
                          }}
                        >
                          {TAB_DESCRIPTIONS[tab.id]}
                        </p>
                      </div>
                    </div>
                  ) : (
                    <ul
                      style={{ listStyle: "none", margin: 0, padding: 0 }}
                      aria-label={`${tab.label} list`}
                    >
                      {tabReports.map((report, idx) => (
                        <li
                          key={report.id}
                          style={{
                            display: "flex",
                            alignItems: "flex-start",
                            justifyContent: "space-between",
                            padding: "var(--space-4) var(--space-5)",
                            borderBottom:
                              idx < tabReports.length - 1
                                ? "1px solid var(--color-border)"
                                : "none",
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
                            <span style={{ fontSize: "var(--text-sm)", fontWeight: 500 }}>
                              {report.title}
                            </span>
                            {report.relatedBaseName && (
                              <span
                                style={{
                                  fontSize: "var(--text-xs)",
                                  color: "var(--color-text-muted)",
                                }}
                              >
                                {report.relatedBaseName}
                              </span>
                            )}
                            <span
                              style={{
                                fontSize: "var(--text-xs)",
                                color: "var(--color-text-muted)",
                              }}
                            >
                              {new Date(report.createdAt).toLocaleDateString(undefined, {
                                year: "numeric",
                                month: "short",
                                day: "numeric",
                              })}
                              {" · "}
                              {report.itemCount} {report.itemCount === 1 ? "item" : "items"}
                            </span>
                          </div>
                          <span
                            style={{
                              fontSize: "var(--text-xs)",
                              fontWeight: 600,
                              color: severityColor(report.severity),
                              flexShrink: 0,
                              textTransform: "capitalize",
                            }}
                          >
                            {report.severity}
                          </span>
                        </li>
                      ))}
                    </ul>
                  )}
                </div>
              )}
            </div>
          ))}
        </section>
      </div>
    </div>
  );
}
