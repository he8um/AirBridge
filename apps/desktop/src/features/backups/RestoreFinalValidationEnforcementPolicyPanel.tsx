import type {
  FinalValidationEnforcementPolicyResult,
  FinalValidationEnforcementCheckStatus,
} from "../../backend/types";

interface Props {
  result: FinalValidationEnforcementPolicyResult | null;
  loading: boolean;
  onVerify: () => void;
}

function checkBadge(status: FinalValidationEnforcementCheckStatus): string {
  if (status === "passed") return "badge badge-success badge-sm";
  if (status === "warning") return "badge badge-warning badge-sm";
  return "badge badge-error badge-sm";
}

/**
 * Final Validation Enforcement Policy Panel — Gate 15.
 *
 * Safety invariants:
 * - Always shows a disabled notice that restore writes remain unavailable.
 * - No execute button.
 * - No token input.
 * - No "succeeded" language.
 * - Compliant status does NOT imply writes are enabled.
 * - Compliant status does NOT introduce a restore success state.
 * - No result may be labeled complete or successful before final validation passes.
 */
export function RestoreFinalValidationEnforcementPolicyPanel({ result, loading, onVerify }: Props) {
  return (
    <div data-testid="restore-fve-panel">
      <div data-testid="fve-writes-disabled-notice" className="alert alert-warning mb-4">
        <span>
          Live restore writes are disabled. Verifying this policy does not enable writes or start
          any restore operation. No result may be labeled complete or successful without final
          validation explicitly passing.
        </span>
      </div>

      <h3 className="font-semibold mb-2">Gate 15 — Final Validation Enforcement Policy</h3>

      <button
        data-testid="fve-verify-button"
        className="btn btn-primary mb-4"
        onClick={onVerify}
        disabled={loading}
      >
        {loading ? "Checking…" : "Verify final validation enforcement policy"}
      </button>

      {result && (
        <div data-testid="fve-result">
          <div className="flex gap-2 items-center mb-3">
            {result.status === "compliant" && (
              <span data-testid="fve-compliant-badge" className="badge badge-success">
                Compliant
              </span>
            )}
            {result.status === "warning" && (
              <span data-testid="fve-warning-badge" className="badge badge-warning">
                Warning
              </span>
            )}
            {result.status === "blocked" && (
              <span data-testid="fve-blocked-badge" className="badge badge-error">
                Blocked
              </span>
            )}
            <span data-testid="fve-writes-disabled-tag" className="badge badge-outline badge-sm">
              Writes disabled
            </span>
          </div>

          <p data-testid="fve-message" className="text-sm mb-4">
            {result.message}
          </p>

          {result.enforcementSummary && (
            <div
              data-testid="fve-enforcement-summary"
              className="mb-4 p-3 bg-base-200 rounded text-sm"
            >
              <h4 className="font-semibold mb-2">Enforcement Summary</h4>
              <ul className="space-y-1">
                <li>
                  <span className="font-medium">Schema validation:</span>{" "}
                  <span data-testid="fve-summary-schema-state">
                    {result.enforcementSummary.schemaValidationState}
                  </span>
                </li>
                <li>
                  <span className="font-medium">Record count validation:</span>{" "}
                  <span data-testid="fve-summary-record-count-state">
                    {result.enforcementSummary.recordCountValidationState}
                  </span>
                </li>
                <li>
                  <span className="font-medium">ID mapping validation:</span>{" "}
                  <span data-testid="fve-summary-id-mapping-state">
                    {result.enforcementSummary.idMappingValidationState}
                  </span>
                </li>
                <li>
                  <span className="font-medium">Linked record validation:</span>{" "}
                  <span data-testid="fve-summary-linked-record-state">
                    {result.enforcementSummary.linkedRecordValidationState}
                  </span>
                </li>
                <li>
                  <span className="font-medium">Attachment validation:</span>{" "}
                  <span data-testid="fve-summary-attachment-state">
                    {result.enforcementSummary.attachmentMetadataValidationState}
                  </span>
                  {result.enforcementSummary.attachmentValidationMetadataOnly && (
                    <span className="ml-1 badge badge-warning badge-xs">metadata-only</span>
                  )}
                </li>
                <li>
                  <span className="font-medium">Manifest checksum validation:</span>{" "}
                  <span data-testid="fve-summary-manifest-state">
                    {result.enforcementSummary.manifestChecksumValidationState}
                  </span>
                  {!result.enforcementSummary.packageManifestPresent && (
                    <span className="ml-1 text-base-content/50">(no manifest)</span>
                  )}
                </li>
                <li>
                  <span className="font-medium">Completion guard declared:</span>{" "}
                  <span data-testid="fve-summary-guard-declared">
                    {result.enforcementSummary.completionGuardDeclared ? "Yes" : "No"}
                  </span>
                </li>
                <li>
                  <span className="font-medium">Blocks completion without validation:</span>{" "}
                  <span data-testid="fve-summary-blocks-completion">
                    {result.enforcementSummary.blocksCompletionWithoutFinalValidation
                      ? "Yes"
                      : "No"}
                  </span>
                </li>
                <li>
                  <span className="font-medium">Failed validation blocks completion:</span>{" "}
                  <span data-testid="fve-summary-failed-blocks">
                    {result.enforcementSummary.failedValidationBlocksCompletion ? "Yes" : "No"}
                  </span>
                </li>
              </ul>
            </div>
          )}

          <div data-testid="fve-checks" className="space-y-2">
            {result.checks.map((check) => (
              <div
                key={check.checkId}
                data-testid={`fve-check-${check.checkId.toLowerCase()}`}
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
                    data-testid={`fve-remediation-${check.checkId.toLowerCase()}`}
                    className="text-xs text-warning"
                  >
                    {check.remediation}
                  </p>
                )}
              </div>
            ))}
          </div>

          <div data-testid="fve-no-changes-made" className="mt-4 text-xs text-base-content/50">
            No changes made · No network writes attempted · Writes disabled
          </div>
        </div>
      )}
    </div>
  );
}
