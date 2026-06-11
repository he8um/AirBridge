import { useState, useRef } from "react";
import type { AirBridgeService } from "../../services/airBridgeService";
import type {
  BackupPlan,
  RecordsExportPlan,
  RunBackupCommandResponse,
  RunBackupTableSpec,
} from "../../backend/types";
import { BackupConfirmationBox } from "./BackupConfirmationBox";
import { BackupJobResultCard } from "./BackupJobResultCard";
import { pickBackupOutputPath } from "./BackupOutputPicker";
import {
  BACKUP_CONFIRMATION_TEXT,
  getDisplayFileName,
  hasAirbridgeExtension,
  redactOutputPath,
} from "./backupExecutionHelpers";

interface BackupExecutionPanelProps {
  backupPlan: BackupPlan | null;
  exportPlan: RecordsExportPlan | null;
  service: AirBridgeService;
}

type RunState = "idle" | "running" | "done";

export function BackupExecutionPanel({
  backupPlan,
  exportPlan,
  service,
}: BackupExecutionPanelProps) {
  const [outputPath, setOutputPath] = useState<string | null>(null);
  const [pathValidating, setPathValidating] = useState(false);
  const [pathValid, setPathValid] = useState(false);
  const [pathErrorMessage, setPathErrorMessage] = useState<string | null>(null);
  const [confirmationText, setConfirmationText] = useState("");
  const [token, setToken] = useState("");
  const [runState, setRunState] = useState<RunState>("idle");
  const [runError, setRunError] = useState<string | null>(null);
  const [runResponse, setRunResponse] = useState<RunBackupCommandResponse | null>(null);
  const [activeJobId, setActiveJobId] = useState<string | null>(null);
  const tokenRef = useRef<HTMLInputElement>(null);

  const displayFileName = outputPath ? getDisplayFileName(outputPath) : null;
  const isConfirmed = confirmationText === BACKUP_CONFIRMATION_TEXT;
  const hasToken = token.length > 0;
  const hasPlans = backupPlan !== null && exportPlan !== null;

  const canRun = hasPlans && pathValid && isConfirmed && hasToken && runState === "idle";

  async function handlePickPath() {
    const baseName = backupPlan?.baseName ?? "backup";
    const suggested = `${baseName}.airbridge`;
    const picked = await pickBackupOutputPath(suggested);
    if (!picked) return;

    setOutputPath(picked);
    setPathValid(false);
    setPathErrorMessage(null);
    setRunResponse(null);
    setRunError(null);

    if (!hasAirbridgeExtension(picked)) {
      setPathErrorMessage("File must have a .airbridge extension.");
      return;
    }

    setPathValidating(true);
    try {
      const result = await service.validateBackupOutputPath(picked);
      setPathValid(result.valid);
      setPathErrorMessage(result.valid ? null : (result.errorMessage ?? "Invalid output path."));
    } finally {
      setPathValidating(false);
    }
  }

  function clearSensitiveState() {
    setToken("");
    setConfirmationText("");
    if (tokenRef.current) tokenRef.current.value = "";
  }

  async function handleRun() {
    if (!canRun || !outputPath || !backupPlan || !exportPlan) return;

    const pendingJobId = `job-ui-${Date.now()}`;
    setRunState("running");
    setRunError(null);
    setRunResponse(null);
    setActiveJobId(pendingJobId);

    const tableSpecs: RunBackupTableSpec[] = backupPlan.tables.map((t) => ({
      tableId: t.id,
      tableName: t.name,
      linkedFieldNames: t.fields
        .filter((f) => f.fieldType === "multipleRecordLinks")
        .map((f) => f.name),
      attachmentFieldNames: t.fields
        .filter((f) => f.fieldType === "multipleAttachments")
        .map((f) => f.name),
    }));

    const capturedToken = token;

    try {
      const response = await service.runBackupJob({
        token: capturedToken,
        outputPath,
        confirmation: confirmationText,
        baseId: backupPlan.baseId,
        baseName: backupPlan.baseName,
        baseJson: [],
        schemaJson: [],
        tableSpecs,
        pageSize: exportPlan.pageSize,
        jobId: pendingJobId,
      });
      setActiveJobId(response.jobResult?.jobId ?? pendingJobId);
      setRunResponse(response);
    } catch {
      setRunError("Backup job failed unexpectedly. Check the output and try again.");
    } finally {
      clearSensitiveState();
      setRunState("done");
    }
  }

  async function handleCancel() {
    if (runState !== "running") return;
    const jobId = activeJobId;
    clearSensitiveState();
    setActiveJobId(null);
    setRunState("done");
    if (jobId !== null) {
      await service.cancelBackupJob(jobId);
    }
  }

  function handleReset() {
    setOutputPath(null);
    setPathValid(false);
    setPathErrorMessage(null);
    setConfirmationText("");
    setToken("");
    setRunState("idle");
    setRunError(null);
    setRunResponse(null);
    setActiveJobId(null);
  }

  return (
    <div
      className="card"
      style={{ maxWidth: 560, display: "flex", flexDirection: "column", gap: "var(--space-5)" }}
      data-testid="backup-execution-panel"
    >
      {/* Safety notice */}
      <div
        className="notice-neutral"
        style={{
          padding: "var(--space-3) var(--space-4)",
          borderRadius: "var(--radius-md)",
          fontSize: "var(--text-xs)",
          color: "var(--color-text-muted)",
          display: "flex",
          flexDirection: "column",
          gap: "var(--space-1)",
        }}
      >
        <span>The full output path is not displayed.</span>
        <span>The token is not stored.</span>
        <span>Backup creation runs only after confirmation.</span>
      </div>

      {/* Plans readiness */}
      {!hasPlans && (
        <p
          style={{ fontSize: "var(--text-sm)", color: "var(--color-text-muted)", margin: 0 }}
          aria-live="polite"
        >
          Generate a backup plan and records export plan above before running a backup.
        </p>
      )}

      {/* Output file selection */}
      <div
        className="form-field"
        style={{ display: "flex", flexDirection: "column", gap: "var(--space-2)" }}
      >
        <label className="form-label">Output File</label>
        <div style={{ display: "flex", gap: "var(--space-2)", alignItems: "center" }}>
          <button
            type="button"
            className="btn btn-secondary"
            onClick={handlePickPath}
            disabled={!hasPlans || runState === "running"}
            aria-label="Choose output file location"
            data-testid="pick-output-path-button"
          >
            Choose File…
          </button>
          {displayFileName && (
            <span
              style={{
                fontSize: "var(--text-sm)",
                fontFamily: "var(--font-mono)",
                overflow: "hidden",
                textOverflow: "ellipsis",
                whiteSpace: "nowrap",
              }}
              aria-label="Selected output filename"
              data-testid="selected-filename-display"
            >
              {displayFileName}
            </span>
          )}
        </div>

        {/* Path validation status */}
        {outputPath && (
          <p
            style={{
              fontSize: "var(--text-xs)",
              margin: 0,
              color: pathValidating
                ? "var(--color-text-muted)"
                : pathValid
                  ? "var(--color-success, #27ae60)"
                  : "var(--color-error, #c0392b)",
            }}
            aria-live="polite"
            data-testid="path-validation-status"
          >
            {pathValidating
              ? "Validating path…"
              : pathValid
                ? `Path valid — ${redactOutputPath(outputPath)}`
                : (pathErrorMessage ?? "Invalid path.")}
          </p>
        )}
      </div>

      {/* Token field */}
      <div
        className="form-field"
        style={{ display: "flex", flexDirection: "column", gap: "var(--space-2)" }}
      >
        <label htmlFor="backup-token-input" className="form-label">
          Personal Access Token for this backup run
        </label>
        <p
          style={{ fontSize: "var(--text-xs)", color: "var(--color-text-muted)", margin: 0 }}
          id="backup-token-hint"
        >
          Entered here only. Not stored. Cleared after the run completes or fails.
        </p>
        <input
          id="backup-token-input"
          ref={tokenRef}
          type="password"
          className="form-input"
          value={token}
          onChange={(e) => setToken(e.target.value)}
          disabled={!hasPlans || runState !== "idle"}
          aria-describedby="backup-token-hint"
          aria-label="Personal access token for this backup run"
          autoComplete="off"
          data-testid="backup-token-input"
        />
      </div>

      {/* Confirmation */}
      <BackupConfirmationBox
        value={confirmationText}
        onChange={setConfirmationText}
        disabled={!hasPlans || runState !== "idle"}
      />

      {/* Run / Cancel / Reset buttons */}
      <div style={{ display: "flex", gap: "var(--space-3)", flexWrap: "wrap" }}>
        <button
          type="button"
          className="btn btn-primary"
          onClick={handleRun}
          disabled={!canRun}
          aria-label="Run backup job"
          data-testid="run-backup-button"
        >
          {runState === "running" ? "Running backup…" : "Run Backup"}
        </button>
        {runState === "running" && (
          <button
            type="button"
            className="btn btn-secondary"
            onClick={handleCancel}
            aria-label="Cancel backup job"
            data-testid="cancel-backup-button"
          >
            Cancel
          </button>
        )}
        {runState === "done" && (
          <button
            type="button"
            className="btn btn-secondary"
            onClick={handleReset}
            aria-label="Reset backup execution panel"
            data-testid="reset-backup-button"
          >
            Reset
          </button>
        )}
      </div>

      {/* Run error (unexpected exception) */}
      {runError && (
        <p
          style={{ fontSize: "var(--text-sm)", color: "var(--color-error, #c0392b)", margin: 0 }}
          role="alert"
          data-testid="backup-run-error"
        >
          {runError}
        </p>
      )}

      {/* Result card */}
      {runResponse && <BackupJobResultCard response={runResponse} />}
    </div>
  );
}
