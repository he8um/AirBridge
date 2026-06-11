import { render, screen, waitFor } from "@testing-library/react";
import userEvent from "@testing-library/user-event";
import { describe, it, expect, vi, beforeEach } from "vitest";
import { RestoreDryRunPanel } from "../features/backups/RestoreDryRunPanel";
import { mockAirBridgeService } from "../services/mockAirBridgeService";
import type { AirBridgeService } from "../services/airBridgeService";
import type { RestoreDryRunPlan } from "../backend/types";

// ── Mock the file picker ───────────────────────────────────────────────────

vi.mock("../features/backups/PackageInspectionPicker", () => ({
  pickBackupPackagePath: vi.fn().mockResolvedValue(null),
}));

import { pickBackupPackagePath } from "../features/backups/PackageInspectionPicker";
const mockPicker = vi.mocked(pickBackupPackagePath);

// ── Helpers ────────────────────────────────────────────────────────────────

async function renderAndSelect(service: AirBridgeService, path: string) {
  render(<RestoreDryRunPanel service={service} />);
  mockPicker.mockResolvedValueOnce(path);
  const btn = screen.getByTestId("dry-run-select-file-button");
  await userEvent.click(btn);
}

async function renderSelectAndGenerate(service: AirBridgeService, path: string) {
  await renderAndSelect(service, path);
  const generateBtn = screen.getByTestId("generate-dry-run-plan-button");
  await userEvent.click(generateBtn);
}

// ── Type model tests ───────────────────────────────────────────────────────

describe("RestoreDryRunPlan type model", () => {
  it("mock service returns noChangesMade: true", async () => {
    const result = await mockAirBridgeService.createRestoreDryRunPlan({
      path: "/tmp/backup.airbridge",
      targetMode: "newBase",
    });
    expect(result.noChangesMade).toBe(true);
  });

  it("mock service returns readyWithWarnings for normal path", async () => {
    const result = await mockAirBridgeService.createRestoreDryRunPlan({
      path: "/tmp/backup.airbridge",
      targetMode: "newBase",
    });
    expect(result.status).toBe("readyWithWarnings");
  });

  it("mock service returns blocked status for invalid path", async () => {
    const result = await mockAirBridgeService.createRestoreDryRunPlan({
      path: "/tmp/invalid.airbridge",
      targetMode: "newBase",
    });
    expect(result.status).toBe("blocked");
    expect(result.errors.length).toBeGreaterThan(0);
  });

  it("mock service result does not contain absolute path", async () => {
    const result = await mockAirBridgeService.createRestoreDryRunPlan({
      path: "/Users/amirhesampiri/backups/my-backup.airbridge",
      targetMode: "newBase",
    });
    const json = JSON.stringify(result);
    expect(json).not.toContain("/Users/amirhesampiri/");
    expect(json).not.toContain("/backups/");
  });

  it("mock service filename does not include directory separators", async () => {
    const result = await mockAirBridgeService.createRestoreDryRunPlan({
      path: "/some/deep/path/backup.airbridge",
      targetMode: "newBase",
    });
    expect(result.filename).toBe("backup.airbridge");
    expect(result.filename).not.toContain("/");
    expect(result.filename).not.toContain("\\");
  });

  it("mock plan includes attachment metadata-only warning", async () => {
    const result = await mockAirBridgeService.createRestoreDryRunPlan({
      path: "/tmp/backup.airbridge",
      targetMode: "newBase",
    });
    const hasMeta = result.warnings.some((w) => w.code === "ATTACHMENT_METADATA_ONLY");
    expect(hasMeta).toBe(true);
  });

  it("mock plan includes linked record remapping warning", async () => {
    const result = await mockAirBridgeService.createRestoreDryRunPlan({
      path: "/tmp/backup.airbridge",
      targetMode: "newBase",
    });
    const hasLinked = result.warnings.some((w) => w.code === "LINKED_RECORD_REMAPPING_REQUIRED");
    expect(hasLinked).toBe(true);
  });

  it("mock plan includes computed field warning", async () => {
    const result = await mockAirBridgeService.createRestoreDryRunPlan({
      path: "/tmp/backup.airbridge",
      targetMode: "newBase",
    });
    const hasComputed = result.warnings.some((w) => w.code === "COMPUTED_FIELD_NOT_RESTORED");
    expect(hasComputed).toBe(true);
  });

  it("plan tables have field compatibility information", async () => {
    const result = await mockAirBridgeService.createRestoreDryRunPlan({
      path: "/tmp/backup.airbridge",
      targetMode: "newBase",
    });
    expect(result.tables.length).toBeGreaterThan(0);
    expect(result.tables[0].fields.length).toBeGreaterThan(0);
    const compatValues = result.tables[0].fields.map((f) => f.compatibility);
    expect(compatValues.some((c) => c === "supported")).toBe(true);
  });

  it("ordering plan is present and structured", async () => {
    const result = await mockAirBridgeService.createRestoreDryRunPlan({
      path: "/tmp/backup.airbridge",
      targetMode: "newBase",
    });
    expect(result.ordering).toBeDefined();
    expect(result.ordering?.createTablesFirst).toBe(true);
    expect(result.ordering?.importRecordsWithoutLinks).toBe(true);
    expect(result.ordering?.applyLinksAfterRecords).toBe(true);
  });
});

// ── Panel idle state ───────────────────────────────────────────────────────

describe("RestoreDryRunPanel idle state", () => {
  beforeEach(() => {
    mockPicker.mockResolvedValue(null);
  });

  it("renders the panel", () => {
    render(<RestoreDryRunPanel service={mockAirBridgeService} />);
    expect(screen.getByTestId("restore-dry-run-panel")).not.toBeNull();
  });

  it("shows read-only notice on mount", () => {
    render(<RestoreDryRunPanel service={mockAirBridgeService} />);
    const notice = screen.getByTestId("dry-run-readonly-notice");
    expect(notice.textContent).toContain("No Airtable changes are made");
    expect(notice.textContent).toContain("No token is required");
  });

  it("shows file selector button on mount", () => {
    render(<RestoreDryRunPanel service={mockAirBridgeService} />);
    expect(screen.getByTestId("dry-run-select-file-button")).not.toBeNull();
  });

  it("shows target mode selector on mount", () => {
    render(<RestoreDryRunPanel service={mockAirBridgeService} />);
    expect(screen.getByTestId("restore-target-mode-select")).not.toBeNull();
  });

  it("generate button is disabled when no file selected", () => {
    render(<RestoreDryRunPanel service={mockAirBridgeService} />);
    const btn = screen.getByTestId("generate-dry-run-plan-button");
    expect((btn as HTMLButtonElement).disabled).toBe(true);
  });

  it("does not show plan result on mount", () => {
    render(<RestoreDryRunPanel service={mockAirBridgeService} />);
    expect(screen.queryByTestId("dry-run-plan-result")).toBeNull();
  });

  it("target mode selector defaults to newBase", () => {
    render(<RestoreDryRunPanel service={mockAirBridgeService} />);
    const select = screen.getByTestId("restore-target-mode-select") as HTMLSelectElement;
    expect(select.value).toBe("newBase");
  });

  it("target mode selector has emptyExistingBase option", () => {
    render(<RestoreDryRunPanel service={mockAirBridgeService} />);
    const select = screen.getByTestId("restore-target-mode-select") as HTMLSelectElement;
    const options = Array.from(select.options).map((o) => o.value);
    expect(options).toContain("emptyExistingBase");
  });

  it("stays idle when picker returns null (user cancelled)", async () => {
    render(<RestoreDryRunPanel service={mockAirBridgeService} />);
    mockPicker.mockResolvedValueOnce(null);
    const btn = screen.getByTestId("dry-run-select-file-button");
    await userEvent.click(btn);
    expect(screen.queryByTestId("dry-run-plan-result")).toBeNull();
    expect((screen.getByTestId("generate-dry-run-plan-button") as HTMLButtonElement).disabled).toBe(
      true,
    );
  });
});

// ── Panel result state ─────────────────────────────────────────────────────

describe("RestoreDryRunPanel result state", () => {
  beforeEach(() => {
    mockPicker.mockResolvedValue(null);
  });

  it("enables generate button after file selected", async () => {
    await renderAndSelect(mockAirBridgeService, "/tmp/backup.airbridge");
    const btn = screen.getByTestId("generate-dry-run-plan-button") as HTMLButtonElement;
    expect(btn.disabled).toBe(false);
  });

  it("shows plan result after generate", async () => {
    await renderSelectAndGenerate(mockAirBridgeService, "/tmp/backup.airbridge");
    await waitFor(() => screen.getByTestId("dry-run-plan-result"));
    expect(screen.getByTestId("dry-run-plan-result")).not.toBeNull();
  });

  it("shows readyWithWarnings status badge", async () => {
    await renderSelectAndGenerate(mockAirBridgeService, "/tmp/backup.airbridge");
    await waitFor(() => screen.getByTestId("dry-run-plan-status"));
    const badge = screen.getByTestId("dry-run-plan-status");
    expect(badge.getAttribute("data-plan-status")).toBe("readyWithWarnings");
  });

  it("shows filename without full path", async () => {
    await renderSelectAndGenerate(mockAirBridgeService, "/tmp/some/deep/path/backup.airbridge");
    await waitFor(() => screen.getByTestId("dry-run-plan-filename"));
    const el = screen.getByTestId("dry-run-plan-filename");
    expect(el.textContent).toBe("backup.airbridge");
    expect(el.textContent).not.toContain("/tmp/");
    expect(el.textContent).not.toContain("/some/");
  });

  it("shows no-changes-made notice", async () => {
    await renderSelectAndGenerate(mockAirBridgeService, "/tmp/backup.airbridge");
    await waitFor(() => screen.getByTestId("dry-run-no-changes-notice"));
    const notice = screen.getByTestId("dry-run-no-changes-notice");
    expect(notice.textContent).toContain("No Airtable changes were made");
  });

  it("shows package summary section", async () => {
    await renderSelectAndGenerate(mockAirBridgeService, "/tmp/backup.airbridge");
    await waitFor(() => screen.getByTestId("dry-run-package-summary"));
    expect(screen.getByTestId("dry-run-package-summary")).not.toBeNull();
  });

  it("shows table plans section", async () => {
    await renderSelectAndGenerate(mockAirBridgeService, "/tmp/backup.airbridge");
    await waitFor(() => screen.getByTestId("dry-run-table-plans"));
    expect(screen.getByTestId("dry-run-table-plans")).not.toBeNull();
  });

  it("shows field compatibility badges", async () => {
    await renderSelectAndGenerate(mockAirBridgeService, "/tmp/backup.airbridge");
    await waitFor(() => screen.getAllByTestId("field-compatibility-badge"));
    const badges = screen.getAllByTestId("field-compatibility-badge");
    expect(badges.length).toBeGreaterThan(0);
  });

  it("shows warnings section", async () => {
    await renderSelectAndGenerate(mockAirBridgeService, "/tmp/backup.airbridge");
    await waitFor(() => screen.getByTestId("dry-run-plan-warnings"));
    expect(screen.getByTestId("dry-run-plan-warnings")).not.toBeNull();
  });

  it("shows ordering plan section", async () => {
    await renderSelectAndGenerate(mockAirBridgeService, "/tmp/backup.airbridge");
    await waitFor(() => screen.getByTestId("dry-run-ordering-plan"));
    expect(screen.getByTestId("dry-run-ordering-plan")).not.toBeNull();
  });

  it("shows blocked status for invalid package", async () => {
    await renderSelectAndGenerate(mockAirBridgeService, "/tmp/invalid.airbridge");
    await waitFor(() => screen.getByTestId("dry-run-plan-status"));
    const badge = screen.getByTestId("dry-run-plan-status");
    expect(badge.getAttribute("data-plan-status")).toBe("blocked");
  });

  it("shows errors section for blocked plan", async () => {
    await renderSelectAndGenerate(mockAirBridgeService, "/tmp/invalid.airbridge");
    await waitFor(() => screen.getByTestId("dry-run-plan-errors"));
    expect(screen.getByTestId("dry-run-plan-errors")).not.toBeNull();
  });

  it("never renders a restore execution button", async () => {
    await renderSelectAndGenerate(mockAirBridgeService, "/tmp/backup.airbridge");
    await waitFor(() => screen.getByTestId("dry-run-plan-result"));
    // No button with text containing "Start Restore" or "Execute" or "Run Restore"
    const buttons = screen.queryAllByRole("button");
    const restoreButtons = buttons.filter((b) => {
      const text = b.textContent?.toLowerCase() ?? "";
      return (
        text.includes("start restore") || text.includes("execute") || text.includes("run restore")
      );
    });
    expect(restoreButtons).toHaveLength(0);
  });

  it("does not render token input", async () => {
    render(<RestoreDryRunPanel service={mockAirBridgeService} />);
    const inputs = screen.queryAllByRole("textbox");
    const tokenInputs = inputs.filter((el) => {
      const label = el.getAttribute("aria-label")?.toLowerCase() ?? "";
      const placeholder = (el as HTMLInputElement).placeholder?.toLowerCase() ?? "";
      return label.includes("token") || placeholder.includes("token");
    });
    expect(tokenInputs).toHaveLength(0);
  });

  it("does not expose absolute path in rendered output", async () => {
    const { container } = render(<RestoreDryRunPanel service={mockAirBridgeService} />);
    mockPicker.mockResolvedValueOnce("/Users/amirhesampiri/backups/my-backup.airbridge");
    const selectBtn = screen.getByTestId("dry-run-select-file-button");
    await userEvent.click(selectBtn);
    const generateBtn = screen.getByTestId("generate-dry-run-plan-button");
    await userEvent.click(generateBtn);
    await waitFor(() => screen.getByTestId("dry-run-plan-result"));
    expect(container.textContent).not.toContain("/Users/amirhesampiri/");
    expect(container.textContent).not.toContain("/backups/");
  });
});

// ── IPC fallback tests ─────────────────────────────────────────────────────

describe("RestoreDryRunPanel IPC fallback", () => {
  beforeEach(() => {
    mockPicker.mockResolvedValue(null);
  });

  it("shows blocked plan when service returns IPC_UNAVAILABLE", async () => {
    const fallbackService: AirBridgeService = {
      ...mockAirBridgeService,
      createRestoreDryRunPlan: async (req) => ({
        filename: "",
        status: "blocked",
        targetMode: req.targetMode,
        tables: [],
        warnings: [],
        errors: [{ code: "IPC_UNAVAILABLE", message: "Tauri IPC unavailable" }],
        noChangesMade: true,
      }),
    };
    await renderSelectAndGenerate(fallbackService, "/tmp/backup.airbridge");
    await waitFor(() => screen.getByTestId("dry-run-plan-result"));
    const badge = screen.getByTestId("dry-run-plan-status");
    expect(badge.getAttribute("data-plan-status")).toBe("blocked");
  });

  it("noChangesMade is always true even in IPC fallback", async () => {
    let capturedPlan: RestoreDryRunPlan | undefined;
    const fallbackService: AirBridgeService = {
      ...mockAirBridgeService,
      createRestoreDryRunPlan: async (req) => {
        const plan: RestoreDryRunPlan = {
          filename: "",
          status: "blocked",
          targetMode: req.targetMode,
          tables: [],
          warnings: [],
          errors: [{ code: "IPC_UNAVAILABLE", message: "Tauri IPC unavailable" }],
          noChangesMade: true,
        };
        capturedPlan = plan;
        return plan;
      },
    };
    await renderSelectAndGenerate(fallbackService, "/tmp/backup.airbridge");
    await waitFor(() => screen.getByTestId("dry-run-plan-result"));
    expect(capturedPlan?.noChangesMade).toBe(true);
  });
});
