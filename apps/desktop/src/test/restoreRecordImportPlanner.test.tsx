import { render, screen, waitFor } from "@testing-library/react";
import userEvent from "@testing-library/user-event";
import { describe, it, expect, vi } from "vitest";
import { RestoreRecordImportPlanPanel } from "../features/backups/RestoreRecordImportPlanPanel";
import { mockAirBridgeService } from "../services/mockAirBridgeService";
import type { AirBridgeService } from "../services/airBridgeService";
import type {
  RecordImportTableInput,
  RestoreRecordImportPlan,
  RestoreRecordImportPlanRequest,
} from "../backend/types";

// ── Helpers ────────────────────────────────────────────────────────────────

const MOCK_TABLES: RecordImportTableInput[] = [
  {
    tableId: "tblA",
    tableName: "Projects",
    recordCount: 20,
    fields: [
      { fieldId: "fld01", fieldName: "Name", fieldType: "singleLineText" },
      { fieldId: "fld02", fieldName: "Status", fieldType: "singleSelect" },
    ],
  },
];

const DEFAULT_PROPS = {
  service: mockAirBridgeService,
  packageFilename: "backup.airbridge",
  dryRunStatus: "readyWithWarnings" as const,
  schemaPlanStatus: "readyWithWarnings" as const,
  targetMode: "newBase" as const,
  targetBaseName: undefined,
  tables: MOCK_TABLES,
};

type PanelOverrides = Partial<{
  service: AirBridgeService;
  packageFilename: string | null;
  dryRunStatus: "ready" | "readyWithWarnings" | "blocked" | null;
  schemaPlanStatus: "ready" | "readyWithWarnings" | "blocked" | null;
  targetMode: "newBase" | "emptyExistingBase";
  targetBaseName: string | undefined;
  tables: RecordImportTableInput[];
}>;

function renderPanel(overrides: PanelOverrides = {}) {
  const props = { ...DEFAULT_PROPS, ...overrides };
  return render(<RestoreRecordImportPlanPanel {...props} />);
}

// ── Mock service type model tests ───────────────────────────────────────────

describe("RestoreRecordImportPlan type model (mock service)", () => {
  it("mock returns readyWithWarnings for ready inputs", async () => {
    const plan = await mockAirBridgeService.createRestoreRecordImportPlan({
      packageFilename: "backup.airbridge",
      dryRunStatus: "ready",
      schemaPlanStatus: "ready",
      targetMode: "newBase",
      tables: MOCK_TABLES,
    });
    expect(plan.status === "ready" || plan.status === "readyWithWarnings").toBe(true);
  });

  it("mock plan has no_changes_made true", async () => {
    const plan = await mockAirBridgeService.createRestoreRecordImportPlan({
      packageFilename: "backup.airbridge",
      dryRunStatus: "ready",
      schemaPlanStatus: "ready",
      targetMode: "newBase",
      tables: MOCK_TABLES,
    });
    expect(plan.noChangesMade).toBe(true);
  });

  it("mock plan has table plans", async () => {
    const plan = await mockAirBridgeService.createRestoreRecordImportPlan({
      packageFilename: "backup.airbridge",
      dryRunStatus: "ready",
      schemaPlanStatus: "ready",
      targetMode: "newBase",
      tables: MOCK_TABLES,
    });
    expect(plan.tablePlans.length).toBeGreaterThan(0);
  });

  it("mock plan has known record count with batches", async () => {
    const plan = await mockAirBridgeService.createRestoreRecordImportPlan({
      packageFilename: "backup.airbridge",
      dryRunStatus: "ready",
      schemaPlanStatus: "ready",
      targetMode: "newBase",
      tables: MOCK_TABLES,
    });
    const knownTable = plan.tablePlans.find((tp) => tp.recordCountKnown);
    expect(knownTable).toBeDefined();
    expect(knownTable?.createBatchCount).toBeGreaterThan(0);
    expect(knownTable?.firstPassBatches.length).toBeGreaterThan(0);
  });

  it("mock plan has unknown record count table", async () => {
    const plan = await mockAirBridgeService.createRestoreRecordImportPlan({
      packageFilename: "backup.airbridge",
      dryRunStatus: "ready",
      schemaPlanStatus: "ready",
      targetMode: "newBase",
      tables: MOCK_TABLES,
    });
    const unknownTable = plan.tablePlans.find((tp) => !tp.recordCountKnown);
    expect(unknownTable).toBeDefined();
    expect(unknownTable?.createBatchCount).toBeUndefined();
  });

  it("mock plan has linked record update plans", async () => {
    const plan = await mockAirBridgeService.createRestoreRecordImportPlan({
      packageFilename: "backup.airbridge",
      dryRunStatus: "ready",
      schemaPlanStatus: "ready",
      targetMode: "newBase",
      tables: MOCK_TABLES,
    });
    expect(plan.linkedRecordUpdatePlans.length).toBeGreaterThan(0);
  });

  it("mock plan has attachment metadata policy", async () => {
    const plan = await mockAirBridgeService.createRestoreRecordImportPlan({
      packageFilename: "backup.airbridge",
      dryRunStatus: "ready",
      schemaPlanStatus: "ready",
      targetMode: "newBase",
      tables: MOCK_TABLES,
    });
    const hasAttachment = plan.tablePlans.some((tp) => tp.attachmentPolicies.length > 0);
    expect(hasAttachment).toBe(true);
    const policy = plan.tablePlans.flatMap((tp) => tp.attachmentPolicies)[0];
    expect(policy.policy).toBe("metadataOnly");
  });

  it("mock plan has skipped computed fields", async () => {
    const plan = await mockAirBridgeService.createRestoreRecordImportPlan({
      packageFilename: "backup.airbridge",
      dryRunStatus: "ready",
      schemaPlanStatus: "ready",
      targetMode: "newBase",
      tables: MOCK_TABLES,
    });
    const hasSkipped = plan.tablePlans.some((tp) =>
      tp.fieldPolicies.some((fp) => fp.policy === "skip"),
    );
    expect(hasSkipped).toBe(true);
  });

  it("mock plan has checkpoint plan", async () => {
    const plan = await mockAirBridgeService.createRestoreRecordImportPlan({
      packageFilename: "backup.airbridge",
      dryRunStatus: "ready",
      schemaPlanStatus: "ready",
      targetMode: "newBase",
      tables: MOCK_TABLES,
    });
    const tp = plan.tablePlans[0];
    expect(tp.checkpointPlan).toBeDefined();
    expect(tp.checkpointPlan.tableId).toBe(tp.tableId);
  });

  it("mock plan filename is not a path", async () => {
    const plan = await mockAirBridgeService.createRestoreRecordImportPlan({
      packageFilename: "backup.airbridge",
      dryRunStatus: "ready",
      schemaPlanStatus: "ready",
      targetMode: "newBase",
      tables: MOCK_TABLES,
    });
    expect(plan.filename).not.toContain("/");
    expect(plan.filename).not.toContain("\\");
  });

  it("mock plan does not expose a token", async () => {
    const plan = await mockAirBridgeService.createRestoreRecordImportPlan({
      packageFilename: "backup.airbridge",
      dryRunStatus: "ready",
      schemaPlanStatus: "ready",
      targetMode: "newBase",
      tables: MOCK_TABLES,
    });
    const json = JSON.stringify(plan);
    expect(json).not.toContain("token");
    expect(json).not.toContain("apiKey");
  });

  it("mock plan has retry policy", async () => {
    const plan = await mockAirBridgeService.createRestoreRecordImportPlan({
      packageFilename: "backup.airbridge",
      dryRunStatus: "ready",
      schemaPlanStatus: "ready",
      targetMode: "newBase",
      tables: MOCK_TABLES,
    });
    expect(plan.retryPolicy.maxRetriesOnRateLimit).toBeGreaterThan(0);
    expect(plan.retryPolicy.initialBackoffMs).toBeGreaterThan(0);
    expect(plan.retryPolicy.backoffMultiplier).toBeGreaterThan(1);
  });
});

// ── RestoreRecordImportPlanPanel rendering ─────────────────────────────────

describe("RestoreRecordImportPlanPanel rendering", () => {
  it("renders the panel container", () => {
    renderPanel();
    expect(screen.getByTestId("restore-record-import-plan-panel")).toBeInTheDocument();
  });

  it("renders the generate button", () => {
    renderPanel();
    expect(screen.getByTestId("record-import-plan-generate-btn")).toBeInTheDocument();
  });

  it("button is enabled when package and both plan statuses are ready", () => {
    renderPanel();
    expect(screen.getByTestId("record-import-plan-generate-btn")).not.toBeDisabled();
  });

  it("button is disabled when packageFilename is null", () => {
    renderPanel({ packageFilename: null });
    expect(screen.getByTestId("record-import-plan-generate-btn")).toBeDisabled();
  });

  it("button is disabled when dry-run status is blocked", () => {
    renderPanel({ dryRunStatus: "blocked" });
    expect(screen.getByTestId("record-import-plan-generate-btn")).toBeDisabled();
  });

  it("button is disabled when schema plan status is blocked", () => {
    renderPanel({ schemaPlanStatus: "blocked" });
    expect(screen.getByTestId("record-import-plan-generate-btn")).toBeDisabled();
  });

  it("shows requires-inspection notice when packageFilename is null", () => {
    renderPanel({ packageFilename: null });
    expect(screen.getByTestId("record-import-plan-requires-inspection")).toBeInTheDocument();
  });

  it("shows requires-dry-run notice when dryRunStatus is null", () => {
    renderPanel({ dryRunStatus: null });
    expect(screen.getByTestId("record-import-plan-requires-dry-run")).toBeInTheDocument();
  });

  it("shows requires-schema-plan notice when schemaPlanStatus is null", () => {
    renderPanel({ schemaPlanStatus: null });
    expect(screen.getByTestId("record-import-plan-requires-schema-plan")).toBeInTheDocument();
  });

  it("does not render result before generating", () => {
    renderPanel();
    expect(screen.queryByTestId("record-import-plan-result")).not.toBeInTheDocument();
  });
});

// ── RestoreRecordImportPlanPanel after generating ──────────────────────────

describe("RestoreRecordImportPlanPanel after generating", () => {
  async function renderAndGenerate(overrides: PanelOverrides = {}) {
    const user = userEvent.setup();
    renderPanel(overrides);
    await user.click(screen.getByTestId("record-import-plan-generate-btn"));
    await waitFor(() =>
      expect(screen.getByTestId("record-import-plan-result")).toBeInTheDocument(),
    );
  }

  it("shows plan result after clicking generate", async () => {
    await renderAndGenerate();
    expect(screen.getByTestId("record-import-plan-result")).toBeInTheDocument();
  });

  it("shows status badge after generating", async () => {
    await renderAndGenerate();
    expect(screen.getByTestId("record-import-plan-status-badge")).toBeInTheDocument();
  });

  it("shows table list after generating", async () => {
    await renderAndGenerate();
    expect(screen.getByTestId("record-import-plan-table-list")).toBeInTheDocument();
  });

  it("shows retry note after generating", async () => {
    await renderAndGenerate();
    expect(screen.getByTestId("record-import-plan-retry-note")).toBeInTheDocument();
  });

  it("shows no-changes disclaimer after generating", async () => {
    await renderAndGenerate();
    expect(screen.getByTestId("record-import-plan-no-changes")).toBeInTheDocument();
  });

  it("shows linked record update section when present", async () => {
    await renderAndGenerate();
    expect(screen.getByTestId("record-import-plan-linked-updates")).toBeInTheDocument();
  });

  it("shows warnings section when warnings are present", async () => {
    await renderAndGenerate();
    expect(screen.getByTestId("record-import-plan-warnings")).toBeInTheDocument();
  });

  it("service is called with correct packageFilename", async () => {
    const mockService: AirBridgeService = {
      ...mockAirBridgeService,
      createRestoreRecordImportPlan: vi.fn(
        (req: RestoreRecordImportPlanRequest): Promise<RestoreRecordImportPlan> => {
          expect(req.packageFilename).toBe("backup.airbridge");
          return mockAirBridgeService.createRestoreRecordImportPlan(req);
        },
      ),
    };
    const user = userEvent.setup();
    render(<RestoreRecordImportPlanPanel {...DEFAULT_PROPS} service={mockService} />);
    await user.click(screen.getByTestId("record-import-plan-generate-btn"));
    await waitFor(() =>
      expect(screen.getByTestId("record-import-plan-result")).toBeInTheDocument(),
    );
    expect(mockService.createRestoreRecordImportPlan).toHaveBeenCalled();
  });

  it("service request has no token field", async () => {
    const mockService: AirBridgeService = {
      ...mockAirBridgeService,
      createRestoreRecordImportPlan: vi.fn(
        (req: RestoreRecordImportPlanRequest): Promise<RestoreRecordImportPlan> => {
          const json = JSON.stringify(req);
          expect(json).not.toContain("token");
          expect(json).not.toContain("apiKey");
          return mockAirBridgeService.createRestoreRecordImportPlan(req);
        },
      ),
    };
    const user = userEvent.setup();
    render(<RestoreRecordImportPlanPanel {...DEFAULT_PROPS} service={mockService} />);
    await user.click(screen.getByTestId("record-import-plan-generate-btn"));
    await waitFor(() =>
      expect(screen.getByTestId("record-import-plan-result")).toBeInTheDocument(),
    );
  });
});
