import type { FinalValidationPolicyResult, FinalValidationCheckStatus } from "../../backend/types";

interface Props {
  result: FinalValidationPolicyResult | null;
  loading: boolean;
  onVerify: () => void;
}

function checkBadge(status: FinalValidationCheckStatus): string {
  if (status === "passed") return "badge badge-success badge-sm";
  if (status === "warning") return "badge badge-warning badge-sm";
  return "badge badge-error badge-sm";
}

/**
 * Final Validation Policy Panel — Gate 11.
 *
 * Safety invariants:
 * - Always shows a disabled notice that restore writes remain unavailable.
 * - No execute button.
 * - No token input.
 * - No "succeeded" language.
 * - Compliant status does NOT imply writes are enabled.
 * - Compliant status does NOT introduce a restore success state.
 */
export function RestoreFinalValidationPolicyPanel({ result, loading, onVerify }: Props) {
  return (
    <div data-testid="restore-fvp-panel">
      <div data-testid="fvp-writes-disabled-notice" className="alert alert-warning mb-4">
        <span>
          Live restore writes are disabled. Verifying this policy does not enable writes or start
          any restore operation.
        </span>
      </div>

      <h3 className="font-semibold mb-2">Gate 11 — Final Validation Policy</h3>

      <button
        data-testid="fvp-verify-button"
        className="btn btn-primary mb-4"
        onClick={onVerify}
        disabled={loading}
      >
        {loading ? "Checking…" : "Verify final validation policy"}
      </button>

      {result && (
        <div data-testid="fvp-result">
          <div className="flex gap-2 items-center mb-3">
            {result.status === "compliant" && (
              <span data-testid="fvp-compliant-badge" className="badge badge-success">
                Compliant
              </span>
            )}
            {result.status === "warning" && (
              <span data-testid="fvp-warning-badge" className="badge badge-warning">
                Warning
              </span>
            )}
            {result.status === "blocked" && (
              <span data-testid="fvp-blocked-badge" className="badge badge-error">
                Blocked
              </span>
            )}
            <span data-testid="fvp-message" className="text-sm">
              {result.message}
            </span>
          </div>

          {result.planSummary && (
            <div data-testid="fvp-plan-summary" className="bg-base-200 rounded p-3 mb-3 text-sm">
              <p className="font-semibold mb-1">Declared final validation plan</p>
              <ul className="space-y-0.5 text-xs">
                <li data-testid="fvp-schema-count-validation">
                  Schema count validation:{" "}
                  <strong>
                    {result.planSummary.hasSchemaCountValidation ? "declared" : "missing"}
                  </strong>
                </li>
                <li data-testid="fvp-table-field-validation">
                  Table/field validation:{" "}
                  <strong>
                    {result.planSummary.hasTableFieldValidation ? "declared" : "missing"}
                  </strong>
                </li>
                <li data-testid="fvp-record-count-validation">
                  Record count validation:{" "}
                  <strong>
                    {result.planSummary.hasRecordCountValidation ? "declared" : "missing"}
                  </strong>
                </li>
                <li data-testid="fvp-id-mapping-validation">
                  ID mapping validation:{" "}
                  <strong>
                    {result.planSummary.hasIdMappingValidation ? "declared" : "missing"}
                  </strong>
                </li>
                <li data-testid="fvp-linked-record-validation">
                  Linked record validation:{" "}
                  <strong>
                    {result.planSummary.hasLinkedRecordValidation ? "declared" : "missing"}
                  </strong>
                </li>
                <li data-testid="fvp-attachment-metadata-validation">
                  Attachment metadata validation:{" "}
                  <strong>
                    {result.planSummary.hasAttachmentMetadataValidation ? "declared" : "missing"}
                  </strong>
                </li>
                <li data-testid="fvp-attachment-metadata-only">
                  Attachment validation scope:{" "}
                  <strong>
                    {result.planSummary.attachmentValidationMetadataOnly ? "metadata only" : "full"}
                  </strong>
                </li>
                <li data-testid="fvp-manifest-checksum-validation">
                  Manifest checksum validation:{" "}
                  <strong>
                    {result.planSummary.hasManifestChecksumValidation ? "declared" : "missing"}
                  </strong>
                </li>
                <li data-testid="fvp-blocks-success-without-validation">
                  Blocks success without validation:{" "}
                  <strong>
                    {result.planSummary.blocksSuccessWithoutValidation ? "yes" : "no"}
                  </strong>
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
                  <tr key={check.checkId} data-testid="fvp-check-row">
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

          <div data-testid="fvp-safety-summary" className="text-xs text-base-content/60 mt-2">
            <span data-testid="fvp-no-changes-notice">No changes made.</span>{" "}
            <span>Network writes not attempted.</span>{" "}
            <span>Writes enabled: {result.writesEnabled ? "yes" : "no"}.</span>
          </div>

          {result.status === "compliant" && (
            <div data-testid="fvp-compliant-notice" className="alert alert-success mt-3 text-sm">
              Final validation plan is complete and within safe bounds. Restore writes remain
              disabled — compliance does not start any write operation and does not introduce a
              restore success state.
            </div>
          )}
          {result.status === "warning" && (
            <div data-testid="fvp-warning-notice" className="alert alert-warning mt-3 text-sm">
              Final validation plan has warnings. Review incomplete validation steps before
              proceeding. Restore writes remain disabled.
            </div>
          )}
          {result.status === "blocked" && (
            <div data-testid="fvp-blocked-notice" className="alert alert-error mt-3 text-sm">
              Final validation plan is blocked. Resolve all missing validation steps before any live
              write is considered. Restore writes remain disabled.
            </div>
          )}
        </div>
      )}
    </div>
  );
}
