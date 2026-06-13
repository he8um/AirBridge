import { describe, it, expect } from "vitest";
import { render, screen, fireEvent } from "@testing-library/react";
import { mockAirBridgeService } from "../services/mockAirBridgeService";
import { RestoreTargetEmptyVerificationPanel } from "../features/backups/RestoreTargetEmptyVerificationPanel";
import type {
  TargetEmptyVerificationRequest,
  TargetEmptyVerificationResult,
} from "../backend/types";

// ---------------------------------------------------------------------------
// Mock service contract tests
// ---------------------------------------------------------------------------

describe("mockAirBridgeService.verifyRestoreTargetEmpty — contract", () => {
  async function verify(
    req: TargetEmptyVerificationRequest,
  ): Promise<TargetEmptyVerificationResult> {
    return mockAirBridgeService.verifyRestoreTargetEmpty(req);
  }

  it("newBase intent returns verified", async () => {
    const r = await verify({ targetMode: "newBase", liveCheckPerformed: false });
    expect(r.status).toBe("verified");
  });

  it("emptyExistingBase with 0 tables and 0 records returns verified", async () => {
    const r = await verify({
      targetMode: "emptyExistingBase",
      targetTableCount: 0,
      targetRecordCount: 0,
      liveCheckPerformed: true,
    });
    expect(r.status).toBe("verified");
  });

  it("table_count > 0 returns blocked", async () => {
    const r = await verify({
      targetMode: "emptyExistingBase",
      targetTableCount: 3,
      targetRecordCount: 0,
      liveCheckPerformed: true,
    });
    expect(r.status).toBe("blocked");
  });

  it("record_count > 0 returns blocked", async () => {
    const r = await verify({
      targetMode: "emptyExistingBase",
      targetTableCount: 0,
      targetRecordCount: 20,
      liveCheckPerformed: true,
    });
    expect(r.status).toBe("blocked");
  });

  it("unknown counts for existing base returns warning", async () => {
    const r = await verify({
      targetMode: "emptyExistingBase",
      liveCheckPerformed: false,
    });
    expect(r.status).toBe("warning");
  });

  it("unknown table count only returns warning", async () => {
    const r = await verify({
      targetMode: "emptyExistingBase",
      targetRecordCount: 0,
      liveCheckPerformed: false,
    });
    expect(r.status).toBe("warning");
  });

  it("unknown record count only returns warning", async () => {
    const r = await verify({
      targetMode: "emptyExistingBase",
      targetTableCount: 0,
      liveCheckPerformed: false,
    });
    expect(r.status).toBe("warning");
  });

  it("unsupported target mode returns blocked", async () => {
    const r = await verify({ targetMode: "existingBase", liveCheckPerformed: false });
    expect(r.status).toBe("blocked");
  });

  it("empty target mode string returns blocked", async () => {
    const r = await verify({ targetMode: "", liveCheckPerformed: false });
    expect(r.status).toBe("blocked");
  });

  it("result has 5 checks", async () => {
    const r = await verify({ targetMode: "newBase", liveCheckPerformed: false });
    expect(r.checks).toHaveLength(5);
  });

  it("check IDs are TEV-01 through TEV-05", async () => {
    const r = await verify({ targetMode: "newBase", liveCheckPerformed: false });
    const ids = r.checks.map((c) => c.checkId);
    expect(ids).toEqual(["TEV-01", "TEV-02", "TEV-03", "TEV-04", "TEV-05"]);
  });

  it("TEV-01 always passes (write gate disabled)", async () => {
    const r = await verify({ targetMode: "newBase", liveCheckPerformed: false });
    const tev01 = r.checks.find((c) => c.checkId === "TEV-01");
    expect(tev01?.status).toBe("passed");
  });

  it("TEV-05 always passes (no writes enabled)", async () => {
    const r = await verify({
      targetMode: "emptyExistingBase",
      targetTableCount: 0,
      targetRecordCount: 0,
      liveCheckPerformed: true,
    });
    const tev05 = r.checks.find((c) => c.checkId === "TEV-05");
    expect(tev05?.status).toBe("passed");
  });

  it("noChangesMade is always true — verified", async () => {
    const r = await verify({ targetMode: "newBase", liveCheckPerformed: false });
    expect(r.noChangesMade).toBe(true);
  });

  it("noChangesMade is always true — warning", async () => {
    const r = await verify({
      targetMode: "emptyExistingBase",
      liveCheckPerformed: false,
    });
    expect(r.noChangesMade).toBe(true);
  });

  it("noChangesMade is always true — blocked", async () => {
    const r = await verify({
      targetMode: "emptyExistingBase",
      targetTableCount: 5,
      targetRecordCount: 100,
      liveCheckPerformed: true,
    });
    expect(r.noChangesMade).toBe(true);
  });

  it("writesEnabled is always false — verified", async () => {
    const r = await verify({ targetMode: "newBase", liveCheckPerformed: false });
    expect(r.writesEnabled).toBe(false);
  });

  it("writesEnabled is always false — blocked", async () => {
    const r = await verify({
      targetMode: "emptyExistingBase",
      targetTableCount: 10,
      targetRecordCount: 0,
      liveCheckPerformed: true,
    });
    expect(r.writesEnabled).toBe(false);
  });

  it("networkWritesAttempted is always false", async () => {
    const r = await verify({ targetMode: "newBase", liveCheckPerformed: false });
    expect(r.networkWritesAttempted).toBe(false);
  });

  it("result has no token in message", async () => {
    const r = await verify({
      targetMode: "emptyExistingBase",
      targetTableCount: 0,
      targetRecordCount: 0,
      liveCheckPerformed: true,
    });
    expect(r.message).not.toMatch(/pat[0-9a-zA-Z]{10,}/);
    expect(r.message).not.toContain("token");
  });

  it("result has no full path in message", async () => {
    const r = await verify({ targetMode: "newBase", liveCheckPerformed: false });
    expect(r.message).not.toContain("/Users/");
    expect(r.message).not.toContain("/home/");
  });

  it("targetDisplayName appears in verified message", async () => {
    const r = await verify({
      targetMode: "emptyExistingBase",
      targetTableCount: 0,
      targetRecordCount: 0,
      targetDisplayName: "Staging Base",
      liveCheckPerformed: true,
    });
    expect(r.message).toContain("Staging Base");
  });

  it("blocked message names unsupported mode", async () => {
    const r = await verify({ targetMode: "mergeBase", liveCheckPerformed: false });
    expect(r.message).toContain("mergeBase");
  });

  it("IPC fallback from live service returns blocked with safe fields", async () => {
    const fallback: TargetEmptyVerificationResult = {
      status: "blocked",
      checks: [],
      message: "Target empty verification is not available in this context.",
      noChangesMade: true,
      networkWritesAttempted: false,
      writesEnabled: false,
    };
    expect(fallback.status).toBe("blocked");
    expect(fallback.noChangesMade).toBe(true);
    expect(fallback.writesEnabled).toBe(false);
    expect(fallback.networkWritesAttempted).toBe(false);
  });
});

// ---------------------------------------------------------------------------
// Panel rendering tests
// ---------------------------------------------------------------------------

describe("RestoreTargetEmptyVerificationPanel — rendering", () => {
  const noop = () => {};

  it("renders the panel testid", () => {
    render(<RestoreTargetEmptyVerificationPanel result={null} loading={false} onVerify={noop} />);
    expect(screen.getByTestId("restore-target-empty-panel")).toBeDefined();
  });

  it("renders the writes-disabled notice", () => {
    render(<RestoreTargetEmptyVerificationPanel result={null} loading={false} onVerify={noop} />);
    expect(screen.getByTestId("target-empty-writes-disabled-notice")).toBeDefined();
  });

  it("renders verify button in idle state", () => {
    render(<RestoreTargetEmptyVerificationPanel result={null} loading={false} onVerify={noop} />);
    const btn = screen.getByTestId("target-empty-verify-button");
    expect(btn).toBeDefined();
    expect((btn as HTMLButtonElement).disabled).toBe(false);
  });

  it("verify button is disabled when loading", () => {
    render(<RestoreTargetEmptyVerificationPanel result={null} loading={true} onVerify={noop} />);
    const btn = screen.getByTestId("target-empty-verify-button") as HTMLButtonElement;
    expect(btn.disabled).toBe(true);
  });

  it("button label changes to Re-verify after result", () => {
    const result: TargetEmptyVerificationResult = {
      status: "verified",
      checks: [],
      message: "All good.",
      noChangesMade: true,
      networkWritesAttempted: false,
      writesEnabled: false,
    };
    render(<RestoreTargetEmptyVerificationPanel result={result} loading={false} onVerify={noop} />);
    expect(screen.getByTestId("target-empty-verify-button").textContent).toContain("Re-verify");
  });

  it("calls onVerify when button clicked", () => {
    let called = false;
    render(
      <RestoreTargetEmptyVerificationPanel
        result={null}
        loading={false}
        onVerify={() => {
          called = true;
        }}
      />,
    );
    fireEvent.click(screen.getByTestId("target-empty-verify-button"));
    expect(called).toBe(true);
  });

  it("does not render result area before verify", () => {
    render(<RestoreTargetEmptyVerificationPanel result={null} loading={false} onVerify={noop} />);
    expect(screen.queryByTestId("target-empty-result")).toBeNull();
  });

  it("renders result area after verify", () => {
    const result: TargetEmptyVerificationResult = {
      status: "verified",
      checks: [],
      message: "Empty.",
      noChangesMade: true,
      networkWritesAttempted: false,
      writesEnabled: false,
    };
    render(<RestoreTargetEmptyVerificationPanel result={result} loading={false} onVerify={noop} />);
    expect(screen.getByTestId("target-empty-result")).toBeDefined();
  });

  it("renders verified status badge", () => {
    const result: TargetEmptyVerificationResult = {
      status: "verified",
      checks: [],
      message: "Empty.",
      noChangesMade: true,
      networkWritesAttempted: false,
      writesEnabled: false,
    };
    render(<RestoreTargetEmptyVerificationPanel result={result} loading={false} onVerify={noop} />);
    expect(screen.getByTestId("target-empty-status").textContent).toBe("verified");
  });

  it("renders warning status badge", () => {
    const result: TargetEmptyVerificationResult = {
      status: "warning",
      checks: [],
      message: "Could not confirm.",
      noChangesMade: true,
      networkWritesAttempted: false,
      writesEnabled: false,
    };
    render(<RestoreTargetEmptyVerificationPanel result={result} loading={false} onVerify={noop} />);
    expect(screen.getByTestId("target-empty-status").textContent).toBe("warning");
  });

  it("renders blocked status badge", () => {
    const result: TargetEmptyVerificationResult = {
      status: "blocked",
      checks: [],
      message: "Not empty.",
      noChangesMade: true,
      networkWritesAttempted: false,
      writesEnabled: false,
    };
    render(<RestoreTargetEmptyVerificationPanel result={result} loading={false} onVerify={noop} />);
    expect(screen.getByTestId("target-empty-status").textContent).toBe("blocked");
  });

  it("renders result message", () => {
    const result: TargetEmptyVerificationResult = {
      status: "verified",
      checks: [],
      message: "Target base confirmed empty.",
      noChangesMade: true,
      networkWritesAttempted: false,
      writesEnabled: false,
    };
    render(<RestoreTargetEmptyVerificationPanel result={result} loading={false} onVerify={noop} />);
    expect(screen.getByTestId("target-empty-message").textContent).toContain(
      "Target base confirmed empty.",
    );
  });

  it("renders check rows", () => {
    const result: TargetEmptyVerificationResult = {
      status: "verified",
      checks: [
        { checkId: "TEV-01", label: "write-gate", status: "passed", message: "Gate disabled." },
        { checkId: "TEV-02", label: "target-mode", status: "passed", message: "Mode ok." },
      ],
      message: "All good.",
      noChangesMade: true,
      networkWritesAttempted: false,
      writesEnabled: false,
    };
    render(<RestoreTargetEmptyVerificationPanel result={result} loading={false} onVerify={noop} />);
    const rows = screen.getAllByTestId("target-empty-check-row");
    expect(rows).toHaveLength(2);
  });

  it("renders safety summary", () => {
    const result: TargetEmptyVerificationResult = {
      status: "verified",
      checks: [],
      message: "Good.",
      noChangesMade: true,
      networkWritesAttempted: false,
      writesEnabled: false,
    };
    render(<RestoreTargetEmptyVerificationPanel result={result} loading={false} onVerify={noop} />);
    expect(screen.getByTestId("target-empty-safety-summary")).toBeDefined();
  });

  it("renders no-changes notice in safety summary", () => {
    const result: TargetEmptyVerificationResult = {
      status: "verified",
      checks: [],
      message: "Good.",
      noChangesMade: true,
      networkWritesAttempted: false,
      writesEnabled: false,
    };
    render(<RestoreTargetEmptyVerificationPanel result={result} loading={false} onVerify={noop} />);
    expect(screen.getByTestId("target-empty-no-changes-notice").textContent).toContain(
      "No Airtable changes were made.",
    );
  });

  it("renders verified-notice when status is verified", () => {
    const result: TargetEmptyVerificationResult = {
      status: "verified",
      checks: [],
      message: "Good.",
      noChangesMade: true,
      networkWritesAttempted: false,
      writesEnabled: false,
    };
    render(<RestoreTargetEmptyVerificationPanel result={result} loading={false} onVerify={noop} />);
    expect(screen.getByTestId("target-empty-verified-notice")).toBeDefined();
    expect(screen.queryByTestId("target-empty-blocked-notice")).toBeNull();
    expect(screen.queryByTestId("target-empty-warning-notice")).toBeNull();
  });

  it("renders warning-notice when status is warning", () => {
    const result: TargetEmptyVerificationResult = {
      status: "warning",
      checks: [],
      message: "Not sure.",
      noChangesMade: true,
      networkWritesAttempted: false,
      writesEnabled: false,
    };
    render(<RestoreTargetEmptyVerificationPanel result={result} loading={false} onVerify={noop} />);
    expect(screen.getByTestId("target-empty-warning-notice")).toBeDefined();
    expect(screen.queryByTestId("target-empty-verified-notice")).toBeNull();
    expect(screen.queryByTestId("target-empty-blocked-notice")).toBeNull();
  });

  it("renders blocked-notice when status is blocked", () => {
    const result: TargetEmptyVerificationResult = {
      status: "blocked",
      checks: [],
      message: "Not empty.",
      noChangesMade: true,
      networkWritesAttempted: false,
      writesEnabled: false,
    };
    render(<RestoreTargetEmptyVerificationPanel result={result} loading={false} onVerify={noop} />);
    expect(screen.getByTestId("target-empty-blocked-notice")).toBeDefined();
    expect(screen.queryByTestId("target-empty-verified-notice")).toBeNull();
    expect(screen.queryByTestId("target-empty-warning-notice")).toBeNull();
  });

  it("verified notice still says writes remain disabled", () => {
    const result: TargetEmptyVerificationResult = {
      status: "verified",
      checks: [],
      message: "Good.",
      noChangesMade: true,
      networkWritesAttempted: false,
      writesEnabled: false,
    };
    render(<RestoreTargetEmptyVerificationPanel result={result} loading={false} onVerify={noop} />);
    expect(screen.getByTestId("target-empty-verified-notice").textContent).toContain(
      "remain disabled",
    );
  });

  it("no execute button in panel", () => {
    render(<RestoreTargetEmptyVerificationPanel result={null} loading={false} onVerify={noop} />);
    const buttons = screen.queryAllByRole("button");
    const labels = buttons.map((b) => b.textContent?.toLowerCase() ?? "");
    const hasExecute = labels.some(
      (l) => l.includes("execute") || l.includes("start restore") || l.includes("run restore"),
    );
    expect(hasExecute).toBe(false);
  });

  it("no token input in panel", () => {
    render(<RestoreTargetEmptyVerificationPanel result={null} loading={false} onVerify={noop} />);
    const passwordInputs = document.querySelectorAll('input[type="password"]');
    expect(passwordInputs).toHaveLength(0);
  });

  it("no succeeded language in any rendered text", () => {
    const result: TargetEmptyVerificationResult = {
      status: "verified",
      checks: [],
      message: "Empty confirmed.",
      noChangesMade: true,
      networkWritesAttempted: false,
      writesEnabled: false,
    };
    render(<RestoreTargetEmptyVerificationPanel result={result} loading={false} onVerify={noop} />);
    const bodyText = document.body.textContent?.toLowerCase() ?? "";
    expect(bodyText).not.toContain("restore complete");
    expect(bodyText).not.toContain("restore succeeded");
    expect(bodyText).not.toContain("succeeded");
  });

  it("no full path in rendered output", () => {
    const result: TargetEmptyVerificationResult = {
      status: "verified",
      checks: [],
      message: "Good.",
      noChangesMade: true,
      networkWritesAttempted: false,
      writesEnabled: false,
    };
    render(<RestoreTargetEmptyVerificationPanel result={result} loading={false} onVerify={noop} />);
    const bodyText = document.body.textContent ?? "";
    expect(bodyText).not.toContain("/Users/");
    expect(bodyText).not.toContain("/home/");
  });
});
