import { render, screen, fireEvent } from "@testing-library/react";
import { describe, it, expect, vi } from "vitest";
import { RestoreFinalValidationEnforcementPolicyPanel } from "../features/backups/RestoreFinalValidationEnforcementPolicyPanel";
import type {
  FinalValidationEnforcementPolicyResult,
  FinalValidationEnforcementSummary,
} from "../backend/types";

const safeEnforcementSummary: FinalValidationEnforcementSummary = {
  schemaValidationState: "passed",
  recordCountValidationState: "passed",
  idMappingValidationState: "passed",
  linkedRecordValidationState: "passed",
  attachmentMetadataValidationState: "passed",
  attachmentValidationMetadataOnly: false,
  manifestChecksumValidationState: "passed",
  packageManifestPresent: true,
  completionGuardDeclared: true,
  blocksCompletionWithoutFinalValidation: true,
  failedValidationBlocksCompletion: true,
};

const allPassedChecks: FinalValidationEnforcementPolicyResult["checks"] = [
  {
    checkId: "FVE-01",
    label: "write-gate-disabled",
    status: "passed",
    message: "Write gate is disabled.",
  },
  { checkId: "FVE-02", label: "plan-declared", status: "passed", message: "Plan is declared." },
  {
    checkId: "FVE-03",
    label: "completion-guard-declared",
    status: "passed",
    message: "Completion guard is fully declared.",
  },
  {
    checkId: "FVE-04",
    label: "schema-validation-passed",
    status: "passed",
    message: "Schema validation passed.",
  },
  {
    checkId: "FVE-05",
    label: "record-count-validation-passed",
    status: "passed",
    message: "Record count validation passed.",
  },
  {
    checkId: "FVE-06",
    label: "id-mapping-validation-before-linked",
    status: "passed",
    message: "ID mapping passed.",
  },
  {
    checkId: "FVE-07",
    label: "linked-record-validation-passed",
    status: "passed",
    message: "Linked record validation passed.",
  },
  {
    checkId: "FVE-08",
    label: "attachment-validation-explicit",
    status: "passed",
    message: "Attachment validation passed.",
  },
  {
    checkId: "FVE-09",
    label: "manifest-validation-if-present",
    status: "passed",
    message: "Manifest validation passed.",
  },
  {
    checkId: "FVE-10",
    label: "no-partial-as-completion",
    status: "passed",
    message: "Partial validation cannot be treated as completion.",
  },
  {
    checkId: "FVE-11",
    label: "failed-validation-blocks",
    status: "passed",
    message: "Failed validation blocks completion.",
  },
  {
    checkId: "FVE-12",
    label: "no-unsafe-skip",
    status: "passed",
    message: "No validation steps are skipped.",
  },
  {
    checkId: "FVE-13",
    label: "no-success-without-validation",
    status: "passed",
    message: "No success state introduced.",
  },
  {
    checkId: "FVE-14",
    label: "no-token-path-payload",
    status: "passed",
    message: "No token, path, or payload.",
  },
  {
    checkId: "FVE-15",
    label: "writes-remain-disabled",
    status: "passed",
    message: "Writes remain disabled.",
  },
];

const compliantResult: FinalValidationEnforcementPolicyResult = {
  status: "compliant",
  checks: allPassedChecks,
  message:
    "Final validation enforcement policy is compliant. All required validation steps have explicitly passed. Restore writes remain disabled.",
  enforcementSummary: safeEnforcementSummary,
  noChangesMade: true,
  networkWritesAttempted: false,
  writesEnabled: false,
};

const warningResult: FinalValidationEnforcementPolicyResult = {
  status: "warning",
  checks: [
    ...allPassedChecks.slice(0, 7),
    {
      checkId: "FVE-08",
      label: "attachment-validation-explicit",
      status: "warning",
      message: "Attachment validation is metadata-only.",
    },
    ...allPassedChecks.slice(8),
  ],
  message: "Final validation enforcement policy has warnings. Restore writes remain disabled.",
  enforcementSummary: { ...safeEnforcementSummary, attachmentValidationMetadataOnly: true },
  noChangesMade: true,
  networkWritesAttempted: false,
  writesEnabled: false,
};

const blockedResult: FinalValidationEnforcementPolicyResult = {
  status: "blocked",
  checks: [
    {
      checkId: "FVE-01",
      label: "write-gate-disabled",
      status: "passed",
      message: "Write gate is disabled.",
    },
    {
      checkId: "FVE-02",
      label: "plan-declared",
      status: "failed",
      message: "No final validation enforcement plan declared.",
      remediation: "Declare a FinalValidationEnforcementPlan.",
    },
  ],
  message:
    "Final validation enforcement policy is blocked. No result may be labeled complete or successful without final validation explicitly passing. Restore writes remain disabled.",
  noChangesMade: true,
  networkWritesAttempted: false,
  writesEnabled: false,
};

describe("RestoreFinalValidationEnforcementPolicyPanel", () => {
  it("renders the panel container", () => {
    render(
      <RestoreFinalValidationEnforcementPolicyPanel
        result={null}
        loading={false}
        onVerify={() => {}}
      />,
    );
    expect(screen.getByTestId("restore-fve-panel")).toBeDefined();
  });

  it("always shows the writes-disabled notice", () => {
    render(
      <RestoreFinalValidationEnforcementPolicyPanel
        result={null}
        loading={false}
        onVerify={() => {}}
      />,
    );
    expect(screen.getByTestId("fve-writes-disabled-notice")).toBeDefined();
  });

  it("notice text mentions no result labeled complete without final validation", () => {
    render(
      <RestoreFinalValidationEnforcementPolicyPanel
        result={null}
        loading={false}
        onVerify={() => {}}
      />,
    );
    const notice = screen.getByTestId("fve-writes-disabled-notice");
    expect(notice.textContent).toContain("complete or successful without final validation");
  });

  it("shows verify button", () => {
    render(
      <RestoreFinalValidationEnforcementPolicyPanel
        result={null}
        loading={false}
        onVerify={() => {}}
      />,
    );
    expect(screen.getByTestId("fve-verify-button")).toBeDefined();
  });

  it("calls onVerify when button is clicked", () => {
    const onVerify = vi.fn();
    render(
      <RestoreFinalValidationEnforcementPolicyPanel
        result={null}
        loading={false}
        onVerify={onVerify}
      />,
    );
    fireEvent.click(screen.getByTestId("fve-verify-button"));
    expect(onVerify).toHaveBeenCalledTimes(1);
  });

  it("disables button when loading", () => {
    render(
      <RestoreFinalValidationEnforcementPolicyPanel
        result={null}
        loading={true}
        onVerify={() => {}}
      />,
    );
    const btn = screen.getByTestId("fve-verify-button") as HTMLButtonElement;
    expect(btn.disabled).toBe(true);
  });

  it("shows 'Checking…' text when loading", () => {
    render(
      <RestoreFinalValidationEnforcementPolicyPanel
        result={null}
        loading={true}
        onVerify={() => {}}
      />,
    );
    expect(screen.getByTestId("fve-verify-button").textContent).toBe("Checking…");
  });

  it("does not show result when result is null", () => {
    render(
      <RestoreFinalValidationEnforcementPolicyPanel
        result={null}
        loading={false}
        onVerify={() => {}}
      />,
    );
    expect(screen.queryByTestId("fve-result")).toBeNull();
  });

  it("shows result panel when result is provided", () => {
    render(
      <RestoreFinalValidationEnforcementPolicyPanel
        result={compliantResult}
        loading={false}
        onVerify={() => {}}
      />,
    );
    expect(screen.getByTestId("fve-result")).toBeDefined();
  });

  it("shows compliant badge for compliant result", () => {
    render(
      <RestoreFinalValidationEnforcementPolicyPanel
        result={compliantResult}
        loading={false}
        onVerify={() => {}}
      />,
    );
    expect(screen.getByTestId("fve-compliant-badge")).toBeDefined();
    expect(screen.queryByTestId("fve-warning-badge")).toBeNull();
    expect(screen.queryByTestId("fve-blocked-badge")).toBeNull();
  });

  it("shows warning badge for warning result", () => {
    render(
      <RestoreFinalValidationEnforcementPolicyPanel
        result={warningResult}
        loading={false}
        onVerify={() => {}}
      />,
    );
    expect(screen.getByTestId("fve-warning-badge")).toBeDefined();
    expect(screen.queryByTestId("fve-compliant-badge")).toBeNull();
  });

  it("shows blocked badge for blocked result", () => {
    render(
      <RestoreFinalValidationEnforcementPolicyPanel
        result={blockedResult}
        loading={false}
        onVerify={() => {}}
      />,
    );
    expect(screen.getByTestId("fve-blocked-badge")).toBeDefined();
    expect(screen.queryByTestId("fve-compliant-badge")).toBeNull();
  });

  it("always shows writes-disabled tag", () => {
    render(
      <RestoreFinalValidationEnforcementPolicyPanel
        result={compliantResult}
        loading={false}
        onVerify={() => {}}
      />,
    );
    expect(screen.getByTestId("fve-writes-disabled-tag")).toBeDefined();
  });

  it("shows result message", () => {
    render(
      <RestoreFinalValidationEnforcementPolicyPanel
        result={compliantResult}
        loading={false}
        onVerify={() => {}}
      />,
    );
    expect(screen.getByTestId("fve-message").textContent).toContain(
      "Final validation enforcement policy is compliant",
    );
  });

  it("shows enforcement summary when present", () => {
    render(
      <RestoreFinalValidationEnforcementPolicyPanel
        result={compliantResult}
        loading={false}
        onVerify={() => {}}
      />,
    );
    expect(screen.getByTestId("fve-enforcement-summary")).toBeDefined();
  });

  it("shows schema validation state in enforcement summary", () => {
    render(
      <RestoreFinalValidationEnforcementPolicyPanel
        result={compliantResult}
        loading={false}
        onVerify={() => {}}
      />,
    );
    expect(screen.getByTestId("fve-summary-schema-state").textContent).toBe("passed");
  });

  it("shows completion guard declared in enforcement summary", () => {
    render(
      <RestoreFinalValidationEnforcementPolicyPanel
        result={compliantResult}
        loading={false}
        onVerify={() => {}}
      />,
    );
    expect(screen.getByTestId("fve-summary-guard-declared").textContent).toBe("Yes");
  });

  it("does not show enforcement summary when absent", () => {
    render(
      <RestoreFinalValidationEnforcementPolicyPanel
        result={blockedResult}
        loading={false}
        onVerify={() => {}}
      />,
    );
    expect(screen.queryByTestId("fve-enforcement-summary")).toBeNull();
  });

  it("shows all 15 checks for compliant result", () => {
    render(
      <RestoreFinalValidationEnforcementPolicyPanel
        result={compliantResult}
        loading={false}
        onVerify={() => {}}
      />,
    );
    expect(screen.getByTestId("fve-check-fve-01")).toBeDefined();
    expect(screen.getByTestId("fve-check-fve-15")).toBeDefined();
  });

  it("shows remediation for blocked check", () => {
    render(
      <RestoreFinalValidationEnforcementPolicyPanel
        result={blockedResult}
        loading={false}
        onVerify={() => {}}
      />,
    );
    expect(screen.getByTestId("fve-remediation-fve-02")).toBeDefined();
  });

  it("shows no-changes-made footer", () => {
    render(
      <RestoreFinalValidationEnforcementPolicyPanel
        result={compliantResult}
        loading={false}
        onVerify={() => {}}
      />,
    );
    expect(screen.getByTestId("fve-no-changes-made")).toBeDefined();
  });

  it("compliant result does not contain execute button", () => {
    render(
      <RestoreFinalValidationEnforcementPolicyPanel
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
      <RestoreFinalValidationEnforcementPolicyPanel
        result={compliantResult}
        loading={false}
        onVerify={() => {}}
      />,
    );
    const message = screen.getByTestId("fve-message").textContent ?? "";
    expect(message.toLowerCase()).not.toContain("restore complete");
    expect(message.toLowerCase()).not.toContain("succeeded");
  });

  it("result message for blocked mentions no result labeled complete without validation", () => {
    render(
      <RestoreFinalValidationEnforcementPolicyPanel
        result={blockedResult}
        loading={false}
        onVerify={() => {}}
      />,
    );
    const message = screen.getByTestId("fve-message").textContent ?? "";
    expect(message.toLowerCase()).toContain("no result may be labeled complete");
  });

  it("result message for blocked mentions writes remain disabled", () => {
    render(
      <RestoreFinalValidationEnforcementPolicyPanel
        result={blockedResult}
        loading={false}
        onVerify={() => {}}
      />,
    );
    const message = screen.getByTestId("fve-message").textContent ?? "";
    expect(message.toLowerCase()).toContain("writes remain disabled");
  });
});
