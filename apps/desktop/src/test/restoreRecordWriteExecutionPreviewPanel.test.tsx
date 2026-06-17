import { describe, it, expect, vi } from "vitest";
import { render, screen, fireEvent, waitFor } from "@testing-library/react";
import { RestoreRecordWriteExecutionPreviewPanel } from "../features/backups/RestoreRecordWriteExecutionPreviewPanel";
import { mockAirBridgeService } from "../services/mockAirBridgeService";
import type {
  RecordWriteExecutionPreviewRequest,
  RecordWriteExecutionPreviewResult,
} from "../backend/types";

function safeRequest(): RecordWriteExecutionPreviewRequest {
  return {
    packageFilename: "test-backup.airbridge",
    schemaPreviewReady: true,
    sandboxFlagPresent: true,
    targetEmptyVerified: true,
    recordImportPlanReady: true,
    recordWriteRequestPlanReady: true,
    tableCount: 2,
    totalFirstPassBatches: 4,
    totalSecondPassBatches: 2,
    totalRecordCount: 35,
    batchSize: 10,
    rateLimitBackoffSafe: true,
    checkpointDurabilitySafe: true,
    sensitiveDataSafe: true,
    attachmentPhaseDisabled: true,
    finalValidationEnforcementPresent: true,
    liveWriteReadinessSatisfied: true,
  };
}

function blockedRequest(): RecordWriteExecutionPreviewRequest {
  return {};
}

async function renderWithResult(
  request: RecordWriteExecutionPreviewRequest,
  result: RecordWriteExecutionPreviewResult | null = null,
  loading = false,
) {
  const onPreview = vi
    .fn()
    .mockResolvedValue(await mockAirBridgeService.previewRecordWriteExecution(request));
  render(
    <RestoreRecordWriteExecutionPreviewPanel
      request={request}
      onPreview={onPreview}
      result={result}
      loading={loading}
    />,
  );
  return { onPreview };
}

// ── Panel rendering ────────────────────────────────────────────────────────────

describe("RestoreRecordWriteExecutionPreviewPanel", () => {
  it("renders the panel container", async () => {
    await renderWithResult(safeRequest());
    expect(screen.getByTestId("restore-rwep-panel")).toBeInTheDocument();
  });

  it("renders the writes-disabled notice", async () => {
    await renderWithResult(safeRequest());
    expect(screen.getByTestId("rwep-writes-disabled-notice")).toBeInTheDocument();
  });

  it("renders the preview button", async () => {
    await renderWithResult(safeRequest());
    expect(screen.getByTestId("rwep-preview-button")).toBeInTheDocument();
  });

  it("does not render result when result is null", async () => {
    await renderWithResult(safeRequest(), null);
    expect(screen.queryByTestId("rwep-result")).not.toBeInTheDocument();
  });

  it("shows loading state when loading is true", async () => {
    await renderWithResult(safeRequest(), null, true);
    expect(screen.getByTestId("rwep-preview-button")).toBeDisabled();
  });

  // ── Button behavior ──────────────────────────────────────────────────────────

  it("calls onPreview when button is clicked", async () => {
    const { onPreview } = await renderWithResult(safeRequest());
    fireEvent.click(screen.getByTestId("rwep-preview-button"));
    await waitFor(() => expect(onPreview).toHaveBeenCalledOnce());
  });

  // ── Blocked result ─────────────────────────────────────────────────────────

  it("shows blocked badge when result is blocked", async () => {
    const result = await mockAirBridgeService.previewRecordWriteExecution(blockedRequest());
    const onPreview = vi.fn().mockResolvedValue(result);
    render(
      <RestoreRecordWriteExecutionPreviewPanel
        request={blockedRequest()}
        onPreview={onPreview}
        result={result}
        loading={false}
      />,
    );
    expect(screen.getByTestId("rwep-blocked-badge")).toBeInTheDocument();
    expect(screen.queryByTestId("rwep-dry-run-badge")).not.toBeInTheDocument();
  });

  it("shows blocked reason when blocked", async () => {
    const result = await mockAirBridgeService.previewRecordWriteExecution(blockedRequest());
    const onPreview = vi.fn().mockResolvedValue(result);
    render(
      <RestoreRecordWriteExecutionPreviewPanel
        request={blockedRequest()}
        onPreview={onPreview}
        result={result}
        loading={false}
      />,
    );
    expect(screen.getByTestId("rwep-blocked-reason")).toBeInTheDocument();
  });

  // ── DryRunReady result ─────────────────────────────────────────────────────

  it("shows dry-run badge when dryRunReady", async () => {
    const result = await mockAirBridgeService.previewRecordWriteExecution(safeRequest());
    const onPreview = vi.fn().mockResolvedValue(result);
    render(
      <RestoreRecordWriteExecutionPreviewPanel
        request={safeRequest()}
        onPreview={onPreview}
        result={result}
        loading={false}
      />,
    );
    expect(screen.getByTestId("rwep-dry-run-badge")).toBeInTheDocument();
    expect(screen.queryByTestId("rwep-blocked-badge")).not.toBeInTheDocument();
  });

  it("shows writes-disabled tag when result present", async () => {
    const result = await mockAirBridgeService.previewRecordWriteExecution(safeRequest());
    const onPreview = vi.fn().mockResolvedValue(result);
    render(
      <RestoreRecordWriteExecutionPreviewPanel
        request={safeRequest()}
        onPreview={onPreview}
        result={result}
        loading={false}
      />,
    );
    expect(screen.getByTestId("rwep-writes-disabled-tag")).toBeInTheDocument();
  });

  it("shows message when result present", async () => {
    const result = await mockAirBridgeService.previewRecordWriteExecution(safeRequest());
    const onPreview = vi.fn().mockResolvedValue(result);
    render(
      <RestoreRecordWriteExecutionPreviewPanel
        request={safeRequest()}
        onPreview={onPreview}
        result={result}
        loading={false}
      />,
    );
    expect(screen.getByTestId("rwep-message")).toBeInTheDocument();
  });

  it("shows batch counts when result present", async () => {
    const result = await mockAirBridgeService.previewRecordWriteExecution(safeRequest());
    const onPreview = vi.fn().mockResolvedValue(result);
    render(
      <RestoreRecordWriteExecutionPreviewPanel
        request={safeRequest()}
        onPreview={onPreview}
        result={result}
        loading={false}
      />,
    );
    expect(screen.getByTestId("rwep-batch-counts")).toBeInTheDocument();
  });

  it("shows batches list when result present", async () => {
    const result = await mockAirBridgeService.previewRecordWriteExecution(safeRequest());
    const onPreview = vi.fn().mockResolvedValue(result);
    render(
      <RestoreRecordWriteExecutionPreviewPanel
        request={safeRequest()}
        onPreview={onPreview}
        result={result}
        loading={false}
      />,
    );
    expect(screen.getByTestId("rwep-batches")).toBeInTheDocument();
  });

  it("shows no-changes-made tag when result present", async () => {
    const result = await mockAirBridgeService.previewRecordWriteExecution(safeRequest());
    const onPreview = vi.fn().mockResolvedValue(result);
    render(
      <RestoreRecordWriteExecutionPreviewPanel
        request={safeRequest()}
        onPreview={onPreview}
        result={result}
        loading={false}
      />,
    );
    expect(screen.getByTestId("rwep-no-changes-made")).toBeInTheDocument();
  });

  // ── Safety invariants via mock service ────────────────────────────────────

  it("mock: blocked when missing all prerequisites", async () => {
    const result = await mockAirBridgeService.previewRecordWriteExecution(blockedRequest());
    expect(result.status).toBe("blocked");
    expect(result.writesEnabled).toBe(false);
    expect(result.noChangesMade).toBe(true);
    expect(result.networkWritesAttempted).toBe(false);
  });

  it("mock: dryRunReady for safe request", async () => {
    const result = await mockAirBridgeService.previewRecordWriteExecution(safeRequest());
    expect(result.status).toBe("dryRunReady");
    expect(result.mode).toBe("dryRunOnly");
    expect(result.writesEnabled).toBe(false);
    expect(result.noChangesMade).toBe(true);
    expect(result.networkWritesAttempted).toBe(false);
  });

  it("mock: blocked when schema preview not ready", async () => {
    const result = await mockAirBridgeService.previewRecordWriteExecution({
      ...safeRequest(),
      schemaPreviewReady: false,
    });
    expect(result.status).toBe("blocked");
  });

  it("mock: blocked when batch size too large", async () => {
    const result = await mockAirBridgeService.previewRecordWriteExecution({
      ...safeRequest(),
      batchSize: 11,
    });
    expect(result.status).toBe("blocked");
  });

  it("mock: blocked when rate-limit policy unsafe", async () => {
    const result = await mockAirBridgeService.previewRecordWriteExecution({
      ...safeRequest(),
      rateLimitBackoffSafe: false,
    });
    expect(result.status).toBe("blocked");
  });

  it("mock: blocked when checkpoint durability unsafe", async () => {
    const result = await mockAirBridgeService.previewRecordWriteExecution({
      ...safeRequest(),
      checkpointDurabilitySafe: false,
    });
    expect(result.status).toBe("blocked");
  });

  it("mock: blocked when live write readiness not satisfied", async () => {
    const result = await mockAirBridgeService.previewRecordWriteExecution({
      ...safeRequest(),
      liveWriteReadinessSatisfied: false,
    });
    expect(result.status).toBe("blocked");
  });

  it("mock: dryRunReady result has first-pass batches before second-pass", async () => {
    const result = await mockAirBridgeService.previewRecordWriteExecution(safeRequest());
    const fpLast = result.batches
      .filter((b) => b.operationClass === "first-pass-create")
      .map((b) => b.batchIndex)
      .reduce((a, b) => Math.max(a, b), -1);
    const spFirst = result.batches
      .filter((b) => b.operationClass === "second-pass-linked-update")
      .map((b) => b.batchIndex)
      .reduce((a, b) => Math.min(a, b), Infinity);
    expect(fpLast).toBeLessThan(spFirst);
  });

  it("mock: no token in dry-run-ready result", async () => {
    const result = await mockAirBridgeService.previewRecordWriteExecution(safeRequest());
    const json = JSON.stringify(result);
    expect(json).not.toContain('"token"');
    expect(json).not.toContain("pat_");
  });

  it("mock: no absolute path in dry-run-ready result", async () => {
    const result = await mockAirBridgeService.previewRecordWriteExecution(safeRequest());
    const json = JSON.stringify(result);
    expect(json).not.toContain("/Users/");
    expect(json).not.toContain("/home/");
  });

  it("mock: no attachment URL in dry-run-ready result", async () => {
    const result = await mockAirBridgeService.previewRecordWriteExecution(safeRequest());
    const json = JSON.stringify(result);
    expect(json).not.toContain("cdn.airtable.com");
    expect(json).not.toContain("attachmentUrl");
  });

  it("mock: no raw record payload in dry-run-ready result", async () => {
    const result = await mockAirBridgeService.previewRecordWriteExecution(safeRequest());
    const json = JSON.stringify(result);
    expect(json).not.toContain('"fields":{');
    expect(json).not.toContain('"records":[{');
  });

  it("mock: message states writes remain disabled", async () => {
    const result = await mockAirBridgeService.previewRecordWriteExecution(safeRequest());
    expect(result.message.toLowerCase()).toContain("disabled");
  });

  it("mock: message states no restore execution is started", async () => {
    const result = await mockAirBridgeService.previewRecordWriteExecution(safeRequest());
    expect(result.message.toLowerCase()).toContain("does not start any restore execution");
  });

  it("mock: batch indices are sequential", async () => {
    const result = await mockAirBridgeService.previewRecordWriteExecution(safeRequest());
    result.batches
      .filter((b) => b.operationClass !== "blocked" && b.operationClass !== "no-operations")
      .forEach((b, i) => {
        expect(b.batchIndex).toBe(i);
      });
  });

  it("mock: batch record count does not exceed batch size", async () => {
    const result = await mockAirBridgeService.previewRecordWriteExecution(safeRequest());
    result.batches.forEach((b) => {
      expect(b.recordCount).toBeLessThanOrEqual(result.batchSize);
    });
  });

  it("mock: safety snapshot writeGateDisabled is true", async () => {
    const result = await mockAirBridgeService.previewRecordWriteExecution(safeRequest());
    expect(result.safetySnapshot.writeGateDisabled).toBe(true);
  });

  it("mock: no succeeded state introduced", async () => {
    const result = await mockAirBridgeService.previewRecordWriteExecution(safeRequest());
    expect(result.writesEnabled).toBe(false);
    expect(result.status).not.toBe("succeeded");
    const json = JSON.stringify(result);
    expect(json).not.toContain('"succeeded"');
  });
});
