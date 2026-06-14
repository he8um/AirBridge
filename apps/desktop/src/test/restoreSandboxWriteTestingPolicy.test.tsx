import { describe, it, expect, vi } from "vitest";
import { render, screen, fireEvent } from "@testing-library/react";
import { RestoreSandboxWriteTestingPolicyPanel } from "../features/backups/RestoreSandboxWriteTestingPolicyPanel";
import { mockAirBridgeService } from "../services/mockAirBridgeService";
import type {
  SandboxWriteTestingPolicyRequest,
  SandboxWriteTestingPolicyResult,
  SandboxWriteTestEvidence,
} from "../backend/types";

// ── Helpers ───────────────────────────────────────────────────────────────────

function completeEvidence(): SandboxWriteTestEvidence {
  return {
    sandboxBaseVerified: true,
    testPackageFilename: "test-backup.airbridge",
    dryRunCompleted: true,
    schemaPlanReviewed: true,
    recordPlanReviewed: true,
    reviewerLabel: "sandbox-test",
  };
}

function compliantRequest(name = "My Base"): SandboxWriteTestingPolicyRequest {
  return {
    targetClassification: "sandbox",
    sandboxVerificationPassed: true,
    evidence: completeEvidence(),
    targetDisplayName: name,
  };
}

// ── Mock service contract ─────────────────────────────────────────────────────

describe("mockAirBridgeService — verifySandboxWriteTestingPolicy contract", () => {
  it("complete evidence and sandbox target returns compliant", async () => {
    const result = await mockAirBridgeService.verifySandboxWriteTestingPolicy(compliantRequest());
    expect(result.status).toBe("compliant");
  });

  it("no evidence returns blocked", async () => {
    const result = await mockAirBridgeService.verifySandboxWriteTestingPolicy({
      targetClassification: "sandbox",
      sandboxVerificationPassed: true,
      targetDisplayName: "My Base",
    });
    expect(result.status).toBe("blocked");
  });

  it("production target returns blocked", async () => {
    const result = await mockAirBridgeService.verifySandboxWriteTestingPolicy({
      ...compliantRequest(),
      targetClassification: "production",
    });
    expect(result.status).toBe("blocked");
  });

  it("unknown target returns blocked", async () => {
    const result = await mockAirBridgeService.verifySandboxWriteTestingPolicy({
      ...compliantRequest(),
      targetClassification: "unknown",
    });
    expect(result.status).toBe("blocked");
  });

  it("sandbox verification not passed returns blocked", async () => {
    const result = await mockAirBridgeService.verifySandboxWriteTestingPolicy({
      ...compliantRequest(),
      sandboxVerificationPassed: false,
    });
    expect(result.status).toBe("blocked");
  });

  it("partial evidence returns warning", async () => {
    const result = await mockAirBridgeService.verifySandboxWriteTestingPolicy({
      ...compliantRequest(),
      evidence: {
        sandboxBaseVerified: true,
        testPackageFilename: "test.airbridge",
        dryRunCompleted: true,
        schemaPlanReviewed: false,
        recordPlanReviewed: false,
      },
    });
    expect(result.status).toBe("warning");
  });

  it("missing filename produces warning", async () => {
    const ev = completeEvidence();
    delete (ev as Partial<typeof ev>).testPackageFilename;
    const result = await mockAirBridgeService.verifySandboxWriteTestingPolicy({
      ...compliantRequest(),
      evidence: ev,
    });
    expect(result.status).toBe("warning");
  });

  it("filename with path separator produces warning", async () => {
    const result = await mockAirBridgeService.verifySandboxWriteTestingPolicy({
      ...compliantRequest(),
      evidence: { ...completeEvidence(), testPackageFilename: "/Users/test/test.airbridge" },
    });
    expect(result.status).toBe("warning");
  });

  it("five checks always present", async () => {
    const result = await mockAirBridgeService.verifySandboxWriteTestingPolicy(compliantRequest());
    expect(result.checks).toHaveLength(5);
  });

  it("check IDs are SWT-01 through SWT-05", async () => {
    const result = await mockAirBridgeService.verifySandboxWriteTestingPolicy(compliantRequest());
    const ids = result.checks.map((c) => c.checkId);
    expect(ids).toContain("SWT-01");
    expect(ids).toContain("SWT-02");
    expect(ids).toContain("SWT-03");
    expect(ids).toContain("SWT-04");
    expect(ids).toContain("SWT-05");
  });

  it("SWT-01 always passes", async () => {
    const result = await mockAirBridgeService.verifySandboxWriteTestingPolicy({
      targetClassification: "unknown",
      sandboxVerificationPassed: false,
    });
    const swt01 = result.checks.find((c) => c.checkId === "SWT-01");
    expect(swt01?.status).toBe("passed");
  });

  it("noChangesMade always true", async () => {
    for (const req of [
      compliantRequest(),
      { targetClassification: "production" as const, sandboxVerificationPassed: false },
    ]) {
      const result = await mockAirBridgeService.verifySandboxWriteTestingPolicy(req);
      expect(result.noChangesMade).toBe(true);
    }
  });

  it("writesEnabled always false", async () => {
    for (const req of [
      compliantRequest(),
      {
        targetClassification: "sandbox" as const,
        sandboxVerificationPassed: true,
        evidence: completeEvidence(),
      },
    ]) {
      const result = await mockAirBridgeService.verifySandboxWriteTestingPolicy(req);
      expect(result.writesEnabled).toBe(false);
    }
  });

  it("networkWritesAttempted always false", async () => {
    const result = await mockAirBridgeService.verifySandboxWriteTestingPolicy(compliantRequest());
    expect(result.networkWritesAttempted).toBe(false);
  });

  it("compliant result does not enable writes", async () => {
    const result = await mockAirBridgeService.verifySandboxWriteTestingPolicy(compliantRequest());
    expect(result.status).toBe("compliant");
    expect(result.writesEnabled).toBe(false);
  });

  it("no token in message", async () => {
    for (const req of [
      compliantRequest(),
      { targetClassification: "production" as const, sandboxVerificationPassed: false },
    ]) {
      const result = await mockAirBridgeService.verifySandboxWriteTestingPolicy(req);
      expect(result.message).not.toContain("token");
      expect(result.message).not.toContain("pat_");
    }
  });

  it("no full path in message", async () => {
    const result = await mockAirBridgeService.verifySandboxWriteTestingPolicy(compliantRequest());
    expect(result.message).not.toContain("/Users/");
    expect(result.message).not.toContain("/home/");
  });

  it("no record payload in message", async () => {
    const result = await mockAirBridgeService.verifySandboxWriteTestingPolicy(compliantRequest());
    expect(result.message).not.toContain("fields");
    expect(result.message).not.toContain("recordId");
  });

  it("compliant message says writes remain disabled", async () => {
    const result = await mockAirBridgeService.verifySandboxWriteTestingPolicy(compliantRequest());
    expect(result.message).toContain("disabled");
  });

  it("display name appears in message", async () => {
    const result = await mockAirBridgeService.verifySandboxWriteTestingPolicy(compliantRequest());
    expect(result.message).toContain("My Base");
  });

  it("IPC fallback shape has safety invariants", () => {
    const fallback: SandboxWriteTestingPolicyResult = {
      status: "blocked",
      checks: [],
      message: "Sandbox write testing policy check is not available in this context.",
      noChangesMade: true,
      networkWritesAttempted: false,
      writesEnabled: false,
    };
    expect(fallback.writesEnabled).toBe(false);
    expect(fallback.noChangesMade).toBe(true);
    expect(fallback.networkWritesAttempted).toBe(false);
  });

  it("no evidence result has five checks", async () => {
    const result = await mockAirBridgeService.verifySandboxWriteTestingPolicy({
      targetClassification: "sandbox",
      sandboxVerificationPassed: true,
    });
    expect(result.checks).toHaveLength(5);
  });
});

// ── Panel rendering ───────────────────────────────────────────────────────────

function renderPanel(
  result: SandboxWriteTestingPolicyResult | null,
  loading = false,
  onVerify = vi.fn(),
) {
  return render(
    <RestoreSandboxWriteTestingPolicyPanel result={result} loading={loading} onVerify={onVerify} />,
  );
}

const COMPLIANT_RESULT: SandboxWriteTestingPolicyResult = {
  status: "compliant",
  checks: [
    {
      checkId: "SWT-01",
      label: "write-gate-disabled",
      status: "passed",
      message: "Gate disabled.",
    },
    {
      checkId: "SWT-02",
      label: "sandbox-target-classification",
      status: "passed",
      message: "Sandbox.",
    },
    {
      checkId: "SWT-03",
      label: "sandbox-verification-passed",
      status: "passed",
      message: "Verified.",
    },
    {
      checkId: "SWT-04",
      label: "sandbox-test-evidence-present",
      status: "passed",
      message: "Present.",
    },
    {
      checkId: "SWT-05",
      label: "sandbox-evidence-complete",
      status: "passed",
      message: "Complete.",
    },
  ],
  message:
    "Sandbox write testing policy for My Base is satisfied. All required evidence is present. Restore writes remain disabled.",
  noChangesMade: true,
  networkWritesAttempted: false,
  writesEnabled: false,
};

const BLOCKED_RESULT: SandboxWriteTestingPolicyResult = {
  status: "blocked",
  checks: [
    {
      checkId: "SWT-01",
      label: "write-gate-disabled",
      status: "passed",
      message: "Gate disabled.",
    },
    {
      checkId: "SWT-02",
      label: "sandbox-target-classification",
      status: "failed",
      message: "Production target.",
      remediation: "Use sandbox.",
    },
    {
      checkId: "SWT-03",
      label: "sandbox-verification-passed",
      status: "passed",
      message: "Verified.",
    },
    {
      checkId: "SWT-04",
      label: "sandbox-test-evidence-present",
      status: "passed",
      message: "Present.",
    },
    {
      checkId: "SWT-05",
      label: "sandbox-evidence-complete",
      status: "passed",
      message: "Complete.",
    },
  ],
  message: "Sandbox write testing policy for My Base is blocked.",
  noChangesMade: true,
  networkWritesAttempted: false,
  writesEnabled: false,
};

describe("RestoreSandboxWriteTestingPolicyPanel rendering", () => {
  it("panel testid present", () => {
    renderPanel(null);
    expect(screen.getByTestId("restore-swt-panel")).toBeInTheDocument();
  });

  it("writes disabled notice always shown", () => {
    renderPanel(null);
    expect(screen.getByTestId("swt-writes-disabled-notice")).toBeInTheDocument();
  });

  it("verify button present", () => {
    renderPanel(null);
    expect(screen.getByTestId("swt-verify-button")).toBeInTheDocument();
  });

  it("button disabled when loading", () => {
    renderPanel(null, true);
    expect(screen.getByTestId("swt-verify-button")).toBeDisabled();
  });

  it("button shows re-verify label when result present", () => {
    renderPanel(COMPLIANT_RESULT);
    expect(screen.getByTestId("swt-verify-button")).toHaveTextContent("Re-verify");
  });

  it("onVerify callback fired on button click", () => {
    const onVerify = vi.fn();
    renderPanel(null, false, onVerify);
    fireEvent.click(screen.getByTestId("swt-verify-button"));
    expect(onVerify).toHaveBeenCalledOnce();
  });

  it("no result area before verify", () => {
    renderPanel(null);
    expect(screen.queryByTestId("swt-result")).not.toBeInTheDocument();
  });

  it("result area shown after verify", () => {
    renderPanel(COMPLIANT_RESULT);
    expect(screen.getByTestId("swt-result")).toBeInTheDocument();
  });

  it("compliant badge shown for compliant status", () => {
    renderPanel(COMPLIANT_RESULT);
    expect(screen.getByTestId("swt-compliant-badge")).toBeInTheDocument();
  });

  it("blocked badge shown for blocked status", () => {
    renderPanel(BLOCKED_RESULT);
    expect(screen.getByTestId("swt-blocked-badge")).toBeInTheDocument();
  });

  it("warning badge shown for warning status", () => {
    const warnResult: SandboxWriteTestingPolicyResult = {
      ...COMPLIANT_RESULT,
      status: "warning",
      message: "Evidence incomplete.",
    };
    renderPanel(warnResult);
    expect(screen.getByTestId("swt-warning-badge")).toBeInTheDocument();
  });

  it("message text rendered", () => {
    renderPanel(COMPLIANT_RESULT);
    expect(screen.getByTestId("swt-message")).toHaveTextContent("Sandbox write testing policy");
  });

  it("check rows rendered", () => {
    renderPanel(COMPLIANT_RESULT);
    const rows = screen.getAllByTestId("swt-check-row");
    expect(rows).toHaveLength(5);
  });

  it("safety summary rendered", () => {
    renderPanel(COMPLIANT_RESULT);
    expect(screen.getByTestId("swt-safety-summary")).toBeInTheDocument();
  });

  it("no-changes notice rendered", () => {
    renderPanel(COMPLIANT_RESULT);
    expect(screen.getByTestId("swt-no-changes-notice")).toBeInTheDocument();
  });

  it("compliant-notice shown only for compliant", () => {
    renderPanel(COMPLIANT_RESULT);
    expect(screen.getByTestId("swt-compliant-notice")).toBeInTheDocument();
    expect(screen.queryByTestId("swt-warning-notice")).not.toBeInTheDocument();
    expect(screen.queryByTestId("swt-blocked-notice")).not.toBeInTheDocument();
  });

  it("blocked-notice shown only for blocked", () => {
    renderPanel(BLOCKED_RESULT);
    expect(screen.getByTestId("swt-blocked-notice")).toBeInTheDocument();
    expect(screen.queryByTestId("swt-compliant-notice")).not.toBeInTheDocument();
    expect(screen.queryByTestId("swt-warning-notice")).not.toBeInTheDocument();
  });

  it("compliant notice says writes remain disabled", () => {
    renderPanel(COMPLIANT_RESULT);
    const notice = screen.getByTestId("swt-compliant-notice");
    expect(notice).toHaveTextContent("disabled");
  });

  it("no execute button present", () => {
    renderPanel(COMPLIANT_RESULT);
    expect(screen.queryByText(/execute/i)).not.toBeInTheDocument();
    expect(screen.queryByText(/run restore/i)).not.toBeInTheDocument();
  });

  it("no token input present", () => {
    renderPanel(COMPLIANT_RESULT);
    expect(screen.queryByLabelText(/token/i)).not.toBeInTheDocument();
    expect(screen.queryByPlaceholderText(/token/i)).not.toBeInTheDocument();
  });

  it("no succeeded language in panel", () => {
    renderPanel(COMPLIANT_RESULT);
    expect(screen.queryByText(/succeeded/i)).not.toBeInTheDocument();
    expect(screen.queryByText(/restore complete/i)).not.toBeInTheDocument();
  });
});
