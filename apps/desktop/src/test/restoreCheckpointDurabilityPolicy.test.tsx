import { render, screen } from "@testing-library/react";
import userEvent from "@testing-library/user-event";
import { describe, it, expect, vi, beforeEach } from "vitest";
import { RestoreCheckpointDurabilityPolicyPanel } from "../features/backups/RestoreCheckpointDurabilityPolicyPanel";
import type {
  CheckpointDurabilityPlan,
  CheckpointDurabilityPolicyRequest,
  CheckpointDurabilityPolicyResult,
} from "../backend/types";
import { mockAirBridgeService } from "../services/mockAirBridgeService";

// ── Fixtures ──────────────────────────────────────────────────────────────────

const SAFE_PLAN: CheckpointDurabilityPlan = {
  checkpointAfterEachTable: true,
  checkpointAfterEachBatch: true,
  hasPhaseMarkers: true,
  hasIdMappingCheckpoint: true,
  hasResumeSafeStopCondition: true,
  hasLinkedUpdates: true,
  durabilityBackend: "disk",
};

async function runMock(
  request: CheckpointDurabilityPolicyRequest,
): Promise<CheckpointDurabilityPolicyResult> {
  return mockAirBridgeService.verifyCheckpointDurabilityPolicy(request);
}

// ── Service contract tests ─────────────────────────────────────────────────────

describe("verifyCheckpointDurabilityPolicy service contract", () => {
  // ── Status outcomes ────────────────────────────────────────────────────────

  it("complete plan returns compliant", async () => {
    const result = await runMock({ plan: SAFE_PLAN });
    expect(result.status).toBe("compliant");
  });

  it("no plan returns blocked", async () => {
    const result = await runMock({});
    expect(result.status).toBe("blocked");
  });

  it("missing table checkpoint returns blocked", async () => {
    const result = await runMock({ plan: { ...SAFE_PLAN, checkpointAfterEachTable: false } });
    expect(result.status).toBe("blocked");
  });

  it("missing batch checkpoint returns blocked", async () => {
    const result = await runMock({ plan: { ...SAFE_PLAN, checkpointAfterEachBatch: false } });
    expect(result.status).toBe("blocked");
  });

  it("missing phase markers returns blocked", async () => {
    const result = await runMock({ plan: { ...SAFE_PLAN, hasPhaseMarkers: false } });
    expect(result.status).toBe("blocked");
  });

  it("linked updates without id mapping checkpoint returns blocked", async () => {
    const result = await runMock({
      plan: { ...SAFE_PLAN, hasLinkedUpdates: true, hasIdMappingCheckpoint: false },
    });
    expect(result.status).toBe("blocked");
  });

  it("no linked updates, no id mapping checkpoint returns compliant", async () => {
    const result = await runMock({
      plan: { ...SAFE_PLAN, hasLinkedUpdates: false, hasIdMappingCheckpoint: false },
    });
    expect(result.status).toBe("compliant");
  });

  it("missing resume-safe stop condition returns blocked", async () => {
    const result = await runMock({ plan: { ...SAFE_PLAN, hasResumeSafeStopCondition: false } });
    expect(result.status).toBe("blocked");
  });

  it("memory-only backend returns warning", async () => {
    const result = await runMock({ plan: { ...SAFE_PLAN, durabilityBackend: "memory" } });
    expect(result.status).toBe("warning");
  });

  it("unknown backend returns warning", async () => {
    const result = await runMock({ plan: { ...SAFE_PLAN, durabilityBackend: undefined } });
    expect(result.status).toBe("warning");
  });

  it("remote backend returns compliant", async () => {
    const result = await runMock({ plan: { ...SAFE_PLAN, durabilityBackend: "remote" } });
    expect(result.status).toBe("compliant");
  });

  // ── Check counts ───────────────────────────────────────────────────────────

  it("9 checks present when plan declared", async () => {
    const result = await runMock({ plan: SAFE_PLAN });
    expect(result.checks).toHaveLength(9);
  });

  it("2 checks when no plan", async () => {
    const result = await runMock({});
    expect(result.checks).toHaveLength(2);
  });

  // ── Check IDs ─────────────────────────────────────────────────────────────

  it("check IDs CDP-01 through CDP-09 all present", async () => {
    const result = await runMock({ plan: SAFE_PLAN });
    const ids = result.checks.map((c) => c.checkId);
    for (let i = 1; i <= 9; i++) {
      const expected = `CDP-${String(i).padStart(2, "0")}`;
      expect(ids).toContain(expected);
    }
  });

  it("CDP-01 always passes", async () => {
    const result = await runMock({});
    const cdp01 = result.checks.find((c) => c.checkId === "CDP-01");
    expect(cdp01?.status).toBe("passed");
  });

  it("CDP-09 always passes", async () => {
    const result = await runMock({ plan: SAFE_PLAN });
    const cdp09 = result.checks.find((c) => c.checkId === "CDP-09");
    expect(cdp09?.status).toBe("passed");
  });

  it("CDP-03 fails when no table checkpoint", async () => {
    const result = await runMock({ plan: { ...SAFE_PLAN, checkpointAfterEachTable: false } });
    const cdp03 = result.checks.find((c) => c.checkId === "CDP-03");
    expect(cdp03?.status).toBe("failed");
  });

  it("CDP-04 fails when no batch checkpoint", async () => {
    const result = await runMock({ plan: { ...SAFE_PLAN, checkpointAfterEachBatch: false } });
    const cdp04 = result.checks.find((c) => c.checkId === "CDP-04");
    expect(cdp04?.status).toBe("failed");
  });

  it("CDP-06 fails when linked updates without id mapping", async () => {
    const result = await runMock({
      plan: { ...SAFE_PLAN, hasLinkedUpdates: true, hasIdMappingCheckpoint: false },
    });
    const cdp06 = result.checks.find((c) => c.checkId === "CDP-06");
    expect(cdp06?.status).toBe("failed");
  });

  it("CDP-08 warns on memory backend", async () => {
    const result = await runMock({ plan: { ...SAFE_PLAN, durabilityBackend: "memory" } });
    const cdp08 = result.checks.find((c) => c.checkId === "CDP-08");
    expect(cdp08?.status).toBe("warning");
  });

  // ── Plan summary ───────────────────────────────────────────────────────────

  it("plan summary present when plan declared", async () => {
    const result = await runMock({ plan: SAFE_PLAN });
    expect(result.planSummary).toBeDefined();
    expect(result.planSummary?.checkpointAfterEachTable).toBe(true);
    expect(result.planSummary?.checkpointAfterEachBatch).toBe(true);
    expect(result.planSummary?.hasPhaseMarkers).toBe(true);
    expect(result.planSummary?.durabilityBackend).toBe("disk");
  });

  it("plan summary absent when no plan", async () => {
    const result = await runMock({});
    expect(result.planSummary).toBeUndefined();
  });

  // ── Safety invariants ─────────────────────────────────────────────────────

  it("no_changes_made always true (compliant)", async () => {
    const result = await runMock({ plan: SAFE_PLAN });
    expect(result.noChangesMade).toBe(true);
  });

  it("no_changes_made always true (blocked)", async () => {
    const result = await runMock({});
    expect(result.noChangesMade).toBe(true);
  });

  it("writes_enabled always false (compliant)", async () => {
    const result = await runMock({ plan: SAFE_PLAN });
    expect(result.writesEnabled).toBe(false);
  });

  it("writes_enabled always false (blocked)", async () => {
    const result = await runMock({ plan: { ...SAFE_PLAN, checkpointAfterEachBatch: false } });
    expect(result.writesEnabled).toBe(false);
  });

  it("network_writes_attempted always false", async () => {
    const result = await runMock({ plan: SAFE_PLAN });
    expect(result.networkWritesAttempted).toBe(false);
  });

  it("compliant does not enable writes", async () => {
    const result = await runMock({ plan: SAFE_PLAN });
    expect(result.status).toBe("compliant");
    expect(result.writesEnabled).toBe(false);
  });

  // ── Message safety ─────────────────────────────────────────────────────────

  it("message says writes remain disabled when compliant", async () => {
    const result = await runMock({ plan: SAFE_PLAN });
    expect(result.message).toMatch(/disabled/i);
  });

  it("message does not contain token", async () => {
    const result = await runMock({ plan: SAFE_PLAN });
    expect(result.message).not.toMatch(/token/i);
    expect(result.message).not.toMatch(/pat_/);
  });

  it("message does not contain path", async () => {
    const result = await runMock({ plan: SAFE_PLAN });
    expect(result.message).not.toMatch(/\/Users\//);
    expect(result.message).not.toMatch(/\/home\//);
  });

  it("message does not contain record payload language", async () => {
    const result = await runMock({ plan: SAFE_PLAN });
    expect(result.message).not.toMatch(/"fields"/);
    expect(result.message).not.toMatch(/recordId/);
  });

  it("message does not contain succeeded language", async () => {
    const result = await runMock({ plan: SAFE_PLAN });
    expect(result.message).not.toMatch(/succeeded/i);
    expect(result.message).not.toMatch(/success/i);
  });

  // ── No write calls ─────────────────────────────────────────────────────────

  it("no write calls made during verification", async () => {
    const result = await runMock({ plan: SAFE_PLAN });
    expect(result.networkWritesAttempted).toBe(false);
    expect(result.writesEnabled).toBe(false);
  });
});

// ── UI panel tests ─────────────────────────────────────────────────────────────

describe("RestoreCheckpointDurabilityPolicyPanel", () => {
  beforeEach(() => {
    vi.clearAllMocks();
  });

  it("renders panel with writes-disabled notice", () => {
    render(
      <RestoreCheckpointDurabilityPolicyPanel result={null} loading={false} onVerify={vi.fn()} />,
    );
    expect(screen.getByTestId("restore-cdp-panel")).toBeInTheDocument();
    expect(screen.getByTestId("cdp-writes-disabled-notice")).toBeInTheDocument();
  });

  it("renders verify button", () => {
    render(
      <RestoreCheckpointDurabilityPolicyPanel result={null} loading={false} onVerify={vi.fn()} />,
    );
    expect(screen.getByTestId("cdp-verify-button")).toBeInTheDocument();
  });

  it("does not render result section when result is null", () => {
    render(
      <RestoreCheckpointDurabilityPolicyPanel result={null} loading={false} onVerify={vi.fn()} />,
    );
    expect(screen.queryByTestId("cdp-result")).not.toBeInTheDocument();
  });

  it("calls onVerify when button clicked", async () => {
    const onVerify = vi.fn();
    render(
      <RestoreCheckpointDurabilityPolicyPanel result={null} loading={false} onVerify={onVerify} />,
    );
    await userEvent.click(screen.getByTestId("cdp-verify-button"));
    expect(onVerify).toHaveBeenCalledTimes(1);
  });

  it("disables button when loading", () => {
    render(
      <RestoreCheckpointDurabilityPolicyPanel result={null} loading={true} onVerify={vi.fn()} />,
    );
    expect(screen.getByTestId("cdp-verify-button")).toBeDisabled();
  });

  it("shows checking text when loading", () => {
    render(
      <RestoreCheckpointDurabilityPolicyPanel result={null} loading={true} onVerify={vi.fn()} />,
    );
    expect(screen.getByTestId("cdp-verify-button")).toHaveTextContent("Checking");
  });

  // ── Compliant result rendering ─────────────────────────────────────────────

  it("renders compliant badge when status is compliant", async () => {
    const result = await runMock({ plan: SAFE_PLAN });
    render(
      <RestoreCheckpointDurabilityPolicyPanel result={result} loading={false} onVerify={vi.fn()} />,
    );
    expect(screen.getByTestId("cdp-compliant-badge")).toBeInTheDocument();
    expect(screen.queryByTestId("cdp-warning-badge")).not.toBeInTheDocument();
    expect(screen.queryByTestId("cdp-blocked-badge")).not.toBeInTheDocument();
  });

  it("renders compliant notice when status is compliant", async () => {
    const result = await runMock({ plan: SAFE_PLAN });
    render(
      <RestoreCheckpointDurabilityPolicyPanel result={result} loading={false} onVerify={vi.fn()} />,
    );
    expect(screen.getByTestId("cdp-compliant-notice")).toBeInTheDocument();
    expect(screen.queryByTestId("cdp-warning-notice")).not.toBeInTheDocument();
    expect(screen.queryByTestId("cdp-blocked-notice")).not.toBeInTheDocument();
  });

  // ── Warning result rendering ───────────────────────────────────────────────

  it("renders warning badge when status is warning", async () => {
    const result = await runMock({ plan: { ...SAFE_PLAN, durabilityBackend: "memory" } });
    render(
      <RestoreCheckpointDurabilityPolicyPanel result={result} loading={false} onVerify={vi.fn()} />,
    );
    expect(screen.getByTestId("cdp-warning-badge")).toBeInTheDocument();
    expect(screen.queryByTestId("cdp-compliant-badge")).not.toBeInTheDocument();
    expect(screen.queryByTestId("cdp-blocked-badge")).not.toBeInTheDocument();
  });

  it("renders warning notice when status is warning", async () => {
    const result = await runMock({ plan: { ...SAFE_PLAN, durabilityBackend: "memory" } });
    render(
      <RestoreCheckpointDurabilityPolicyPanel result={result} loading={false} onVerify={vi.fn()} />,
    );
    expect(screen.getByTestId("cdp-warning-notice")).toBeInTheDocument();
  });

  // ── Blocked result rendering ───────────────────────────────────────────────

  it("renders blocked badge when status is blocked", async () => {
    const result = await runMock({});
    render(
      <RestoreCheckpointDurabilityPolicyPanel result={result} loading={false} onVerify={vi.fn()} />,
    );
    expect(screen.getByTestId("cdp-blocked-badge")).toBeInTheDocument();
    expect(screen.queryByTestId("cdp-compliant-badge")).not.toBeInTheDocument();
    expect(screen.queryByTestId("cdp-warning-badge")).not.toBeInTheDocument();
  });

  it("renders blocked notice when status is blocked", async () => {
    const result = await runMock({});
    render(
      <RestoreCheckpointDurabilityPolicyPanel result={result} loading={false} onVerify={vi.fn()} />,
    );
    expect(screen.getByTestId("cdp-blocked-notice")).toBeInTheDocument();
  });

  // ── Plan summary rendering ─────────────────────────────────────────────────

  it("renders plan summary when plan is declared", async () => {
    const result = await runMock({ plan: SAFE_PLAN });
    render(
      <RestoreCheckpointDurabilityPolicyPanel result={result} loading={false} onVerify={vi.fn()} />,
    );
    expect(screen.getByTestId("cdp-plan-summary")).toBeInTheDocument();
    expect(screen.getByTestId("cdp-table-checkpoint")).toBeInTheDocument();
    expect(screen.getByTestId("cdp-batch-checkpoint")).toBeInTheDocument();
    expect(screen.getByTestId("cdp-phase-markers")).toBeInTheDocument();
    expect(screen.getByTestId("cdp-id-mapping-checkpoint")).toBeInTheDocument();
    expect(screen.getByTestId("cdp-resume-stop-condition")).toBeInTheDocument();
    expect(screen.getByTestId("cdp-linked-updates")).toBeInTheDocument();
    expect(screen.getByTestId("cdp-durability-backend")).toBeInTheDocument();
  });

  it("does not render plan summary when no plan declared", async () => {
    const result = await runMock({});
    render(
      <RestoreCheckpointDurabilityPolicyPanel result={result} loading={false} onVerify={vi.fn()} />,
    );
    expect(screen.queryByTestId("cdp-plan-summary")).not.toBeInTheDocument();
  });

  // ── Check table rendering ──────────────────────────────────────────────────

  it("renders check rows for each check", async () => {
    const result = await runMock({ plan: SAFE_PLAN });
    render(
      <RestoreCheckpointDurabilityPolicyPanel result={result} loading={false} onVerify={vi.fn()} />,
    );
    const rows = screen.getAllByTestId("cdp-check-row");
    expect(rows).toHaveLength(9);
  });

  // ── Safety summary ─────────────────────────────────────────────────────────

  it("renders safety summary with no-changes notice", async () => {
    const result = await runMock({ plan: SAFE_PLAN });
    render(
      <RestoreCheckpointDurabilityPolicyPanel result={result} loading={false} onVerify={vi.fn()} />,
    );
    expect(screen.getByTestId("cdp-safety-summary")).toBeInTheDocument();
    expect(screen.getByTestId("cdp-no-changes-notice")).toBeInTheDocument();
  });

  // ── Safety: no token, no execute, no success ───────────────────────────────

  it("does not render any token input", async () => {
    const result = await runMock({ plan: SAFE_PLAN });
    render(
      <RestoreCheckpointDurabilityPolicyPanel result={result} loading={false} onVerify={vi.fn()} />,
    );
    expect(screen.queryByRole("textbox", { name: /token/i })).not.toBeInTheDocument();
    const inputs = document.querySelectorAll('input[name="token"], input[type="password"]');
    expect(inputs).toHaveLength(0);
  });

  it("does not render any execute/start-restore button", async () => {
    const result = await runMock({ plan: SAFE_PLAN });
    render(
      <RestoreCheckpointDurabilityPolicyPanel result={result} loading={false} onVerify={vi.fn()} />,
    );
    expect(screen.queryByRole("button", { name: /execute/i })).not.toBeInTheDocument();
    expect(screen.queryByRole("button", { name: /start restore/i })).not.toBeInTheDocument();
  });

  it("does not show succeeded language", async () => {
    const result = await runMock({ plan: SAFE_PLAN });
    render(
      <RestoreCheckpointDurabilityPolicyPanel result={result} loading={false} onVerify={vi.fn()} />,
    );
    expect(screen.queryByText(/succeeded/i)).not.toBeInTheDocument();
    expect(screen.queryByText(/restore complete/i)).not.toBeInTheDocument();
  });

  it("writes-disabled notice always visible", async () => {
    const result = await runMock({ plan: SAFE_PLAN });
    render(
      <RestoreCheckpointDurabilityPolicyPanel result={result} loading={false} onVerify={vi.fn()} />,
    );
    expect(screen.getByTestId("cdp-writes-disabled-notice")).toBeInTheDocument();
  });
});
