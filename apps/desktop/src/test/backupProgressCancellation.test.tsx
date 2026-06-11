import { render, screen, waitFor } from "@testing-library/react";
import userEvent from "@testing-library/user-event";
import { describe, it, expect, vi, beforeEach } from "vitest";
import { BackupExecutionPanel } from "../features/backups/BackupExecutionPanel";
import { BackupJobResultCard } from "../features/backups/BackupJobResultCard";
import { mockAirBridgeService } from "../services/mockAirBridgeService";
import type { AirBridgeService } from "../services/airBridgeService";
import type {
  BackupJobCancellationResult,
  BackupJobEvent,
  BackupJobProgressSnapshot,
  BackupPlan,
  RecordsExportPlan,
  RunBackupCommandResponse,
} from "../backend/types";

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
    totalFieldCount: 1,
    tables: [
      {
        id: "tbl01",
        name: "Items",
        fieldCount: 1,
        fields: [
          { id: "fld01", name: "Name", fieldType: "singleLineText", compatibility: "restorable" },
        ],
        warnings: [],
        compatibility: { restorableCount: 1, metadataOnlyCount: 0, unknownCount: 0, totalCount: 1 },
      },
    ],
    compatibility: { restorableCount: 1, metadataOnlyCount: 0, unknownCount: 0, totalCount: 1 },
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
    tables: [
      {
        tableId: "tbl01",
        tableName: "Items",
        recordCount: { type: "unknown" },
        requestEstimate: { type: "unknown" },
        pageSize: 100,
        jsonlOutput: { entryPath: "tables/tbl01/records.jsonl", plannedOnly: true },
        tableMetadataPath: "tables/tbl01/table.json",
        fieldsMetadataPath: "tables/tbl01/fields.json",
        fields: [],
        linkedRecordPlans: [],
        attachmentPlans: [],
        warnings: [],
      },
    ],
    warnings: [],
    plannedOnly: true,
  };
}

async function setupAndRun(service: AirBridgeService): Promise<{ panel: HTMLElement }> {
  const { container } = render(
    <BackupExecutionPanel
      backupPlan={makePlan()}
      exportPlan={makeExportPlan()}
      service={service}
    />,
  );
  mockPicker.mockResolvedValueOnce("/tmp/test-backup.airbridge");

  const pickBtn = screen.getByTestId("pick-output-path-button");
  await userEvent.click(pickBtn);
  await waitFor(() => screen.getByTestId("path-validation-status"));

  const tokenInput = screen.getByTestId("backup-token-input");
  await userEvent.type(tokenInput, "pat_test_token_for_progress");

  const confirmInput = screen.getByTestId("backup-confirmation-input");
  await userEvent.type(confirmInput, "CREATE BACKUP");

  const panel = container.querySelector("[data-testid='backup-execution-panel']") as HTMLElement;
  return { panel };
}

// ── Type model tests ───────────────────────────────────────────────────────

describe("BackupJobProgressSnapshot type model", () => {
  it("has required fields", () => {
    const snap: BackupJobProgressSnapshot = {
      jobId: "job-test-001",
      phase: "recordsExport",
      status: "running",
      completedTables: 1,
      unknownTotal: false,
      warningCount: 0,
      errorCount: 0,
    };
    expect(snap.jobId).toBe("job-test-001");
    expect(snap.phase).toBe("recordsExport");
    expect(snap.status).toBe("running");
    expect(snap.unknownTotal).toBe(false);
  });

  it("supports optional fields", () => {
    const snap: BackupJobProgressSnapshot = {
      jobId: "job-test-002",
      phase: "recordsExport",
      status: "running",
      completedTables: 2,
      totalTables: 5,
      unknownTotal: false,
      currentTableId: "tbl01",
      currentTableName: "Projects",
      warningCount: 1,
      errorCount: 0,
    };
    expect(snap.totalTables).toBe(5);
    expect(snap.currentTableName).toBe("Projects");
  });
});

describe("BackupJobCancellationResult type model", () => {
  it("not_running shape", () => {
    const result: BackupJobCancellationResult = {
      jobId: "job-cancel-test-001",
      wasRunning: false,
      statusAtCancellation: "not_running",
    };
    expect(result.wasRunning).toBe(false);
    expect(result.statusAtCancellation).toBe("not_running");
  });
});

// ── Mock service cancellation tests ───────────────────────────────────────

describe("mockAirBridgeService cancellation", () => {
  it("cancelBackupJob returns not_running", async () => {
    const result = await mockAirBridgeService.cancelBackupJob("job-001");
    expect(result.wasRunning).toBe(false);
    expect(result.statusAtCancellation).toBe("not_running");
    expect(result.jobId).toBe("job-001");
  });

  it("getBackupJobStatus returns null (synchronous model)", async () => {
    const snapshot = await mockAirBridgeService.getBackupJobStatus("job-001");
    expect(snapshot).toBeNull();
  });
});

// ── Cancel button visibility ───────────────────────────────────────────────

describe("BackupExecutionPanel cancel button", () => {
  let slowService: AirBridgeService;

  beforeEach(() => {
    mockPicker.mockResolvedValue(null);

    slowService = {
      ...mockAirBridgeService,
      runBackupJob: () => new Promise(() => {}),
      cancelBackupJob: async (jobId: string) => {
        return { jobId, wasRunning: false, statusAtCancellation: "not_running" };
      },
    };
  });

  it("cancel button is not visible when idle", async () => {
    render(
      <BackupExecutionPanel
        backupPlan={makePlan()}
        exportPlan={makeExportPlan()}
        service={mockAirBridgeService}
      />,
    );
    expect(screen.queryByTestId("cancel-backup-button")).toBeNull();
  });

  it("cancel button appears while running", async () => {
    const { panel } = await setupAndRun(slowService);
    const runBtn = screen.getByTestId("run-backup-button");
    await userEvent.click(runBtn);
    await waitFor(() => screen.getByTestId("cancel-backup-button"));
    const cancelBtn = panel.querySelector("[data-testid='cancel-backup-button']");
    expect(cancelBtn).not.toBeNull();
  });

  it("cancel button clears token", async () => {
    await setupAndRun(slowService);
    const runBtn = screen.getByTestId("run-backup-button");
    await userEvent.click(runBtn);
    await waitFor(() => screen.getByTestId("cancel-backup-button"));

    const cancelBtn = screen.getByTestId("cancel-backup-button");
    await userEvent.click(cancelBtn);

    const tokenInput = screen.getByTestId("backup-token-input") as HTMLInputElement;
    expect(tokenInput.value).toBe("");
  });

  it("cancel button disappears after cancellation", async () => {
    await setupAndRun(slowService);
    const runBtn = screen.getByTestId("run-backup-button");
    await userEvent.click(runBtn);
    await waitFor(() => screen.getByTestId("cancel-backup-button"));

    const cancelBtn = screen.getByTestId("cancel-backup-button");
    await userEvent.click(cancelBtn);
    await waitFor(() => expect(screen.queryByTestId("cancel-backup-button")).toBeNull());
  });

  it("token value is not visible outside the password input while running", async () => {
    const { panel } = await setupAndRun(slowService);
    const runBtn = screen.getByTestId("run-backup-button");
    await userEvent.click(runBtn);
    await waitFor(() => screen.getByTestId("cancel-backup-button"));

    const allText = Array.from(panel.querySelectorAll("*:not(input)"))
      .map((el) => el.textContent ?? "")
      .join("");
    expect(allText).not.toContain("pat_test_token_for_progress");
  });
});

// ── Event timeline rendering ───────────────────────────────────────────────

describe("BackupJobResultCard event timeline", () => {
  function makeResponse(events: BackupJobEvent[]): RunBackupCommandResponse {
    return {
      success: true,
      packageFilename: "test.airbridge",
      safetyErrors: [],
      jobResult: {
        jobId: "job-timeline-001",
        status: "succeeded",
        baseId: "appTest01",
        baseName: "Test Base",
        tables: [],
        warnings: [],
        errors: [],
        events,
      },
      pathValidation: { valid: true },
    };
  }

  it("timeline section not rendered when events array is empty", () => {
    render(<BackupJobResultCard response={makeResponse([])} />);
    expect(screen.queryByTestId("backup-event-timeline")).toBeNull();
  });

  it("timeline section not rendered when events field absent", () => {
    const response: RunBackupCommandResponse = {
      success: true,
      packageFilename: "test.airbridge",
      safetyErrors: [],
      jobResult: {
        jobId: "job-001",
        status: "succeeded",
        baseId: "appTest01",
        baseName: "Test Base",
        tables: [],
        warnings: [],
        errors: [],
      },
      pathValidation: { valid: true },
    };
    render(<BackupJobResultCard response={response} />);
    expect(screen.queryByTestId("backup-event-timeline")).toBeNull();
  });

  it("timeline renders all provided events", () => {
    const events: BackupJobEvent[] = [
      { kind: "jobStarted", jobId: "job-001", baseName: "Test Base", tableCount: 1 },
      { kind: "phaseStarted", jobId: "job-001", phase: "recordsExport" },
      { kind: "jobSucceeded", jobId: "job-001", totalRecords: 5 },
    ];
    render(<BackupJobResultCard response={makeResponse(events)} />);
    const timeline = screen.getByTestId("backup-event-timeline");
    expect(timeline).not.toBeNull();
    const items = timeline.querySelectorAll("li");
    expect(items.length).toBe(3);
  });

  it("each event item has a data-event-kind attribute", () => {
    const events: BackupJobEvent[] = [
      { kind: "jobStarted", jobId: "job-001", baseName: "Test Base", tableCount: 1 },
      { kind: "jobSucceeded", jobId: "job-001", totalRecords: 10 },
    ];
    render(<BackupJobResultCard response={makeResponse(events)} />);
    const timeline = screen.getByTestId("backup-event-timeline");
    const items = Array.from(timeline.querySelectorAll("li"));
    expect(items[0].getAttribute("data-event-kind")).toBe("jobStarted");
    expect(items[1].getAttribute("data-event-kind")).toBe("jobSucceeded");
  });

  it("events contain no token sentinel", () => {
    const SENTINEL = "pat_timeline_test_sentinel_0123456789";
    const events: BackupJobEvent[] = [
      { kind: "jobFailed", jobId: "job-001", errorCode: "AUTH_FAILED", message: "auth failed" },
    ];
    const { container } = render(<BackupJobResultCard response={makeResponse(events)} />);
    expect(container.textContent).not.toContain(SENTINEL);
  });

  it("events contain no absolute path", () => {
    const events: BackupJobEvent[] = [{ kind: "jobSucceeded", jobId: "job-001", totalRecords: 0 }];
    const { container } = render(<BackupJobResultCard response={makeResponse(events)} />);
    expect(container.textContent).not.toContain("/Users/");
    expect(container.textContent).not.toContain("/home/");
  });

  it("tableExportCompleted event shows record count", () => {
    const events: BackupJobEvent[] = [
      {
        kind: "tableExportCompleted",
        jobId: "job-001",
        tableId: "tbl01",
        tableName: "Projects",
        recordCount: 42,
        pagesFetched: 1,
      },
    ];
    render(<BackupJobResultCard response={makeResponse(events)} />);
    const timeline = screen.getByTestId("backup-event-timeline");
    expect(timeline.textContent).toContain("42");
  });

  it("jobCancelled event shows phase", () => {
    const events: BackupJobEvent[] = [
      { kind: "jobCancelled", jobId: "job-001", atPhase: "recordsExport" },
    ];
    render(<BackupJobResultCard response={makeResponse(events)} />);
    const timeline = screen.getByTestId("backup-event-timeline");
    expect(timeline.textContent).toContain("recordsExport");
  });
});

// ── Successful run includes events in result ───────────────────────────────

describe("BackupExecutionPanel run result with events", () => {
  beforeEach(() => {
    mockPicker.mockResolvedValue(null);
  });

  it("result card shows timeline when run response includes events", async () => {
    const events: BackupJobEvent[] = [
      { kind: "jobStarted", jobId: "mock-job-001", baseName: "Test Base", tableCount: 1 },
      { kind: "jobSucceeded", jobId: "mock-job-001", totalRecords: 0 },
    ];

    const serviceWithEvents: AirBridgeService = {
      ...mockAirBridgeService,
      runBackupJob: async (req) => {
        void req;
        return {
          success: true,
          packageFilename: "test.airbridge",
          safetyErrors: [],
          jobResult: {
            jobId: "mock-job-001",
            status: "succeeded",
            baseId: "appTest01",
            baseName: "Test Base",
            tables: [],
            warnings: [],
            errors: [],
            events,
          },
          pathValidation: { valid: true },
        };
      },
    };

    await setupAndRun(serviceWithEvents);
    const runBtn = screen.getByTestId("run-backup-button");
    await userEvent.click(runBtn);
    await waitFor(() => screen.getByTestId("backup-job-result-card"));

    const timeline = screen.getByTestId("backup-event-timeline");
    const items = timeline.querySelectorAll("li");
    expect(items.length).toBe(2);
    expect(items[0].getAttribute("data-event-kind")).toBe("jobStarted");
  });
});
