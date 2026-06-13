import type {
  DestructiveOperationPolicyResult,
  DestructiveOperationCheckStatus,
} from "../../backend/types";

interface Props {
  result: DestructiveOperationPolicyResult | null;
  loading: boolean;
  onVerify: () => void;
}

function checkBadge(status: DestructiveOperationCheckStatus): string {
  if (status === "passed") return "badge badge-success badge-sm";
  if (status === "warning") return "badge badge-warning badge-sm";
  return "badge badge-error badge-sm";
}

/**
 * Destructive Operation Policy Panel — Gate 4.
 *
 * Safety invariants:
 * - Always shows a disabled notice that restore writes remain unavailable.
 * - No execute button.
 * - No token input.
 * - No "succeeded" language.
 * - Compliant status does NOT imply writes are enabled.
 */
export function RestoreDestructiveOperationPolicyPanel({ result, loading, onVerify }: Props) {
  return (
    <div data-testid="restore-dop-panel">
      <h3 className="font-semibold text-base mb-1">Destructive Operation Policy (Gate 4)</h3>

      <div
        role="note"
        aria-label="Destructive operation policy writes disabled notice"
        data-testid="dop-writes-disabled-notice"
        className="alert alert-info mb-3 text-sm"
      >
        Destructive operation policy checks declared operations only. Restore writes remain disabled
        — no Airtable changes will be made.
      </div>

      <button
        className="btn btn-sm btn-outline mb-3"
        data-testid="dop-verify-button"
        disabled={loading}
        onClick={onVerify}
        aria-label={
          result !== null
            ? "Re-verify destructive operation policy"
            : "Verify destructive operation policy"
        }
      >
        {loading ? "Checking…" : result !== null ? "Re-verify" : "Verify operation policy"}
      </button>

      {result === null && !loading && null}

      {result !== null && (
        <div data-testid="dop-result">
          <div className="flex items-center gap-2 mb-2">
            <span data-testid="dop-status">
              {result.status === "compliant" && (
                <span className="badge badge-success" data-testid="dop-compliant-badge">
                  compliant
                </span>
              )}
              {result.status === "warning" && (
                <span className="badge badge-warning" data-testid="dop-warning-badge">
                  warning
                </span>
              )}
              {result.status === "blocked" && (
                <span className="badge badge-error" data-testid="dop-blocked-badge">
                  blocked
                </span>
              )}
            </span>
            <span className="text-sm" data-testid="dop-message">
              {result.message}
            </span>
          </div>

          {result.blockedOperations.length > 0 && (
            <div className="text-sm text-error mb-2" data-testid="dop-blocked-ops-list">
              Blocked operations:{" "}
              {result.blockedOperations.map((op, i) => (
                <span key={i} className="font-mono mr-1" data-testid="dop-blocked-op-item">
                  {op}
                </span>
              ))}
            </div>
          )}

          <div className="overflow-x-auto mb-2" data-testid="dop-checks">
            <table className="table table-xs">
              <thead>
                <tr>
                  <th>ID</th>
                  <th>Check</th>
                  <th>Status</th>
                  <th>Message</th>
                </tr>
              </thead>
              <tbody>
                {result.checks.map((check) => (
                  <tr key={check.checkId} data-testid="dop-check-row">
                    <td className="font-mono text-xs">{check.checkId}</td>
                    <td>{check.label}</td>
                    <td>
                      <span className={checkBadge(check.status)}>{check.status}</span>
                    </td>
                    <td className="text-xs">{check.message}</td>
                  </tr>
                ))}
              </tbody>
            </table>
          </div>

          <div className="text-xs text-base-content/60 mt-1" data-testid="dop-safety-summary">
            noChangesMade: {result.noChangesMade ? "Yes" : "No"} &nbsp;|&nbsp; writesEnabled:{" "}
            {result.writesEnabled ? "Yes" : "No"} &nbsp;|&nbsp; networkWritesAttempted:{" "}
            {result.networkWritesAttempted ? "Yes" : "No"}
          </div>

          <div className="text-xs mt-1" data-testid="dop-no-changes-notice">
            No changes have been made to Airtable.
          </div>

          {result.status === "compliant" && (
            <div className="alert alert-success text-sm mt-2" data-testid="dop-compliant-notice">
              All operations are create-only. Restore writes remain disabled — no Airtable changes
              will be made.
            </div>
          )}

          {result.status === "warning" && (
            <div className="alert alert-warning text-sm mt-2" data-testid="dop-warning-notice">
              Some operations could not be classified. Manual review is required before enabling
              live writes.
            </div>
          )}

          {result.status === "blocked" && (
            <div className="alert alert-error text-sm mt-2" data-testid="dop-blocked-notice">
              Destructive operations detected. Remove all blocked operations before proceeding.
            </div>
          )}
        </div>
      )}
    </div>
  );
}
