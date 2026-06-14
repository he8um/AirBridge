import { render, screen, fireEvent, waitFor } from "@testing-library/react";
import { describe, it, expect, vi } from "vitest";
import { RestoreLiveWriteConfirmationPolicyPanel } from "../features/backups/RestoreLiveWriteConfirmationPolicyPanel";
import { mockAirBridgeService } from "../services/mockAirBridgeService";
import type {
  LiveWriteConfirmationPolicyRequest,
  LiveWriteConfirmationPolicyResult,
} from "../backend/types";

// ── Helpers ───────────────────────────────────────────────────────────────────

function allGatesOk() {
  return {
    sandboxVerificationStatus: "verified",
    destructiveOperationPolicyStatus: "compliant",
    attachmentUploadPolicyStatus: "compliant",
    schemaRecordOrderPolicyStatus: "compliant",
    sandboxWriteTestingPolicyStatus: "compliant",
  };
}

function requiredTextFor(label: string): string {
  const safe =
    label
      .replace(/[^a-zA-Z0-9\-_. ]/g, "")
      .trim()
      .slice(0, 64)
      .toUpperCase() || "TARGET";
  return `LIVE RESTORE ${safe} — WRITES REMAIN DISABLED`;
}

async function runMockVerify(
  request: LiveWriteConfirmationPolicyRequest,
): Promise<LiveWriteConfirmationPolicyResult> {
  return mockAirBridgeService.verifyLiveWriteConfirmationPolicy(request);
}

// ── mock service: status outcomes ────────────────────────────────────────────

describe("mockAirBridgeService.verifyLiveWriteConfirmationPolicy — status", () => {
  it("returns confirmed when text matches and all gates ok", async () => {
    const target = "My Base";
    const result = await runMockVerify({
      enteredText: requiredTextFor(target),
      targetLabel: target,
      priorGateStatuses: allGatesOk(),
    });
    expect(result.status).toBe("confirmed");
  });

  it("returns rejected when text is wrong", async () => {
    const result = await runMockVerify({
      enteredText: "wrong text",
      targetLabel: "My Base",
      priorGateStatuses: allGatesOk(),
    });
    expect(result.status).toBe("rejected");
  });

  it("returns rejected when text is correct but lowercased", async () => {
    const target = "My Base";
    const result = await runMockVerify({
      enteredText: requiredTextFor(target).toLowerCase(),
      targetLabel: target,
      priorGateStatuses: allGatesOk(),
    });
    expect(result.status).toBe("rejected");
  });

  it("returns blocked when sandbox gate is blocked (with correct text)", async () => {
    const target = "My Base";
    const result = await runMockVerify({
      enteredText: requiredTextFor(target),
      targetLabel: target,
      priorGateStatuses: { ...allGatesOk(), sandboxVerificationStatus: "blocked" },
    });
    expect(result.status).toBe("blocked");
  });

  it("returns blocked when SWT gate is blocked (with correct text)", async () => {
    const target = "My Base";
    const result = await runMockVerify({
      enteredText: requiredTextFor(target),
      targetLabel: target,
      priorGateStatuses: { ...allGatesOk(), sandboxWriteTestingPolicyStatus: "blocked" },
    });
    expect(result.status).toBe("blocked");
  });

  it("returns rejected (not blocked) when gate is blocked and text is wrong", async () => {
    const result = await runMockVerify({
      enteredText: "wrong text",
      targetLabel: "My Base",
      priorGateStatuses: { ...allGatesOk(), sandboxVerificationStatus: "blocked" },
    });
    expect(result.status).toBe("rejected");
  });

  it("returns warning when prior gate has warning and text matches", async () => {
    const target = "My Base";
    const result = await runMockVerify({
      enteredText: requiredTextFor(target),
      targetLabel: target,
      priorGateStatuses: { ...allGatesOk(), sandboxVerificationStatus: "warning" },
    });
    expect(result.status).toBe("warning");
  });

  it("returns confirmed with no prior gate statuses provided", async () => {
    const target = "My Base";
    const result = await runMockVerify({
      enteredText: requiredTextFor(target),
      targetLabel: target,
      priorGateStatuses: undefined,
    });
    expect(result.status).toBe("confirmed");
  });

  it("returns confirmed with no target label", async () => {
    const result = await runMockVerify({
      enteredText: "LIVE RESTORE TARGET — WRITES REMAIN DISABLED",
      targetLabel: undefined,
      priorGateStatuses: allGatesOk(),
    });
    expect(result.status).toBe("confirmed");
  });
});

// ── mock service: check IDs ───────────────────────────────────────────────────

describe("mockAirBridgeService.verifyLiveWriteConfirmationPolicy — checks", () => {
  it("always returns exactly 5 checks", async () => {
    const result = await runMockVerify({
      enteredText: "wrong",
      targetLabel: "My Base",
      priorGateStatuses: undefined,
    });
    expect(result.checks).toHaveLength(5);
  });

  it("check IDs are LWC-01 through LWC-05", async () => {
    const result = await runMockVerify({
      enteredText: requiredTextFor("My Base"),
      targetLabel: "My Base",
      priorGateStatuses: allGatesOk(),
    });
    const ids = result.checks.map((c) => c.checkId);
    expect(ids).toContain("LWC-01");
    expect(ids).toContain("LWC-02");
    expect(ids).toContain("LWC-03");
    expect(ids).toContain("LWC-04");
    expect(ids).toContain("LWC-05");
  });

  it("LWC-01 always passes", async () => {
    const result = await runMockVerify({
      enteredText: "wrong",
      targetLabel: undefined,
      priorGateStatuses: undefined,
    });
    const lwc01 = result.checks.find((c) => c.checkId === "LWC-01");
    expect(lwc01?.status).toBe("passed");
  });

  it("LWC-05 always passes", async () => {
    const result = await runMockVerify({
      enteredText: "wrong",
      targetLabel: undefined,
      priorGateStatuses: undefined,
    });
    const lwc05 = result.checks.find((c) => c.checkId === "LWC-05");
    expect(lwc05?.status).toBe("passed");
  });

  it("LWC-04 fails when text does not match", async () => {
    const result = await runMockVerify({
      enteredText: "wrong text",
      targetLabel: "My Base",
      priorGateStatuses: allGatesOk(),
    });
    const lwc04 = result.checks.find((c) => c.checkId === "LWC-04");
    expect(lwc04?.status).toBe("failed");
  });

  it("LWC-04 passes when text matches exactly", async () => {
    const target = "My Base";
    const result = await runMockVerify({
      enteredText: requiredTextFor(target),
      targetLabel: target,
      priorGateStatuses: allGatesOk(),
    });
    const lwc04 = result.checks.find((c) => c.checkId === "LWC-04");
    expect(lwc04?.status).toBe("passed");
  });

  it("LWC-02 fails when sandbox gate is blocked", async () => {
    const target = "My Base";
    const result = await runMockVerify({
      enteredText: requiredTextFor(target),
      targetLabel: target,
      priorGateStatuses: { ...allGatesOk(), sandboxVerificationStatus: "blocked" },
    });
    const lwc02 = result.checks.find((c) => c.checkId === "LWC-02");
    expect(lwc02?.status).toBe("failed");
  });

  it("LWC-03 fails when SWT gate is blocked", async () => {
    const target = "My Base";
    const result = await runMockVerify({
      enteredText: requiredTextFor(target),
      targetLabel: target,
      priorGateStatuses: { ...allGatesOk(), sandboxWriteTestingPolicyStatus: "blocked" },
    });
    const lwc03 = result.checks.find((c) => c.checkId === "LWC-03");
    expect(lwc03?.status).toBe("failed");
  });
});

// ── mock service: required text ───────────────────────────────────────────────

describe("mockAirBridgeService.verifyLiveWriteConfirmationPolicy — required text", () => {
  it("required text contains WRITES REMAIN DISABLED", async () => {
    const result = await runMockVerify({
      enteredText: "",
      targetLabel: "My Base",
      priorGateStatuses: allGatesOk(),
    });
    expect(result.requiredText).toContain("WRITES REMAIN DISABLED");
  });

  it("required text starts with LIVE RESTORE", async () => {
    const result = await runMockVerify({
      enteredText: "",
      targetLabel: "My Base",
      priorGateStatuses: allGatesOk(),
    });
    expect(result.requiredText).toMatch(/^LIVE RESTORE /);
  });

  it("required text includes uppercased target label", async () => {
    const result = await runMockVerify({
      enteredText: "",
      targetLabel: "my-base",
      priorGateStatuses: allGatesOk(),
    });
    expect(result.requiredText).toContain("MY-BASE");
  });

  it("required text falls back to TARGET when no label provided", async () => {
    const result = await runMockVerify({
      enteredText: "",
      targetLabel: undefined,
      priorGateStatuses: allGatesOk(),
    });
    expect(result.requiredText).toContain("TARGET");
  });

  it("required text does not contain path separators when label has slashes", async () => {
    const result = await runMockVerify({
      enteredText: "",
      targetLabel: "/Users/test/mybase",
      priorGateStatuses: allGatesOk(),
    });
    expect(result.requiredText).not.toContain("/");
  });
});

// ── mock service: safety invariants ──────────────────────────────────────────

describe("mockAirBridgeService.verifyLiveWriteConfirmationPolicy — safety", () => {
  it("noChangesMade is always true (confirmed)", async () => {
    const target = "My Base";
    const result = await runMockVerify({
      enteredText: requiredTextFor(target),
      targetLabel: target,
      priorGateStatuses: allGatesOk(),
    });
    expect(result.noChangesMade).toBe(true);
  });

  it("noChangesMade is always true (rejected)", async () => {
    const result = await runMockVerify({
      enteredText: "wrong",
      targetLabel: "My Base",
      priorGateStatuses: allGatesOk(),
    });
    expect(result.noChangesMade).toBe(true);
  });

  it("writesEnabled is always false (confirmed)", async () => {
    const target = "My Base";
    const result = await runMockVerify({
      enteredText: requiredTextFor(target),
      targetLabel: target,
      priorGateStatuses: allGatesOk(),
    });
    expect(result.writesEnabled).toBe(false);
  });

  it("writesEnabled is always false (rejected)", async () => {
    const result = await runMockVerify({
      enteredText: "wrong",
      targetLabel: "My Base",
      priorGateStatuses: allGatesOk(),
    });
    expect(result.writesEnabled).toBe(false);
  });

  it("networkWritesAttempted is always false", async () => {
    const target = "My Base";
    const result = await runMockVerify({
      enteredText: requiredTextFor(target),
      targetLabel: target,
      priorGateStatuses: allGatesOk(),
    });
    expect(result.networkWritesAttempted).toBe(false);
  });

  it("confirmed status does not enable writes", async () => {
    const target = "My Base";
    const result = await runMockVerify({
      enteredText: requiredTextFor(target),
      targetLabel: target,
      priorGateStatuses: allGatesOk(),
    });
    expect(result.status).toBe("confirmed");
    expect(result.writesEnabled).toBe(false);
  });

  it("message mentions disabled for confirmed result", async () => {
    const target = "My Base";
    const result = await runMockVerify({
      enteredText: requiredTextFor(target),
      targetLabel: target,
      priorGateStatuses: allGatesOk(),
    });
    expect(result.message.toLowerCase()).toContain("disabled");
  });

  it("result does not contain token-like strings", async () => {
    const result = await runMockVerify({
      enteredText: requiredTextFor("My Base"),
      targetLabel: "My Base",
      priorGateStatuses: allGatesOk(),
    });
    const json = JSON.stringify(result);
    expect(json).not.toContain("pat_");
    expect(json).not.toContain('"token"');
  });

  it("result does not contain full filesystem paths", async () => {
    const result = await runMockVerify({
      enteredText: requiredTextFor("My Base"),
      targetLabel: "My Base",
      priorGateStatuses: allGatesOk(),
    });
    const json = JSON.stringify(result);
    expect(json).not.toContain("/Users/");
    expect(json).not.toContain("/home/");
  });
});

// ── UI Panel ──────────────────────────────────────────────────────────────────

function makeConfirmedResult(target: string): LiveWriteConfirmationPolicyResult {
  return {
    status: "confirmed",
    checks: [
      {
        checkId: "LWC-01",
        label: "write-gate-disabled",
        status: "passed",
        message: "Write gate disabled.",
      },
      {
        checkId: "LWC-02",
        label: "prior-gates-not-blocked",
        status: "passed",
        message: "Prior gates not blocked.",
      },
      {
        checkId: "LWC-03",
        label: "sandbox-write-testing-not-blocked",
        status: "passed",
        message: "Sandbox write testing not blocked.",
      },
      {
        checkId: "LWC-04",
        label: "confirmation-text-match",
        status: "passed",
        message: "Text matched.",
      },
      {
        checkId: "LWC-05",
        label: "writes-remain-disabled",
        status: "passed",
        message: "Writes remain disabled.",
      },
    ],
    requiredText: requiredTextFor(target),
    message: `Live-write confirmation for ${target} accepted. Restore writes remain disabled.`,
    noChangesMade: true,
    networkWritesAttempted: false,
    writesEnabled: false,
  };
}

function makeRejectedResult(): LiveWriteConfirmationPolicyResult {
  return {
    status: "rejected",
    checks: [
      {
        checkId: "LWC-01",
        label: "write-gate-disabled",
        status: "passed",
        message: "Write gate disabled.",
      },
      {
        checkId: "LWC-02",
        label: "prior-gates-not-blocked",
        status: "passed",
        message: "Prior gates not blocked.",
      },
      {
        checkId: "LWC-03",
        label: "sandbox-write-testing-not-blocked",
        status: "passed",
        message: "Sandbox write testing not blocked.",
      },
      {
        checkId: "LWC-04",
        label: "confirmation-text-match",
        status: "failed",
        message: "Text did not match.",
        remediation: "Type exactly: LIVE RESTORE MY BASE — WRITES REMAIN DISABLED",
      },
      {
        checkId: "LWC-05",
        label: "writes-remain-disabled",
        status: "passed",
        message: "Writes remain disabled.",
      },
    ],
    requiredText: requiredTextFor("My Base"),
    message: "Confirmation rejected. Text did not match.",
    noChangesMade: true,
    networkWritesAttempted: false,
    writesEnabled: false,
  };
}

describe("RestoreLiveWriteConfirmationPolicyPanel", () => {
  it("renders the panel root", () => {
    render(
      <RestoreLiveWriteConfirmationPolicyPanel
        result={null}
        loading={false}
        requiredText={requiredTextFor("My Base")}
        onVerify={vi.fn()}
      />,
    );
    expect(screen.getByTestId("restore-lwc-panel")).toBeInTheDocument();
  });

  it("shows writes-disabled notice", () => {
    render(
      <RestoreLiveWriteConfirmationPolicyPanel
        result={null}
        loading={false}
        requiredText={requiredTextFor("My Base")}
        onVerify={vi.fn()}
      />,
    );
    expect(screen.getByTestId("lwc-writes-disabled-notice")).toBeInTheDocument();
  });

  it("shows the required text", () => {
    const req = requiredTextFor("My Base");
    render(
      <RestoreLiveWriteConfirmationPolicyPanel
        result={null}
        loading={false}
        requiredText={req}
        onVerify={vi.fn()}
      />,
    );
    expect(screen.getByTestId("lwc-required-text")).toHaveTextContent(req);
  });

  it("shows the verify button", () => {
    render(
      <RestoreLiveWriteConfirmationPolicyPanel
        result={null}
        loading={false}
        requiredText={requiredTextFor("My Base")}
        onVerify={vi.fn()}
      />,
    );
    expect(screen.getByTestId("lwc-verify-button")).toBeInTheDocument();
  });

  it("calls onVerify with entered text when button clicked", async () => {
    const onVerify = vi.fn();
    render(
      <RestoreLiveWriteConfirmationPolicyPanel
        result={null}
        loading={false}
        requiredText={requiredTextFor("My Base")}
        onVerify={onVerify}
      />,
    );
    const input = screen.getByTestId("lwc-confirmation-input");
    fireEvent.change(input, { target: { value: "LIVE RESTORE MY BASE — WRITES REMAIN DISABLED" } });
    fireEvent.click(screen.getByTestId("lwc-verify-button"));
    await waitFor(() => {
      expect(onVerify).toHaveBeenCalledWith("LIVE RESTORE MY BASE — WRITES REMAIN DISABLED");
    });
  });

  it("verify button is disabled when input is empty", () => {
    render(
      <RestoreLiveWriteConfirmationPolicyPanel
        result={null}
        loading={false}
        requiredText={requiredTextFor("My Base")}
        onVerify={vi.fn()}
      />,
    );
    expect(screen.getByTestId("lwc-verify-button")).toBeDisabled();
  });

  it("shows loading state", () => {
    render(
      <RestoreLiveWriteConfirmationPolicyPanel
        result={null}
        loading={true}
        requiredText={requiredTextFor("My Base")}
        onVerify={vi.fn()}
      />,
    );
    expect(screen.getByTestId("lwc-verify-button")).toHaveTextContent("Checking…");
  });

  it("shows confirmed badge and confirmed notice when status is confirmed", () => {
    render(
      <RestoreLiveWriteConfirmationPolicyPanel
        result={makeConfirmedResult("My Base")}
        loading={false}
        requiredText={requiredTextFor("My Base")}
        onVerify={vi.fn()}
      />,
    );
    expect(screen.getByTestId("lwc-confirmed-badge")).toBeInTheDocument();
    expect(screen.getByTestId("lwc-confirmed-notice")).toBeInTheDocument();
  });

  it("shows rejected badge and rejected notice when status is rejected", () => {
    render(
      <RestoreLiveWriteConfirmationPolicyPanel
        result={makeRejectedResult()}
        loading={false}
        requiredText={requiredTextFor("My Base")}
        onVerify={vi.fn()}
      />,
    );
    expect(screen.getByTestId("lwc-rejected-badge")).toBeInTheDocument();
    expect(screen.getByTestId("lwc-rejected-notice")).toBeInTheDocument();
  });

  it("shows check rows in the result table", () => {
    render(
      <RestoreLiveWriteConfirmationPolicyPanel
        result={makeConfirmedResult("My Base")}
        loading={false}
        requiredText={requiredTextFor("My Base")}
        onVerify={vi.fn()}
      />,
    );
    const rows = screen.getAllByTestId("lwc-check-row");
    expect(rows.length).toBe(5);
  });

  it("shows the message", () => {
    const result = makeConfirmedResult("My Base");
    render(
      <RestoreLiveWriteConfirmationPolicyPanel
        result={result}
        loading={false}
        requiredText={requiredTextFor("My Base")}
        onVerify={vi.fn()}
      />,
    );
    expect(screen.getByTestId("lwc-message")).toBeInTheDocument();
  });

  it("shows safety summary with no-changes-notice", () => {
    render(
      <RestoreLiveWriteConfirmationPolicyPanel
        result={makeConfirmedResult("My Base")}
        loading={false}
        requiredText={requiredTextFor("My Base")}
        onVerify={vi.fn()}
      />,
    );
    expect(screen.getByTestId("lwc-safety-summary")).toBeInTheDocument();
    expect(screen.getByTestId("lwc-no-changes-notice")).toBeInTheDocument();
  });

  it("safety summary shows writesEnabled: no", () => {
    render(
      <RestoreLiveWriteConfirmationPolicyPanel
        result={makeConfirmedResult("My Base")}
        loading={false}
        requiredText={requiredTextFor("My Base")}
        onVerify={vi.fn()}
      />,
    );
    expect(screen.getByTestId("lwc-safety-summary")).toHaveTextContent("Writes enabled: no");
  });

  it("result section is not shown before verify", () => {
    render(
      <RestoreLiveWriteConfirmationPolicyPanel
        result={null}
        loading={false}
        requiredText={requiredTextFor("My Base")}
        onVerify={vi.fn()}
      />,
    );
    expect(screen.queryByTestId("lwc-result")).not.toBeInTheDocument();
  });

  it("does not contain a token input field", () => {
    render(
      <RestoreLiveWriteConfirmationPolicyPanel
        result={makeConfirmedResult("My Base")}
        loading={false}
        requiredText={requiredTextFor("My Base")}
        onVerify={vi.fn()}
      />,
    );
    const allInputs = document.querySelectorAll('input[type="password"], input[name="token"]');
    expect(allInputs).toHaveLength(0);
  });

  it("does not use the word 'succeeded' anywhere", () => {
    const { container } = render(
      <RestoreLiveWriteConfirmationPolicyPanel
        result={makeConfirmedResult("My Base")}
        loading={false}
        requiredText={requiredTextFor("My Base")}
        onVerify={vi.fn()}
      />,
    );
    expect(container.textContent).not.toMatch(/succeeded/i);
  });
});
