import { describe, it, expect, vi } from "vitest";
import { render, screen, fireEvent } from "@testing-library/react";
import { mockAirBridgeService } from "../services/mockAirBridgeService";
import type { FinalValidationPlan, FinalValidationPolicyRequest } from "../backend/types";
import { RestoreFinalValidationPolicyPanel } from "../features/backups/RestoreFinalValidationPolicyPanel";

// ── Helpers ───────────────────────────────────────────────────────────────────

function safePlan(): FinalValidationPlan {
  return {
    hasSchemaCountValidation: true,
    hasTableFieldValidation: true,
    hasRecordCountValidation: true,
    hasIdMappingValidation: true,
    hasLinkedRecordValidation: true,
    hasAttachmentMetadataValidation: true,
    attachmentValidationMetadataOnly: false,
    hasManifestChecksumValidation: true,
    blocksSuccessWithoutValidation: true,
  };
}

function requestWithPlan(plan: FinalValidationPlan): FinalValidationPolicyRequest {
  return { plan };
}

function requestNoPlan(): FinalValidationPolicyRequest {
  return { plan: undefined };
}

// ── Service contract tests ────────────────────────────────────────────────────

describe("FinalValidationPolicy service contract", () => {
  it("complete plan returns compliant", async () => {
    const result = await mockAirBridgeService.verifyFinalValidationPolicy(
      requestWithPlan(safePlan()),
    );
    expect(result.status).toBe("compliant");
  });

  it("no plan returns blocked", async () => {
    const result = await mockAirBridgeService.verifyFinalValidationPolicy(requestNoPlan());
    expect(result.status).toBe("blocked");
  });

  it("missing schema count validation returns blocked", async () => {
    const plan = { ...safePlan(), hasSchemaCountValidation: false };
    const result = await mockAirBridgeService.verifyFinalValidationPolicy(requestWithPlan(plan));
    expect(result.status).toBe("blocked");
  });

  it("missing table field validation returns blocked", async () => {
    const plan = { ...safePlan(), hasTableFieldValidation: false };
    const result = await mockAirBridgeService.verifyFinalValidationPolicy(requestWithPlan(plan));
    expect(result.status).toBe("blocked");
  });

  it("missing record count validation returns blocked", async () => {
    const plan = { ...safePlan(), hasRecordCountValidation: false };
    const result = await mockAirBridgeService.verifyFinalValidationPolicy(requestWithPlan(plan));
    expect(result.status).toBe("blocked");
  });

  it("missing id mapping validation returns blocked", async () => {
    const plan = { ...safePlan(), hasIdMappingValidation: false };
    const result = await mockAirBridgeService.verifyFinalValidationPolicy(requestWithPlan(plan));
    expect(result.status).toBe("blocked");
  });

  it("missing linked record validation returns blocked", async () => {
    const plan = { ...safePlan(), hasLinkedRecordValidation: false };
    const result = await mockAirBridgeService.verifyFinalValidationPolicy(requestWithPlan(plan));
    expect(result.status).toBe("blocked");
  });

  it("missing attachment metadata validation returns blocked", async () => {
    const plan = { ...safePlan(), hasAttachmentMetadataValidation: false };
    const result = await mockAirBridgeService.verifyFinalValidationPolicy(requestWithPlan(plan));
    expect(result.status).toBe("blocked");
  });

  it("missing manifest checksum validation returns blocked", async () => {
    const plan = { ...safePlan(), hasManifestChecksumValidation: false };
    const result = await mockAirBridgeService.verifyFinalValidationPolicy(requestWithPlan(plan));
    expect(result.status).toBe("blocked");
  });

  it("blocksSuccessWithoutValidation false returns blocked", async () => {
    const plan = { ...safePlan(), blocksSuccessWithoutValidation: false };
    const result = await mockAirBridgeService.verifyFinalValidationPolicy(requestWithPlan(plan));
    expect(result.status).toBe("blocked");
  });

  it("metadata-only attachment validation returns warning", async () => {
    const plan = { ...safePlan(), attachmentValidationMetadataOnly: true };
    const result = await mockAirBridgeService.verifyFinalValidationPolicy(requestWithPlan(plan));
    expect(result.status).toBe("warning");
  });

  it("complete plan produces 12 checks", async () => {
    const result = await mockAirBridgeService.verifyFinalValidationPolicy(
      requestWithPlan(safePlan()),
    );
    expect(result.checks).toHaveLength(12);
  });

  it("no plan produces 2 checks", async () => {
    const result = await mockAirBridgeService.verifyFinalValidationPolicy(requestNoPlan());
    expect(result.checks).toHaveLength(2);
  });

  it("check IDs are FVP-01 through FVP-12", async () => {
    const result = await mockAirBridgeService.verifyFinalValidationPolicy(
      requestWithPlan(safePlan()),
    );
    const ids = result.checks.map((c) => c.checkId);
    expect(ids).toEqual([
      "FVP-01",
      "FVP-02",
      "FVP-03",
      "FVP-04",
      "FVP-05",
      "FVP-06",
      "FVP-07",
      "FVP-08",
      "FVP-09",
      "FVP-10",
      "FVP-11",
      "FVP-12",
    ]);
  });

  it("no plan check IDs are FVP-01 and FVP-02", async () => {
    const result = await mockAirBridgeService.verifyFinalValidationPolicy(requestNoPlan());
    const ids = result.checks.map((c) => c.checkId);
    expect(ids).toEqual(["FVP-01", "FVP-02"]);
  });

  it("FVP-01 always passes with plan", async () => {
    const result = await mockAirBridgeService.verifyFinalValidationPolicy(
      requestWithPlan(safePlan()),
    );
    const fvp01 = result.checks.find((c) => c.checkId === "FVP-01");
    expect(fvp01?.status).toBe("passed");
  });

  it("FVP-01 always passes without plan", async () => {
    const result = await mockAirBridgeService.verifyFinalValidationPolicy(requestNoPlan());
    const fvp01 = result.checks.find((c) => c.checkId === "FVP-01");
    expect(fvp01?.status).toBe("passed");
  });

  it("FVP-12 always passes", async () => {
    const result = await mockAirBridgeService.verifyFinalValidationPolicy(
      requestWithPlan(safePlan()),
    );
    const fvp12 = result.checks.find((c) => c.checkId === "FVP-12");
    expect(fvp12?.status).toBe("passed");
  });

  it("FVP-03 fails when schema count validation missing", async () => {
    const plan = { ...safePlan(), hasSchemaCountValidation: false };
    const result = await mockAirBridgeService.verifyFinalValidationPolicy(requestWithPlan(plan));
    const fvp03 = result.checks.find((c) => c.checkId === "FVP-03");
    expect(fvp03?.status).toBe("failed");
  });

  it("FVP-05 fails when record count validation missing", async () => {
    const plan = { ...safePlan(), hasRecordCountValidation: false };
    const result = await mockAirBridgeService.verifyFinalValidationPolicy(requestWithPlan(plan));
    const fvp05 = result.checks.find((c) => c.checkId === "FVP-05");
    expect(fvp05?.status).toBe("failed");
  });

  it("FVP-06 fails when id mapping validation missing", async () => {
    const plan = { ...safePlan(), hasIdMappingValidation: false };
    const result = await mockAirBridgeService.verifyFinalValidationPolicy(requestWithPlan(plan));
    const fvp06 = result.checks.find((c) => c.checkId === "FVP-06");
    expect(fvp06?.status).toBe("failed");
  });

  it("FVP-09 warns when attachment validation is metadata-only", async () => {
    const plan = { ...safePlan(), attachmentValidationMetadataOnly: true };
    const result = await mockAirBridgeService.verifyFinalValidationPolicy(requestWithPlan(plan));
    const fvp09 = result.checks.find((c) => c.checkId === "FVP-09");
    expect(fvp09?.status).toBe("warning");
  });

  it("FVP-11 fails when blocksSuccessWithoutValidation false", async () => {
    const plan = { ...safePlan(), blocksSuccessWithoutValidation: false };
    const result = await mockAirBridgeService.verifyFinalValidationPolicy(requestWithPlan(plan));
    const fvp11 = result.checks.find((c) => c.checkId === "FVP-11");
    expect(fvp11?.status).toBe("failed");
  });

  it("plan summary present when plan provided", async () => {
    const result = await mockAirBridgeService.verifyFinalValidationPolicy(
      requestWithPlan(safePlan()),
    );
    expect(result.planSummary).toBeDefined();
  });

  it("plan summary absent when no plan", async () => {
    const result = await mockAirBridgeService.verifyFinalValidationPolicy(requestNoPlan());
    expect(result.planSummary).toBeUndefined();
  });

  it("noChangesMade always true — compliant", async () => {
    const result = await mockAirBridgeService.verifyFinalValidationPolicy(
      requestWithPlan(safePlan()),
    );
    expect(result.noChangesMade).toBe(true);
  });

  it("noChangesMade always true — blocked", async () => {
    const result = await mockAirBridgeService.verifyFinalValidationPolicy(requestNoPlan());
    expect(result.noChangesMade).toBe(true);
  });

  it("networkWritesAttempted always false — compliant", async () => {
    const result = await mockAirBridgeService.verifyFinalValidationPolicy(
      requestWithPlan(safePlan()),
    );
    expect(result.networkWritesAttempted).toBe(false);
  });

  it("networkWritesAttempted always false — blocked", async () => {
    const result = await mockAirBridgeService.verifyFinalValidationPolicy(requestNoPlan());
    expect(result.networkWritesAttempted).toBe(false);
  });

  it("writesEnabled always false — compliant", async () => {
    const result = await mockAirBridgeService.verifyFinalValidationPolicy(
      requestWithPlan(safePlan()),
    );
    expect(result.writesEnabled).toBe(false);
  });

  it("writesEnabled always false — blocked", async () => {
    const result = await mockAirBridgeService.verifyFinalValidationPolicy(requestNoPlan());
    expect(result.writesEnabled).toBe(false);
  });

  it("compliant message says writes remain disabled", async () => {
    const result = await mockAirBridgeService.verifyFinalValidationPolicy(
      requestWithPlan(safePlan()),
    );
    expect(result.status).toBe("compliant");
    expect(result.message.toLowerCase()).toContain("disabled");
  });

  it("compliant message does not contain succeeded", async () => {
    const result = await mockAirBridgeService.verifyFinalValidationPolicy(
      requestWithPlan(safePlan()),
    );
    expect(result.message.toLowerCase()).not.toContain("succeeded");
  });

  it("result does not contain token", async () => {
    const result = await mockAirBridgeService.verifyFinalValidationPolicy(
      requestWithPlan(safePlan()),
    );
    const json = JSON.stringify(result);
    expect(json).not.toContain("pat_");
    expect(json).not.toContain('"token"');
  });

  it("result does not contain path", async () => {
    const result = await mockAirBridgeService.verifyFinalValidationPolicy(
      requestWithPlan(safePlan()),
    );
    const json = JSON.stringify(result);
    expect(json).not.toContain("/Users/");
    expect(json).not.toContain("/home/");
  });

  it("result does not contain record payload", async () => {
    const result = await mockAirBridgeService.verifyFinalValidationPolicy(
      requestWithPlan(safePlan()),
    );
    const json = JSON.stringify(result);
    expect(json).not.toContain('"fields"');
    expect(json).not.toContain('"recordId"');
  });
});

// ── UI panel tests ────────────────────────────────────────────────────────────

describe("RestoreFinalValidationPolicyPanel", () => {
  it("renders with writes-disabled notice", () => {
    render(<RestoreFinalValidationPolicyPanel result={null} loading={false} onVerify={() => {}} />);
    expect(screen.getByTestId("fvp-writes-disabled-notice")).toBeDefined();
  });

  it("renders verify button", () => {
    render(<RestoreFinalValidationPolicyPanel result={null} loading={false} onVerify={() => {}} />);
    expect(screen.getByTestId("fvp-verify-button")).toBeDefined();
  });

  it("result section not shown when null", () => {
    render(<RestoreFinalValidationPolicyPanel result={null} loading={false} onVerify={() => {}} />);
    expect(screen.queryByTestId("fvp-result")).toBeNull();
  });

  it("calls onVerify when button clicked", () => {
    const onVerify = vi.fn();
    render(<RestoreFinalValidationPolicyPanel result={null} loading={false} onVerify={onVerify} />);
    fireEvent.click(screen.getByTestId("fvp-verify-button"));
    expect(onVerify).toHaveBeenCalledTimes(1);
  });

  it("disables button when loading", () => {
    render(<RestoreFinalValidationPolicyPanel result={null} loading={true} onVerify={() => {}} />);
    const btn = screen.getByTestId("fvp-verify-button") as HTMLButtonElement;
    expect(btn.disabled).toBe(true);
  });

  it("shows checking text when loading", () => {
    render(<RestoreFinalValidationPolicyPanel result={null} loading={true} onVerify={() => {}} />);
    expect(screen.getByTestId("fvp-verify-button").textContent).toContain("Checking");
  });

  it("shows compliant badge for compliant result", async () => {
    const result = await mockAirBridgeService.verifyFinalValidationPolicy(
      requestWithPlan(safePlan()),
    );
    render(
      <RestoreFinalValidationPolicyPanel result={result} loading={false} onVerify={() => {}} />,
    );
    expect(screen.getByTestId("fvp-compliant-badge")).toBeDefined();
    expect(screen.queryByTestId("fvp-warning-badge")).toBeNull();
    expect(screen.queryByTestId("fvp-blocked-badge")).toBeNull();
  });

  it("shows warning badge for warning result", async () => {
    const plan = { ...safePlan(), attachmentValidationMetadataOnly: true };
    const result = await mockAirBridgeService.verifyFinalValidationPolicy(requestWithPlan(plan));
    render(
      <RestoreFinalValidationPolicyPanel result={result} loading={false} onVerify={() => {}} />,
    );
    expect(screen.getByTestId("fvp-warning-badge")).toBeDefined();
    expect(screen.queryByTestId("fvp-compliant-badge")).toBeNull();
    expect(screen.queryByTestId("fvp-blocked-badge")).toBeNull();
  });

  it("shows blocked badge for blocked result", async () => {
    const result = await mockAirBridgeService.verifyFinalValidationPolicy(requestNoPlan());
    render(
      <RestoreFinalValidationPolicyPanel result={result} loading={false} onVerify={() => {}} />,
    );
    expect(screen.getByTestId("fvp-blocked-badge")).toBeDefined();
    expect(screen.queryByTestId("fvp-compliant-badge")).toBeNull();
    expect(screen.queryByTestId("fvp-warning-badge")).toBeNull();
  });

  it("shows compliant notice for compliant result", async () => {
    const result = await mockAirBridgeService.verifyFinalValidationPolicy(
      requestWithPlan(safePlan()),
    );
    render(
      <RestoreFinalValidationPolicyPanel result={result} loading={false} onVerify={() => {}} />,
    );
    expect(screen.getByTestId("fvp-compliant-notice")).toBeDefined();
  });

  it("shows warning notice for warning result", async () => {
    const plan = { ...safePlan(), attachmentValidationMetadataOnly: true };
    const result = await mockAirBridgeService.verifyFinalValidationPolicy(requestWithPlan(plan));
    render(
      <RestoreFinalValidationPolicyPanel result={result} loading={false} onVerify={() => {}} />,
    );
    expect(screen.getByTestId("fvp-warning-notice")).toBeDefined();
  });

  it("shows blocked notice for blocked result", async () => {
    const result = await mockAirBridgeService.verifyFinalValidationPolicy(requestNoPlan());
    render(
      <RestoreFinalValidationPolicyPanel result={result} loading={false} onVerify={() => {}} />,
    );
    expect(screen.getByTestId("fvp-blocked-notice")).toBeDefined();
  });

  it("renders plan summary with all 9 fields", async () => {
    const result = await mockAirBridgeService.verifyFinalValidationPolicy(
      requestWithPlan(safePlan()),
    );
    render(
      <RestoreFinalValidationPolicyPanel result={result} loading={false} onVerify={() => {}} />,
    );
    expect(screen.getByTestId("fvp-plan-summary")).toBeDefined();
    expect(screen.getByTestId("fvp-schema-count-validation")).toBeDefined();
    expect(screen.getByTestId("fvp-table-field-validation")).toBeDefined();
    expect(screen.getByTestId("fvp-record-count-validation")).toBeDefined();
    expect(screen.getByTestId("fvp-id-mapping-validation")).toBeDefined();
    expect(screen.getByTestId("fvp-linked-record-validation")).toBeDefined();
    expect(screen.getByTestId("fvp-attachment-metadata-validation")).toBeDefined();
    expect(screen.getByTestId("fvp-attachment-metadata-only")).toBeDefined();
    expect(screen.getByTestId("fvp-manifest-checksum-validation")).toBeDefined();
    expect(screen.getByTestId("fvp-blocks-success-without-validation")).toBeDefined();
  });

  it("renders 12 check rows for complete plan", async () => {
    const result = await mockAirBridgeService.verifyFinalValidationPolicy(
      requestWithPlan(safePlan()),
    );
    render(
      <RestoreFinalValidationPolicyPanel result={result} loading={false} onVerify={() => {}} />,
    );
    expect(screen.getAllByTestId("fvp-check-row")).toHaveLength(12);
  });

  it("renders safety summary", async () => {
    const result = await mockAirBridgeService.verifyFinalValidationPolicy(
      requestWithPlan(safePlan()),
    );
    render(
      <RestoreFinalValidationPolicyPanel result={result} loading={false} onVerify={() => {}} />,
    );
    expect(screen.getByTestId("fvp-safety-summary")).toBeDefined();
    expect(screen.getByTestId("fvp-no-changes-notice")).toBeDefined();
  });

  it("no token input rendered", () => {
    render(<RestoreFinalValidationPolicyPanel result={null} loading={false} onVerify={() => {}} />);
    expect(screen.queryByDisplayValue(/pat_/)).toBeNull();
    const inputs = document.querySelectorAll('input[type="password"]');
    expect(inputs).toHaveLength(0);
  });

  it("no execute or start-restore button rendered", async () => {
    const result = await mockAirBridgeService.verifyFinalValidationPolicy(
      requestWithPlan(safePlan()),
    );
    render(
      <RestoreFinalValidationPolicyPanel result={result} loading={false} onVerify={() => {}} />,
    );
    const buttons = screen.getAllByRole("button");
    for (const btn of buttons) {
      const text = btn.textContent?.toLowerCase() ?? "";
      expect(text).not.toContain("execute");
      expect(text).not.toContain("start restore");
      expect(text).not.toContain("run restore");
    }
  });

  it("no succeeded language rendered for compliant result", async () => {
    const result = await mockAirBridgeService.verifyFinalValidationPolicy(
      requestWithPlan(safePlan()),
    );
    render(
      <RestoreFinalValidationPolicyPanel result={result} loading={false} onVerify={() => {}} />,
    );
    const body = document.body.textContent?.toLowerCase() ?? "";
    expect(body).not.toContain("restore succeeded");
    expect(body).not.toContain("restore complete");
  });

  it("writes-disabled notice always visible regardless of result status", async () => {
    const result = await mockAirBridgeService.verifyFinalValidationPolicy(
      requestWithPlan(safePlan()),
    );
    render(
      <RestoreFinalValidationPolicyPanel result={result} loading={false} onVerify={() => {}} />,
    );
    expect(screen.getByTestId("fvp-writes-disabled-notice")).toBeDefined();
  });

  it("compliant notice says writes remain disabled", async () => {
    const result = await mockAirBridgeService.verifyFinalValidationPolicy(
      requestWithPlan(safePlan()),
    );
    render(
      <RestoreFinalValidationPolicyPanel result={result} loading={false} onVerify={() => {}} />,
    );
    const notice = screen.getByTestId("fvp-compliant-notice");
    expect(notice.textContent?.toLowerCase()).toContain("disabled");
  });
});
