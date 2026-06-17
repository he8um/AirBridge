import { useState } from "react";
import type {
  RecordWriteExecutionPreviewRequest,
  RecordWriteExecutionPreviewResult,
} from "../../backend/types";

interface Props {
  request: RecordWriteExecutionPreviewRequest;
  onPreview: (
    request: RecordWriteExecutionPreviewRequest,
  ) => Promise<RecordWriteExecutionPreviewResult>;
  result: RecordWriteExecutionPreviewResult | null;
  loading: boolean;
}

export function RestoreRecordWriteExecutionPreviewPanel({
  request,
  onPreview,
  result,
  loading,
}: Props) {
  const [localLoading, setLocalLoading] = useState(false);

  const isLoading = loading || localLoading;

  async function handlePreview() {
    setLocalLoading(true);
    try {
      await onPreview(request);
    } finally {
      setLocalLoading(false);
    }
  }

  return (
    <div data-testid="restore-rwep-panel">
      <div data-testid="rwep-writes-disabled-notice">
        Live record writes disabled — preview only
      </div>

      <button
        data-testid="rwep-preview-button"
        onClick={handlePreview}
        disabled={isLoading}
        type="button"
      >
        {isLoading ? "Loading…" : "Preview Record Write Execution"}
      </button>

      {result !== null && (
        <div data-testid="rwep-result">
          {result.status === "dryRunReady" ? (
            <span data-testid="rwep-dry-run-badge">Dry-run ready</span>
          ) : (
            <span data-testid="rwep-blocked-badge">Blocked</span>
          )}

          <span data-testid="rwep-writes-disabled-tag">Live record writes disabled</span>

          <p data-testid="rwep-message">{result.message}</p>

          {result.blockedReason !== undefined && result.blockedReason !== null && (
            <p data-testid="rwep-blocked-reason">{result.blockedReason}</p>
          )}

          <div data-testid="rwep-batch-counts">
            <span data-testid="rwep-first-pass-count">{result.firstPassBatchCount}</span>
            <span data-testid="rwep-second-pass-count">{result.secondPassBatchCount}</span>
            <span data-testid="rwep-total-batch-count">{result.totalBatchCount}</span>
            <span data-testid="rwep-total-record-count">{result.totalRecordCount}</span>
            <span data-testid="rwep-batch-size">{result.batchSize}</span>
          </div>

          <ul data-testid="rwep-batches">
            {result.batches.map((batch) => (
              <li key={batch.batchId} data-testid={`rwep-batch-${batch.batchId.toLowerCase()}`}>
                {batch.tableLabel} — {batch.operationClass} — {batch.recordCount} record(s)
              </li>
            ))}
          </ul>

          <span data-testid="rwep-no-changes-made">
            {result.noChangesMade ? "No changes made" : ""}
          </span>
        </div>
      )}
    </div>
  );
}
