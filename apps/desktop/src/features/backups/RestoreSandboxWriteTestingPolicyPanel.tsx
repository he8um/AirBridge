import type {
  SandboxWriteTestingPolicyResult,
  SandboxWriteTestingCheckStatus,
} from "../../backend/types";

interface Props {
  result: SandboxWriteTestingPolicyResult | null;
  loading: boolean;
  onVerify: () => void;
}

function checkBadge(status: SandboxWriteTestingCheckStatus): string {
  if (status === "passed") return "badge badge-success badge-sm";
  if (status === "warning") return "badge badge-warning badge-sm";
  return "badge badge-error badge-sm";
}

/**
 * Sandbox Write Testing Policy Panel — Gate 7.
 *
 * Safety invariants:
 * - Always shows a disabled notice that restore writes remain unavailable.
 * - No execute button.
 * - No token input.
 * - No "succeeded" language.
 * - Compliant status does NOT imply writes are enabled.
 */
export function RestoreSandboxWriteTestingPolicyPanel({ result, loading, onVerify }: Props) {
  return (
    <div data-testid="restore-swt-panel">
      <h3 className="font-semibold text-base mb-1">Sandbox Write Testing Policy (Gate 7)</h3>

      <div
        role="note"
        aria-label="Sandbox write testing policy writes disabled notice"
        data-testid="swt-writes-disabled-notice"
        className="alert alert-info mb-3 text-sm"
      >
        Sandbox write testing policy checks that sandbox testing evidence has been recorded. Restore
        writes remain disabled — no Airtable changes will be made.
      </div>

      <button
        className="btn btn-sm btn-outline mb-3"
        data-testid="swt-verify-button"
        disabled={loading}
        onClick={onVerify}
        aria-label={
          result !== null
            ? "Re-verify sandbox write testing policy"
            : "Verify sandbox write testing policy"
        }
      >
        {loading ? "Checking…" : result !== null ? "Re-verify" : "Verify sandbox testing"}
      </button>

      {result !== null && (
        <div data-testid="swt-result">
          <div className="flex items-center gap-2 mb-2">
            {result.status === "compliant" && (
              <span data-testid="swt-compliant-badge" className="badge badge-success">
                compliant
              </span>
            )}
            {result.status === "warning" && (
              <span data-testid="swt-warning-badge" className="badge badge-warning">
                warning
              </span>
            )}
            {result.status === "blocked" && (
              <span data-testid="swt-blocked-badge" className="badge badge-error">
                blocked
              </span>
            )}
          </div>

          <p data-testid="swt-message" className="text-sm mb-3">
            {result.message}
          </p>

          {result.checks.length > 0 && (
            <table className="table table-xs mb-3 w-full">
              <thead>
                <tr>
                  <th>ID</th>
                  <th>Label</th>
                  <th>Status</th>
                  <th>Message</th>
                </tr>
              </thead>
              <tbody>
                {result.checks.map((check) => (
                  <tr key={check.checkId} data-testid="swt-check-row">
                    <td>{check.checkId}</td>
                    <td>{check.label}</td>
                    <td>
                      <span className={checkBadge(check.status)}>{check.status}</span>
                    </td>
                    <td>{check.message}</td>
                  </tr>
                ))}
              </tbody>
            </table>
          )}

          <div data-testid="swt-safety-summary" className="text-xs text-base-content/60 mb-2">
            <span>noChangesMade: {String(result.noChangesMade)}</span>
            {" · "}
            <span>writesEnabled: {String(result.writesEnabled)}</span>
            {" · "}
            <span>networkWritesAttempted: {String(result.networkWritesAttempted)}</span>
          </div>

          <div
            role="note"
            data-testid="swt-no-changes-notice"
            className="alert alert-info text-xs mb-2"
          >
            No changes have been made to Airtable.
          </div>

          {result.status === "compliant" && (
            <div
              role="note"
              data-testid="swt-compliant-notice"
              className="alert alert-success text-xs mb-2"
            >
              Sandbox write testing policy satisfied. Restore writes remain disabled — compliant
              status does not enable live writes.
            </div>
          )}

          {result.status === "warning" && (
            <div
              role="note"
              data-testid="swt-warning-notice"
              className="alert alert-warning text-xs mb-2"
            >
              Sandbox write testing evidence is incomplete or partial. Resolve warnings before
              proceeding with live write testing.
            </div>
          )}

          {result.status === "blocked" && (
            <div
              role="note"
              data-testid="swt-blocked-notice"
              className="alert alert-error text-xs mb-2"
            >
              Sandbox write testing policy is blocked. Target must be a sandbox base and all
              required evidence must be present before live write testing.
            </div>
          )}
        </div>
      )}
    </div>
  );
}
