import type { RunBackupCommandResponse } from "../../backend/types";

interface BackupJobResultCardProps {
  response: RunBackupCommandResponse;
}

export function BackupJobResultCard({ response }: BackupJobResultCardProps) {
  const { success, packageFilename, safetyErrors, jobResult, pathValidation } = response;

  return (
    <div
      className={`card ${success ? "notice-neutral" : ""}`}
      style={{ display: "flex", flexDirection: "column", gap: "var(--space-3)" }}
      data-testid="backup-job-result-card"
      role="status"
      aria-live="polite"
    >
      {/* Overall status */}
      <p style={{ fontSize: "var(--text-sm)", fontWeight: 600, margin: 0 }}>
        {success ? "Backup succeeded" : "Backup failed"}
      </p>

      {/* Safety errors */}
      {safetyErrors && safetyErrors.length > 0 && (
        <ul
          style={{
            listStyle: "none",
            margin: 0,
            padding: 0,
            display: "flex",
            flexDirection: "column",
            gap: "var(--space-1)",
          }}
          aria-label="Safety errors"
        >
          {safetyErrors.map((e, i) => (
            <li
              key={i}
              style={{ fontSize: "var(--text-sm)", color: "var(--color-error, #c0392b)" }}
            >
              <span style={{ fontWeight: 500 }}>{e.code}</span>
              {": "}
              {e.message}
            </li>
          ))}
        </ul>
      )}

      {/* Path validation error */}
      {!pathValidation.valid && pathValidation.errorMessage && (
        <p
          style={{
            fontSize: "var(--text-sm)",
            color: "var(--color-error, #c0392b)",
            margin: 0,
          }}
          role="alert"
        >
          Output path: {pathValidation.errorMessage}
        </p>
      )}

      {/* Package filename — filename-only, no absolute path */}
      {success && packageFilename && (
        <p style={{ fontSize: "var(--text-sm)", margin: 0 }}>
          Package: <span style={{ fontFamily: "var(--font-mono)" }}>{packageFilename}</span>
        </p>
      )}

      {/* Job result summary */}
      {jobResult && (
        <div
          style={{ display: "flex", flexDirection: "column", gap: "var(--space-2)" }}
          aria-label="Job result summary"
        >
          <p style={{ fontSize: "var(--text-xs)", color: "var(--color-text-muted)", margin: 0 }}>
            Job ID: <span style={{ fontFamily: "var(--font-mono)" }}>{jobResult.jobId}</span>
            {" · "}
            Status: {jobResult.status}
            {" · "}
            Base: {jobResult.baseName}
          </p>

          {/* Package summary */}
          {jobResult.packageSummary && (
            <p style={{ fontSize: "var(--text-xs)", color: "var(--color-text-muted)", margin: 0 }}>
              {jobResult.packageSummary.tableCount}{" "}
              {jobResult.packageSummary.tableCount === 1 ? "table" : "tables"}
              {" · "}
              {jobResult.packageSummary.recordCount} records
              {" · "}
              {jobResult.packageSummary.entryCount} entries
            </p>
          )}

          {/* Validation summary */}
          {jobResult.validationSummary && (
            <p style={{ fontSize: "var(--text-xs)", color: "var(--color-text-muted)", margin: 0 }}>
              Validation: {jobResult.validationSummary.status}
              {jobResult.validationSummary.errorCount > 0 &&
                ` · ${jobResult.validationSummary.errorCount} error${jobResult.validationSummary.errorCount !== 1 ? "s" : ""}`}
              {jobResult.validationSummary.warningCount > 0 &&
                ` · ${jobResult.validationSummary.warningCount} warning${jobResult.validationSummary.warningCount !== 1 ? "s" : ""}`}
            </p>
          )}

          {/* Job-level warnings */}
          {jobResult.warnings.length > 0 && (
            <ul
              style={{
                listStyle: "none",
                margin: 0,
                padding: 0,
                display: "flex",
                flexDirection: "column",
                gap: "var(--space-1)",
              }}
              aria-label="Job warnings"
            >
              {jobResult.warnings.map((w, i) => (
                <li
                  key={i}
                  style={{ fontSize: "var(--text-xs)", color: "var(--color-warning, #d68910)" }}
                >
                  {w.code}: {w.message}
                </li>
              ))}
            </ul>
          )}

          {/* Job-level errors */}
          {jobResult.errors.length > 0 && (
            <ul
              style={{
                listStyle: "none",
                margin: 0,
                padding: 0,
                display: "flex",
                flexDirection: "column",
                gap: "var(--space-1)",
              }}
              aria-label="Job errors"
            >
              {jobResult.errors.map((e, i) => (
                <li
                  key={i}
                  style={{ fontSize: "var(--text-xs)", color: "var(--color-error, #c0392b)" }}
                >
                  {e.code}: {e.message}
                  {e.recoverable && (
                    <span style={{ color: "var(--color-text-muted)" }}> (recoverable)</span>
                  )}
                </li>
              ))}
            </ul>
          )}
        </div>
      )}
    </div>
  );
}
