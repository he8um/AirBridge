import type {
  CheckpointDurabilityPolicyResult,
  CheckpointDurabilityCheckStatus,
} from "../../backend/types";

interface Props {
  result: CheckpointDurabilityPolicyResult | null;
  loading: boolean;
  onVerify: () => void;
}

function checkBadge(status: CheckpointDurabilityCheckStatus): string {
  if (status === "passed") return "badge badge-success badge-sm";
  if (status === "warning") return "badge badge-warning badge-sm";
  return "badge badge-error badge-sm";
}

/**
 * Checkpoint Durability Policy Panel — Gate 10.
 *
 * Safety invariants:
 * - Always shows a disabled notice that restore writes remain unavailable.
 * - No execute button.
 * - No token input.
 * - No "succeeded" language.
 * - Compliant status does NOT imply writes are enabled.
 */
export function RestoreCheckpointDurabilityPolicyPanel({ result, loading, onVerify }: Props) {
  return (
    <div data-testid="restore-cdp-panel">
      <div data-testid="cdp-writes-disabled-notice" className="alert alert-warning mb-4">
        <span>
          Live restore writes are disabled. Verifying this policy does not enable writes or start
          any restore operation.
        </span>
      </div>

      <h3 className="font-semibold mb-2">Gate 10 — Checkpoint Durability Policy</h3>

      <button
        data-testid="cdp-verify-button"
        className="btn btn-primary mb-4"
        onClick={onVerify}
        disabled={loading}
      >
        {loading ? "Checking…" : "Verify checkpoint durability policy"}
      </button>

      {result && (
        <div data-testid="cdp-result">
          <div className="flex gap-2 items-center mb-3">
            {result.status === "compliant" && (
              <span data-testid="cdp-compliant-badge" className="badge badge-success">
                Compliant
              </span>
            )}
            {result.status === "warning" && (
              <span data-testid="cdp-warning-badge" className="badge badge-warning">
                Warning
              </span>
            )}
            {result.status === "blocked" && (
              <span data-testid="cdp-blocked-badge" className="badge badge-error">
                Blocked
              </span>
            )}
            <span data-testid="cdp-message" className="text-sm">
              {result.message}
            </span>
          </div>

          {result.planSummary && (
            <div data-testid="cdp-plan-summary" className="bg-base-200 rounded p-3 mb-3 text-sm">
              <p className="font-semibold mb-1">Declared checkpoint plan</p>
              <ul className="space-y-0.5 text-xs">
                <li data-testid="cdp-table-checkpoint">
                  Checkpoint after each table:{" "}
                  <strong>
                    {result.planSummary.checkpointAfterEachTable ? "declared" : "missing"}
                  </strong>
                </li>
                <li data-testid="cdp-batch-checkpoint">
                  Checkpoint after each batch:{" "}
                  <strong>
                    {result.planSummary.checkpointAfterEachBatch ? "declared" : "missing"}
                  </strong>
                </li>
                <li data-testid="cdp-phase-markers">
                  Phase markers:{" "}
                  <strong>{result.planSummary.hasPhaseMarkers ? "declared" : "missing"}</strong>
                </li>
                <li data-testid="cdp-id-mapping-checkpoint">
                  ID mapping checkpoint:{" "}
                  <strong>
                    {result.planSummary.hasIdMappingCheckpoint ? "declared" : "not declared"}
                  </strong>
                </li>
                <li data-testid="cdp-resume-stop-condition">
                  Resume-safe stop condition:{" "}
                  <strong>
                    {result.planSummary.hasResumeSafeStopCondition ? "declared" : "missing"}
                  </strong>
                </li>
                <li data-testid="cdp-linked-updates">
                  Linked updates:{" "}
                  <strong>{result.planSummary.hasLinkedUpdates ? "yes" : "no"}</strong>
                </li>
                <li data-testid="cdp-durability-backend">
                  Durability backend:{" "}
                  <strong>{result.planSummary.durabilityBackend ?? "not declared"}</strong>
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
                  <tr key={check.checkId} data-testid="cdp-check-row">
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

          <div data-testid="cdp-safety-summary" className="text-xs text-base-content/60 mt-2">
            <span data-testid="cdp-no-changes-notice">No changes made.</span>{" "}
            <span>Network writes not attempted.</span>{" "}
            <span>Writes enabled: {result.writesEnabled ? "yes" : "no"}.</span>
          </div>

          {result.status === "compliant" && (
            <div data-testid="cdp-compliant-notice" className="alert alert-success mt-3 text-sm">
              Checkpoint durability plan is complete and within safe bounds. Restore writes remain
              disabled — compliance does not start any write operation.
            </div>
          )}
          {result.status === "warning" && (
            <div data-testid="cdp-warning-notice" className="alert alert-warning mt-3 text-sm">
              Checkpoint durability plan has warnings. Review incomplete fields before proceeding.
              Restore writes remain disabled.
            </div>
          )}
          {result.status === "blocked" && (
            <div data-testid="cdp-blocked-notice" className="alert alert-error mt-3 text-sm">
              Checkpoint durability plan is blocked. Resolve all missing checkpoint fields before
              any live write is considered. Restore writes remain disabled.
            </div>
          )}
        </div>
      )}
    </div>
  );
}
