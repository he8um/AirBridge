import type { RestoreWriteEngineResult } from "../../backend/types";

interface RestoreWriteEnginePanelProps {
  /** Pre-computed result from the write engine skeleton preview, or null if not yet run. */
  result: RestoreWriteEngineResult | null;
}

/**
 * Displays the write engine skeleton status.
 *
 * - Always shows "Restore execution is disabled" notice.
 * - If a preview result is available, shows per-phase status.
 * - Never shows a success message.
 * - Never shows an execute button.
 * - Never requests a token.
 * - noChangesMade is always true in any result shown.
 */
export function RestoreWriteEnginePanel({ result }: RestoreWriteEnginePanelProps) {
  return (
    <div data-testid="restore-write-engine-panel">
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
          Write Engine
        </p>
      </div>

      {/* Always-visible disabled notice */}
      <div
        className="notice notice-warning"
        role="note"
        aria-label="Write engine disabled"
        data-testid="write-engine-disabled-notice"
        style={{ fontSize: "var(--text-xs)", marginBottom: "var(--space-3)" }}
      >
        <span>
          Restore write execution is not enabled in this version. Schema creation and record import
          are planning-only operations. No Airtable changes are made.
        </span>
      </div>

      {/* Preview result — shown only when a skeleton preview has been computed */}
      {result !== null && (
        <div
          data-testid="write-engine-preview-result"
          style={{ display: "flex", flexDirection: "column", gap: "var(--space-2)" }}
        >
          {/* No-changes safety statement */}
          <div
            className="notice notice-info"
            role="note"
            aria-label="No Airtable changes were made"
            data-testid="write-engine-no-changes-notice"
            style={{ fontSize: "var(--text-xs)" }}
          >
            <span>No Airtable changes were made.</span>
          </div>

          {/* Phase summary table */}
          <div style={{ display: "flex", flexDirection: "column", gap: "var(--space-1)" }}>
            {result.phaseSummaries.map((phase, i) => (
              <div
                key={i}
                style={{
                  display: "flex",
                  alignItems: "flex-start",
                  gap: "var(--space-2)",
                  fontSize: "var(--text-xs)",
                }}
                data-testid="write-engine-phase-row"
                data-phase={phase.phase}
                data-status={phase.status}
              >
                <span
                  style={{
                    color: "var(--color-text-muted)",
                    fontWeight: 600,
                    width: 12,
                    textAlign: "center",
                    flexShrink: 0,
                  }}
                >
                  —
                </span>
                <span style={{ color: "var(--color-text-muted)" }}>{phase.note}</span>
              </div>
            ))}
          </div>
        </div>
      )}
    </div>
  );
}

