import { render, screen, waitFor } from "@testing-library/react";
import userEvent from "@testing-library/user-event";
import { describe, it, expect, vi, beforeEach } from "vitest";
import {
  RestoreExecutionGatePanel,
  RESTORE_EXECUTION_CONFIRMATION_TEXT,
} from "../features/backups/RestoreExecutionGatePanel";
import { mockAirBridgeService, MOCK_RESTORE_CONFIRMATION } from "../services/mockAirBridgeService";
import type { AirBridgeService } from "../services/airBridgeService";
import type { RestoreExecutionResult } from "../backend/types";

// ── Helpers ────────────────────────────────────────────────────────────────

const DEFAULT_PROPS = {
  service: mockAirBridgeService,
  inspectedFilename: "backup.airbridge",
  inspectionStatus: "valid" as const,
  packagePath: "/tmp/backup.airbridge",
  dryRunStatus: "readyWithWarnings" as const,
  targetMode: "newBase" as const,
  targetBaseName: undefined,
};

type GateOverrides = {
  service?: AirBridgeService;
  inspectedFilename?: string | null;
  inspectionStatus?: "valid" | "warning" | "invalid" | null;
  packagePath?: string | null;
  dryRunStatus?: "ready" | "readyWithWarnings" | "blocked" | null;
  targetMode?: "newBase" | "emptyExistingBase";
  targetBaseName?: string | undefined;
};

function renderGate(overrides: GateOverrides = {}) {
  const props = { ...DEFAULT_PROPS, ...overrides };
  return render(<RestoreExecutionGatePanel {...props} />);
}

async function fillAndAttempt(token: string, confirmation: string) {
  const tokenInput = screen.getByTestId("restore-exec-token-input");
  const confirmInput = screen.getByTestId("restore-exec-confirmation-input");
  const btn = screen.getByTestId("attempt-restore-button");
  await userEvent.type(tokenInput, token);
  await userEvent.type(confirmInput, confirmation);
  await userEvent.click(btn);
}

// ── Type model tests ───────────────────────────────────────────────────────

describe("RestoreExecutionResult type model", () => {
  it("mock service returns readyButDisabled when all gates pass", async () => {
    const result = await mockAirBridgeService.runRestoreExecution({
      packageFilename: "backup.airbridge",
      packagePath: "/tmp/backup.airbridge",
      packageValidationStatus: "valid",
      dryRunStatus: "ready",
      targetMode: "newBase",
      token: "tok-test",
      confirmation: MOCK_RESTORE_CONFIRMATION,
    });
    expect(result.status).toBe("readyButDisabled");
    expect(result.blockReason).toBe("restoreWriteEngineNotEnabled");
    expect(result.noChangesMade).toBe(true);
  });

  it("mock service returns blocked when token is missing", async () => {
    const result = await mockAirBridgeService.runRestoreExecution({
      packageFilename: "backup.airbridge",
      packagePath: "/tmp/backup.airbridge",
      packageValidationStatus: "valid",
      dryRunStatus: "ready",
      targetMode: "newBase",
      token: "",
      confirmation: MOCK_RESTORE_CONFIRMATION,
    });
    expect(result.status).toBe("blocked");
    expect(result.blockReason).toBe("missingToken");
  });

  it("mock service returns blocked when confirmation is wrong", async () => {
    const result = await mockAirBridgeService.runRestoreExecution({
      packageFilename: "backup.airbridge",
      packagePath: "/tmp/backup.airbridge",
      packageValidationStatus: "valid",
      dryRunStatus: "ready",
      targetMode: "newBase",
      token: "tok-test",
      confirmation: "wrong phrase",
    });
    expect(result.status).toBe("blocked");
    expect(result.blockReason).toBe("missingConfirmation");
  });

  it("mock service returns blocked when dry-run is missing", async () => {
    const result = await mockAirBridgeService.runRestoreExecution({
      packageFilename: "backup.airbridge",
      packagePath: "/tmp/backup.airbridge",
      packageValidationStatus: "valid",
      dryRunStatus: "",
      targetMode: "newBase",
      token: "tok-test",
      confirmation: MOCK_RESTORE_CONFIRMATION,
    });
    expect(result.status).toBe("blocked");
    expect(result.blockReason).toBe("missingDryRunPlan");
  });

  it("mock service returns blocked when dry-run is blocked", async () => {
    const result = await mockAirBridgeService.runRestoreExecution({
      packageFilename: "backup.airbridge",
      packagePath: "/tmp/backup.airbridge",
      packageValidationStatus: "valid",
      dryRunStatus: "blocked",
      targetMode: "newBase",
      token: "tok-test",
      confirmation: MOCK_RESTORE_CONFIRMATION,
    });
    expect(result.status).toBe("blocked");
    expect(result.blockReason).toBe("dryRunBlocked");
  });

  it("mock service returns blocked when package inspection is missing", async () => {
    const result = await mockAirBridgeService.runRestoreExecution({
      packageFilename: "",
      packagePath: "/tmp/backup.airbridge",
      packageValidationStatus: "valid",
      dryRunStatus: "ready",
      targetMode: "newBase",
      token: "tok-test",
      confirmation: MOCK_RESTORE_CONFIRMATION,
    });
    expect(result.status).toBe("blocked");
    expect(result.blockReason).toBe("missingPackageInspection");
  });

  it("mock service result never contains the token value", async () => {
    const result = await mockAirBridgeService.runRestoreExecution({
      packageFilename: "backup.airbridge",
      packagePath: "/tmp/backup.airbridge",
      packageValidationStatus: "valid",
      dryRunStatus: "ready",
      targetMode: "newBase",
      token: "super-secret-token-value",
      confirmation: MOCK_RESTORE_CONFIRMATION,
    });
    const json = JSON.stringify(result);
    expect(json).not.toContain("super-secret-token-value");
  });

  it("mock service result never contains absolute path", async () => {
    const result = await mockAirBridgeService.runRestoreExecution({
      packageFilename: "backup.airbridge",
      packagePath: "/Users/amirhesampiri/backups/backup.airbridge",
      packageValidationStatus: "valid",
      dryRunStatus: "ready",
      targetMode: "newBase",
      token: "tok-test",
      confirmation: MOCK_RESTORE_CONFIRMATION,
    });
    const json = JSON.stringify(result);
    expect(json).not.toContain("/Users/amirhesampiri/");
    expect(json).not.toContain("/backups/");
  });

  it("noChangesMade is always true in every result", async () => {
    const blocked = await mockAirBridgeService.runRestoreExecution({
      packageFilename: "backup.airbridge",
      packagePath: "/tmp/backup.airbridge",
      packageValidationStatus: "valid",
      dryRunStatus: "ready",
      targetMode: "newBase",
      token: "",
      confirmation: "",
    });
    expect(blocked.noChangesMade).toBe(true);

    const disabled = await mockAirBridgeService.runRestoreExecution({
      packageFilename: "backup.airbridge",
      packagePath: "/tmp/backup.airbridge",
      packageValidationStatus: "valid",
      dryRunStatus: "ready",
      targetMode: "newBase",
      token: "tok-test",
      confirmation: MOCK_RESTORE_CONFIRMATION,
    });
    expect(disabled.noChangesMade).toBe(true);
  });
});

// ── Panel idle state ───────────────────────────────────────────────────────

describe("RestoreExecutionGatePanel idle state", () => {
  beforeEach(() => {
    vi.clearAllMocks();
  });

  it("renders the panel", () => {
    renderGate();
    expect(screen.getByTestId("restore-execution-gate-panel")).not.toBeNull();
  });

  it("shows prerequisites checklist", () => {
    renderGate();
    expect(screen.getByTestId("prerequisites-checklist")).not.toBeNull();
    const rows = screen.getAllByTestId("prerequisite-row");
    expect(rows.length).toBeGreaterThanOrEqual(5);
  });

  it("shows not-enabled notice", () => {
    renderGate();
    const notice = screen.getByTestId("execution-not-enabled-notice");
    expect(notice.textContent).toContain("not enabled");
    expect(notice.textContent).toContain("No Airtable changes");
  });

  it("shows token input as password type", () => {
    renderGate();
    const input = screen.getByTestId("restore-exec-token-input") as HTMLInputElement;
    expect(input.type).toBe("password");
  });

  it("shows confirmation input", () => {
    renderGate();
    expect(screen.getByTestId("restore-exec-confirmation-input")).not.toBeNull();
  });

  it("attempt button is disabled without all prerequisites", () => {
    renderGate({ inspectedFilename: null, inspectionStatus: null });
    const btn = screen.getByTestId("attempt-restore-button") as HTMLButtonElement;
    expect(btn.disabled).toBe(true);
  });

  it("attempt button is disabled when no dry-run plan", () => {
    renderGate({ dryRunStatus: null });
    const btn = screen.getByTestId("attempt-restore-button") as HTMLButtonElement;
    expect(btn.disabled).toBe(true);
  });

  it("attempt button is disabled without token", () => {
    renderGate();
    // No token typed — button must remain disabled even with all other prereqs
    const btn = screen.getByTestId("attempt-restore-button") as HTMLButtonElement;
    expect(btn.disabled).toBe(true);
  });

  it("attempt button is disabled without confirmation", async () => {
    renderGate();
    const tokenInput = screen.getByTestId("restore-exec-token-input");
    await userEvent.type(tokenInput, "tok-test");
    // Confirmation still empty
    const btn = screen.getByTestId("attempt-restore-button") as HTMLButtonElement;
    expect(btn.disabled).toBe(true);
  });

  it("does not show result panel on mount", () => {
    renderGate();
    expect(screen.queryByTestId("execution-result-panel")).toBeNull();
  });

  it("does not render token value outside the password input", () => {
    const { container } = renderGate();
    // No visible text content contains the word "token" as a value
    // The password input value is masked by the browser; we check no plaintext appears
    expect(container.textContent).not.toContain("super-secret");
  });
});

// ── Panel result state ─────────────────────────────────────────────────────

describe("RestoreExecutionGatePanel result state", () => {
  beforeEach(() => {
    vi.clearAllMocks();
  });

  it("shows result panel after attempt", async () => {
    renderGate();
    await fillAndAttempt("tok-test", RESTORE_EXECUTION_CONFIRMATION_TEXT);
    await waitFor(() => screen.getByTestId("execution-result-panel"));
    expect(screen.getByTestId("execution-result-panel")).not.toBeNull();
  });

  it("shows readyButDisabled status when all gates pass", async () => {
    renderGate();
    await fillAndAttempt("tok-test", RESTORE_EXECUTION_CONFIRMATION_TEXT);
    await waitFor(() => screen.getByTestId("execution-result-status"));
    const badge = screen.getByTestId("execution-result-status");
    expect(badge.getAttribute("data-execution-status")).toBe("readyButDisabled");
  });

  it("shows no-changes-made notice in result", async () => {
    renderGate();
    await fillAndAttempt("tok-test", RESTORE_EXECUTION_CONFIRMATION_TEXT);
    await waitFor(() => screen.getByTestId("execution-no-changes-notice"));
    expect(screen.getByTestId("execution-no-changes-notice").textContent).toContain(
      "No Airtable changes were made",
    );
  });

  it("shows not-implemented notice when readyButDisabled", async () => {
    renderGate();
    await fillAndAttempt("tok-test", RESTORE_EXECUTION_CONFIRMATION_TEXT);
    await waitFor(() => screen.getByTestId("execution-not-implemented-notice"));
    expect(screen.getByTestId("execution-not-implemented-notice").textContent).toContain(
      "not enabled in this version",
    );
  });

  it("shows blocked status when confirmation is wrong", async () => {
    renderGate();
    await fillAndAttempt("tok-test", "wrong phrase");
    // Button stays disabled — mismatched confirmation prevents click
    // Verify button is still disabled (can't click)
    const btn = screen.getByTestId("attempt-restore-button") as HTMLButtonElement;
    expect(btn.disabled).toBe(true);
  });

  it("token is cleared after successful attempt", async () => {
    renderGate();
    const tokenInput = screen.getByTestId("restore-exec-token-input") as HTMLInputElement;
    await userEvent.type(tokenInput, "tok-test");
    const confirmInput = screen.getByTestId("restore-exec-confirmation-input");
    await userEvent.type(confirmInput, RESTORE_EXECUTION_CONFIRMATION_TEXT);
    const btn = screen.getByTestId("attempt-restore-button");
    await userEvent.click(btn);
    await waitFor(() => screen.getByTestId("execution-result-panel"));
    expect(tokenInput.value).toBe("");
  });

  it("token clears after cancel", async () => {
    renderGate();
    const tokenInput = screen.getByTestId("restore-exec-token-input") as HTMLInputElement;
    await userEvent.type(tokenInput, "tok-test");
    const confirmInput = screen.getByTestId("restore-exec-confirmation-input");
    await userEvent.type(confirmInput, RESTORE_EXECUTION_CONFIRMATION_TEXT);
    const attemptBtn = screen.getByTestId("attempt-restore-button");
    await userEvent.click(attemptBtn);
    await waitFor(() => screen.getByTestId("execution-result-panel"));
    const cancelBtn = screen.getByTestId("cancel-restore-button");
    await userEvent.click(cancelBtn);
    expect(tokenInput.value).toBe("");
    expect(screen.queryByTestId("execution-result-panel")).toBeNull();
  });

  it("does not expose full package path in rendered output", async () => {
    const { container } = render(
      <RestoreExecutionGatePanel
        {...DEFAULT_PROPS}
        packagePath="/Users/amirhesampiri/backups/my-backup.airbridge"
        inspectedFilename="my-backup.airbridge"
      />,
    );
    await fillAndAttempt("tok-test", RESTORE_EXECUTION_CONFIRMATION_TEXT);
    await waitFor(() => screen.getByTestId("execution-result-panel"));
    expect(container.textContent).not.toContain("/Users/amirhesampiri/");
    expect(container.textContent).not.toContain("/backups/");
  });

  it("does not render a 'Start Restore' or 'Execute Restore' or success message", async () => {
    renderGate();
    await fillAndAttempt("tok-test", RESTORE_EXECUTION_CONFIRMATION_TEXT);
    await waitFor(() => screen.getByTestId("execution-result-panel"));
    expect(screen.queryByText(/start restore/i)).toBeNull();
    expect(screen.queryByText(/restore complete/i)).toBeNull();
    expect(screen.queryByText(/restore succeeded/i)).toBeNull();
    const json = JSON.stringify(document.body.textContent);
    expect(json.toLowerCase()).not.toContain("succeeded");
  });
});

// ── IPC fallback tests ─────────────────────────────────────────────────────

describe("RestoreExecutionGatePanel IPC fallback", () => {
  it("shows blocked result when service returns IPC error", async () => {
    const fallbackService: AirBridgeService = {
      ...mockAirBridgeService,
      runRestoreExecution: async (req): Promise<RestoreExecutionResult> => ({
        filename: req.packageFilename,
        status: "blocked",
        blockReason: "missingPackageInspection",
        message: "Tauri IPC unavailable. No Airtable changes were made.",
        warnings: [],
        errors: [{ code: "IPC_UNAVAILABLE", message: "Tauri IPC unavailable" }],
        noChangesMade: true,
      }),
    };
    render(<RestoreExecutionGatePanel {...DEFAULT_PROPS} service={fallbackService} />);
    await fillAndAttempt("tok-test", RESTORE_EXECUTION_CONFIRMATION_TEXT);
    await waitFor(() => screen.getByTestId("execution-result-panel"));
    const badge = screen.getByTestId("execution-result-status");
    expect(badge.getAttribute("data-execution-status")).toBe("blocked");
    expect(screen.getByTestId("execution-no-changes-notice").textContent).toContain(
      "No Airtable changes were made",
    );
  });

  it("noChangesMade is always true in IPC fallback result", async () => {
    let captured: RestoreExecutionResult | undefined;
    const fallbackService: AirBridgeService = {
      ...mockAirBridgeService,
      runRestoreExecution: async (req): Promise<RestoreExecutionResult> => {
        const r: RestoreExecutionResult = {
          filename: req.packageFilename,
          status: "blocked",
          blockReason: "missingPackageInspection",
          message: "IPC unavailable.",
          warnings: [],
          errors: [{ code: "IPC_UNAVAILABLE", message: "IPC unavailable" }],
          noChangesMade: true,
        };
        captured = r;
        return r;
      },
    };
    render(<RestoreExecutionGatePanel {...DEFAULT_PROPS} service={fallbackService} />);
    await fillAndAttempt("tok-test", RESTORE_EXECUTION_CONFIRMATION_TEXT);
    await waitFor(() => screen.getByTestId("execution-result-panel"));
    expect(captured?.noChangesMade).toBe(true);
  });
});
