import type {
  SchemaRecordOrderPolicyResult,
  SchemaRecordOrderCheckStatus,
} from "../../backend/types";

interface Props {
  result: SchemaRecordOrderPolicyResult | null;
  loading: boolean;
  onVerify: () => void;
}

function checkBadge(status: SchemaRecordOrderCheckStatus): string {
  if (status === "passed") return "badge badge-success badge-sm";
  if (status === "warning") return "badge badge-warning badge-sm";
  return "badge badge-error badge-sm";
}

/**
 * Schema Record Order Policy Panel — Gate 6.
 *
 * Safety invariants:
 * - Always shows a disabled notice that restore writes remain unavailable.
 * - No execute button.
 * - No token input.
 * - No "succeeded" language.
 * - Compliant status does NOT imply writes are enabled.
 */
export function RestoreSchemaRecordOrderPolicyPanel({ result, loading, onVerify }: Props) {
  return (
    <div data-testid="restore-sro-panel">
      <h3 className="font-semibold text-base mb-1">Schema Record Order Policy (Gate 6)</h3>

      <div
        role="note"
        aria-label="Schema record order policy writes disabled notice"
        data-testid="sro-writes-disabled-notice"
        className="alert alert-info mb-3 text-sm"
      >
        Schema record order policy checks declared write phase ordering only. Restore writes remain
        disabled — no Airtable changes will be made.
      </div>

      <button
        className="btn btn-sm btn-outline mb-3"
        data-testid="sro-verify-button"
        disabled={loading}
        onClick={onVerify}
        aria-label={
          result !== null
            ? "Re-verify schema record order policy"
            : "Verify schema record order policy"
        }
      >
        {loading ? "Checking…" : result !== null ? "Re-verify" : "Verify phase ordering"}
      </button>

      {result === null && !loading && null}

      {result !== null && (
        <div data-testid="sro-result">
          <div className="flex items-center gap-2 mb-2">
            <span data-testid="sro-status">
              {result.status === "compliant" && (
                <span className="badge badge-success" data-testid="sro-compliant-badge">
                  compliant
                </span>
              )}
              {result.status === "warning" && (
                <span className="badge badge-warning" data-testid="sro-warning-badge">
                  warning
                </span>
              )}
              {result.status === "blocked" && (
                <span className="badge badge-error" data-testid="sro-blocked-badge">
                  blocked
                </span>
              )}
            </span>
            <span className="text-sm" data-testid="sro-message">
              {result.message}
            </span>
          </div>

          {result.orderingViolations.length > 0 && (
            <div className="text-sm text-error mb-2" data-testid="sro-violations-list">
              Ordering violations:{" "}
              {result.orderingViolations.map((v, i) => (
                <span key={i} className="font-mono mr-1" data-testid="sro-violation-item">
                  {v}
                </span>
              ))}
            </div>
          )}

          <div className="overflow-x-auto mb-2" data-testid="sro-checks">
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
                  <tr key={check.checkId} data-testid="sro-check-row">
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

          <div className="text-xs text-base-content/60 mt-1" data-testid="sro-safety-summary">
            noChangesMade: {result.noChangesMade ? "Yes" : "No"} &nbsp;|&nbsp; writesEnabled:{" "}
            {result.writesEnabled ? "Yes" : "No"} &nbsp;|&nbsp; networkWritesAttempted:{" "}
            {result.networkWritesAttempted ? "Yes" : "No"}
          </div>

          <div className="text-xs mt-1" data-testid="sro-no-changes-notice">
            No changes have been made to Airtable.
          </div>

          {result.status === "compliant" && (
            <div className="alert alert-success text-sm mt-2" data-testid="sro-compliant-notice">
              Phase ordering is valid. Restore writes remain disabled — no Airtable changes will be
              made.
            </div>
          )}

          {result.status === "warning" && (
            <div className="alert alert-warning text-sm mt-2" data-testid="sro-warning-notice">
              Phase ordering could not be fully verified. Provide complete phase data before
              enabling live writes.
            </div>
          )}

          {result.status === "blocked" && (
            <div className="alert alert-error text-sm mt-2" data-testid="sro-blocked-notice">
              Phase ordering violation detected. Resolve all ordering issues before proceeding.
            </div>
          )}
        </div>
      )}
    </div>
  );
}
