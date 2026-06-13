import { useState } from "react";
import type { RestoreConfirmationResult } from "../../backend/types";

interface RestoreConfirmationPanelProps {
  /** Pre-computed confirmation result, or null if not yet validated. */
  result: RestoreConfirmationResult | null;
  /** Whether a validation run is currently in progress. */
  loading: boolean;
  /** The exact text the user must type. Shown in the panel. */
  requiredText: string;
  /** Callback invoked with the entered text when the user submits. */
  onValidate: (enteredText: string) => void;
}

/**
 * Displays restore confirmation (Gate 2).
 *
 * - Always shows a disabled notice that restore writes remain unavailable.
 * - Shows the required confirmation text.
 * - Provides a text input and validate button.
 * - Shows confirmed/rejected/blocked result.
 * - Never shows an execute button.
 * - Never shows a token input.
 * - Never shows a success message.
 */
export function RestoreConfirmationPanel({
  result,
  loading,
  requiredText,
  onValidate,
}: RestoreConfirmationPanelProps) {
  const [enteredText, setEnteredText] = useState("");

  const handleSubmit = () => {
    onValidate(enteredText);
  };

  return (
    <div data-testid="restore-confirmation-panel">
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
          Restore Confirmation (Gate 2)
        </p>
      </div>

      {/* Always-visible disabled notice */}
      <div
        className="notice notice-warning"
        role="note"
        aria-label="Confirmation writes disabled notice"
        data-testid="confirmation-writes-disabled-notice"
        style={{ fontSize: "var(--text-xs)", marginBottom: "var(--space-3)" }}
      >
        <span>
          Restore write execution remains disabled in this version. Confirming does not trigger any
          Airtable write operations.
        </span>
      </div>

      {/* Required text display */}
      <div
        style={{
          marginBottom: "var(--space-3)",
          display: "flex",
          flexDirection: "column",
          gap: "var(--space-1)",
        }}
      >
        <p
          style={{
            fontSize: "var(--text-xs)",
            color: "var(--color-text-muted)",
            margin: 0,
          }}
        >
          To confirm, type the following text exactly:
        </p>
        <code
          data-testid="confirmation-required-text"
          style={{
            fontSize: "var(--text-xs)",
            fontWeight: 700,
            letterSpacing: "0.04em",
            color: "var(--color-text)",
            background: "var(--color-surface-raised)",
            padding: "var(--space-1) var(--space-2)",
            borderRadius: "var(--radius-sm)",
            display: "inline-block",
          }}
        >
          {requiredText}
        </code>
      </div>

      {/* Confirmation input */}
      <div
        style={{
          marginBottom: "var(--space-3)",
          display: "flex",
          gap: "var(--space-2)",
          alignItems: "center",
        }}
      >
        <input
          type="text"
          data-testid="confirmation-text-input"
          value={enteredText}
          onChange={(e) => setEnteredText(e.target.value)}
          placeholder="Type the confirmation text above"
          aria-label="Restore confirmation text"
          disabled={loading}
          style={{ flex: 1, fontSize: "var(--text-xs)" }}
        />
        <button
          data-testid="confirmation-validate-button"
          onClick={handleSubmit}
          disabled={loading || enteredText.trim() === ""}
          style={{ fontSize: "var(--text-xs)", flexShrink: 0 }}
        >
          {loading ? "Validating..." : result === null ? "Validate" : "Re-validate"}
        </button>
      </div>

      {/* Result — shown only when a validation result is available */}
      {result !== null && (
        <div
          data-testid="confirmation-result"
          style={{ display: "flex", flexDirection: "column", gap: "var(--space-3)" }}
        >
          {/* Status badge + message */}
          <div style={{ display: "flex", alignItems: "center", gap: "var(--space-2)" }}>
            <span
              data-testid="confirmation-status"
              data-status={result.status}
              style={{
                fontSize: "var(--text-xs)",
                fontWeight: 600,
                textTransform: "uppercase",
                letterSpacing: "0.04em",
                color:
                  result.status === "blocked"
                    ? "var(--color-danger)"
                    : result.status === "confirmed"
                      ? "var(--color-success)"
                      : "var(--color-warning)",
              }}
            >
              {result.status}
            </span>
            <span
              data-testid="confirmation-message"
              style={{ fontSize: "var(--text-xs)", color: "var(--color-text-muted)" }}
            >
              {result.message}
            </span>
          </div>

          {/* Per-check rows */}
          {result.checks.length > 0 && (
            <div style={{ display: "flex", flexDirection: "column", gap: "var(--space-2)" }}>
              {result.checks.map((check) => (
                <div
                  key={check.checkId}
                  data-testid="confirmation-check-row"
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
                        : check.status === "passed"
                          ? "var(--color-success)"
                          : "var(--color-border)"
                    }`,
                  }}
                >
                  <div style={{ display: "flex", alignItems: "center", gap: "var(--space-2)" }}>
                    <span style={{ fontWeight: 600, color: "var(--color-text)" }}>
                      {check.label}
                    </span>
                    <span
                      style={{
                        fontWeight: 600,
                        textTransform: "uppercase",
                        letterSpacing: "0.04em",
                        color:
                          check.status === "failed"
                            ? "var(--color-danger)"
                            : check.status === "passed"
                              ? "var(--color-success)"
                              : "var(--color-text-muted)",
                      }}
                    >
                      {check.status}
                    </span>
                  </div>
                  <span style={{ color: "var(--color-text-muted)" }}>{check.message}</span>
                </div>
              ))}
            </div>
          )}

          {/* Safety summary */}
          <div
            data-testid="confirmation-safety-summary"
            style={{
              display: "flex",
              flexDirection: "column",
              gap: "var(--space-1)",
              fontSize: "var(--text-xs)",
              color: "var(--color-text-muted)",
            }}
          >
            <div style={{ fontWeight: 600, marginBottom: "var(--space-1)" }}>Safety Summary</div>
            <div>Writes enabled: {result.writesEnabled ? "Yes" : "No"}</div>
            <div>Network writes attempted: {result.networkWritesAttempted ? "Yes" : "No"}</div>
            <div data-testid="confirmation-no-changes-notice">No Airtable changes were made</div>
          </div>

          {/* Confirmed notice — writes still disabled */}
          {result.status === "confirmed" && (
            <div
              className="notice notice-info"
              role="note"
              aria-label="Confirmation accepted writes still disabled"
              data-testid="confirmation-accepted-notice"
              style={{ fontSize: "var(--text-xs)" }}
            >
              <span>
                Confirmation accepted. Restore write execution remains disabled in this version.
              </span>
            </div>
          )}

          {/* Rejected notice */}
          {result.status === "rejected" && (
            <div
              className="notice notice-warning"
              role="note"
              aria-label="Confirmation rejected notice"
              data-testid="confirmation-rejected-notice"
              style={{ fontSize: "var(--text-xs)" }}
            >
              <span>Confirmation rejected. Type the exact required text and try again.</span>
            </div>
          )}

          {/* Blocked notice */}
          {result.status === "blocked" && (
            <div
              className="notice notice-danger"
              role="alert"
              aria-label="Confirmation blocked notice"
              data-testid="confirmation-blocked-notice"
              style={{ fontSize: "var(--text-xs)" }}
            >
              <span>Confirmation is blocked. Resolve the listed issues before proceeding.</span>
            </div>
          )}
        </div>
      )}
    </div>
  );
}
