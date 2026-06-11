import { render, screen } from "@testing-library/react";
import userEvent from "@testing-library/user-event";
import { describe, it, expect } from "vitest";
import { App } from "../app/App";
import * as commands from "../backend/commands";
import { mockAirBridgeService } from "../services/mockAirBridgeService";
import type {
  OutputPathValidationResult,
  RunBackupCommandRequest,
  RunBackupCommandResponse,
} from "../backend/types";

// ── Command bridge exports ─────────────────────────────────────────────────

describe("command bridge — safe backup contract exports", () => {
  it("exports validateBackupOutputPath", () => {
    expect(typeof commands.validateBackupOutputPath).toBe("function");
  });

  it("exports runBackupJob", () => {
    expect(typeof commands.runBackupJob).toBe("function");
  });
});

// ── OutputPathValidationResult type shape ──────────────────────────────────

describe("OutputPathValidationResult type shape", () => {
  it("valid result has valid:true and no error fields", () => {
    const r: OutputPathValidationResult = { valid: true };
    expect(r.valid).toBe(true);
    expect(r.errorCode).toBeUndefined();
    expect(r.errorMessage).toBeUndefined();
  });

  it("invalid result carries error code and message", () => {
    const r: OutputPathValidationResult = {
      valid: false,
      errorCode: "WRONG_EXTENSION",
      errorMessage: "output path must have a .airbridge extension",
    };
    expect(r.valid).toBe(false);
    expect(r.errorCode).toBe("WRONG_EXTENSION");
  });

  it("invalid result for empty path has EMPTY_PATH code", () => {
    const r: OutputPathValidationResult = {
      valid: false,
      errorCode: "EMPTY_PATH",
      errorMessage: "output path must not be empty",
    };
    expect(r.errorCode).toBe("EMPTY_PATH");
  });
});

// ── RunBackupCommandResponse type shape ────────────────────────────────────

describe("RunBackupCommandResponse type shape", () => {
  it("successful response has success:true and packageFilename", () => {
    const resp: RunBackupCommandResponse = {
      success: true,
      packageFilename: "my-backup.airbridge",
      pathValidation: { valid: true },
    };
    expect(resp.success).toBe(true);
    expect(resp.packageFilename).toBe("my-backup.airbridge");
  });

  it("failed response from safety check has safetyErrors", () => {
    const resp: RunBackupCommandResponse = {
      success: false,
      safetyErrors: [{ code: "CONFIRMATION_REQUIRED", message: "confirmation missing" }],
      pathValidation: { valid: true },
    };
    expect(resp.success).toBe(false);
    expect(resp.safetyErrors?.[0].code).toBe("CONFIRMATION_REQUIRED");
  });

  it("failed response from path validation has invalid pathValidation", () => {
    const resp: RunBackupCommandResponse = {
      success: false,
      safetyErrors: [{ code: "INVALID_OUTPUT_PATH", message: "path invalid" }],
      pathValidation: { valid: false, errorCode: "WRONG_EXTENSION" },
    };
    expect(resp.pathValidation.valid).toBe(false);
    expect(resp.pathValidation.errorCode).toBe("WRONG_EXTENSION");
  });

  it("response does not include token", () => {
    const SENTINEL = "pat_ts_contract_sentinel_0123456789";
    const resp: RunBackupCommandResponse = {
      success: true,
      pathValidation: { valid: true },
      jobResult: {
        jobId: "job-001",
        status: "succeeded",
        baseId: "appSyn01",
        baseName: "Synthetic",
        tables: [],
        warnings: [],
        errors: [],
      },
    };
    const json = JSON.stringify(resp);
    expect(json).not.toContain(SENTINEL);
  });

  it("response does not include absolute output path", () => {
    const resp: RunBackupCommandResponse = {
      success: true,
      packageFilename: "my-backup.airbridge",
      pathValidation: { valid: true },
    };
    const json = JSON.stringify(resp);
    expect(json).not.toContain("/Users/");
    expect(json).not.toContain("/home/");
  });
});

// ── Mock service — validateBackupOutputPath ────────────────────────────────

describe("mock service — validateBackupOutputPath", () => {
  it("returns valid for a .airbridge path", async () => {
    const result = await mockAirBridgeService.validateBackupOutputPath("/tmp/output.airbridge");
    expect(result.valid).toBe(true);
    expect(result.errorCode).toBeUndefined();
  });

  it("returns invalid for empty path", async () => {
    const result = await mockAirBridgeService.validateBackupOutputPath("");
    expect(result.valid).toBe(false);
    expect(result.errorCode).toBe("EMPTY_PATH");
  });

  it("returns invalid for wrong extension", async () => {
    const result = await mockAirBridgeService.validateBackupOutputPath("/tmp/output.zip");
    expect(result.valid).toBe(false);
    expect(result.errorCode).toBe("WRONG_EXTENSION");
  });

  it("returns a deterministic result (same input → same output)", async () => {
    const r1 = await mockAirBridgeService.validateBackupOutputPath("/tmp/a.airbridge");
    const r2 = await mockAirBridgeService.validateBackupOutputPath("/tmp/a.airbridge");
    expect(r1.valid).toBe(r2.valid);
    expect(r1.errorCode).toBe(r2.errorCode);
  });
});

// ── Mock service — runBackupJob ────────────────────────────────────────────

function makeMockRequest(overrides?: Partial<RunBackupCommandRequest>): RunBackupCommandRequest {
  return {
    token: "pat_mock_test_token",
    outputPath: "/tmp/backup.airbridge",
    confirmation: "CREATE BACKUP",
    baseId: "appSyn01",
    baseName: "Synthetic",
    baseJson: [],
    schemaJson: [],
    tableSpecs: [],
    ...overrides,
  };
}

describe("mock service — runBackupJob", () => {
  it("returns success for correct confirmation and valid path", async () => {
    const result = await mockAirBridgeService.runBackupJob(makeMockRequest());
    expect(result.success).toBe(true);
  });

  it("rejects missing confirmation", async () => {
    const result = await mockAirBridgeService.runBackupJob(makeMockRequest({ confirmation: "" }));
    expect(result.success).toBe(false);
    expect(result.safetyErrors?.[0].code).toBe("CONFIRMATION_REQUIRED");
  });

  it("rejects wrong confirmation phrase", async () => {
    const result = await mockAirBridgeService.runBackupJob(
      makeMockRequest({ confirmation: "yes please" }),
    );
    expect(result.success).toBe(false);
    expect(result.safetyErrors?.[0].code).toBe("CONFIRMATION_REQUIRED");
  });

  it("rejects wrong extension in output path", async () => {
    const result = await mockAirBridgeService.runBackupJob(
      makeMockRequest({ outputPath: "/tmp/backup.zip" }),
    );
    expect(result.success).toBe(false);
    expect(result.pathValidation.valid).toBe(false);
  });

  it("does not write any files (mock is safe)", async () => {
    const result = await mockAirBridgeService.runBackupJob(makeMockRequest());
    // The response should carry a mock jobResult but no real file is written.
    // We just verify the response is well-formed and contains no file path.
    expect(result.success).toBe(true);
    const json = JSON.stringify(result);
    expect(json).not.toContain("/Users/");
    expect(json).not.toContain("/home/");
  });

  it("does not store token in response", async () => {
    const SENTINEL = "pat_mock_test_token";
    const result = await mockAirBridgeService.runBackupJob(makeMockRequest());
    const json = JSON.stringify(result);
    expect(json).not.toContain(SENTINEL);
  });

  it("packageFilename is filename-only, not full path", async () => {
    const result = await mockAirBridgeService.runBackupJob(
      makeMockRequest({ outputPath: "/some/dir/my-backup.airbridge" }),
    );
    expect(result.success).toBe(true);
    expect(result.packageFilename).toBe("my-backup.airbridge");
    expect(result.packageFilename).not.toContain("/");
  });
});

// ── Backups page UI ────────────────────────────────────────────────────────

async function navigateToBackups(user: ReturnType<typeof userEvent.setup>) {
  await user.click(screen.getByRole("button", { name: "Backups" }));
}

describe("Backups page — backup execution panel (safe command contract)", () => {
  it("shows the backup execution panel", async () => {
    const user = userEvent.setup();
    render(<App />);
    await navigateToBackups(user);
    expect(screen.getByTestId("backup-execution-panel")).toBeTruthy();
  });

  it("shows safety copy: full output path is not displayed", async () => {
    const user = userEvent.setup();
    render(<App />);
    await navigateToBackups(user);
    const panel = screen.getByTestId("backup-execution-panel");
    expect(panel.textContent).toMatch(/full output path is not displayed/i);
  });

  it("shows safety copy: token is not stored", async () => {
    const user = userEvent.setup();
    render(<App />);
    await navigateToBackups(user);
    const panel = screen.getByTestId("backup-execution-panel");
    expect(panel.textContent).toMatch(/token is not stored/i);
  });

  it("mentions CREATE BACKUP as required confirmation text", async () => {
    const user = userEvent.setup();
    render(<App />);
    await navigateToBackups(user);
    const panel = screen.getByTestId("backup-execution-panel");
    expect(panel.textContent).toContain("CREATE BACKUP");
  });

  it("has no enabled production backup-trigger button at initial render", async () => {
    const user = userEvent.setup();
    render(<App />);
    await navigateToBackups(user);
    const buttons = screen.queryAllByRole("button", {
      name: /start backup|run backup|create backup/i,
    });
    const enabled = buttons.filter((b) => !b.hasAttribute("disabled"));
    expect(enabled).toHaveLength(0);
  });
});
