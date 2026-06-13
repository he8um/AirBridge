import type { TargetEmptyVerificationResult } from "../../backend/types";

interface RestoreTargetEmptyVerificationPanelProps {
  /** Verification result, or null if not yet run. */
  result: TargetEmptyVerificationResult | null;
  /** Whether a verification run is currently in progress. */
  loading: boolean;
  /** Callback to trigger a new verification run. */
  onVerify: () => void;
}

/**
 * Displays target empty verification (Gate 3).
 *
 * - Always shows a disabled notice that restore writes remain unavailable.
 * - Shows a verify button and result once run.
 * - Never shows an execute button.
 * - Never shows a token input.
 * - Never shows a success message.
 */
export function RestoreTargetEmptyVerificationPanel({
  result,
  loading,
  onVerify,
}: RestoreTargetEmptyVerificationPanelProps) {
  return (
    <div data-testid="restore-target-empty-panel">
      {/* Section header */}
      <div style={{ marginBottom: "var(--space-3)" }}>
        <p
          style={{
            fontSize: "var(--text-xs)",
            fontWeight: 600,
            color: "var(--color-text-muted)",
            textTransform: "uppercase",
            letterSpacing: "0.06em",
            margin: 0,
          }}
        >
          Target Empty Verification (Gate 3)
        </p>
      </div>

      {/* Always-visible disabled notice */}
      <div
        className="notice notice-warning"
        role="note"
        aria-label="Target empty verification writes disabled notice"
        data-testid="target-empty-writes-disabled-notice"
      >
        Target empty verification checks local and reported counts only. Restore writes remain
        disabled in this version.
      </div>

      {/* Verify button */}
      <div style={{ marginTop: "var(--space-3)" }}>
        <button
          type="button"
          className="btn btn-secondary"
          onClick={onVerify}
          disabled={loading}
          data-testid="target-empty-verify-button"
          aria-label={result ? "Re-verify target empty status" : "Verify target empty status"}
        >
          {loading ? "Verifying…" : result ? "Re-verify" : "Verify target is empty"}
        </button>
      </div>

      {/* Result */}
      {result && (
        <div style={{ marginTop: "var(--space-4)" }} data-testid="target-empty-result">
          {/* Status badge */}
          <div style={{ marginBottom: "var(--space-2)" }}>
            <span
              className={
                result.status === "verified"
                  ? "badge badge-success"
                  : result.status === "warning"
                    ? "badge badge-warning"
                    : "badge badge-error"
              }
              data-testid="target-empty-status"
            >
              {result.status}
            </span>
          </div>

          {/* Message */}
          <p
            style={{
              fontSize: "var(--text-sm)",
              color: "var(--color-text-muted)",
              margin: "0 0 var(--space-3) 0",
            }}
            data-testid="target-empty-message"
          >
            {result.message}
          </p>

          {/* Check rows */}
          {result.checks.length > 0 && (
            <div
              style={{ display: "flex", flexDirection: "column", gap: "var(--space-1)" }}
              data-testid="target-empty-checks"
            >
              {result.checks.map((check) => (
                <div
                  key={check.checkId}
                  style={{
                    display: "flex",
                    gap: "var(--space-2)",
                    alignItems: "flex-start",
                    fontSize: "var(--text-xs)",
                  }}
                  data-testid="target-empty-check-row"
                >
                  <span
                    className={
                      check.status === "passed"
                        ? "badge badge-success"
                        : check.status === "warning"
                          ? "badge badge-warning"
                          : check.status === "skipped"
                            ? "badge badge-neutral"
                            : "badge badge-error"
                    }
                    style={{ flexShrink: 0 }}
                  >
                    {check.checkId}
                  </span>
                  <span style={{ color: "var(--color-text-muted)" }}>
                    {check.label}: {check.message}
                  </span>
                </div>
              ))}
            </div>
          )}

          {/* Safety summary */}
          <div style={{ marginTop: "var(--space-3)" }} data-testid="target-empty-safety-summary">
            <p
              style={{
                fontSize: "var(--text-xs)",
                color: "var(--color-text-muted)",
                margin: "0 0 var(--space-1) 0",
              }}
            >
              writesEnabled: {result.writesEnabled ? "Yes" : "No"} &nbsp;|&nbsp;
              networkWritesAttempted: {result.networkWritesAttempted ? "Yes" : "No"}
            </p>
            <p
              style={{ fontSize: "var(--text-xs)", color: "var(--color-text-muted)", margin: 0 }}
              data-testid="target-empty-no-changes-notice"
            >
              No Airtable changes were made.
            </p>
          </div>

          {/* Status-specific notices */}
          {result.status === "verified" && (
            <div
              className="notice notice-success"
              role="note"
              style={{ marginTop: "var(--space-3)" }}
              data-testid="target-empty-verified-notice"
            >
              Target is empty. Restore writes remain disabled — no Airtable changes will be made.
            </div>
          )}
          {result.status === "warning" && (
            <div
              className="notice notice-warning"
              role="note"
              style={{ marginTop: "var(--space-3)" }}
              data-testid="target-empty-warning-notice"
            >
              Target emptiness could not be confirmed. Resolve before enabling live writes.
            </div>
          )}
          {result.status === "blocked" && (
            <div
              className="notice notice-error"
              role="alert"
              style={{ marginTop: "var(--space-3)" }}
              data-testid="target-empty-blocked-notice"
            >
              Target verification is blocked. Resolve the issues listed above before proceeding.
            </div>
          )}
        </div>
      )}
    </div>
  );
}
