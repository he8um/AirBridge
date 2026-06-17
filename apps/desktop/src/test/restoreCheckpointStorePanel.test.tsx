import { describe, it, expect, vi } from "vitest";
import { render, screen, fireEvent, waitFor } from "@testing-library/react";
import { RestoreCheckpointStorePanel } from "../features/backups/RestoreCheckpointStorePanel";
import { mockAirBridgeService } from "../services/mockAirBridgeService";
import type { RestoreCheckpointStoreRequest, RestoreCheckpointStoreResult } from "../backend/types";

function safeRequest(): RestoreCheckpointStoreRequest {
  return {
    checkpointLabel: "test-checkpoint",
    checkpointDurabilitySafe: true,
    sensitiveDataSafe: true,
    mappingCheckpointPreviewReady: true,
    finalValidationPreviewReady: true,
    phases: [
      { phaseLabel: "schema", boundaryCount: 2, note: "Schema phase." },
      { phaseLabel: "record-create", boundaryCount: 3, note: "Record create phase." },
    ],
    boundaries: [
      { boundaryLabel: "schema-complete", boundaryIndex: 0, itemCount: 3, note: "Schema done." },
      { boundaryLabel: "batch-001", boundaryIndex: 1, itemCount: 10, note: "Batch 1 done." },
      { boundaryLabel: "batch-002", boundaryIndex: 2, itemCount: 10, note: "Batch 2 done." },
    ],
  };
}

function blockedRequest(): RestoreCheckpointStoreRequest {
  return {};
}

async function renderPanel(
  request: RestoreCheckpointStoreRequest,
  result: RestoreCheckpointStoreResult | null = null,
  loading = false,
) {
  const onStore = vi
    .fn()
    .mockResolvedValue(await mockAirBridgeService.storeRestoreCheckpointMetadata(request));
  render(
    <RestoreCheckpointStorePanel
      request={request}
      onStore={onStore}
      result={result}
      loading={loading}
    />,
  );
  return { onStore };
}

// ── Panel rendering ────────────────────────────────────────────────────────────

describe("RestoreCheckpointStorePanel", () => {
  it("renders the panel container", async () => {
    await renderPanel(safeRequest());
    expect(screen.getByTestId("restore-checkpoint-store-panel")).toBeInTheDocument();
  });

  it("renders the restore-not-triggered notice", async () => {
    await renderPanel(safeRequest());
    expect(screen.getByTestId("rcps-restore-not-triggered-notice")).toBeInTheDocument();
  });

  it("restore-not-triggered notice mentions execution disabled", async () => {
    await renderPanel(safeRequest());
    expect(screen.getByTestId("rcps-restore-not-triggered-notice").textContent).toMatch(
      /not execute restore/i,
    );
  });

  it("renders the store button", async () => {
    await renderPanel(safeRequest());
    expect(screen.getByTestId("rcps-store-button")).toBeInTheDocument();
  });

  it("renders the metadata-only badge", async () => {
    await renderPanel(safeRequest());
    expect(screen.getByTestId("rcps-metadata-only-badge")).toBeInTheDocument();
  });

  it("does not render result before store is run", async () => {
    await renderPanel(safeRequest());
    expect(screen.queryByTestId("rcps-result")).not.toBeInTheDocument();
  });

  it("shows loading state while store is in progress", async () => {
    await renderPanel(safeRequest(), null, true);
    expect(screen.getByTestId("rcps-store-button")).toBeDisabled();
  });

  // ── Stored state ──────────────────────────────────────────────────────────

  it("shows stored badge when result is stored", async () => {
    const result = await mockAirBridgeService.storeRestoreCheckpointMetadata(safeRequest());
    await renderPanel(safeRequest(), result);
    expect(screen.getByTestId("rcps-stored-badge")).toBeInTheDocument();
  });

  it("shows restore-not-triggered tag in stored result", async () => {
    const result = await mockAirBridgeService.storeRestoreCheckpointMetadata(safeRequest());
    await renderPanel(safeRequest(), result);
    expect(screen.getByTestId("rcps-restore-not-triggered-tag")).toBeInTheDocument();
  });

  it("shows message in stored result", async () => {
    const result = await mockAirBridgeService.storeRestoreCheckpointMetadata(safeRequest());
    await renderPanel(safeRequest(), result);
    expect(screen.getByTestId("rcps-message")).toBeInTheDocument();
  });

  it("message mentions metadata-only", async () => {
    const result = await mockAirBridgeService.storeRestoreCheckpointMetadata(safeRequest());
    await renderPanel(safeRequest(), result);
    expect(screen.getByTestId("rcps-message").textContent).toMatch(/metadata-only/i);
  });

  it("message mentions restore not triggered", async () => {
    const result = await mockAirBridgeService.storeRestoreCheckpointMetadata(safeRequest());
    await renderPanel(safeRequest(), result);
    expect(screen.getByTestId("rcps-message").textContent).toMatch(/not triggered/i);
  });

  it("shows no blocked reason for stored result", async () => {
    const result = await mockAirBridgeService.storeRestoreCheckpointMetadata(safeRequest());
    await renderPanel(safeRequest(), result);
    expect(screen.queryByTestId("rcps-blocked-reason")).not.toBeInTheDocument();
  });

  it("shows summary in stored result", async () => {
    const result = await mockAirBridgeService.storeRestoreCheckpointMetadata(safeRequest());
    await renderPanel(safeRequest(), result);
    expect(screen.getByTestId("rcps-summary")).toBeInTheDocument();
  });

  it("summary shows correct checkpoint label", async () => {
    const result = await mockAirBridgeService.storeRestoreCheckpointMetadata(safeRequest());
    await renderPanel(safeRequest(), result);
    expect(screen.getByTestId("rcps-summary-label").textContent).toBe("test-checkpoint");
  });

  it("summary shows correct boundary count", async () => {
    const result = await mockAirBridgeService.storeRestoreCheckpointMetadata(safeRequest());
    await renderPanel(safeRequest(), result);
    expect(screen.getByTestId("rcps-summary-boundary-count").textContent).toBe("3");
  });

  it("summary shows correct phase count", async () => {
    const result = await mockAirBridgeService.storeRestoreCheckpointMetadata(safeRequest());
    await renderPanel(safeRequest(), result);
    expect(screen.getByTestId("rcps-summary-phase-count").textContent).toBe("2");
  });

  it("summary shows correct item count", async () => {
    const result = await mockAirBridgeService.storeRestoreCheckpointMetadata(safeRequest());
    await renderPanel(safeRequest(), result);
    expect(screen.getByTestId("rcps-summary-item-count").textContent).toBe("23");
  });

  it("summary safe filename starts with rcps-", async () => {
    const result = await mockAirBridgeService.storeRestoreCheckpointMetadata(safeRequest());
    await renderPanel(safeRequest(), result);
    expect(screen.getByTestId("rcps-summary-safe-filename").textContent).toMatch(/^rcps-/);
  });

  it("summary safe filename ends with .json", async () => {
    const result = await mockAirBridgeService.storeRestoreCheckpointMetadata(safeRequest());
    await renderPanel(safeRequest(), result);
    expect(screen.getByTestId("rcps-summary-safe-filename").textContent).toMatch(/\.json$/);
  });

  it("summary safe filename has no path separator", async () => {
    const result = await mockAirBridgeService.storeRestoreCheckpointMetadata(safeRequest());
    await renderPanel(safeRequest(), result);
    const fn = screen.getByTestId("rcps-summary-safe-filename").textContent ?? "";
    expect(fn).not.toContain("/");
    expect(fn).not.toContain("\\");
  });

  it("shows writes-disabled badge in stored result", async () => {
    const result = await mockAirBridgeService.storeRestoreCheckpointMetadata(safeRequest());
    await renderPanel(safeRequest(), result);
    expect(screen.getByTestId("rcps-writes-disabled")).toBeInTheDocument();
    expect(screen.getByTestId("rcps-writes-disabled").textContent).toContain(
      "Restore writes disabled",
    );
  });

  // ── Blocked state ──────────────────────────────────────────────────────────

  it("shows blocked badge when result is blocked", async () => {
    const result = await mockAirBridgeService.storeRestoreCheckpointMetadata(blockedRequest());
    await renderPanel(blockedRequest(), result);
    expect(screen.getByTestId("rcps-blocked-badge")).toBeInTheDocument();
  });

  it("shows blocked reason when blocked", async () => {
    const result = await mockAirBridgeService.storeRestoreCheckpointMetadata(blockedRequest());
    await renderPanel(blockedRequest(), result);
    expect(screen.getByTestId("rcps-blocked-reason")).toBeInTheDocument();
  });

  it("does not show stored badge when blocked", async () => {
    const result = await mockAirBridgeService.storeRestoreCheckpointMetadata(blockedRequest());
    await renderPanel(blockedRequest(), result);
    expect(screen.queryByTestId("rcps-stored-badge")).not.toBeInTheDocument();
  });

  it("does not show summary when blocked", async () => {
    const result = await mockAirBridgeService.storeRestoreCheckpointMetadata(blockedRequest());
    await renderPanel(blockedRequest(), result);
    expect(screen.queryByTestId("rcps-summary")).not.toBeInTheDocument();
  });

  // ── Button interaction ────────────────────────────────────────────────────

  it("calls onStore when store button is clicked", async () => {
    const { onStore } = await renderPanel(safeRequest());
    fireEvent.click(screen.getByTestId("rcps-store-button"));
    await waitFor(() => expect(onStore).toHaveBeenCalledTimes(1));
  });

  it("passes request to onStore", async () => {
    const req = safeRequest();
    const { onStore } = await renderPanel(req);
    fireEvent.click(screen.getByTestId("rcps-store-button"));
    await waitFor(() => expect(onStore).toHaveBeenCalledWith(req));
  });

  // ── Safety invariants ─────────────────────────────────────────────────────

  it("writesEnabled is always false in stored result", async () => {
    const result = await mockAirBridgeService.storeRestoreCheckpointMetadata(safeRequest());
    expect(result.writesEnabled).toBe(false);
  });

  it("networkWritesAttempted is always false in stored result", async () => {
    const result = await mockAirBridgeService.storeRestoreCheckpointMetadata(safeRequest());
    expect(result.networkWritesAttempted).toBe(false);
  });

  it("noChangesMade is false in stored result", async () => {
    const result = await mockAirBridgeService.storeRestoreCheckpointMetadata(safeRequest());
    expect(result.noChangesMade).toBe(false);
  });

  it("writesEnabled is always false in blocked result", async () => {
    const result = await mockAirBridgeService.storeRestoreCheckpointMetadata(blockedRequest());
    expect(result.writesEnabled).toBe(false);
  });

  it("networkWritesAttempted is always false in blocked result", async () => {
    const result = await mockAirBridgeService.storeRestoreCheckpointMetadata(blockedRequest());
    expect(result.networkWritesAttempted).toBe(false);
  });

  it("noChangesMade is true in blocked result", async () => {
    const result = await mockAirBridgeService.storeRestoreCheckpointMetadata(blockedRequest());
    expect(result.noChangesMade).toBe(true);
  });

  it("stored result contains no token", async () => {
    const result = await mockAirBridgeService.storeRestoreCheckpointMetadata(safeRequest());
    const json = JSON.stringify(result);
    expect(json).not.toContain("pat_");
  });

  it("blocked result contains no token", async () => {
    const result = await mockAirBridgeService.storeRestoreCheckpointMetadata(blockedRequest());
    const json = JSON.stringify(result);
    expect(json).not.toContain("pat_");
  });

  it("stored result contains no full path", async () => {
    const result = await mockAirBridgeService.storeRestoreCheckpointMetadata(safeRequest());
    const json = JSON.stringify(result);
    expect(json).not.toContain("/Users/");
    expect(json).not.toContain("/home/");
    expect(json).not.toContain("/tmp/");
  });

  it("stored result contains no restore success state", async () => {
    const result = await mockAirBridgeService.storeRestoreCheckpointMetadata(safeRequest());
    const json = JSON.stringify(result);
    expect(json).not.toContain("restoreComplete");
    expect(json).not.toContain("restoreSuccess");
    expect(json).not.toContain('"succeeded"');
  });

  // ── Prerequisite cascade ──────────────────────────────────────────────────

  it("blocked when checkpointDurabilitySafe is false", async () => {
    const result = await mockAirBridgeService.storeRestoreCheckpointMetadata({
      ...safeRequest(),
      checkpointDurabilitySafe: false,
    });
    expect(result.status).toBe("blocked");
    expect(result.blockedReason).toMatch(/RCPS-PRE-02/);
  });

  it("blocked when sensitiveDataSafe is false", async () => {
    const result = await mockAirBridgeService.storeRestoreCheckpointMetadata({
      ...safeRequest(),
      sensitiveDataSafe: false,
    });
    expect(result.status).toBe("blocked");
    expect(result.blockedReason).toMatch(/RCPS-PRE-03/);
  });

  it("blocked when mappingCheckpointPreviewReady is false", async () => {
    const result = await mockAirBridgeService.storeRestoreCheckpointMetadata({
      ...safeRequest(),
      mappingCheckpointPreviewReady: false,
    });
    expect(result.status).toBe("blocked");
    expect(result.blockedReason).toMatch(/RCPS-PRE-04/);
  });

  it("blocked when finalValidationPreviewReady is false", async () => {
    const result = await mockAirBridgeService.storeRestoreCheckpointMetadata({
      ...safeRequest(),
      finalValidationPreviewReady: false,
    });
    expect(result.status).toBe("blocked");
    expect(result.blockedReason).toMatch(/RCPS-PRE-05/);
  });

  // ── No execute/enable button / no token input / no old/new IDs ───────────

  it("does not render an execute button", async () => {
    await renderPanel(safeRequest());
    const buttons = screen.queryAllByRole("button");
    for (const btn of buttons) {
      expect(btn.textContent?.toLowerCase()).not.toMatch(/execute/i);
      expect(btn.textContent?.toLowerCase()).not.toMatch(/enable/i);
    }
  });

  it("does not render a token input", async () => {
    await renderPanel(safeRequest());
    expect(screen.queryByRole("textbox", { name: /token/i })).not.toBeInTheDocument();
  });
});
