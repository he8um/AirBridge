import { useRef, useState } from "react";
import type { AirBridgeService } from "../../services/airBridgeService";
import type {
  RestoreExecutionResult,
  RestorePlanStatus,
  RestoreTargetMode,
} from "../../backend/types";

export const RESTORE_EXECUTION_CONFIRMATION_TEXT = "RESTORE BACKUP";

interface RestoreExecutionGatePanelProps {
  service: AirBridgeService;
  /** Filename from the most recent package inspection (filename only, no path). */
  inspectedFilename: string | null;
  /** Validation status from the most recent inspection. */
  inspectionStatus: "valid" | "warning" | "invalid" | null;
  /** Full path to the selected package — used to call the command; never rendered. */
  packagePath: string | null;
  /** Status from the most recent dry-run plan. */
  dryRunStatus: RestorePlanStatus | null;
  /** Target mode selected in the dry-run panel. */
  targetMode: RestoreTargetMode;
  /** Optional target base name. */
  targetBaseName?: string;
}

type ExecState = "idle" | "running" | "done";

export function RestoreExecutionGatePanel({
  service,
  inspectedFilename,
  inspectionStatus,
  packagePath,
  dryRunStatus,
  targetMode,
  targetBaseName,
}: RestoreExecutionGatePanelProps) {
  const [token, setToken] = useState("");
  const [confirmationText, setConfirmationText] = useState("");
  const [execState, setExecState] = useState<ExecState>("idle");
  const [result, setResult] = useState<RestoreExecutionResult | null>(null);
  const tokenRef = useRef<HTMLInputElement>(null);

  const hasInspection =
    !!inspectedFilename && (inspectionStatus === "valid" || inspectionStatus === "warning");
  const hasDryRun = dryRunStatus === "ready" || dryRunStatus === "readyWithWarnings";
  const hasToken = token.length > 0;
  const isConfirmed = confirmationText === RESTORE_EXECUTION_CONFIRMATION_TEXT;

  const canAttempt = hasInspection && hasDryRun && hasToken && isConfirmed && execState === "idle";

  function clearSensitiveState() {
    setToken("");
    if (tokenRef.current) {
      tokenRef.current.value = "";
    }
  }

  async function handleAttempt() {
    if (!canAttempt || !packagePath || !inspectedFilename) return;
    setExecState("running");
    setResult(null);

    try {
      const res = await service.runRestoreExecution({
        packageFilename: inspectedFilename,
        packagePath,
        packageValidationStatus: inspectionStatus ?? "",
        dryRunStatus: dryRunStatus ?? "",
        targetMode,
        targetBaseName: targetBaseName || undefined,
        token,
        confirmation: confirmationText,
      });
      setResult(res);
    } finally {
      clearSensitiveState();
      setExecState("done");
    }
  }

  function handleCancel() {
    clearSensitiveState();
    setConfirmationText("");
    setResult(null);
    setExecState("idle");
  }

  return (
    <div data-testid="restore-execution-gate-panel">
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
          Restore Execution
        </p>
      </div>

      {/* Not-enabled notice */}
      <div
        className="notice notice-warning"
        role="note"
        aria-label="Restore execution not enabled"
        data-testid="execution-not-enabled-notice"
        style={{ fontSize: "var(--text-xs)", marginBottom: "var(--space-4)" }}
      >
        <span>
          Restore execution is not enabled in this version. The safety gate is active but the write
          engine is disabled. No Airtable changes will be made.
        </span>
      </div>

      {/* Prerequisites checklist */}
      <div
        style={{
          marginBottom: "var(--space-4)",
          display: "flex",
          flexDirection: "column",
          gap: "var(--space-2)",
        }}
        data-testid="prerequisites-checklist"
      >
        <p
          style={{
            fontSize: "var(--text-xs)",
            fontWeight: 600,
            color: "var(--color-text-muted)",
            textTransform: "uppercase",
            letterSpacing: "0.06em",
            margin: 0,
            marginBottom: "var(--space-1)",
          }}
        >
          Prerequisites
        </p>
        <PrerequisiteRow done={hasInspection} label="Package inspected and valid" />
        <PrerequisiteRow done={hasDryRun} label="Restore plan preview ready" />
        <PrerequisiteRow done={!!targetMode} label="Target mode selected" />
        <PrerequisiteRow done={hasToken} label="Access token provided" />
        <PrerequisiteRow done={isConfirmed} label="Confirmation text entered" />
      </div>

      {/* Token input */}
      <div className="form-field" style={{ marginBottom: "var(--space-4)" }}>
        <label className="form-label" htmlFor="restore-exec-token">
          Access Token{" "}
          <span style={{ color: "var(--color-text-muted)" }}>(required for this action)</span>
        </label>
        <input
          id="restore-exec-token"
          ref={tokenRef}
          type="password"
          className="form-input"
          value={token}
          onChange={(e) => setToken(e.target.value)}
          disabled={execState === "running"}
          placeholder="Enter your Airtable personal access token"
          data-testid="restore-exec-token-input"
          aria-label="Airtable access token for restore execution"
          autoComplete="off"
        />
        <p
          style={{
            fontSize: "var(--text-xs)",
            color: "var(--color-text-muted)",
            marginTop: "var(--space-1)",
          }}
        >
          The token is used only for this action and is not stored.
        </p>
      </div>

      {/* Confirmation input */}
      <div className="form-field" style={{ marginBottom: "var(--space-4)" }}>
        <label className="form-label" htmlFor="restore-exec-confirmation">
          Confirmation
        </label>
        <input
          id="restore-exec-confirmation"
          type="text"
          className="form-input"
          value={confirmationText}
          onChange={(e) => setConfirmationText(e.target.value)}
          disabled={execState === "running"}
          placeholder={`Type "${RESTORE_EXECUTION_CONFIRMATION_TEXT}" to proceed`}
          data-testid="restore-exec-confirmation-input"
          aria-label="Confirmation text for restore execution"
          autoComplete="off"
        />
        <p
          style={{
            fontSize: "var(--text-xs)",
            color: "var(--color-text-muted)",
            marginTop: "var(--space-1)",
          }}
        >
          Type <strong>{RESTORE_EXECUTION_CONFIRMATION_TEXT}</strong> exactly to enable the button.
        </p>
      </div>

      {/* Action buttons */}
      <div
        style={{
          display: "flex",
          gap: "var(--space-3)",
          alignItems: "center",
          marginBottom: "var(--space-4)",
        }}
      >
        <button
          type="button"
          className="btn btn-secondary"
          onClick={handleAttempt}
          disabled={!canAttempt}
          data-testid="attempt-restore-button"
          aria-label="Attempt restore (gate validation only — write engine disabled)"
        >
          {execState === "running" ? "Checking gate…" : "Attempt Restore"}
        </button>

        {(execState === "running" || execState === "done") && (
          <button
            type="button"
            className="btn btn-ghost"
            onClick={handleCancel}
            disabled={execState === "running"}
            data-testid="cancel-restore-button"
            aria-label="Cancel and clear token"
          >
            Cancel
          </button>
        )}
      </div>

      {/* Result */}
      {execState === "done" && result !== null && <ExecutionResultPanel result={result} />}
    </div>
  );
}

function ExecutionResultPanel({ result }: { result: RestoreExecutionResult }) {
  const isDisabled = result.status === "readyButDisabled";
  const isBlocked = result.status === "blocked";

  return (
    <div
      data-testid="execution-result-panel"
      style={{ display: "flex", flexDirection: "column", gap: "var(--space-3)" }}
    >
      {/* No-changes safety statement */}
      <div
        className="notice notice-info"
        role="note"
        aria-label="No Airtable changes were made"
        data-testid="execution-no-changes-notice"
        style={{ fontSize: "var(--text-xs)" }}
      >
        <span>No Airtable changes were made.</span>
      </div>

      {/* Status and filename */}
      <div
        style={{ display: "flex", alignItems: "center", gap: "var(--space-3)", flexWrap: "wrap" }}
      >
        {result.filename && (
          <span
            style={{ fontWeight: 600, fontSize: "var(--text-sm)" }}
            data-testid="execution-result-filename"
          >
            {result.filename}
          </span>
        )}
        <span
          className={`badge ${isDisabled ? "badge-warning" : isBlocked ? "badge-danger" : "badge-warning"}`}
          data-testid="execution-result-status"
          data-execution-status={result.status}
        >
          {isDisabled ? "Disabled" : isBlocked ? "Blocked" : result.status}
        </span>
      </div>

      {/* Message */}
      <div
        className={`notice ${isDisabled ? "notice-warning" : "notice-danger"}`}
        data-testid="execution-result-message"
        style={{ fontSize: "var(--text-xs)" }}
      >
        <span>{result.message}</span>
      </div>

      {/* Not-implemented statement */}
      {isDisabled && (
        <div
          className="notice notice-neutral"
          role="note"
          aria-label="Restore execution not implemented"
          data-testid="execution-not-implemented-notice"
          style={{ fontSize: "var(--text-xs)" }}
        >
          <span>Restore execution is not enabled in this version.</span>
        </div>
      )}

      {/* Errors */}
      {result.errors.length > 0 && (
        <div data-testid="execution-result-errors">
          {result.errors.map((e, i) => (
            <div key={i} className="notice notice-danger" style={{ fontSize: "var(--text-xs)" }}>
              <strong>{e.code}</strong>: {e.message}
            </div>
          ))}
        </div>
      )}

      {/* Warnings */}
      {result.warnings.length > 0 && (
        <div data-testid="execution-result-warnings">
          {result.warnings.map((w, i) => (
            <div key={i} className="notice notice-warning" style={{ fontSize: "var(--text-xs)" }}>
              <strong>{w.code}</strong>: {w.message}
            </div>
          ))}
        </div>
      )}
    </div>
  );
}

function PrerequisiteRow({ done, label }: { done: boolean; label: string }) {
  return (
    <div
      style={{
        display: "flex",
        alignItems: "center",
        gap: "var(--space-2)",
        fontSize: "var(--text-xs)",
      }}
      data-testid="prerequisite-row"
    >
      <span
        style={{
          color: done ? "var(--color-text-success)" : "var(--color-text-muted)",
          fontWeight: 600,
          width: 12,
          textAlign: "center",
        }}
      >
        {done ? "✓" : "—"}
      </span>
      <span style={{ color: done ? "var(--color-text-primary)" : "var(--color-text-muted)" }}>
        {label}
      </span>
    </div>
  );
}
