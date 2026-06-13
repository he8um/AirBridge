import { useState } from "react";
import type { AirBridgeService } from "../../services/airBridgeService";
import type {
  RecordImportTableInput,
  RestoreRecordImportPlan,
  RestoreRecordImportPlanStatus,
  RestoreTargetMode,
} from "../../backend/types";

interface RestoreRecordImportPlanPanelProps {
  service: AirBridgeService;
  /** Filename from the most recent package inspection (filename only, no path). */
  packageFilename: string | null;
  /** Dry-run status gate ("ready" | "readyWithWarnings" | "blocked" | null). */
  dryRunStatus: RestoreRecordImportPlanStatus | null;
  /** Schema plan status gate ("ready" | "readyWithWarnings" | "blocked" | null). */
  schemaPlanStatus: RestoreRecordImportPlanStatus | null;
  /** Target mode from the dry-run panel. */
  targetMode: RestoreTargetMode;
  /** Optional target base name. */
  targetBaseName?: string;
  /** Tables with field metadata for import planning. */
  tables: RecordImportTableInput[];
}

type PanelState = "idle" | "loading" | "done";

function statusBadge(status: RestoreRecordImportPlanStatus) {
  const labels: Record<RestoreRecordImportPlanStatus, string> = {
    ready: "Ready",
    readyWithWarnings: "Ready with Warnings",
    blocked: "Blocked",
  };
  const colors: Record<RestoreRecordImportPlanStatus, string> = {
    ready: "var(--color-success, #22c55e)",
    readyWithWarnings: "var(--color-warning, #f59e0b)",
    blocked: "var(--color-danger, #ef4444)",
  };
  return (
    <span
      data-testid="record-import-plan-status-badge"
      style={{
        fontSize: "var(--text-xs)",
        fontWeight: 600,
        padding: "2px 8px",
        borderRadius: 4,
        background: colors[status],
        color: "#fff",
        letterSpacing: "0.04em",
      }}
    >
      {labels[status]}
    </span>
  );
}

export function RestoreRecordImportPlanPanel({
  service,
  packageFilename,
  dryRunStatus,
  schemaPlanStatus,
  targetMode,
  targetBaseName,
  tables,
}: RestoreRecordImportPlanPanelProps) {
  const [panelState, setPanelState] = useState<PanelState>("idle");
  const [plan, setPlan] = useState<RestoreRecordImportPlan | null>(null);

  const dryRunReady = dryRunStatus === "ready" || dryRunStatus === "readyWithWarnings";
  const schemaReady = schemaPlanStatus === "ready" || schemaPlanStatus === "readyWithWarnings";
  const canGenerate = !!packageFilename && dryRunReady && schemaReady && panelState === "idle";

  async function handleGeneratePlan() {
    if (!packageFilename) return;
    setPanelState("loading");
    setPlan(null);
    try {
      const result = await service.createRestoreRecordImportPlan({
        packageFilename,
        dryRunStatus: dryRunStatus ?? "blocked",
        schemaPlanStatus: schemaPlanStatus ?? "blocked",
        targetMode,
        targetBaseName,
        tables,
      });
      setPlan(result);
    } finally {
      setPanelState("done");
    }
  }

  return (
    <div data-testid="restore-record-import-plan-panel">
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
          Record Import Plan
        </p>

        {plan && statusBadge(plan.status)}
      </div>

      <p
        style={{
          fontSize: "var(--text-sm)",
          color: "var(--color-text-muted)",
          margin: "0 0 var(--space-3)",
        }}
      >
        Review the record import batch plan before executing a restore. No Airtable changes are
        made.
      </p>

      <button
        type="button"
        className="btn btn-secondary"
        data-testid="record-import-plan-generate-btn"
        disabled={!canGenerate}
        onClick={handleGeneratePlan}
        style={{ marginBottom: "var(--space-4)" }}
      >
        {panelState === "loading" ? "Planning…" : "Preview Record Import Plan"}
      </button>

      {!packageFilename && panelState === "idle" && (
        <p
          data-testid="record-import-plan-requires-inspection"
          style={{ fontSize: "var(--text-xs)", color: "var(--color-text-muted)" }}
        >
          Inspect a package first.
        </p>
      )}

      {packageFilename && !dryRunReady && panelState === "idle" && (
        <p
          data-testid="record-import-plan-requires-dry-run"
          style={{ fontSize: "var(--text-xs)", color: "var(--color-text-muted)" }}
        >
          Generate a restore plan preview first.
        </p>
      )}

      {packageFilename && dryRunReady && !schemaReady && panelState === "idle" && (
        <p
          data-testid="record-import-plan-requires-schema-plan"
          style={{ fontSize: "var(--text-xs)", color: "var(--color-text-muted)" }}
        >
          Generate a schema creation plan first.
        </p>
      )}

      {plan && (
        <div
          data-testid="record-import-plan-result"
          style={{ display: "flex", flexDirection: "column", gap: "var(--space-4)" }}
        >
          {/* Table import plans */}
          {plan.tablePlans.length > 0 && (
            <section>
              <p
                style={{
                  fontSize: "var(--text-xs)",
                  fontWeight: 600,
                  color: "var(--color-text-muted)",
                  textTransform: "uppercase",
                  letterSpacing: "0.06em",
                  marginBottom: "var(--space-2)",
                }}
              >
                Table Import Plans
              </p>
              <ul
                data-testid="record-import-plan-table-list"
                style={{
                  margin: 0,
                  padding: 0,
                  listStyle: "none",
                  display: "flex",
                  flexDirection: "column",
                  gap: "var(--space-2)",
                }}
              >
                {plan.tablePlans.map((tp) => (
                  <li
                    key={tp.tableId}
                    data-testid={`record-import-plan-table-${tp.tableId}`}
                    style={{
                      fontSize: "var(--text-sm)",
                      padding: "var(--space-2) var(--space-3)",
                      border: "1px solid var(--color-border)",
                      borderRadius: 6,
                    }}
                  >
                    <div style={{ fontWeight: 600 }}>
                      {tp.importOrder + 1}. {tp.tableName}
                    </div>
                    <div style={{ color: "var(--color-text-muted)", marginTop: 2 }}>
                      {tp.recordCountKnown ? (
                        <>
                          {tp.recordCount} records — {tp.createBatchCount} create batch
                          {tp.createBatchCount !== 1 ? "es" : ""} of {tp.batchSize}
                          {tp.updateBatchCount != null &&
                            `, ${tp.updateBatchCount} linked record update batch${tp.updateBatchCount !== 1 ? "es" : ""}`}
                        </>
                      ) : (
                        "Record count unknown — batch count determined at import time"
                      )}
                    </div>
                    {tp.attachmentPolicies.length > 0 && (
                      <div
                        data-testid={`record-import-plan-attachment-note-${tp.tableId}`}
                        style={{
                          color: "var(--color-text-muted)",
                          marginTop: 2,
                          fontSize: "var(--text-xs)",
                        }}
                      >
                        Attachment fields: metadata only, manual re-attachment required
                      </div>
                    )}
                  </li>
                ))}
              </ul>
            </section>
          )}

          {/* Linked record second-pass plans */}
          {plan.linkedRecordUpdatePlans.length > 0 && (
            <section>
              <p
                style={{
                  fontSize: "var(--text-xs)",
                  fontWeight: 600,
                  color: "var(--color-text-muted)",
                  textTransform: "uppercase",
                  letterSpacing: "0.06em",
                  marginBottom: "var(--space-2)",
                }}
              >
                Linked Record Second-Pass Updates
              </p>
              <ul
                data-testid="record-import-plan-linked-updates"
                style={{
                  margin: 0,
                  padding: "0 0 0 var(--space-5)",
                  display: "flex",
                  flexDirection: "column",
                  gap: "var(--space-1)",
                }}
              >
                {plan.linkedRecordUpdatePlans.map((lp) => (
                  <li key={`${lp.tableId}-${lp.fieldId}`} style={{ fontSize: "var(--text-sm)" }}>
                    <strong>{lp.fieldName}</strong> in {lp.tableName}
                    {" — links to "}
                    {lp.linkedTableName}
                    {lp.updateBatchCount != null && (
                      <>
                        , {lp.updateBatchCount} batch{lp.updateBatchCount !== 1 ? "es" : ""}
                      </>
                    )}
                  </li>
                ))}
              </ul>
            </section>
          )}

          {/* Warnings */}
          {plan.warnings.length > 0 && (
            <section>
              <p
                style={{
                  fontSize: "var(--text-xs)",
                  fontWeight: 600,
                  color: "var(--color-text-muted)",
                  textTransform: "uppercase",
                  letterSpacing: "0.06em",
                  marginBottom: "var(--space-2)",
                }}
              >
                Warnings
              </p>
              <ul
                data-testid="record-import-plan-warnings"
                style={{
                  margin: 0,
                  padding: "0 0 0 var(--space-5)",
                  display: "flex",
                  flexDirection: "column",
                  gap: "var(--space-1)",
                }}
              >
                {plan.warnings.map((w, i) => (
                  <li key={i} style={{ fontSize: "var(--text-sm)" }}>
                    <strong>[{w.code}]</strong> {w.message}
                  </li>
                ))}
              </ul>
            </section>
          )}

          {/* Errors */}
          {plan.errors.length > 0 && (
            <section>
              <p
                style={{
                  fontSize: "var(--text-xs)",
                  fontWeight: 600,
                  color: "var(--color-danger)",
                  textTransform: "uppercase",
                  letterSpacing: "0.06em",
                  marginBottom: "var(--space-2)",
                }}
              >
                Errors
              </p>
              <ul
                data-testid="record-import-plan-errors"
                style={{
                  margin: 0,
                  padding: "0 0 0 var(--space-5)",
                  display: "flex",
                  flexDirection: "column",
                  gap: "var(--space-1)",
                }}
              >
                {plan.errors.map((e, i) => (
                  <li key={i} style={{ fontSize: "var(--text-sm)", color: "var(--color-danger)" }}>
                    <strong>[{e.code}]</strong> {e.message}
                  </li>
                ))}
              </ul>
            </section>
          )}

          {/* Retry policy note */}
          <p
            data-testid="record-import-plan-retry-note"
            style={{
              fontSize: "var(--text-xs)",
              color: "var(--color-text-muted)",
              margin: 0,
            }}
          >
            Retry policy: up to {plan.retryPolicy.maxRetriesOnRateLimit} retries on rate-limit
            responses, starting at {plan.retryPolicy.initialBackoffMs}ms backoff (
            {plan.retryPolicy.backoffMultiplier}× multiplier).
          </p>

          {/* No changes disclaimer */}
          <p
            data-testid="record-import-plan-no-changes"
            style={{
              fontSize: "var(--text-xs)",
              color: "var(--color-text-muted)",
              margin: 0,
              fontStyle: "italic",
            }}
          >
            No Airtable records were created or modified.
          </p>
        </div>
      )}
    </div>
  );
}
