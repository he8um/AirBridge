import { render, screen, fireEvent } from "@testing-library/react";
import { describe, it, expect, vi } from "vitest";
import { RestoreSchemaWriteExecutionPreviewPanel } from "../features/backups/RestoreSchemaWriteExecutionPreviewPanel";
import type { SchemaWriteExecutionPreviewResult } from "../backend/types";
import { mockAirBridgeService } from "../services/mockAirBridgeService";

const blockedResult: SchemaWriteExecutionPreviewResult = {
  status: "blocked",
  mode: "liveBlocked",
  message:
    "Schema write execution preview is blocked. SWEP-PRE-02: Sandbox environment check has not passed. Live schema writes remain disabled.",
  steps: [
    {
      stepIndex: 0,
      stepId: "SWEP-STEP-BLOCKED",
      label: "Preview blocked",
      status: "blocked",
      note: "Safety prerequisites not satisfied. No steps can be previewed.",
    },
  ],
  safetySnapshot: {
    writeGateDisabled: true,
    sandboxFlagPresent: false,
    targetEmptyVerified: false,
    schemaPlanReady: false,
    destructivePolicySafe: false,
    sensitiveDataSafe: false,
    attachmentPhaseDisabled: false,
    finalValidationEnforcementPresent: false,
    liveWriteReadinessSatisfied: false,
  },
  tableStepCount: 0,
  fieldStepCount: 0,
  deferredStepCount: 0,
  manualStepCount: 0,
  totalStepCount: 0,
  blockedReason: "SWEP-PRE-02: Sandbox environment check has not passed.",
  noChangesMade: true,
  networkWritesAttempted: false,
  writesEnabled: false,
};

const dryRunReadyResult: SchemaWriteExecutionPreviewResult = {
  status: "dryRunReady",
  mode: "dryRunOnly",
  message:
    "Schema write execution preview is ready (dry-run only). 2 table(s), 4 direct field(s), 1 deferred field(s), 0 manual action(s) planned. Live schema writes remain disabled. This preview does not start any restore execution.",
  steps: [
    {
      stepIndex: 0,
      stepId: "SWEP-STEP-VAL",
      label: "Validate schema plan inputs",
      status: "pending",
      note: "Validates that the schema plan is complete. No API calls.",
    },
    {
      stepIndex: 1,
      stepId: "SWEP-STEP-TBL-000",
      label: "Create table 1 of 2",
      status: "pending",
      note: "Would call Airtable create-table endpoint. Disabled.",
    },
    {
      stepIndex: 2,
      stepId: "SWEP-STEP-TBL-001",
      label: "Create table 2 of 2",
      status: "pending",
      note: "Would call Airtable create-table endpoint. Disabled.",
    },
    {
      stepIndex: 3,
      stepId: "SWEP-STEP-FLD-DIRECT",
      label: "Create 4 direct field(s)",
      status: "pending",
      note: "Would call Airtable create-field endpoint. Disabled.",
    },
    {
      stepIndex: 4,
      stepId: "SWEP-STEP-FLD-DEFERRED",
      label: "Defer 1 linked field(s) to second pass",
      status: "pending",
      note: "Linked fields deferred. Disabled.",
    },
    {
      stepIndex: 5,
      stepId: "SWEP-STEP-POST",
      label: "Post-schema safety verification",
      status: "pending",
      note: "Would verify schema matches backup plan. Disabled.",
    },
  ],
  safetySnapshot: {
    writeGateDisabled: true,
    sandboxFlagPresent: true,
    targetEmptyVerified: true,
    schemaPlanReady: true,
    destructivePolicySafe: true,
    sensitiveDataSafe: true,
    attachmentPhaseDisabled: true,
    finalValidationEnforcementPresent: true,
    liveWriteReadinessSatisfied: true,
  },
  tableStepCount: 2,
  fieldStepCount: 4,
  deferredStepCount: 1,
  manualStepCount: 0,
  totalStepCount: 6,
  noChangesMade: true,
  networkWritesAttempted: false,
  writesEnabled: false,
};

describe("RestoreSchemaWriteExecutionPreviewPanel", () => {
  // ── Panel container and notices ──────────────────────────────────────────────

  it("renders the panel container", () => {
    render(
      <RestoreSchemaWriteExecutionPreviewPanel
        result={null}
        loading={false}
        onPreview={() => {}}
      />,
    );
    expect(screen.getByTestId("restore-swep-panel")).toBeDefined();
  });

  it("always shows the writes-disabled notice", () => {
    render(
      <RestoreSchemaWriteExecutionPreviewPanel
        result={null}
        loading={false}
        onPreview={() => {}}
      />,
    );
    expect(screen.getByTestId("swep-writes-disabled-notice")).toBeDefined();
  });

  it("writes-disabled notice states live schema writes are disabled", () => {
    render(
      <RestoreSchemaWriteExecutionPreviewPanel
        result={null}
        loading={false}
        onPreview={() => {}}
      />,
    );
    const notice = screen.getByTestId("swep-writes-disabled-notice");
    expect(notice.textContent?.toLowerCase()).toContain("live schema writes are disabled");
  });

  it("writes-disabled notice states does not start any restore execution", () => {
    render(
      <RestoreSchemaWriteExecutionPreviewPanel
        result={null}
        loading={false}
        onPreview={() => {}}
      />,
    );
    const notice = screen.getByTestId("swep-writes-disabled-notice");
    expect(notice.textContent?.toLowerCase()).toContain("does not start any restore execution");
  });

  // ── Button behavior ──────────────────────────────────────────────────────────

  it("shows preview button", () => {
    render(
      <RestoreSchemaWriteExecutionPreviewPanel
        result={null}
        loading={false}
        onPreview={() => {}}
      />,
    );
    expect(screen.getByTestId("swep-preview-button")).toBeDefined();
  });

  it("calls onPreview when button is clicked", () => {
    const onPreview = vi.fn();
    render(
      <RestoreSchemaWriteExecutionPreviewPanel
        result={null}
        loading={false}
        onPreview={onPreview}
      />,
    );
    fireEvent.click(screen.getByTestId("swep-preview-button"));
    expect(onPreview).toHaveBeenCalledTimes(1);
  });

  it("disables button when loading", () => {
    render(
      <RestoreSchemaWriteExecutionPreviewPanel result={null} loading={true} onPreview={() => {}} />,
    );
    const btn = screen.getByTestId("swep-preview-button") as HTMLButtonElement;
    expect(btn.disabled).toBe(true);
  });

  it("shows Loading text when loading", () => {
    render(
      <RestoreSchemaWriteExecutionPreviewPanel result={null} loading={true} onPreview={() => {}} />,
    );
    expect(screen.getByTestId("swep-preview-button").textContent).toContain("Loading");
  });

  it("does not show result when result is null", () => {
    render(
      <RestoreSchemaWriteExecutionPreviewPanel
        result={null}
        loading={false}
        onPreview={() => {}}
      />,
    );
    expect(screen.queryByTestId("swep-result")).toBeNull();
  });

  // ── No execute / enable button ───────────────────────────────────────────────

  it("has no execute or enable-writes button for dry-run-ready result", () => {
    render(
      <RestoreSchemaWriteExecutionPreviewPanel
        result={dryRunReadyResult}
        loading={false}
        onPreview={() => {}}
      />,
    );
    const buttons = screen.queryAllByRole("button");
    const labels = buttons.map((b) => b.textContent?.toLowerCase() ?? "");
    expect(labels.every((l) => !l.includes("execute"))).toBe(true);
    expect(labels.every((l) => !l.includes("enable writes"))).toBe(true);
    expect(labels.every((l) => !l.includes("start restore"))).toBe(true);
  });

  it("has no execute or enable-writes button for blocked result", () => {
    render(
      <RestoreSchemaWriteExecutionPreviewPanel
        result={blockedResult}
        loading={false}
        onPreview={() => {}}
      />,
    );
    const buttons = screen.queryAllByRole("button");
    const labels = buttons.map((b) => b.textContent?.toLowerCase() ?? "");
    expect(labels.every((l) => !l.includes("execute"))).toBe(true);
    expect(labels.every((l) => !l.includes("enable writes"))).toBe(true);
  });

  // ── Result panel — blocked ────────────────────────────────────────────────────

  it("shows blocked badge for blocked result", () => {
    render(
      <RestoreSchemaWriteExecutionPreviewPanel
        result={blockedResult}
        loading={false}
        onPreview={() => {}}
      />,
    );
    expect(screen.getByTestId("swep-blocked-badge")).toBeDefined();
    expect(screen.queryByTestId("swep-dry-run-badge")).toBeNull();
  });

  it("shows blocked reason for blocked result", () => {
    render(
      <RestoreSchemaWriteExecutionPreviewPanel
        result={blockedResult}
        loading={false}
        onPreview={() => {}}
      />,
    );
    expect(screen.getByTestId("swep-blocked-reason")).toBeDefined();
  });

  it("blocked result message states live schema writes remain disabled", () => {
    render(
      <RestoreSchemaWriteExecutionPreviewPanel
        result={blockedResult}
        loading={false}
        onPreview={() => {}}
      />,
    );
    const msg = screen.getByTestId("swep-message").textContent ?? "";
    expect(msg.toLowerCase()).toContain("disabled");
  });

  it("blocked result shows the blocked step", () => {
    render(
      <RestoreSchemaWriteExecutionPreviewPanel
        result={blockedResult}
        loading={false}
        onPreview={() => {}}
      />,
    );
    expect(screen.getByTestId("swep-step-swep-step-blocked")).toBeDefined();
  });

  it("blocked result does not show step counts section", () => {
    render(
      <RestoreSchemaWriteExecutionPreviewPanel
        result={blockedResult}
        loading={false}
        onPreview={() => {}}
      />,
    );
    expect(screen.queryByTestId("swep-step-counts")).toBeNull();
  });

  // ── Result panel — dry-run-ready ──────────────────────────────────────────────

  it("shows dry-run badge for dry-run-ready result", () => {
    render(
      <RestoreSchemaWriteExecutionPreviewPanel
        result={dryRunReadyResult}
        loading={false}
        onPreview={() => {}}
      />,
    );
    expect(screen.getByTestId("swep-dry-run-badge")).toBeDefined();
    expect(screen.queryByTestId("swep-blocked-badge")).toBeNull();
  });

  it("dry-run-ready result shows writes-disabled tag", () => {
    render(
      <RestoreSchemaWriteExecutionPreviewPanel
        result={dryRunReadyResult}
        loading={false}
        onPreview={() => {}}
      />,
    );
    expect(screen.getByTestId("swep-writes-disabled-tag")).toBeDefined();
  });

  it("dry-run-ready result shows step counts section", () => {
    render(
      <RestoreSchemaWriteExecutionPreviewPanel
        result={dryRunReadyResult}
        loading={false}
        onPreview={() => {}}
      />,
    );
    expect(screen.getByTestId("swep-step-counts")).toBeDefined();
  });

  it("dry-run-ready result shows correct table count", () => {
    render(
      <RestoreSchemaWriteExecutionPreviewPanel
        result={dryRunReadyResult}
        loading={false}
        onPreview={() => {}}
      />,
    );
    expect(screen.getByTestId("swep-table-count").textContent).toBe("2");
  });

  it("dry-run-ready result shows correct total step count", () => {
    render(
      <RestoreSchemaWriteExecutionPreviewPanel
        result={dryRunReadyResult}
        loading={false}
        onPreview={() => {}}
      />,
    );
    expect(screen.getByTestId("swep-total-count").textContent).toBe("6");
  });

  // ── Step rendering and ordering ───────────────────────────────────────────────

  it("renders ordered steps list", () => {
    render(
      <RestoreSchemaWriteExecutionPreviewPanel
        result={dryRunReadyResult}
        loading={false}
        onPreview={() => {}}
      />,
    );
    expect(screen.getByTestId("swep-steps")).toBeDefined();
  });

  it("validation step appears first", () => {
    render(
      <RestoreSchemaWriteExecutionPreviewPanel
        result={dryRunReadyResult}
        loading={false}
        onPreview={() => {}}
      />,
    );
    expect(screen.getByTestId("swep-step-swep-step-val")).toBeDefined();
  });

  it("table steps appear before field steps", () => {
    render(
      <RestoreSchemaWriteExecutionPreviewPanel
        result={dryRunReadyResult}
        loading={false}
        onPreview={() => {}}
      />,
    );
    const allSteps = screen.getByTestId("swep-steps");
    const html = allSteps.innerHTML;
    const tblPos = html.indexOf("swep-step-tbl-000");
    const fldPos = html.indexOf("swep-step-fld-direct");
    expect(tblPos).toBeGreaterThan(0);
    expect(fldPos).toBeGreaterThan(tblPos);
  });

  // ── No-changes-made footer ────────────────────────────────────────────────────

  it("shows no-changes-made footer for dry-run-ready result", () => {
    render(
      <RestoreSchemaWriteExecutionPreviewPanel
        result={dryRunReadyResult}
        loading={false}
        onPreview={() => {}}
      />,
    );
    expect(screen.getByTestId("swep-no-changes-made")).toBeDefined();
  });

  it("footer mentions live schema writes disabled", () => {
    render(
      <RestoreSchemaWriteExecutionPreviewPanel
        result={dryRunReadyResult}
        loading={false}
        onPreview={() => {}}
      />,
    );
    const footer = screen.getByTestId("swep-no-changes-made");
    expect(footer.textContent?.toLowerCase()).toContain("disabled");
  });

  // ── Safety: no token / path / payload / raw HTTP / success wording ───────────

  it("panel does not expose token fields", () => {
    render(
      <RestoreSchemaWriteExecutionPreviewPanel
        result={dryRunReadyResult}
        loading={false}
        onPreview={() => {}}
      />,
    );
    const inputs = screen.queryAllByRole("textbox");
    const names = inputs.map((i) => (i as HTMLInputElement).name?.toLowerCase() ?? "");
    expect(names.every((n) => !n.includes("token"))).toBe(true);
    expect(names.every((n) => !n.includes("api_key"))).toBe(true);
  });

  it("panel does not expose attachment URLs", () => {
    render(
      <RestoreSchemaWriteExecutionPreviewPanel
        result={dryRunReadyResult}
        loading={false}
        onPreview={() => {}}
      />,
    );
    const html = document.body.innerHTML;
    expect(html).not.toMatch(/https?:\/\/[^\s"]+attachment/i);
    expect(html).not.toMatch(/cdn\.airtable\.com/i);
  });

  it("dry-run-ready message does not contain restore success wording", () => {
    render(
      <RestoreSchemaWriteExecutionPreviewPanel
        result={dryRunReadyResult}
        loading={false}
        onPreview={() => {}}
      />,
    );
    const msg = screen.getByTestId("swep-message").textContent ?? "";
    expect(msg.toLowerCase()).not.toContain("restore complete");
    expect(msg.toLowerCase()).not.toContain("restore succeeded");
    expect(msg.toLowerCase()).not.toContain("restore success");
  });

  it("dry-run-ready message mentions preview does not start restore execution", () => {
    render(
      <RestoreSchemaWriteExecutionPreviewPanel
        result={dryRunReadyResult}
        loading={false}
        onPreview={() => {}}
      />,
    );
    const msg = screen.getByTestId("swep-message").textContent ?? "";
    expect(msg.toLowerCase()).toContain("does not start any restore execution");
  });

  // ── Mock service: blocked when prerequisites missing ──────────────────────────

  it("mock service returns blocked when prerequisites are all missing", async () => {
    const result = await mockAirBridgeService.previewSchemaWriteExecution({});
    expect(result.status).toBe("blocked");
    expect(result.writesEnabled).toBe(false);
    expect(result.noChangesMade).toBe(true);
    expect(result.networkWritesAttempted).toBe(false);
  });

  it("mock service returns blocked when sandbox flag missing", async () => {
    const result = await mockAirBridgeService.previewSchemaWriteExecution({
      sandboxFlagPresent: false,
      targetEmptyVerified: true,
      schemaPlanReady: true,
      destructivePolicySafe: true,
      sensitiveDataSafe: true,
      attachmentPhaseDisabled: true,
      finalValidationEnforcementPresent: true,
      liveWriteReadinessSatisfied: true,
    });
    expect(result.status).toBe("blocked");
  });

  it("mock service returns dryRunReady for safe plan", async () => {
    const result = await mockAirBridgeService.previewSchemaWriteExecution({
      packageFilename: "test.airbridge",
      sandboxFlagPresent: true,
      targetEmptyVerified: true,
      schemaPlanReady: true,
      tableCount: 2,
      directFieldCount: 3,
      deferredFieldCount: 1,
      manualActionCount: 0,
      destructivePolicySafe: true,
      sensitiveDataSafe: true,
      attachmentPhaseDisabled: true,
      finalValidationEnforcementPresent: true,
      liveWriteReadinessSatisfied: true,
    });
    expect(result.status).toBe("dryRunReady");
    expect(result.writesEnabled).toBe(false);
    expect(result.noChangesMade).toBe(true);
    expect(result.networkWritesAttempted).toBe(false);
    expect(result.tableStepCount).toBe(2);
    expect(result.fieldStepCount).toBe(3);
    expect(result.deferredStepCount).toBe(1);
  });

  it("mock service dry-run-ready message states writes remain disabled", async () => {
    const result = await mockAirBridgeService.previewSchemaWriteExecution({
      sandboxFlagPresent: true,
      targetEmptyVerified: true,
      schemaPlanReady: true,
      destructivePolicySafe: true,
      sensitiveDataSafe: true,
      attachmentPhaseDisabled: true,
      finalValidationEnforcementPresent: true,
      liveWriteReadinessSatisfied: true,
    });
    expect(result.message.toLowerCase()).toContain("disabled");
  });

  it("mock service dry-run-ready result has ordered steps", async () => {
    const result = await mockAirBridgeService.previewSchemaWriteExecution({
      sandboxFlagPresent: true,
      targetEmptyVerified: true,
      schemaPlanReady: true,
      tableCount: 1,
      directFieldCount: 2,
      destructivePolicySafe: true,
      sensitiveDataSafe: true,
      attachmentPhaseDisabled: true,
      finalValidationEnforcementPresent: true,
      liveWriteReadinessSatisfied: true,
    });
    for (let i = 0; i < result.steps.length; i++) {
      expect(result.steps[i].stepIndex).toBe(i);
    }
  });

  it("mock service blocked result has safety snapshot with writeGateDisabled=true", async () => {
    const result = await mockAirBridgeService.previewSchemaWriteExecution({});
    expect(result.safetySnapshot.writeGateDisabled).toBe(true);
  });
});
