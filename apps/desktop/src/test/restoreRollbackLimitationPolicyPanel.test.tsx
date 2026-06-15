import { render, screen, fireEvent } from "@testing-library/react";
import { describe, it, expect, vi } from "vitest";
import { RestoreRollbackLimitationPolicyPanel } from "../features/backups/RestoreRollbackLimitationPolicyPanel";
import type { RollbackLimitationPolicyResult, RollbackLimitationSummary } from "../backend/types";

const safePlanSummary: RollbackLimitationSummary = {
  rollbackBehavior: "noAutomaticRollback",
  partialRestoreIsNotSuccess: true,
  recoveryGuidanceDeclared: true,
  includesCheckpointGuidance: true,
  userVisibleNotice: true,
  manualCleanupRequiresSeparateAction: true,
};

const compliantResult: RollbackLimitationPolicyResult = {
  status: "compliant",
  checks: [
    {
      checkId: "RLP-01",
      label: "write-gate-disabled",
      status: "passed",
      message: "Write gate is disabled.",
    },
    { checkId: "RLP-02", label: "plan-declared", status: "passed", message: "Plan is declared." },
    {
      checkId: "RLP-03",
      label: "no-automatic-destructive-rollback",
      status: "passed",
      message: "No automatic destructive rollback.",
    },
    {
      checkId: "RLP-04",
      label: "no-automatic-delete-cleanup",
      status: "passed",
      message: "No automatic delete cleanup.",
    },
    {
      checkId: "RLP-05",
      label: "no-automatic-update-revert-cleanup",
      status: "passed",
      message: "No automatic update/revert cleanup.",
    },
    {
      checkId: "RLP-06",
      label: "partial-restore-is-not-success",
      status: "passed",
      message: "Partial restore is not success.",
    },
    {
      checkId: "RLP-07",
      label: "checkpoint-recovery-guidance",
      status: "passed",
      message: "Checkpoint guidance declared.",
    },
    {
      checkId: "RLP-08",
      label: "user-visible-limitation-notice",
      status: "passed",
      message: "User-visible notice declared.",
    },
    {
      checkId: "RLP-09",
      label: "manual-cleanup-separate-action",
      status: "passed",
      message: "Manual cleanup requires separate action.",
    },
    {
      checkId: "RLP-10",
      label: "no-token-path-payload",
      status: "passed",
      message: "No token, path, or payload.",
    },
    {
      checkId: "RLP-11",
      label: "no-network-writes",
      status: "passed",
      message: "No network writes.",
    },
    {
      checkId: "RLP-12",
      label: "writes-remain-disabled",
      status: "passed",
      message: "Writes remain disabled.",
    },
  ],
  message: "Rollback limitation policy is compliant. Restore writes remain disabled.",
  planSummary: safePlanSummary,
  noChangesMade: true,
  networkWritesAttempted: false,
  writesEnabled: false,
};

const warningResult: RollbackLimitationPolicyResult = {
  status: "warning",
  checks: [
    {
      checkId: "RLP-01",
      label: "write-gate-disabled",
      status: "passed",
      message: "Write gate is disabled.",
    },
    { checkId: "RLP-02", label: "plan-declared", status: "passed", message: "Plan is declared." },
    {
      checkId: "RLP-03",
      label: "no-automatic-destructive-rollback",
      status: "passed",
      message: "No automatic destructive rollback.",
    },
    {
      checkId: "RLP-04",
      label: "no-automatic-delete-cleanup",
      status: "passed",
      message: "No automatic delete cleanup.",
    },
    {
      checkId: "RLP-05",
      label: "no-automatic-update-revert-cleanup",
      status: "passed",
      message: "No automatic update/revert cleanup.",
    },
    {
      checkId: "RLP-06",
      label: "partial-restore-is-not-success",
      status: "passed",
      message: "Partial restore is not success.",
    },
    {
      checkId: "RLP-07",
      label: "checkpoint-recovery-guidance",
      status: "warning",
      message: "No recovery guidance declared.",
      remediation: "Set recoveryGuidance to checkpointBasedResume or manualCleanupRequired.",
    },
    {
      checkId: "RLP-08",
      label: "user-visible-limitation-notice",
      status: "warning",
      message: "Notice missing limitation details.",
      remediation: "Set noticeIncludesLimitationDetails: true.",
    },
    {
      checkId: "RLP-09",
      label: "manual-cleanup-separate-action",
      status: "passed",
      message: "Manual cleanup requires separate action.",
    },
    {
      checkId: "RLP-10",
      label: "no-token-path-payload",
      status: "passed",
      message: "No token, path, or payload.",
    },
    {
      checkId: "RLP-11",
      label: "no-network-writes",
      status: "passed",
      message: "No network writes.",
    },
    {
      checkId: "RLP-12",
      label: "writes-remain-disabled",
      status: "passed",
      message: "Writes remain disabled.",
    },
  ],
  message: "Rollback limitation policy has warnings. Restore writes remain disabled.",
  planSummary: {
    ...safePlanSummary,
    recoveryGuidanceDeclared: false,
    includesCheckpointGuidance: false,
  },
  noChangesMade: true,
  networkWritesAttempted: false,
  writesEnabled: false,
};

const blockedResult: RollbackLimitationPolicyResult = {
  status: "blocked",
  checks: [
    {
      checkId: "RLP-01",
      label: "write-gate-disabled",
      status: "passed",
      message: "Write gate is disabled.",
    },
    {
      checkId: "RLP-02",
      label: "plan-declared",
      status: "failed",
      message: "No rollback limitation plan declared.",
      remediation: "Declare a RollbackLimitationPlan.",
    },
  ],
  message: "Rollback limitation policy is blocked. Restore writes remain disabled.",
  noChangesMade: true,
  networkWritesAttempted: false,
  writesEnabled: false,
};

describe("RestoreRollbackLimitationPolicyPanel", () => {
  it("renders the panel container", () => {
    render(
      <RestoreRollbackLimitationPolicyPanel result={null} loading={false} onVerify={() => {}} />,
    );
    expect(screen.getByTestId("restore-rlp-panel")).toBeDefined();
  });

  it("always shows the writes-disabled notice", () => {
    render(
      <RestoreRollbackLimitationPolicyPanel result={null} loading={false} onVerify={() => {}} />,
    );
    expect(screen.getByTestId("rlp-writes-disabled-notice")).toBeDefined();
  });

  it("notice text mentions automatic rollback not available", () => {
    render(
      <RestoreRollbackLimitationPolicyPanel result={null} loading={false} onVerify={() => {}} />,
    );
    const notice = screen.getByTestId("rlp-writes-disabled-notice");
    expect(notice.textContent).toContain("Automatic rollback is not available");
  });

  it("shows verify button", () => {
    render(
      <RestoreRollbackLimitationPolicyPanel result={null} loading={false} onVerify={() => {}} />,
    );
    expect(screen.getByTestId("rlp-verify-button")).toBeDefined();
  });

  it("calls onVerify when button is clicked", () => {
    const onVerify = vi.fn();
    render(
      <RestoreRollbackLimitationPolicyPanel result={null} loading={false} onVerify={onVerify} />,
    );
    fireEvent.click(screen.getByTestId("rlp-verify-button"));
    expect(onVerify).toHaveBeenCalledTimes(1);
  });

  it("disables button when loading", () => {
    render(
      <RestoreRollbackLimitationPolicyPanel result={null} loading={true} onVerify={() => {}} />,
    );
    const btn = screen.getByTestId("rlp-verify-button") as HTMLButtonElement;
    expect(btn.disabled).toBe(true);
  });

  it("shows 'Checking…' text when loading", () => {
    render(
      <RestoreRollbackLimitationPolicyPanel result={null} loading={true} onVerify={() => {}} />,
    );
    expect(screen.getByTestId("rlp-verify-button").textContent).toBe("Checking…");
  });

  it("does not show result when result is null", () => {
    render(
      <RestoreRollbackLimitationPolicyPanel result={null} loading={false} onVerify={() => {}} />,
    );
    expect(screen.queryByTestId("rlp-result")).toBeNull();
  });

  it("shows result panel when result is provided", () => {
    render(
      <RestoreRollbackLimitationPolicyPanel
        result={compliantResult}
        loading={false}
        onVerify={() => {}}
      />,
    );
    expect(screen.getByTestId("rlp-result")).toBeDefined();
  });

  it("shows compliant badge for compliant result", () => {
    render(
      <RestoreRollbackLimitationPolicyPanel
        result={compliantResult}
        loading={false}
        onVerify={() => {}}
      />,
    );
    expect(screen.getByTestId("rlp-compliant-badge")).toBeDefined();
    expect(screen.queryByTestId("rlp-warning-badge")).toBeNull();
    expect(screen.queryByTestId("rlp-blocked-badge")).toBeNull();
  });

  it("shows warning badge for warning result", () => {
    render(
      <RestoreRollbackLimitationPolicyPanel
        result={warningResult}
        loading={false}
        onVerify={() => {}}
      />,
    );
    expect(screen.getByTestId("rlp-warning-badge")).toBeDefined();
    expect(screen.queryByTestId("rlp-compliant-badge")).toBeNull();
  });

  it("shows blocked badge for blocked result", () => {
    render(
      <RestoreRollbackLimitationPolicyPanel
        result={blockedResult}
        loading={false}
        onVerify={() => {}}
      />,
    );
    expect(screen.getByTestId("rlp-blocked-badge")).toBeDefined();
    expect(screen.queryByTestId("rlp-compliant-badge")).toBeNull();
  });

  it("always shows writes-disabled tag", () => {
    render(
      <RestoreRollbackLimitationPolicyPanel
        result={compliantResult}
        loading={false}
        onVerify={() => {}}
      />,
    );
    expect(screen.getByTestId("rlp-writes-disabled-tag")).toBeDefined();
  });

  it("shows result message", () => {
    render(
      <RestoreRollbackLimitationPolicyPanel
        result={compliantResult}
        loading={false}
        onVerify={() => {}}
      />,
    );
    expect(screen.getByTestId("rlp-message").textContent).toContain(
      "Rollback limitation policy is compliant",
    );
  });

  it("shows plan summary when present", () => {
    render(
      <RestoreRollbackLimitationPolicyPanel
        result={compliantResult}
        loading={false}
        onVerify={() => {}}
      />,
    );
    expect(screen.getByTestId("rlp-plan-summary")).toBeDefined();
  });

  it("shows rollback behavior in plan summary", () => {
    render(
      <RestoreRollbackLimitationPolicyPanel
        result={compliantResult}
        loading={false}
        onVerify={() => {}}
      />,
    );
    expect(screen.getByTestId("rlp-summary-rollback-behavior").textContent).toBe(
      "noAutomaticRollback",
    );
  });

  it("shows partialRestoreIsNotSuccess in plan summary", () => {
    render(
      <RestoreRollbackLimitationPolicyPanel
        result={compliantResult}
        loading={false}
        onVerify={() => {}}
      />,
    );
    expect(screen.getByTestId("rlp-summary-partial-not-success").textContent).toBe("Yes");
  });

  it("shows recovery guidance declared in plan summary", () => {
    render(
      <RestoreRollbackLimitationPolicyPanel
        result={compliantResult}
        loading={false}
        onVerify={() => {}}
      />,
    );
    expect(screen.getByTestId("rlp-summary-recovery-guidance").textContent).toBe("Yes");
  });

  it("does not show plan summary when absent", () => {
    render(
      <RestoreRollbackLimitationPolicyPanel
        result={blockedResult}
        loading={false}
        onVerify={() => {}}
      />,
    );
    expect(screen.queryByTestId("rlp-plan-summary")).toBeNull();
  });

  it("shows all 12 checks for compliant result", () => {
    render(
      <RestoreRollbackLimitationPolicyPanel
        result={compliantResult}
        loading={false}
        onVerify={() => {}}
      />,
    );
    expect(screen.getByTestId("rlp-check-rlp-01")).toBeDefined();
    expect(screen.getByTestId("rlp-check-rlp-12")).toBeDefined();
  });

  it("shows remediation for warning check", () => {
    render(
      <RestoreRollbackLimitationPolicyPanel
        result={warningResult}
        loading={false}
        onVerify={() => {}}
      />,
    );
    expect(screen.getByTestId("rlp-remediation-rlp-07")).toBeDefined();
  });

  it("shows remediation for blocked check", () => {
    render(
      <RestoreRollbackLimitationPolicyPanel
        result={blockedResult}
        loading={false}
        onVerify={() => {}}
      />,
    );
    expect(screen.getByTestId("rlp-remediation-rlp-02")).toBeDefined();
  });

  it("shows no-changes-made footer", () => {
    render(
      <RestoreRollbackLimitationPolicyPanel
        result={compliantResult}
        loading={false}
        onVerify={() => {}}
      />,
    );
    expect(screen.getByTestId("rlp-no-changes-made")).toBeDefined();
  });

  it("compliant result does not contain execute button", () => {
    render(
      <RestoreRollbackLimitationPolicyPanel
        result={compliantResult}
        loading={false}
        onVerify={() => {}}
      />,
    );
    const buttons = screen.queryAllByRole("button");
    const buttonLabels = buttons.map((b) => b.textContent?.toLowerCase() ?? "");
    expect(buttonLabels.every((l) => !l.includes("execute"))).toBe(true);
    expect(buttonLabels.every((l) => !l.includes("start restore"))).toBe(true);
  });

  it("compliant result does not contain cleanup/delete/revert button", () => {
    render(
      <RestoreRollbackLimitationPolicyPanel
        result={compliantResult}
        loading={false}
        onVerify={() => {}}
      />,
    );
    const buttons = screen.queryAllByRole("button");
    const buttonLabels = buttons.map((b) => b.textContent?.toLowerCase() ?? "");
    expect(buttonLabels.every((l) => !l.includes("cleanup"))).toBe(true);
    expect(buttonLabels.every((l) => !l.includes("delete all"))).toBe(true);
    expect(buttonLabels.every((l) => !l.includes("revert"))).toBe(true);
  });

  it("result message does not contain success wording for compliant", () => {
    render(
      <RestoreRollbackLimitationPolicyPanel
        result={compliantResult}
        loading={false}
        onVerify={() => {}}
      />,
    );
    const message = screen.getByTestId("rlp-message").textContent ?? "";
    expect(message.toLowerCase()).not.toContain("restore complete");
    expect(message.toLowerCase()).not.toContain("succeeded");
  });

  it("result message for blocked mentions writes remain disabled", () => {
    render(
      <RestoreRollbackLimitationPolicyPanel
        result={blockedResult}
        loading={false}
        onVerify={() => {}}
      />,
    );
    const message = screen.getByTestId("rlp-message").textContent ?? "";
    expect(message.toLowerCase()).toContain("writes remain disabled");
  });
});
