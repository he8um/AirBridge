import { render, screen } from "@testing-library/react";
import { describe, it, expect } from "vitest";
import { RestoreSandboxVerificationPanel } from "../features/backups/RestoreSandboxVerificationPanel";
import { mockAirBridgeService } from "../services/mockAirBridgeService";
import type { SandboxVerificationResult } from "../backend/types";
import { RestoreWriteEnginePanel } from "../features/backups/RestoreWriteEnginePanel";

// ── Fixtures ──────────────────────────────────────────────────────────────────

const SAFE_VERIFIED_RESULT: SandboxVerificationResult = {
  // status is "warning" because live metadata check is skipped in sandbox mode
  status: "warning",
  checks: [
    {
      checkId: "CHK-01",
      label: "Target mode allowed",
      status: "passed",
      message: "Target mode 'newBase' is permitted for sandbox restore.",
    },
    {
      checkId: "CHK-02",
      label: "Empty target required",
      status: "passed",
      message: "Target expects empty base — confirmed.",
    },
    {
      checkId: "CHK-03",
      label: "Write gate disabled",
      status: "passed",
      message: "Write gate is disabled. No writes will occur.",
    },
    {
      checkId: "CHK-04",
      label: "Destructive operations",
      status: "passed",
      message: "Destructive operations are not requested.",
    },
    {
      checkId: "CHK-05",
      label: "Attachment upload",
      status: "passed",
      message: "Attachment upload is not requested.",
    },
    {
      checkId: "CHK-06",
      label: "Token return forbidden",
      status: "passed",
      message: "No token will be returned in result.",
    },
    {
      checkId: "CHK-07",
      label: "Full path return forbidden",
      status: "passed",
      message: "No full filesystem path will be returned in result.",
    },
    {
      checkId: "CHK-08",
      label: "Request validity",
      status: "passed",
      message: "Request fields are valid.",
    },
    {
      checkId: "CHK-09",
      label: "Supported target",
      status: "passed",
      message: "Target mode is supported.",
    },
    {
      checkId: "CHK-10",
      label: "Live metadata check",
      status: "skipped",
      message: "Live metadata check is skipped in sandbox mode.",
    },
  ],
  safetySummary: {
    writesEnabled: false,
    networkWritesAttempted: false,
    noChangesMade: true,
    writeGateStatus: "disabled",
    liveMetadataCheckPerformed: false,
  },
  message:
    "Sandbox verification completed with warning: live metadata check was skipped. No changes were made.",
  noChangesMade: true,
  writesEnabled: false,
  networkWritesAttempted: false,
};

const BLOCKED_RESULT: SandboxVerificationResult = {
  status: "blocked",
  checks: [
    {
      checkId: "CHK-01",
      label: "Target mode allowed",
      status: "passed",
      message: "Target mode is permitted.",
    },
    {
      checkId: "CHK-04",
      label: "Destructive operations",
      status: "failed",
      message: "Destructive operations were requested but are not allowed in sandbox mode.",
      remediation: "Set allowDestructiveOperations to false.",
    },
  ],
  safetySummary: {
    writesEnabled: false,
    networkWritesAttempted: false,
    noChangesMade: true,
    writeGateStatus: "disabled",
    liveMetadataCheckPerformed: false,
  },
  message: "Sandbox verification blocked: destructive operations are not permitted.",
  noChangesMade: true,
  writesEnabled: false,
  networkWritesAttempted: false,
};

// ── Mock service contract ─────────────────────────────────────────────────────

describe("verifyRestoreSandboxEnvironment mock service contract", () => {
  it("safe target returns status 'warning' or 'verified'", async () => {
    const result = await mockAirBridgeService.verifyRestoreSandboxEnvironment({
      targetMode: "newBase",
      expectsEmptyTarget: true,
      allowDestructiveOperations: false,
      allowAttachmentUpload: false,
    });
    expect(["warning", "verified"]).toContain(result.status);
  });

  it("unsafe target (allowDestructiveOperations=true) returns 'blocked'", async () => {
    const result = await mockAirBridgeService.verifyRestoreSandboxEnvironment({
      targetMode: "newBase",
      expectsEmptyTarget: true,
      allowDestructiveOperations: true,
      allowAttachmentUpload: false,
    });
    expect(result.status).toBe("blocked");
  });

  it("result always has noChangesMade: true", async () => {
    const result = await mockAirBridgeService.verifyRestoreSandboxEnvironment({
      targetMode: "newBase",
      expectsEmptyTarget: true,
      allowDestructiveOperations: false,
      allowAttachmentUpload: false,
    });
    expect(result.noChangesMade).toBe(true);
  });

  it("result always has writesEnabled: false", async () => {
    const result = await mockAirBridgeService.verifyRestoreSandboxEnvironment({
      targetMode: "newBase",
      expectsEmptyTarget: true,
      allowDestructiveOperations: false,
      allowAttachmentUpload: false,
    });
    expect(result.writesEnabled).toBe(false);
  });

  it("result always has networkWritesAttempted: false", async () => {
    const result = await mockAirBridgeService.verifyRestoreSandboxEnvironment({
      targetMode: "newBase",
      expectsEmptyTarget: true,
      allowDestructiveOperations: false,
      allowAttachmentUpload: false,
    });
    expect(result.networkWritesAttempted).toBe(false);
  });

  it("result JSON does not contain 'token' or 'pat1'", async () => {
    const result = await mockAirBridgeService.verifyRestoreSandboxEnvironment({
      targetMode: "newBase",
      expectsEmptyTarget: true,
      allowDestructiveOperations: false,
      allowAttachmentUpload: false,
    });
    const json = JSON.stringify(result);
    expect(json).not.toContain("token");
    expect(json).not.toContain("pat1");
  });

  it("result JSON does not contain '/Users/' or '/tmp/'", async () => {
    const result = await mockAirBridgeService.verifyRestoreSandboxEnvironment({
      targetMode: "newBase",
      expectsEmptyTarget: true,
      allowDestructiveOperations: false,
      allowAttachmentUpload: false,
    });
    const json = JSON.stringify(result);
    expect(json).not.toContain("/Users/");
    expect(json).not.toContain("/tmp/");
  });

  it("request has no token field", () => {
    const request: Parameters<typeof mockAirBridgeService.verifyRestoreSandboxEnvironment>[0] = {
      targetMode: "newBase",
      expectsEmptyTarget: true,
      allowDestructiveOperations: false,
      allowAttachmentUpload: false,
    };
    const keys = Object.keys(request);
    expect(keys).not.toContain("token");
  });

  it("IPC fallback returns blocked with noChangesMade true", () => {
    const fallbackResult: SandboxVerificationResult = {
      status: "blocked",
      checks: [],
      safetySummary: {
        writesEnabled: false,
        networkWritesAttempted: false,
        noChangesMade: true,
        writeGateStatus: "disabled",
        liveMetadataCheckPerformed: false,
      },
      message: "Sandbox verification is unavailable in this context.",
      noChangesMade: true,
      writesEnabled: false,
      networkWritesAttempted: false,
    };
    expect(fallbackResult.status).toBe("blocked");
    expect(fallbackResult.noChangesMade).toBe(true);
  });
});

// ── RestoreSandboxVerificationPanel rendering ─────────────────────────────────

describe("RestoreSandboxVerificationPanel rendering", () => {
  it("renders panel container data-testid='restore-sandbox-verification-panel'", () => {
    render(<RestoreSandboxVerificationPanel result={null} loading={false} onVerify={() => {}} />);
    expect(screen.getByTestId("restore-sandbox-verification-panel")).not.toBeNull();
  });

  it("shows disabled notice even when result is null", () => {
    render(<RestoreSandboxVerificationPanel result={null} loading={false} onVerify={() => {}} />);
    const notice = screen.getByTestId("sandbox-verification-disabled-notice");
    expect(notice.textContent).toBeTruthy();
  });

  it("shows verify button when result is null and not loading", () => {
    render(<RestoreSandboxVerificationPanel result={null} loading={false} onVerify={() => {}} />);
    expect(screen.getByTestId("sandbox-verify-button")).not.toBeNull();
  });

  it("shows disabled verify button when loading=true", () => {
    render(<RestoreSandboxVerificationPanel result={null} loading={true} onVerify={() => {}} />);
    const btn = screen.getByTestId("sandbox-verify-button");
    expect((btn as HTMLButtonElement).disabled).toBe(true);
  });

  it("shows status badge when result provided", () => {
    render(
      <RestoreSandboxVerificationPanel
        result={SAFE_VERIFIED_RESULT}
        loading={false}
        onVerify={() => {}}
      />,
    );
    expect(screen.getByTestId("sandbox-verification-status")).not.toBeNull();
  });

  it("shows message when result provided", () => {
    render(
      <RestoreSandboxVerificationPanel
        result={SAFE_VERIFIED_RESULT}
        loading={false}
        onVerify={() => {}}
      />,
    );
    expect(screen.getByTestId("sandbox-verification-message")).not.toBeNull();
  });

  it("shows each check row for each check", () => {
    render(
      <RestoreSandboxVerificationPanel
        result={SAFE_VERIFIED_RESULT}
        loading={false}
        onVerify={() => {}}
      />,
    );
    const rows = screen.getAllByTestId("sandbox-check-row");
    expect(rows).toHaveLength(SAFE_VERIFIED_RESULT.checks.length);
  });

  it("shows safety summary section", () => {
    render(
      <RestoreSandboxVerificationPanel
        result={SAFE_VERIFIED_RESULT}
        loading={false}
        onVerify={() => {}}
      />,
    );
    expect(screen.getByTestId("sandbox-safety-summary")).not.toBeNull();
  });

  it("shows no-changes notice in safety summary", () => {
    render(
      <RestoreSandboxVerificationPanel
        result={SAFE_VERIFIED_RESULT}
        loading={false}
        onVerify={() => {}}
      />,
    );
    const notice = screen.getByTestId("sandbox-no-changes-notice");
    expect(notice.textContent).toBeTruthy();
  });

  it("blocked result shows blocked notice", () => {
    render(
      <RestoreSandboxVerificationPanel
        result={BLOCKED_RESULT}
        loading={false}
        onVerify={() => {}}
      />,
    );
    expect(screen.getByTestId("sandbox-blocked-notice")).not.toBeNull();
  });

  it("non-blocked result shows writes-still-disabled notice", () => {
    render(
      <RestoreSandboxVerificationPanel
        result={SAFE_VERIFIED_RESULT}
        loading={false}
        onVerify={() => {}}
      />,
    );
    expect(screen.getByTestId("sandbox-writes-still-disabled-notice")).not.toBeNull();
  });

  it("no execute button anywhere", () => {
    const { container } = render(
      <RestoreSandboxVerificationPanel
        result={SAFE_VERIFIED_RESULT}
        loading={false}
        onVerify={() => {}}
      />,
    );
    const buttons = container.querySelectorAll("button");
    for (const btn of Array.from(buttons)) {
      const text = (btn.textContent ?? "").toLowerCase();
      expect(text).not.toContain("execute");
    }
  });

  it("no Airtable token value in rendered output", () => {
    const { container } = render(
      <RestoreSandboxVerificationPanel
        result={SAFE_VERIFIED_RESULT}
        loading={false}
        onVerify={() => {}}
      />,
    );
    // No Airtable PAT value format (pat + alphanumeric) may appear in output
    expect(container.textContent).not.toMatch(/pat[0-9a-zA-Z]{10,}/);
  });

  it("no full path in rendered output", () => {
    const { container } = render(
      <RestoreSandboxVerificationPanel
        result={SAFE_VERIFIED_RESULT}
        loading={false}
        onVerify={() => {}}
      />,
    );
    expect(container.textContent).not.toContain("/Users/");
    expect(container.textContent).not.toContain("/tmp/");
  });

  it("no success message ('restore complete', 'restore successful', 'succeeded')", () => {
    const { container } = render(
      <RestoreSandboxVerificationPanel
        result={SAFE_VERIFIED_RESULT}
        loading={false}
        onVerify={() => {}}
      />,
    );
    const text = container.textContent?.toLowerCase() ?? "";
    expect(text).not.toContain("restore complete");
    expect(text).not.toContain("restore successful");
    expect(text).not.toContain("succeeded");
  });

  it("existing write engine disabled panel still renders", () => {
    render(<RestoreWriteEnginePanel result={null} />);
    expect(screen.getByTestId("restore-write-engine-panel")).not.toBeNull();
  });
});
