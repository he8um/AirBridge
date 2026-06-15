import type {
  WritePhaseOrderingPolicyResult,
  WritePhaseOrderingCheckStatus,
} from "../../backend/types";

interface Props {
  result: WritePhaseOrderingPolicyResult | null;
  loading: boolean;
  onVerify: () => void;
}

function checkBadge(status: WritePhaseOrderingCheckStatus): string {
  if (status === "passed") return "badge badge-success badge-sm";
  if (status === "warning") return "badge badge-warning badge-sm";
  return "badge badge-error badge-sm";
}

/**
 * Write Phase Ordering Policy Panel — Gate 12.
 *
 * Safety invariants:
 * - Always shows a disabled notice that restore writes remain unavailable.
 * - No execute button.
 * - No token input.
 * - No "succeeded" language.
 * - Compliant status does NOT imply writes are enabled.
 * - Compliant status does NOT introduce a restore success state.
 */
export function RestoreWritePhaseOrderingPolicyPanel({ result, loading, onVerify }: Props) {
  return (
    <div data-testid="restore-wpo-panel">
      <div data-testid="wpo-writes-disabled-notice" className="alert alert-warning mb-4">
        <span>
          Live restore writes are disabled. Verifying this policy does not enable writes or start
          any restore operation.
        </span>
      </div>

      <h3 className="font-semibold mb-2">Gate 12 — Write Phase Ordering Policy</h3>

      <button
        data-testid="wpo-verify-button"
        className="btn btn-primary mb-4"
        onClick={onVerify}
        disabled={loading}
      >
        {loading ? "Checking…" : "Verify write phase ordering policy"}
      </button>

      {result && (
        <div data-testid="wpo-result">
          <div className="flex gap-2 items-center mb-3">
            {result.status === "compliant" && (
              <span data-testid="wpo-compliant-badge" className="badge badge-success">
                Compliant
              </span>
            )}
            {result.status === "warning" && (
              <span data-testid="wpo-warning-badge" className="badge badge-warning">
                Warning
              </span>
            )}
            {result.status === "blocked" && (
              <span data-testid="wpo-blocked-badge" className="badge badge-error">
                Blocked
              </span>
            )}
            <span data-testid="wpo-message" className="text-sm">
              {result.message}
            </span>
          </div>

          {result.phaseSummary && result.phaseSummary.length > 0 && (
            <div data-testid="wpo-phase-summary" className="bg-base-200 rounded p-3 mb-3 text-sm">
              <p className="font-semibold mb-1">Declared write phase sequence</p>
              <table className="table table-xs w-full">
                <thead>
                  <tr>
                    <th>#</th>
                    <th>Phase</th>
                    <th>Status</th>
                    <th>Note</th>
                  </tr>
                </thead>
                <tbody>
                  {result.phaseSummary.map((entry) => (
                    <tr key={entry.kind} data-testid="wpo-phase-row">
                      <td className="font-mono text-xs">{entry.canonicalPosition}</td>
                      <td className="font-mono text-xs">{entry.kind}</td>
                      <td className="text-xs">{entry.status}</td>
                      <td className="text-xs">{entry.skipReason ?? "—"}</td>
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
                  <tr key={check.checkId} data-testid="wpo-check-row">
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

          <div data-testid="wpo-safety-summary" className="text-xs text-base-content/60 mt-2">
            <span data-testid="wpo-no-changes-notice">No changes made.</span>{" "}
            <span>Network writes not attempted.</span>{" "}
            <span>Writes enabled: {result.writesEnabled ? "yes" : "no"}.</span>
          </div>

          {result.status === "compliant" && (
            <div data-testid="wpo-compliant-notice" className="alert alert-success mt-3 text-sm">
              Write phase ordering is compliant. All phases are declared in canonical order with no
              unsafe transitions. Restore writes remain disabled — compliance does not start any
              write operation and does not introduce a restore success state.
            </div>
          )}
          {result.status === "warning" && (
            <div data-testid="wpo-warning-notice" className="alert alert-warning mt-3 text-sm">
              Write phase ordering has warnings. Review skipped or incomplete phases before
              proceeding. Restore writes remain disabled.
            </div>
          )}
          {result.status === "blocked" && (
            <div data-testid="wpo-blocked-notice" className="alert alert-error mt-3 text-sm">
              Write phase ordering is blocked. Resolve all phase ordering violations before any live
              write is considered. Restore writes remain disabled.
            </div>
          )}
        </div>
      )}
    </div>
  );
}
