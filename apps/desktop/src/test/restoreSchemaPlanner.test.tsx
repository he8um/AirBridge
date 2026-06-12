import { render, screen, waitFor } from "@testing-library/react";
import userEvent from "@testing-library/user-event";
import { describe, it, expect, vi } from "vitest";
import { RestoreSchemaPlanPanel } from "../features/backups/RestoreSchemaPlanPanel";
import { mockAirBridgeService } from "../services/mockAirBridgeService";
import type { AirBridgeService } from "../services/airBridgeService";
import type { RestoreSchemaPlan, RestoreSchemaPlanRequest } from "../backend/types";

// ── Helpers ────────────────────────────────────────────────────────────────

const MOCK_TABLES: RestoreSchemaPlanRequest["tables"] = [
  {
    tableId: "tblA",
    tableName: "Projects",
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
  targetMode: "newBase" as const,
  targetBaseName: undefined,
  tables: MOCK_TABLES,
};

type PanelOverrides = Partial<{
  service: AirBridgeService;
  packageFilename: string | null;
  dryRunStatus: "ready" | "readyWithWarnings" | "blocked" | null;
  targetMode: "newBase" | "emptyExistingBase";
  targetBaseName: string | undefined;
  tables: RestoreSchemaPlanRequest["tables"];
}>;

function renderPanel(overrides: PanelOverrides = {}) {
  const props = { ...DEFAULT_PROPS, ...overrides };
  return render(<RestoreSchemaPlanPanel {...props} />);
}

// ── Mock service type model tests ───────────────────────────────────────────

describe("RestoreSchemaPlan type model (mock service)", () => {
  it("mock returns readyWithWarnings for ready dry-run", async () => {
    const plan = await mockAirBridgeService.createRestoreSchemaPlan({
      packageFilename: "backup.airbridge",
      dryRunStatus: "ready",
      targetMode: "newBase",
      tables: MOCK_TABLES,
    });
    expect(plan.status === "ready" || plan.status === "readyWithWarnings").toBe(true);
  });

  it("mock returns blocked for blocked dry-run", async () => {
    const plan = await mockAirBridgeService.createRestoreSchemaPlan({
      packageFilename: "backup.airbridge",
      dryRunStatus: "blocked",
      targetMode: "newBase",
      tables: MOCK_TABLES,
    });
    expect(plan.status).toBe("blocked");
    expect(plan.errors.length).toBeGreaterThan(0);
  });

  it("mock noChangesMade is always true", async () => {
    for (const status of ["ready", "readyWithWarnings", "blocked"] as const) {
      const plan = await mockAirBridgeService.createRestoreSchemaPlan({
        packageFilename: "backup.airbridge",
        dryRunStatus: status,
        targetMode: "newBase",
        tables: MOCK_TABLES,
      });
      expect(plan.noChangesMade).toBe(true);
    }
  });

  it("mock result filename is not an absolute path", async () => {
    const plan = await mockAirBridgeService.createRestoreSchemaPlan({
      packageFilename: "backup.airbridge",
      dryRunStatus: "ready",
      targetMode: "newBase",
      tables: MOCK_TABLES,
    });
    expect(plan.filename).not.toContain("/");
    expect(plan.filename).not.toContain("\\");
  });

  it("mock result contains no token sentinel", async () => {
    const sentinel = "pat_schema_frontend_sentinel_9999999999";
    const plan = await mockAirBridgeService.createRestoreSchemaPlan({
      packageFilename: "backup.airbridge",
      dryRunStatus: "ready",
      targetMode: "newBase",
      tables: MOCK_TABLES,
    });
    const json = JSON.stringify(plan);
    expect(json).not.toContain(sentinel);
  });

  it("mock result does not contain succeeded status", async () => {
    const plan = await mockAirBridgeService.createRestoreSchemaPlan({
      packageFilename: "backup.airbridge",
      dryRunStatus: "ready",
      targetMode: "newBase",
      tables: MOCK_TABLES,
    });
    const json = JSON.stringify(plan);
    expect(json).not.toContain("succeeded");
    expect(json).not.toContain("Succeeded");
  });

  it("mock result has table steps and field steps for ready plan", async () => {
    const plan = await mockAirBridgeService.createRestoreSchemaPlan({
      packageFilename: "backup.airbridge",
      dryRunStatus: "ready",
      targetMode: "newBase",
      tables: MOCK_TABLES,
    });
    if (plan.status !== "blocked") {
      expect(plan.tableSteps.length).toBeGreaterThan(0);
      expect(plan.fieldSteps.length).toBeGreaterThan(0);
    }
  });
});

// ── Component rendering tests ───────────────────────────────────────────────

describe("RestoreSchemaPlanPanel rendering", () => {
  it("renders the panel container", () => {
    renderPanel();
    expect(screen.getByTestId("restore-schema-plan-panel")).toBeInTheDocument();
  });

  it("renders the generate button", () => {
    renderPanel();
    expect(screen.getByTestId("schema-plan-generate-btn")).toBeInTheDocument();
  });

  it("button is enabled when inspection and dry-run are ready", () => {
    renderPanel();
    const btn = screen.getByTestId("schema-plan-generate-btn");
    expect(btn).not.toBeDisabled();
  });

  it("button is disabled when filename is null", () => {
    renderPanel({ packageFilename: null });
    const btn = screen.getByTestId("schema-plan-generate-btn");
    expect(btn).toBeDisabled();
  });

  it("button is disabled when dry-run status is blocked", () => {
    renderPanel({ dryRunStatus: "blocked" });
    const btn = screen.getByTestId("schema-plan-generate-btn");
    expect(btn).toBeDisabled();
  });

  it("button is disabled when dry-run status is null", () => {
    renderPanel({ dryRunStatus: null });
    const btn = screen.getByTestId("schema-plan-generate-btn");
    expect(btn).toBeDisabled();
  });

  it("shows requires-inspection message when no filename", () => {
    renderPanel({ packageFilename: null });
    expect(screen.getByTestId("schema-plan-requires-inspection")).toBeInTheDocument();
  });

  it("shows requires-dry-run message when dry-run blocked", () => {
    renderPanel({ dryRunStatus: "blocked" });
    expect(screen.getByTestId("schema-plan-requires-dry-run")).toBeInTheDocument();
  });

  it("no plan result shown before clicking generate", () => {
    renderPanel();
    expect(screen.queryByTestId("schema-plan-result")).not.toBeInTheDocument();
  });
});

// ── Post-generate tests ────────────────────────────────────────────────────

describe("RestoreSchemaPlanPanel after generating", () => {
  it("shows plan result after clicking generate", async () => {
    renderPanel();
    await userEvent.click(screen.getByTestId("schema-plan-generate-btn"));
    await waitFor(() => {
      expect(screen.getByTestId("schema-plan-result")).toBeInTheDocument();
    });
  });

  it("shows no-changes-made notice", async () => {
    renderPanel();
    await userEvent.click(screen.getByTestId("schema-plan-generate-btn"));
    await waitFor(() => {
      expect(screen.getByTestId("schema-plan-no-changes-made")).toBeInTheDocument();
    });
  });

  it("shows table creation steps", async () => {
    renderPanel();
    await userEvent.click(screen.getByTestId("schema-plan-generate-btn"));
    await waitFor(() => {
      expect(screen.getByTestId("schema-plan-table-steps")).toBeInTheDocument();
    });
  });

  it("shows field creation steps", async () => {
    renderPanel();
    await userEvent.click(screen.getByTestId("schema-plan-generate-btn"));
    await waitFor(() => {
      expect(screen.getByTestId("schema-plan-field-steps")).toBeInTheDocument();
    });
  });

  it("shows status badge", async () => {
    renderPanel();
    await userEvent.click(screen.getByTestId("schema-plan-generate-btn"));
    await waitFor(() => {
      expect(screen.getByTestId("schema-plan-status-badge")).toBeInTheDocument();
    });
  });

  it("shows deferred steps section when mock returns deferred fields", async () => {
    renderPanel();
    await userEvent.click(screen.getByTestId("schema-plan-generate-btn"));
    await waitFor(() => {
      expect(screen.getByTestId("schema-plan-deferred-steps")).toBeInTheDocument();
    });
  });

  it("shows manual action fields section when mock returns manual fields", async () => {
    renderPanel();
    await userEvent.click(screen.getByTestId("schema-plan-generate-btn"));
    await waitFor(() => {
      expect(screen.getByTestId("schema-plan-manual-action-fields")).toBeInTheDocument();
    });
  });

  it("shows dependency graph section when mock has edges", async () => {
    renderPanel();
    await userEvent.click(screen.getByTestId("schema-plan-generate-btn"));
    await waitFor(() => {
      expect(screen.getByTestId("schema-plan-dependency-graph")).toBeInTheDocument();
    });
  });

  it("shows warnings section when mock returns warnings", async () => {
    renderPanel();
    await userEvent.click(screen.getByTestId("schema-plan-generate-btn"));
    await waitFor(() => {
      expect(screen.getByTestId("schema-plan-warnings")).toBeInTheDocument();
    });
  });

  it("service is called with no token field", async () => {
    const spy = vi.fn().mockResolvedValue({
      filename: "backup.airbridge",
      status: "ready" as const,
      targetMode: "newBase" as const,
      tableSteps: [],
      fieldSteps: [],
      deferredSteps: [],
      manualActionFields: [],
      dependencyGraph: { edges: [], hasCircularDependency: false, resolutionNote: "" },
      warnings: [],
      errors: [],
      noChangesMade: true,
    } satisfies RestoreSchemaPlan);

    const service: AirBridgeService = { ...mockAirBridgeService, createRestoreSchemaPlan: spy };
    renderPanel({ service });
    await userEvent.click(screen.getByTestId("schema-plan-generate-btn"));

    await waitFor(() => {
      expect(spy).toHaveBeenCalledOnce();
    });

    const callArg = spy.mock.calls[0][0] as RestoreSchemaPlanRequest;
    expect(callArg).not.toHaveProperty("token");
    expect(callArg).not.toHaveProperty("apiKey");
  });

  it("service is called with filename only — not a path", async () => {
    const spy = vi.fn().mockResolvedValue({
      filename: "backup.airbridge",
      status: "ready" as const,
      targetMode: "newBase" as const,
      tableSteps: [],
      fieldSteps: [],
      deferredSteps: [],
      manualActionFields: [],
      dependencyGraph: { edges: [], hasCircularDependency: false, resolutionNote: "" },
      warnings: [],
      errors: [],
      noChangesMade: true,
    } satisfies RestoreSchemaPlan);

    const service: AirBridgeService = { ...mockAirBridgeService, createRestoreSchemaPlan: spy };
    renderPanel({ service, packageFilename: "mybackup.airbridge" });
    await userEvent.click(screen.getByTestId("schema-plan-generate-btn"));

    await waitFor(() => {
      expect(spy).toHaveBeenCalledOnce();
    });

    const callArg = spy.mock.calls[0][0] as RestoreSchemaPlanRequest;
    expect(callArg.packageFilename).toBe("mybackup.airbridge");
    expect(callArg.packageFilename).not.toContain("/");
  });
});
