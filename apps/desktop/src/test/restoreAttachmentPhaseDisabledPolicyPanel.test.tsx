import { render, screen, fireEvent } from "@testing-library/react";
import { describe, it, expect, vi } from "vitest";
import { RestoreAttachmentPhaseDisabledPolicyPanel } from "../features/backups/RestoreAttachmentPhaseDisabledPolicyPanel";
import type {
  AttachmentPhaseDisabledPolicyResult,
  AttachmentPhaseDisabledSummary,
} from "../backend/types";

const fullSummary: AttachmentPhaseDisabledSummary = {
  metadataInspectionEnabled: true,
  metadataVerificationEnabled: true,
  binaryHandlingDisabled: true,
  urlExposureDisabled: true,
  fieldMutationDisabled: true,
  phaseRequiredForCompletionDisabled: true,
  finalValidationTreatsAsMetadataOnly: true,
  blockedOperationsDeclared: 0,
};

const allPassedChecks: AttachmentPhaseDisabledPolicyResult["checks"] = [
  {
    checkId: "APD-01",
    label: "write-gate-disabled",
    status: "passed",
    message: "Write gate is disabled.",
  },
  {
    checkId: "APD-02",
    label: "plan-declared",
    status: "passed",
    message: "Attachment metadata plan is declared.",
  },
  {
    checkId: "APD-03",
    label: "metadata-inspection-flag",
    status: "passed",
    message: "Metadata inspection enabled flag is set.",
  },
  {
    checkId: "APD-04",
    label: "metadata-verification-flag",
    status: "passed",
    message: "Metadata verification enabled.",
  },
  {
    checkId: "APD-05",
    label: "binary-download-blocked",
    status: "passed",
    message: "Binary download is disabled.",
  },
  {
    checkId: "APD-06",
    label: "binary-upload-blocked",
    status: "passed",
    message: "Binary upload is disabled.",
  },
  {
    checkId: "APD-07",
    label: "url-fetch-blocked",
    status: "passed",
    message: "URL fetch is disabled.",
  },
  {
    checkId: "APD-08",
    label: "file-read-blocked",
    status: "passed",
    message: "File read is disabled.",
  },
  {
    checkId: "APD-09",
    label: "file-write-blocked",
    status: "passed",
    message: "File write is disabled.",
  },
  {
    checkId: "APD-10",
    label: "raw-attachment-transfer-blocked",
    status: "passed",
    message: "Raw attachment transfer is disabled.",
  },
  {
    checkId: "APD-11",
    label: "field-mutation-blocked",
    status: "passed",
    message: "Attachment field mutation is disabled.",
  },
  {
    checkId: "APD-12",
    label: "url-exposure-blocked",
    status: "passed",
    message: "Attachment URL exposure is disabled.",
  },
  {
    checkId: "APD-13",
    label: "phase-not-required-for-completion",
    status: "passed",
    message: "Phase is not required for completion.",
  },
  {
    checkId: "APD-14",
    label: "final-validation-metadata-only",
    status: "passed",
    message: "Final validation treats attachments as metadata only.",
  },
  {
    checkId: "APD-15",
    label: "no-binary-operations-declared",
    status: "passed",
    message: "No binary attachment operations are declared.",
  },
  {
    checkId: "APD-16",
    label: "no-blocked-ops-required",
    status: "passed",
    message: "No blocked operations are required for completion.",
  },
];

const compliantResult: AttachmentPhaseDisabledPolicyResult = {
  status: "compliant",
  checks: allPassedChecks,
  message:
    "Attachment phase disabled policy is compliant. Binary attachment operations are blocked. Restore writes remain disabled.",
  phaseSummary: fullSummary,
  noChangesMade: true,
  networkWritesAttempted: false,
  writesEnabled: false,
};

const warningResult: AttachmentPhaseDisabledPolicyResult = {
  status: "warning",
  checks: [
    ...allPassedChecks.slice(0, 3),
    {
      checkId: "APD-04",
      label: "metadata-verification-flag",
      status: "warning",
      message: "Metadata verification is disabled but a skip reason was provided.",
      remediation: "Consider enabling metadata verification when possible.",
    },
    ...allPassedChecks.slice(4),
  ],
  message: "Attachment phase disabled policy has warnings. Restore writes remain disabled.",
  phaseSummary: { ...fullSummary, metadataVerificationEnabled: false },
  noChangesMade: true,
  networkWritesAttempted: false,
  writesEnabled: false,
};

const blockedResult: AttachmentPhaseDisabledPolicyResult = {
  status: "blocked",
  checks: [
    {
      checkId: "APD-01",
      label: "write-gate-disabled",
      status: "passed",
      message: "Write gate is disabled.",
    },
    {
      checkId: "APD-02",
      label: "plan-declared",
      status: "failed",
      message: "No attachment metadata plan declared.",
      remediation: "Declare an AttachmentMetadataOnlyPlan.",
    },
  ],
  message:
    "Attachment phase disabled policy is blocked. Binary attachment operations must remain disabled. Restore writes remain disabled.",
  noChangesMade: true,
  networkWritesAttempted: false,
  writesEnabled: false,
};

describe("RestoreAttachmentPhaseDisabledPolicyPanel", () => {
  it("renders the panel container", () => {
    render(
      <RestoreAttachmentPhaseDisabledPolicyPanel
        result={null}
        loading={false}
        onVerify={() => {}}
      />,
    );
    expect(screen.getByTestId("restore-apd-panel")).toBeDefined();
  });

  it("always shows the writes-disabled notice", () => {
    render(
      <RestoreAttachmentPhaseDisabledPolicyPanel
        result={null}
        loading={false}
        onVerify={() => {}}
      />,
    );
    expect(screen.getByTestId("apd-writes-disabled-notice")).toBeDefined();
  });

  it("writes-disabled notice text mentions binary attachment operations", () => {
    render(
      <RestoreAttachmentPhaseDisabledPolicyPanel
        result={null}
        loading={false}
        onVerify={() => {}}
      />,
    );
    const notice = screen.getByTestId("apd-writes-disabled-notice");
    expect(notice.textContent).toContain("Binary attachment");
  });

  it("always shows the metadata-only notice", () => {
    render(
      <RestoreAttachmentPhaseDisabledPolicyPanel
        result={null}
        loading={false}
        onVerify={() => {}}
      />,
    );
    expect(screen.getByTestId("apd-metadata-only-notice")).toBeDefined();
  });

  it("metadata-only notice text mentions metadata-only", () => {
    render(
      <RestoreAttachmentPhaseDisabledPolicyPanel
        result={null}
        loading={false}
        onVerify={() => {}}
      />,
    );
    const notice = screen.getByTestId("apd-metadata-only-notice");
    expect(notice.textContent?.toLowerCase()).toContain("metadata-only");
  });

  it("shows verify button", () => {
    render(
      <RestoreAttachmentPhaseDisabledPolicyPanel
        result={null}
        loading={false}
        onVerify={() => {}}
      />,
    );
    expect(screen.getByTestId("apd-verify-button")).toBeDefined();
  });

  it("calls onVerify when button is clicked", () => {
    const onVerify = vi.fn();
    render(
      <RestoreAttachmentPhaseDisabledPolicyPanel
        result={null}
        loading={false}
        onVerify={onVerify}
      />,
    );
    fireEvent.click(screen.getByTestId("apd-verify-button"));
    expect(onVerify).toHaveBeenCalledTimes(1);
  });

  it("disables button when loading", () => {
    render(
      <RestoreAttachmentPhaseDisabledPolicyPanel
        result={null}
        loading={true}
        onVerify={() => {}}
      />,
    );
    const btn = screen.getByTestId("apd-verify-button") as HTMLButtonElement;
    expect(btn.disabled).toBe(true);
  });

  it("shows 'Checking…' text when loading", () => {
    render(
      <RestoreAttachmentPhaseDisabledPolicyPanel
        result={null}
        loading={true}
        onVerify={() => {}}
      />,
    );
    expect(screen.getByTestId("apd-verify-button").textContent).toBe("Checking…");
  });

  it("does not show result when result is null", () => {
    render(
      <RestoreAttachmentPhaseDisabledPolicyPanel
        result={null}
        loading={false}
        onVerify={() => {}}
      />,
    );
    expect(screen.queryByTestId("apd-result")).toBeNull();
  });

  it("shows result panel when result is provided", () => {
    render(
      <RestoreAttachmentPhaseDisabledPolicyPanel
        result={compliantResult}
        loading={false}
        onVerify={() => {}}
      />,
    );
    expect(screen.getByTestId("apd-result")).toBeDefined();
  });

  it("shows compliant badge for compliant result", () => {
    render(
      <RestoreAttachmentPhaseDisabledPolicyPanel
        result={compliantResult}
        loading={false}
        onVerify={() => {}}
      />,
    );
    expect(screen.getByTestId("apd-compliant-badge")).toBeDefined();
    expect(screen.queryByTestId("apd-warning-badge")).toBeNull();
    expect(screen.queryByTestId("apd-blocked-badge")).toBeNull();
  });

  it("shows warning badge for warning result", () => {
    render(
      <RestoreAttachmentPhaseDisabledPolicyPanel
        result={warningResult}
        loading={false}
        onVerify={() => {}}
      />,
    );
    expect(screen.getByTestId("apd-warning-badge")).toBeDefined();
    expect(screen.queryByTestId("apd-compliant-badge")).toBeNull();
  });

  it("shows blocked badge for blocked result", () => {
    render(
      <RestoreAttachmentPhaseDisabledPolicyPanel
        result={blockedResult}
        loading={false}
        onVerify={() => {}}
      />,
    );
    expect(screen.getByTestId("apd-blocked-badge")).toBeDefined();
    expect(screen.queryByTestId("apd-compliant-badge")).toBeNull();
  });

  it("always shows writes-disabled tag when result is present", () => {
    render(
      <RestoreAttachmentPhaseDisabledPolicyPanel
        result={compliantResult}
        loading={false}
        onVerify={() => {}}
      />,
    );
    expect(screen.getByTestId("apd-writes-disabled-tag")).toBeDefined();
  });

  it("always shows metadata-only tag when result is present", () => {
    render(
      <RestoreAttachmentPhaseDisabledPolicyPanel
        result={compliantResult}
        loading={false}
        onVerify={() => {}}
      />,
    );
    expect(screen.getByTestId("apd-metadata-only-tag")).toBeDefined();
  });

  it("shows result message", () => {
    render(
      <RestoreAttachmentPhaseDisabledPolicyPanel
        result={compliantResult}
        loading={false}
        onVerify={() => {}}
      />,
    );
    expect(screen.getByTestId("apd-message").textContent).toContain(
      "Attachment phase disabled policy is compliant",
    );
  });

  it("shows phase summary when present", () => {
    render(
      <RestoreAttachmentPhaseDisabledPolicyPanel
        result={compliantResult}
        loading={false}
        onVerify={() => {}}
      />,
    );
    expect(screen.getByTestId("apd-phase-summary")).toBeDefined();
  });

  it("does not show phase summary when absent", () => {
    render(
      <RestoreAttachmentPhaseDisabledPolicyPanel
        result={blockedResult}
        loading={false}
        onVerify={() => {}}
      />,
    );
    expect(screen.queryByTestId("apd-phase-summary")).toBeNull();
  });

  it("shows binary-handling-disabled as Yes in summary", () => {
    render(
      <RestoreAttachmentPhaseDisabledPolicyPanel
        result={compliantResult}
        loading={false}
        onVerify={() => {}}
      />,
    );
    expect(screen.getByTestId("apd-summary-binary-disabled").textContent).toBe("Yes");
  });

  it("shows blocked-operations count in summary", () => {
    render(
      <RestoreAttachmentPhaseDisabledPolicyPanel
        result={compliantResult}
        loading={false}
        onVerify={() => {}}
      />,
    );
    expect(screen.getByTestId("apd-summary-blocked-operations").textContent).toBe("0");
  });

  it("shows metadata-verification as No in summary for warning result", () => {
    render(
      <RestoreAttachmentPhaseDisabledPolicyPanel
        result={warningResult}
        loading={false}
        onVerify={() => {}}
      />,
    );
    expect(screen.getByTestId("apd-summary-metadata-verification").textContent).toBe("No");
  });

  it("shows the operation table", () => {
    render(
      <RestoreAttachmentPhaseDisabledPolicyPanel
        result={compliantResult}
        loading={false}
        onVerify={() => {}}
      />,
    );
    expect(screen.getByTestId("apd-operation-table")).toBeDefined();
  });

  it("shows binaryDownload as blocked in operation table", () => {
    render(
      <RestoreAttachmentPhaseDisabledPolicyPanel
        result={compliantResult}
        loading={false}
        onVerify={() => {}}
      />,
    );
    const row = screen.getByTestId("apd-op-binary-download");
    expect(row.textContent).toContain("blocked");
  });

  it("shows metadataInspect as permitted in operation table", () => {
    render(
      <RestoreAttachmentPhaseDisabledPolicyPanel
        result={compliantResult}
        loading={false}
        onVerify={() => {}}
      />,
    );
    const row = screen.getByTestId("apd-op-metadata-inspect");
    expect(row.textContent).toContain("permitted");
  });

  it("shows all 16 checks for compliant result", () => {
    render(
      <RestoreAttachmentPhaseDisabledPolicyPanel
        result={compliantResult}
        loading={false}
        onVerify={() => {}}
      />,
    );
    expect(screen.getByTestId("apd-check-apd-01")).toBeDefined();
    expect(screen.getByTestId("apd-check-apd-16")).toBeDefined();
  });

  it("shows remediation for blocked check", () => {
    render(
      <RestoreAttachmentPhaseDisabledPolicyPanel
        result={blockedResult}
        loading={false}
        onVerify={() => {}}
      />,
    );
    expect(screen.getByTestId("apd-remediation-apd-02")).toBeDefined();
  });

  it("shows no-changes-made footer", () => {
    render(
      <RestoreAttachmentPhaseDisabledPolicyPanel
        result={compliantResult}
        loading={false}
        onVerify={() => {}}
      />,
    );
    expect(screen.getByTestId("apd-no-changes-made")).toBeDefined();
  });

  it("footer text mentions no changes made", () => {
    render(
      <RestoreAttachmentPhaseDisabledPolicyPanel
        result={compliantResult}
        loading={false}
        onVerify={() => {}}
      />,
    );
    const footer = screen.getByTestId("apd-no-changes-made");
    expect(footer.textContent?.toLowerCase()).toContain("no changes made");
  });

  it("compliant result does not contain execute button", () => {
    render(
      <RestoreAttachmentPhaseDisabledPolicyPanel
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
      <RestoreAttachmentPhaseDisabledPolicyPanel
        result={compliantResult}
        loading={false}
        onVerify={() => {}}
      />,
    );
    const message = screen.getByTestId("apd-message").textContent ?? "";
    expect(message.toLowerCase()).not.toContain("restore complete");
    expect(message.toLowerCase()).not.toContain("succeeded");
  });

  it("result message for blocked mentions writes remain disabled", () => {
    render(
      <RestoreAttachmentPhaseDisabledPolicyPanel
        result={blockedResult}
        loading={false}
        onVerify={() => {}}
      />,
    );
    const message = screen.getByTestId("apd-message").textContent ?? "";
    expect(message.toLowerCase()).toContain("writes remain disabled");
  });

  it("result message for blocked mentions binary attachment operations", () => {
    render(
      <RestoreAttachmentPhaseDisabledPolicyPanel
        result={blockedResult}
        loading={false}
        onVerify={() => {}}
      />,
    );
    const message = screen.getByTestId("apd-message").textContent ?? "";
    expect(message.toLowerCase()).toContain("binary attachment");
  });

  it("warning result shows warning check APD-04", () => {
    render(
      <RestoreAttachmentPhaseDisabledPolicyPanel
        result={warningResult}
        loading={false}
        onVerify={() => {}}
      />,
    );
    const checkEl = screen.getByTestId("apd-check-apd-04");
    expect(checkEl.textContent).toContain("warning");
  });

  it("panel does not expose attachment URLs", () => {
    render(
      <RestoreAttachmentPhaseDisabledPolicyPanel
        result={compliantResult}
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
      <RestoreAttachmentPhaseDisabledPolicyPanel
        result={compliantResult}
        loading={false}
        onVerify={() => {}}
      />,
    );
    const inputs = screen.queryAllByRole("textbox");
    const inputLabels = inputs.map((i) => (i as HTMLInputElement).name?.toLowerCase() ?? "");
    expect(inputLabels.every((l) => !l.includes("token"))).toBe(true);
    expect(inputLabels.every((l) => !l.includes("api_key"))).toBe(true);
  });
});
