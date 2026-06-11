import { render, screen } from "@testing-library/react";
import userEvent from "@testing-library/user-event";
import { describe, it, expect } from "vitest";
import { App } from "../app/App";
import type {
  BackupJobError,
  BackupJobPackageSummary,
  BackupJobResult,
  BackupJobTableResult,
  BackupJobValidationSummary,
  BackupJobWarning,
} from "../backend/types";

// ── Type shape tests ──────────────────────────────────────────────────────────

describe("BackupJobResult type shape", () => {
  it("succeeded result has correct status", () => {
    const result: BackupJobResult = {
      jobId: "job-ts-001",
      status: "succeeded",
      baseId: "appSyn01",
      baseName: "Synthetic",
      tables: [],
      warnings: [],
      errors: [],
    };
    expect(result.status).toBe("succeeded");
    expect(result.errors).toHaveLength(0);
  });

  it("failed result carries error codes", () => {
    const err: BackupJobError = {
      code: "AUTH_FAILED",
      message: "authentication failed",
      recoverable: false,
    };
    const result: BackupJobResult = {
      jobId: "job-ts-002",
      status: "failed",
      baseId: "appSyn01",
      baseName: "Synthetic",
      tables: [],
      warnings: [],
      errors: [err],
    };
    expect(result.status).toBe("failed");
    expect(result.errors[0].code).toBe("AUTH_FAILED");
    expect(result.errors[0].recoverable).toBe(false);
  });

  it("cancelled result has no package summary", () => {
    const result: BackupJobResult = {
      jobId: "job-ts-003",
      status: "cancelled",
      baseId: "appSyn01",
      baseName: "Synthetic",
      tables: [],
      warnings: [],
      errors: [],
    };
    expect(result.status).toBe("cancelled");
    expect(result.packageSummary).toBeUndefined();
    expect(result.validationSummary).toBeUndefined();
  });

  it("package summary has encrypted false for V0.1", () => {
    const summary: BackupJobPackageSummary = {
      packageId: "00000000-0000-0000-0000-000000000001",
      formatVersion: "0.1.0",
      tableCount: 2,
      recordCount: 10,
      entryCount: 8,
      checksumCount: 7,
      encrypted: false,
      attachmentPolicy: "metadataOnly",
    };
    expect(summary.encrypted).toBe(false);
    expect(summary.attachmentPolicy).toBe("metadataOnly");
  });

  it("validation summary carries entry count", () => {
    const vs: BackupJobValidationSummary = {
      status: "valid",
      errorCount: 0,
      warningCount: 0,
      entryCount: 6,
    };
    expect(vs.status).toBe("valid");
    expect(vs.entryCount).toBe(6);
  });

  it("table result has record and page counts", () => {
    const t: BackupJobTableResult = {
      tableId: "tbl01",
      tableName: "Projects",
      recordCount: 42,
      pagesFetched: 1,
    };
    expect(t.recordCount).toBe(42);
    expect(t.pagesFetched).toBe(1);
  });

  it("warning carries optional tableId", () => {
    const w: BackupJobWarning = {
      code: "RATE_LIMITED",
      message: "request was rate limited",
      tableId: "tbl01",
    };
    expect(w.tableId).toBe("tbl01");
  });

  it("warning tableId is optional", () => {
    const w: BackupJobWarning = {
      code: "UNKNOWN_RECORD_COUNT",
      message: "record count unknown",
    };
    expect(w.tableId).toBeUndefined();
  });

  it("result does not contain token sentinel", () => {
    const SENTINEL = "pat_ts_type_sentinel_0123456789";
    const result: BackupJobResult = {
      jobId: "job-ts-004",
      status: "succeeded",
      baseId: "appSyn01",
      baseName: "Synthetic",
      tables: [],
      warnings: [],
      errors: [],
    };
    const json = JSON.stringify(result);
    expect(json).not.toContain(SENTINEL);
  });

  it("all valid job statuses are accepted", () => {
    const statuses: BackupJobResult["status"][] = [
      "queued",
      "running",
      "succeeded",
      "failed",
      "cancelled",
    ];
    for (const s of statuses) {
      const r: BackupJobResult = {
        jobId: "job-ts-005",
        status: s,
        baseId: "appSyn01",
        baseName: "Synthetic",
        tables: [],
        warnings: [],
        errors: [],
      };
      expect(r.status).toBe(s);
    }
  });
});

// ── Backups page UI tests ─────────────────────────────────────────────────────

async function navigateToBackups(user: ReturnType<typeof userEvent.setup>) {
  await user.click(screen.getByRole("button", { name: "Backups" }));
}

describe("Backups page — backup job pipeline section", () => {
  it("shows the Backup Job Pipeline section", async () => {
    const user = userEvent.setup();
    render(<App />);
    await navigateToBackups(user);
    expect(screen.getByTestId("backup-job-pipeline-section")).toBeTruthy();
  });

  it("states live backup creation is not enabled yet", async () => {
    const user = userEvent.setup();
    render(<App />);
    await navigateToBackups(user);
    const section = screen.getByTestId("backup-job-pipeline-section");
    expect(section.textContent).toMatch(/live backup creation.*(not enabled|not enabled yet)/i);
  });

  it("states no file is created from the screen", async () => {
    const user = userEvent.setup();
    render(<App />);
    await navigateToBackups(user);
    const section = screen.getByTestId("backup-job-pipeline-section");
    expect(section.textContent).toMatch(/no file is created from this screen/i);
  });

  it("mentions cancellation support", async () => {
    const user = userEvent.setup();
    render(<App />);
    await navigateToBackups(user);
    const section = screen.getByTestId("backup-job-pipeline-section");
    expect(section.textContent).toMatch(/cancellation/i);
  });

  it("states no file picker in V0.1", async () => {
    const user = userEvent.setup();
    render(<App />);
    await navigateToBackups(user);
    const section = screen.getByTestId("backup-job-pipeline-section");
    expect(section.textContent).toMatch(/no file picker/i);
  });

  it("has no enabled production backup-trigger button", async () => {
    const user = userEvent.setup();
    render(<App />);
    await navigateToBackups(user);
    // Any backup-start button must be disabled (not wired to live export).
    const buttons = screen.queryAllByRole("button", {
      name: /start backup|run backup|create backup/i,
    });
    const enabled = buttons.filter((b) => !b.hasAttribute("disabled"));
    expect(enabled).toHaveLength(0);
  });
});
