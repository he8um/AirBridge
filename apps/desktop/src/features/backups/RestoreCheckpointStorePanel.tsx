import type {
  RestoreCheckpointStoreRequest,
  RestoreCheckpointStoreResult,
} from "../../backend/types";

interface Props {
  request: RestoreCheckpointStoreRequest;
  onStore: (request: RestoreCheckpointStoreRequest) => Promise<RestoreCheckpointStoreResult | null>;
  result: RestoreCheckpointStoreResult | null;
  loading: boolean;
}

export function RestoreCheckpointStorePanel({ request, onStore, result, loading }: Props) {
  return (
    <div
      data-testid="restore-checkpoint-store-panel"
      style={{ display: "flex", flexDirection: "column", gap: "var(--space-3)" }}
    >
      <div style={{ display: "flex", alignItems: "center", gap: "var(--space-2)" }}>
        <strong style={{ fontSize: "var(--text-sm)" }}>Checkpoint Metadata Store</strong>
        <span
          className="badge badge-neutral"
          style={{ fontSize: "var(--text-xs)" }}
          data-testid="rcps-metadata-only-badge"
        >
          Metadata Only
        </span>
      </div>

      <div
        className="notice notice-info"
        data-testid="rcps-restore-not-triggered-notice"
        style={{ fontSize: "var(--text-xs)" }}
      >
        Storing checkpoint metadata does not execute restore. Live restore writes remain disabled.
        No token, full path, record payload, record IDs, raw HTTP data, or attachment URL is stored.
      </div>

      <button
        className="btn btn-secondary"
        data-testid="rcps-store-button"
        disabled={loading}
        onClick={() => onStore(request)}
        style={{ alignSelf: "flex-start", fontSize: "var(--text-sm)" }}
      >
        {loading ? "Storing…" : "Store Checkpoint Metadata"}
      </button>

      {result && (
        <div
          data-testid="rcps-result"
          style={{ display: "flex", flexDirection: "column", gap: "var(--space-2)" }}
        >
          <div style={{ display: "flex", alignItems: "center", gap: "var(--space-2)" }}>
            {result.status === "stored" ? (
              <span
                className="badge badge-neutral"
                data-testid="rcps-stored-badge"
                style={{ fontSize: "var(--text-xs)" }}
              >
                Stored
              </span>
            ) : (
              <span
                className="badge badge-danger"
                data-testid="rcps-blocked-badge"
                style={{ fontSize: "var(--text-xs)" }}
              >
                Blocked
              </span>
            )}
            <span
              className="badge badge-neutral"
              data-testid="rcps-restore-not-triggered-tag"
              style={{ fontSize: "var(--text-xs)" }}
            >
              Restore Not Triggered
            </span>
          </div>

          <p data-testid="rcps-message" style={{ fontSize: "var(--text-xs)", margin: 0 }}>
            {result.message}
          </p>

          {result.blockedReason && (
            <p
              data-testid="rcps-blocked-reason"
              className="notice notice-danger"
              style={{ fontSize: "var(--text-xs)", margin: 0 }}
            >
              {result.blockedReason}
            </p>
          )}

          {result.summary && (
            <div
              data-testid="rcps-summary"
              style={{
                fontSize: "var(--text-xs)",
                display: "flex",
                flexDirection: "column",
                gap: "var(--space-1)",
              }}
            >
              <div>
                <strong>Checkpoint label:</strong>{" "}
                <span data-testid="rcps-summary-label">{result.summary.checkpointLabel}</span>
              </div>
              <div>
                <strong>Boundaries stored:</strong>{" "}
                <span data-testid="rcps-summary-boundary-count">
                  {result.summary.totalBoundaryCount}
                </span>
              </div>
              <div>
                <strong>Phases:</strong>{" "}
                <span data-testid="rcps-summary-phase-count">{result.summary.phaseCount}</span>
              </div>
              <div>
                <strong>Items covered:</strong>{" "}
                <span data-testid="rcps-summary-item-count">{result.summary.totalItemCount}</span>
              </div>
              <div>
                <strong>Checkpoint file:</strong>{" "}
                <span data-testid="rcps-summary-safe-filename">{result.summary.safeFilename}</span>
              </div>
              <p style={{ margin: 0, color: "var(--color-text-muted)" }}>{result.summary.note}</p>
            </div>
          )}

          <div style={{ display: "flex", gap: "var(--space-2)", flexWrap: "wrap" }}>
            <span
              className="badge badge-neutral"
              data-testid="rcps-no-changes-made"
              style={{ fontSize: "var(--text-xs)" }}
            >
              {result.noChangesMade ? "No metadata written" : "Metadata written (local only)"}
            </span>
            <span
              className="badge badge-neutral"
              data-testid="rcps-writes-disabled"
              style={{ fontSize: "var(--text-xs)" }}
            >
              Restore writes disabled
            </span>
          </div>
        </div>
      )}
    </div>
  );
}
