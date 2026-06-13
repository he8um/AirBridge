import { useState, useEffect } from "react";
import type { AirBridgeService } from "../../services/airBridgeService";
import type { JobHistoryItem, JobHistoryListResult, JobHistoryStatus } from "../../backend/types";

interface JobHistoryPanelProps {
  service: AirBridgeService;
  /** Maximum items to display. Defaults to 20. */
  limit?: number;
}

const KIND_LABELS: Record<string, string> = {
  connectionCheck: "Connection Check",
  backupPlan: "Backup Plan",
  recordsExportPlan: "Records Export Plan",
  backupExecution: "Backup",
  packageInspection: "Package Inspection",
  restoreDryRun: "Restore Dry-Run",
  restoreSchemaplan: "Schema Creation Plan",
  restoreRecordImportPlan: "Record Import Plan",
  restoreExecutionAttempt: "Restore Attempt",
};

const STATUS_COLORS: Record<JobHistoryStatus, string> = {
  planned: "var(--color-text-muted)",
  running: "var(--color-info, #3b82f6)",
  succeeded: "var(--color-success, #22c55e)",
  succeededWithWarnings: "var(--color-warning, #f59e0b)",
  blocked: "var(--color-warning, #f59e0b)",
  failed: "var(--color-danger, #ef4444)",
  cancelled: "var(--color-text-muted)",
};

const STATUS_LABELS: Record<JobHistoryStatus, string> = {
  planned: "Planned",
  running: "Running",
  succeeded: "OK",
  succeededWithWarnings: "Warnings",
  blocked: "Blocked",
  failed: "Failed",
  cancelled: "Cancelled",
};

function formatTimestamp(ts: string | undefined): string {
  if (!ts) return "—";
  try {
    return new Date(ts).toLocaleString(undefined, {
      month: "short",
      day: "numeric",
      hour: "2-digit",
      minute: "2-digit",
    });
  } catch {
    return ts;
  }
}

interface HistoryItemRowProps {
  item: JobHistoryItem;
  isLast: boolean;
}

function HistoryItemRow({ item, isLast }: HistoryItemRowProps) {
  const kindLabel = KIND_LABELS[item.kind] ?? item.kind;
  const statusColor = STATUS_COLORS[item.status] ?? "var(--color-text-muted)";
  const statusLabel = STATUS_LABELS[item.status] ?? item.status;

  return (
    <li
      data-testid={`job-history-item-${item.id[0]}`}
      style={{
        display: "flex",
        alignItems: "flex-start",
        justifyContent: "space-between",
        padding: "var(--space-3) var(--space-4)",
        borderBottom: isLast ? "none" : "1px solid var(--color-border)",
        gap: "var(--space-3)",
      }}
    >
      <div
        style={{
          display: "flex",
          flexDirection: "column",
          gap: "var(--space-1)",
          minWidth: 0,
          flex: 1,
        }}
      >
        <span
          data-testid="job-history-item-title"
          style={{ fontSize: "var(--text-sm)", fontWeight: 500 }}
        >
          {item.summary.title}
        </span>

        <span
          data-testid="job-history-item-kind"
          style={{ fontSize: "var(--text-xs)", color: "var(--color-text-muted)" }}
        >
          {kindLabel}
          {item.summary.packageFilename && (
            <>
              {" · "}
              <span data-testid="job-history-item-filename">{item.summary.packageFilename}</span>
            </>
          )}
          {item.summary.baseName && (
            <>
              {" · "}
              <span data-testid="job-history-item-basename">{item.summary.baseName}</span>
            </>
          )}
        </span>

        {(item.summary.warningCount > 0 || item.summary.errorCount > 0) && (
          <span
            data-testid="job-history-item-counts"
            style={{ fontSize: "var(--text-xs)", color: "var(--color-text-muted)" }}
          >
            {item.summary.warningCount > 0 && (
              <span style={{ color: "var(--color-warning, #f59e0b)" }}>
                {item.summary.warningCount} warning{item.summary.warningCount !== 1 ? "s" : ""}
              </span>
            )}
            {item.summary.warningCount > 0 && item.summary.errorCount > 0 && " · "}
            {item.summary.errorCount > 0 && (
              <span style={{ color: "var(--color-danger, #ef4444)" }}>
                {item.summary.errorCount} error{item.summary.errorCount !== 1 ? "s" : ""}
              </span>
            )}
          </span>
        )}

        {item.summary.validationStatus && (
          <span
            data-testid="job-history-item-validation"
            style={{ fontSize: "var(--text-xs)", color: "var(--color-text-muted)" }}
          >
            Validation: {item.summary.validationStatus}
          </span>
        )}

        <span
          data-testid="job-history-item-timestamp"
          style={{ fontSize: "var(--text-xs)", color: "var(--color-text-muted)" }}
        >
          {formatTimestamp(item.finishedAt ?? item.startedAt)}
        </span>
      </div>

      <span
        data-testid="job-history-item-status"
        style={{
          fontSize: "var(--text-xs)",
          fontWeight: 600,
          color: statusColor,
          flexShrink: 0,
          textTransform: "capitalize",
        }}
      >
        {statusLabel}
      </span>
    </li>
  );
}

export function JobHistoryPanel({ service, limit = 20 }: JobHistoryPanelProps) {
  const [result, setResult] = useState<JobHistoryListResult | null>(null);
  const [loading, setLoading] = useState(true);

  useEffect(() => {
    let cancelled = false;
    setLoading(true);
    service
      .listJobHistory({ limit })
      .then((r) => {
        if (!cancelled) {
          setResult(r);
          setLoading(false);
        }
      })
      .catch(() => {
        if (!cancelled) {
          setResult({ items: [], totalCount: 0, filtered: false });
          setLoading(false);
        }
      });
    return () => {
      cancelled = true;
    };
  }, [service, limit]);

  return (
    <div data-testid="job-history-panel">
      <h3
        style={{
          fontSize: "var(--text-sm)",
          fontWeight: 600,
          color: "var(--color-text-muted)",
          textTransform: "uppercase",
          letterSpacing: "0.06em",
          marginBottom: "var(--space-3)",
        }}
      >
        Recent Activity
      </h3>

      <div className="card" style={{ padding: 0, overflow: "hidden" }}>
        {loading ? (
          <div
            data-testid="job-history-loading"
            style={{
              padding: "var(--space-8)",
              textAlign: "center",
              fontSize: "var(--text-sm)",
              color: "var(--color-text-muted)",
            }}
          >
            Loading activity…
          </div>
        ) : !result || result.items.length === 0 ? (
          <div
            data-testid="job-history-empty"
            style={{
              padding: "var(--space-10) var(--space-8)",
              textAlign: "center",
              fontSize: "var(--text-sm)",
              color: "var(--color-text-muted)",
            }}
            role="status"
            aria-label="No recent activity"
          >
            No recent activity to show.
          </div>
        ) : (
          <ul
            data-testid="job-history-list"
            aria-label="Recent activity"
            style={{ listStyle: "none", margin: 0, padding: 0 }}
          >
            {result.items.map((item, idx) => (
              <HistoryItemRow
                key={item.id[0]}
                item={item}
                isLast={idx === result.items.length - 1}
              />
            ))}
          </ul>
        )}
      </div>

      {result && result.totalCount > result.items.length && (
        <p
          data-testid="job-history-truncated-note"
          style={{
            fontSize: "var(--text-xs)",
            color: "var(--color-text-muted)",
            marginTop: "var(--space-2)",
          }}
        >
          Showing {result.items.length} of {result.totalCount} items.
        </p>
      )}

      <p
        data-testid="job-history-persistence-note"
        style={{
          fontSize: "var(--text-xs)",
          color: "var(--color-text-muted)",
          marginTop: "var(--space-2)",
          fontStyle: "italic",
        }}
      >
        Activity is stored in memory only and does not persist between sessions.
      </p>
    </div>
  );
}
