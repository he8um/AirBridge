import { describe, it, expect } from "vitest";
import { render, screen, waitFor } from "@testing-library/react";
import { ReportsPage } from "../pages/ReportsPage";
import { mockAirBridgeService } from "../services/mockAirBridgeService";
import type { JobHistoryFilter, JobHistoryListResult } from "../backend/types";

// ─── Mock service data shape ────────────────────────────────────────────────

describe("mock service job history", () => {
  it("returns items", async () => {
    const result = await mockAirBridgeService.listJobHistory();
    expect(result.items.length).toBeGreaterThan(0);
  });

  it("returns most recent item first", async () => {
    const result = await mockAirBridgeService.listJobHistory();
    expect(result.items[0].id[0]).toBe("hist-006");
  });

  it("items have no token-like values in title or filename", async () => {
    const result = await mockAirBridgeService.listJobHistory();
    for (const item of result.items) {
      expect(item.summary.title).not.toMatch(/Bearer /i);
      expect(item.summary.title).not.toMatch(/^pat[A-Za-z0-9]{10}/);
      if (item.summary.packageFilename) {
        expect(item.summary.packageFilename).not.toContain("/Users/");
        expect(item.summary.packageFilename).not.toContain("/home/");
        expect(item.summary.packageFilename).not.toContain(":\\");
      }
    }
  });

  it("items have no full paths in package filename", async () => {
    const result = await mockAirBridgeService.listJobHistory();
    for (const item of result.items) {
      if (item.summary.packageFilename) {
        expect(item.summary.packageFilename).not.toContain("/");
        expect(item.summary.packageFilename).not.toContain("\\");
      }
    }
  });

  it("planning items have noChangesMade: true", async () => {
    const result = await mockAirBridgeService.listJobHistory();
    const planningKinds = [
      "packageInspection",
      "restoreDryRun",
      "restoreSchemaplan",
      "restoreRecordImportPlan",
      "restoreExecutionAttempt",
    ];
    for (const item of result.items) {
      if (planningKinds.includes(item.kind)) {
        expect(item.noChangesMade).toBe(true);
      }
    }
  });

  it("filter by kind returns only matching items", async () => {
    const filter: JobHistoryFilter = { kind: "packageInspection" };
    const result = await mockAirBridgeService.listJobHistory(filter);
    expect(result.items.length).toBe(1);
    expect(result.items[0].kind).toBe("packageInspection");
    expect(result.filtered).toBe(true);
  });

  it("filter by status returns only matching items", async () => {
    const filter: JobHistoryFilter = { status: "blocked" };
    const result = await mockAirBridgeService.listJobHistory(filter);
    expect(result.items.every((i) => i.status === "blocked")).toBe(true);
  });

  it("limit restricts result count", async () => {
    const filter: JobHistoryFilter = { limit: 2 };
    const result = await mockAirBridgeService.listJobHistory(filter);
    expect(result.items.length).toBe(2);
    expect(result.totalCount).toBeGreaterThan(2);
  });

  it("clearJobHistory returns 0", async () => {
    const count = await mockAirBridgeService.clearJobHistory();
    expect(count).toBe(0);
  });

  it("serialized result has no token sentinel", async () => {
    const result: JobHistoryListResult = await mockAirBridgeService.listJobHistory();
    const json = JSON.stringify(result);
    expect(json).not.toMatch(/Bearer /i);
    expect(json).not.toMatch(/patXXX/);
  });

  it("backup execution item has correct kind", async () => {
    const result = await mockAirBridgeService.listJobHistory();
    const backupItem = result.items.find((i) => i.kind === "backupExecution");
    expect(backupItem).toBeDefined();
  });
});

// ─── JobHistoryPanel rendering ───────────────────────────────────────────────

describe("ReportsPage with JobHistoryPanel", () => {
  it("renders job history panel", async () => {
    render(<ReportsPage service={mockAirBridgeService} />);
    await waitFor(() => {
      expect(screen.getByTestId("job-history-panel")).toBeTruthy();
    });
  });

  it("renders recent activity list", async () => {
    render(<ReportsPage service={mockAirBridgeService} />);
    await waitFor(() => {
      expect(screen.getByTestId("job-history-list")).toBeTruthy();
    });
  });

  it("renders activity items", async () => {
    render(<ReportsPage service={mockAirBridgeService} />);
    await waitFor(() => {
      const items = screen.getAllByTestId(/^job-history-item-hist-/);
      expect(items.length).toBeGreaterThan(0);
    });
  });

  it("renders item titles", async () => {
    render(<ReportsPage service={mockAirBridgeService} />);
    await waitFor(() => {
      const titles = screen.getAllByTestId("job-history-item-title");
      expect(titles.length).toBeGreaterThan(0);
    });
  });

  it("renders status badges", async () => {
    render(<ReportsPage service={mockAirBridgeService} />);
    await waitFor(() => {
      const statuses = screen.getAllByTestId("job-history-item-status");
      expect(statuses.length).toBeGreaterThan(0);
    });
  });

  it("renders timestamps", async () => {
    render(<ReportsPage service={mockAirBridgeService} />);
    await waitFor(() => {
      const timestamps = screen.getAllByTestId("job-history-item-timestamp");
      expect(timestamps.length).toBeGreaterThan(0);
    });
  });

  it("renders filename only (no full path)", async () => {
    render(<ReportsPage service={mockAirBridgeService} />);
    await waitFor(() => {
      const filenames = screen.getAllByTestId("job-history-item-filename");
      for (const el of filenames) {
        expect(el.textContent).not.toContain("/Users/");
        expect(el.textContent).not.toContain("/home/");
        expect(el.textContent).not.toContain(":\\");
      }
    });
  });

  it("renders warning counts where present", async () => {
    render(<ReportsPage service={mockAirBridgeService} />);
    await waitFor(() => {
      const counts = screen.queryAllByTestId("job-history-item-counts");
      expect(counts.length).toBeGreaterThan(0);
    });
  });

  it("renders validation status for inspection item", async () => {
    render(<ReportsPage service={mockAirBridgeService} />);
    await waitFor(() => {
      const validation = screen.queryAllByTestId("job-history-item-validation");
      expect(validation.length).toBeGreaterThan(0);
    });
  });

  it("does not render a full path anywhere in the DOM", async () => {
    render(<ReportsPage service={mockAirBridgeService} />);
    await waitFor(() => {
      expect(screen.getByTestId("job-history-panel")).toBeTruthy();
    });
    const html = document.body.innerHTML;
    expect(html).not.toContain("/Users/");
    expect(html).not.toContain("/home/");
  });

  it("does not render a token in the DOM", async () => {
    render(<ReportsPage service={mockAirBridgeService} />);
    await waitFor(() => {
      expect(screen.getByTestId("job-history-panel")).toBeTruthy();
    });
    const html = document.body.innerHTML;
    expect(html).not.toMatch(/Bearer /i);
  });

  it("renders persistence note", async () => {
    render(<ReportsPage service={mockAirBridgeService} />);
    await waitFor(() => {
      expect(screen.getByTestId("job-history-persistence-note")).toBeTruthy();
    });
  });
});

// ─── Empty state ─────────────────────────────────────────────────────────────

describe("JobHistoryPanel empty state", () => {
  it("renders empty state when no items", async () => {
    const emptyService = {
      ...mockAirBridgeService,
      listJobHistory: (): Promise<JobHistoryListResult> =>
        Promise.resolve({ items: [], totalCount: 0, filtered: false }),
    };
    render(<ReportsPage service={emptyService} />);
    await waitFor(() => {
      expect(screen.getByTestId("job-history-empty")).toBeTruthy();
    });
  });
});
