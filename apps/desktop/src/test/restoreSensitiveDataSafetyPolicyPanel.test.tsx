import { render, screen, fireEvent } from "@testing-library/react";
import { describe, it, expect, vi } from "vitest";
import { RestoreSensitiveDataSafetyPolicyPanel } from "../features/backups/RestoreSensitiveDataSafetyPolicyPanel";
import type { SensitiveDataSafetyPolicyResult, SensitiveDataSafetySummary } from "../backend/types";

const safeSummary: SensitiveDataSafetySummary = {
  totalRedactionRules: 10,
  surfacesCovered: 10,
  allRulesNamed: true,
  noTokenInResults: true,
  noFullPathInResults: true,
  packageReferencesFilenameOnly: true,
  noRecordPayloadInResults: true,
  noAttachmentUrlInResults: true,
  noRawHttpInResults: true,
  errorMessagesUseSafeSummaries: true,
  summariesArePayloadFree: true,
};

const allPassedChecks: SensitiveDataSafetyPolicyResult["checks"] = [
  {
    checkId: "SDS-01",
    label: "write-gate-disabled",
    status: "passed",
    message: "Write gate is disabled.",
  },
  {
    checkId: "SDS-02",
    label: "safety-plan-declared",
    status: "passed",
    message: "Safety plan is declared.",
  },
  {
    checkId: "SDS-03",
    label: "all-surfaces-covered",
    status: "passed",
    message: "All surfaces covered.",
  },
  {
    checkId: "SDS-04",
    label: "no-token-in-results",
    status: "passed",
    message: "No token in results.",
  },
  {
    checkId: "SDS-05",
    label: "no-full-path-in-results",
    status: "passed",
    message: "No full path in results.",
  },
  {
    checkId: "SDS-06",
    label: "package-references-filename-only",
    status: "passed",
    message: "Package references filename only.",
  },
  {
    checkId: "SDS-07",
    label: "no-record-payload-in-results",
    status: "passed",
    message: "No record payload in results.",
  },
  {
    checkId: "SDS-08",
    label: "no-attachment-url-in-results",
    status: "passed",
    message: "No attachment URL in results.",
  },
  {
    checkId: "SDS-09",
    label: "no-raw-http-in-results",
    status: "passed",
    message: "No raw HTTP in results.",
  },
  {
    checkId: "SDS-10",
    label: "error-messages-safe-summaries",
    status: "passed",
    message: "Error messages use safe summaries.",
  },
  {
    checkId: "SDS-11",
    label: "summaries-payload-free",
    status: "passed",
    message: "Summaries are payload-free.",
  },
  {
    checkId: "SDS-12",
    label: "redaction-rules-named",
    status: "passed",
    message: "All redaction rules are named.",
  },
  {
    checkId: "SDS-13",
    label: "no-success-state",
    status: "passed",
    message: "No success state introduced.",
  },
  {
    checkId: "SDS-14",
    label: "no-token-path-payload-in-result",
    status: "passed",
    message: "No token, path, or payload in result.",
  },
  {
    checkId: "SDS-15",
    label: "writes-remain-disabled",
    status: "passed",
    message: "Writes remain disabled.",
  },
];

const compliantResult: SensitiveDataSafetyPolicyResult = {
  status: "compliant",
  checks: allPassedChecks,
  message:
    "Sensitive data safety policy is compliant. All exposure surfaces have redaction coverage. Restore writes remain disabled.",
  safetySummary: safeSummary,
  noChangesMade: true,
  networkWritesAttempted: false,
  writesEnabled: false,
};

const warningResult: SensitiveDataSafetyPolicyResult = {
  status: "warning",
  checks: [
    ...allPassedChecks.slice(0, 11),
    {
      checkId: "SDS-12",
      label: "redaction-rules-named",
      status: "warning",
      message: "1 redaction rule(s) have no named rule.",
      remediation: "Provide a named redaction rule string for every entry.",
    },
    ...allPassedChecks.slice(12),
  ],
  message: "Sensitive data safety policy has warnings. Restore writes remain disabled.",
  safetySummary: { ...safeSummary, allRulesNamed: false },
  noChangesMade: true,
  networkWritesAttempted: false,
  writesEnabled: false,
};

const blockedResult: SensitiveDataSafetyPolicyResult = {
  status: "blocked",
  checks: [
    {
      checkId: "SDS-01",
      label: "write-gate-disabled",
      status: "passed",
      message: "Write gate is disabled.",
    },
    {
      checkId: "SDS-02",
      label: "safety-plan-declared",
      status: "failed",
      message: "No sensitive data safety plan declared.",
      remediation: "Declare a SensitiveDataSafetyPlan.",
    },
  ],
  message:
    "Sensitive data safety policy is blocked. Sensitive material must never be exposed. Restore writes remain disabled.",
  noChangesMade: true,
  networkWritesAttempted: false,
  writesEnabled: false,
};

describe("RestoreSensitiveDataSafetyPolicyPanel", () => {
  it("renders the panel container", () => {
    render(
      <RestoreSensitiveDataSafetyPolicyPanel result={null} loading={false} onVerify={() => {}} />,
    );
    expect(screen.getByTestId("restore-sds-panel")).toBeDefined();
  });

  it("always shows the writes-disabled notice", () => {
    render(
      <RestoreSensitiveDataSafetyPolicyPanel result={null} loading={false} onVerify={() => {}} />,
    );
    expect(screen.getByTestId("sds-writes-disabled-notice")).toBeDefined();
  });

  it("notice text mentions sensitive material must not be exposed", () => {
    render(
      <RestoreSensitiveDataSafetyPolicyPanel result={null} loading={false} onVerify={() => {}} />,
    );
    const notice = screen.getByTestId("sds-writes-disabled-notice");
    expect(notice.textContent).toContain("Sensitive material must never be exposed");
  });

  it("shows verify button", () => {
    render(
      <RestoreSensitiveDataSafetyPolicyPanel result={null} loading={false} onVerify={() => {}} />,
    );
    expect(screen.getByTestId("sds-verify-button")).toBeDefined();
  });

  it("calls onVerify when button is clicked", () => {
    const onVerify = vi.fn();
    render(
      <RestoreSensitiveDataSafetyPolicyPanel result={null} loading={false} onVerify={onVerify} />,
    );
    fireEvent.click(screen.getByTestId("sds-verify-button"));
    expect(onVerify).toHaveBeenCalledTimes(1);
  });

  it("disables button when loading", () => {
    render(
      <RestoreSensitiveDataSafetyPolicyPanel result={null} loading={true} onVerify={() => {}} />,
    );
    const btn = screen.getByTestId("sds-verify-button") as HTMLButtonElement;
    expect(btn.disabled).toBe(true);
  });

  it("shows 'Checking…' text when loading", () => {
    render(
      <RestoreSensitiveDataSafetyPolicyPanel result={null} loading={true} onVerify={() => {}} />,
    );
    expect(screen.getByTestId("sds-verify-button").textContent).toBe("Checking…");
  });

  it("does not show result when result is null", () => {
    render(
      <RestoreSensitiveDataSafetyPolicyPanel result={null} loading={false} onVerify={() => {}} />,
    );
    expect(screen.queryByTestId("sds-result")).toBeNull();
  });

  it("shows result panel when result is provided", () => {
    render(
      <RestoreSensitiveDataSafetyPolicyPanel
        result={compliantResult}
        loading={false}
        onVerify={() => {}}
      />,
    );
    expect(screen.getByTestId("sds-result")).toBeDefined();
  });

  it("shows compliant badge for compliant result", () => {
    render(
      <RestoreSensitiveDataSafetyPolicyPanel
        result={compliantResult}
        loading={false}
        onVerify={() => {}}
      />,
    );
    expect(screen.getByTestId("sds-compliant-badge")).toBeDefined();
    expect(screen.queryByTestId("sds-warning-badge")).toBeNull();
    expect(screen.queryByTestId("sds-blocked-badge")).toBeNull();
  });

  it("shows warning badge for warning result", () => {
    render(
      <RestoreSensitiveDataSafetyPolicyPanel
        result={warningResult}
        loading={false}
        onVerify={() => {}}
      />,
    );
    expect(screen.getByTestId("sds-warning-badge")).toBeDefined();
    expect(screen.queryByTestId("sds-compliant-badge")).toBeNull();
  });

  it("shows blocked badge for blocked result", () => {
    render(
      <RestoreSensitiveDataSafetyPolicyPanel
        result={blockedResult}
        loading={false}
        onVerify={() => {}}
      />,
    );
    expect(screen.getByTestId("sds-blocked-badge")).toBeDefined();
    expect(screen.queryByTestId("sds-compliant-badge")).toBeNull();
  });

  it("always shows writes-disabled tag", () => {
    render(
      <RestoreSensitiveDataSafetyPolicyPanel
        result={compliantResult}
        loading={false}
        onVerify={() => {}}
      />,
    );
    expect(screen.getByTestId("sds-writes-disabled-tag")).toBeDefined();
  });

  it("shows result message", () => {
    render(
      <RestoreSensitiveDataSafetyPolicyPanel
        result={compliantResult}
        loading={false}
        onVerify={() => {}}
      />,
    );
    expect(screen.getByTestId("sds-message").textContent).toContain(
      "Sensitive data safety policy is compliant",
    );
  });

  it("shows safety summary when present", () => {
    render(
      <RestoreSensitiveDataSafetyPolicyPanel
        result={compliantResult}
        loading={false}
        onVerify={() => {}}
      />,
    );
    expect(screen.getByTestId("sds-safety-summary")).toBeDefined();
  });

  it("shows surfaces covered count in safety summary", () => {
    render(
      <RestoreSensitiveDataSafetyPolicyPanel
        result={compliantResult}
        loading={false}
        onVerify={() => {}}
      />,
    );
    expect(screen.getByTestId("sds-summary-surfaces-covered").textContent).toContain("10");
  });

  it("shows all rules named in safety summary", () => {
    render(
      <RestoreSensitiveDataSafetyPolicyPanel
        result={compliantResult}
        loading={false}
        onVerify={() => {}}
      />,
    );
    expect(screen.getByTestId("sds-summary-all-named").textContent).toBe("Yes");
  });

  it("shows no-token flag in safety summary", () => {
    render(
      <RestoreSensitiveDataSafetyPolicyPanel
        result={compliantResult}
        loading={false}
        onVerify={() => {}}
      />,
    );
    expect(screen.getByTestId("sds-summary-no-token").textContent).toBe("Yes");
  });

  it("does not show safety summary when absent", () => {
    render(
      <RestoreSensitiveDataSafetyPolicyPanel
        result={blockedResult}
        loading={false}
        onVerify={() => {}}
      />,
    );
    expect(screen.queryByTestId("sds-safety-summary")).toBeNull();
  });

  it("shows all 15 checks for compliant result", () => {
    render(
      <RestoreSensitiveDataSafetyPolicyPanel
        result={compliantResult}
        loading={false}
        onVerify={() => {}}
      />,
    );
    expect(screen.getByTestId("sds-check-sds-01")).toBeDefined();
    expect(screen.getByTestId("sds-check-sds-15")).toBeDefined();
  });

  it("shows remediation for blocked check", () => {
    render(
      <RestoreSensitiveDataSafetyPolicyPanel
        result={blockedResult}
        loading={false}
        onVerify={() => {}}
      />,
    );
    expect(screen.getByTestId("sds-remediation-sds-02")).toBeDefined();
  });

  it("shows no-changes-made footer", () => {
    render(
      <RestoreSensitiveDataSafetyPolicyPanel
        result={compliantResult}
        loading={false}
        onVerify={() => {}}
      />,
    );
    expect(screen.getByTestId("sds-no-changes-made")).toBeDefined();
  });

  it("compliant result does not contain execute button", () => {
    render(
      <RestoreSensitiveDataSafetyPolicyPanel
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

  it("result message does not contain success wording for compliant", () => {
    render(
      <RestoreSensitiveDataSafetyPolicyPanel
        result={compliantResult}
        loading={false}
        onVerify={() => {}}
      />,
    );
    const message = screen.getByTestId("sds-message").textContent ?? "";
    expect(message.toLowerCase()).not.toContain("restore complete");
    expect(message.toLowerCase()).not.toContain("succeeded");
  });

  it("result message for blocked mentions writes remain disabled", () => {
    render(
      <RestoreSensitiveDataSafetyPolicyPanel
        result={blockedResult}
        loading={false}
        onVerify={() => {}}
      />,
    );
    const message = screen.getByTestId("sds-message").textContent ?? "";
    expect(message.toLowerCase()).toContain("writes remain disabled");
  });

  it("result message for blocked mentions sensitive material must not be exposed", () => {
    render(
      <RestoreSensitiveDataSafetyPolicyPanel
        result={blockedResult}
        loading={false}
        onVerify={() => {}}
      />,
    );
    const message = screen.getByTestId("sds-message").textContent ?? "";
    expect(message.toLowerCase()).toContain("sensitive material");
  });

  it("warning result shows warning check SDS-12", () => {
    render(
      <RestoreSensitiveDataSafetyPolicyPanel
        result={warningResult}
        loading={false}
        onVerify={() => {}}
      />,
    );
    const checkEl = screen.getByTestId("sds-check-sds-12");
    expect(checkEl.textContent).toContain("warning");
  });

  it("safety summary shows all-named false for warning result", () => {
    render(
      <RestoreSensitiveDataSafetyPolicyPanel
        result={warningResult}
        loading={false}
        onVerify={() => {}}
      />,
    );
    expect(screen.getByTestId("sds-summary-all-named").textContent).toBe("No");
  });
});
