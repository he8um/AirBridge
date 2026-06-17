import { describe, it, expect, vi } from "vitest";
import { render, screen, fireEvent, waitFor } from "@testing-library/react";
import { RestoreMappingCheckpointExecutionPreviewPanel } from "../features/backups/RestoreMappingCheckpointExecutionPreviewPanel";
import { mockAirBridgeService } from "../services/mockAirBridgeService";
import type {
  MappingCheckpointExecutionPreviewRequest,
  MappingCheckpointExecutionPreviewResult,
} from "../backend/types";

function safeRequest(): MappingCheckpointExecutionPreviewRequest {
  return {
    packageFilename: "test-backup.airbridge",
    recordWritePreviewReady: true,
    firstPassBatchCount: 4,
    secondPassBatchCount: 2,
    totalRecordCount: 35,
    tablesRequiringRemapping: 2,
    checkpointDurabilitySafe: true,
    failureModesSafe: true,
    rollbackLimitationSafe: true,
    finalValidationEnforcementPresent: true,
    sensitiveDataSafe: true,
    liveWriteReadinessSatisfied: true,
  };
}

function blockedRequest(): MappingCheckpointExecutionPreviewRequest {
  return {};
}

async function renderPanel(
  request: MappingCheckpointExecutionPreviewRequest,
  result: MappingCheckpointExecutionPreviewResult | null = null,
  loading = false,
) {
  const onPreview = vi
    .fn()
    .mockResolvedValue(await mockAirBridgeService.previewMappingCheckpointExecution(request));
  render(
    <RestoreMappingCheckpointExecutionPreviewPanel
      request={request}
      onPreview={onPreview}
      result={result}
      loading={loading}
    />,
  );
  return { onPreview };
}

// ── Panel rendering ────────────────────────────────────────────────────────────

describe("RestoreMappingCheckpointExecutionPreviewPanel", () => {
  it("renders the panel container", async () => {
    await renderPanel(safeRequest());
    expect(screen.getByTestId("restore-mcep-panel")).toBeInTheDocument();
  });

  it("renders the execution-disabled notice", async () => {
    await renderPanel(safeRequest());
    expect(screen.getByTestId("mcep-execution-disabled-notice")).toBeInTheDocument();
  });

  it("renders the preview button", async () => {
    await renderPanel(safeRequest());
    expect(screen.getByTestId("mcep-preview-button")).toBeInTheDocument();
  });

  it("does not render result when result is null", async () => {
    await renderPanel(safeRequest(), null);
    expect(screen.queryByTestId("mcep-result")).not.toBeInTheDocument();
  });

  it("shows loading state when loading is true", async () => {
    await renderPanel(safeRequest(), null, true);
    expect(screen.getByTestId("mcep-preview-button")).toBeDisabled();
  });

  it("calls onPreview when button is clicked", async () => {
    const { onPreview } = await renderPanel(safeRequest());
    fireEvent.click(screen.getByTestId("mcep-preview-button"));
    await waitFor(() => expect(onPreview).toHaveBeenCalledOnce());
  });

  // ── Blocked result ─────────────────────────────────────────────────────────

  it("shows blocked badge when result is blocked", async () => {
    const result = await mockAirBridgeService.previewMappingCheckpointExecution(blockedRequest());
    render(
      <RestoreMappingCheckpointExecutionPreviewPanel
        request={blockedRequest()}
        onPreview={vi.fn()}
        result={result}
        loading={false}
      />,
    );
    expect(screen.getByTestId("mcep-blocked-badge")).toBeInTheDocument();
    expect(screen.queryByTestId("mcep-dry-run-badge")).not.toBeInTheDocument();
  });

  it("shows blocked reason when blocked", async () => {
    const result = await mockAirBridgeService.previewMappingCheckpointExecution(blockedRequest());
    render(
      <RestoreMappingCheckpointExecutionPreviewPanel
        request={blockedRequest()}
        onPreview={vi.fn()}
        result={result}
        loading={false}
      />,
    );
    expect(screen.getByTestId("mcep-blocked-reason")).toBeInTheDocument();
  });

  // ── DryRunReady result ─────────────────────────────────────────────────────

  it("shows dry-run badge when dryRunReady", async () => {
    const result = await mockAirBridgeService.previewMappingCheckpointExecution(safeRequest());
    render(
      <RestoreMappingCheckpointExecutionPreviewPanel
        request={safeRequest()}
        onPreview={vi.fn()}
        result={result}
        loading={false}
      />,
    );
    expect(screen.getByTestId("mcep-dry-run-badge")).toBeInTheDocument();
    expect(screen.queryByTestId("mcep-blocked-badge")).not.toBeInTheDocument();
  });

  it("renders execution-disabled tag when result present", async () => {
    const result = await mockAirBridgeService.previewMappingCheckpointExecution(safeRequest());
    render(
      <RestoreMappingCheckpointExecutionPreviewPanel
        request={safeRequest()}
        onPreview={vi.fn()}
        result={result}
        loading={false}
      />,
    );
    expect(screen.getByTestId("mcep-execution-disabled-tag")).toBeInTheDocument();
  });

  it("renders ID mapping summary when result present", async () => {
    const result = await mockAirBridgeService.previewMappingCheckpointExecution(safeRequest());
    render(
      <RestoreMappingCheckpointExecutionPreviewPanel
        request={safeRequest()}
        onPreview={vi.fn()}
        result={result}
        loading={false}
      />,
    );
    expect(screen.getByTestId("mcep-id-mapping-summary")).toBeInTheDocument();
    expect(screen.getByTestId("mcep-total-mapping-count")).toBeInTheDocument();
  });

  it("renders checkpoint boundary table when result present", async () => {
    const result = await mockAirBridgeService.previewMappingCheckpointExecution(safeRequest());
    render(
      <RestoreMappingCheckpointExecutionPreviewPanel
        request={safeRequest()}
        onPreview={vi.fn()}
        result={result}
        loading={false}
      />,
    );
    expect(screen.getByTestId("mcep-checkpoint-summary")).toBeInTheDocument();
    expect(screen.getByTestId("mcep-total-checkpoint-count")).toBeInTheDocument();
  });

  it("renders steps list when result present", async () => {
    const result = await mockAirBridgeService.previewMappingCheckpointExecution(safeRequest());
    render(
      <RestoreMappingCheckpointExecutionPreviewPanel
        request={safeRequest()}
        onPreview={vi.fn()}
        result={result}
        loading={false}
      />,
    );
    expect(screen.getByTestId("mcep-steps")).toBeInTheDocument();
  });

  it("renders no-changes-made tag when result present", async () => {
    const result = await mockAirBridgeService.previewMappingCheckpointExecution(safeRequest());
    render(
      <RestoreMappingCheckpointExecutionPreviewPanel
        request={safeRequest()}
        onPreview={vi.fn()}
        result={result}
        loading={false}
      />,
    );
    expect(screen.getByTestId("mcep-no-changes-made")).toBeInTheDocument();
  });

  it("has no execute button", async () => {
    const result = await mockAirBridgeService.previewMappingCheckpointExecution(safeRequest());
    render(
      <RestoreMappingCheckpointExecutionPreviewPanel
        request={safeRequest()}
        onPreview={vi.fn()}
        result={result}
        loading={false}
      />,
    );
    expect(screen.queryByTestId("mcep-execute-button")).not.toBeInTheDocument();
    expect(screen.queryByTestId("mcep-enable-button")).not.toBeInTheDocument();
  });

  it("has no token input", async () => {
    await renderPanel(safeRequest());
    expect(screen.queryByLabelText(/token/i)).not.toBeInTheDocument();
    expect(document.querySelector('input[type="password"]')).not.toBeInTheDocument();
  });

  // ── Safety invariants via mock service ────────────────────────────────────

  it("mock: blocked when missing all prerequisites", async () => {
    const result = await mockAirBridgeService.previewMappingCheckpointExecution(blockedRequest());
    expect(result.status).toBe("blocked");
    expect(result.writesEnabled).toBe(false);
    expect(result.noChangesMade).toBe(true);
    expect(result.networkWritesAttempted).toBe(false);
  });

  it("mock: dryRunReady for safe request", async () => {
    const result = await mockAirBridgeService.previewMappingCheckpointExecution(safeRequest());
    expect(result.status).toBe("dryRunReady");
    expect(result.mode).toBe("dryRunOnly");
    expect(result.writesEnabled).toBe(false);
    expect(result.noChangesMade).toBe(true);
    expect(result.networkWritesAttempted).toBe(false);
  });

  it("mock: blocked when record write preview not ready", async () => {
    const result = await mockAirBridgeService.previewMappingCheckpointExecution({
      ...safeRequest(),
      recordWritePreviewReady: false,
    });
    expect(result.status).toBe("blocked");
    expect(result.blockedReason).toContain("MCEP-PRE-02");
  });

  it("mock: blocked when checkpoint durability unsafe", async () => {
    const result = await mockAirBridgeService.previewMappingCheckpointExecution({
      ...safeRequest(),
      checkpointDurabilitySafe: false,
    });
    expect(result.status).toBe("blocked");
  });

  it("mock: blocked when failure modes unsafe", async () => {
    const result = await mockAirBridgeService.previewMappingCheckpointExecution({
      ...safeRequest(),
      failureModesSafe: false,
    });
    expect(result.status).toBe("blocked");
  });

  it("mock: blocked when rollback limitation unsafe", async () => {
    const result = await mockAirBridgeService.previewMappingCheckpointExecution({
      ...safeRequest(),
      rollbackLimitationSafe: false,
    });
    expect(result.status).toBe("blocked");
  });

  it("mock: blocked when final validation enforcement missing", async () => {
    const result = await mockAirBridgeService.previewMappingCheckpointExecution({
      ...safeRequest(),
      finalValidationEnforcementPresent: false,
    });
    expect(result.status).toBe("blocked");
  });

  it("mock: blocked when sensitive data unsafe", async () => {
    const result = await mockAirBridgeService.previewMappingCheckpointExecution({
      ...safeRequest(),
      sensitiveDataSafe: false,
    });
    expect(result.status).toBe("blocked");
  });

  it("mock: blocked when live write readiness missing", async () => {
    const result = await mockAirBridgeService.previewMappingCheckpointExecution({
      ...safeRequest(),
      liveWriteReadinessSatisfied: false,
    });
    expect(result.status).toBe("blocked");
  });

  it("mock: dryRunReady result has schema checkpoint first", async () => {
    const result = await mockAirBridgeService.previewMappingCheckpointExecution(safeRequest());
    expect(result.steps[0].stepId).toBe("MCEP-CHK-SCHEMA");
  });

  it("mock: dryRunReady result ends with pre-final-validation checkpoint", async () => {
    const result = await mockAirBridgeService.previewMappingCheckpointExecution(safeRequest());
    const last = result.steps[result.steps.length - 1];
    expect(last.stepId).toBe("MCEP-CHK-PRE-FV");
  });

  it("mock: record mapping steps come before pre-linked-update checkpoint", async () => {
    const result = await mockAirBridgeService.previewMappingCheckpointExecution(safeRequest());
    const mapLastIdx = result.steps
      .filter((s) => s.stepId.startsWith("MCEP-MAP-REC-B"))
      .map((s) => s.stepIndex)
      .reduce((a, b) => Math.max(a, b), -1);
    const preLinkIdx =
      result.steps.find((s) => s.stepId === "MCEP-CHK-PRE-LINK")?.stepIndex ?? Infinity;
    expect(mapLastIdx).toBeLessThan(preLinkIdx);
  });

  it("mock: no token in serialized result", async () => {
    const result = await mockAirBridgeService.previewMappingCheckpointExecution(safeRequest());
    const json = JSON.stringify(result);
    expect(json).not.toContain('"token"');
    expect(json).not.toContain("pat_");
  });

  it("mock: no absolute path in serialized result", async () => {
    const result = await mockAirBridgeService.previewMappingCheckpointExecution(safeRequest());
    const json = JSON.stringify(result);
    expect(json).not.toContain("/Users/");
    expect(json).not.toContain("/home/");
  });

  it("mock: no record payload in serialized result", async () => {
    const result = await mockAirBridgeService.previewMappingCheckpointExecution(safeRequest());
    const json = JSON.stringify(result);
    expect(json).not.toContain('"fields":{');
    expect(json).not.toContain('"records":[{');
  });

  it("mock: no attachment URL in serialized result", async () => {
    const result = await mockAirBridgeService.previewMappingCheckpointExecution(safeRequest());
    const json = JSON.stringify(result);
    expect(json).not.toContain("cdn.airtable.com");
    expect(json).not.toContain("attachmentUrl");
  });

  it("mock: no succeeded state in serialized result", async () => {
    const result = await mockAirBridgeService.previewMappingCheckpointExecution(safeRequest());
    const json = JSON.stringify(result);
    expect(json).not.toContain('"succeeded"');
    expect(result.writesEnabled).toBe(false);
  });

  it("mock: message states execution remains disabled", async () => {
    const result = await mockAirBridgeService.previewMappingCheckpointExecution(safeRequest());
    expect(result.message.toLowerCase()).toContain("disabled");
  });

  it("mock: message states no restore execution is started", async () => {
    const result = await mockAirBridgeService.previewMappingCheckpointExecution(safeRequest());
    expect(result.message.toLowerCase()).toContain("does not start any restore execution");
  });

  it("mock: message states no checkpoint files are written", async () => {
    const result = await mockAirBridgeService.previewMappingCheckpointExecution(safeRequest());
    expect(result.message.toLowerCase()).toContain("no checkpoint files are written");
  });

  it("mock: safetySnapshot writeGateDisabled is always true", async () => {
    const result = await mockAirBridgeService.previewMappingCheckpointExecution(safeRequest());
    expect(result.safetySnapshot.writeGateDisabled).toBe(true);
  });

  it("mock: checkpoint summary counts match request", async () => {
    const result = await mockAirBridgeService.previewMappingCheckpointExecution(safeRequest());
    expect(result.checkpointSummary.recordCreateCheckpointCount).toBe(4);
    expect(result.checkpointSummary.linkedUpdateCheckpointCount).toBe(2);
    expect(result.checkpointSummary.hasPreRecordCreateCheckpoint).toBe(true);
    expect(result.checkpointSummary.hasPreLinkedUpdateCheckpoint).toBe(true);
    expect(result.checkpointSummary.hasPreFinalValidationCheckpoint).toBe(true);
  });

  it("mock: id mapping summary counts match request", async () => {
    const result = await mockAirBridgeService.previewMappingCheckpointExecution(safeRequest());
    expect(result.idMappingSummary.totalMappingCount).toBe(35);
    expect(result.idMappingSummary.tablesRequiringRemapping).toBe(2);
    expect(result.idMappingSummary.firstPassBatchCount).toBe(4);
    expect(result.idMappingSummary.mappingAvailableBeforeSecondPass).toBe(true);
  });
});
