import type {
  RateLimitBackoffPolicyResult,
  RateLimitBackoffCheckStatus,
} from "../../backend/types";

interface Props {
  result: RateLimitBackoffPolicyResult | null;
  loading: boolean;
  onVerify: () => void;
}

function checkBadge(status: RateLimitBackoffCheckStatus): string {
  if (status === "passed") return "badge badge-success badge-sm";
  if (status === "warning") return "badge badge-warning badge-sm";
  return "badge badge-error badge-sm";
}

/**
 * Rate-Limit and Backoff Policy Panel — Gate 9.
 *
 * Safety invariants:
 * - Always shows a disabled notice that restore writes remain unavailable.
 * - No execute button.
 * - No token input.
 * - No "succeeded" language.
 * - Compliant status does NOT imply writes are enabled.
 */
export function RestoreRateLimitBackoffPolicyPanel({ result, loading, onVerify }: Props) {
  return (
    <div data-testid="restore-rlb-panel">
      <div data-testid="rlb-writes-disabled-notice" className="alert alert-warning mb-4">
        <span>
          Live restore writes are disabled. Verifying this policy does not enable writes or start
          any restore operation.
        </span>
      </div>

      <h3 className="font-semibold mb-2">Gate 9 — Rate-Limit and Backoff Policy</h3>

      <button
        data-testid="rlb-verify-button"
        className="btn btn-primary mb-4"
        onClick={onVerify}
        disabled={loading}
      >
        {loading ? "Checking…" : "Verify rate-limit policy"}
      </button>

      {result && (
        <div data-testid="rlb-result">
          <div className="flex gap-2 items-center mb-3">
            {result.status === "compliant" && (
              <span data-testid="rlb-compliant-badge" className="badge badge-success">
                Compliant
              </span>
            )}
            {result.status === "warning" && (
              <span data-testid="rlb-warning-badge" className="badge badge-warning">
                Warning
              </span>
            )}
            {result.status === "blocked" && (
              <span data-testid="rlb-blocked-badge" className="badge badge-error">
                Blocked
              </span>
            )}
            <span data-testid="rlb-message" className="text-sm">
              {result.message}
            </span>
          </div>

          {result.planSummary && (
            <div data-testid="rlb-plan-summary" className="bg-base-200 rounded p-3 mb-3 text-sm">
              <p className="font-semibold mb-1">Declared plan</p>
              <ul className="space-y-0.5 text-xs">
                <li data-testid="rlb-max-rps">
                  Max requests/sec: <strong>{result.planSummary.maxRequestsPerSecond}</strong>
                </li>
                <li data-testid="rlb-batch-size">
                  Batch size: <strong>{result.planSummary.batchSize}</strong>
                </li>
                <li data-testid="rlb-handles-429">
                  Handles 429: <strong>{result.planSummary.handles429 ? "yes" : "no"}</strong>
                </li>
                <li data-testid="rlb-max-retries">
                  Max retries:{" "}
                  <strong>
                    {result.planSummary.maxRetries !== undefined
                      ? result.planSummary.maxRetries
                      : "not declared"}
                  </strong>
                </li>
                <li data-testid="rlb-backoff-strategy">
                  Backoff strategy:{" "}
                  <strong>{result.planSummary.hasBackoffStrategy ? "declared" : "missing"}</strong>
                </li>
                <li data-testid="rlb-stop-condition">
                  Stop condition:{" "}
                  <strong>{result.planSummary.hasStopCondition ? "declared" : "missing"}</strong>
                </li>
                <li data-testid="rlb-checkpoint">
                  Checkpoint/resume:{" "}
                  <strong>{result.planSummary.checkpointCompatibility ?? "not declared"}</strong>
                </li>
              </ul>
            </div>
          )}

          {result.checks.length > 0 && (
            <table className="table table-xs w-full mb-3">
              <thead>
                <tr>
                  <th>Check</th>
                  <th>Status</th>
                  <th>Detail</th>
                </tr>
              </thead>
              <tbody>
                {result.checks.map((check) => (
                  <tr key={check.checkId} data-testid="rlb-check-row">
                    <td className="font-mono text-xs">{check.checkId}</td>
                    <td>
                      <span className={checkBadge(check.status)}>{check.status}</span>
                    </td>
                    <td className="text-xs">
                      {check.message}
                      {check.remediation && (
                        <span className="block text-warning mt-0.5">{check.remediation}</span>
                      )}
                    </td>
                  </tr>
                ))}
              </tbody>
            </table>
          )}

          <div data-testid="rlb-safety-summary" className="text-xs text-base-content/60 mt-2">
            <span data-testid="rlb-no-changes-notice">No changes made.</span>{" "}
            <span>Network writes not attempted.</span>{" "}
            <span>Writes enabled: {result.writesEnabled ? "yes" : "no"}.</span>
          </div>

          {result.status === "compliant" && (
            <div data-testid="rlb-compliant-notice" className="alert alert-success mt-3 text-sm">
              Rate-limit and backoff plan is within safe bounds. Restore writes remain disabled —
              compliance does not start any write operation.
            </div>
          )}
          {result.status === "warning" && (
            <div data-testid="rlb-warning-notice" className="alert alert-warning mt-3 text-sm">
              Rate-limit plan has warnings. Review incomplete fields before proceeding. Restore
              writes remain disabled.
            </div>
          )}
          {result.status === "blocked" && (
            <div data-testid="rlb-blocked-notice" className="alert alert-error mt-3 text-sm">
              Rate-limit plan is blocked. Resolve all issues before any live write is considered.
              Restore writes remain disabled.
            </div>
          )}
        </div>
      )}
    </div>
  );
}
