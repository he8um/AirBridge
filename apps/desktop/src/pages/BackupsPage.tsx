import { useState } from "react";
import { SectionHeader } from "../components/SectionHeader";
import { EmptyState } from "../components/EmptyState";
import { StatusBadge } from "../components/StatusBadge";
import { useAppState } from "../state/useAppState";
import { liveAirBridgeService } from "../services/liveAirBridgeService";
import type { AirBridgeService } from "../services/airBridgeService";
import type { BackupStatus } from "../domain/backup";
import type { AirtableBaseSummary } from "../domain/airtable";
import type {
  AccessibleBaseSummary,
  BackupPlan,
  BackupPlanRequest,
  BackupPlanWarning,
  BaseSchemaSummary,
  RecordCountState,
  RecordReadEstimate,
  RecordsExportPlan,
  RecordsExportPlanRequest,
  RequestEstimate,
} from "../backend/types";

const SCOPE_OPTIONS = [
  { value: "full", label: "Full backup", description: "Schema and all records" },
  { value: "schemaOnly", label: "Schema only", description: "Table structure, no records" },
  { value: "recordsOnly", label: "Records only", description: "Records without schema" },
] as const;

const EMPTY_ICON =
  "M20 13V6a2 2 0 00-2-2H6a2 2 0 00-2 2v7m16 0v5a2 2 0 01-2 2H6a2 2 0 01-2-2v-5m16 0h-2.586a1 1 0 00-.707.293l-2.414 2.414a1 1 0 01-.707.293h-3.172a1 1 0 01-.707-.293l-2.414-2.414A1 1 0 006.586 13H4";

function backupStatusBadge(status: BackupStatus): "connected" | "error" | "warning" | "idle" {
  switch (status) {
    case "succeeded":
      return "connected";
    case "failed":
      return "error";
    case "running":
    case "pending":
      return "warning";
    default:
      return "idle";
  }
}

function formatBytes(bytes: number): string {
  if (bytes === 0) return "0 B";
  if (bytes < 1024) return `${bytes} B`;
  if (bytes < 1024 * 1024) return `${(bytes / 1024).toFixed(1)} KB`;
  return `${(bytes / (1024 * 1024)).toFixed(1)} MB`;
}

function formatEstimate(est: RecordReadEstimate): string {
  if (est.type === "known") return `~${est.value} page${est.value === 1 ? "" : "s"}`;
  return "unknown (no record counts available)";
}

function warningSeverityStyle(sev: BackupPlanWarning["severity"]): React.CSSProperties {
  switch (sev) {
    case "error":
      return { color: "var(--color-error, #c0392b)" };
    case "warning":
      return { color: "var(--color-warning, #d68910)" };
    default:
      return { color: "var(--color-text-muted)" };
  }
}

function formatRecordCount(state: RecordCountState): string {
  if (state.type === "known") return state.count.toLocaleString();
  return "unknown";
}

function formatRequestEstimate(est: RequestEstimate): string {
  if (est.type === "known") return `~${est.pages} page${est.pages === 1 ? "" : "s"}`;
  return "unknown";
}

function RecordsExportPlanResult({ plan }: { plan: RecordsExportPlan }) {
  return (
    <div
      style={{ display: "flex", flexDirection: "column", gap: "var(--space-4)" }}
      aria-label="Records export plan result"
    >
      {/* Planning-only notice */}
      <div
        className="card notice-neutral"
        style={{ padding: "var(--space-3) var(--space-4)" }}
        role="status"
        aria-live="polite"
      >
        <p style={{ fontSize: "var(--text-sm)", color: "var(--color-text-muted)", margin: 0 }}>
          No records have been fetched and no backup file has been written. This is a planning
          summary only.
        </p>
      </div>

      {/* Summary row */}
      <div
        style={{
          display: "flex",
          flexWrap: "wrap",
          gap: "var(--space-4)",
          fontSize: "var(--text-sm)",
        }}
      >
        <span>
          <strong>{plan.tableCount}</strong> {plan.tableCount === 1 ? "table" : "tables"}
        </span>
        <span>Page size: {plan.pageSize}</span>
      </div>

      {/* Warnings */}
      {plan.warnings.length > 0 && (
        <div>
          <p
            style={{
              fontSize: "var(--text-xs)",
              fontWeight: 600,
              textTransform: "uppercase",
              letterSpacing: "0.05em",
              color: "var(--color-text-muted)",
              margin: "0 0 var(--space-2) 0",
            }}
          >
            Notices
          </p>
          <ul
            style={{
              listStyle: "none",
              margin: 0,
              padding: 0,
              display: "flex",
              flexDirection: "column",
              gap: "var(--space-2)",
            }}
            aria-label="Export plan notices"
          >
            {plan.warnings.map((w, i) => (
              <li
                key={i}
                style={{ fontSize: "var(--text-sm)", ...warningSeverityStyle(w.severity) }}
              >
                <span style={{ fontWeight: 500 }}>{w.code}</span>
                {w.tableName && (
                  <span style={{ color: "var(--color-text-muted)" }}> · {w.tableName}</span>
                )}
                {": "}
                {w.message}
              </li>
            ))}
          </ul>
        </div>
      )}

      {/* Per-table breakdown */}
      {plan.tables.length > 0 && (
        <div>
          <p
            style={{
              fontSize: "var(--text-xs)",
              fontWeight: 600,
              textTransform: "uppercase",
              letterSpacing: "0.05em",
              color: "var(--color-text-muted)",
              margin: "0 0 var(--space-2) 0",
            }}
          >
            Table export plans
          </p>
          <div
            className="card"
            style={{ padding: 0, overflow: "hidden" }}
            aria-label="Table export plans"
          >
            <ul style={{ listStyle: "none", margin: 0, padding: 0 }}>
              {plan.tables.map((t, idx) => (
                <li
                  key={t.tableId}
                  style={{
                    display: "flex",
                    flexDirection: "column",
                    gap: "var(--space-1)",
                    padding: "var(--space-3) var(--space-4)",
                    borderBottom:
                      idx < plan.tables.length - 1 ? "1px solid var(--color-border)" : "none",
                  }}
                >
                  <span style={{ fontSize: "var(--text-sm)", fontWeight: 500 }}>{t.tableName}</span>
                  <span style={{ fontSize: "var(--text-xs)", color: "var(--color-text-muted)" }}>
                    Records: {formatRecordCount(t.recordCount)}
                    {" · "}
                    Estimated pages: {formatRequestEstimate(t.requestEstimate)}
                  </span>
                  <span
                    style={{
                      fontSize: "var(--text-xs)",
                      color: "var(--color-text-muted)",
                      fontFamily: "var(--font-mono)",
                    }}
                  >
                    {t.jsonlOutput.entryPath}
                  </span>
                  {t.linkedRecordPlans.length > 0 && (
                    <span style={{ fontSize: "var(--text-xs)", color: "var(--color-text-muted)" }}>
                      Linked record extraction: {t.linkedRecordPlans[0].policy}
                    </span>
                  )}
                  {t.attachmentPlans.length > 0 && (
                    <span style={{ fontSize: "var(--text-xs)", color: "var(--color-text-muted)" }}>
                      Attachment extraction: {t.attachmentPlans[0].policy}
                    </span>
                  )}
                </li>
              ))}
            </ul>
          </div>
        </div>
      )}
    </div>
  );
}

interface RecordsExportPlanCardProps {
  backupPlan: BackupPlan | null;
  service: AirBridgeService;
}

function RecordsExportPlanCard({ backupPlan, service }: RecordsExportPlanCardProps) {
  const [exportPlan, setExportPlan] = useState<RecordsExportPlan | null>(null);
  const [loading, setLoading] = useState(false);
  const [error, setError] = useState<string | null>(null);

  const canGenerate = backupPlan !== null && !loading;

  async function handleGenerate() {
    if (!backupPlan) return;
    setLoading(true);
    setError(null);
    try {
      const request: RecordsExportPlanRequest = {
        baseId: backupPlan.baseId,
        baseName: backupPlan.baseName,
        backupPlan,
      };
      const result = await service.createRecordsExportPlan(request);
      setExportPlan(result);
    } catch {
      setError("Failed to generate records export plan.");
    } finally {
      setLoading(false);
    }
  }

  return (
    <div className="card" style={{ maxWidth: 560 }}>
      <div style={{ display: "flex", flexDirection: "column", gap: "var(--space-5)" }}>
        {!backupPlan && (
          <p style={{ fontSize: "var(--text-sm)", color: "var(--color-text-muted)", margin: 0 }}>
            Generate a backup plan above first, then generate the records export plan here.
          </p>
        )}

        {error && (
          <p
            style={{ fontSize: "var(--text-sm)", color: "var(--color-error, #c0392b)", margin: 0 }}
            role="alert"
          >
            {error}
          </p>
        )}

        <div>
          <button
            type="button"
            className="btn btn-secondary"
            onClick={handleGenerate}
            disabled={!canGenerate}
            aria-label="Generate records export plan"
          >
            {loading ? "Generating…" : "Generate Records Export Plan"}
          </button>
        </div>

        {exportPlan && <RecordsExportPlanResult plan={exportPlan} />}
      </div>
    </div>
  );
}

function BaseCatalogCard({ bases }: { bases: AirtableBaseSummary[] }) {
  if (bases.length === 0) {
    return (
      <div className="card notice-neutral" style={{ maxWidth: 560 }}>
        <p style={{ fontSize: "var(--text-sm)", color: "var(--color-text-muted)" }}>
          No bases loaded. Connect an Airtable account to see available bases.
        </p>
      </div>
    );
  }

  return (
    <div className="card" style={{ maxWidth: 560, padding: 0, overflow: "hidden" }}>
      <ul style={{ listStyle: "none", margin: 0, padding: 0 }} aria-label="Available bases catalog">
        {bases.map((base, idx) => {
          const totalFields = base.tables?.reduce((sum, t) => sum + t.fieldCount, 0) ?? 0;
          return (
            <li
              key={base.id}
              style={{
                display: "flex",
                flexDirection: "column",
                gap: "var(--space-1)",
                padding: "var(--space-4) var(--space-5)",
                borderBottom: idx < bases.length - 1 ? "1px solid var(--color-border)" : "none",
              }}
            >
              <span style={{ fontSize: "var(--text-sm)", fontWeight: 500 }}>{base.name}</span>
              <span style={{ fontSize: "var(--text-xs)", color: "var(--color-text-muted)" }}>
                {base.tableCount} {base.tableCount === 1 ? "table" : "tables"}
                {totalFields > 0 && ` · ${totalFields} fields`}
              </span>
              <span
                style={{
                  fontSize: "var(--text-xs)",
                  color: "var(--color-text-muted)",
                  fontFamily: "var(--font-mono)",
                }}
              >
                {base.id}
              </span>
            </li>
          );
        })}
      </ul>
    </div>
  );
}

function BackupPlanResult({ plan }: { plan: BackupPlan }) {
  return (
    <div
      style={{ display: "flex", flexDirection: "column", gap: "var(--space-4)" }}
      aria-label="Backup plan result"
    >
      {/* Dry-run notice */}
      <div
        className="card notice-neutral"
        style={{ padding: "var(--space-3) var(--space-4)" }}
        role="status"
        aria-live="polite"
      >
        <p
          style={{
            fontSize: "var(--text-sm)",
            color: "var(--color-text-muted)",
            margin: 0,
          }}
        >
          No backup file has been created yet. This is a dry-run plan only.
        </p>
      </div>

      {/* Summary row */}
      <div
        style={{
          display: "flex",
          flexWrap: "wrap",
          gap: "var(--space-4)",
          fontSize: "var(--text-sm)",
        }}
      >
        <span>
          <strong>{plan.tableCount}</strong> {plan.tableCount === 1 ? "table" : "tables"}
        </span>
        <span>
          <strong>{plan.totalFieldCount}</strong> {plan.totalFieldCount === 1 ? "field" : "fields"}
        </span>
        <span>
          <strong>{plan.compatibility.restorableCount}</strong> restorable
        </span>
        {plan.compatibility.metadataOnlyCount > 0 && (
          <span style={{ color: "var(--color-text-muted)" }}>
            <strong>{plan.compatibility.metadataOnlyCount}</strong> metadata-only
          </span>
        )}
        {plan.compatibility.unknownCount > 0 && (
          <span style={{ color: "var(--color-text-muted)" }}>
            <strong>{plan.compatibility.unknownCount}</strong> unknown
          </span>
        )}
      </div>

      {/* Estimates */}
      <div style={{ fontSize: "var(--text-sm)", color: "var(--color-text-muted)" }}>
        <span>Estimated API read pages: {formatEstimate(plan.estimate.recordReadPages)}</span>
        {plan.estimate.note && (
          <span style={{ marginLeft: "var(--space-2)", fontStyle: "italic" }}>
            — {plan.estimate.note}
          </span>
        )}
      </div>

      {/* Warnings */}
      {plan.warnings.length > 0 && (
        <div>
          <p
            style={{
              fontSize: "var(--text-xs)",
              fontWeight: 600,
              textTransform: "uppercase",
              letterSpacing: "0.05em",
              color: "var(--color-text-muted)",
              margin: "0 0 var(--space-2) 0",
            }}
          >
            Notices
          </p>
          <ul
            style={{
              listStyle: "none",
              margin: 0,
              padding: 0,
              display: "flex",
              flexDirection: "column",
              gap: "var(--space-2)",
            }}
            aria-label="Backup plan notices"
          >
            {plan.warnings.map((w, i) => (
              <li
                key={i}
                style={{
                  fontSize: "var(--text-sm)",
                  ...warningSeverityStyle(w.severity),
                }}
              >
                <span style={{ fontWeight: 500 }}>{w.code}</span>
                {w.tableName && (
                  <span style={{ color: "var(--color-text-muted)" }}> · {w.tableName}</span>
                )}
                {w.fieldName && (
                  <span style={{ color: "var(--color-text-muted)" }}> / {w.fieldName}</span>
                )}
                {": "}
                {w.message}
              </li>
            ))}
          </ul>
        </div>
      )}

      {/* Table breakdown */}
      {plan.tables.length > 0 && (
        <div>
          <p
            style={{
              fontSize: "var(--text-xs)",
              fontWeight: 600,
              textTransform: "uppercase",
              letterSpacing: "0.05em",
              color: "var(--color-text-muted)",
              margin: "0 0 var(--space-2) 0",
            }}
          >
            Tables included
          </p>
          <div
            className="card"
            style={{ padding: 0, overflow: "hidden" }}
            aria-label="Tables included in backup plan"
          >
            <ul style={{ listStyle: "none", margin: 0, padding: 0 }}>
              {plan.tables.map((t, idx) => (
                <li
                  key={t.id}
                  style={{
                    display: "flex",
                    flexDirection: "column",
                    gap: "var(--space-1)",
                    padding: "var(--space-3) var(--space-4)",
                    borderBottom:
                      idx < plan.tables.length - 1 ? "1px solid var(--color-border)" : "none",
                  }}
                >
                  <span style={{ fontSize: "var(--text-sm)", fontWeight: 500 }}>{t.name}</span>
                  <span style={{ fontSize: "var(--text-xs)", color: "var(--color-text-muted)" }}>
                    {t.fieldCount} {t.fieldCount === 1 ? "field" : "fields"}
                    {t.recordCount != null && ` · ${t.recordCount} records`}
                    {" · "}
                    {t.compatibility.restorableCount} restorable
                    {t.compatibility.metadataOnlyCount > 0 &&
                      `, ${t.compatibility.metadataOnlyCount} metadata-only`}
                    {t.compatibility.unknownCount > 0 &&
                      `, ${t.compatibility.unknownCount} unknown`}
                  </span>
                </li>
              ))}
            </ul>
          </div>
        </div>
      )}
    </div>
  );
}

interface BackupPlanningCardProps {
  accessibleBases: AccessibleBaseSummary[];
  service: AirBridgeService;
  onPlanGenerated?: (plan: BackupPlan | null) => void;
}

function BackupPlanningCard({
  accessibleBases,
  service,
  onPlanGenerated,
}: BackupPlanningCardProps) {
  const [selectedBaseId, setSelectedBaseId] = useState("");
  const [schema, setSchema] = useState<BaseSchemaSummary | null>(null);
  const [schemaLoading, setSchemaLoading] = useState(false);
  const [schemaError, setSchemaError] = useState<string | null>(null);
  const [plan, setPlan] = useState<BackupPlan | null>(null);
  const [planLoading, setPlanLoading] = useState(false);
  const [planError, setPlanError] = useState<string | null>(null);

  const selectedBase = accessibleBases.find((b) => b.id === selectedBaseId) ?? null;

  function handleBaseChange(id: string) {
    setSelectedBaseId(id);
    setSchema(null);
    setSchemaError(null);
    setPlan(null);
    setPlanError(null);
    onPlanGenerated?.(null);
  }

  async function handleLoadSchema() {
    if (!selectedBaseId) return;
    setSchemaLoading(true);
    setSchemaError(null);
    setSchema(null);
    setPlan(null);
    setPlanError(null);
    try {
      // Schema fetch requires the token to be supplied. Since token is session-local
      // and not persisted, this path is deferred to a connected session. For now,
      // we indicate that schema loading requires an active connection.
      setSchemaError(
        "Schema loading requires an active connection. Use the Connection page to connect first.",
      );
    } catch {
      setSchemaError("Failed to load schema.");
    } finally {
      setSchemaLoading(false);
    }
  }

  async function handleGeneratePlan() {
    if (!schema || !selectedBase) return;
    setPlanLoading(true);
    setPlanError(null);
    try {
      const request: BackupPlanRequest = {
        baseId: schema.baseId,
        baseName: selectedBase.name,
        scope: "full",
        tables: schema.tables.map((t) => ({
          id: t.id,
          name: t.name,
          fields: t.fieldTypeCounts.map((ftc, i) => ({
            id: `${t.id}-f${i}`,
            name: ftc.fieldType,
            fieldType: ftc.fieldType,
          })),
          recordCount: undefined,
        })),
      };
      const result = await service.createBackupPlan(request);
      setPlan(result);
      onPlanGenerated?.(result);
    } catch {
      setPlanError("Failed to generate backup plan.");
    } finally {
      setPlanLoading(false);
    }
  }

  const canLoadSchema = selectedBaseId !== "" && !schemaLoading;
  const canGeneratePlan = schema !== null && !planLoading;

  return (
    <div className="card" style={{ maxWidth: 560 }}>
      <div style={{ display: "flex", flexDirection: "column", gap: "var(--space-5)" }}>
        {/* Base selection */}
        <div className="form-field">
          <label htmlFor="plan-base-select" className="form-label">
            Select Base
          </label>
          <select
            id="plan-base-select"
            className="form-input"
            value={selectedBaseId}
            onChange={(e) => handleBaseChange(e.target.value)}
            disabled={accessibleBases.length === 0}
            aria-label="Select Airtable base for backup plan"
          >
            {accessibleBases.length === 0 ? (
              <option value="">No accessible bases — connect an account first</option>
            ) : (
              <>
                <option value="">Choose a base…</option>
                {accessibleBases.map((base) => (
                  <option key={base.id} value={base.id}>
                    {base.name}
                  </option>
                ))}
              </>
            )}
          </select>
        </div>

        {/* Schema summary (once loaded) */}
        {schema && (
          <div
            style={{
              padding: "var(--space-3) var(--space-4)",
              backgroundColor: "var(--color-bg-subtle, var(--color-bg))",
              borderRadius: "var(--radius-md)",
              border: "1px solid var(--color-border)",
              fontSize: "var(--text-sm)",
            }}
            aria-label="Schema summary"
          >
            <p style={{ margin: "0 0 var(--space-2) 0", fontWeight: 500 }}>Schema loaded</p>
            <p style={{ margin: 0, color: "var(--color-text-muted)" }}>
              {schema.tableCount} {schema.tableCount === 1 ? "table" : "tables"}
              {" · "}
              {schema.compatibility.restorableCount} restorable fields
              {schema.compatibility.metadataOnlyCount > 0 &&
                `, ${schema.compatibility.metadataOnlyCount} metadata-only`}
              {schema.compatibility.unknownCount > 0 &&
                `, ${schema.compatibility.unknownCount} unknown`}
            </p>
          </div>
        )}

        {/* Schema error */}
        {schemaError && (
          <p
            style={{
              fontSize: "var(--text-sm)",
              color: "var(--color-error, #c0392b)",
              margin: 0,
            }}
            role="alert"
          >
            {schemaError}
          </p>
        )}

        {/* Plan error */}
        {planError && (
          <p
            style={{
              fontSize: "var(--text-sm)",
              color: "var(--color-error, #c0392b)",
              margin: 0,
            }}
            role="alert"
          >
            {planError}
          </p>
        )}

        {/* Actions */}
        <div style={{ display: "flex", gap: "var(--space-3)", flexWrap: "wrap" }}>
          <button
            type="button"
            className="btn btn-secondary"
            onClick={handleLoadSchema}
            disabled={!canLoadSchema}
            aria-label="Load schema for selected base"
          >
            {schemaLoading ? "Loading schema…" : "Load Schema"}
          </button>
          <button
            type="button"
            className="btn btn-primary"
            onClick={handleGeneratePlan}
            disabled={!canGeneratePlan}
            aria-label="Generate backup plan"
          >
            {planLoading ? "Generating…" : "Generate Backup Plan"}
          </button>
        </div>

        {/* Plan result */}
        {plan && <BackupPlanResult plan={plan} />}
      </div>
    </div>
  );
}

interface BackupsPageProps {
  service?: AirBridgeService;
}

export function BackupsPage({ service = liveAirBridgeService }: BackupsPageProps) {
  const { recentBackups, state } = useAppState();
  const [generatedBackupPlan, setGeneratedBackupPlan] = useState<BackupPlan | null>(null);

  const bases = state.bases;
  const accessibleBases: AccessibleBaseSummary[] = bases.map((b) => ({ id: b.id, name: b.name }));

  return (
    <div className="page">
      <div className="page-content">
        {/* Backup Planning section */}
        <section aria-labelledby="backup-plan-heading">
          <SectionHeader title="Backup Planning" />
          <BackupPlanningCard
            accessibleBases={accessibleBases}
            service={service}
            onPlanGenerated={setGeneratedBackupPlan}
          />
        </section>

        {/* Records Export Plan section */}
        <section aria-labelledby="records-export-plan-heading">
          <SectionHeader title="Records Export Plan" />
          <RecordsExportPlanCard backupPlan={generatedBackupPlan} service={service} />
        </section>

        {/* Package Format section */}
        <section aria-labelledby="package-format-heading">
          <SectionHeader title="Package Format" />
          <div className="card notice-neutral" style={{ maxWidth: 560 }}>
            <div style={{ display: "flex", flexDirection: "column", gap: "var(--space-3)" }}>
              <p style={{ fontSize: "var(--text-sm)", margin: 0, fontWeight: 500 }}>
                .airbridge package format — foundation ready
              </p>
              <p
                style={{ fontSize: "var(--text-sm)", color: "var(--color-text-muted)", margin: 0 }}
              >
                The package writer, reader, and validator are implemented and tested. Backup files
                are not yet created from the UI. Live export will be enabled in a future release
                once the record export engine is connected to this package format.
              </p>
              <ul
                style={{
                  fontSize: "var(--text-xs)",
                  color: "var(--color-text-muted)",
                  margin: 0,
                  paddingLeft: "var(--space-4)",
                  display: "flex",
                  flexDirection: "column",
                  gap: "var(--space-1)",
                }}
              >
                <li>ZIP-compatible .airbridge archive</li>
                <li>manifest.json, schema.json, base.json, per-table records.jsonl</li>
                <li>SHA-256 checksums for all entries</li>
                <li>Attachment metadata only (no file content) in V0.1</li>
                <li>No tokens or local filesystem paths stored inside the package</li>
              </ul>
            </div>
          </div>
        </section>

        {/* New Backup section */}
        <section aria-labelledby="new-backup-heading">
          <SectionHeader title="New Backup" />

          <div className="card" style={{ maxWidth: 560 }}>
            <div style={{ display: "flex", flexDirection: "column", gap: "var(--space-5)" }}>
              {/* Select Base */}
              <div className="form-field">
                <label htmlFor="base-select" className="form-label">
                  Select Base
                </label>
                <select
                  id="base-select"
                  className="form-input"
                  disabled={bases.length === 0}
                  aria-label="Select Airtable base to back up"
                >
                  {bases.length === 0 ? (
                    <option value="">No bases connected</option>
                  ) : (
                    <>
                      <option value="">Choose a base…</option>
                      {bases.map((base) => (
                        <option key={base.id} value={base.id}>
                          {base.name}
                          {base.tableCount > 0
                            ? ` (${base.tableCount} ${base.tableCount === 1 ? "table" : "tables"})`
                            : ""}
                        </option>
                      ))}
                    </>
                  )}
                </select>
              </div>

              {/* Backup Scope */}
              <fieldset style={{ border: "none", padding: 0 }}>
                <legend className="form-label" style={{ marginBottom: "var(--space-3)" }}>
                  Backup Scope
                </legend>
                <div style={{ display: "flex", flexDirection: "column", gap: "var(--space-2)" }}>
                  {SCOPE_OPTIONS.map((opt, idx) => (
                    <label
                      key={opt.value}
                      style={{
                        display: "flex",
                        alignItems: "center",
                        gap: "var(--space-3)",
                        padding: "var(--space-3)",
                        borderRadius: "var(--radius-md)",
                        border: `1px solid ${idx === 0 ? "var(--color-accent)" : "var(--color-border)"}`,
                        backgroundColor:
                          idx === 0 ? "var(--color-accent-light)" : "var(--color-bg)",
                        cursor: "not-allowed",
                        opacity: 0.7,
                      }}
                    >
                      <input
                        type="radio"
                        name="backup-scope"
                        value={opt.value}
                        defaultChecked={idx === 0}
                        disabled
                        aria-label={opt.label}
                      />
                      <div>
                        <div style={{ fontSize: "var(--text-sm)", fontWeight: 500 }}>
                          {opt.label}
                        </div>
                        <div
                          style={{ fontSize: "var(--text-xs)", color: "var(--color-text-muted)" }}
                        >
                          {opt.description}
                        </div>
                      </div>
                    </label>
                  ))}
                </div>
              </fieldset>

              {/* Output location */}
              <div className="form-field">
                <label htmlFor="output-path" className="form-label">
                  Output Location
                </label>
                <div style={{ display: "flex", gap: "var(--space-2)" }}>
                  <input
                    id="output-path"
                    type="text"
                    className="form-input"
                    placeholder="~/Documents/AirBridge/backups/"
                    disabled
                    aria-label="Output directory for backup files"
                  />
                  <button
                    type="button"
                    className="btn btn-secondary"
                    disabled
                    style={{ flexShrink: 0 }}
                  >
                    Browse
                  </button>
                </div>
              </div>

              {/* Actions */}
              <div style={{ paddingTop: "var(--space-2)" }}>
                <button
                  type="button"
                  className="btn btn-primary"
                  disabled
                  aria-label="Start backup job"
                >
                  <svg
                    width="14"
                    height="14"
                    viewBox="0 0 24 24"
                    fill="none"
                    stroke="currentColor"
                    strokeWidth="2"
                    strokeLinecap="round"
                    strokeLinejoin="round"
                    aria-hidden="true"
                  >
                    <path d="M8 7H5a2 2 0 00-2 2v9a2 2 0 002 2h14a2 2 0 002-2V9a2 2 0 00-2-2h-3m-1 4l-3 3m0 0l-3-3m3 3V4" />
                  </svg>
                  Start Backup
                </button>
              </div>
            </div>
          </div>
        </section>

        {/* Base Catalog section */}
        <section aria-labelledby="base-catalog-heading">
          <SectionHeader title="Available Bases" />
          <BaseCatalogCard bases={bases} />
        </section>

        {/* Recent Backups section */}
        <section aria-labelledby="recent-backups-heading">
          <SectionHeader title="Recent Backups" />

          <div className="card" style={{ padding: 0, overflow: "hidden" }}>
            {recentBackups.length === 0 ? (
              <EmptyState
                icon={EMPTY_ICON}
                title="No backups yet"
                description="Run your first backup to see results here."
              />
            ) : (
              <ul style={{ listStyle: "none", margin: 0, padding: 0 }} aria-label="Recent backups">
                {recentBackups.map((pkg, idx) => (
                  <li
                    key={pkg.id}
                    style={{
                      display: "flex",
                      alignItems: "center",
                      justifyContent: "space-between",
                      padding: "var(--space-4) var(--space-5)",
                      borderBottom:
                        idx < recentBackups.length - 1 ? "1px solid var(--color-border)" : "none",
                      gap: "var(--space-4)",
                    }}
                  >
                    <div
                      style={{
                        display: "flex",
                        flexDirection: "column",
                        gap: "var(--space-1)",
                        minWidth: 0,
                      }}
                    >
                      <span
                        style={{
                          fontSize: "var(--text-sm)",
                          fontWeight: 500,
                          overflow: "hidden",
                          textOverflow: "ellipsis",
                          whiteSpace: "nowrap",
                        }}
                      >
                        {pkg.baseName}
                      </span>
                      <span
                        style={{ fontSize: "var(--text-xs)", color: "var(--color-text-muted)" }}
                      >
                        {pkg.scope === "full"
                          ? "Full"
                          : pkg.scope === "schema_only"
                            ? "Schema only"
                            : "Records only"}
                        {" · "}
                        {pkg.tableCount} {pkg.tableCount === 1 ? "table" : "tables"}
                        {pkg.scope !== "schema_only" && ` · ${pkg.recordCount} records`}
                        {pkg.fileSizeBytes > 0 && ` · ${formatBytes(pkg.fileSizeBytes)}`}
                      </span>
                      <span
                        style={{ fontSize: "var(--text-xs)", color: "var(--color-text-muted)" }}
                      >
                        {new Date(pkg.createdAt).toLocaleDateString(undefined, {
                          year: "numeric",
                          month: "short",
                          day: "numeric",
                        })}
                      </span>
                    </div>
                    <StatusBadge
                      status={backupStatusBadge(pkg.status)}
                      label={pkg.status.charAt(0).toUpperCase() + pkg.status.slice(1)}
                    />
                  </li>
                ))}
              </ul>
            )}
          </div>
        </section>
      </div>
    </div>
  );
}
