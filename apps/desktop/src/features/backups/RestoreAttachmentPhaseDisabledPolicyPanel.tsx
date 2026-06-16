import type {
  AttachmentPhaseDisabledPolicyResult,
  AttachmentPhaseDisabledCheckStatus,
} from "../../backend/types";

interface Props {
  result: AttachmentPhaseDisabledPolicyResult | null;
  loading: boolean;
  onVerify: () => void;
}

function checkBadge(status: AttachmentPhaseDisabledCheckStatus): string {
  if (status === "passed") return "badge badge-success badge-sm";
  if (status === "warning") return "badge badge-warning badge-sm";
  return "badge badge-error badge-sm";
}

/**
 * Attachment Phase Disabled Policy Panel — Gate 17.
 *
 * Safety invariants:
 * - Always shows a disabled notice that restore writes remain unavailable.
 * - Always shows a metadata-only notice.
 * - No execute button.
 * - No upload/download/fetch button.
 * - No token input.
 * - No path/package-path display.
 * - No record payload display.
 * - No attachment URL display.
 * - No success/completed wording except when stating it remains blocked or unavailable.
 * - Compliant status does NOT imply writes are enabled.
 * - Compliant status does NOT introduce a restore success state.
 * - Binary attachment restore is out of scope.
 */
export function RestoreAttachmentPhaseDisabledPolicyPanel({ result, loading, onVerify }: Props) {
  return (
    <div data-testid="restore-apd-panel">
      <div data-testid="apd-writes-disabled-notice" className="alert alert-warning mb-4">
        <span>
          Live restore writes are disabled. Verifying this policy does not enable writes or start
          any restore operation. Binary attachment download, upload, fetch, and transfer are not
          permitted.
        </span>
      </div>

      <div data-testid="apd-metadata-only-notice" className="alert alert-info mb-4">
        <span>
          Attachment handling is metadata-only. Field names, MIME types, and file sizes may be
          inspected, but no attachment binary content is downloaded, uploaded, fetched, or
          transferred. Binary attachment restore is out of scope.
        </span>
      </div>

      <h3 className="font-semibold mb-2">Gate 17 — Attachment Phase Disabled Policy</h3>

      <button
        data-testid="apd-verify-button"
        className="btn btn-primary mb-4"
        onClick={onVerify}
        disabled={loading}
      >
        {loading ? "Checking…" : "Verify attachment phase disabled policy"}
      </button>

      {result && (
        <div data-testid="apd-result">
          <div className="flex gap-2 items-center mb-3">
            {result.status === "compliant" && (
              <span data-testid="apd-compliant-badge" className="badge badge-success">
                Compliant
              </span>
            )}
            {result.status === "warning" && (
              <span data-testid="apd-warning-badge" className="badge badge-warning">
                Warning
              </span>
            )}
            {result.status === "blocked" && (
              <span data-testid="apd-blocked-badge" className="badge badge-error">
                Blocked
              </span>
            )}
            <span data-testid="apd-writes-disabled-tag" className="badge badge-outline badge-sm">
              Writes disabled
            </span>
            <span
              data-testid="apd-metadata-only-tag"
              className="badge badge-outline badge-sm badge-info"
            >
              Metadata only
            </span>
          </div>

          <p data-testid="apd-message" className="text-sm mb-4">
            {result.message}
          </p>

          {result.phaseSummary && (
            <div data-testid="apd-phase-summary" className="mb-4 p-3 bg-base-200 rounded text-sm">
              <h4 className="font-semibold mb-2">Attachment Phase Summary</h4>
              <ul className="space-y-1">
                <li>
                  <span className="font-medium">Metadata inspection enabled:</span>{" "}
                  <span data-testid="apd-summary-metadata-inspection">
                    {result.phaseSummary.metadataInspectionEnabled ? "Yes" : "No"}
                  </span>
                </li>
                <li>
                  <span className="font-medium">Metadata verification enabled:</span>{" "}
                  <span data-testid="apd-summary-metadata-verification">
                    {result.phaseSummary.metadataVerificationEnabled ? "Yes" : "No"}
                  </span>
                </li>
                <li>
                  <span className="font-medium">Binary handling disabled:</span>{" "}
                  <span data-testid="apd-summary-binary-disabled">
                    {result.phaseSummary.binaryHandlingDisabled ? "Yes" : "No"}
                  </span>
                </li>
                <li>
                  <span className="font-medium">URL exposure disabled:</span>{" "}
                  <span data-testid="apd-summary-url-disabled">
                    {result.phaseSummary.urlExposureDisabled ? "Yes" : "No"}
                  </span>
                </li>
                <li>
                  <span className="font-medium">Field mutation disabled:</span>{" "}
                  <span data-testid="apd-summary-field-mutation-disabled">
                    {result.phaseSummary.fieldMutationDisabled ? "Yes" : "No"}
                  </span>
                </li>
                <li>
                  <span className="font-medium">Phase not required for completion:</span>{" "}
                  <span data-testid="apd-summary-not-required">
                    {result.phaseSummary.phaseRequiredForCompletionDisabled ? "Yes" : "No"}
                  </span>
                </li>
                <li>
                  <span className="font-medium">Final validation metadata-only:</span>{" "}
                  <span data-testid="apd-summary-final-validation-metadata-only">
                    {result.phaseSummary.finalValidationTreatsAsMetadataOnly ? "Yes" : "No"}
                  </span>
                </li>
                <li>
                  <span className="font-medium">Blocked operations declared:</span>{" "}
                  <span data-testid="apd-summary-blocked-operations">
                    {result.phaseSummary.blockedOperationsDeclared}
                  </span>
                </li>
              </ul>
            </div>
          )}

          <div data-testid="apd-operation-table" className="mb-4 p-3 bg-base-200 rounded text-sm">
            <h4 className="font-semibold mb-2">Attachment Operation Classes</h4>
            <table className="w-full text-xs">
              <thead>
                <tr>
                  <th className="text-left py-1">Operation</th>
                  <th className="text-left py-1">Status</th>
                </tr>
              </thead>
              <tbody>
                <tr data-testid="apd-op-metadata-inspect">
                  <td>metadataInspect</td>
                  <td>
                    <span className="badge badge-success badge-xs">permitted</span>
                  </td>
                </tr>
                <tr data-testid="apd-op-metadata-verify">
                  <td>metadataVerify</td>
                  <td>
                    <span className="badge badge-success badge-xs">permitted</span>
                  </td>
                </tr>
                <tr data-testid="apd-op-binary-download">
                  <td>binaryDownload</td>
                  <td>
                    <span className="badge badge-error badge-xs">blocked</span>
                  </td>
                </tr>
                <tr data-testid="apd-op-binary-upload">
                  <td>binaryUpload</td>
                  <td>
                    <span className="badge badge-error badge-xs">blocked</span>
                  </td>
                </tr>
                <tr data-testid="apd-op-url-fetch">
                  <td>urlFetch</td>
                  <td>
                    <span className="badge badge-error badge-xs">blocked</span>
                  </td>
                </tr>
                <tr data-testid="apd-op-file-read">
                  <td>fileRead</td>
                  <td>
                    <span className="badge badge-error badge-xs">blocked</span>
                  </td>
                </tr>
                <tr data-testid="apd-op-file-write">
                  <td>fileWrite</td>
                  <td>
                    <span className="badge badge-error badge-xs">blocked</span>
                  </td>
                </tr>
                <tr data-testid="apd-op-raw-transfer">
                  <td>rawAttachmentTransfer</td>
                  <td>
                    <span className="badge badge-error badge-xs">blocked</span>
                  </td>
                </tr>
                <tr data-testid="apd-op-field-mutation">
                  <td>attachmentFieldMutation</td>
                  <td>
                    <span className="badge badge-error badge-xs">blocked</span>
                  </td>
                </tr>
                <tr data-testid="apd-op-url-exposure">
                  <td>attachmentUrlExposure</td>
                  <td>
                    <span className="badge badge-error badge-xs">blocked</span>
                  </td>
                </tr>
              </tbody>
            </table>
          </div>

          <div data-testid="apd-checks" className="space-y-2">
            {result.checks.map((check) => (
              <div
                key={check.checkId}
                data-testid={`apd-check-${check.checkId.toLowerCase()}`}
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
                    data-testid={`apd-remediation-${check.checkId.toLowerCase()}`}
                    className="text-xs text-warning"
                  >
                    {check.remediation}
                  </p>
                )}
              </div>
            ))}
          </div>

          <div data-testid="apd-no-changes-made" className="mt-4 text-xs text-base-content/50">
            No changes made · No network writes attempted · Writes disabled · Metadata only
          </div>
        </div>
      )}
    </div>
  );
}
