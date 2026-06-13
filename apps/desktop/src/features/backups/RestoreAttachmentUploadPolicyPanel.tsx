import type {
  AttachmentUploadPolicyResult,
  AttachmentUploadPolicyCheckStatus,
} from "../../backend/types";

interface Props {
  result: AttachmentUploadPolicyResult | null;
  loading: boolean;
  onVerify: () => void;
}

function checkBadge(status: AttachmentUploadPolicyCheckStatus): string {
  if (status === "passed") return "badge badge-success badge-sm";
  if (status === "warning") return "badge badge-warning badge-sm";
  return "badge badge-error badge-sm";
}

/**
 * Attachment Upload Policy Panel — Gate 5.
 *
 * Safety invariants:
 * - Always shows a disabled notice that restore writes remain unavailable.
 * - No execute button.
 * - No token input.
 * - No "succeeded" language.
 * - Compliant status does NOT imply writes are enabled.
 * - Attachment file bytes are never uploaded.
 */
export function RestoreAttachmentUploadPolicyPanel({ result, loading, onVerify }: Props) {
  return (
    <div data-testid="restore-aup-panel">
      <h3 className="font-semibold text-base mb-1">Attachment Upload Policy (Gate 5)</h3>

      <div
        role="note"
        aria-label="Attachment upload policy writes disabled notice"
        data-testid="aup-writes-disabled-notice"
        className="alert alert-info mb-3 text-sm"
      >
        Attachment upload policy checks declared attachment field intents only. Restore writes
        remain disabled — no Airtable changes will be made. Attachment file bytes are never
        uploaded.
      </div>

      <button
        className="btn btn-sm btn-outline mb-3"
        data-testid="aup-verify-button"
        disabled={loading}
        onClick={onVerify}
        aria-label={
          result !== null ? "Re-verify attachment upload policy" : "Verify attachment upload policy"
        }
      >
        {loading ? "Checking…" : result !== null ? "Re-verify" : "Verify attachment policy"}
      </button>

      {result === null && !loading && null}

      {result !== null && (
        <div data-testid="aup-result">
          <div className="flex items-center gap-2 mb-2">
            <span data-testid="aup-status">
              {result.status === "compliant" && (
                <span className="badge badge-success" data-testid="aup-compliant-badge">
                  compliant
                </span>
              )}
              {result.status === "warning" && (
                <span className="badge badge-warning" data-testid="aup-warning-badge">
                  warning
                </span>
              )}
              {result.status === "blocked" && (
                <span className="badge badge-error" data-testid="aup-blocked-badge">
                  blocked
                </span>
              )}
            </span>
            <span className="text-sm" data-testid="aup-message">
              {result.message}
            </span>
          </div>

          {result.blockedFieldNames.length > 0 && (
            <div className="text-sm text-error mb-2" data-testid="aup-blocked-fields-list">
              Blocked fields:{" "}
              {result.blockedFieldNames.map((name, i) => (
                <span key={i} className="font-mono mr-1" data-testid="aup-blocked-field-item">
                  {name}
                </span>
              ))}
            </div>
          )}

          <div className="overflow-x-auto mb-2" data-testid="aup-checks">
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
                  <tr key={check.checkId} data-testid="aup-check-row">
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

          <div className="text-xs text-base-content/60 mt-1" data-testid="aup-safety-summary">
            noChangesMade: {result.noChangesMade ? "Yes" : "No"} &nbsp;|&nbsp; writesEnabled:{" "}
            {result.writesEnabled ? "Yes" : "No"} &nbsp;|&nbsp; networkWritesAttempted:{" "}
            {result.networkWritesAttempted ? "Yes" : "No"} &nbsp;|&nbsp; metadataOnlyFields:{" "}
            {result.metadataOnlyFieldCount}
          </div>

          <div className="text-xs mt-1" data-testid="aup-no-changes-notice">
            No changes have been made to Airtable. Attachment file bytes have not been uploaded.
          </div>

          {result.status === "compliant" && (
            <div className="alert alert-success text-sm mt-2" data-testid="aup-compliant-notice">
              All {result.metadataOnlyFieldCount} declared attachment field(s) use metadata-only
              handling. Restore writes remain disabled — no Airtable changes will be made.
            </div>
          )}

          {result.status === "warning" && (
            <div className="alert alert-warning text-sm mt-2" data-testid="aup-warning-notice">
              Some attachment fields have deferred or unknown intent. File bytes will not be
              uploaded or downloaded in this version. Manual review is required before enabling live
              writes.
            </div>
          )}

          {result.status === "blocked" && (
            <div className="alert alert-error text-sm mt-2" data-testid="aup-blocked-notice">
              Attachment upload is not permitted. Change all blocked attachment fields to
              metadata-only intent before proceeding.
            </div>
          )}
        </div>
      )}
    </div>
  );
}
