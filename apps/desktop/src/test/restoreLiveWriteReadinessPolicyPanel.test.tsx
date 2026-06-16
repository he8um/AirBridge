import { render, screen, fireEvent } from "@testing-library/react";
import { describe, it, expect, vi } from "vitest";
import { RestoreLiveWriteReadinessPolicyPanel } from "../features/backups/RestoreLiveWriteReadinessPolicyPanel";
import type { LiveWriteReadinessPolicyResult, LiveWriteReadinessSummary } from "../backend/types";

const REQUIRED_GATE_IDS = [
  "sandboxEnvironment",
  "restoreConfirmation",
  "targetEmpty",
  "destructiveOperationPolicy",
  "attachmentUploadPolicy",
  "schemaRecordOrder",
  "sandboxWriteTesting",
  "liveWriteConfirmation",
  "rateLimitBackoff",
  "checkpointDurability",
  "finalValidationPlan",
  "writePhaseOrdering",
  "failureModes",
  "rollbackLimitation",
  "finalValidationEnforcement",
  "sensitiveDataSafety",
  "attachmentPhaseDisabled",
];

const allPassedSummary: LiveWriteReadinessSummary = {
  totalGates: 17,
  passedGates: 17,
  warningGates: 0,
  failedGates: 0,
  notEvaluatedGates: 0,
  missingRequiredGates: 0,
  allRequiredGatesDeclared: true,
  liveExecutionAvailable: false,
};

const allPassedChecks: LiveWriteReadinessPolicyResult["checks"] = REQUIRED_GATE_IDS.map((_, i) => ({
  checkId: `LWR-0${i + 1}`,
  label: `check-${i + 1}`,
  status: "passed" as const,
  message: `Check ${i + 1} passed.`,
})).slice(0, 10);

const readyResult: LiveWriteReadinessPolicyResult = {
  status: "ready",
  checks: allPassedChecks,
  message:
    "Live-write readiness policy is satisfied. All 17 required safety gates are declared and none are failed. This result is advisory only — restore writes remain disabled, and a Ready status does not enable any restore execution.",
  gateSummary: allPassedSummary,
  noChangesMade: true,
  networkWritesAttempted: false,
  writesEnabled: false,
};

const warningResult: LiveWriteReadinessPolicyResult = {
  status: "warning",
  checks: [
    ...allPassedChecks.slice(0, 3),
    {
      checkId: "LWR-04",
      label: "warnings-summarized",
      status: "warning",
      message:
        "1 required gate(s) have a warning status: sandboxEnvironment. Writes remain disabled.",
      remediation: "Review warning gates before live write implementation.",
    },
    ...allPassedChecks.slice(4),
  ],
  message:
    "Live-write readiness policy has warnings. All required gates are declared and none are failed, but at least one gate has a warning. This result is advisory only — restore writes remain disabled.",
  gateSummary: { ...allPassedSummary, passedGates: 16, warningGates: 1 },
  noChangesMade: true,
  networkWritesAttempted: false,
  writesEnabled: false,
};

const blockedResult: LiveWriteReadinessPolicyResult = {
  status: "blocked",
  checks: [
    {
      checkId: "LWR-01",
      label: "write-gate-disabled",
      status: "passed",
      message: "Write gate is disabled. No restore writes are attempted by this policy check.",
    },
    {
      checkId: "LWR-02",
      label: "all-required-gates-declared",
      status: "failed",
      message: "No gate statuses were provided. All 17 required safety gates must be declared.",
      remediation: "Provide a gates array containing the status of every required safety gate.",
    },
  ],
  message:
    "Live-write readiness policy is blocked. No gates were declared. Restore writes remain disabled.",
  noChangesMade: true,
  networkWritesAttempted: false,
  writesEnabled: false,
};

describe("RestoreLiveWriteReadinessPolicyPanel", () => {
  it("renders the panel container", () => {
    render(
      <RestoreLiveWriteReadinessPolicyPanel result={null} loading={false} onVerify={() => {}} />,
    );
    expect(screen.getByTestId("restore-lwr-panel")).toBeDefined();
  });

  it("always shows the writes-disabled notice", () => {
    render(
      <RestoreLiveWriteReadinessPolicyPanel result={null} loading={false} onVerify={() => {}} />,
    );
    expect(screen.getByTestId("lwr-writes-disabled-notice")).toBeDefined();
  });

  it("writes-disabled notice mentions restore execution is not started", () => {
    render(
      <RestoreLiveWriteReadinessPolicyPanel result={null} loading={false} onVerify={() => {}} />,
    );
    const notice = screen.getByTestId("lwr-writes-disabled-notice");
    expect(notice.textContent?.toLowerCase()).toContain("does not enable writes");
  });

  it("always shows the advisory-only notice", () => {
    render(
      <RestoreLiveWriteReadinessPolicyPanel result={null} loading={false} onVerify={() => {}} />,
    );
    expect(screen.getByTestId("lwr-advisory-only-notice")).toBeDefined();
  });

  it("advisory-only notice mentions advisory", () => {
    render(
      <RestoreLiveWriteReadinessPolicyPanel result={null} loading={false} onVerify={() => {}} />,
    );
    const notice = screen.getByTestId("lwr-advisory-only-notice");
    expect(notice.textContent?.toLowerCase()).toContain("advisory");
  });

  it("advisory-only notice mentions restore completion remains unavailable", () => {
    render(
      <RestoreLiveWriteReadinessPolicyPanel result={null} loading={false} onVerify={() => {}} />,
    );
    const notice = screen.getByTestId("lwr-advisory-only-notice");
    expect(notice.textContent?.toLowerCase()).toContain("restore completion remains unavailable");
  });

  it("shows verify button", () => {
    render(
      <RestoreLiveWriteReadinessPolicyPanel result={null} loading={false} onVerify={() => {}} />,
    );
    expect(screen.getByTestId("lwr-verify-button")).toBeDefined();
  });

  it("calls onVerify when button is clicked", () => {
    const onVerify = vi.fn();
    render(
      <RestoreLiveWriteReadinessPolicyPanel result={null} loading={false} onVerify={onVerify} />,
    );
    fireEvent.click(screen.getByTestId("lwr-verify-button"));
    expect(onVerify).toHaveBeenCalledTimes(1);
  });

  it("disables button when loading", () => {
    render(
      <RestoreLiveWriteReadinessPolicyPanel result={null} loading={true} onVerify={() => {}} />,
    );
    const btn = screen.getByTestId("lwr-verify-button") as HTMLButtonElement;
    expect(btn.disabled).toBe(true);
  });

  it("shows 'Checking…' text when loading", () => {
    render(
      <RestoreLiveWriteReadinessPolicyPanel result={null} loading={true} onVerify={() => {}} />,
    );
    expect(screen.getByTestId("lwr-verify-button").textContent).toBe("Checking…");
  });

  it("does not show result when result is null", () => {
    render(
      <RestoreLiveWriteReadinessPolicyPanel result={null} loading={false} onVerify={() => {}} />,
    );
    expect(screen.queryByTestId("lwr-result")).toBeNull();
  });

  it("shows result panel when result is provided", () => {
    render(
      <RestoreLiveWriteReadinessPolicyPanel
        result={readyResult}
        loading={false}
        onVerify={() => {}}
      />,
    );
    expect(screen.getByTestId("lwr-result")).toBeDefined();
  });

  it("shows ready badge with advisory label for ready result", () => {
    render(
      <RestoreLiveWriteReadinessPolicyPanel
        result={readyResult}
        loading={false}
        onVerify={() => {}}
      />,
    );
    expect(screen.getByTestId("lwr-ready-badge")).toBeDefined();
    expect(screen.getByTestId("lwr-ready-badge").textContent?.toLowerCase()).toContain("advisory");
    expect(screen.queryByTestId("lwr-warning-badge")).toBeNull();
    expect(screen.queryByTestId("lwr-blocked-badge")).toBeNull();
  });

  it("shows warning badge for warning result", () => {
    render(
      <RestoreLiveWriteReadinessPolicyPanel
        result={warningResult}
        loading={false}
        onVerify={() => {}}
      />,
    );
    expect(screen.getByTestId("lwr-warning-badge")).toBeDefined();
    expect(screen.queryByTestId("lwr-ready-badge")).toBeNull();
  });

  it("shows blocked badge for blocked result", () => {
    render(
      <RestoreLiveWriteReadinessPolicyPanel
        result={blockedResult}
        loading={false}
        onVerify={() => {}}
      />,
    );
    expect(screen.getByTestId("lwr-blocked-badge")).toBeDefined();
    expect(screen.queryByTestId("lwr-ready-badge")).toBeNull();
  });

  it("always shows writes-disabled tag when result is present", () => {
    render(
      <RestoreLiveWriteReadinessPolicyPanel
        result={readyResult}
        loading={false}
        onVerify={() => {}}
      />,
    );
    expect(screen.getByTestId("lwr-writes-disabled-tag")).toBeDefined();
  });

  it("always shows advisory tag when result is present", () => {
    render(
      <RestoreLiveWriteReadinessPolicyPanel
        result={readyResult}
        loading={false}
        onVerify={() => {}}
      />,
    );
    expect(screen.getByTestId("lwr-advisory-tag")).toBeDefined();
  });

  it("shows result message", () => {
    render(
      <RestoreLiveWriteReadinessPolicyPanel
        result={readyResult}
        loading={false}
        onVerify={() => {}}
      />,
    );
    expect(screen.getByTestId("lwr-message").textContent).toContain(
      "Live-write readiness policy is satisfied",
    );
  });

  it("shows gate summary when present", () => {
    render(
      <RestoreLiveWriteReadinessPolicyPanel
        result={readyResult}
        loading={false}
        onVerify={() => {}}
      />,
    );
    expect(screen.getByTestId("lwr-gate-summary")).toBeDefined();
  });

  it("does not show gate summary when absent", () => {
    render(
      <RestoreLiveWriteReadinessPolicyPanel
        result={blockedResult}
        loading={false}
        onVerify={() => {}}
      />,
    );
    expect(screen.queryByTestId("lwr-gate-summary")).toBeNull();
  });

  it("gate summary shows total gates as 17", () => {
    render(
      <RestoreLiveWriteReadinessPolicyPanel
        result={readyResult}
        loading={false}
        onVerify={() => {}}
      />,
    );
    expect(screen.getByTestId("lwr-summary-total").textContent).toBe("17");
  });

  it("gate summary shows all-required-declared as Yes", () => {
    render(
      <RestoreLiveWriteReadinessPolicyPanel
        result={readyResult}
        loading={false}
        onVerify={() => {}}
      />,
    );
    expect(screen.getByTestId("lwr-summary-all-declared").textContent).toBe("Yes");
  });

  it("gate summary shows live-execution-available as No", () => {
    render(
      <RestoreLiveWriteReadinessPolicyPanel
        result={readyResult}
        loading={false}
        onVerify={() => {}}
      />,
    );
    expect(screen.getByTestId("lwr-summary-live-execution").textContent).toBe("No");
  });

  it("gate summary shows warning count for warning result", () => {
    render(
      <RestoreLiveWriteReadinessPolicyPanel
        result={warningResult}
        loading={false}
        onVerify={() => {}}
      />,
    );
    expect(screen.getByTestId("lwr-summary-warning").textContent).toBe("1");
  });

  it("shows checks list", () => {
    render(
      <RestoreLiveWriteReadinessPolicyPanel
        result={readyResult}
        loading={false}
        onVerify={() => {}}
      />,
    );
    expect(screen.getByTestId("lwr-checks")).toBeDefined();
  });

  it("shows LWR-01 check", () => {
    render(
      <RestoreLiveWriteReadinessPolicyPanel
        result={readyResult}
        loading={false}
        onVerify={() => {}}
      />,
    );
    expect(screen.getByTestId("lwr-check-lwr-01")).toBeDefined();
  });

  it("shows remediation for blocked check", () => {
    render(
      <RestoreLiveWriteReadinessPolicyPanel
        result={blockedResult}
        loading={false}
        onVerify={() => {}}
      />,
    );
    expect(screen.getByTestId("lwr-remediation-lwr-02")).toBeDefined();
  });

  it("shows no-changes-made footer", () => {
    render(
      <RestoreLiveWriteReadinessPolicyPanel
        result={readyResult}
        loading={false}
        onVerify={() => {}}
      />,
    );
    expect(screen.getByTestId("lwr-no-changes-made")).toBeDefined();
  });

  it("footer mentions advisory only", () => {
    render(
      <RestoreLiveWriteReadinessPolicyPanel
        result={readyResult}
        loading={false}
        onVerify={() => {}}
      />,
    );
    const footer = screen.getByTestId("lwr-no-changes-made");
    expect(footer.textContent?.toLowerCase()).toContain("advisory only");
  });

  it("ready result does not contain execute or enable button", () => {
    render(
      <RestoreLiveWriteReadinessPolicyPanel
        result={readyResult}
        loading={false}
        onVerify={() => {}}
      />,
    );
    const buttons = screen.queryAllByRole("button");
    const buttonLabels = buttons.map((b) => b.textContent?.toLowerCase() ?? "");
    expect(buttonLabels.every((l) => !l.includes("execute"))).toBe(true);
    expect(buttonLabels.every((l) => !l.includes("start restore"))).toBe(true);
    expect(buttonLabels.every((l) => !l.includes("enable writes"))).toBe(true);
  });

  it("ready result message does not contain restore success wording", () => {
    render(
      <RestoreLiveWriteReadinessPolicyPanel
        result={readyResult}
        loading={false}
        onVerify={() => {}}
      />,
    );
    const message = screen.getByTestId("lwr-message").textContent ?? "";
    expect(message.toLowerCase()).not.toContain("restore complete");
    expect(message.toLowerCase()).not.toContain("succeeded");
  });

  it("ready result message says writes remain disabled", () => {
    render(
      <RestoreLiveWriteReadinessPolicyPanel
        result={readyResult}
        loading={false}
        onVerify={() => {}}
      />,
    );
    const message = screen.getByTestId("lwr-message").textContent ?? "";
    expect(message.toLowerCase()).toContain("writes remain disabled");
  });

  it("ready result message says advisory only", () => {
    render(
      <RestoreLiveWriteReadinessPolicyPanel
        result={readyResult}
        loading={false}
        onVerify={() => {}}
      />,
    );
    const message = screen.getByTestId("lwr-message").textContent ?? "";
    expect(message.toLowerCase()).toContain("advisory only");
  });

  it("blocked result message says writes remain disabled", () => {
    render(
      <RestoreLiveWriteReadinessPolicyPanel
        result={blockedResult}
        loading={false}
        onVerify={() => {}}
      />,
    );
    const message = screen.getByTestId("lwr-message").textContent ?? "";
    expect(message.toLowerCase()).toContain("writes remain disabled");
  });

  it("warning result shows warning check LWR-04", () => {
    render(
      <RestoreLiveWriteReadinessPolicyPanel
        result={warningResult}
        loading={false}
        onVerify={() => {}}
      />,
    );
    const checkEl = screen.getByTestId("lwr-check-lwr-04");
    expect(checkEl.textContent).toContain("warning");
  });

  it("ready result shows gate table with 17 gates note", () => {
    render(
      <RestoreLiveWriteReadinessPolicyPanel
        result={readyResult}
        loading={false}
        onVerify={() => {}}
      />,
    );
    expect(screen.getByTestId("lwr-gate-table")).toBeDefined();
    const table = screen.getByTestId("lwr-gate-table");
    expect(table.textContent).toContain("17");
  });

  it("panel does not expose attachment URLs", () => {
    render(
      <RestoreLiveWriteReadinessPolicyPanel
        result={readyResult}
        loading={false}
        onVerify={() => {}}
      />,
    );
    const html = document.body.innerHTML;
    expect(html).not.toMatch(/https?:\/\/[^\s"]+attachment/i);
    expect(html).not.toMatch(/cdn\.airtable\.com/i);
  });

  it("panel does not expose token fields", () => {
    render(
      <RestoreLiveWriteReadinessPolicyPanel
        result={readyResult}
        loading={false}
        onVerify={() => {}}
      />,
    );
    const inputs = screen.queryAllByRole("textbox");
    const names = inputs.map((i) => (i as HTMLInputElement).name?.toLowerCase() ?? "");
    expect(names.every((n) => !n.includes("token"))).toBe(true);
    expect(names.every((n) => !n.includes("api_key"))).toBe(true);
  });
});
