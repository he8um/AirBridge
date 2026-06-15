import type { FailureModesPolicyResult, FailureModesCheckStatus } from "../../backend/types";

interface Props {
  result: FailureModesPolicyResult | null;
  loading: boolean;
  onVerify: () => void;
}

function checkBadge(status: FailureModesCheckStatus): string {
  if (status === "passed") return "badge badge-success badge-sm";
  if (status === "warning") return "badge badge-warning badge-sm";
  return "badge badge-error badge-sm";
}

/**
 * Failure Modes Policy Panel — Gate 13.
 *
 * Safety invariants:
 * - Always shows a disabled notice that restore writes remain unavailable.
 * - No execute button.
 * - No token input.
 * - No "succeeded" language.
 * - Compliant status does NOT imply writes are enabled.
 * - Compliant status does NOT introduce a restore success state.
 */
export function RestoreFailureModesPolicyPanel({ result, loading, onVerify }: Props) {
  return (
    <div data-testid="restore-fmp-panel">
      <div data-testid="fmp-writes-disabled-notice" className="alert alert-warning mb-4">
        <span>
          Live restore writes are disabled. Verifying this policy does not enable writes or start
          any restore operation.
        </span>
      </div>

      <h3 className="font-semibold mb-2">Gate 13 — Failure Modes Policy</h3>

      <button
        data-testid="fmp-verify-button"
        className="btn btn-primary mb-4"
        onClick={onVerify}
        disabled={loading}
      >
        {loading ? "Checking…" : "Verify failure modes policy"}
      </button>

      {result && (
        <div data-testid="fmp-result">
          <div className="flex gap-2 items-center mb-3">
            {result.status === "compliant" && (
              <span data-testid="fmp-compliant-badge" className="badge badge-success">
                Compliant
              </span>
            )}
            {result.status === "warning" && (
              <span data-testid="fmp-warning-badge" className="badge badge-warning">
                Warning
              </span>
            )}
            {result.status === "blocked" && (
              <span data-testid="fmp-blocked-badge" className="badge badge-error">
                Blocked
              </span>
            )}
            <span data-testid="fmp-message" className="text-sm">
              {result.message}
            </span>
          </div>

          {result.handlingSummary && result.handlingSummary.length > 0 && (
            <div
              data-testid="fmp-handling-summary"
              className="bg-base-200 rounded p-3 mb-3 text-sm"
            >
              <p className="font-semibold mb-1">Declared failure mode stop behaviors</p>
              <table className="table table-xs w-full">
                <thead>
                  <tr>
                    <th>Failure Mode</th>
                    <th>Stop Behavior</th>
                    <th>Preserves Checkpoint</th>
                    <th>Captures Diagnostics</th>
                  </tr>
                </thead>
                <tbody>
                  {result.handlingSummary.map((entry) => (
                    <tr key={entry.mode} data-testid="fmp-mode-row">
                      <td className="font-mono text-xs">{entry.mode}</td>
                      <td className="font-mono text-xs">{entry.stopBehavior}</td>
                      <td className="text-xs">{entry.preservesCheckpoint ? "yes" : "no"}</td>
                      <td className="text-xs">{entry.capturesDiagnosticContext ? "yes" : "no"}</td>
                    </tr>
                  ))}
                </tbody>
              </table>
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
                  <tr key={check.checkId} data-testid="fmp-check-row">
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

          <div data-testid="fmp-safety-summary" className="text-xs text-base-content/60 mt-2">
            <span data-testid="fmp-no-changes-notice">No changes made.</span>{" "}
            <span>Network writes not attempted.</span>{" "}
            <span>Writes enabled: {result.writesEnabled ? "yes" : "no"}.</span>
          </div>

          {result.status === "compliant" && (
            <div data-testid="fmp-compliant-notice" className="alert alert-success mt-3 text-sm">
              Failure modes policy is compliant. All required failure modes have explicit, safe
              stop-behavior declarations. Restore writes remain disabled — compliance does not start
              any write operation and does not introduce a restore success state.
            </div>
          )}
          {result.status === "warning" && (
            <div data-testid="fmp-warning-notice" className="alert alert-warning mt-3 text-sm">
              Failure modes policy has warnings. Review modes with incomplete diagnostic context
              before proceeding. Restore writes remain disabled.
            </div>
          )}
          {result.status === "blocked" && (
            <div data-testid="fmp-blocked-notice" className="alert alert-error mt-3 text-sm">
              Failure modes policy is blocked. Resolve all violations before any live write is
              considered. Restore writes remain disabled.
            </div>
          )}
        </div>
      )}
    </div>
  );
}
