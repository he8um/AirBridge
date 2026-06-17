import { describe, it, expect, vi } from "vitest";
import { render, screen, fireEvent, waitFor } from "@testing-library/react";
import { RestoreFinalValidationExecutionPreviewPanel } from "../features/backups/RestoreFinalValidationExecutionPreviewPanel";
import { mockAirBridgeService } from "../services/mockAirBridgeService";
import type {
  FinalValidationExecutionPreviewRequest,
  FinalValidationExecutionPreviewResult,
} from "../backend/types";

function safeRequest(): FinalValidationExecutionPreviewRequest {
  return {
    packageFilename: "test-backup.airbridge",
    schemaWritePreviewReady: true,
    recordWritePreviewReady: true,
    mappingCheckpointPreviewReady: true,
    linkedSecondPassPreviewReady: true,
    finalValidationPolicySafe: true,
    finalValidationEnforcementPolicySafe: true,
    sensitiveDataSafe: true,
    attachmentPhaseDisabledSafe: true,
    liveWriteReadinessSatisfied: true,
    tableCount: 3,
    fieldCount: 12,
    recordCount: 150,
    idMappingEntryCount: 150,
    linkedCoverageCount: 4,
    attachmentMetadataCount: 8,
    manifestPresent: true,
  };
}

function blockedRequest(): FinalValidationExecutionPreviewRequest {
  return {};
}

async function renderPanel(
  request: FinalValidationExecutionPreviewRequest,
  result: FinalValidationExecutionPreviewResult | null = null,
  loading = false,
) {
  const onPreview = vi
    .fn()
    .mockResolvedValue(await mockAirBridgeService.previewFinalValidationExecution(request));
  render(
    <RestoreFinalValidationExecutionPreviewPanel
      request={request}
      onPreview={onPreview}
      result={result}
      loading={loading}
    />,
  );
  return { onPreview };
}

// ── Panel rendering ────────────────────────────────────────────────────────────

describe("RestoreFinalValidationExecutionPreviewPanel", () => {
  it("renders the panel container", async () => {
    await renderPanel(safeRequest());
    expect(screen.getByTestId("restore-fvep-panel")).toBeInTheDocument();
  });

  it("renders the execution-disabled notice", async () => {
    await renderPanel(safeRequest());
    expect(screen.getByTestId("fvep-execution-disabled-notice")).toBeInTheDocument();
  });

  it("renders the preview button", async () => {
    await renderPanel(safeRequest());
    expect(screen.getByTestId("fvep-preview-button")).toBeInTheDocument();
  });

  it("does not render result before preview is run", async () => {
    await renderPanel(safeRequest());
    expect(screen.queryByTestId("fvep-result")).not.toBeInTheDocument();
  });

  it("shows loading state while preview is in progress", async () => {
    await renderPanel(safeRequest(), null, true);
    expect(screen.getByTestId("fvep-preview-button")).toBeDisabled();
  });

  // ── DryRunReady state ─────────────────────────────────────────────────────

  it("shows dry-run badge when result is dryRunReady", async () => {
    const result = await mockAirBridgeService.previewFinalValidationExecution(safeRequest());
    await renderPanel(safeRequest(), result);
    expect(screen.getByTestId("fvep-dry-run-badge")).toBeInTheDocument();
  });

  it("shows execution-disabled tag in result", async () => {
    const result = await mockAirBridgeService.previewFinalValidationExecution(safeRequest());
    await renderPanel(safeRequest(), result);
    expect(screen.getByTestId("fvep-execution-disabled-tag")).toBeInTheDocument();
  });

  it("shows message in result", async () => {
    const result = await mockAirBridgeService.previewFinalValidationExecution(safeRequest());
    await renderPanel(safeRequest(), result);
    expect(screen.getByTestId("fvep-message")).toBeInTheDocument();
  });

  it("message mentions dry-run advisory", async () => {
    const result = await mockAirBridgeService.previewFinalValidationExecution(safeRequest());
    await renderPanel(safeRequest(), result);
    expect(screen.getByTestId("fvep-message").textContent).toMatch(/dry-run/i);
  });

  it("message mentions live execution disabled", async () => {
    const result = await mockAirBridgeService.previewFinalValidationExecution(safeRequest());
    await renderPanel(safeRequest(), result);
    expect(screen.getByTestId("fvep-message").textContent).toMatch(/disabled/i);
  });

  it("no blocked reason shown for dryRunReady result", async () => {
    const result = await mockAirBridgeService.previewFinalValidationExecution(safeRequest());
    await renderPanel(safeRequest(), result);
    expect(screen.queryByTestId("fvep-blocked-reason")).not.toBeInTheDocument();
  });

  it("shows summary in dryRunReady result", async () => {
    const result = await mockAirBridgeService.previewFinalValidationExecution(safeRequest());
    await renderPanel(safeRequest(), result);
    expect(screen.getByTestId("fvep-summary")).toBeInTheDocument();
  });

  it("shows checks list in dryRunReady result", async () => {
    const result = await mockAirBridgeService.previewFinalValidationExecution(safeRequest());
    await renderPanel(safeRequest(), result);
    expect(screen.getByTestId("fvep-checks")).toBeInTheDocument();
  });

  it("shows 8 checks in dryRunReady result with manifest", async () => {
    const result = await mockAirBridgeService.previewFinalValidationExecution(safeRequest());
    await renderPanel(safeRequest(), result);
    expect(result.checks).toHaveLength(8);
  });

  it("shows no-changes-made tag", async () => {
    const result = await mockAirBridgeService.previewFinalValidationExecution(safeRequest());
    await renderPanel(safeRequest(), result);
    expect(screen.getByTestId("fvep-no-changes-made")).toBeInTheDocument();
    expect(screen.getByTestId("fvep-no-changes-made").textContent).toContain("No changes made");
  });

  // ── Blocked state ─────────────────────────────────────────────────────────

  it("shows blocked badge when result is blocked", async () => {
    const result = await mockAirBridgeService.previewFinalValidationExecution(blockedRequest());
    await renderPanel(blockedRequest(), result);
    expect(screen.getByTestId("fvep-blocked-badge")).toBeInTheDocument();
  });

  it("shows blocked reason when blocked", async () => {
    const result = await mockAirBridgeService.previewFinalValidationExecution(blockedRequest());
    await renderPanel(blockedRequest(), result);
    expect(screen.getByTestId("fvep-blocked-reason")).toBeInTheDocument();
  });

  it("does not show dry-run badge when blocked", async () => {
    const result = await mockAirBridgeService.previewFinalValidationExecution(blockedRequest());
    await renderPanel(blockedRequest(), result);
    expect(screen.queryByTestId("fvep-dry-run-badge")).not.toBeInTheDocument();
  });

  // ── Button interaction ────────────────────────────────────────────────────

  it("calls onPreview when preview button is clicked", async () => {
    const { onPreview } = await renderPanel(safeRequest());
    fireEvent.click(screen.getByTestId("fvep-preview-button"));
    await waitFor(() => expect(onPreview).toHaveBeenCalledTimes(1));
  });

  it("passes request to onPreview", async () => {
    const req = safeRequest();
    const { onPreview } = await renderPanel(req);
    fireEvent.click(screen.getByTestId("fvep-preview-button"));
    await waitFor(() => expect(onPreview).toHaveBeenCalledWith(req));
  });

  // ── Safety invariants ─────────────────────────────────────────────────────

  it("writesEnabled is always false in dryRunReady result", async () => {
    const result = await mockAirBridgeService.previewFinalValidationExecution(safeRequest());
    expect(result.writesEnabled).toBe(false);
  });

  it("noChangesMade is always true in dryRunReady result", async () => {
    const result = await mockAirBridgeService.previewFinalValidationExecution(safeRequest());
    expect(result.noChangesMade).toBe(true);
  });

  it("networkWritesAttempted is always false in dryRunReady result", async () => {
    const result = await mockAirBridgeService.previewFinalValidationExecution(safeRequest());
    expect(result.networkWritesAttempted).toBe(false);
  });

  it("writesEnabled is always false in blocked result", async () => {
    const result = await mockAirBridgeService.previewFinalValidationExecution(blockedRequest());
    expect(result.writesEnabled).toBe(false);
  });

  it("noChangesMade is always true in blocked result", async () => {
    const result = await mockAirBridgeService.previewFinalValidationExecution(blockedRequest());
    expect(result.noChangesMade).toBe(true);
  });

  it("networkWritesAttempted is always false in blocked result", async () => {
    const result = await mockAirBridgeService.previewFinalValidationExecution(blockedRequest());
    expect(result.networkWritesAttempted).toBe(false);
  });

  it("writeGateDisabled is always true in safety snapshot (safe)", async () => {
    const result = await mockAirBridgeService.previewFinalValidationExecution(safeRequest());
    expect(result.safetySnapshot.writeGateDisabled).toBe(true);
  });

  it("writeGateDisabled is always true in safety snapshot (blocked)", async () => {
    const result = await mockAirBridgeService.previewFinalValidationExecution(blockedRequest());
    expect(result.safetySnapshot.writeGateDisabled).toBe(true);
  });

  // ── Prerequisite cascade ──────────────────────────────────────────────────

  it("blocked when schemaWritePreviewReady is false", async () => {
    const result = await mockAirBridgeService.previewFinalValidationExecution({
      ...safeRequest(),
      schemaWritePreviewReady: false,
    });
    expect(result.status).toBe("blocked");
    expect(result.blockedReason).toMatch(/FVEP-PRE-02/);
  });

  it("blocked when recordWritePreviewReady is false", async () => {
    const result = await mockAirBridgeService.previewFinalValidationExecution({
      ...safeRequest(),
      recordWritePreviewReady: false,
    });
    expect(result.status).toBe("blocked");
    expect(result.blockedReason).toMatch(/FVEP-PRE-03/);
  });

  it("blocked when mappingCheckpointPreviewReady is false", async () => {
    const result = await mockAirBridgeService.previewFinalValidationExecution({
      ...safeRequest(),
      mappingCheckpointPreviewReady: false,
    });
    expect(result.status).toBe("blocked");
    expect(result.blockedReason).toMatch(/FVEP-PRE-04/);
  });

  it("blocked when linkedSecondPassPreviewReady is false", async () => {
    const result = await mockAirBridgeService.previewFinalValidationExecution({
      ...safeRequest(),
      linkedSecondPassPreviewReady: false,
    });
    expect(result.status).toBe("blocked");
    expect(result.blockedReason).toMatch(/FVEP-PRE-05/);
  });

  it("blocked when finalValidationPolicySafe is false", async () => {
    const result = await mockAirBridgeService.previewFinalValidationExecution({
      ...safeRequest(),
      finalValidationPolicySafe: false,
    });
    expect(result.status).toBe("blocked");
    expect(result.blockedReason).toMatch(/FVEP-PRE-06/);
  });

  it("blocked when finalValidationEnforcementPolicySafe is false", async () => {
    const result = await mockAirBridgeService.previewFinalValidationExecution({
      ...safeRequest(),
      finalValidationEnforcementPolicySafe: false,
    });
    expect(result.status).toBe("blocked");
    expect(result.blockedReason).toMatch(/FVEP-PRE-07/);
  });

  it("blocked when sensitiveDataSafe is false", async () => {
    const result = await mockAirBridgeService.previewFinalValidationExecution({
      ...safeRequest(),
      sensitiveDataSafe: false,
    });
    expect(result.status).toBe("blocked");
    expect(result.blockedReason).toMatch(/FVEP-PRE-08/);
  });

  it("blocked when attachmentPhaseDisabledSafe is false", async () => {
    const result = await mockAirBridgeService.previewFinalValidationExecution({
      ...safeRequest(),
      attachmentPhaseDisabledSafe: false,
    });
    expect(result.status).toBe("blocked");
    expect(result.blockedReason).toMatch(/FVEP-PRE-09/);
  });

  it("blocked when liveWriteReadinessSatisfied is false", async () => {
    const result = await mockAirBridgeService.previewFinalValidationExecution({
      ...safeRequest(),
      liveWriteReadinessSatisfied: false,
    });
    expect(result.status).toBe("blocked");
    expect(result.blockedReason).toMatch(/FVEP-PRE-10/);
  });

  // ── Check ordering ────────────────────────────────────────────────────────

  it("checks have correct IDs in order", async () => {
    const result = await mockAirBridgeService.previewFinalValidationExecution(safeRequest());
    const ids = result.checks.map((c) => c.checkId);
    expect(ids).toEqual([
      "FVEP-CHK-SCHEMA",
      "FVEP-CHK-FIELDS",
      "FVEP-CHK-RECORDS",
      "FVEP-CHK-MAPPING",
      "FVEP-CHK-LINKED",
      "FVEP-CHK-ATTACH",
      "FVEP-CHK-MANIFEST",
      "FVEP-CHK-GUARD",
    ]);
  });

  it("manifest check is skipped when no manifest present", async () => {
    const result = await mockAirBridgeService.previewFinalValidationExecution({
      ...safeRequest(),
      manifestPresent: false,
    });
    const manifestCheck = result.checks.find((c) => c.checkId === "FVEP-CHK-MANIFEST");
    expect(manifestCheck?.status).toBe("skipped");
  });

  it("manifest check is pending when manifest present", async () => {
    const result = await mockAirBridgeService.previewFinalValidationExecution(safeRequest());
    const manifestCheck = result.checks.find((c) => c.checkId === "FVEP-CHK-MANIFEST");
    expect(manifestCheck?.status).toBe("pending");
  });

  it("summary counts reflect request values", async () => {
    const result = await mockAirBridgeService.previewFinalValidationExecution(safeRequest());
    expect(result.summary.tableCount).toBe(3);
    expect(result.summary.fieldCount).toBe(12);
    expect(result.summary.recordCount).toBe(150);
    expect(result.summary.idMappingEntryCount).toBe(150);
    expect(result.summary.linkedCoverageCount).toBe(4);
    expect(result.summary.attachmentMetadataCount).toBe(8);
    expect(result.summary.manifestPresent).toBe(true);
  });
});
