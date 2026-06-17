import { describe, it, expect, vi } from "vitest";
import { render, screen, fireEvent, waitFor } from "@testing-library/react";
import { RestoreLinkedSecondPassExecutionPreviewPanel } from "../features/backups/RestoreLinkedSecondPassExecutionPreviewPanel";
import { mockAirBridgeService } from "../services/mockAirBridgeService";
import type {
  LinkedSecondPassExecutionPreviewRequest,
  LinkedSecondPassExecutionPreviewResult,
} from "../backend/types";

function safeRequest(): LinkedSecondPassExecutionPreviewRequest {
  return {
    packageFilename: "test-backup.airbridge",
    recordWritePreviewReady: true,
    mappingCheckpointPreviewReady: true,
    secondPassBatchCount: 3,
    totalUpdateCount: 20,
    tablesWithLinkedFields: 2,
    totalLinkedFields: 3,
    batchSize: 10,
    fieldSummaries: [
      {
        tableLabel: "Projects",
        fieldLabel: "Tasks",
        recordCount: 15,
        batchCount: 2,
        unresolvedLinkCount: 0,
      },
      {
        tableLabel: "Tasks",
        fieldLabel: "Owner",
        recordCount: 5,
        batchCount: 1,
        unresolvedLinkCount: 0,
      },
    ],
    writePhaseOrderingSafe: true,
    checkpointDurabilitySafe: true,
    sensitiveDataSafe: true,
    finalValidationEnforcementPresent: true,
    liveWriteReadinessSatisfied: true,
  };
}

function blockedRequest(): LinkedSecondPassExecutionPreviewRequest {
  return {};
}

async function renderPanel(
  request: LinkedSecondPassExecutionPreviewRequest,
  result: LinkedSecondPassExecutionPreviewResult | null = null,
  loading = false,
) {
  const onPreview = vi
    .fn()
    .mockResolvedValue(await mockAirBridgeService.previewLinkedSecondPassExecution(request));
  render(
    <RestoreLinkedSecondPassExecutionPreviewPanel
      request={request}
      onPreview={onPreview}
      result={result}
      loading={loading}
    />,
  );
  return { onPreview };
}

// ── Panel rendering ────────────────────────────────────────────────────────────

describe("RestoreLinkedSecondPassExecutionPreviewPanel", () => {
  it("renders the panel container", async () => {
    await renderPanel(safeRequest());
    expect(screen.getByTestId("restore-lsep-panel")).toBeInTheDocument();
  });

  it("renders the execution-disabled notice", async () => {
    await renderPanel(safeRequest());
    expect(screen.getByTestId("lsep-execution-disabled-notice")).toBeInTheDocument();
  });

  it("renders the preview button", async () => {
    await renderPanel(safeRequest());
    expect(screen.getByTestId("lsep-preview-button")).toBeInTheDocument();
  });

  it("does not render result when result is null", async () => {
    await renderPanel(safeRequest(), null);
    expect(screen.queryByTestId("lsep-result")).not.toBeInTheDocument();
  });

  it("shows loading state when loading is true", async () => {
    await renderPanel(safeRequest(), null, true);
    expect(screen.getByTestId("lsep-preview-button")).toBeDisabled();
  });

  it("calls onPreview when button is clicked", async () => {
    const { onPreview } = await renderPanel(safeRequest());
    fireEvent.click(screen.getByTestId("lsep-preview-button"));
    await waitFor(() => expect(onPreview).toHaveBeenCalledOnce());
  });

  // ── Blocked result ─────────────────────────────────────────────────────────

  it("shows blocked badge when result is blocked", async () => {
    const result = await mockAirBridgeService.previewLinkedSecondPassExecution(blockedRequest());
    render(
      <RestoreLinkedSecondPassExecutionPreviewPanel
        request={blockedRequest()}
        onPreview={vi.fn()}
        result={result}
        loading={false}
      />,
    );
    expect(screen.getByTestId("lsep-blocked-badge")).toBeInTheDocument();
    expect(screen.queryByTestId("lsep-dry-run-badge")).not.toBeInTheDocument();
  });

  it("shows blocked reason when blocked", async () => {
    const result = await mockAirBridgeService.previewLinkedSecondPassExecution(blockedRequest());
    render(
      <RestoreLinkedSecondPassExecutionPreviewPanel
        request={blockedRequest()}
        onPreview={vi.fn()}
        result={result}
        loading={false}
      />,
    );
    expect(screen.getByTestId("lsep-blocked-reason")).toBeInTheDocument();
  });

  // ── DryRunReady result ─────────────────────────────────────────────────────

  it("shows dry-run badge when dryRunReady", async () => {
    const result = await mockAirBridgeService.previewLinkedSecondPassExecution(safeRequest());
    render(
      <RestoreLinkedSecondPassExecutionPreviewPanel
        request={safeRequest()}
        onPreview={vi.fn()}
        result={result}
        loading={false}
      />,
    );
    expect(screen.getByTestId("lsep-dry-run-badge")).toBeInTheDocument();
    expect(screen.queryByTestId("lsep-blocked-badge")).not.toBeInTheDocument();
  });

  it("renders execution-disabled tag when result present", async () => {
    const result = await mockAirBridgeService.previewLinkedSecondPassExecution(safeRequest());
    render(
      <RestoreLinkedSecondPassExecutionPreviewPanel
        request={safeRequest()}
        onPreview={vi.fn()}
        result={result}
        loading={false}
      />,
    );
    expect(screen.getByTestId("lsep-execution-disabled-tag")).toBeInTheDocument();
  });

  it("renders linked field summary when result present", async () => {
    const result = await mockAirBridgeService.previewLinkedSecondPassExecution(safeRequest());
    render(
      <RestoreLinkedSecondPassExecutionPreviewPanel
        request={safeRequest()}
        onPreview={vi.fn()}
        result={result}
        loading={false}
      />,
    );
    expect(screen.getByTestId("lsep-field-summary")).toBeInTheDocument();
    expect(screen.getByTestId("lsep-mapping-summary")).toBeInTheDocument();
    expect(screen.getByTestId("lsep-total-update-count")).toBeInTheDocument();
  });

  it("renders batch summary table when result present", async () => {
    const result = await mockAirBridgeService.previewLinkedSecondPassExecution(safeRequest());
    render(
      <RestoreLinkedSecondPassExecutionPreviewPanel
        request={safeRequest()}
        onPreview={vi.fn()}
        result={result}
        loading={false}
      />,
    );
    expect(screen.getByTestId("lsep-batch-summary")).toBeInTheDocument();
    expect(screen.getByTestId("lsep-total-batch-count")).toBeInTheDocument();
    expect(screen.getByTestId("lsep-batches")).toBeInTheDocument();
  });

  it("renders unresolved-link summary when result present", async () => {
    const result = await mockAirBridgeService.previewLinkedSecondPassExecution(safeRequest());
    render(
      <RestoreLinkedSecondPassExecutionPreviewPanel
        request={safeRequest()}
        onPreview={vi.fn()}
        result={result}
        loading={false}
      />,
    );
    expect(screen.getByTestId("lsep-unresolved-link-count")).toBeInTheDocument();
  });

  it("renders no-changes-made tag when result present", async () => {
    const result = await mockAirBridgeService.previewLinkedSecondPassExecution(safeRequest());
    render(
      <RestoreLinkedSecondPassExecutionPreviewPanel
        request={safeRequest()}
        onPreview={vi.fn()}
        result={result}
        loading={false}
      />,
    );
    expect(screen.getByTestId("lsep-no-changes-made")).toBeInTheDocument();
  });

  it("has no execute button", async () => {
    const result = await mockAirBridgeService.previewLinkedSecondPassExecution(safeRequest());
    render(
      <RestoreLinkedSecondPassExecutionPreviewPanel
        request={safeRequest()}
        onPreview={vi.fn()}
        result={result}
        loading={false}
      />,
    );
    expect(screen.queryByTestId("lsep-execute-button")).not.toBeInTheDocument();
    expect(screen.queryByTestId("lsep-enable-button")).not.toBeInTheDocument();
  });

  it("has no token input", async () => {
    await renderPanel(safeRequest());
    expect(screen.queryByLabelText(/token/i)).not.toBeInTheDocument();
    expect(document.querySelector('input[type="password"]')).not.toBeInTheDocument();
  });

  // ── Safety invariants via mock service ────────────────────────────────────

  it("mock: blocked when missing all prerequisites", async () => {
    const result = await mockAirBridgeService.previewLinkedSecondPassExecution(blockedRequest());
    expect(result.status).toBe("blocked");
    expect(result.writesEnabled).toBe(false);
    expect(result.noChangesMade).toBe(true);
    expect(result.networkWritesAttempted).toBe(false);
  });

  it("mock: dryRunReady for safe request", async () => {
    const result = await mockAirBridgeService.previewLinkedSecondPassExecution(safeRequest());
    expect(result.status).toBe("dryRunReady");
    expect(result.mode).toBe("dryRunOnly");
    expect(result.writesEnabled).toBe(false);
    expect(result.noChangesMade).toBe(true);
    expect(result.networkWritesAttempted).toBe(false);
  });

  it("mock: blocked when record write preview not ready", async () => {
    const result = await mockAirBridgeService.previewLinkedSecondPassExecution({
      ...safeRequest(),
      recordWritePreviewReady: false,
    });
    expect(result.status).toBe("blocked");
    expect(result.blockedReason).toContain("LSEP-PRE-02");
  });

  it("mock: blocked when mapping checkpoint preview not ready", async () => {
    const result = await mockAirBridgeService.previewLinkedSecondPassExecution({
      ...safeRequest(),
      mappingCheckpointPreviewReady: false,
    });
    expect(result.status).toBe("blocked");
    expect(result.blockedReason).toContain("LSEP-PRE-03");
  });

  it("mock: blocked when write phase ordering unsafe", async () => {
    const result = await mockAirBridgeService.previewLinkedSecondPassExecution({
      ...safeRequest(),
      writePhaseOrderingSafe: false,
    });
    expect(result.status).toBe("blocked");
    expect(result.blockedReason).toContain("LSEP-PRE-04");
  });

  it("mock: blocked when checkpoint durability unsafe", async () => {
    const result = await mockAirBridgeService.previewLinkedSecondPassExecution({
      ...safeRequest(),
      checkpointDurabilitySafe: false,
    });
    expect(result.status).toBe("blocked");
  });

  it("mock: blocked when sensitive data unsafe", async () => {
    const result = await mockAirBridgeService.previewLinkedSecondPassExecution({
      ...safeRequest(),
      sensitiveDataSafe: false,
    });
    expect(result.status).toBe("blocked");
  });

  it("mock: blocked when final validation enforcement missing", async () => {
    const result = await mockAirBridgeService.previewLinkedSecondPassExecution({
      ...safeRequest(),
      finalValidationEnforcementPresent: false,
    });
    expect(result.status).toBe("blocked");
  });

  it("mock: blocked when live write readiness missing", async () => {
    const result = await mockAirBridgeService.previewLinkedSecondPassExecution({
      ...safeRequest(),
      liveWriteReadinessSatisfied: false,
    });
    expect(result.status).toBe("blocked");
  });

  it("mock: warns on unresolved links, not blocked", async () => {
    const result = await mockAirBridgeService.previewLinkedSecondPassExecution({
      ...safeRequest(),
      fieldSummaries: [
        {
          tableLabel: "Projects",
          fieldLabel: "Tasks",
          recordCount: 10,
          batchCount: 1,
          unresolvedLinkCount: 3,
        },
      ],
    });
    expect(result.status).toBe("dryRunReady");
    expect(result.mappingSummary.unresolvedLinkCount).toBeGreaterThan(0);
    expect(result.message).toContain("unresolved");
  });

  it("mock: no token in serialized result", async () => {
    const result = await mockAirBridgeService.previewLinkedSecondPassExecution(safeRequest());
    const json = JSON.stringify(result);
    expect(json).not.toContain('"token"');
    expect(json).not.toContain("pat_");
  });

  it("mock: no absolute path in serialized result", async () => {
    const result = await mockAirBridgeService.previewLinkedSecondPassExecution(safeRequest());
    const json = JSON.stringify(result);
    expect(json).not.toContain("/Users/");
    expect(json).not.toContain("/home/");
  });

  it("mock: no record payload in serialized result", async () => {
    const result = await mockAirBridgeService.previewLinkedSecondPassExecution(safeRequest());
    const json = JSON.stringify(result);
    expect(json).not.toContain('"fields":{');
    expect(json).not.toContain('"records":[{');
  });

  it("mock: no attachment URL in serialized result", async () => {
    const result = await mockAirBridgeService.previewLinkedSecondPassExecution(safeRequest());
    const json = JSON.stringify(result);
    expect(json).not.toContain("cdn.airtable.com");
    expect(json).not.toContain("attachmentUrl");
  });

  it("mock: no old or new record IDs in serialized result", async () => {
    const result = await mockAirBridgeService.previewLinkedSecondPassExecution(safeRequest());
    const json = JSON.stringify(result);
    expect(json).not.toContain('"oldRecordId"');
    expect(json).not.toContain('"newRecordId"');
  });

  it("mock: no success state in serialized result", async () => {
    const result = await mockAirBridgeService.previewLinkedSecondPassExecution(safeRequest());
    const json = JSON.stringify(result);
    expect(json).not.toContain('"succeeded"');
    expect(result.writesEnabled).toBe(false);
  });

  it("mock: message states live updates disabled", async () => {
    const result = await mockAirBridgeService.previewLinkedSecondPassExecution(safeRequest());
    expect(result.message.toLowerCase()).toContain("disabled");
  });

  it("mock: message states no restore execution started", async () => {
    const result = await mockAirBridgeService.previewLinkedSecondPassExecution(safeRequest());
    expect(result.message.toLowerCase()).toContain("does not start any restore execution");
  });

  it("mock: message states no checkpoint files written", async () => {
    const result = await mockAirBridgeService.previewLinkedSecondPassExecution(safeRequest());
    expect(result.message.toLowerCase()).toContain("no checkpoint files are written");
  });

  it("mock: message states no record IDs", async () => {
    const result = await mockAirBridgeService.previewLinkedSecondPassExecution(safeRequest());
    expect(result.message.toLowerCase()).toContain("no record ids are present");
  });

  it("mock: safetySnapshot writeGateDisabled is always true", async () => {
    const result = await mockAirBridgeService.previewLinkedSecondPassExecution(safeRequest());
    expect(result.safetySnapshot.writeGateDisabled).toBe(true);
  });

  it("mock: mapping summary counts match request", async () => {
    const result = await mockAirBridgeService.previewLinkedSecondPassExecution(safeRequest());
    expect(result.mappingSummary.totalUpdateCount).toBe(20);
    expect(result.mappingSummary.tablesWithLinkedFields).toBe(2);
    expect(result.mappingSummary.totalLinkedFields).toBe(3);
    expect(result.mappingSummary.mappingCompleteBeforeSecondPass).toBe(true);
  });

  it("mock: batch count matches field summaries", async () => {
    const result = await mockAirBridgeService.previewLinkedSecondPassExecution(safeRequest());
    // Projects/Tasks: 15 records / 10 = 2 batches; Tasks/Owner: 5 / 10 = 1 batch
    expect(result.batches.length).toBe(3);
  });

  it("mock: batch update count never exceeds batch size", async () => {
    const result = await mockAirBridgeService.previewLinkedSecondPassExecution(safeRequest());
    for (const batch of result.batches) {
      expect(batch.updateCount).toBeLessThanOrEqual(10);
    }
  });
});
