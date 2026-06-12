import { useState } from "react";
import type { AirBridgeService } from "../../services/airBridgeService";
import type { BackupPackageInspectionResult } from "../../backend/types";
import { pickBackupPackagePath } from "./PackageInspectionPicker";

interface PackageInspectionPanelProps {
  service: AirBridgeService;
  /** Optional callback invoked after a successful inspection. Receives the result and the full path (for passing to commands). */
  onInspected?: (result: BackupPackageInspectionResult, path: string) => void;
}

type InspectionState = "idle" | "loading" | "done";

export function PackageInspectionPanel({ service, onInspected }: PackageInspectionPanelProps) {
  const [inspectionState, setInspectionState] = useState<InspectionState>("idle");
  const [result, setResult] = useState<BackupPackageInspectionResult | null>(null);

  async function handleSelect() {
    const path = await pickBackupPackagePath();
    if (path === null) return;

    setInspectionState("loading");
    setResult(null);

    try {
      const inspection = await service.inspectBackupPackage(path);
      setResult(inspection);
      onInspected?.(inspection, path);
    } finally {
      setInspectionState("done");
    }
  }

  return (
    <div data-testid="package-inspection-panel">
      <div
        style={{
          display: "flex",
          alignItems: "center",
          justifyContent: "space-between",
          marginBottom: "var(--space-3)",
        }}
      >
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
          Inspection
        </p>
        <button
          type="button"
          className="btn btn-secondary"
          onClick={handleSelect}
          disabled={inspectionState === "loading"}
          data-testid="select-package-button"
          aria-label="Select .airbridge file to inspect"
          style={{ fontSize: "var(--text-xs)" }}
        >
          <svg
            width="12"
            height="12"
            viewBox="0 0 24 24"
            fill="none"
            stroke="currentColor"
            strokeWidth="2"
            strokeLinecap="round"
            strokeLinejoin="round"
            aria-hidden="true"
          >
            <path d="M14 2H6a2 2 0 00-2 2v16a2 2 0 002 2h12a2 2 0 002-2V8z" />
            <polyline points="14 2 14 8 20 8" />
          </svg>
          {inspectionState === "loading" ? "Inspecting…" : "Select Package"}
        </button>
      </div>

      {inspectionState === "idle" && (
        <div
          className="notice notice-neutral"
          style={{ justifyContent: "center", textAlign: "center" }}
          role="status"
          aria-label="Inspection idle"
          data-testid="inspection-idle-notice"
        >
          <span>Select a backup file to inspect its contents.</span>
        </div>
      )}

      {inspectionState === "loading" && (
        <div
          className="notice notice-neutral"
          role="status"
          aria-label="Inspection loading"
          data-testid="inspection-loading-notice"
        >
          <span>Inspecting package…</span>
        </div>
      )}

      {inspectionState === "done" && result !== null && <InspectionResultPanel result={result} />}
    </div>
  );
}

interface InspectionResultPanelProps {
  result: BackupPackageInspectionResult;
}

function InspectionResultPanel({ result }: InspectionResultPanelProps) {
  const isValid = result.validationStatus === "valid";
  const isWarning = result.validationStatus === "warning";

  return (
    <div
      data-testid="inspection-result-panel"
      style={{ display: "flex", flexDirection: "column", gap: "var(--space-4)" }}
    >
      {/* Read-only notice */}
      <div
        className="notice notice-info"
        role="note"
        aria-label="Inspection is read-only"
        data-testid="inspection-readonly-notice"
        style={{ fontSize: "var(--text-xs)" }}
      >
        <span>Inspection is read-only. No files are extracted. Restore is not started.</span>
      </div>

      {/* Filename and status row */}
      <div
        style={{ display: "flex", alignItems: "center", gap: "var(--space-3)", flexWrap: "wrap" }}
      >
        <span
          style={{ fontWeight: 600, fontSize: "var(--text-sm)" }}
          data-testid="inspection-filename"
        >
          {result.filename}
        </span>
        <span
          className={`badge ${isValid ? "badge-success" : isWarning ? "badge-warning" : "badge-danger"}`}
          data-testid="inspection-validation-status"
          data-validation-status={result.validationStatus}
        >
          {result.validationStatus}
        </span>
      </div>

      {/* Errors */}
      {result.errors.length > 0 && (
        <div data-testid="inspection-errors">
          {result.errors.map((e, i) => (
            <div key={i} className="notice notice-danger" style={{ fontSize: "var(--text-xs)" }}>
              <strong>{e.code}</strong>: {e.message}
            </div>
          ))}
        </div>
      )}

      {/* Warnings */}
      {result.warnings.length > 0 && (
        <div data-testid="inspection-warnings">
          {result.warnings.map((w, i) => (
            <div key={i} className="notice notice-warning" style={{ fontSize: "var(--text-xs)" }}>
              <strong>{w.code}</strong>: {w.message}
            </div>
          ))}
        </div>
      )}

      {/* Manifest summary */}
      {result.manifest && (
        <div data-testid="inspection-manifest-summary">
          <SummarySection label="Format">
            {result.manifest.format} v{result.manifest.formatVersion}
          </SummarySection>
          <SummarySection label="Source">
            {result.manifest.provider} / {result.manifest.baseName}
          </SummarySection>
          <SummarySection label="Base ID">{result.manifest.baseId}</SummarySection>
          <SummarySection label="Created">{result.manifest.createdAt}</SummarySection>
          <SummarySection label="App version">{result.manifest.appVersion}</SummarySection>
        </div>
      )}

      {/* Contents summary */}
      {result.contents && (
        <div
          data-testid="inspection-contents-summary"
          style={{ display: "flex", gap: "var(--space-4)", flexWrap: "wrap" }}
        >
          <CountBadge label="Tables" value={result.contents.tableCount} />
          <CountBadge label="Fields" value={result.contents.fieldCount} />
          <CountBadge label="Records" value={result.contents.recordCount} />
          <CountBadge label="Entries" value={result.entryCount} />
          <CountBadge label="Checksums" value={result.checksums?.checksumCount ?? 0} />
        </div>
      )}

      {/* Security summary */}
      {result.security && (
        <div data-testid="inspection-security-summary">
          <FlagRow label="Contains record data" value={result.security.containsRecordData} />
          <FlagRow
            label="Contains attachment URLs"
            value={result.security.containsAttachmentUrls}
          />
          <FlagRow label="Encrypted" value={result.security.encrypted} />
          {result.security.redactionsApplied.length > 0 && (
            <SummarySection label="Redactions">
              {result.security.redactionsApplied.join(", ")}
            </SummarySection>
          )}
        </div>
      )}

      {/* Checksum validity */}
      {result.checksums && !result.checksums.allValid && (
        <div
          className="notice notice-danger"
          role="alert"
          data-testid="inspection-checksum-warning"
          style={{ fontSize: "var(--text-xs)" }}
        >
          One or more checksums did not match. The package may have been modified after creation.
        </div>
      )}
    </div>
  );
}

function SummarySection({ label, children }: { label: string; children: React.ReactNode }) {
  return (
    <div
      style={{
        display: "flex",
        gap: "var(--space-2)",
        fontSize: "var(--text-xs)",
        marginBottom: "var(--space-1)",
      }}
    >
      <span style={{ color: "var(--color-text-muted)", minWidth: 100 }}>{label}</span>
      <span>{children}</span>
    </div>
  );
}

function CountBadge({ label, value }: { label: string; value: number }) {
  return (
    <div style={{ display: "flex", flexDirection: "column", gap: 2, alignItems: "center" }}>
      <span style={{ fontSize: "var(--text-base)", fontWeight: 600 }}>{value}</span>
      <span style={{ fontSize: "var(--text-xs)", color: "var(--color-text-muted)" }}>{label}</span>
    </div>
  );
}

function FlagRow({ label, value }: { label: string; value: boolean }) {
  return (
    <div
      style={{
        display: "flex",
        gap: "var(--space-2)",
        fontSize: "var(--text-xs)",
        marginBottom: "var(--space-1)",
      }}
    >
      <span style={{ color: "var(--color-text-muted)", minWidth: 160 }}>{label}</span>
      <span>{value ? "Yes" : "No"}</span>
    </div>
  );
}
