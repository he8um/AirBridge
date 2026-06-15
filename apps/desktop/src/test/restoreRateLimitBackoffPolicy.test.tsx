import { render, screen, fireEvent } from "@testing-library/react";
import { describe, it, expect, vi } from "vitest";
import { RestoreRateLimitBackoffPolicyPanel } from "../features/backups/RestoreRateLimitBackoffPolicyPanel";
import { mockAirBridgeService } from "../services/mockAirBridgeService";
import type {
  RateLimitBackoffPolicyRequest,
  RateLimitBackoffPolicyResult,
  RateLimitBackoffPlan,
} from "../backend/types";

// ── Helpers ───────────────────────────────────────────────────────────────────

const SAFE_PLAN: RateLimitBackoffPlan = {
  maxRequestsPerSecond: 5,
  batchSize: 10,
  handles429: true,
  maxRetries: 3,
  hasBackoffStrategy: true,
  hasStopCondition: true,
  checkpointCompatibility: "full",
};

async function runMock(
  request: RateLimitBackoffPolicyRequest,
): Promise<RateLimitBackoffPolicyResult> {
  return mockAirBridgeService.verifyRateLimitBackoffPolicy(request);
}

// ── mock service: status outcomes ─────────────────────────────────────────────

describe("mockAirBridgeService.verifyRateLimitBackoffPolicy — status", () => {
  it("returns compliant for a safe plan", async () => {
    const result = await runMock({ plan: SAFE_PLAN });
    expect(result.status).toBe("compliant");
  });

  it("returns blocked when no plan provided", async () => {
    const result = await runMock({});
    expect(result.status).toBe("blocked");
  });

  it("returns blocked when plan is undefined", async () => {
    const result = await runMock({ plan: undefined });
    expect(result.status).toBe("blocked");
  });

  it("returns blocked when rps exceeds 5", async () => {
    const result = await runMock({ plan: { ...SAFE_PLAN, maxRequestsPerSecond: 6 } });
    expect(result.status).toBe("blocked");
  });

  it("returns blocked when rps equals 6", async () => {
    const result = await runMock({ plan: { ...SAFE_PLAN, maxRequestsPerSecond: 6 } });
    expect(result.status).toBe("blocked");
  });

  it("returns compliant when rps equals 5 (boundary)", async () => {
    const result = await runMock({ plan: { ...SAFE_PLAN, maxRequestsPerSecond: 5 } });
    expect(result.status).toBe("compliant");
  });

  it("returns compliant when rps equals 1", async () => {
    const result = await runMock({ plan: { ...SAFE_PLAN, maxRequestsPerSecond: 1 } });
    expect(result.status).toBe("compliant");
  });

  it("returns blocked when batch_size exceeds 10", async () => {
    const result = await runMock({ plan: { ...SAFE_PLAN, batchSize: 11 } });
    expect(result.status).toBe("blocked");
  });

  it("returns compliant when batch_size equals 10 (boundary)", async () => {
    const result = await runMock({ plan: { ...SAFE_PLAN, batchSize: 10 } });
    expect(result.status).toBe("compliant");
  });

  it("returns blocked when handles_429 is false", async () => {
    const result = await runMock({ plan: { ...SAFE_PLAN, handles429: false } });
    expect(result.status).toBe("blocked");
  });

  it("returns blocked when max_retries is undefined (unbounded)", async () => {
    const result = await runMock({ plan: { ...SAFE_PLAN, maxRetries: undefined } });
    expect(result.status).toBe("blocked");
  });

  it("returns blocked when has_backoff_strategy is false", async () => {
    const result = await runMock({ plan: { ...SAFE_PLAN, hasBackoffStrategy: false } });
    expect(result.status).toBe("blocked");
  });

  it("returns blocked when has_stop_condition is false", async () => {
    const result = await runMock({ plan: { ...SAFE_PLAN, hasStopCondition: false } });
    expect(result.status).toBe("blocked");
  });

  it("returns warning when checkpoint_compatibility is partial", async () => {
    const result = await runMock({
      plan: { ...SAFE_PLAN, checkpointCompatibility: "partial" },
    });
    expect(result.status).toBe("warning");
  });

  it("returns warning when checkpoint_compatibility is none", async () => {
    const result = await runMock({
      plan: { ...SAFE_PLAN, checkpointCompatibility: "none" },
    });
    expect(result.status).toBe("warning");
  });

  it("returns warning when checkpoint_compatibility is unknown", async () => {
    const result = await runMock({
      plan: { ...SAFE_PLAN, checkpointCompatibility: "unknown" },
    });
    expect(result.status).toBe("warning");
  });

  it("returns compliant when checkpoint_compatibility is full", async () => {
    const result = await runMock({
      plan: { ...SAFE_PLAN, checkpointCompatibility: "full" },
    });
    expect(result.status).toBe("compliant");
  });

  it("returns blocked (not warning) when hard constraint + bad checkpoint", async () => {
    const result = await runMock({
      plan: { ...SAFE_PLAN, handles429: false, checkpointCompatibility: "partial" },
    });
    expect(result.status).toBe("blocked");
  });
});

// ── mock service: check counts ────────────────────────────────────────────────

describe("mockAirBridgeService.verifyRateLimitBackoffPolicy — check counts", () => {
  it("returns 10 checks for a complete plan", async () => {
    const result = await runMock({ plan: SAFE_PLAN });
    expect(result.checks).toHaveLength(10);
  });

  it("returns 2 checks when no plan provided (short-circuit)", async () => {
    const result = await runMock({});
    expect(result.checks).toHaveLength(2);
  });
});

// ── mock service: check IDs ───────────────────────────────────────────────────

describe("mockAirBridgeService.verifyRateLimitBackoffPolicy — check IDs", () => {
  it("has RLB-01 through RLB-10 for complete plan", async () => {
    const result = await runMock({ plan: SAFE_PLAN });
    const ids = result.checks.map((c) => c.checkId);
    for (let i = 1; i <= 10; i++) {
      expect(ids).toContain(`RLB-${String(i).padStart(2, "0")}`);
    }
  });

  it("RLB-01 always passes (no plan)", async () => {
    const result = await runMock({});
    const rlb01 = result.checks.find((c) => c.checkId === "RLB-01");
    expect(rlb01?.status).toBe("passed");
  });

  it("RLB-01 always passes (safe plan)", async () => {
    const result = await runMock({ plan: SAFE_PLAN });
    const rlb01 = result.checks.find((c) => c.checkId === "RLB-01");
    expect(rlb01?.status).toBe("passed");
  });

  it("RLB-10 always passes", async () => {
    const result = await runMock({ plan: SAFE_PLAN });
    const rlb10 = result.checks.find((c) => c.checkId === "RLB-10");
    expect(rlb10?.status).toBe("passed");
  });

  it("RLB-02 fails when no plan", async () => {
    const result = await runMock({});
    const rlb02 = result.checks.find((c) => c.checkId === "RLB-02");
    expect(rlb02?.status).toBe("failed");
  });

  it("RLB-02 passes when plan present", async () => {
    const result = await runMock({ plan: SAFE_PLAN });
    const rlb02 = result.checks.find((c) => c.checkId === "RLB-02");
    expect(rlb02?.status).toBe("passed");
  });

  it("RLB-03 fails when rps > 5", async () => {
    const result = await runMock({ plan: { ...SAFE_PLAN, maxRequestsPerSecond: 10 } });
    const rlb03 = result.checks.find((c) => c.checkId === "RLB-03");
    expect(rlb03?.status).toBe("failed");
  });

  it("RLB-04 fails when batch_size > 10", async () => {
    const result = await runMock({ plan: { ...SAFE_PLAN, batchSize: 20 } });
    const rlb04 = result.checks.find((c) => c.checkId === "RLB-04");
    expect(rlb04?.status).toBe("failed");
  });

  it("RLB-05 fails when handles_429 is false", async () => {
    const result = await runMock({ plan: { ...SAFE_PLAN, handles429: false } });
    const rlb05 = result.checks.find((c) => c.checkId === "RLB-05");
    expect(rlb05?.status).toBe("failed");
  });

  it("RLB-06 fails when max_retries is undefined", async () => {
    const result = await runMock({ plan: { ...SAFE_PLAN, maxRetries: undefined } });
    const rlb06 = result.checks.find((c) => c.checkId === "RLB-06");
    expect(rlb06?.status).toBe("failed");
  });

  it("RLB-07 fails when has_backoff_strategy is false", async () => {
    const result = await runMock({ plan: { ...SAFE_PLAN, hasBackoffStrategy: false } });
    const rlb07 = result.checks.find((c) => c.checkId === "RLB-07");
    expect(rlb07?.status).toBe("failed");
  });

  it("RLB-08 fails when has_stop_condition is false", async () => {
    const result = await runMock({ plan: { ...SAFE_PLAN, hasStopCondition: false } });
    const rlb08 = result.checks.find((c) => c.checkId === "RLB-08");
    expect(rlb08?.status).toBe("failed");
  });

  it("RLB-09 warns when checkpoint_compatibility is partial", async () => {
    const result = await runMock({
      plan: { ...SAFE_PLAN, checkpointCompatibility: "partial" },
    });
    const rlb09 = result.checks.find((c) => c.checkId === "RLB-09");
    expect(rlb09?.status).toBe("warning");
  });

  it("RLB-09 passes when checkpoint_compatibility is full", async () => {
    const result = await runMock({
      plan: { ...SAFE_PLAN, checkpointCompatibility: "full" },
    });
    const rlb09 = result.checks.find((c) => c.checkId === "RLB-09");
    expect(rlb09?.status).toBe("passed");
  });
});

// ── mock service: plan summary ────────────────────────────────────────────────

describe("mockAirBridgeService.verifyRateLimitBackoffPolicy — plan summary", () => {
  it("includes planSummary when plan is provided", async () => {
    const result = await runMock({ plan: SAFE_PLAN });
    expect(result.planSummary).toBeDefined();
  });

  it("planSummary is undefined when no plan", async () => {
    const result = await runMock({});
    expect(result.planSummary).toBeUndefined();
  });

  it("planSummary.maxRequestsPerSecond matches plan", async () => {
    const result = await runMock({ plan: { ...SAFE_PLAN, maxRequestsPerSecond: 3 } });
    expect(result.planSummary?.maxRequestsPerSecond).toBe(3);
  });

  it("planSummary.batchSize matches plan", async () => {
    const result = await runMock({ plan: { ...SAFE_PLAN, batchSize: 5 } });
    expect(result.planSummary?.batchSize).toBe(5);
  });

  it("planSummary.maxRetries is undefined when not declared", async () => {
    const result = await runMock({ plan: { ...SAFE_PLAN, maxRetries: undefined } });
    expect(result.planSummary?.maxRetries).toBeUndefined();
  });

  it("planSummary.checkpointCompatibility is present when declared", async () => {
    const result = await runMock({
      plan: { ...SAFE_PLAN, checkpointCompatibility: "full" },
    });
    expect(result.planSummary?.checkpointCompatibility).toBe("full");
  });
});

// ── mock service: safety invariants ──────────────────────────────────────────

describe("mockAirBridgeService.verifyRateLimitBackoffPolicy — safety", () => {
  it("noChangesMade is always true (compliant)", async () => {
    const result = await runMock({ plan: SAFE_PLAN });
    expect(result.noChangesMade).toBe(true);
  });

  it("noChangesMade is always true (blocked)", async () => {
    const result = await runMock({});
    expect(result.noChangesMade).toBe(true);
  });

  it("writesEnabled is always false (compliant)", async () => {
    const result = await runMock({ plan: SAFE_PLAN });
    expect(result.writesEnabled).toBe(false);
  });

  it("writesEnabled is always false (blocked)", async () => {
    const result = await runMock({});
    expect(result.writesEnabled).toBe(false);
  });

  it("networkWritesAttempted is always false (compliant)", async () => {
    const result = await runMock({ plan: SAFE_PLAN });
    expect(result.networkWritesAttempted).toBe(false);
  });

  it("networkWritesAttempted is always false (blocked)", async () => {
    const result = await runMock({});
    expect(result.networkWritesAttempted).toBe(false);
  });

  it("compliant status does not enable writes", async () => {
    const result = await runMock({ plan: SAFE_PLAN });
    expect(result.status).toBe("compliant");
    expect(result.writesEnabled).toBe(false);
  });

  it("result does not contain token-like strings", async () => {
    const result = await runMock({ plan: SAFE_PLAN });
    const json = JSON.stringify(result);
    expect(json).not.toContain("pat_");
    expect(json).not.toContain('"token"');
  });

  it("result does not contain full filesystem paths", async () => {
    const result = await runMock({ plan: SAFE_PLAN });
    const json = JSON.stringify(result);
    expect(json).not.toContain("/Users/");
    expect(json).not.toContain("/home/");
  });
});

// ── UI Panel ──────────────────────────────────────────────────────────────────

function makeCompliantResult(): RateLimitBackoffPolicyResult {
  return {
    status: "compliant",
    checks: Array.from({ length: 10 }, (_, i) => ({
      checkId: `RLB-${String(i + 1).padStart(2, "0")}`,
      label: `check-${i + 1}`,
      status: "passed" as const,
      message: `Check ${i + 1} passed.`,
    })),
    message: "Rate-limit and backoff plan is compliant. Restore writes remain disabled.",
    planSummary: {
      maxRequestsPerSecond: 5,
      batchSize: 10,
      handles429: true,
      maxRetries: 3,
      hasBackoffStrategy: true,
      hasStopCondition: true,
      checkpointCompatibility: "full",
    },
    noChangesMade: true,
    networkWritesAttempted: false,
    writesEnabled: false,
  };
}

function makeBlockedResult(): RateLimitBackoffPolicyResult {
  return {
    status: "blocked",
    checks: [
      {
        checkId: "RLB-01",
        label: "write-gate-disabled",
        status: "passed" as const,
        message: "Write gate is disabled.",
      },
      {
        checkId: "RLB-02",
        label: "plan-declared",
        status: "failed" as const,
        message: "No rate-limit plan declared.",
        remediation: "Declare a rate-limit and backoff plan before proceeding.",
      },
    ],
    message: "Rate-limit plan not declared. Restore writes remain disabled.",
    noChangesMade: true,
    networkWritesAttempted: false,
    writesEnabled: false,
  };
}

function makeWarningResult(): RateLimitBackoffPolicyResult {
  return {
    status: "warning",
    checks: Array.from({ length: 10 }, (_, i) => ({
      checkId: `RLB-${String(i + 1).padStart(2, "0")}`,
      label: `check-${i + 1}`,
      status: (i === 8 ? "warning" : "passed") as "passed" | "warning" | "failed",
      message: i === 8 ? "Checkpoint compatibility is partial." : `Check ${i + 1} passed.`,
    })),
    message: "Rate-limit plan has warnings. Restore writes remain disabled.",
    planSummary: {
      maxRequestsPerSecond: 5,
      batchSize: 10,
      handles429: true,
      maxRetries: 3,
      hasBackoffStrategy: true,
      hasStopCondition: true,
      checkpointCompatibility: "partial",
    },
    noChangesMade: true,
    networkWritesAttempted: false,
    writesEnabled: false,
  };
}

describe("RestoreRateLimitBackoffPolicyPanel", () => {
  it("renders the panel root", () => {
    render(<RestoreRateLimitBackoffPolicyPanel result={null} loading={false} onVerify={vi.fn()} />);
    expect(screen.getByTestId("restore-rlb-panel")).toBeInTheDocument();
  });

  it("shows writes-disabled notice", () => {
    render(<RestoreRateLimitBackoffPolicyPanel result={null} loading={false} onVerify={vi.fn()} />);
    expect(screen.getByTestId("rlb-writes-disabled-notice")).toBeInTheDocument();
  });

  it("shows the verify button", () => {
    render(<RestoreRateLimitBackoffPolicyPanel result={null} loading={false} onVerify={vi.fn()} />);
    expect(screen.getByTestId("rlb-verify-button")).toBeInTheDocument();
  });

  it("calls onVerify when button clicked", () => {
    const onVerify = vi.fn();
    render(
      <RestoreRateLimitBackoffPolicyPanel result={null} loading={false} onVerify={onVerify} />,
    );
    fireEvent.click(screen.getByTestId("rlb-verify-button"));
    expect(onVerify).toHaveBeenCalledTimes(1);
  });

  it("shows loading state", () => {
    render(<RestoreRateLimitBackoffPolicyPanel result={null} loading={true} onVerify={vi.fn()} />);
    expect(screen.getByTestId("rlb-verify-button")).toHaveTextContent("Checking…");
    expect(screen.getByTestId("rlb-verify-button")).toBeDisabled();
  });

  it("does not show result section before verify", () => {
    render(<RestoreRateLimitBackoffPolicyPanel result={null} loading={false} onVerify={vi.fn()} />);
    expect(screen.queryByTestId("rlb-result")).not.toBeInTheDocument();
  });

  it("shows compliant badge and notice when status is compliant", () => {
    render(
      <RestoreRateLimitBackoffPolicyPanel
        result={makeCompliantResult()}
        loading={false}
        onVerify={vi.fn()}
      />,
    );
    expect(screen.getByTestId("rlb-compliant-badge")).toBeInTheDocument();
    expect(screen.getByTestId("rlb-compliant-notice")).toBeInTheDocument();
  });

  it("shows warning badge and notice when status is warning", () => {
    render(
      <RestoreRateLimitBackoffPolicyPanel
        result={makeWarningResult()}
        loading={false}
        onVerify={vi.fn()}
      />,
    );
    expect(screen.getByTestId("rlb-warning-badge")).toBeInTheDocument();
    expect(screen.getByTestId("rlb-warning-notice")).toBeInTheDocument();
  });

  it("shows blocked badge and notice when status is blocked", () => {
    render(
      <RestoreRateLimitBackoffPolicyPanel
        result={makeBlockedResult()}
        loading={false}
        onVerify={vi.fn()}
      />,
    );
    expect(screen.getByTestId("rlb-blocked-badge")).toBeInTheDocument();
    expect(screen.getByTestId("rlb-blocked-notice")).toBeInTheDocument();
  });

  it("shows check rows in result table", () => {
    render(
      <RestoreRateLimitBackoffPolicyPanel
        result={makeCompliantResult()}
        loading={false}
        onVerify={vi.fn()}
      />,
    );
    const rows = screen.getAllByTestId("rlb-check-row");
    expect(rows).toHaveLength(10);
  });

  it("shows 2 check rows for blocked no-plan result", () => {
    render(
      <RestoreRateLimitBackoffPolicyPanel
        result={makeBlockedResult()}
        loading={false}
        onVerify={vi.fn()}
      />,
    );
    const rows = screen.getAllByTestId("rlb-check-row");
    expect(rows).toHaveLength(2);
  });

  it("shows the result message", () => {
    render(
      <RestoreRateLimitBackoffPolicyPanel
        result={makeCompliantResult()}
        loading={false}
        onVerify={vi.fn()}
      />,
    );
    expect(screen.getByTestId("rlb-message")).toBeInTheDocument();
  });

  it("shows plan summary when plan is present", () => {
    render(
      <RestoreRateLimitBackoffPolicyPanel
        result={makeCompliantResult()}
        loading={false}
        onVerify={vi.fn()}
      />,
    );
    expect(screen.getByTestId("rlb-plan-summary")).toBeInTheDocument();
    expect(screen.getByTestId("rlb-max-rps")).toBeInTheDocument();
    expect(screen.getByTestId("rlb-batch-size")).toBeInTheDocument();
  });

  it("does not show plan summary for no-plan result", () => {
    render(
      <RestoreRateLimitBackoffPolicyPanel
        result={makeBlockedResult()}
        loading={false}
        onVerify={vi.fn()}
      />,
    );
    expect(screen.queryByTestId("rlb-plan-summary")).not.toBeInTheDocument();
  });

  it("shows safety summary with no-changes-notice", () => {
    render(
      <RestoreRateLimitBackoffPolicyPanel
        result={makeCompliantResult()}
        loading={false}
        onVerify={vi.fn()}
      />,
    );
    expect(screen.getByTestId("rlb-safety-summary")).toBeInTheDocument();
    expect(screen.getByTestId("rlb-no-changes-notice")).toBeInTheDocument();
  });

  it("safety summary shows writesEnabled: no", () => {
    render(
      <RestoreRateLimitBackoffPolicyPanel
        result={makeCompliantResult()}
        loading={false}
        onVerify={vi.fn()}
      />,
    );
    expect(screen.getByTestId("rlb-safety-summary")).toHaveTextContent("Writes enabled: no");
  });

  it("does not contain a token input field", () => {
    render(
      <RestoreRateLimitBackoffPolicyPanel
        result={makeCompliantResult()}
        loading={false}
        onVerify={vi.fn()}
      />,
    );
    const allInputs = document.querySelectorAll('input[type="password"], input[name="token"]');
    expect(allInputs).toHaveLength(0);
  });

  it("does not use the word 'succeeded' anywhere", () => {
    const { container } = render(
      <RestoreRateLimitBackoffPolicyPanel
        result={makeCompliantResult()}
        loading={false}
        onVerify={vi.fn()}
      />,
    );
    expect(container.textContent).not.toMatch(/succeeded/i);
  });

  it("compliant notice says writes remain disabled", () => {
    render(
      <RestoreRateLimitBackoffPolicyPanel
        result={makeCompliantResult()}
        loading={false}
        onVerify={vi.fn()}
      />,
    );
    expect(screen.getByTestId("rlb-compliant-notice").textContent?.toLowerCase()).toContain(
      "disabled",
    );
  });

  it("renders max-rps plan summary value", () => {
    render(
      <RestoreRateLimitBackoffPolicyPanel
        result={makeCompliantResult()}
        loading={false}
        onVerify={vi.fn()}
      />,
    );
    expect(screen.getByTestId("rlb-max-rps")).toHaveTextContent("5");
  });

  it("renders batch-size plan summary value", () => {
    render(
      <RestoreRateLimitBackoffPolicyPanel
        result={makeCompliantResult()}
        loading={false}
        onVerify={vi.fn()}
      />,
    );
    expect(screen.getByTestId("rlb-batch-size")).toHaveTextContent("10");
  });

  it("renders handles-429 plan summary value", () => {
    render(
      <RestoreRateLimitBackoffPolicyPanel
        result={makeCompliantResult()}
        loading={false}
        onVerify={vi.fn()}
      />,
    );
    expect(screen.getByTestId("rlb-handles-429")).toHaveTextContent("yes");
  });

  it("renders checkpoint plan summary value", () => {
    render(
      <RestoreRateLimitBackoffPolicyPanel
        result={makeCompliantResult()}
        loading={false}
        onVerify={vi.fn()}
      />,
    );
    expect(screen.getByTestId("rlb-checkpoint")).toHaveTextContent("full");
  });
});
