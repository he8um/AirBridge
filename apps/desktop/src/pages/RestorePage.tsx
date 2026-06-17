import { useState } from "react";
import { SectionHeader } from "../components/SectionHeader";
import { useAppState } from "../state/useAppState";
import { PackageInspectionPanel } from "../features/backups/PackageInspectionPanel";
import { RestoreDryRunPanel } from "../features/backups/RestoreDryRunPanel";
import { RestoreExecutionGatePanel } from "../features/backups/RestoreExecutionGatePanel";
import { RestoreRecordImportPlanPanel } from "../features/backups/RestoreRecordImportPlanPanel";
import { RestoreSchemaPlanPanel } from "../features/backups/RestoreSchemaPlanPanel";
import { RestoreConfirmationPanel } from "../features/backups/RestoreConfirmationPanel";
import { RestoreSandboxVerificationPanel } from "../features/backups/RestoreSandboxVerificationPanel";
import { RestoreTargetEmptyVerificationPanel } from "../features/backups/RestoreTargetEmptyVerificationPanel";
import { RestoreDestructiveOperationPolicyPanel } from "../features/backups/RestoreDestructiveOperationPolicyPanel";
import { RestoreAttachmentUploadPolicyPanel } from "../features/backups/RestoreAttachmentUploadPolicyPanel";
import { RestoreSchemaRecordOrderPolicyPanel } from "../features/backups/RestoreSchemaRecordOrderPolicyPanel";
import { RestoreSandboxWriteTestingPolicyPanel } from "../features/backups/RestoreSandboxWriteTestingPolicyPanel";
import { RestoreLiveWriteConfirmationPolicyPanel } from "../features/backups/RestoreLiveWriteConfirmationPolicyPanel";
import { RestoreRateLimitBackoffPolicyPanel } from "../features/backups/RestoreRateLimitBackoffPolicyPanel";
import { RestoreCheckpointDurabilityPolicyPanel } from "../features/backups/RestoreCheckpointDurabilityPolicyPanel";
import { RestoreFinalValidationPolicyPanel } from "../features/backups/RestoreFinalValidationPolicyPanel";
import { RestoreWritePhaseOrderingPolicyPanel } from "../features/backups/RestoreWritePhaseOrderingPolicyPanel";
import { RestoreFailureModesPolicyPanel } from "../features/backups/RestoreFailureModesPolicyPanel";
import { RestoreRollbackLimitationPolicyPanel } from "../features/backups/RestoreRollbackLimitationPolicyPanel";
import { RestoreFinalValidationEnforcementPolicyPanel } from "../features/backups/RestoreFinalValidationEnforcementPolicyPanel";
import { RestoreSensitiveDataSafetyPolicyPanel } from "../features/backups/RestoreSensitiveDataSafetyPolicyPanel";
import { RestoreAttachmentPhaseDisabledPolicyPanel } from "../features/backups/RestoreAttachmentPhaseDisabledPolicyPanel";
import { RestoreLiveWriteReadinessPolicyPanel } from "../features/backups/RestoreLiveWriteReadinessPolicyPanel";
import { RestoreSchemaWriteExecutionPreviewPanel } from "../features/backups/RestoreSchemaWriteExecutionPreviewPanel";
import { RestoreRecordWriteExecutionPreviewPanel } from "../features/backups/RestoreRecordWriteExecutionPreviewPanel";
import { RestoreMappingCheckpointExecutionPreviewPanel } from "../features/backups/RestoreMappingCheckpointExecutionPreviewPanel";
import { RestoreWriteEnginePanel } from "../features/backups/RestoreWriteEnginePanel";
import { liveAirBridgeService } from "../services/liveAirBridgeService";
import type { BackupPackageInspectionResult } from "../backend/types";
import type {
  RecordImportTableInput,
  RestoreConfirmationResult,
  RestoreDryRunPlan,
  RestoreRecordImportPlanStatus,
  RestoreSchemaPlan,
  RestoreSchemaPlanRequest,
  RestoreTargetMode,
  RestoreWriteEngineResult,
  SandboxVerificationResult,
  TargetEmptyVerificationResult,
  DestructiveOperationPolicyResult,
  AttachmentUploadPolicyResult,
  SchemaRecordOrderPolicyResult,
  SandboxWriteTestingPolicyResult,
  LiveWriteConfirmationPolicyResult,
  RateLimitBackoffPolicyResult,
  CheckpointDurabilityPolicyResult,
  FinalValidationPolicyResult,
  WritePhaseOrderingPolicyResult,
  FailureModesPolicyResult,
  RollbackLimitationPolicyResult,
  FinalValidationEnforcementPolicyResult,
  SensitiveDataSafetyPolicyResult,
  AttachmentPhaseDisabledPolicyResult,
  LiveWriteReadinessPolicyResult,
  SchemaWriteExecutionPreviewResult,
  RecordWriteExecutionPreviewResult,
  MappingCheckpointExecutionPreviewResult,
} from "../backend/types";

export function RestorePage() {
  const { state, compatibilitySummary } = useAppState();
  const plan = state.restorePlans[0];
  const bases = state.bases;

  // Shared state lifted from inspection and dry-run panels so the execution gate
  // can read prerequisites without mounting a separate file picker.
  const [inspection, setInspection] = useState<BackupPackageInspectionResult | null>(null);
  const [packagePath, setPackagePath] = useState<string | null>(null);
  const [dryRunPlan, setDryRunPlan] = useState<RestoreDryRunPlan | null>(null);
  const [targetMode, setTargetMode] = useState<RestoreTargetMode>("newBase");
  const [targetBaseName, setTargetBaseName] = useState<string | undefined>(undefined);
  const [schemaPlanStatus, setSchemaPlanStatus] = useState<RestoreRecordImportPlanStatus | null>(
    null,
  );
  const [schemaPlan, setSchemaPlan] = useState<RestoreSchemaPlan | null>(null);
  const [recordImportTables, setRecordImportTables] = useState<RecordImportTableInput[]>([]);
  const [writeEngineResult, setWriteEngineResult] = useState<RestoreWriteEngineResult | null>(null);
  const [sandboxResult, setSandboxResult] = useState<SandboxVerificationResult | null>(null);
  const [sandboxLoading, setSandboxLoading] = useState(false);
  const [confirmationResult, setConfirmationResult] = useState<RestoreConfirmationResult | null>(
    null,
  );
  const [confirmationLoading, setConfirmationLoading] = useState(false);
  const [targetEmptyResult, setTargetEmptyResult] = useState<TargetEmptyVerificationResult | null>(
    null,
  );
  const [targetEmptyLoading, setTargetEmptyLoading] = useState(false);
  const [dopResult, setDopResult] = useState<DestructiveOperationPolicyResult | null>(null);
  const [dopLoading, setDopLoading] = useState(false);
  const [aupResult, setAupResult] = useState<AttachmentUploadPolicyResult | null>(null);
  const [aupLoading, setAupLoading] = useState(false);
  const [sroResult, setSroResult] = useState<SchemaRecordOrderPolicyResult | null>(null);
  const [sroLoading, setSroLoading] = useState(false);
  const [swtResult, setSwtResult] = useState<SandboxWriteTestingPolicyResult | null>(null);
  const [swtLoading, setSwtLoading] = useState(false);
  const [lwcResult, setLwcResult] = useState<LiveWriteConfirmationPolicyResult | null>(null);
  const [lwcLoading, setLwcLoading] = useState(false);
  const [rlbResult, setRlbResult] = useState<RateLimitBackoffPolicyResult | null>(null);
  const [rlbLoading, setRlbLoading] = useState(false);
  const [cdpResult, setCdpResult] = useState<CheckpointDurabilityPolicyResult | null>(null);
  const [cdpLoading, setCdpLoading] = useState(false);
  const [fvpResult, setFvpResult] = useState<FinalValidationPolicyResult | null>(null);
  const [fvpLoading, setFvpLoading] = useState(false);
  const [wpoResult, setWpoResult] = useState<WritePhaseOrderingPolicyResult | null>(null);
  const [wpoLoading, setWpoLoading] = useState(false);
  const [fmpResult, setFmpResult] = useState<FailureModesPolicyResult | null>(null);
  const [fmpLoading, setFmpLoading] = useState(false);
  const [rlpResult, setRlpResult] = useState<RollbackLimitationPolicyResult | null>(null);
  const [rlpLoading, setRlpLoading] = useState(false);
  const [fveResult, setFveResult] = useState<FinalValidationEnforcementPolicyResult | null>(null);
  const [fveLoading, setFveLoading] = useState(false);
  const [sdsResult, setSdsResult] = useState<SensitiveDataSafetyPolicyResult | null>(null);
  const [sdsLoading, setSdsLoading] = useState(false);
  const [apdResult, setApdResult] = useState<AttachmentPhaseDisabledPolicyResult | null>(null);
  const [apdLoading, setApdLoading] = useState(false);
  const [lwrResult, setLwrResult] = useState<LiveWriteReadinessPolicyResult | null>(null);
  const [lwrLoading, setLwrLoading] = useState(false);
  const [swepResult, setSwepResult] = useState<SchemaWriteExecutionPreviewResult | null>(null);
  const [swepLoading, setSwepLoading] = useState(false);
  const [rwepResult, setRwepResult] = useState<RecordWriteExecutionPreviewResult | null>(null);
  const [rwepLoading, setRwepLoading] = useState(false);
  const [mcepResult, setMcepResult] = useState<MappingCheckpointExecutionPreviewResult | null>(
    null,
  );
  const [mcepLoading, setMcepLoading] = useState(false);

  return (
    <div className="page">
      <div className="page-content">
        {/* Restore form section */}
        <section aria-labelledby="restore-heading">
          <SectionHeader title="Restore from Backup" />

          <div
            className="card"
            style={{
              maxWidth: 600,
              display: "flex",
              flexDirection: "column",
              gap: "var(--space-6)",
            }}
          >
            {/* Backup file selector (placeholder — path shared with inspection panel) */}
            <div className="form-field">
              <label className="form-label">Backup File</label>
              <div style={{ display: "flex", gap: "var(--space-2)", alignItems: "center" }}>
                <input
                  type="text"
                  className="form-input"
                  placeholder="No file selected"
                  disabled
                  readOnly
                  aria-label="Selected backup file path"
                />
                <button
                  type="button"
                  className="btn btn-secondary"
                  disabled
                  aria-label="Choose .airbridge backup file"
                  style={{ flexShrink: 0, whiteSpace: "nowrap" }}
                >
                  <svg
                    width="13"
                    height="13"
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
                  Choose .airbridge file
                </button>
              </div>
            </div>

            <div className="divider" style={{ margin: 0 }} />

            {/* Inspection panel — notifies RestorePage of results */}
            <PackageInspectionPanel
              service={liveAirBridgeService}
              onInspected={(result, path) => {
                setInspection(result);
                setPackagePath(path);
                setDryRunPlan(null);
              }}
            />

            <div className="divider" style={{ margin: 0 }} />

            {/* Restore plan preview — notifies RestorePage of plan + target selection */}
            <RestoreDryRunPanel
              service={liveAirBridgeService}
              onPlanReady={(plan, mode, baseName) => {
                setDryRunPlan(plan);
                setTargetMode(mode);
                setTargetBaseName(baseName);
              }}
            />

            <div className="divider" style={{ margin: 0 }} />

            {/* Schema creation plan — no Airtable calls, no token */}
            <RestoreSchemaPlanPanel
              service={liveAirBridgeService}
              packageFilename={inspection?.filename ?? null}
              dryRunStatus={
                dryRunPlan ? (dryRunPlan.status as "ready" | "readyWithWarnings" | "blocked") : null
              }
              targetMode={targetMode}
              targetBaseName={targetBaseName}
              tables={
                dryRunPlan
                  ? dryRunPlan.tables.map((t): RestoreSchemaPlanRequest["tables"][number] => ({
                      tableId: t.tableId,
                      tableName: t.tableName,
                      fields: t.fields.map((f) => ({
                        fieldId: f.fieldId,
                        fieldName: f.fieldName,
                        fieldType: f.fieldType,
                        linkedTableId:
                          t.linkedRecordPlans.find((lp) => lp.fieldId === f.fieldId)
                            ?.linkedTableId ?? undefined,
                      })),
                    }))
                  : []
              }
              onPlanReady={(plan) => {
                setSchemaPlan(plan);
                setSchemaPlanStatus(plan.status as RestoreRecordImportPlanStatus);
                setRecordImportTables(
                  dryRunPlan
                    ? dryRunPlan.tables.map((t) => ({
                        tableId: t.tableId,
                        tableName: t.tableName,
                        recordCount: undefined,
                        fields: t.fields.map((f) => ({
                          fieldId: f.fieldId,
                          fieldName: f.fieldName,
                          fieldType: f.fieldType,
                          linkedTableId:
                            t.linkedRecordPlans.find((lp) => lp.fieldId === f.fieldId)
                              ?.linkedTableId ?? undefined,
                        })),
                      }))
                    : [],
                );
                // Request the write engine skeleton preview using counts from the schema plan.
                // No token required. No Airtable calls are made.
                liveAirBridgeService
                  .previewRestoreWriteEngine({
                    packageFilename: plan.filename,
                    packagePath: packagePath ?? "",
                    schemaTableCount: plan.tableSteps.length,
                    schemaDirectFieldCount: plan.fieldSteps.length,
                    schemaDeferredFieldCount: plan.deferredSteps.length,
                    schemaManualActionCount: plan.manualActionFields.length,
                    schemaUnsupportedCount: 0,
                  })
                  .then(setWriteEngineResult)
                  .catch(() => {
                    setWriteEngineResult(null);
                  });
              }}
            />

            <div className="divider" style={{ margin: 0 }} />

            {/* Record import plan — no Airtable calls, no token */}
            <RestoreRecordImportPlanPanel
              service={liveAirBridgeService}
              packageFilename={inspection?.filename ?? null}
              dryRunStatus={
                dryRunPlan ? (dryRunPlan.status as "ready" | "readyWithWarnings" | "blocked") : null
              }
              schemaPlanStatus={schemaPlanStatus}
              targetMode={targetMode}
              targetBaseName={targetBaseName}
              tables={recordImportTables}
            />

            <div className="divider" style={{ margin: 0 }} />

            {/* Sandbox verification — Gate 1. No Airtable calls. No token. No writes. */}
            <RestoreSandboxVerificationPanel
              result={sandboxResult}
              loading={sandboxLoading}
              onVerify={() => {
                setSandboxLoading(true);
                liveAirBridgeService
                  .verifyRestoreSandboxEnvironment({
                    targetMode,
                    targetBaseName,
                    expectsEmptyTarget: true,
                    allowAttachmentUpload: false,
                    allowDestructiveOperations: false,
                    sourcePackageFilename: inspection?.filename ?? undefined,
                    schemaPlanStatus: schemaPlanStatus ?? undefined,
                  })
                  .then((r) => {
                    setSandboxResult(r);
                  })
                  .catch(() => {
                    setSandboxResult(null);
                  })
                  .finally(() => {
                    setSandboxLoading(false);
                  });
              }}
            />

            <div className="divider" style={{ margin: 0 }} />

            {/* Restore confirmation — Gate 2. No Airtable calls. No token. No writes. */}
            <RestoreConfirmationPanel
              result={confirmationResult}
              loading={confirmationLoading}
              requiredText={
                confirmationResult?.requiredText ??
                (targetBaseName
                  ? `RESTORE TO ${targetBaseName.toUpperCase()}`
                  : inspection?.filename
                    ? `RESTORE ${inspection.filename.toUpperCase()}`
                    : "RESTORE BACKUP")
              }
              onValidate={(enteredText) => {
                setConfirmationLoading(true);
                liveAirBridgeService
                  .validateRestoreConfirmationGate({
                    enteredText,
                    sourcePackageLabel: inspection?.filename ?? undefined,
                    targetLabel: targetBaseName ?? undefined,
                    sandboxVerificationStatus: sandboxResult?.status ?? undefined,
                  })
                  .then((r) => {
                    setConfirmationResult(r);
                  })
                  .catch(() => {
                    setConfirmationResult(null);
                  })
                  .finally(() => {
                    setConfirmationLoading(false);
                  });
              }}
            />

            <div className="divider" style={{ margin: 0 }} />

            {/* Target empty verification — Gate 3. No writes. No token. */}
            <RestoreTargetEmptyVerificationPanel
              result={targetEmptyResult}
              loading={targetEmptyLoading}
              onVerify={() => {
                setTargetEmptyLoading(true);
                liveAirBridgeService
                  .verifyRestoreTargetEmpty({
                    targetMode: targetMode,
                    targetDisplayName: targetBaseName ?? undefined,
                    liveCheckPerformed: false,
                  })
                  .then((r) => {
                    setTargetEmptyResult(r);
                  })
                  .catch(() => {
                    setTargetEmptyResult(null);
                  })
                  .finally(() => {
                    setTargetEmptyLoading(false);
                  });
              }}
            />

            <div className="divider" style={{ margin: 0 }} />

            {/* Destructive operation policy — Gate 4. No writes. No token. */}
            <RestoreDestructiveOperationPolicyPanel
              result={dopResult}
              loading={dopLoading}
              onVerify={() => {
                setDopLoading(true);
                liveAirBridgeService
                  .verifyDestructiveOperationPolicy({
                    declaredOperations: [],
                    targetDisplayName: targetBaseName ?? undefined,
                  })
                  .then((r) => {
                    setDopResult(r);
                  })
                  .catch(() => {
                    setDopResult(null);
                  })
                  .finally(() => {
                    setDopLoading(false);
                  });
              }}
            />

            <div className="divider" style={{ margin: 0 }} />

            {/* Attachment upload policy — Gate 5. No writes. No token. No file bytes uploaded. */}
            <RestoreAttachmentUploadPolicyPanel
              result={aupResult}
              loading={aupLoading}
              onVerify={() => {
                setAupLoading(true);
                liveAirBridgeService
                  .verifyAttachmentUploadPolicy({
                    declaredAttachmentFields: [],
                    targetDisplayName: targetBaseName ?? undefined,
                  })
                  .then((r) => {
                    setAupResult(r);
                  })
                  .catch(() => {
                    setAupResult(null);
                  })
                  .finally(() => {
                    setAupLoading(false);
                  });
              }}
            />

            <div className="divider" style={{ margin: 0 }} />

            {/* Schema record order policy — Gate 6. No writes. No token. No record payload. */}
            <RestoreSchemaRecordOrderPolicyPanel
              result={sroResult}
              loading={sroLoading}
              onVerify={() => {
                setSroLoading(true);
                liveAirBridgeService
                  .verifySchemaRecordOrderPolicy({
                    declaredPhases: [],
                    targetDisplayName: targetBaseName ?? undefined,
                  })
                  .then((r) => {
                    setSroResult(r);
                  })
                  .catch(() => {
                    setSroResult(null);
                  })
                  .finally(() => {
                    setSroLoading(false);
                  });
              }}
            />

            <div className="divider" style={{ margin: 0 }} />

            {/* Sandbox write testing policy — Gate 7. No writes. No token. No record payload. */}
            <RestoreSandboxWriteTestingPolicyPanel
              result={swtResult}
              loading={swtLoading}
              onVerify={() => {
                setSwtLoading(true);
                liveAirBridgeService
                  .verifySandboxWriteTestingPolicy({
                    targetClassification: "unknown",
                    sandboxVerificationPassed: false,
                    targetDisplayName: targetBaseName ?? undefined,
                  })
                  .then((r) => {
                    setSwtResult(r);
                  })
                  .catch(() => {
                    setSwtResult(null);
                  })
                  .finally(() => {
                    setSwtLoading(false);
                  });
              }}
            />

            <div className="divider" style={{ margin: 0 }} />

            {/* Live write confirmation policy — Gate 8. No writes. No token. No record payload. */}
            <RestoreLiveWriteConfirmationPolicyPanel
              result={lwcResult}
              loading={lwcLoading}
              requiredText={
                lwcResult?.requiredText ??
                `LIVE RESTORE ${
                  (targetBaseName ?? "TARGET")
                    .replace(/[^a-zA-Z0-9\-_. ]/g, "")
                    .trim()
                    .slice(0, 64)
                    .toUpperCase() || "TARGET"
                } — WRITES REMAIN DISABLED`
              }
              onVerify={(enteredText) => {
                setLwcLoading(true);
                liveAirBridgeService
                  .verifyLiveWriteConfirmationPolicy({
                    enteredText,
                    targetLabel: targetBaseName ?? undefined,
                    priorGateStatuses: {
                      sandboxVerificationStatus: sandboxResult?.status ?? undefined,
                      sandboxWriteTestingPolicyStatus: swtResult?.status ?? undefined,
                    },
                  })
                  .then((r) => {
                    setLwcResult(r);
                  })
                  .catch(() => {
                    setLwcResult(null);
                  })
                  .finally(() => {
                    setLwcLoading(false);
                  });
              }}
            />

            <div className="divider" style={{ margin: 0 }} />

            {/* Rate-limit and backoff policy — Gate 9. No writes. No token. No record payload. */}
            <RestoreRateLimitBackoffPolicyPanel
              result={rlbResult}
              loading={rlbLoading}
              onVerify={() => {
                setRlbLoading(true);
                liveAirBridgeService
                  .verifyRateLimitBackoffPolicy({
                    targetLabel: targetBaseName ?? undefined,
                  })
                  .then((r) => {
                    setRlbResult(r);
                  })
                  .catch(() => {
                    setRlbResult(null);
                  })
                  .finally(() => {
                    setRlbLoading(false);
                  });
              }}
            />

            <div className="divider" style={{ margin: 0 }} />

            {/* Checkpoint durability policy — Gate 10. No writes. No token. No record payload. */}
            <RestoreCheckpointDurabilityPolicyPanel
              result={cdpResult}
              loading={cdpLoading}
              onVerify={() => {
                setCdpLoading(true);
                liveAirBridgeService
                  .verifyCheckpointDurabilityPolicy({
                    targetLabel: targetBaseName ?? undefined,
                  })
                  .then((r) => {
                    setCdpResult(r);
                  })
                  .catch(() => {
                    setCdpResult(null);
                  })
                  .finally(() => {
                    setCdpLoading(false);
                  });
              }}
            />

            <div className="divider" style={{ margin: 0 }} />

            {/* Final validation policy — Gate 11. No writes. No token. No record payload. No success state. */}
            <RestoreFinalValidationPolicyPanel
              result={fvpResult}
              loading={fvpLoading}
              onVerify={() => {
                setFvpLoading(true);
                liveAirBridgeService
                  .verifyFinalValidationPolicy({
                    targetLabel: targetBaseName ?? undefined,
                  })
                  .then((r) => {
                    setFvpResult(r);
                  })
                  .catch(() => {
                    setFvpResult(null);
                  })
                  .finally(() => {
                    setFvpLoading(false);
                  });
              }}
            />

            <div className="divider" style={{ margin: 0 }} />

            {/* Write phase ordering policy — Gate 12. No writes. No token. No record payload. No success state. */}
            <RestoreWritePhaseOrderingPolicyPanel
              result={wpoResult}
              loading={wpoLoading}
              onVerify={() => {
                setWpoLoading(true);
                liveAirBridgeService
                  .verifyWritePhaseOrderingPolicy({
                    targetLabel: targetBaseName ?? undefined,
                  })
                  .then((r) => {
                    setWpoResult(r);
                  })
                  .catch(() => {
                    setWpoResult(null);
                  })
                  .finally(() => {
                    setWpoLoading(false);
                  });
              }}
            />

            <div className="divider" style={{ margin: 0 }} />

            {/* Failure modes policy — Gate 13. No writes. No token. No record payload. No success state. */}
            <RestoreFailureModesPolicyPanel
              result={fmpResult}
              loading={fmpLoading}
              onVerify={() => {
                setFmpLoading(true);
                liveAirBridgeService
                  .verifyFailureModesPolicy({
                    targetLabel: targetBaseName ?? undefined,
                  })
                  .then((r) => {
                    setFmpResult(r);
                  })
                  .catch(() => {
                    setFmpResult(null);
                  })
                  .finally(() => {
                    setFmpLoading(false);
                  });
              }}
            />

            <div className="divider" style={{ margin: 0 }} />

            {/* Rollback limitation policy — Gate 14. No writes. No token. No record payload. No success state. No automatic rollback/cleanup. */}
            <RestoreRollbackLimitationPolicyPanel
              result={rlpResult}
              loading={rlpLoading}
              onVerify={() => {
                setRlpLoading(true);
                liveAirBridgeService
                  .verifyRollbackLimitationPolicy({
                    targetLabel: targetBaseName ?? undefined,
                  })
                  .then((r) => {
                    setRlpResult(r);
                  })
                  .catch(() => {
                    setRlpResult(null);
                  })
                  .finally(() => {
                    setRlpLoading(false);
                  });
              }}
            />

            <div className="divider" style={{ margin: 0 }} />

            {/* Final validation enforcement policy — Gate 15. No writes. No token. No record payload. No success state. No result labeled complete without final validation passing. */}
            <RestoreFinalValidationEnforcementPolicyPanel
              result={fveResult}
              loading={fveLoading}
              onVerify={() => {
                setFveLoading(true);
                liveAirBridgeService
                  .verifyFinalValidationEnforcementPolicy({
                    targetLabel: targetBaseName ?? undefined,
                  })
                  .then((r) => {
                    setFveResult(r);
                  })
                  .catch(() => {
                    setFveResult(null);
                  })
                  .finally(() => {
                    setFveLoading(false);
                  });
              }}
            />

            <div className="divider" style={{ margin: 0 }} />

            {/* Sensitive data safety policy — Gate 16. No writes. No token. No full path. No package path. No record payload. No attachment URL. No raw HTTP. */}
            <RestoreSensitiveDataSafetyPolicyPanel
              result={sdsResult}
              loading={sdsLoading}
              onVerify={() => {
                setSdsLoading(true);
                liveAirBridgeService
                  .verifySensitiveDataSafetyPolicy({
                    targetLabel: targetBaseName ?? undefined,
                  })
                  .then((r) => {
                    setSdsResult(r);
                  })
                  .catch(() => {
                    setSdsResult(null);
                  })
                  .finally(() => {
                    setSdsLoading(false);
                  });
              }}
            />

            <div className="divider" style={{ margin: 0 }} />

            {/* Attachment phase disabled policy — Gate 17. No writes. No binary download. No upload. No URL fetch. No attachment transfer. No token. No record payload. No attachment URL. */}
            <RestoreAttachmentPhaseDisabledPolicyPanel
              result={apdResult}
              loading={apdLoading}
              onVerify={() => {
                setApdLoading(true);
                liveAirBridgeService
                  .verifyAttachmentPhaseDisabledPolicy({
                    targetLabel: targetBaseName ?? undefined,
                  })
                  .then((r) => {
                    setApdResult(r);
                  })
                  .catch(() => {
                    setApdResult(null);
                  })
                  .finally(() => {
                    setApdLoading(false);
                  });
              }}
            />

            <div className="divider" style={{ margin: 0 }} />

            {/* Live write readiness policy — Gate 18. Advisory only. No writes. No execute. No token. No record payload. No attachment URL. Ready does NOT enable writes. */}
            <RestoreLiveWriteReadinessPolicyPanel
              result={lwrResult}
              loading={lwrLoading}
              onVerify={() => {
                setLwrLoading(true);
                liveAirBridgeService
                  .verifyLiveWriteReadinessPolicy({
                    targetLabel: targetBaseName ?? undefined,
                  })
                  .then((r) => {
                    setLwrResult(r);
                  })
                  .catch(() => {
                    setLwrResult(null);
                  })
                  .finally(() => {
                    setLwrLoading(false);
                  });
              }}
            />

            <div className="divider" style={{ margin: 0 }} />

            {/* Schema write execution preview — dry-run only. No live writes. No execute button. No token. No record payload. No attachment URL. DryRunReady does NOT enable writes. */}
            <RestoreSchemaWriteExecutionPreviewPanel
              result={swepResult}
              loading={swepLoading}
              onPreview={() => {
                setSwepLoading(true);
                liveAirBridgeService
                  .previewSchemaWriteExecution({
                    packageFilename: inspection?.filename ?? undefined,
                    sandboxFlagPresent: true,
                    targetEmptyVerified: true,
                    schemaPlanReady: true,
                    destructivePolicySafe: true,
                    sensitiveDataSafe: true,
                    attachmentPhaseDisabled: true,
                    finalValidationEnforcementPresent: true,
                    liveWriteReadinessSatisfied:
                      lwrResult?.status === "ready" || lwrResult?.status === "warning",
                  })
                  .then((r) => {
                    setSwepResult(r);
                  })
                  .catch(() => {
                    setSwepResult(null);
                  })
                  .finally(() => {
                    setSwepLoading(false);
                  });
              }}
            />

            <div className="divider" style={{ margin: 0 }} />

            {/* Record write execution preview — dry-run only. No live writes. No execute button. No token. No record payload. No attachment URL. DryRunReady does NOT enable writes. */}
            <RestoreRecordWriteExecutionPreviewPanel
              result={rwepResult}
              loading={rwepLoading}
              request={{
                packageFilename: inspection?.filename ?? undefined,
                schemaPreviewReady: swepResult?.status === "dryRunReady",
                sandboxFlagPresent: true,
                targetEmptyVerified: true,
                recordImportPlanReady: true,
                recordWriteRequestPlanReady: true,
                tableCount: schemaPlan?.tableSteps?.length ?? 0,
                totalFirstPassBatches: 2,
                totalSecondPassBatches: 1,
                totalRecordCount: recordImportTables.reduce(
                  (sum, t) => sum + (t.recordCount ?? 0),
                  0,
                ),
                batchSize: 10,
                rateLimitBackoffSafe: true,
                checkpointDurabilitySafe: true,
                sensitiveDataSafe: true,
                attachmentPhaseDisabled: true,
                finalValidationEnforcementPresent: true,
                liveWriteReadinessSatisfied:
                  lwrResult?.status === "ready" || lwrResult?.status === "warning",
              }}
              onPreview={async (req) => {
                setRwepLoading(true);
                try {
                  const r = await liveAirBridgeService.previewRecordWriteExecution(req);
                  setRwepResult(r);
                  return r;
                } catch {
                  setRwepResult(null);
                  throw new Error("Record write execution preview failed.");
                } finally {
                  setRwepLoading(false);
                }
              }}
            />

            <div className="divider" style={{ margin: 0 }} />

            {/* Mapping/checkpoint execution preview — dry-run only. No live writes. No execute button. No token. No record IDs. No checkpoint files. DryRunReady does NOT enable writes. */}
            <RestoreMappingCheckpointExecutionPreviewPanel
              result={mcepResult}
              loading={mcepLoading}
              request={{
                packageFilename: inspection?.filename ?? undefined,
                recordWritePreviewReady: rwepResult?.status === "dryRunReady",
                firstPassBatchCount: rwepResult?.firstPassBatchCount ?? 0,
                secondPassBatchCount: rwepResult?.secondPassBatchCount ?? 0,
                totalRecordCount: rwepResult?.totalRecordCount ?? 0,
                tablesRequiringRemapping: schemaPlan?.tableSteps?.length ?? 0,
                checkpointDurabilitySafe: true,
                failureModesSafe: true,
                rollbackLimitationSafe: true,
                finalValidationEnforcementPresent: true,
                sensitiveDataSafe: true,
                liveWriteReadinessSatisfied:
                  lwrResult?.status === "ready" || lwrResult?.status === "warning",
              }}
              onPreview={async (req) => {
                setMcepLoading(true);
                try {
                  const r = await liveAirBridgeService.previewMappingCheckpointExecution(req);
                  setMcepResult(r);
                  return r;
                } catch {
                  setMcepResult(null);
                  throw new Error("Mapping/checkpoint execution preview failed.");
                } finally {
                  setMcepLoading(false);
                }
              }}
            />

            <div className="divider" style={{ margin: 0 }} />

            {/* Restore execution gate */}
            <RestoreExecutionGatePanel
              service={liveAirBridgeService}
              inspectedFilename={inspection?.filename ?? null}
              inspectionStatus={
                inspection ? (inspection.validationStatus as "valid" | "warning" | "invalid") : null
              }
              packagePath={packagePath}
              dryRunStatus={
                dryRunPlan ? (dryRunPlan.status as "ready" | "readyWithWarnings" | "blocked") : null
              }
              targetMode={targetMode}
              targetBaseName={targetBaseName}
            />

            <div className="divider" style={{ margin: 0 }} />

            {/* Write engine skeleton — always shows disabled notice; shows preview when schema plan is ready */}
            <RestoreWriteEnginePanel result={schemaPlan !== null ? writeEngineResult : null} />

            <div className="divider" style={{ margin: 0 }} />

            {/* Restore options (legacy placeholder) */}
            <div>
              <p
                style={{
                  fontSize: "var(--text-xs)",
                  fontWeight: 600,
                  color: "var(--color-text-muted)",
                  textTransform: "uppercase",
                  letterSpacing: "0.06em",
                  marginBottom: "var(--space-4)",
                }}
              >
                Restore Options
              </p>

              <div style={{ display: "flex", flexDirection: "column", gap: "var(--space-4)" }}>
                {/* Dry-run toggle */}
                <label
                  style={{
                    display: "flex",
                    alignItems: "center",
                    gap: "var(--space-3)",
                    cursor: "not-allowed",
                    opacity: 0.7,
                  }}
                >
                  <input
                    type="checkbox"
                    disabled
                    style={{ width: 16, height: 16, flexShrink: 0 }}
                    aria-label="Enable dry-run mode"
                  />
                  <div>
                    <div style={{ fontSize: "var(--text-sm)", fontWeight: 500 }}>Dry-run mode</div>
                    <div style={{ fontSize: "var(--text-xs)", color: "var(--color-text-muted)" }}>
                      Simulate the restore without writing any changes
                    </div>
                  </div>
                </label>

                {/* Target base */}
                <div className="form-field">
                  <label htmlFor="target-base-select" className="form-label">
                    Target Base
                  </label>
                  <select
                    id="target-base-select"
                    className="form-input"
                    disabled
                    aria-label="Select target Airtable base for restore"
                  >
                    {bases.length === 0 ? (
                      <option value="">No bases connected</option>
                    ) : (
                      bases.map((base) => (
                        <option key={base.id} value={base.id}>
                          {base.name}
                        </option>
                      ))
                    )}
                  </select>
                </div>
              </div>
            </div>
          </div>
        </section>

        {/* Compatibility section */}
        <section aria-labelledby="compatibility-heading">
          <SectionHeader title="Compatibility" />

          <div
            className="card"
            style={{
              maxWidth: 600,
              display: "flex",
              flexDirection: "column",
              gap: "var(--space-4)",
            }}
          >
            {/* Summary row */}
            <div style={{ display: "flex", gap: "var(--space-6)", flexWrap: "wrap" }}>
              <div style={{ display: "flex", flexDirection: "column", gap: "var(--space-1)" }}>
                <span
                  style={{
                    fontSize: "var(--text-xs)",
                    color: "var(--color-text-muted)",
                    textTransform: "uppercase",
                    letterSpacing: "0.06em",
                  }}
                >
                  Restorable
                </span>
                <span style={{ fontSize: "var(--text-lg)", fontWeight: 600 }}>
                  {compatibilitySummary.bySupport.restorable}
                </span>
              </div>
              <div style={{ display: "flex", flexDirection: "column", gap: "var(--space-1)" }}>
                <span
                  style={{
                    fontSize: "var(--text-xs)",
                    color: "var(--color-text-muted)",
                    textTransform: "uppercase",
                    letterSpacing: "0.06em",
                  }}
                >
                  Partial
                </span>
                <span style={{ fontSize: "var(--text-lg)", fontWeight: 600 }}>
                  {compatibilitySummary.bySupport.partially_restorable}
                </span>
              </div>
              <div style={{ display: "flex", flexDirection: "column", gap: "var(--space-1)" }}>
                <span
                  style={{
                    fontSize: "var(--text-xs)",
                    color: "var(--color-text-muted)",
                    textTransform: "uppercase",
                    letterSpacing: "0.06em",
                  }}
                >
                  Unsupported
                </span>
                <span style={{ fontSize: "var(--text-lg)", fontWeight: 600 }}>
                  {compatibilitySummary.bySupport.unsupported_for_restore}
                </span>
              </div>
            </div>

            {/* Plan warnings */}
            {plan && plan.warnings.length > 0 && (
              <div>
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
                  Warnings for selected plan
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
                >
                  {plan.warnings.map((w) => (
                    <li
                      key={w.fieldId}
                      className={`notice notice-${w.severity === "warning" ? "warning" : w.severity === "error" ? "danger" : "info"}`}
                    >
                      <span>
                        <strong>{w.fieldName}</strong> ({w.fieldType}): {w.message}
                      </span>
                    </li>
                  ))}
                </ul>
              </div>
            )}

            {!plan && (
              <p style={{ fontSize: "var(--text-sm)", color: "var(--color-text-muted)" }}>
                Field compatibility and warnings will appear here after selecting a backup file.
              </p>
            )}
          </div>
        </section>
      </div>
    </div>
  );
}
