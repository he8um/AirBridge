import type {
  SensitiveDataSafetyPolicyResult,
  SensitiveDataSafetyCheckStatus,
} from "../../backend/types";

interface Props {
  result: SensitiveDataSafetyPolicyResult | null;
  loading: boolean;
  onVerify: () => void;
}

function checkBadge(status: SensitiveDataSafetyCheckStatus): string {
  if (status === "passed") return "badge badge-success badge-sm";
  if (status === "warning") return "badge badge-warning badge-sm";
  return "badge badge-error badge-sm";
}

/**
 * Sensitive Data Safety Policy Panel — Gate 16.
 *
 * Safety invariants:
 * - Always shows a disabled notice that restore writes remain unavailable.
 * - No execute button.
 * - No token input.
 * - No "succeeded" language.
 * - Compliant status does NOT imply writes are enabled.
 * - Compliant status does NOT introduce a restore success state.
 * - No token, full path, package path, record payload, or attachment URL is returned or displayed.
 */
export function RestoreSensitiveDataSafetyPolicyPanel({ result, loading, onVerify }: Props) {
  return (
    <div data-testid="restore-sds-panel">
      <div data-testid="sds-writes-disabled-notice" className="alert alert-warning mb-4">
        <span>
          Live restore writes are disabled. Verifying this policy does not enable writes or start
          any restore operation. Sensitive material must never be exposed through any restore write
          surface.
        </span>
      </div>

      <h3 className="font-semibold mb-2">Gate 16 — Sensitive Data Safety Policy</h3>

      <button
        data-testid="sds-verify-button"
        className="btn btn-primary mb-4"
        onClick={onVerify}
        disabled={loading}
      >
        {loading ? "Checking…" : "Verify sensitive data safety policy"}
      </button>

      {result && (
        <div data-testid="sds-result">
          <div className="flex gap-2 items-center mb-3">
            {result.status === "compliant" && (
              <span data-testid="sds-compliant-badge" className="badge badge-success">
                Compliant
              </span>
            )}
            {result.status === "warning" && (
              <span data-testid="sds-warning-badge" className="badge badge-warning">
                Warning
              </span>
            )}
            {result.status === "blocked" && (
              <span data-testid="sds-blocked-badge" className="badge badge-error">
                Blocked
              </span>
            )}
            <span data-testid="sds-writes-disabled-tag" className="badge badge-outline badge-sm">
              Writes disabled
            </span>
          </div>

          <p data-testid="sds-message" className="text-sm mb-4">
            {result.message}
          </p>

          {result.safetySummary && (
            <div data-testid="sds-safety-summary" className="mb-4 p-3 bg-base-200 rounded text-sm">
              <h4 className="font-semibold mb-2">Safety Summary</h4>
              <ul className="space-y-1">
                <li>
                  <span className="font-medium">Surfaces covered:</span>{" "}
                  <span data-testid="sds-summary-surfaces-covered">
                    {result.safetySummary.surfacesCovered} / 10
                  </span>
                </li>
                <li>
                  <span className="font-medium">Total redaction rules:</span>{" "}
                  <span data-testid="sds-summary-total-rules">
                    {result.safetySummary.totalRedactionRules}
                  </span>
                </li>
                <li>
                  <span className="font-medium">All rules named:</span>{" "}
                  <span data-testid="sds-summary-all-named">
                    {result.safetySummary.allRulesNamed ? "Yes" : "No"}
                  </span>
                </li>
                <li>
                  <span className="font-medium">No token in results:</span>{" "}
                  <span data-testid="sds-summary-no-token">
                    {result.safetySummary.noTokenInResults ? "Yes" : "No"}
                  </span>
                </li>
                <li>
                  <span className="font-medium">No full path in results:</span>{" "}
                  <span data-testid="sds-summary-no-full-path">
                    {result.safetySummary.noFullPathInResults ? "Yes" : "No"}
                  </span>
                </li>
                <li>
                  <span className="font-medium">Package references filename only:</span>{" "}
                  <span data-testid="sds-summary-filename-only">
                    {result.safetySummary.packageReferencesFilenameOnly ? "Yes" : "No"}
                  </span>
                </li>
                <li>
                  <span className="font-medium">No record payload in results:</span>{" "}
                  <span data-testid="sds-summary-no-record-payload">
                    {result.safetySummary.noRecordPayloadInResults ? "Yes" : "No"}
                  </span>
                </li>
                <li>
                  <span className="font-medium">No attachment URL in results:</span>{" "}
                  <span data-testid="sds-summary-no-attachment-url">
                    {result.safetySummary.noAttachmentUrlInResults ? "Yes" : "No"}
                  </span>
                </li>
                <li>
                  <span className="font-medium">No raw HTTP in results:</span>{" "}
                  <span data-testid="sds-summary-no-raw-http">
                    {result.safetySummary.noRawHttpInResults ? "Yes" : "No"}
                  </span>
                </li>
                <li>
                  <span className="font-medium">Error messages use safe summaries:</span>{" "}
                  <span data-testid="sds-summary-safe-errors">
                    {result.safetySummary.errorMessagesUseSafeSummaries ? "Yes" : "No"}
                  </span>
                </li>
                <li>
                  <span className="font-medium">Summaries are payload-free:</span>{" "}
                  <span data-testid="sds-summary-payload-free">
                    {result.safetySummary.summariesArePayloadFree ? "Yes" : "No"}
                  </span>
                </li>
              </ul>
            </div>
          )}

          <div data-testid="sds-checks" className="space-y-2">
            {result.checks.map((check) => (
              <div
                key={check.checkId}
                data-testid={`sds-check-${check.checkId.toLowerCase()}`}
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
                    data-testid={`sds-remediation-${check.checkId.toLowerCase()}`}
                    className="text-xs text-warning"
                  >
                    {check.remediation}
                  </p>
                )}
              </div>
            ))}
          </div>

          <div data-testid="sds-no-changes-made" className="mt-4 text-xs text-base-content/50">
            No changes made · No network writes attempted · Writes disabled
          </div>
        </div>
      )}
    </div>
  );
}
