import { useState } from "react";
import type {
  LiveWriteConfirmationPolicyResult,
  LiveWriteConfirmationCheckStatus,
} from "../../backend/types";

interface Props {
  result: LiveWriteConfirmationPolicyResult | null;
  loading: boolean;
  requiredText: string;
  onVerify: (enteredText: string) => void;
}

function checkBadge(status: LiveWriteConfirmationCheckStatus): string {
  if (status === "passed") return "badge badge-success badge-sm";
  if (status === "warning") return "badge badge-warning badge-sm";
  return "badge badge-error badge-sm";
}

/**
 * Live Write Confirmation Policy Panel — Gate 8.
 *
 * Safety invariants:
 * - Always shows a disabled notice that restore writes remain unavailable.
 * - No execute button.
 * - No token input.
 * - No "succeeded" language.
 * - Confirmed status does NOT imply writes are enabled.
 */
export function RestoreLiveWriteConfirmationPolicyPanel({
  result,
  loading,
  requiredText,
  onVerify,
}: Props) {
  const [enteredText, setEnteredText] = useState("");

  return (
    <div data-testid="restore-lwc-panel">
      <div data-testid="lwc-writes-disabled-notice" className="alert alert-warning mb-4">
        <span>
          Live restore writes are disabled. Confirming this phrase does not enable writes or
          start any restore operation.
        </span>
      </div>

      <h3 className="font-semibold mb-2">Gate 8 — Live Write Confirmation</h3>

      {requiredText && (
        <div className="mb-3">
          <p className="text-sm mb-1">Type exactly to confirm:</p>
          <code
            data-testid="lwc-required-text"
            className="block bg-base-200 px-3 py-2 rounded text-sm font-mono select-all"
          >
            {requiredText}
          </code>
        </div>
      )}

      <div className="flex gap-2 mb-4">
        <input
          data-testid="lwc-confirmation-input"
          type="text"
          className="input input-bordered flex-1 font-mono text-sm"
          placeholder="Type the required phrase above"
          value={enteredText}
          onChange={(e) => setEnteredText(e.target.value)}
          disabled={loading}
          aria-label="Confirmation phrase"
          autoComplete="off"
          spellCheck={false}
        />
        <button
          data-testid="lwc-verify-button"
          className="btn btn-primary"
          onClick={() => onVerify(enteredText)}
          disabled={loading || enteredText.trim() === ""}
        >
          {loading ? "Checking…" : "Verify"}
        </button>
      </div>

      {result && (
        <div data-testid="lwc-result">
          <div className="flex gap-2 items-center mb-3">
            {result.status === "confirmed" && (
              <span data-testid="lwc-confirmed-badge" className="badge badge-success">
                Confirmed
              </span>
            )}
            {result.status === "warning" && (
              <span data-testid="lwc-warning-badge" className="badge badge-warning">
                Warning
              </span>
            )}
            {result.status === "blocked" && (
              <span data-testid="lwc-blocked-badge" className="badge badge-error">
                Blocked
              </span>
            )}
            {result.status === "rejected" && (
              <span data-testid="lwc-rejected-badge" className="badge badge-error">
                Rejected
              </span>
            )}
            <span data-testid="lwc-message" className="text-sm">
              {result.message}
            </span>
          </div>

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
                  <tr key={check.checkId} data-testid="lwc-check-row">
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

          <div data-testid="lwc-safety-summary" className="text-xs text-base-content/60 mt-2">
            <span data-testid="lwc-no-changes-notice">No changes made.</span>{" "}
            <span>Network writes not attempted.</span>{" "}
            <span>Writes enabled: {result.writesEnabled ? "yes" : "no"}.</span>
          </div>

          {result.status === "confirmed" && (
            <div data-testid="lwc-confirmed-notice" className="alert alert-success mt-3 text-sm">
              Phrase accepted. Restore writes remain disabled — confirmation does not start any
              write operation.
            </div>
          )}
          {result.status === "warning" && (
            <div data-testid="lwc-warning-notice" className="alert alert-warning mt-3 text-sm">
              Phrase accepted with warnings. Review prior gate warnings before proceeding. Restore
              writes remain disabled.
            </div>
          )}
          {result.status === "blocked" && (
            <div data-testid="lwc-blocked-notice" className="alert alert-error mt-3 text-sm">
              Confirmation blocked. Resolve blocked prior gates before attempting confirmation.
              Restore writes remain disabled.
            </div>
          )}
          {result.status === "rejected" && (
            <div data-testid="lwc-rejected-notice" className="alert alert-error mt-3 text-sm">
              Confirmation rejected. The phrase did not match exactly. Restore writes remain
              disabled.
            </div>
          )}
        </div>
      )}
    </div>
  );
}
