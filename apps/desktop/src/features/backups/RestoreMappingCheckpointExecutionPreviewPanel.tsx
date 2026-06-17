import { useState } from "react";
import type {
  MappingCheckpointExecutionPreviewRequest,
  MappingCheckpointExecutionPreviewResult,
} from "../../backend/types";

interface Props {
  request: MappingCheckpointExecutionPreviewRequest;
  onPreview: (
    request: MappingCheckpointExecutionPreviewRequest,
  ) => Promise<MappingCheckpointExecutionPreviewResult>;
  result: MappingCheckpointExecutionPreviewResult | null;
  loading: boolean;
}

export function RestoreMappingCheckpointExecutionPreviewPanel({
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
    <div data-testid="restore-mcep-panel">
      <div data-testid="mcep-execution-disabled-notice">
        Mapping and checkpoint execution disabled — preview only
      </div>

      <button
        data-testid="mcep-preview-button"
        onClick={handlePreview}
        disabled={isLoading}
        type="button"
      >
        {isLoading ? "Loading…" : "Preview Mapping & Checkpoint Execution"}
      </button>

      {result !== null && (
        <div data-testid="mcep-result">
          {result.status === "dryRunReady" ? (
            <span data-testid="mcep-dry-run-badge">Dry-run ready</span>
          ) : (
            <span data-testid="mcep-blocked-badge">Blocked</span>
          )}

          <span data-testid="mcep-execution-disabled-tag">
            Mapping and checkpoint execution disabled
          </span>

          <p data-testid="mcep-message">{result.message}</p>

          {result.blockedReason !== undefined && result.blockedReason !== null && (
            <p data-testid="mcep-blocked-reason">{result.blockedReason}</p>
          )}

          <div data-testid="mcep-id-mapping-summary">
            <span data-testid="mcep-total-mapping-count">
              {result.idMappingSummary.totalMappingCount}
            </span>
            <span data-testid="mcep-tables-requiring-remapping">
              {result.idMappingSummary.tablesRequiringRemapping}
            </span>
            <span data-testid="mcep-mapping-first-pass-count">
              {result.idMappingSummary.firstPassBatchCount}
            </span>
            <span data-testid="mcep-mapping-note">{result.idMappingSummary.note}</span>
          </div>

          <div data-testid="mcep-checkpoint-summary">
            <span data-testid="mcep-total-checkpoint-count">
              {result.checkpointSummary.totalCheckpointCount}
            </span>
            <span data-testid="mcep-record-create-checkpoint-count">
              {result.checkpointSummary.recordCreateCheckpointCount}
            </span>
            <span data-testid="mcep-linked-update-checkpoint-count">
              {result.checkpointSummary.linkedUpdateCheckpointCount}
            </span>
            <span data-testid="mcep-checkpoint-note">{result.checkpointSummary.note}</span>
          </div>

          <ul data-testid="mcep-steps">
            {result.steps.map((step) => (
              <li key={step.stepId} data-testid={`mcep-step-${step.stepId.toLowerCase()}`}>
                {step.phaseLabel} — {step.checkpointBoundaryLabel}
              </li>
            ))}
          </ul>

          <span data-testid="mcep-no-changes-made">
            {result.noChangesMade ? "No changes made" : ""}
          </span>
        </div>
      )}
    </div>
  );
}
