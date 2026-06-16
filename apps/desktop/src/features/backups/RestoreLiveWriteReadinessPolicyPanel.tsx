import type {
  LiveWriteReadinessPolicyResult,
  LiveWriteReadinessCheckStatus,
} from "../../backend/types";

interface Props {
  result: LiveWriteReadinessPolicyResult | null;
  loading: boolean;
  onVerify: () => void;
}

function checkBadge(status: LiveWriteReadinessCheckStatus): string {
  if (status === "passed") return "badge badge-success badge-sm";
  if (status === "warning") return "badge badge-warning badge-sm";
  return "badge badge-error badge-sm";
}

/**
 * Live Write Readiness Policy Panel — Gate 18.
 *
 * Safety invariants:
 * - Always shows a disabled notice that restore writes remain unavailable.
 * - Always shows an advisory-only notice.
 * - No execute button.
 * - No enable-writes button.
 * - No token input.
 * - No path/package-path display.
 * - No record payload display.
 * - No attachment URL display.
 * - No success/completed wording except when stating it remains blocked or unavailable.
 * - Ready status does NOT imply writes are enabled.
 * - Ready status does NOT introduce a restore success state.
 * - This panel is advisory only.
 */
export function RestoreLiveWriteReadinessPolicyPanel({ result, loading, onVerify }: Props) {
  return (
    <div data-testid="restore-lwr-panel">
      <div data-testid="lwr-writes-disabled-notice" className="alert alert-warning mb-4">
        <span>
          Live restore writes are disabled. Verifying this policy does not enable writes, does not
          start any restore operation, and does not introduce a restore success state.
        </span>
      </div>

      <div data-testid="lwr-advisory-only-notice" className="alert alert-info mb-4">
        <span>
          This is an advisory readiness check only. A Ready result confirms that all 17 required
          safety gates are declared and none are failed — it does not enable write execution.
          Restore completion remains unavailable.
        </span>
      </div>

      <h3 className="font-semibold mb-2">Gate 18 — Live Write Readiness Policy</h3>

      <button
        data-testid="lwr-verify-button"
        className="btn btn-primary mb-4"
        onClick={onVerify}
        disabled={loading}
      >
        {loading ? "Checking…" : "Verify live write readiness"}
      </button>

      {result && (
        <div data-testid="lwr-result">
          <div className="flex gap-2 items-center mb-3">
            {result.status === "ready" && (
              <span data-testid="lwr-ready-badge" className="badge badge-success">
                Ready (advisory only)
              </span>
            )}
            {result.status === "warning" && (
              <span data-testid="lwr-warning-badge" className="badge badge-warning">
                Warning
              </span>
            )}
            {result.status === "blocked" && (
              <span data-testid="lwr-blocked-badge" className="badge badge-error">
                Blocked
              </span>
            )}
            <span data-testid="lwr-writes-disabled-tag" className="badge badge-outline badge-sm">
              Writes disabled
            </span>
            <span
              data-testid="lwr-advisory-tag"
              className="badge badge-outline badge-sm badge-info"
            >
              Advisory only
            </span>
          </div>

          <p data-testid="lwr-message" className="text-sm mb-4">
            {result.message}
          </p>

          {result.gateSummary && (
            <div data-testid="lwr-gate-summary" className="mb-4 p-3 bg-base-200 rounded text-sm">
              <h4 className="font-semibold mb-2">Gate Summary</h4>
              <ul className="space-y-1">
                <li>
                  <span className="font-medium">Total required gates:</span>{" "}
                  <span data-testid="lwr-summary-total">{result.gateSummary.totalGates}</span>
                </li>
                <li>
                  <span className="font-medium">Passed:</span>{" "}
                  <span data-testid="lwr-summary-passed">{result.gateSummary.passedGates}</span>
                </li>
                <li>
                  <span className="font-medium">Warning:</span>{" "}
                  <span data-testid="lwr-summary-warning">{result.gateSummary.warningGates}</span>
                </li>
                <li>
                  <span className="font-medium">Failed:</span>{" "}
                  <span data-testid="lwr-summary-failed">{result.gateSummary.failedGates}</span>
                </li>
                <li>
                  <span className="font-medium">Not evaluated:</span>{" "}
                  <span data-testid="lwr-summary-not-evaluated">
                    {result.gateSummary.notEvaluatedGates}
                  </span>
                </li>
                <li>
                  <span className="font-medium">Missing required gates:</span>{" "}
                  <span data-testid="lwr-summary-missing">
                    {result.gateSummary.missingRequiredGates}
                  </span>
                </li>
                <li>
                  <span className="font-medium">All required gates declared:</span>{" "}
                  <span data-testid="lwr-summary-all-declared">
                    {result.gateSummary.allRequiredGatesDeclared ? "Yes" : "No"}
                  </span>
                </li>
                <li>
                  <span className="font-medium">Live execution available:</span>{" "}
                  <span data-testid="lwr-summary-live-execution">
                    {result.gateSummary.liveExecutionAvailable ? "Yes" : "No"}
                  </span>
                </li>
              </ul>
            </div>
          )}

          <div data-testid="lwr-checks" className="space-y-2 mb-4">
            {result.checks.map((check) => (
              <div
                key={check.checkId}
                data-testid={`lwr-check-${check.checkId.toLowerCase()}`}
                className="flex flex-col gap-1 p-2 border rounded"
              >
                <div className="flex gap-2 items-center">
                  <span className={checkBadge(check.status)}>{check.status}</span>
                  <span className="font-mono text-xs text-base-content/60">{check.checkId}</span>
                  <span className="text-sm font-medium">{check.label}</span>
                </div>
                <p className="text-xs text-base-content/70">{check.message}</p>
                {check.remediation && (
                  <p
                    data-testid={`lwr-remediation-${check.checkId.toLowerCase()}`}
                    className="text-xs text-warning"
                  >
                    {check.remediation}
                  </p>
                )}
              </div>
            ))}
          </div>

          {result.status === "ready" && (
            <div data-testid="lwr-gate-table" className="mb-4 p-3 bg-base-200 rounded text-sm">
              <h4 className="font-semibold mb-2">Required Gate Status</h4>
              <p className="text-xs text-base-content/60 mb-2">
                All 17 required safety gates declared. Restore execution remains unavailable.
              </p>
            </div>
          )}

          <div data-testid="lwr-no-changes-made" className="mt-2 text-xs text-base-content/50">
            No changes made · No network writes attempted · Writes disabled · Advisory only
          </div>
        </div>
      )}
    </div>
  );
}
