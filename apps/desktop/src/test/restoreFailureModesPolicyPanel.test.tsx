import { describe, it, expect, vi } from "vitest";
import { render, screen, fireEvent } from "@testing-library/react";
import { mockAirBridgeService } from "../services/mockAirBridgeService";
import type { RestoreFailureHandlingPlan, FailureModesPolicyRequest } from "../backend/types";
import { RestoreFailureModesPolicyPanel } from "../features/backups/RestoreFailureModesPolicyPanel";

// ── Helpers ───────────────────────────────────────────────────────────────────

function allSafePlans(): RestoreFailureHandlingPlan[] {
  return [
    {
      mode: "schemaCreateFailure",
      stopBehavior: "stopAndReport",
      preservesCheckpoint: true,
      triggersDestructiveRollback: false,
      partialFailureLabeledSuccess: false,
      capturesDiagnosticContext: true,
    },
    {
      mode: "schemaVerifyFailure",
      stopBehavior: "stopAndReport",
      preservesCheckpoint: true,
      triggersDestructiveRollback: false,
      partialFailureLabeledSuccess: false,
      capturesDiagnosticContext: true,
    },
    {
      mode: "recordCreateFailure",
      stopBehavior: "stopPreserveCheckpointAndReport",
      preservesCheckpoint: true,
      triggersDestructiveRollback: false,
      partialFailureLabeledSuccess: false,
      capturesDiagnosticContext: true,
    },
    {
      mode: "idMappingFailure",
      stopBehavior: "stopAndReport",
      preservesCheckpoint: true,
      triggersDestructiveRollback: false,
      partialFailureLabeledSuccess: false,
      capturesDiagnosticContext: true,
    },
    {
      mode: "linkedRecordUpdateFailure",
      stopBehavior: "stopPreserveCheckpointAndReport",
      preservesCheckpoint: true,
      triggersDestructiveRollback: false,
      partialFailureLabeledSuccess: false,
      capturesDiagnosticContext: true,
    },
    {
      mode: "checkpointPersistenceFailure",
      stopBehavior: "stopAndReport",
      preservesCheckpoint: false,
      triggersDestructiveRollback: false,
      partialFailureLabeledSuccess: false,
      capturesDiagnosticContext: true,
    },
    {
      mode: "rateLimitExhaustion",
      stopBehavior: "stopAfterRetryLimit",
      preservesCheckpoint: true,
      triggersDestructiveRollback: false,
      partialFailureLabeledSuccess: false,
      capturesDiagnosticContext: true,
    },
    {
      mode: "targetMutationDetected",
      stopBehavior: "blockAndRequireManualReview",
      preservesCheckpoint: true,
      triggersDestructiveRollback: false,
      partialFailureLabeledSuccess: false,
      capturesDiagnosticContext: true,
    },
    {
      mode: "finalValidationFailure",
      stopBehavior: "stopAndReport",
      preservesCheckpoint: true,
      triggersDestructiveRollback: false,
      partialFailureLabeledSuccess: false,
      capturesDiagnosticContext: true,
    },
    {
      mode: "unknownFailure",
      stopBehavior: "stopAndReport",
      preservesCheckpoint: true,
      triggersDestructiveRollback: false,
      partialFailureLabeledSuccess: false,
      capturesDiagnosticContext: true,
    },
  ];
}

function requestWith(plans: RestoreFailureHandlingPlan[]): FailureModesPolicyRequest {
  return { handlingPlans: plans };
}

function requestNone(): FailureModesPolicyRequest {
  return { handlingPlans: undefined };
}

// ── Service tests ─────────────────────────────────────────────────────────────

describe("verifyFailureModesPolicy service", () => {
  it("returns compliant for complete safe failure plan", async () => {
    const result = await mockAirBridgeService.verifyFailureModesPolicy(requestWith(allSafePlans()));
    expect(result.status).toBe("compliant");
    expect(result.noChangesMade).toBe(true);
    expect(result.writesEnabled).toBe(false);
    expect(result.networkWritesAttempted).toBe(false);
  });

  it("blocks when no plans declared", async () => {
    const result = await mockAirBridgeService.verifyFailureModesPolicy(requestNone());
    expect(result.status).toBe("blocked");
    expect(result.checks.length).toBe(2);
  });

  it("blocks when a required mode is missing", async () => {
    const plans = allSafePlans().filter((p) => p.mode !== "recordCreateFailure");
    const result = await mockAirBridgeService.verifyFailureModesPolicy(requestWith(plans));
    expect(result.status).toBe("blocked");
    const fmp03 = result.checks.find((c) => c.checkId === "FMP-03");
    expect(fmp03?.status).toBe("failed");
    expect(fmp03?.message).toContain("recordCreateFailure");
  });

  it("blocks when destructive rollback is declared", async () => {
    const plans = allSafePlans();
    plans[0] = { ...plans[0], triggersDestructiveRollback: true };
    const result = await mockAirBridgeService.verifyFailureModesPolicy(requestWith(plans));
    expect(result.status).toBe("blocked");
    const fmp05 = result.checks.find((c) => c.checkId === "FMP-05");
    expect(fmp05?.status).toBe("failed");
  });

  it("blocks when unknown failure does not stop all writes — all current stop behaviors do stop writes", async () => {
    // All FailureStopBehavior variants stop writes; validate FMP-06 passes for safe plans
    const result = await mockAirBridgeService.verifyFailureModesPolicy(requestWith(allSafePlans()));
    const fmp06 = result.checks.find((c) => c.checkId === "FMP-06");
    expect(fmp06?.status).toBe("passed");
  });

  it("blocks when final validation failure allows success", async () => {
    const plans = allSafePlans();
    const fvIdx = plans.findIndex((p) => p.mode === "finalValidationFailure");
    plans[fvIdx] = { ...plans[fvIdx], partialFailureLabeledSuccess: true };
    const result = await mockAirBridgeService.verifyFailureModesPolicy(requestWith(plans));
    expect(result.status).toBe("blocked");
    const fmp08 = result.checks.find((c) => c.checkId === "FMP-08");
    expect(fmp08?.status).toBe("failed");
  });

  it("blocks when checkpoint persistence failure does not stop writes — validates FMP-09 passes for safe plans", async () => {
    const result = await mockAirBridgeService.verifyFailureModesPolicy(requestWith(allSafePlans()));
    const fmp09 = result.checks.find((c) => c.checkId === "FMP-09");
    expect(fmp09?.status).toBe("passed");
  });

  it("returns warning when a mode lacks diagnostic context", async () => {
    const plans = allSafePlans();
    plans[0] = { ...plans[0], capturesDiagnosticContext: false };
    const result = await mockAirBridgeService.verifyFailureModesPolicy(requestWith(plans));
    expect(result.status).toBe("warning");
    const warn = result.checks.find((c) => c.status === "warning");
    expect(warn).toBeTruthy();
  });

  it("blocks when partial failure is labeled as success", async () => {
    const plans = allSafePlans();
    plans[2] = { ...plans[2], partialFailureLabeledSuccess: true };
    const result = await mockAirBridgeService.verifyFailureModesPolicy(requestWith(plans));
    expect(result.status).toBe("blocked");
    const fmp10 = result.checks.find((c) => c.checkId === "FMP-10");
    expect(fmp10?.status).toBe("failed");
  });

  it("compliant result does not contain 'succeeded'", async () => {
    const result = await mockAirBridgeService.verifyFailureModesPolicy(requestWith(allSafePlans()));
    expect(result.message.toLowerCase()).not.toContain("succeeded");
  });

  it("returns 11 numbered checks for complete plan", async () => {
    const result = await mockAirBridgeService.verifyFailureModesPolicy(requestWith(allSafePlans()));
    const numbered = result.checks.filter((c) => /^FMP-\d/.test(c.checkId));
    expect(numbered.length).toBe(11);
  });

  it("no-plans short-circuits with 2 checks", async () => {
    const result = await mockAirBridgeService.verifyFailureModesPolicy(requestNone());
    expect(result.checks.length).toBe(2);
  });

  it("FMP-01 and FMP-11 always pass", async () => {
    const result = await mockAirBridgeService.verifyFailureModesPolicy(requestWith(allSafePlans()));
    const fmp01 = result.checks.find((c) => c.checkId === "FMP-01");
    const fmp11 = result.checks.find((c) => c.checkId === "FMP-11");
    expect(fmp01?.status).toBe("passed");
    expect(fmp11?.status).toBe("passed");
  });

  it("handling summary is present for complete plan", async () => {
    const result = await mockAirBridgeService.verifyFailureModesPolicy(requestWith(allSafePlans()));
    expect(result.handlingSummary).toBeTruthy();
    expect(result.handlingSummary!.length).toBe(10);
    expect(result.handlingSummary!.some((e) => e.mode === "unknownFailure")).toBe(true);
  });

  it("safety invariants: no token or payload in result", async () => {
    const result = await mockAirBridgeService.verifyFailureModesPolicy(requestWith(allSafePlans()));
    const json = JSON.stringify(result);
    expect(json).not.toContain("token");
    expect(json).not.toContain("api_key");
    expect(json).not.toContain("/Users/");
    expect(json).not.toContain("record_payload");
  });
});

// ── UI Panel tests ────────────────────────────────────────────────────────────

describe("RestoreFailureModesPolicyPanel", () => {
  it("renders without result", () => {
    render(<RestoreFailureModesPolicyPanel result={null} loading={false} onVerify={vi.fn()} />);
    expect(screen.getByTestId("restore-fmp-panel")).toBeTruthy();
    expect(screen.getByTestId("fmp-writes-disabled-notice")).toBeTruthy();
    expect(screen.getByTestId("fmp-verify-button")).toBeTruthy();
    expect(screen.queryByTestId("fmp-result")).toBeNull();
  });

  it("calls onVerify when button clicked", () => {
    const onVerify = vi.fn();
    render(<RestoreFailureModesPolicyPanel result={null} loading={false} onVerify={onVerify} />);
    fireEvent.click(screen.getByTestId("fmp-verify-button"));
    expect(onVerify).toHaveBeenCalledOnce();
  });

  it("shows compliant badge and notice", async () => {
    const result = await mockAirBridgeService.verifyFailureModesPolicy(requestWith(allSafePlans()));
    render(<RestoreFailureModesPolicyPanel result={result} loading={false} onVerify={vi.fn()} />);
    expect(screen.getByTestId("fmp-compliant-badge")).toBeTruthy();
    expect(screen.getByTestId("fmp-compliant-notice")).toBeTruthy();
    expect(screen.queryByTestId("fmp-blocked-badge")).toBeNull();
    expect(screen.queryByTestId("fmp-warning-badge")).toBeNull();
  });

  it("shows blocked badge and notice when missing modes", async () => {
    const result = await mockAirBridgeService.verifyFailureModesPolicy(requestNone());
    render(<RestoreFailureModesPolicyPanel result={result} loading={false} onVerify={vi.fn()} />);
    expect(screen.getByTestId("fmp-blocked-badge")).toBeTruthy();
    expect(screen.getByTestId("fmp-blocked-notice")).toBeTruthy();
  });

  it("shows warning badge when diagnostic context missing", async () => {
    const plans = allSafePlans();
    plans[0] = { ...plans[0], capturesDiagnosticContext: false };
    const result = await mockAirBridgeService.verifyFailureModesPolicy(requestWith(plans));
    render(<RestoreFailureModesPolicyPanel result={result} loading={false} onVerify={vi.fn()} />);
    expect(screen.getByTestId("fmp-warning-badge")).toBeTruthy();
    expect(screen.getByTestId("fmp-warning-notice")).toBeTruthy();
  });

  it("renders check rows", async () => {
    const result = await mockAirBridgeService.verifyFailureModesPolicy(requestWith(allSafePlans()));
    render(<RestoreFailureModesPolicyPanel result={result} loading={false} onVerify={vi.fn()} />);
    const rows = screen.getAllByTestId("fmp-check-row");
    expect(rows.length).toBeGreaterThan(0);
  });

  it("renders handling summary table", async () => {
    const result = await mockAirBridgeService.verifyFailureModesPolicy(requestWith(allSafePlans()));
    render(<RestoreFailureModesPolicyPanel result={result} loading={false} onVerify={vi.fn()} />);
    expect(screen.getByTestId("fmp-handling-summary")).toBeTruthy();
    const rows = screen.getAllByTestId("fmp-mode-row");
    expect(rows.length).toBe(10);
  });

  it("shows no-changes notice in safety summary", async () => {
    const result = await mockAirBridgeService.verifyFailureModesPolicy(requestWith(allSafePlans()));
    render(<RestoreFailureModesPolicyPanel result={result} loading={false} onVerify={vi.fn()} />);
    expect(screen.getByTestId("fmp-no-changes-notice")).toBeTruthy();
    expect(screen.getByTestId("fmp-safety-summary")).toBeTruthy();
  });

  it("does not contain execute button", () => {
    render(<RestoreFailureModesPolicyPanel result={null} loading={false} onVerify={vi.fn()} />);
    const buttons = screen.getAllByRole("button");
    for (const btn of buttons) {
      expect(btn.textContent?.toLowerCase()).not.toMatch(/execute|start restore|run restore/);
    }
  });

  it("does not contain token, path, payload, or succeeded language", async () => {
    const result = await mockAirBridgeService.verifyFailureModesPolicy(requestWith(allSafePlans()));
    render(<RestoreFailureModesPolicyPanel result={result} loading={false} onVerify={vi.fn()} />);
    const panel = screen.getByTestId("restore-fmp-panel");
    const text = panel.textContent?.toLowerCase() ?? "";
    expect(text).not.toContain("token");
    expect(text).not.toContain("record payload");
    expect(text).not.toContain("succeeded");
  });

  it("writes-disabled notice is always visible with result", async () => {
    const result = await mockAirBridgeService.verifyFailureModesPolicy(requestWith(allSafePlans()));
    render(<RestoreFailureModesPolicyPanel result={result} loading={false} onVerify={vi.fn()} />);
    expect(screen.getByTestId("fmp-writes-disabled-notice")).toBeTruthy();
  });
});
