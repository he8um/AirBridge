import { describe, it, expect, vi } from "vitest";
import { render, screen, fireEvent } from "@testing-library/react";
import { mockAirBridgeService } from "../services/mockAirBridgeService";
import type { WpoPhaseDeclaration, WritePhaseOrderingPolicyRequest } from "../backend/types";
import { RestoreWritePhaseOrderingPolicyPanel } from "../features/backups/RestoreWritePhaseOrderingPolicyPanel";

// ── Helpers ───────────────────────────────────────────────────────────────────

function canonicalPhases(): WpoPhaseDeclaration[] {
  return [
    { kind: "preflight", status: "completed" },
    { kind: "schemaCreate", status: "completed" },
    { kind: "schemaVerify", status: "completed" },
    { kind: "recordCreate", status: "completed" },
    { kind: "recordVerify", status: "completed" },
    { kind: "linkedRecordUpdate", status: "completed" },
    { kind: "linkedRecordVerify", status: "completed" },
    { kind: "attachmentMetadataVerify", status: "completed" },
    { kind: "finalValidation", status: "planned" },
  ];
}

function requestWithPhases(phases: WpoPhaseDeclaration[]): WritePhaseOrderingPolicyRequest {
  return { phases };
}

function requestNoPhases(): WritePhaseOrderingPolicyRequest {
  return { phases: undefined };
}

// ── Service contract tests ────────────────────────────────────────────────────

describe("WritePhaseOrderingPolicy service contract", () => {
  it("canonical phases returns compliant", async () => {
    const result = await mockAirBridgeService.verifyWritePhaseOrderingPolicy(
      requestWithPhases(canonicalPhases()),
    );
    expect(result.status).toBe("compliant");
  });

  it("no phases returns blocked", async () => {
    const result = await mockAirBridgeService.verifyWritePhaseOrderingPolicy(requestNoPhases());
    expect(result.status).toBe("blocked");
  });

  it("record_create active before schema_verify completed is blocked", async () => {
    const phases = canonicalPhases().map((p) =>
      p.kind === "schemaVerify" ? { ...p, status: "planned" as const } : p,
    );
    const result = await mockAirBridgeService.verifyWritePhaseOrderingPolicy(
      requestWithPhases(phases),
    );
    expect(result.status).toBe("blocked");
    const wpo05 = result.checks.find((c) => c.checkId === "WPO-05");
    expect(wpo05?.status).toBe("failed");
  });

  it("linked_record_update active before record_verify completed is blocked", async () => {
    const phases = canonicalPhases().map((p) =>
      p.kind === "recordVerify" ? { ...p, status: "planned" as const } : p,
    );
    const result = await mockAirBridgeService.verifyWritePhaseOrderingPolicy(
      requestWithPhases(phases),
    );
    expect(result.status).toBe("blocked");
    const wpo06 = result.checks.find((c) => c.checkId === "WPO-06");
    expect(wpo06?.status).toBe("failed");
  });

  it("final_validation active before linked_record_verify completed is blocked", async () => {
    const phases = canonicalPhases().map((p) => {
      if (p.kind === "linkedRecordVerify") return { ...p, status: "planned" as const };
      if (p.kind === "finalValidation") return { ...p, status: "ready" as const };
      return p;
    });
    const result = await mockAirBridgeService.verifyWritePhaseOrderingPolicy(
      requestWithPhases(phases),
    );
    expect(result.status).toBe("blocked");
    const wpo07 = result.checks.find((c) => c.checkId === "WPO-07");
    expect(wpo07?.status).toBe("failed");
  });

  it("upload/binary/download language in non-metadata phase is blocked", async () => {
    const phases = canonicalPhases().map((p) =>
      p.kind === "recordCreate" ? { ...p, skipReason: "attachment binary upload required" } : p,
    );
    const result = await mockAirBridgeService.verifyWritePhaseOrderingPolicy(
      requestWithPhases(phases),
    );
    expect(result.status).toBe("blocked");
    const wpo08 = result.checks.find((c) => c.checkId === "WPO-08");
    expect(wpo08?.status).toBe("failed");
  });

  it("binary download required in attachment_metadata_verify is blocked", async () => {
    const phases = canonicalPhases().map((p) =>
      p.kind === "attachmentMetadataVerify"
        ? { ...p, skipReason: "attachment binary download required" }
        : p,
    );
    const result = await mockAirBridgeService.verifyWritePhaseOrderingPolicy(
      requestWithPhases(phases),
    );
    expect(result.status).toBe("blocked");
    const wpo08 = result.checks.find((c) => c.checkId === "WPO-08");
    expect(wpo08?.status).toBe("failed");
  });

  it("upload required in attachment_metadata_verify is blocked", async () => {
    const phases = canonicalPhases().map((p) =>
      p.kind === "attachmentMetadataVerify"
        ? { ...p, skipReason: "upload required for this phase" }
        : p,
    );
    const result = await mockAirBridgeService.verifyWritePhaseOrderingPolicy(
      requestWithPhases(phases),
    );
    expect(result.status).toBe("blocked");
    const wpo08 = result.checks.find((c) => c.checkId === "WPO-08");
    expect(wpo08?.status).toBe("failed");
  });

  it("metadata-only files not downloaded in attachment_metadata_verify is not blocked by WPO-08", async () => {
    // Safe descriptive language — describes scope, not a demand for binary handling
    const phases = canonicalPhases().map((p) =>
      p.kind === "attachmentMetadataVerify"
        ? { ...p, status: "skipped" as const, skipReason: "metadata-only: files not downloaded" }
        : p,
    );
    const result = await mockAirBridgeService.verifyWritePhaseOrderingPolicy(
      requestWithPhases(phases),
    );
    // WPO-08 must pass
    const wpo08 = result.checks.find((c) => c.checkId === "WPO-08");
    expect(wpo08?.status).toBe("passed");
    // Overall warning from WPO-09, not blocked
    expect(result.status).toBe("warning");
    const wpo09 = result.checks.find((c) => c.checkId === "WPO-09");
    expect(wpo09?.status).toBe("warning");
  });

  it("metadata-only skipped attachment_metadata_verify returns warning", async () => {
    const phases = canonicalPhases().map((p) =>
      p.kind === "attachmentMetadataVerify"
        ? { ...p, status: "skipped" as const, skipReason: "metadata-only: files not downloaded" }
        : p,
    );
    const result = await mockAirBridgeService.verifyWritePhaseOrderingPolicy(
      requestWithPhases(phases),
    );
    expect(result.status).toBe("warning");
    const wpo09 = result.checks.find((c) => c.checkId === "WPO-09");
    expect(wpo09?.status).toBe("warning");
  });

  it("skipped attachment_metadata_verify without metadata reason returns warning", async () => {
    const phases = canonicalPhases().map((p) =>
      p.kind === "attachmentMetadataVerify"
        ? { ...p, status: "skipped" as const, skipReason: undefined }
        : p,
    );
    const result = await mockAirBridgeService.verifyWritePhaseOrderingPolicy(
      requestWithPhases(phases),
    );
    expect(result.status).toBe("warning");
  });

  it("out-of-order phases are blocked", async () => {
    const phases: WpoPhaseDeclaration[] = [
      { kind: "preflight", status: "completed" },
      { kind: "linkedRecordUpdate", status: "planned" },
      { kind: "recordCreate", status: "planned" },
    ];
    const result = await mockAirBridgeService.verifyWritePhaseOrderingPolicy(
      requestWithPhases(phases),
    );
    expect(result.status).toBe("blocked");
    const wpo03 = result.checks.find((c) => c.checkId === "WPO-03");
    expect(wpo03?.status).toBe("failed");
  });

  it("canonical phases produce 10 checks", async () => {
    const result = await mockAirBridgeService.verifyWritePhaseOrderingPolicy(
      requestWithPhases(canonicalPhases()),
    );
    expect(result.checks.length).toBe(10);
  });

  it("no phases produce 2 checks", async () => {
    const result = await mockAirBridgeService.verifyWritePhaseOrderingPolicy(requestNoPhases());
    expect(result.checks.length).toBe(2);
  });

  it("check IDs are WPO-01 through WPO-10", async () => {
    const result = await mockAirBridgeService.verifyWritePhaseOrderingPolicy(
      requestWithPhases(canonicalPhases()),
    );
    const ids = result.checks.map((c) => c.checkId);
    expect(ids).toEqual([
      "WPO-01",
      "WPO-02",
      "WPO-03",
      "WPO-04",
      "WPO-05",
      "WPO-06",
      "WPO-07",
      "WPO-08",
      "WPO-09",
      "WPO-10",
    ]);
  });

  it("no-phases check IDs are WPO-01 and WPO-02", async () => {
    const result = await mockAirBridgeService.verifyWritePhaseOrderingPolicy(requestNoPhases());
    const ids = result.checks.map((c) => c.checkId);
    expect(ids).toEqual(["WPO-01", "WPO-02"]);
  });

  it("WPO-01 always passes with phases", async () => {
    const result = await mockAirBridgeService.verifyWritePhaseOrderingPolicy(
      requestWithPhases(canonicalPhases()),
    );
    const wpo01 = result.checks.find((c) => c.checkId === "WPO-01");
    expect(wpo01?.status).toBe("passed");
  });

  it("WPO-01 always passes without phases", async () => {
    const result = await mockAirBridgeService.verifyWritePhaseOrderingPolicy(requestNoPhases());
    const wpo01 = result.checks.find((c) => c.checkId === "WPO-01");
    expect(wpo01?.status).toBe("passed");
  });

  it("WPO-10 always passes", async () => {
    const result = await mockAirBridgeService.verifyWritePhaseOrderingPolicy(
      requestWithPhases(canonicalPhases()),
    );
    const wpo10 = result.checks.find((c) => c.checkId === "WPO-10");
    expect(wpo10?.status).toBe("passed");
  });

  it("phase summary present when phases provided", async () => {
    const result = await mockAirBridgeService.verifyWritePhaseOrderingPolicy(
      requestWithPhases(canonicalPhases()),
    );
    expect(result.phaseSummary).toBeDefined();
    expect(result.phaseSummary!.length).toBe(9);
  });

  it("phase summary absent when no phases", async () => {
    const result = await mockAirBridgeService.verifyWritePhaseOrderingPolicy(requestNoPhases());
    expect(result.phaseSummary).toBeUndefined();
  });

  it("noChangesMade is always true when compliant", async () => {
    const result = await mockAirBridgeService.verifyWritePhaseOrderingPolicy(
      requestWithPhases(canonicalPhases()),
    );
    expect(result.noChangesMade).toBe(true);
  });

  it("noChangesMade is always true when blocked", async () => {
    const result = await mockAirBridgeService.verifyWritePhaseOrderingPolicy(requestNoPhases());
    expect(result.noChangesMade).toBe(true);
  });

  it("networkWritesAttempted is always false", async () => {
    const result = await mockAirBridgeService.verifyWritePhaseOrderingPolicy(
      requestWithPhases(canonicalPhases()),
    );
    expect(result.networkWritesAttempted).toBe(false);
  });

  it("writesEnabled is always false when compliant", async () => {
    const result = await mockAirBridgeService.verifyWritePhaseOrderingPolicy(
      requestWithPhases(canonicalPhases()),
    );
    expect(result.writesEnabled).toBe(false);
  });

  it("writesEnabled is always false when blocked", async () => {
    const result = await mockAirBridgeService.verifyWritePhaseOrderingPolicy(requestNoPhases());
    expect(result.writesEnabled).toBe(false);
  });

  it("compliant message contains 'disabled'", async () => {
    const result = await mockAirBridgeService.verifyWritePhaseOrderingPolicy(
      requestWithPhases(canonicalPhases()),
    );
    expect(result.status).toBe("compliant");
    expect(result.message).toMatch(/disabled/i);
  });

  it("compliant message does not contain 'succeeded'", async () => {
    const result = await mockAirBridgeService.verifyWritePhaseOrderingPolicy(
      requestWithPhases(canonicalPhases()),
    );
    expect(result.message.toLowerCase()).not.toContain("succeeded");
  });

  it("message does not contain token patterns", async () => {
    const result = await mockAirBridgeService.verifyWritePhaseOrderingPolicy(
      requestWithPhases(canonicalPhases()),
    );
    expect(result.message).not.toMatch(/pat_/);
    expect(result.message).not.toMatch(/apiKey/);
  });

  it("message does not contain absolute path patterns", async () => {
    const result = await mockAirBridgeService.verifyWritePhaseOrderingPolicy(
      requestWithPhases(canonicalPhases()),
    );
    expect(result.message).not.toMatch(/\/Users\//);
    expect(result.message).not.toMatch(/\/home\//);
  });

  it("active dependent without prerequisite declared is blocked", async () => {
    // linked_record_update active but record_verify not declared at all
    const phases: WpoPhaseDeclaration[] = [
      { kind: "preflight", status: "completed" },
      { kind: "schemaCreate", status: "completed" },
      { kind: "schemaVerify", status: "completed" },
      { kind: "recordCreate", status: "completed" },
      { kind: "linkedRecordUpdate", status: "ready" },
    ];
    const result = await mockAirBridgeService.verifyWritePhaseOrderingPolicy(
      requestWithPhases(phases),
    );
    expect(result.status).toBe("blocked");
    const wpo04 = result.checks.find((c) => c.checkId === "WPO-04");
    expect(wpo04?.status).toBe("failed");
  });
});

// ── UI panel tests ────────────────────────────────────────────────────────────

describe("RestoreWritePhaseOrderingPolicyPanel", () => {
  it("renders with no result", () => {
    render(
      <RestoreWritePhaseOrderingPolicyPanel result={null} loading={false} onVerify={vi.fn()} />,
    );
    expect(screen.getByTestId("restore-wpo-panel")).toBeTruthy();
  });

  it("writes-disabled notice is always visible", () => {
    render(
      <RestoreWritePhaseOrderingPolicyPanel result={null} loading={false} onVerify={vi.fn()} />,
    );
    expect(screen.getByTestId("wpo-writes-disabled-notice")).toBeTruthy();
  });

  it("verify button is present", () => {
    render(
      <RestoreWritePhaseOrderingPolicyPanel result={null} loading={false} onVerify={vi.fn()} />,
    );
    expect(screen.getByTestId("wpo-verify-button")).toBeTruthy();
  });

  it("calls onVerify when button clicked", () => {
    const onVerify = vi.fn();
    render(
      <RestoreWritePhaseOrderingPolicyPanel result={null} loading={false} onVerify={onVerify} />,
    );
    fireEvent.click(screen.getByTestId("wpo-verify-button"));
    expect(onVerify).toHaveBeenCalledOnce();
  });

  it("button is disabled when loading", () => {
    render(
      <RestoreWritePhaseOrderingPolicyPanel result={null} loading={true} onVerify={vi.fn()} />,
    );
    expect((screen.getByTestId("wpo-verify-button") as HTMLButtonElement).disabled).toBe(true);
  });

  it("shows no result section when result is null", () => {
    render(
      <RestoreWritePhaseOrderingPolicyPanel result={null} loading={false} onVerify={vi.fn()} />,
    );
    expect(screen.queryByTestId("wpo-result")).toBeNull();
  });

  it("shows compliant badge for compliant result", async () => {
    const result = await mockAirBridgeService.verifyWritePhaseOrderingPolicy(
      requestWithPhases(canonicalPhases()),
    );
    render(
      <RestoreWritePhaseOrderingPolicyPanel result={result} loading={false} onVerify={vi.fn()} />,
    );
    expect(screen.getByTestId("wpo-compliant-badge")).toBeTruthy();
    expect(screen.queryByTestId("wpo-warning-badge")).toBeNull();
    expect(screen.queryByTestId("wpo-blocked-badge")).toBeNull();
  });

  it("shows warning badge for warning result", async () => {
    const phases = canonicalPhases().map((p) =>
      p.kind === "attachmentMetadataVerify"
        ? { ...p, status: "skipped" as const, skipReason: "metadata-only" }
        : p,
    );
    const result = await mockAirBridgeService.verifyWritePhaseOrderingPolicy(
      requestWithPhases(phases),
    );
    render(
      <RestoreWritePhaseOrderingPolicyPanel result={result} loading={false} onVerify={vi.fn()} />,
    );
    expect(screen.getByTestId("wpo-warning-badge")).toBeTruthy();
    expect(screen.queryByTestId("wpo-compliant-badge")).toBeNull();
    expect(screen.queryByTestId("wpo-blocked-badge")).toBeNull();
  });

  it("shows blocked badge for blocked result", async () => {
    const result = await mockAirBridgeService.verifyWritePhaseOrderingPolicy(requestNoPhases());
    render(
      <RestoreWritePhaseOrderingPolicyPanel result={result} loading={false} onVerify={vi.fn()} />,
    );
    expect(screen.getByTestId("wpo-blocked-badge")).toBeTruthy();
    expect(screen.queryByTestId("wpo-compliant-badge")).toBeNull();
    expect(screen.queryByTestId("wpo-warning-badge")).toBeNull();
  });

  it("shows 10 check rows for canonical phases", async () => {
    const result = await mockAirBridgeService.verifyWritePhaseOrderingPolicy(
      requestWithPhases(canonicalPhases()),
    );
    render(
      <RestoreWritePhaseOrderingPolicyPanel result={result} loading={false} onVerify={vi.fn()} />,
    );
    const rows = screen.getAllByTestId("wpo-check-row");
    expect(rows.length).toBe(10);
  });

  it("shows 9 phase rows in phase summary", async () => {
    const result = await mockAirBridgeService.verifyWritePhaseOrderingPolicy(
      requestWithPhases(canonicalPhases()),
    );
    render(
      <RestoreWritePhaseOrderingPolicyPanel result={result} loading={false} onVerify={vi.fn()} />,
    );
    const rows = screen.getAllByTestId("wpo-phase-row");
    expect(rows.length).toBe(9);
  });

  it("shows safety summary", async () => {
    const result = await mockAirBridgeService.verifyWritePhaseOrderingPolicy(
      requestWithPhases(canonicalPhases()),
    );
    render(
      <RestoreWritePhaseOrderingPolicyPanel result={result} loading={false} onVerify={vi.fn()} />,
    );
    expect(screen.getByTestId("wpo-safety-summary")).toBeTruthy();
    expect(screen.getByTestId("wpo-no-changes-notice")).toBeTruthy();
  });

  it("shows compliant notice for compliant result", async () => {
    const result = await mockAirBridgeService.verifyWritePhaseOrderingPolicy(
      requestWithPhases(canonicalPhases()),
    );
    render(
      <RestoreWritePhaseOrderingPolicyPanel result={result} loading={false} onVerify={vi.fn()} />,
    );
    expect(screen.getByTestId("wpo-compliant-notice")).toBeTruthy();
    expect(screen.queryByTestId("wpo-warning-notice")).toBeNull();
    expect(screen.queryByTestId("wpo-blocked-notice")).toBeNull();
  });

  it("shows warning notice for warning result", async () => {
    const phases = canonicalPhases().map((p) =>
      p.kind === "attachmentMetadataVerify"
        ? { ...p, status: "skipped" as const, skipReason: "metadata-only" }
        : p,
    );
    const result = await mockAirBridgeService.verifyWritePhaseOrderingPolicy(
      requestWithPhases(phases),
    );
    render(
      <RestoreWritePhaseOrderingPolicyPanel result={result} loading={false} onVerify={vi.fn()} />,
    );
    expect(screen.getByTestId("wpo-warning-notice")).toBeTruthy();
    expect(screen.queryByTestId("wpo-compliant-notice")).toBeNull();
    expect(screen.queryByTestId("wpo-blocked-notice")).toBeNull();
  });

  it("shows blocked notice for blocked result", async () => {
    const result = await mockAirBridgeService.verifyWritePhaseOrderingPolicy(requestNoPhases());
    render(
      <RestoreWritePhaseOrderingPolicyPanel result={result} loading={false} onVerify={vi.fn()} />,
    );
    expect(screen.getByTestId("wpo-blocked-notice")).toBeTruthy();
    expect(screen.queryByTestId("wpo-compliant-notice")).toBeNull();
    expect(screen.queryByTestId("wpo-warning-notice")).toBeNull();
  });

  it("does not contain token input", () => {
    render(
      <RestoreWritePhaseOrderingPolicyPanel result={null} loading={false} onVerify={vi.fn()} />,
    );
    expect(screen.queryByLabelText(/token/i)).toBeNull();
    expect(screen.queryByPlaceholderText(/pat_/i)).toBeNull();
  });

  it("does not contain execute button", () => {
    render(
      <RestoreWritePhaseOrderingPolicyPanel result={null} loading={false} onVerify={vi.fn()} />,
    );
    const buttons = screen.queryAllByRole("button");
    for (const btn of buttons) {
      expect(btn.textContent?.toLowerCase()).not.toMatch(/execute|start restore|run restore/);
    }
  });

  it("does not contain succeeded language", async () => {
    const result = await mockAirBridgeService.verifyWritePhaseOrderingPolicy(
      requestWithPhases(canonicalPhases()),
    );
    render(
      <RestoreWritePhaseOrderingPolicyPanel result={result} loading={false} onVerify={vi.fn()} />,
    );
    const panel = screen.getByTestId("restore-wpo-panel");
    expect(panel.textContent?.toLowerCase()).not.toContain("succeeded");
  });

  it("writes-disabled notice always visible with result", async () => {
    const result = await mockAirBridgeService.verifyWritePhaseOrderingPolicy(
      requestWithPhases(canonicalPhases()),
    );
    render(
      <RestoreWritePhaseOrderingPolicyPanel result={result} loading={false} onVerify={vi.fn()} />,
    );
    expect(screen.getByTestId("wpo-writes-disabled-notice")).toBeTruthy();
  });
});
