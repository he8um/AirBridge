import { useState } from "react";
import type { AirBridgeService } from "../../services/airBridgeService";
import type {
  RestoreDryRunPlan,
  RestoreFieldCompatibility,
  RestoreFieldPlan,
  RestoreTablePlan,
  RestoreTargetMode,
} from "../../backend/types";
import { pickBackupPackagePath } from "./PackageInspectionPicker";

interface RestoreDryRunPanelProps {
  service: AirBridgeService;
  /** Optional callback invoked when a plan is generated. Receives the plan, mode, and optional base name. */
  onPlanReady?: (
    plan: RestoreDryRunPlan,
    targetMode: RestoreTargetMode,
    targetBaseName: string | undefined,
  ) => void;
}

type PlanState = "idle" | "loading" | "done";

export function RestoreDryRunPanel({ service, onPlanReady }: RestoreDryRunPanelProps) {
  const [planState, setPlanState] = useState<PlanState>("idle");
  const [selectedPath, setSelectedPath] = useState<string | null>(null);
  const [targetMode, setTargetMode] = useState<RestoreTargetMode>("newBase");
  const [targetBaseName, setTargetBaseName] = useState("");
  const [plan, setPlan] = useState<RestoreDryRunPlan | null>(null);

  async function handleSelectFile() {
    const path = await pickBackupPackagePath();
    if (path === null) return;
    setSelectedPath(path);
    setPlan(null);
    setPlanState("idle");
  }

  async function handleGeneratePlan() {
    if (selectedPath === null) return;
    setPlanState("loading");
    setPlan(null);
    try {
      const result = await service.createRestoreDryRunPlan({
        path: selectedPath,
        targetMode,
        targetBaseName: targetBaseName.trim() || undefined,
      });
      setPlan(result);
      onPlanReady?.(result, targetMode, targetBaseName.trim() || undefined);
    } finally {
      setPlanState("done");
    }
  }

  const selectedFilename = selectedPath
    ? (selectedPath.split("/").pop()?.split("\\").pop() ?? "")
    : null;

  return (
    <div data-testid="restore-dry-run-panel">
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
          Restore Plan Preview
        </p>
      </div>

      {/* Read-only notice */}
      <div
        className="notice notice-info"
        role="note"
        aria-label="Dry-run is read-only"
        data-testid="dry-run-readonly-notice"
        style={{ fontSize: "var(--text-xs)", marginBottom: "var(--space-4)" }}
      >
        <span>This generates a plan only. No Airtable changes are made. No token is required.</span>
      </div>

      {/* File selector row */}
      <div className="form-field" style={{ marginBottom: "var(--space-4)" }}>
        <label className="form-label">Backup Package</label>
        <div style={{ display: "flex", gap: "var(--space-2)", alignItems: "center" }}>
          <span
            style={{
              flex: 1,
              fontSize: "var(--text-sm)",
              color: selectedFilename ? "var(--color-text-primary)" : "var(--color-text-muted)",
              fontStyle: selectedFilename ? "normal" : "italic",
            }}
            data-testid="dry-run-selected-filename"
          >
            {selectedFilename ?? "No file selected"}
          </span>
          <button
            type="button"
            className="btn btn-secondary"
            onClick={handleSelectFile}
            disabled={planState === "loading"}
            data-testid="dry-run-select-file-button"
            aria-label="Select .airbridge backup file for restore planning"
            style={{ fontSize: "var(--text-xs)", flexShrink: 0 }}
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
            Choose File
          </button>
        </div>
      </div>

      {/* Target mode selector */}
      <div className="form-field" style={{ marginBottom: "var(--space-4)" }}>
        <label className="form-label" htmlFor="restore-target-mode">
          Target Mode
        </label>
        <select
          id="restore-target-mode"
          className="form-input"
          value={targetMode}
          onChange={(e) => setTargetMode(e.target.value as RestoreTargetMode)}
          disabled={planState === "loading"}
          data-testid="restore-target-mode-select"
          aria-label="Select restore target mode"
        >
          <option value="newBase">New base</option>
          <option value="emptyExistingBase">Empty existing base</option>
        </select>
      </div>

      {/* Optional base name */}
      <div className="form-field" style={{ marginBottom: "var(--space-4)" }}>
        <label className="form-label" htmlFor="restore-target-base-name">
          Target Base Name <span style={{ color: "var(--color-text-muted)" }}>(optional)</span>
        </label>
        <input
          id="restore-target-base-name"
          type="text"
          className="form-input"
          value={targetBaseName}
          onChange={(e) => setTargetBaseName(e.target.value)}
          disabled={planState === "loading"}
          placeholder="Leave blank to use the original base name"
          data-testid="restore-target-base-name-input"
          aria-label="Optional name for the restored base"
        />
      </div>

      {/* Generate plan button */}
      <div style={{ marginBottom: "var(--space-4)" }}>
        <button
          type="button"
          className="btn btn-secondary"
          onClick={handleGeneratePlan}
          disabled={selectedPath === null || planState === "loading"}
          data-testid="generate-dry-run-plan-button"
          aria-label="Generate restore plan"
        >
          {planState === "loading" ? "Generating plan…" : "Generate Restore Plan"}
        </button>
      </div>

      {/* Loading state */}
      {planState === "loading" && (
        <div
          className="notice notice-neutral"
          role="status"
          aria-label="Plan loading"
          data-testid="dry-run-loading-notice"
        >
          <span>Generating plan…</span>
        </div>
      )}

      {/* Plan result */}
      {planState === "done" && plan !== null && <DryRunPlanResult plan={plan} />}
    </div>
  );
}

interface DryRunPlanResultProps {
  plan: RestoreDryRunPlan;
}

function DryRunPlanResult({ plan }: DryRunPlanResultProps) {
  const isBlocked = plan.status === "blocked";
  const isWarning = plan.status === "readyWithWarnings";

  return (
    <div
      data-testid="dry-run-plan-result"
      style={{ display: "flex", flexDirection: "column", gap: "var(--space-4)" }}
    >
      {/* No-changes safety statement */}
      <div
        className="notice notice-info"
        role="note"
        aria-label="No Airtable changes were made"
        data-testid="dry-run-no-changes-notice"
        style={{ fontSize: "var(--text-xs)" }}
      >
        <span>No Airtable changes were made. This is a plan preview only.</span>
      </div>

      {/* Status badge and filename */}
      <div
        style={{ display: "flex", alignItems: "center", gap: "var(--space-3)", flexWrap: "wrap" }}
      >
        <span
          style={{ fontWeight: 600, fontSize: "var(--text-sm)" }}
          data-testid="dry-run-plan-filename"
        >
          {plan.filename}
        </span>
        <span
          className={`badge ${isBlocked ? "badge-danger" : isWarning ? "badge-warning" : "badge-success"}`}
          data-testid="dry-run-plan-status"
          data-plan-status={plan.status}
        >
          {isBlocked ? "Blocked" : isWarning ? "Ready with warnings" : "Ready"}
        </span>
      </div>

      {/* Errors */}
      {plan.errors.length > 0 && (
        <div data-testid="dry-run-plan-errors">
          {plan.errors.map((e, i) => (
            <div key={i} className="notice notice-danger" style={{ fontSize: "var(--text-xs)" }}>
              <strong>{e.code}</strong>: {e.message}
            </div>
          ))}
        </div>
      )}

      {/* Package summary */}
      {plan.packageSummary && (
        <div
          data-testid="dry-run-package-summary"
          style={{ display: "flex", flexDirection: "column", gap: "var(--space-1)" }}
        >
          <SectionLabel>Package Summary</SectionLabel>
          <SummaryRow label="Source">{plan.packageSummary.baseName}</SummaryRow>
          <SummaryRow label="Provider">{plan.packageSummary.provider}</SummaryRow>
          <SummaryRow label="Format">
            {plan.packageSummary.format} v{plan.packageSummary.formatVersion}
          </SummaryRow>
          <SummaryRow label="Created">{plan.packageSummary.createdAt}</SummaryRow>
          <div
            style={{
              display: "flex",
              gap: "var(--space-4)",
              flexWrap: "wrap",
              marginTop: "var(--space-2)",
            }}
          >
            <CountBadge label="Tables" value={plan.packageSummary.tableCount} />
            <CountBadge label="Fields" value={plan.packageSummary.fieldCount} />
            <CountBadge label="Records" value={plan.packageSummary.recordCount} />
          </div>
        </div>
      )}

      {/* Table plans */}
      {plan.tables.length > 0 && (
        <div data-testid="dry-run-table-plans">
          <SectionLabel>Tables</SectionLabel>
          {plan.tables.map((table) => (
            <TablePlanCard key={table.tableId} table={table} />
          ))}
        </div>
      )}

      {/* Ordering plan */}
      {plan.ordering && (
        <div data-testid="dry-run-ordering-plan">
          <SectionLabel>Import Ordering</SectionLabel>
          <OrderingRow done={plan.ordering.createTablesFirst} label="Create tables first" />
          <OrderingRow
            done={plan.ordering.createFieldsAfterTables}
            label="Create fields after tables"
          />
          <OrderingRow
            done={plan.ordering.importRecordsWithoutLinks}
            label="Import records without linked references"
          />
          <OrderingRow
            done={plan.ordering.applyLinksAfterRecords}
            label="Apply linked references in second pass"
          />
          {plan.ordering.note && (
            <p
              style={{
                fontSize: "var(--text-xs)",
                color: "var(--color-text-muted)",
                marginTop: "var(--space-2)",
              }}
            >
              {plan.ordering.note}
            </p>
          )}
        </div>
      )}

      {/* Warnings */}
      {plan.warnings.length > 0 && (
        <div data-testid="dry-run-plan-warnings">
          <SectionLabel>Warnings</SectionLabel>
          {plan.warnings.map((w, i) => (
            <div
              key={i}
              className="notice notice-warning"
              style={{ fontSize: "var(--text-xs)", marginBottom: "var(--space-2)" }}
            >
              <div>
                <strong>{w.code}</strong>
                {w.tableName && (
                  <span style={{ color: "var(--color-text-muted)" }}> · {w.tableName}</span>
                )}
                {w.fieldName && (
                  <span style={{ color: "var(--color-text-muted)" }}> / {w.fieldName}</span>
                )}
              </div>
              <div>{w.message}</div>
            </div>
          ))}
        </div>
      )}
    </div>
  );
}

function TablePlanCard({ table }: { table: RestoreTablePlan }) {
  return (
    <div
      style={{
        border: "1px solid var(--color-border)",
        borderRadius: "var(--radius-md)",
        padding: "var(--space-3)",
        marginBottom: "var(--space-3)",
      }}
      data-testid={`table-plan-${table.tableId}`}
    >
      <div
        style={{
          display: "flex",
          alignItems: "center",
          gap: "var(--space-2)",
          marginBottom: "var(--space-2)",
        }}
      >
        <span style={{ fontWeight: 600, fontSize: "var(--text-sm)" }}>{table.tableName}</span>
        <span style={{ fontSize: "var(--text-xs)", color: "var(--color-text-muted)" }}>
          {table.fieldCount} field{table.fieldCount !== 1 ? "s" : ""}
        </span>
      </div>

      <div
        style={{
          display: "flex",
          gap: "var(--space-3)",
          flexWrap: "wrap",
          marginBottom: "var(--space-2)",
          fontSize: "var(--text-xs)",
        }}
      >
        <span style={{ color: "var(--color-text-success)" }}>
          {table.restorableFieldCount} restorable
        </span>
        {table.partialFieldCount > 0 && (
          <span style={{ color: "var(--color-text-warning)" }}>
            {table.partialFieldCount} partial
          </span>
        )}
        {table.unsupportedFieldCount > 0 && (
          <span style={{ color: "var(--color-text-danger)" }}>
            {table.unsupportedFieldCount} unsupported
          </span>
        )}
      </div>

      {/* Field list */}
      <div style={{ display: "flex", flexDirection: "column", gap: "var(--space-1)" }}>
        {table.fields.map((field) => (
          <FieldPlanRow key={field.fieldId} field={field} />
        ))}
      </div>

      {/* Linked record plans */}
      {table.linkedRecordPlans.length > 0 && (
        <div
          style={{ marginTop: "var(--space-2)" }}
          data-testid={`linked-record-plans-${table.tableId}`}
        >
          {table.linkedRecordPlans.map((lp) => (
            <div
              key={lp.fieldId}
              className="notice notice-warning"
              style={{ fontSize: "var(--text-xs)", marginBottom: "var(--space-1)" }}
            >
              <strong>Linked field:</strong> {lp.fieldName} → remapping required
            </div>
          ))}
        </div>
      )}

      {/* Attachment plans */}
      {table.attachmentPlans.length > 0 && (
        <div
          style={{ marginTop: "var(--space-2)" }}
          data-testid={`attachment-plans-${table.tableId}`}
        >
          {table.attachmentPlans.map((ap) => (
            <div
              key={ap.fieldId}
              className="notice notice-info"
              style={{ fontSize: "var(--text-xs)", marginBottom: "var(--space-1)" }}
            >
              <strong>Attachment field:</strong> {ap.fieldName} — metadata only
            </div>
          ))}
        </div>
      )}
    </div>
  );
}

function FieldPlanRow({ field }: { field: RestoreFieldPlan }) {
  return (
    <div
      style={{
        display: "flex",
        alignItems: "baseline",
        gap: "var(--space-2)",
        fontSize: "var(--text-xs)",
      }}
      data-testid={`field-plan-${field.fieldId}`}
    >
      <span style={{ color: "var(--color-text-muted)", minWidth: 120 }}>{field.fieldName}</span>
      <span style={{ color: "var(--color-text-muted)" }}>{field.fieldType}</span>
      <CompatibilityTag compatibility={field.compatibility} />
    </div>
  );
}

function CompatibilityTag({ compatibility }: { compatibility: RestoreFieldCompatibility }) {
  const map: Record<RestoreFieldCompatibility, { label: string; cls: string }> = {
    supported: { label: "Supported", cls: "badge-success" },
    partiallySupported: { label: "Partial", cls: "badge-warning" },
    metadataOnly: { label: "Metadata only", cls: "badge-warning" },
    unsupported: { label: "Unsupported", cls: "badge-danger" },
    manualActionRequired: { label: "Manual", cls: "badge-danger" },
  };
  const { label, cls } = map[compatibility] ?? { label: compatibility, cls: "badge-neutral" };
  return (
    <span
      className={`badge ${cls}`}
      data-testid="field-compatibility-badge"
      data-compatibility={compatibility}
      style={{ fontSize: "var(--text-xs)" }}
    >
      {label}
    </span>
  );
}

function SectionLabel({ children }: { children: React.ReactNode }) {
  return (
    <p
      style={{
        fontSize: "var(--text-xs)",
        fontWeight: 600,
        color: "var(--color-text-muted)",
        textTransform: "uppercase",
        letterSpacing: "0.06em",
        marginBottom: "var(--space-2)",
        marginTop: 0,
      }}
    >
      {children}
    </p>
  );
}

function SummaryRow({ label, children }: { label: string; children: React.ReactNode }) {
  return (
    <div
      style={{
        display: "flex",
        gap: "var(--space-2)",
        fontSize: "var(--text-xs)",
        marginBottom: "var(--space-1)",
      }}
    >
      <span style={{ color: "var(--color-text-muted)", minWidth: 80 }}>{label}</span>
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

function OrderingRow({ done, label }: { done: boolean; label: string }) {
  return (
    <div
      style={{
        display: "flex",
        alignItems: "center",
        gap: "var(--space-2)",
        fontSize: "var(--text-xs)",
        marginBottom: "var(--space-1)",
      }}
    >
      <span
        style={{
          color: done ? "var(--color-text-success)" : "var(--color-text-muted)",
          fontWeight: 600,
        }}
      >
        {done ? "✓" : "—"}
      </span>
      <span>{label}</span>
    </div>
  );
}
