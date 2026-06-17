import { useState } from "react";
import type {
  FinalValidationExecutionPreviewRequest,
  FinalValidationExecutionPreviewResult,
} from "../../backend/types";

interface Props {
  request: FinalValidationExecutionPreviewRequest;
  onPreview: (
    request: FinalValidationExecutionPreviewRequest,
  ) => Promise<FinalValidationExecutionPreviewResult>;
  result: FinalValidationExecutionPreviewResult | null;
  loading: boolean;
}

export function RestoreFinalValidationExecutionPreviewPanel({
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
    <div data-testid="restore-fvep-panel">
      <div data-testid="fvep-execution-disabled-notice">
        Final validation execution disabled — preview only
      </div>

      <button
        data-testid="fvep-preview-button"
        onClick={handlePreview}
        disabled={isLoading}
        type="button"
      >
        {isLoading ? "Loading…" : "Preview Final Validation Execution"}
      </button>

      {result !== null && (
        <div data-testid="fvep-result">
          {result.status === "dryRunReady" ? (
            <span data-testid="fvep-dry-run-badge">Dry-run ready</span>
          ) : (
            <span data-testid="fvep-blocked-badge">Blocked</span>
          )}

          <span data-testid="fvep-execution-disabled-tag">Final validation execution disabled</span>

          <p data-testid="fvep-message">{result.message}</p>

          {result.blockedReason !== undefined && result.blockedReason !== null && (
            <p data-testid="fvep-blocked-reason">{result.blockedReason}</p>
          )}

          <div data-testid="fvep-summary">
            <span data-testid="fvep-total-check-count">{result.summary.totalCheckCount}</span>
            <span data-testid="fvep-pending-check-count">{result.summary.pendingCheckCount}</span>
            <span data-testid="fvep-table-count">{result.summary.tableCount}</span>
            <span data-testid="fvep-field-count">{result.summary.fieldCount}</span>
            <span data-testid="fvep-record-count">{result.summary.recordCount}</span>
            <span data-testid="fvep-id-mapping-entry-count">
              {result.summary.idMappingEntryCount}
            </span>
            <span data-testid="fvep-linked-coverage-count">
              {result.summary.linkedCoverageCount}
            </span>
            <span data-testid="fvep-attachment-metadata-count">
              {result.summary.attachmentMetadataCount}
            </span>
            <span data-testid="fvep-summary-note">{result.summary.note}</span>
          </div>

          <ul data-testid="fvep-checks">
            {result.checks.map((check) => (
              <li key={check.checkId} data-testid={`fvep-check-${check.checkId.toLowerCase()}`}>
                <span data-testid="fvep-check-label">{check.label}</span>
                <span data-testid="fvep-check-status">{check.status}</span>
                <span data-testid="fvep-check-expected-count">{check.expectedCount}</span>
                <span data-testid="fvep-check-note">{check.note}</span>
              </li>
            ))}
          </ul>

          <span data-testid="fvep-no-changes-made">
            {result.noChangesMade ? "No changes made" : ""}
          </span>
        </div>
      )}
    </div>
  );
}
