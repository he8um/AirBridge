import { useState } from "react";
import type {
  LinkedSecondPassExecutionPreviewRequest,
  LinkedSecondPassExecutionPreviewResult,
} from "../../backend/types";

interface Props {
  request: LinkedSecondPassExecutionPreviewRequest;
  onPreview: (
    request: LinkedSecondPassExecutionPreviewRequest,
  ) => Promise<LinkedSecondPassExecutionPreviewResult>;
  result: LinkedSecondPassExecutionPreviewResult | null;
  loading: boolean;
}

export function RestoreLinkedSecondPassExecutionPreviewPanel({
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
    <div data-testid="restore-lsep-panel">
      <div data-testid="lsep-execution-disabled-notice">
        Linked record update execution disabled — preview only
      </div>

      <button
        data-testid="lsep-preview-button"
        onClick={handlePreview}
        disabled={isLoading}
        type="button"
      >
        {isLoading ? "Loading…" : "Preview Linked Second-Pass Execution"}
      </button>

      {result !== null && (
        <div data-testid="lsep-result">
          {result.status === "dryRunReady" ? (
            <span data-testid="lsep-dry-run-badge">Dry-run ready</span>
          ) : (
            <span data-testid="lsep-blocked-badge">Blocked</span>
          )}

          <span data-testid="lsep-execution-disabled-tag">
            Linked record update execution disabled
          </span>

          <p data-testid="lsep-message">{result.message}</p>

          {result.blockedReason !== undefined && result.blockedReason !== null && (
            <p data-testid="lsep-blocked-reason">{result.blockedReason}</p>
          )}

          <div data-testid="lsep-mapping-summary">
            <span data-testid="lsep-total-update-count">
              {result.mappingSummary.totalUpdateCount}
            </span>
            <span data-testid="lsep-tables-with-linked-fields">
              {result.mappingSummary.tablesWithLinkedFields}
            </span>
            <span data-testid="lsep-total-linked-fields">
              {result.mappingSummary.totalLinkedFields}
            </span>
            <span data-testid="lsep-unresolved-link-count">
              {result.mappingSummary.unresolvedLinkCount}
            </span>
            <span data-testid="lsep-mapping-note">{result.mappingSummary.note}</span>
          </div>

          <div data-testid="lsep-field-summary">
            {result.fieldSummaries.map((f, i) => (
              <div
                key={`${f.tableLabel}-${f.fieldLabel}-${i}`}
                data-testid={`lsep-field-${f.fieldLabel.toLowerCase().replace(/ /g, "-")}`}
              >
                <span data-testid="lsep-field-table-label">{f.tableLabel}</span>
                <span data-testid="lsep-field-label">{f.fieldLabel}</span>
                <span data-testid="lsep-field-record-count">{f.recordCount}</span>
              </div>
            ))}
          </div>

          <div data-testid="lsep-batch-summary">
            <span data-testid="lsep-total-batch-count">{result.totalBatchCount}</span>
            <ul data-testid="lsep-batches">
              {result.batches.map((batch) => (
                <li key={batch.batchId} data-testid={`lsep-batch-${batch.batchId.toLowerCase()}`}>
                  {batch.tableLabel} / {batch.fieldLabel} — {batch.updateCount} record(s)
                </li>
              ))}
            </ul>
          </div>

          <span data-testid="lsep-no-changes-made">
            {result.noChangesMade ? "No changes made" : ""}
          </span>
        </div>
      )}
    </div>
  );
}
