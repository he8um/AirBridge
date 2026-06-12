import { useState } from "react";
import type { AirBridgeService } from "../../services/airBridgeService";
import type {
  RestoreSchemaPlan,
  RestoreSchemaPlanRequest,
  RestoreSchemaPlanStatus,
  RestoreTargetMode,
} from "../../backend/types";

interface RestoreSchemaPlanPanelProps {
  service: AirBridgeService;
  /** Filename from the most recent package inspection (filename only, no path). */
  packageFilename: string | null;
  /** Dry-run status gate ("ready" | "readyWithWarnings" | "blocked" | null). */
  dryRunStatus: RestoreSchemaPlanStatus | null;
  /** Target mode from the dry-run panel. */
  targetMode: RestoreTargetMode;
  /** Optional target base name. */
  targetBaseName?: string;
  /** Tables derived from the dry-run plan tables (mapped to SchemaPlanTableInput). */
  tables: RestoreSchemaPlanRequest["tables"];
}

type PanelState = "idle" | "loading" | "done";

function statusBadge(status: RestoreSchemaPlanStatus) {
  const labels: Record<RestoreSchemaPlanStatus, string> = {
    ready: "Ready",
    readyWithWarnings: "Ready with Warnings",
    blocked: "Blocked",
  };
  const colors: Record<RestoreSchemaPlanStatus, string> = {
    ready: "var(--color-success, #22c55e)",
    readyWithWarnings: "var(--color-warning, #f59e0b)",
    blocked: "var(--color-danger, #ef4444)",
  };
  return (
    <span
      data-testid="schema-plan-status-badge"
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

export function RestoreSchemaPlanPanel({
  service,
  packageFilename,
  dryRunStatus,
  targetMode,
  targetBaseName,
  tables,
}: RestoreSchemaPlanPanelProps) {
  const [panelState, setPanelState] = useState<PanelState>("idle");
  const [plan, setPlan] = useState<RestoreSchemaPlan | null>(null);

  const canGenerate =
    !!packageFilename &&
    (dryRunStatus === "ready" || dryRunStatus === "readyWithWarnings") &&
    panelState === "idle";

  async function handleGeneratePlan() {
    if (!packageFilename) return;
    setPanelState("loading");
    setPlan(null);
    try {
      const result = await service.createRestoreSchemaPlan({
        packageFilename,
        dryRunStatus: dryRunStatus ?? "blocked",
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
    <div data-testid="restore-schema-plan-panel">
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
          Schema Creation Plan
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
        Review the schema creation order before executing a restore. No Airtable changes are made.
      </p>

      <button
        type="button"
        className="btn btn-secondary"
        data-testid="schema-plan-generate-btn"
        disabled={!canGenerate}
        onClick={handleGeneratePlan}
        style={{ marginBottom: "var(--space-4)" }}
      >
        {panelState === "loading" ? "Planning…" : "Preview Schema Creation Plan"}
      </button>

      {!packageFilename && panelState === "idle" && (
        <p
          data-testid="schema-plan-requires-inspection"
          style={{ fontSize: "var(--text-xs)", color: "var(--color-text-muted)" }}
        >
          Inspect a package first.
        </p>
      )}

      {packageFilename &&
        dryRunStatus !== "ready" &&
        dryRunStatus !== "readyWithWarnings" &&
        panelState === "idle" && (
          <p
            data-testid="schema-plan-requires-dry-run"
            style={{ fontSize: "var(--text-xs)", color: "var(--color-text-muted)" }}
          >
            Generate a restore plan preview first.
          </p>
        )}

      {plan && (
        <div
          data-testid="schema-plan-result"
          style={{ display: "flex", flexDirection: "column", gap: "var(--space-4)" }}
        >
          {/* Table creation steps */}
          {plan.tableSteps.length > 0 && (
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
                Table Creation Steps
              </p>
              <ol
                data-testid="schema-plan-table-steps"
                style={{
                  margin: 0,
                  padding: "0 0 0 var(--space-5)",
                  display: "flex",
                  flexDirection: "column",
                  gap: "var(--space-1)",
                }}
              >
                {plan.tableSteps.map((step) => (
                  <li key={step.tableId} style={{ fontSize: "var(--text-sm)" }}>
                    <strong>{step.tableName}</strong>
                    {" — "}
                    {step.directFieldCount} direct, {step.deferredFieldCount} deferred,{" "}
                    {step.manualActionCount} manual
                    {step.unsupportedCount > 0 && <>, {step.unsupportedCount} unsupported</>}
                  </li>
                ))}
              </ol>
            </section>
          )}

          {/* Field creation steps */}
          {plan.fieldSteps.length > 0 && (
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
                Field Creation Steps
              </p>
              <ul
                data-testid="schema-plan-field-steps"
                style={{
                  margin: 0,
                  padding: "0 0 0 var(--space-5)",
                  display: "flex",
                  flexDirection: "column",
                  gap: "var(--space-1)",
                }}
              >
                {plan.fieldSteps.map((step) => (
                  <li
                    key={`${step.tableId}-${step.fieldId}`}
                    style={{ fontSize: "var(--text-sm)" }}
                  >
                    <strong>{step.fieldName}</strong>{" "}
                    <span style={{ color: "var(--color-text-muted)" }}>({step.fieldType})</span>
                    {" in "}
                    {step.tableName}
                    {" — "}
                    {step.classification}
                  </li>
                ))}
              </ul>
            </section>
          )}

          {/* Deferred linked fields */}
          {plan.deferredSteps.length > 0 && (
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
                Deferred Linked Fields
              </p>
              <ul
                data-testid="schema-plan-deferred-steps"
                style={{
                  margin: 0,
                  padding: "0 0 0 var(--space-5)",
                  display: "flex",
                  flexDirection: "column",
                  gap: "var(--space-1)",
                }}
              >
                {plan.deferredSteps.map((step) => (
                  <li
                    key={`${step.tableId}-${step.fieldId}`}
                    style={{ fontSize: "var(--text-sm)" }}
                  >
                    <strong>{step.fieldName}</strong> in {step.tableName}
                    {step.linkedTableId && <> → {step.linkedTableId}</>}
                  </li>
                ))}
              </ul>
            </section>
          )}

          {/* Manual action fields */}
          {plan.manualActionFields.length > 0 && (
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
                Manual Action Required
              </p>
              <ul
                data-testid="schema-plan-manual-action-fields"
                style={{
                  margin: 0,
                  padding: "0 0 0 var(--space-5)",
                  display: "flex",
                  flexDirection: "column",
                  gap: "var(--space-1)",
                }}
              >
                {plan.manualActionFields.map((field) => (
                  <li
                    key={`${field.tableId}-${field.fieldId}`}
                    style={{ fontSize: "var(--text-sm)" }}
                  >
                    <strong>{field.fieldName}</strong>{" "}
                    <span style={{ color: "var(--color-text-muted)" }}>({field.fieldType})</span>
                    {" in "}
                    {field.tableName}
                    {" — "}
                    {field.actionDescription}
                  </li>
                ))}
              </ul>
            </section>
          )}

          {/* Dependency graph summary */}
          {plan.dependencyGraph.edges.length > 0 && (
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
                Linked Record Dependencies
              </p>
              <ul
                data-testid="schema-plan-dependency-graph"
                style={{
                  margin: 0,
                  padding: "0 0 0 var(--space-5)",
                  display: "flex",
                  flexDirection: "column",
                  gap: "var(--space-1)",
                }}
              >
                {plan.dependencyGraph.edges.map((edge) => (
                  <li
                    key={`${edge.sourceTableId}-${edge.fieldId}`}
                    style={{ fontSize: "var(--text-sm)" }}
                  >
                    <strong>{edge.fieldName}</strong>: {edge.sourceTableName} →{" "}
                    {edge.targetTableName}
                    {edge.remappingRequired && (
                      <span style={{ color: "var(--color-text-muted)" }}>
                        {" "}
                        (remapping required)
                      </span>
                    )}
                  </li>
                ))}
              </ul>
              {plan.dependencyGraph.hasCircularDependency && (
                <p
                  data-testid="schema-plan-circular-warning"
                  className="notice notice-warning"
                  style={{ marginTop: "var(--space-2)" }}
                >
                  Circular linked record dependency detected. Manual intervention may be required.
                </p>
              )}
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
                data-testid="schema-plan-warnings"
                style={{
                  margin: 0,
                  padding: 0,
                  listStyle: "none",
                  display: "flex",
                  flexDirection: "column",
                  gap: "var(--space-2)",
                }}
              >
                {plan.warnings.map((w, i) => (
                  <li
                    key={i}
                    className="notice notice-warning"
                    style={{ fontSize: "var(--text-sm)" }}
                  >
                    {w.tableName && <strong>{w.tableName}: </strong>}
                    {w.message}
                  </li>
                ))}
              </ul>
            </section>
          )}

          {/* Errors */}
          {plan.errors.length > 0 && (
            <section>
              <ul
                data-testid="schema-plan-errors"
                style={{
                  margin: 0,
                  padding: 0,
                  listStyle: "none",
                  display: "flex",
                  flexDirection: "column",
                  gap: "var(--space-2)",
                }}
              >
                {plan.errors.map((e, i) => (
                  <li
                    key={i}
                    className="notice notice-danger"
                    style={{ fontSize: "var(--text-sm)" }}
                  >
                    {e.message}
                  </li>
                ))}
              </ul>
            </section>
          )}

          {/* No changes statement */}
          <p
            data-testid="schema-plan-no-changes-made"
            className="notice notice-info"
            style={{ fontSize: "var(--text-xs)", marginTop: "var(--space-2)" }}
          >
            No Airtable changes were made. This is a plan only.
          </p>
        </div>
      )}
    </div>
  );
}
