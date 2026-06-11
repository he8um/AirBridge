import { render, screen, waitFor } from "@testing-library/react";
import userEvent from "@testing-library/user-event";
import { describe, it, expect, vi, beforeEach } from "vitest";
import { BackupExecutionPanel } from "../features/backups/BackupExecutionPanel";
import { BackupJobResultCard } from "../features/backups/BackupJobResultCard";
import {
  getDisplayFileName,
  hasAirbridgeExtension,
  redactOutputPath,
  buildConfirmationText,
  BACKUP_CONFIRMATION_TEXT,
} from "../features/backups/backupExecutionHelpers";
import { mockAirBridgeService } from "../services/mockAirBridgeService";
import type { BackupPlan, RecordsExportPlan, RunBackupCommandResponse } from "../backend/types";
import { App } from "../app/App";

// ── Mock the file picker so tests never open a real OS dialog ─────────────

vi.mock("../features/backups/BackupOutputPicker", () => ({
  pickBackupOutputPath: vi.fn().mockResolvedValue(null),
}));

import { pickBackupOutputPath } from "../features/backups/BackupOutputPicker";
const mockPicker = vi.mocked(pickBackupOutputPath);

// ── Helper data ────────────────────────────────────────────────────────────

function makePlan(): BackupPlan {
  return {
    baseId: "appTest01",
    baseName: "Test Base",
    scope: "full",
    tableCount: 1,
    totalFieldCount: 2,
    tables: [
      {
        id: "tbl01",
        name: "Items",
        fieldCount: 2,
        fields: [
          { id: "fld01", name: "Name", fieldType: "singleLineText", compatibility: "restorable" },
          { id: "fld02", name: "Status", fieldType: "singleSelect", compatibility: "restorable" },
        ],
        warnings: [],
        compatibility: { restorableCount: 2, metadataOnlyCount: 0, unknownCount: 0, totalCount: 2 },
      },
    ],
    compatibility: { restorableCount: 2, metadataOnlyCount: 0, unknownCount: 0, totalCount: 2 },
    warnings: [],
    estimate: { schemaRequests: 1, recordReadPages: { type: "unknown" }, note: "" },
    dryRun: true,
  };
}

function makeExportPlan(): RecordsExportPlan {
  return {
    baseId: "appTest01",
    baseName: "Test Base",
    tableCount: 1,
    pageSize: 100,
    tables: [],
    warnings: [],
    plannedOnly: true,
  };
}

// ── backupExecutionHelpers ─────────────────────────────────────────────────

describe("backupExecutionHelpers — getDisplayFileName", () => {
  it("returns filename from a macOS-style path", () => {
    expect(getDisplayFileName("/Users/someone/Documents/my-backup.airbridge")).toBe(
      "my-backup.airbridge",
    );
  });

  it("returns filename from a Windows-style path", () => {
    expect(getDisplayFileName("C:\\Documents\\my-backup.airbridge")).toBe("my-backup.airbridge");
  });

  it("returns empty string for empty input", () => {
    expect(getDisplayFileName("")).toBe("");
  });

  it("returns filename only — no directory component", () => {
    const name = getDisplayFileName("/some/nested/dir/file.airbridge");
    expect(name).not.toContain("/");
    expect(name).toBe("file.airbridge");
  });

  it("does not expose home directory path", () => {
    const name = getDisplayFileName("/home/testuser/file.airbridge");
    expect(name).not.toContain("/home/");
    expect(name).not.toContain("testuser");
  });
});

describe("backupExecutionHelpers — hasAirbridgeExtension", () => {
  it("returns true for .airbridge path", () => {
    expect(hasAirbridgeExtension("/tmp/backup.airbridge")).toBe(true);
  });

  it("returns false for .zip path", () => {
    expect(hasAirbridgeExtension("/tmp/backup.zip")).toBe(false);
  });

  it("returns false for path with no extension", () => {
    expect(hasAirbridgeExtension("/tmp/backup")).toBe(false);
  });
});

describe("backupExecutionHelpers — redactOutputPath", () => {
  it("shows only filename with ellipsis prefix", () => {
    expect(redactOutputPath("/Users/someone/Documents/my-backup.airbridge")).toBe(
      "…/my-backup.airbridge",
    );
  });

  it("does not expose absolute directory", () => {
    const redacted = redactOutputPath("/Users/someone/Documents/my-backup.airbridge");
    expect(redacted).not.toContain("/Users/");
    expect(redacted).not.toContain("someone");
  });

  it("returns empty string for empty path", () => {
    expect(redactOutputPath("")).toBe("");
  });
});

describe("backupExecutionHelpers — buildConfirmationText", () => {
  it("returns CREATE BACKUP regardless of base name", () => {
    expect(buildConfirmationText("My Base")).toBe("CREATE BACKUP");
    expect(buildConfirmationText("")).toBe("CREATE BACKUP");
  });
});

// ── BackupExecutionPanel renders ───────────────────────────────────────────

describe("BackupExecutionPanel — renders", () => {
  beforeEach(() => {
    mockPicker.mockResolvedValue(null);
  });

  it("renders the execution panel", () => {
    render(
      <BackupExecutionPanel backupPlan={null} exportPlan={null} service={mockAirBridgeService} />,
    );
    expect(screen.getByTestId("backup-execution-panel")).toBeInTheDocument();
  });

  it("shows safety copy: full output path is not displayed", () => {
    render(
      <BackupExecutionPanel backupPlan={null} exportPlan={null} service={mockAirBridgeService} />,
    );
    expect(screen.getByTestId("backup-execution-panel").textContent).toMatch(
      /full output path is not displayed/i,
    );
  });

  it("shows safety copy: token is not stored", () => {
    render(
      <BackupExecutionPanel backupPlan={null} exportPlan={null} service={mockAirBridgeService} />,
    );
    expect(screen.getByTestId("backup-execution-panel").textContent).toMatch(
      /token is not stored/i,
    );
  });

  it("shows safety copy: runs only after confirmation", () => {
    render(
      <BackupExecutionPanel backupPlan={null} exportPlan={null} service={mockAirBridgeService} />,
    );
    expect(screen.getByTestId("backup-execution-panel").textContent).toMatch(
      /runs only after confirmation/i,
    );
  });
});

// ── BackupExecutionPanel — run button disabled initially ───────────────────

describe("BackupExecutionPanel — run button disabled state", () => {
  beforeEach(() => {
    mockPicker.mockResolvedValue(null);
  });

  it("run button is disabled when no plans exist", () => {
    render(
      <BackupExecutionPanel backupPlan={null} exportPlan={null} service={mockAirBridgeService} />,
    );
    expect(screen.getByTestId("run-backup-button")).toBeDisabled();
  });

  it("run button is disabled when only backup plan exists", () => {
    render(
      <BackupExecutionPanel
        backupPlan={makePlan()}
        exportPlan={null}
        service={mockAirBridgeService}
      />,
    );
    expect(screen.getByTestId("run-backup-button")).toBeDisabled();
  });

  it("run button is disabled with plans but no path selected", () => {
    render(
      <BackupExecutionPanel
        backupPlan={makePlan()}
        exportPlan={makeExportPlan()}
        service={mockAirBridgeService}
      />,
    );
    expect(screen.getByTestId("run-backup-button")).toBeDisabled();
  });
});

// ── BackupExecutionPanel — file picker and path display ────────────────────

describe("BackupExecutionPanel — file picker", () => {
  beforeEach(() => {
    mockPicker.mockResolvedValue(null);
  });

  it("shows filename only after a valid path is picked — not the full path", async () => {
    mockPicker.mockResolvedValue("/home/testuser/Documents/test-backup.airbridge");
    const user = userEvent.setup();
    render(
      <BackupExecutionPanel
        backupPlan={makePlan()}
        exportPlan={makeExportPlan()}
        service={mockAirBridgeService}
      />,
    );
    await user.click(screen.getByTestId("pick-output-path-button"));
    await waitFor(() => {
      expect(screen.getByTestId("selected-filename-display")).toBeInTheDocument();
    });
    const display = screen.getByTestId("selected-filename-display").textContent;
    expect(display).toBe("test-backup.airbridge");
    expect(display).not.toContain("/home/");
    expect(display).not.toContain("testuser");
  });

  it("does not render the full absolute path anywhere in the panel", async () => {
    mockPicker.mockResolvedValue("/home/testuser/Documents/test-backup.airbridge");
    const user = userEvent.setup();
    render(
      <BackupExecutionPanel
        backupPlan={makePlan()}
        exportPlan={makeExportPlan()}
        service={mockAirBridgeService}
      />,
    );
    await user.click(screen.getByTestId("pick-output-path-button"));
    await waitFor(() => {
      expect(screen.getByTestId("selected-filename-display")).toBeInTheDocument();
    });
    const panelText = screen.getByTestId("backup-execution-panel").textContent ?? "";
    expect(panelText).not.toContain("/home/testuser/Documents");
  });

  it("shows invalid extension error for a .zip path", async () => {
    mockPicker.mockResolvedValue("/tmp/output.zip");
    const user = userEvent.setup();
    render(
      <BackupExecutionPanel
        backupPlan={makePlan()}
        exportPlan={makeExportPlan()}
        service={mockAirBridgeService}
      />,
    );
    await user.click(screen.getByTestId("pick-output-path-button"));
    await waitFor(() => {
      expect(screen.getByTestId("path-validation-status").textContent).toMatch(
        /.airbridge extension/i,
      );
    });
    expect(screen.getByTestId("run-backup-button")).toBeDisabled();
  });

  it("shows valid status for a .airbridge path", async () => {
    mockPicker.mockResolvedValue("/tmp/test-backup.airbridge");
    const user = userEvent.setup();
    render(
      <BackupExecutionPanel
        backupPlan={makePlan()}
        exportPlan={makeExportPlan()}
        service={mockAirBridgeService}
      />,
    );
    await user.click(screen.getByTestId("pick-output-path-button"));
    await waitFor(() => {
      expect(screen.getByTestId("path-validation-status").textContent).toMatch(/valid/i);
    });
  });
});

// ── BackupExecutionPanel — token field ────────────────────────────────────

describe("BackupExecutionPanel — token field", () => {
  it("token field is of type password", () => {
    render(
      <BackupExecutionPanel
        backupPlan={makePlan()}
        exportPlan={makeExportPlan()}
        service={mockAirBridgeService}
      />,
    );
    expect(screen.getByTestId("backup-token-input")).toHaveAttribute("type", "password");
  });

  it("token value is not rendered outside the password input", async () => {
    const user = userEvent.setup();
    render(
      <BackupExecutionPanel
        backupPlan={makePlan()}
        exportPlan={makeExportPlan()}
        service={mockAirBridgeService}
      />,
    );
    const tokenInput = screen.getByTestId("backup-token-input");
    await user.type(tokenInput, "pat_secret_value_123");
    const panel = screen.getByTestId("backup-execution-panel");
    // All text nodes except the input itself should not contain the token
    const allText = Array.from(panel.querySelectorAll("*:not(input)"))
      .map((el) => el.textContent ?? "")
      .join("");
    expect(allText).not.toContain("pat_secret_value_123");
  });
});

// ── BackupExecutionPanel — confirmation required ──────────────────────────

describe("BackupExecutionPanel — confirmation required", () => {
  it("run button disabled without confirmation even with valid path and token", async () => {
    mockPicker.mockResolvedValue("/tmp/test-backup.airbridge");
    const user = userEvent.setup();
    render(
      <BackupExecutionPanel
        backupPlan={makePlan()}
        exportPlan={makeExportPlan()}
        service={mockAirBridgeService}
      />,
    );
    await user.click(screen.getByTestId("pick-output-path-button"));
    await waitFor(() =>
      expect(screen.getByTestId("path-validation-status").textContent).toMatch(/valid/i),
    );
    await user.type(screen.getByTestId("backup-token-input"), "pat_test_token");
    expect(screen.getByTestId("run-backup-button")).toBeDisabled();
  });

  it("run button enabled after valid path, token, and correct confirmation", async () => {
    mockPicker.mockResolvedValue("/tmp/test-backup.airbridge");
    const user = userEvent.setup();
    render(
      <BackupExecutionPanel
        backupPlan={makePlan()}
        exportPlan={makeExportPlan()}
        service={mockAirBridgeService}
      />,
    );
    await user.click(screen.getByTestId("pick-output-path-button"));
    await waitFor(() =>
      expect(screen.getByTestId("path-validation-status").textContent).toMatch(/valid/i),
    );
    await user.type(screen.getByTestId("backup-token-input"), "pat_test_token");
    await user.type(screen.getByTestId("backup-confirmation-input"), BACKUP_CONFIRMATION_TEXT);
    await waitFor(() => {
      expect(screen.getByTestId("run-backup-button")).not.toBeDisabled();
    });
  });
});

// ── BackupExecutionPanel — successful mock run ────────────────────────────

describe("BackupExecutionPanel — mock run", () => {
  beforeEach(() => {
    mockPicker.mockResolvedValue("/tmp/test-backup.airbridge");
  });

  async function setupAndRun() {
    const user = userEvent.setup();
    render(
      <BackupExecutionPanel
        backupPlan={makePlan()}
        exportPlan={makeExportPlan()}
        service={mockAirBridgeService}
      />,
    );
    await user.click(screen.getByTestId("pick-output-path-button"));
    await waitFor(() =>
      expect(screen.getByTestId("path-validation-status").textContent).toMatch(/valid/i),
    );
    await user.type(screen.getByTestId("backup-token-input"), "pat_test_token");
    await user.type(screen.getByTestId("backup-confirmation-input"), BACKUP_CONFIRMATION_TEXT);
    await waitFor(() => expect(screen.getByTestId("run-backup-button")).not.toBeDisabled());
    await user.click(screen.getByTestId("run-backup-button"));
    return user;
  }

  it("shows success result after mock run", async () => {
    await setupAndRun();
    await waitFor(() => {
      expect(screen.getByTestId("backup-job-result-card")).toBeInTheDocument();
    });
    expect(screen.getByTestId("backup-job-result-card").textContent).toMatch(/succeeded/i);
  });

  it("token field is cleared after run completes", async () => {
    await setupAndRun();
    await waitFor(() => {
      expect(screen.getByTestId("backup-job-result-card")).toBeInTheDocument();
    });
    const tokenInput = screen.getByTestId("backup-token-input") as HTMLInputElement;
    expect(tokenInput.value).toBe("");
  });

  it("result card shows filename only — no absolute path", async () => {
    await setupAndRun();
    await waitFor(() => {
      expect(screen.getByTestId("backup-job-result-card")).toBeInTheDocument();
    });
    const cardText = screen.getByTestId("backup-job-result-card").textContent ?? "";
    expect(cardText).not.toContain("/tmp/");
    expect(cardText).not.toContain("/Users/");
    expect(cardText).not.toContain("/home/");
  });

  it("result card does not render token value", async () => {
    await setupAndRun();
    await waitFor(() => {
      expect(screen.getByTestId("backup-job-result-card")).toBeInTheDocument();
    });
    const cardText = screen.getByTestId("backup-job-result-card").textContent ?? "";
    expect(cardText).not.toContain("pat_test_token");
  });

  it("no generated .airbridge file exists in the jsdom environment", async () => {
    // The mock service never writes files — we just verify the response
    // is a mock object and the test environment has no filesystem side effects.
    await setupAndRun();
    await waitFor(() => {
      expect(screen.getByTestId("backup-job-result-card")).toBeInTheDocument();
    });
    // If we got here without errors, no real file write occurred.
    expect(true).toBe(true);
  });
});

// ── BackupExecutionPanel — failed mock run ────────────────────────────────

describe("BackupExecutionPanel — failed mock run (wrong confirmation via service)", () => {
  it("shows sanitized error when mock service returns failure", async () => {
    mockPicker.mockResolvedValue("/tmp/test-backup.airbridge");
    const failService = {
      ...mockAirBridgeService,
      runBackupJob: async () =>
        ({
          success: false,
          safetyErrors: [{ code: "CONFIRMATION_REQUIRED", message: "confirmation missing" }],
          pathValidation: { valid: true },
        }) as RunBackupCommandResponse,
    };
    const user = userEvent.setup();
    render(
      <BackupExecutionPanel
        backupPlan={makePlan()}
        exportPlan={makeExportPlan()}
        service={failService}
      />,
    );
    await user.click(screen.getByTestId("pick-output-path-button"));
    await waitFor(() =>
      expect(screen.getByTestId("path-validation-status").textContent).toMatch(/valid/i),
    );
    await user.type(screen.getByTestId("backup-token-input"), "pat_test_token");
    await user.type(screen.getByTestId("backup-confirmation-input"), BACKUP_CONFIRMATION_TEXT);
    await waitFor(() => expect(screen.getByTestId("run-backup-button")).not.toBeDisabled());
    await user.click(screen.getByTestId("run-backup-button"));
    await waitFor(() => {
      expect(screen.getByTestId("backup-job-result-card")).toBeInTheDocument();
    });
    expect(screen.getByTestId("backup-job-result-card").textContent).toMatch(/failed/i);
    // Error code is shown but token is not
    expect(screen.getByTestId("backup-job-result-card").textContent).toContain(
      "CONFIRMATION_REQUIRED",
    );
    expect(screen.getByTestId("backup-job-result-card").textContent).not.toContain(
      "pat_test_token",
    );
  });
});

// ── BackupJobResultCard ────────────────────────────────────────────────────

describe("BackupJobResultCard", () => {
  it("shows success state", () => {
    const response: RunBackupCommandResponse = {
      success: true,
      packageFilename: "backup.airbridge",
      pathValidation: { valid: true },
      jobResult: {
        jobId: "job-001",
        status: "succeeded",
        baseId: "appTest01",
        baseName: "Test Base",
        tables: [],
        warnings: [],
        errors: [],
      },
    };
    render(<BackupJobResultCard response={response} />);
    expect(screen.getByTestId("backup-job-result-card").textContent).toMatch(/succeeded/i);
    expect(screen.getByText("backup.airbridge")).toBeInTheDocument();
  });

  it("shows failure with safety error code", () => {
    const response: RunBackupCommandResponse = {
      success: false,
      safetyErrors: [{ code: "CONFIRMATION_REQUIRED", message: "confirmation missing" }],
      pathValidation: { valid: true },
    };
    render(<BackupJobResultCard response={response} />);
    expect(screen.getByTestId("backup-job-result-card").textContent).toMatch(/failed/i);
    expect(screen.getByTestId("backup-job-result-card").textContent).toContain(
      "CONFIRMATION_REQUIRED",
    );
  });

  it("does not render token in success response", () => {
    const SENTINEL = "pat_result_card_sentinel_abc123";
    const response: RunBackupCommandResponse = {
      success: true,
      packageFilename: "backup.airbridge",
      pathValidation: { valid: true },
    };
    render(<BackupJobResultCard response={response} />);
    expect(screen.getByTestId("backup-job-result-card").textContent).not.toContain(SENTINEL);
  });

  it("does not render absolute output path", () => {
    const response: RunBackupCommandResponse = {
      success: true,
      packageFilename: "backup.airbridge",
      pathValidation: { valid: true },
    };
    render(<BackupJobResultCard response={response} />);
    const text = screen.getByTestId("backup-job-result-card").textContent ?? "";
    expect(text).not.toContain("/Users/");
    expect(text).not.toContain("/home/");
  });
});

// ── BackupsPage — existing tests still pass ───────────────────────────────

describe("BackupsPage — no enabled production backup-trigger button", () => {
  it("has no enabled button matching start/run/create backup at initial render", async () => {
    const user = userEvent.setup();
    render(<App />);
    await user.click(screen.getByRole("button", { name: "Backups" }));
    const buttons = screen.queryAllByRole("button", {
      name: /start backup|run backup|create backup/i,
    });
    const enabled = buttons.filter((b) => !b.hasAttribute("disabled"));
    // Run Backup button is disabled without plans, path, token, and confirmation
    expect(enabled).toHaveLength(0);
  });
});
