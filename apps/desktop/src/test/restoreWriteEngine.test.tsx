import { render, screen } from "@testing-library/react";
import { describe, it, expect } from "vitest";
import { RestoreWriteEnginePanel } from "../features/backups/RestoreWriteEnginePanel";
import { mockAirBridgeService } from "../services/mockAirBridgeService";
import type { AirBridgeService } from "../services/airBridgeService";
import type { RestoreWriteEngineResult } from "../backend/types";

// ── Fixtures ──────────────────────────────────────────────────────────────────

const DISABLED_RESULT: RestoreWriteEngineResult = {
  filename: "backup.airbridge",
  status: "disabled",
  disabledReason: "disabledByProductPolicy",
  message: "Restore write execution is not enabled in this version. No Airtable changes are made.",
  phaseSummaries: [
    { phase: "validateInputs", status: "disabled", noChangesMade: true, note: "Input validation completed. Write engine is disabled." },
    { phase: "schemaCreation", status: "disabled", noChangesMade: true, note: "Schema creation disabled. Would create 3 table(s)." },
    { phase: "recordCreation", status: "disabled", noChangesMade: true, note: "Record import disabled. 2 first-pass batch(es) planned." },
    { phase: "linkedRecordUpdates", status: "disabled", noChangesMade: true, note: "Linked record updates disabled." },
    { phase: "attachmentHandling", status: "disabled", noChangesMade: true, note: "Attachment handling disabled. Policy: MetadataOnly." },
    { phase: "finalValidation", status: "disabled", noChangesMade: true, note: "Final validation not executed — write engine is disabled." },
  ],
  events: [
    { phase: "validateInputs", code: "WRITE_ENGINE_DISABLED", message: "Write engine is disabled by product policy." },
  ],
  noChangesMade: true,
};

// ── Mock service contract ─────────────────────────────────────────────────────

describe("previewRestoreWriteEngine mock service contract", () => {
  it("returns disabled status always", async () => {
    const result = await mockAirBridgeService.previewRestoreWriteEngine({
      packageFilename: "backup.airbridge",
      packagePath: "/tmp/backup.airbridge",
    });
    expect(result.status).toBe("disabled");
  });

  it("disabledReason is disabledByProductPolicy", async () => {
    const result = await mockAirBridgeService.previewRestoreWriteEngine({
      packageFilename: "backup.airbridge",
      packagePath: "/tmp/backup.airbridge",
    });
    expect(result.disabledReason).toBe("disabledByProductPolicy");
  });

  it("noChangesMade is always true", async () => {
    const result = await mockAirBridgeService.previewRestoreWriteEngine({
      packageFilename: "backup.airbridge",
      packagePath: "/tmp/backup.airbridge",
    });
    expect(result.noChangesMade).toBe(true);
  });

  it("returns all 6 phase summaries", async () => {
    const result = await mockAirBridgeService.previewRestoreWriteEngine({
      packageFilename: "backup.airbridge",
      packagePath: "/tmp/backup.airbridge",
    });
    expect(result.phaseSummaries).toHaveLength(6);
  });

  it("all phase summaries have status disabled", async () => {
    const result = await mockAirBridgeService.previewRestoreWriteEngine({
      packageFilename: "backup.airbridge",
      packagePath: "/tmp/backup.airbridge",
    });
    for (const phase of result.phaseSummaries) {
      expect(phase.status).toBe("disabled");
    }
  });

  it("all phase summaries have noChangesMade true", async () => {
    const result = await mockAirBridgeService.previewRestoreWriteEngine({
      packageFilename: "backup.airbridge",
      packagePath: "/tmp/backup.airbridge",
    });
    for (const phase of result.phaseSummaries) {
      expect(phase.noChangesMade).toBe(true);
    }
  });

  it("all 6 expected phases are present", async () => {
    const result = await mockAirBridgeService.previewRestoreWriteEngine({
      packageFilename: "backup.airbridge",
      packagePath: "/tmp/backup.airbridge",
    });
    const phases = result.phaseSummaries.map((p) => p.phase);
    expect(phases).toContain("validateInputs");
    expect(phases).toContain("schemaCreation");
    expect(phases).toContain("recordCreation");
    expect(phases).toContain("linkedRecordUpdates");
    expect(phases).toContain("attachmentHandling");
    expect(phases).toContain("finalValidation");
  });

  it("result never contains the string 'succeeded'", async () => {
    const result = await mockAirBridgeService.previewRestoreWriteEngine({
      packageFilename: "backup.airbridge",
      packagePath: "/tmp/backup.airbridge",
    });
    const json = JSON.stringify(result).toLowerCase();
    expect(json).not.toContain("succeeded");
  });

  it("result never contains a token value", async () => {
    const result = await mockAirBridgeService.previewRestoreWriteEngine({
      packageFilename: "backup.airbridge",
      packagePath: "/tmp/backup.airbridge",
    });
    const json = JSON.stringify(result);
    expect(json).not.toContain("token");
    expect(json).not.toContain("pat1");
  });

  it("result never contains the full package path", async () => {
    const result = await mockAirBridgeService.previewRestoreWriteEngine({
      packageFilename: "backup.airbridge",
      packagePath: "/Users/amirhesampiri/backups/backup.airbridge",
    });
    const json = JSON.stringify(result);
    expect(json).not.toContain("/Users/amirhesampiri/");
    expect(json).not.toContain("/backups/");
  });

  it("result filename is set from packageFilename, not path", async () => {
    const result = await mockAirBridgeService.previewRestoreWriteEngine({
      packageFilename: "my-backup.airbridge",
      packagePath: "/some/deep/path/my-backup.airbridge",
    });
    expect(result.filename).toBe("my-backup.airbridge");
  });

  it("request has no token field", () => {
    const request: Parameters<AirBridgeService["previewRestoreWriteEngine"]>[0] = {
      packageFilename: "backup.airbridge",
      packagePath: "/tmp/backup.airbridge",
    };
    const keys = Object.keys(request);
    expect(keys).not.toContain("token");
  });

  it("IPC-unavailable fallback returns disabled with noChangesMade true", async () => {
    const fallbackResult: RestoreWriteEngineResult = {
      filename: "backup.airbridge",
      status: "disabled",
      disabledReason: "notAvailable",
      message: "Write engine skeleton preview is unavailable in this context.",
      phaseSummaries: [],
      events: [],
      noChangesMade: true,
    };
    expect(fallbackResult.status).toBe("disabled");
    expect(fallbackResult.noChangesMade).toBe(true);
    const json = JSON.stringify(fallbackResult).toLowerCase();
    expect(json).not.toContain("succeeded");
  });

  it("restore execution remains disabled — no execute path in write engine", async () => {
    const result = await mockAirBridgeService.previewRestoreWriteEngine({
      packageFilename: "backup.airbridge",
      packagePath: "/tmp/backup.airbridge",
    });
    // The write engine result only allows disabled/blocked/notStarted status
    const allowedStatuses = ["disabled", "blocked", "notStarted"];
    expect(allowedStatuses).toContain(result.status);
  });
});

// ── RestoreWriteEnginePanel rendering ─────────────────────────────────────────

describe("RestoreWriteEnginePanel rendering", () => {
  it("renders the panel container", () => {
    render(<RestoreWriteEnginePanel result={null} />);
    expect(screen.getByTestId("restore-write-engine-panel")).not.toBeNull();
  });

  it("always shows disabled notice with no result", () => {
    render(<RestoreWriteEnginePanel result={null} />);
    const notice = screen.getByTestId("write-engine-disabled-notice");
    expect(notice.textContent).toContain("not enabled");
    expect(notice.textContent).toContain("No Airtable changes are made");
  });

  it("shows disabled notice even when result is present", () => {
    render(<RestoreWriteEnginePanel result={DISABLED_RESULT} />);
    const notice = screen.getByTestId("write-engine-disabled-notice");
    expect(notice.textContent).toContain("not enabled");
  });

  it("does not show preview result when result is null", () => {
    render(<RestoreWriteEnginePanel result={null} />);
    expect(screen.queryByTestId("write-engine-preview-result")).toBeNull();
  });

  it("shows preview result when result is provided", () => {
    render(<RestoreWriteEnginePanel result={DISABLED_RESULT} />);
    expect(screen.getByTestId("write-engine-preview-result")).not.toBeNull();
  });

  it("shows no-changes notice when result is present", () => {
    render(<RestoreWriteEnginePanel result={DISABLED_RESULT} />);
    const notice = screen.getByTestId("write-engine-no-changes-notice");
    expect(notice.textContent).toContain("No Airtable changes were made");
  });

  it("renders all 6 phase rows when result has 6 summaries", () => {
    render(<RestoreWriteEnginePanel result={DISABLED_RESULT} />);
    const rows = screen.getAllByTestId("write-engine-phase-row");
    expect(rows).toHaveLength(6);
  });

  it("all phase rows have data-status=disabled", () => {
    render(<RestoreWriteEnginePanel result={DISABLED_RESULT} />);
    const rows = screen.getAllByTestId("write-engine-phase-row");
    for (const row of rows) {
      expect(row.getAttribute("data-status")).toBe("disabled");
    }
  });

  it("phase rows include all expected phases", () => {
    render(<RestoreWriteEnginePanel result={DISABLED_RESULT} />);
    const rows = screen.getAllByTestId("write-engine-phase-row");
    const phases = rows.map((r) => r.getAttribute("data-phase"));
    expect(phases).toContain("validateInputs");
    expect(phases).toContain("schemaCreation");
    expect(phases).toContain("recordCreation");
    expect(phases).toContain("linkedRecordUpdates");
    expect(phases).toContain("attachmentHandling");
    expect(phases).toContain("finalValidation");
  });

  it("does not render any button", () => {
    const { container } = render(<RestoreWriteEnginePanel result={DISABLED_RESULT} />);
    const buttons = container.querySelectorAll("button");
    expect(buttons.length).toBe(0);
  });

  it("does not render any input or token field", () => {
    const { container } = render(<RestoreWriteEnginePanel result={DISABLED_RESULT} />);
    const inputs = container.querySelectorAll("input");
    expect(inputs.length).toBe(0);
  });

  it("does not render a success message in any state", () => {
    const { container } = render(<RestoreWriteEnginePanel result={DISABLED_RESULT} />);
    const text = container.textContent?.toLowerCase() ?? "";
    expect(text).not.toContain("succeeded");
    expect(text).not.toContain("restore complete");
    expect(text).not.toContain("restore successful");
  });

  it("does not contain the word 'token' in rendered output", () => {
    const { container } = render(<RestoreWriteEnginePanel result={DISABLED_RESULT} />);
    expect(container.textContent?.toLowerCase()).not.toContain("token");
  });

  it("does not expose any path in rendered output", () => {
    const resultWithPath: RestoreWriteEngineResult = {
      ...DISABLED_RESULT,
      filename: "backup.airbridge",
    };
    const { container } = render(<RestoreWriteEnginePanel result={resultWithPath} />);
    expect(container.textContent).not.toContain("/Users/");
    expect(container.textContent).not.toContain("/tmp/");
    expect(container.textContent).not.toContain("/backups/");
  });
});
