import type {
  SchemaWriteExecutionPreviewResult,
  SchemaWriteExecutionPreviewStepStatus,
} from "../../backend/types";

interface Props {
  result: SchemaWriteExecutionPreviewResult | null;
  loading: boolean;
  onPreview: () => void;
}

function stepBadge(status: SchemaWriteExecutionPreviewStepStatus): string {
  if (status === "pending") return "badge badge-info badge-sm";
  if (status === "blocked") return "badge badge-error badge-sm";
  return "badge badge-ghost badge-sm";
}

/**
 * Schema Write Execution Preview Panel.
 *
 * Safety invariants:
 * - Always shows a notice that live schema writes remain disabled.
 * - DryRunReady does NOT enable live writes.
 * - No execute button.
 * - No enable-writes button.
 * - No token input.
 * - No full filesystem path.
 * - No record payload.
 * - No attachment URL.
 * - No raw HTTP request or response body.
 * - No success/completed wording except when explicitly stating it remains unavailable.
 * - writesEnabled is never shown as true.
 */
export function RestoreSchemaWriteExecutionPreviewPanel({ result, loading, onPreview }: Props) {
  return (
    <div data-testid="restore-swep-panel">
      <div data-testid="swep-writes-disabled-notice" className="alert alert-warning mb-4">
        <span>
          Live schema writes are disabled. Requesting this preview does not create, update, or
          delete any Airtable base, table, or field, and does not start any restore execution.
        </span>
      </div>

      <h3 className="font-semibold mb-2">Schema Write Execution Preview</h3>

      <button
        data-testid="swep-preview-button"
        className="btn btn-primary mb-4"
        onClick={onPreview}
        disabled={loading}
      >
        {loading ? "Loading…" : "Preview schema write execution"}
      </button>

      {result && (
        <div data-testid="swep-result">
          <div className="flex gap-2 items-center mb-3">
            {result.status === "dryRunReady" && (
              <span data-testid="swep-dry-run-badge" className="badge badge-info">
                Dry-run ready
              </span>
            )}
            {result.status === "blocked" && (
              <span data-testid="swep-blocked-badge" className="badge badge-error">
                Blocked
              </span>
            )}
            <span
              data-testid="swep-writes-disabled-tag"
              className="badge badge-outline badge-sm badge-warning"
            >
              Writes disabled
            </span>
          </div>

          <p data-testid="swep-message" className="text-sm mb-4">
            {result.message}
          </p>

          {result.blockedReason && (
            <div data-testid="swep-blocked-reason" className="alert alert-error mb-4 text-sm">
              <span>{result.blockedReason}</span>
            </div>
          )}

          {result.status === "dryRunReady" && (
            <div data-testid="swep-step-counts" className="mb-4 p-3 bg-base-200 rounded text-sm">
              <h4 className="font-semibold mb-2">Step Summary</h4>
              <ul className="space-y-1">
                <li>
                  <span className="font-medium">Tables:</span>{" "}
                  <span data-testid="swep-table-count">{result.tableStepCount}</span>
                </li>
                <li>
                  <span className="font-medium">Direct fields:</span>{" "}
                  <span data-testid="swep-field-count">{result.fieldStepCount}</span>
                </li>
                <li>
                  <span className="font-medium">Deferred linked fields:</span>{" "}
                  <span data-testid="swep-deferred-count">{result.deferredStepCount}</span>
                </li>
                <li>
                  <span className="font-medium">Manual actions:</span>{" "}
                  <span data-testid="swep-manual-count">{result.manualStepCount}</span>
                </li>
                <li>
                  <span className="font-medium">Total steps:</span>{" "}
                  <span data-testid="swep-total-count">{result.totalStepCount}</span>
                </li>
              </ul>
            </div>
          )}

          <div data-testid="swep-steps" className="space-y-2 mb-4">
            {result.steps.map((step) => (
              <div
                key={step.stepId}
                data-testid={`swep-step-${step.stepId.toLowerCase()}`}
                className="flex flex-col gap-1 p-2 border rounded"
              >
                <div className="flex gap-2 items-center">
                  <span className={stepBadge(step.status)}>{step.status}</span>
                  <span className="font-mono text-xs text-base-content/60">{step.stepId}</span>
                  <span className="text-sm font-medium">{step.label}</span>
                </div>
                <p className="text-xs text-base-content/70">{step.note}</p>
              </div>
            ))}
          </div>

          <div data-testid="swep-no-changes-made" className="mt-2 text-xs text-base-content/50">
            No changes made · No network writes attempted · Live schema writes disabled
          </div>
        </div>
      )}
    </div>
  );
}
