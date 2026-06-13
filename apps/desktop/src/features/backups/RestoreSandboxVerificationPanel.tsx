import type { SandboxVerificationResult } from "../../backend/types";

interface RestoreSandboxVerificationPanelProps {
  /** Pre-computed sandbox verification result from the parent, or null if not yet run. */
  result: SandboxVerificationResult | null;
  /** Whether a verification run is currently in progress. */
  loading: boolean;
  /** Callback invoked when the user requests a (re-)verification. */
  onVerify: () => void;
}

/**
 * Displays sandbox verification status (Gate 1).
 *
 * - Always shows a notice that restore writes remain disabled.
 * - Shows a verify / re-verify button that delegates to the parent.
 * - If a result is available, shows per-check rows and a safety summary.
 * - Never shows a token field.
 * - Never shows a full path.
 * - Never shows an execute button.
 * - Never shows a success message.
 */
export function RestoreSandboxVerificationPanel({
  result,
  loading,
  onVerify,
}: RestoreSandboxVerificationPanelProps) {
  return (
    <div data-testid="restore-sandbox-verification-panel">
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
          Sandbox Verification (Gate 1)
        </p>
      </div>

      {/* Always-visible disabled notice */}
      <div
        className="notice notice-warning"
        role="note"
        aria-label="Sandbox verification disabled notice"
        data-testid="sandbox-verification-disabled-notice"
        style={{ fontSize: "var(--text-xs)", marginBottom: "var(--space-3)" }}
      >
        <span>
          Sandbox verification checks local safety conditions only. Restore writes remain disabled
          in this version.
        </span>
      </div>

      {/* Verify / Re-verify button */}
      <div style={{ marginBottom: "var(--space-3)" }}>
        {loading ? (
          <button
            data-testid="sandbox-verify-button"
            disabled
            style={{ fontSize: "var(--text-xs)" }}
          >
            Verifying...
          </button>
        ) : result === null ? (
          <button
            data-testid="sandbox-verify-button"
            onClick={onVerify}
            style={{ fontSize: "var(--text-xs)" }}
          >
            Verify sandbox safety
          </button>
        ) : (
          <button
            data-testid="sandbox-verify-button"
            onClick={onVerify}
            style={{ fontSize: "var(--text-xs)" }}
          >
            Re-verify
          </button>
        )}
      </div>

      {/* Result — shown only when a verification result is available */}
      {result !== null && (
        <div
          data-testid="sandbox-verification-result"
          style={{ display: "flex", flexDirection: "column", gap: "var(--space-3)" }}
        >
          {/* Overall status badge */}
          <div style={{ display: "flex", alignItems: "center", gap: "var(--space-2)" }}>
            <span
              data-testid="sandbox-verification-status"
              data-status={result.status}
              style={{
                fontSize: "var(--text-xs)",
                fontWeight: 600,
                textTransform: "uppercase",
                letterSpacing: "0.04em",
                color:
                  result.status === "blocked"
                    ? "var(--color-danger)"
                    : result.status === "warning"
                      ? "var(--color-warning)"
                      : "var(--color-success)",
              }}
            >
              {result.status}
            </span>
            <span
              data-testid="sandbox-verification-message"
              style={{ fontSize: "var(--text-xs)", color: "var(--color-text-muted)" }}
            >
              {result.message}
            </span>
          </div>

          {/* Per-check rows */}
          <div style={{ display: "flex", flexDirection: "column", gap: "var(--space-2)" }}>
            {result.checks.map((check) => (
              <div
                key={check.checkId}
                data-testid="sandbox-check-row"
                data-check-id={check.checkId}
                data-status={check.status}
                style={{
                  display: "flex",
                  flexDirection: "column",
                  gap: "var(--space-1)",
                  fontSize: "var(--text-xs)",
                  paddingLeft: "var(--space-2)",
                  borderLeft: `2px solid ${
                    check.status === "failed"
                      ? "var(--color-danger)"
                      : check.status === "warning"
                        ? "var(--color-warning)"
                        : check.status === "passed"
                          ? "var(--color-success)"
                          : "var(--color-border)"
                  }`,
                }}
              >
                <div style={{ display: "flex", alignItems: "center", gap: "var(--space-2)" }}>
                  <span style={{ fontWeight: 600, color: "var(--color-text)" }}>{check.label}</span>
                  <span
                    style={{
                      fontWeight: 600,
                      textTransform: "uppercase",
                      letterSpacing: "0.04em",
                      color:
                        check.status === "failed"
                          ? "var(--color-danger)"
                          : check.status === "warning"
                            ? "var(--color-warning)"
                            : check.status === "passed"
                              ? "var(--color-success)"
                              : "var(--color-text-muted)",
                    }}
                  >
                    {check.status}
                  </span>
                </div>
                <span style={{ color: "var(--color-text-muted)" }}>{check.message}</span>
                {check.remediation !== undefined && (
                  <span style={{ color: "var(--color-text-muted)", fontStyle: "italic" }}>
                    {check.remediation}
                  </span>
                )}
              </div>
            ))}
          </div>

          {/* Safety summary section */}
          <div
            data-testid="sandbox-safety-summary"
            style={{
              display: "flex",
              flexDirection: "column",
              gap: "var(--space-1)",
              fontSize: "var(--text-xs)",
              color: "var(--color-text-muted)",
            }}
          >
            <div style={{ fontWeight: 600, marginBottom: "var(--space-1)" }}>Safety Summary</div>
            <div>Writes enabled: {result.safetySummary.writesEnabled ? "Yes" : "No"}</div>
            <div>
              Network writes attempted: {result.safetySummary.networkWritesAttempted ? "Yes" : "No"}
            </div>
            <div
              data-testid="sandbox-no-changes-notice"
              style={{
                color: result.safetySummary.noChangesMade
                  ? "var(--color-text-muted)"
                  : "var(--color-danger)",
              }}
            >
              No Airtable changes were made
            </div>
            <div>
              Live metadata check performed:{" "}
              {result.safetySummary.liveMetadataCheckPerformed ? "Yes" : "No"}
            </div>
          </div>

          {/* Blocked notice */}
          {result.status === "blocked" && (
            <div
              className="notice notice-danger"
              role="alert"
              aria-label="Sandbox blocked notice"
              data-testid="sandbox-blocked-notice"
              style={{ fontSize: "var(--text-xs)" }}
            >
              <span>
                Live restore writes remain unavailable. Resolve blocked checks before proceeding.
              </span>
            </div>
          )}

          {/* Writes still disabled notice (verified or warning) */}
          {(result.status === "verified" || result.status === "warning") && (
            <div
              className="notice notice-info"
              role="note"
              aria-label="Writes still disabled notice"
              data-testid="sandbox-writes-still-disabled-notice"
              style={{ fontSize: "var(--text-xs)" }}
            >
              <span>
                Restore write execution remains disabled in this version even if all local checks
                pass.
              </span>
            </div>
          )}
        </div>
      )}
    </div>
  );
}
