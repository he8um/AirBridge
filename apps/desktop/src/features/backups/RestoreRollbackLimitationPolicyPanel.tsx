import type {
  RollbackLimitationPolicyResult,
  RollbackLimitationCheckStatus,
} from "../../backend/types";

interface Props {
  result: RollbackLimitationPolicyResult | null;
  loading: boolean;
  onVerify: () => void;
}

function checkBadge(status: RollbackLimitationCheckStatus): string {
  if (status === "passed") return "badge badge-success badge-sm";
  if (status === "warning") return "badge badge-warning badge-sm";
  return "badge badge-error badge-sm";
}

/**
 * Rollback Limitation Policy Panel — Gate 14.
 *
 * Safety invariants:
 * - Always shows a disabled notice that restore writes remain unavailable.
 * - No execute button.
 * - No cleanup/delete/revert button.
 * - No token input.
 * - No "succeeded" language.
 * - Compliant status does NOT imply writes are enabled.
 * - Compliant status does NOT introduce a restore success state.
 * - No automatic rollback, delete, or update cleanup path exists.
 */
export function RestoreRollbackLimitationPolicyPanel({ result, loading, onVerify }: Props) {
  return (
    <div data-testid="restore-rlp-panel">
      <div data-testid="rlp-writes-disabled-notice" className="alert alert-warning mb-4">
        <span>
          Live restore writes are disabled. Verifying this policy does not enable writes or start
          any restore operation. Automatic rollback is not available — manual cleanup requires a
          separate explicit future action.
        </span>
      </div>

      <h3 className="font-semibold mb-2">Gate 14 — Rollback Limitation Policy</h3>

      <button
        data-testid="rlp-verify-button"
        className="btn btn-primary mb-4"
        onClick={onVerify}
        disabled={loading}
      >
        {loading ? "Checking…" : "Verify rollback limitation policy"}
      </button>

      {result && (
        <div data-testid="rlp-result">
          <div className="flex gap-2 items-center mb-3">
            {result.status === "compliant" && (
              <span data-testid="rlp-compliant-badge" className="badge badge-success">
                Compliant
              </span>
            )}
            {result.status === "warning" && (
              <span data-testid="rlp-warning-badge" className="badge badge-warning">
                Warning
              </span>
            )}
            {result.status === "blocked" && (
              <span data-testid="rlp-blocked-badge" className="badge badge-error">
                Blocked
              </span>
            )}
            <span data-testid="rlp-writes-disabled-tag" className="badge badge-outline badge-sm">
              Writes disabled
            </span>
          </div>

          <p data-testid="rlp-message" className="text-sm mb-4">
            {result.message}
          </p>

          {result.planSummary && (
            <div data-testid="rlp-plan-summary" className="mb-4 p-3 bg-base-200 rounded text-sm">
              <h4 className="font-semibold mb-2">Rollback Limitation Summary</h4>
              <ul className="space-y-1">
                <li>
                  <span className="font-medium">Rollback behavior:</span>{" "}
                  <span data-testid="rlp-summary-rollback-behavior">
                    {result.planSummary.rollbackBehavior}
                  </span>
                </li>
                <li>
                  <span className="font-medium">Partial restore is not success:</span>{" "}
                  <span data-testid="rlp-summary-partial-not-success">
                    {result.planSummary.partialRestoreIsNotSuccess ? "Yes" : "No"}
                  </span>
                </li>
                <li>
                  <span className="font-medium">Recovery guidance declared:</span>{" "}
                  <span data-testid="rlp-summary-recovery-guidance">
                    {result.planSummary.recoveryGuidanceDeclared ? "Yes" : "No"}
                  </span>
                </li>
                <li>
                  <span className="font-medium">Includes checkpoint guidance:</span>{" "}
                  <span data-testid="rlp-summary-checkpoint-guidance">
                    {result.planSummary.includesCheckpointGuidance ? "Yes" : "No"}
                  </span>
                </li>
                <li>
                  <span className="font-medium">User-visible notice:</span>{" "}
                  <span data-testid="rlp-summary-user-notice">
                    {result.planSummary.userVisibleNotice ? "Yes" : "No"}
                  </span>
                </li>
                <li>
                  <span className="font-medium">Manual cleanup requires separate action:</span>{" "}
                  <span data-testid="rlp-summary-separate-action">
                    {result.planSummary.manualCleanupRequiresSeparateAction ? "Yes" : "No"}
                  </span>
                </li>
              </ul>
            </div>
          )}

          <div data-testid="rlp-checks" className="space-y-2">
            {result.checks.map((check) => (
              <div
                key={check.checkId}
                data-testid={`rlp-check-${check.checkId.toLowerCase()}`}
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
                    data-testid={`rlp-remediation-${check.checkId.toLowerCase()}`}
                    className="text-xs text-warning"
                  >
                    {check.remediation}
                  </p>
                )}
              </div>
            ))}
          </div>

          <div data-testid="rlp-no-changes-made" className="mt-4 text-xs text-base-content/50">
            No changes made · No network writes attempted · Writes disabled
          </div>
        </div>
      )}
    </div>
  );
}
