import { render, screen } from "@testing-library/react";
import userEvent from "@testing-library/user-event";
import { axe, toHaveNoViolations } from "jest-axe";
import { App } from "../app/App";

expect.extend(toHaveNoViolations);

// ─── helpers ────────────────────────────────────────────────────────────────

function setup() {
  const user = userEvent.setup();
  const view = render(<App />);
  return { user, ...view };
}

async function navigateTo(user: ReturnType<typeof userEvent.setup>, label: string) {
  await user.click(screen.getByRole("button", { name: label }));
}

// ─── suites ─────────────────────────────────────────────────────────────────

describe("AirBridge App", () => {
  describe("App shell", () => {
    it("renders without crashing", () => {
      setup();
      expect(document.body).toBeTruthy();
    });

    it("shows the AirBridge wordmark in the sidebar", () => {
      setup();
      // The wordmark is plain text inside the sidebar aside element
      const sidebar = screen.getByRole("complementary", { name: "Main navigation" });
      expect(sidebar).toHaveTextContent("AirBridge");
    });

    it("shows all 7 navigation items", () => {
      setup();
      const nav = screen.getByRole("navigation", { name: "Primary navigation" });
      const buttons = Array.from(nav.querySelectorAll("button"));
      expect(buttons).toHaveLength(7);
      const labels = buttons.map((b) => b.textContent?.trim());
      expect(labels).toEqual(
        expect.arrayContaining([
          "Dashboard",
          "Connections",
          "Backups",
          "Restore",
          "Reports",
          "Settings",
          "Logs",
        ]),
      );
    });

    it("starts on the Dashboard page", () => {
      setup();
      expect(screen.getByRole("heading", { name: "Welcome to AirBridge" })).toBeInTheDocument();
    });

    it("shows local-only badge in top bar", () => {
      setup();
      expect(screen.getByText("All data stays on your device")).toBeInTheDocument();
    });

    it("passes accessibility smoke test", async () => {
      const { container } = setup();
      const results = await axe(container);
      expect(results).toHaveNoViolations();
    });
  });

  // ── Navigation ─────────────────────────────────────────────────────────────

  describe("Navigation", () => {
    it("navigates to Connections page when clicking Connections", async () => {
      const { user } = setup();
      await navigateTo(user, "Connections");
      expect(screen.getByLabelText("Personal access token")).toBeInTheDocument();
    });

    it("navigates to Backups page when clicking Backups", async () => {
      const { user } = setup();
      await navigateTo(user, "Backups");
      expect(screen.getByRole("heading", { level: 2, name: "Recent Backups" })).toBeInTheDocument();
    });

    it("navigates to Restore page when clicking Restore", async () => {
      const { user } = setup();
      await navigateTo(user, "Restore");
      expect(screen.getByRole("heading", { name: "Restore from Backup" })).toBeInTheDocument();
    });

    it("navigates to Reports page when clicking Reports", async () => {
      const { user } = setup();
      await navigateTo(user, "Reports");
      expect(screen.getByRole("tab", { name: "Backup Reports" })).toBeInTheDocument();
    });

    it("navigates to Settings page when clicking Settings", async () => {
      const { user } = setup();
      await navigateTo(user, "Settings");
      expect(screen.getByRole("heading", { level: 2, name: "Local Storage" })).toBeInTheDocument();
    });

    it("navigates to Logs page when clicking Logs", async () => {
      const { user } = setup();
      await navigateTo(user, "Logs");
      expect(screen.getByRole("heading", { level: 2, name: "Job Logs" })).toBeInTheDocument();
    });

    it("marks the active nav item with aria-current='page'", async () => {
      const { user } = setup();

      // Dashboard is active by default
      const dashboardBtn = screen.getByRole("button", { name: "Dashboard" });
      expect(dashboardBtn).toHaveAttribute("aria-current", "page");

      // After navigating to Connections, Dashboard loses aria-current
      await navigateTo(user, "Connections");
      expect(dashboardBtn).not.toHaveAttribute("aria-current", "page");
      expect(screen.getByRole("button", { name: "Connections" })).toHaveAttribute(
        "aria-current",
        "page",
      );
    });
  });

  // ── Dashboard page ─────────────────────────────────────────────────────────

  describe("Dashboard page", () => {
    it("shows welcome heading", () => {
      setup();
      expect(screen.getByRole("heading", { name: "Welcome to AirBridge" })).toBeInTheDocument();
    });

    it("shows three stat cards", () => {
      setup();
      expect(screen.getByText("Recent Backups")).toBeInTheDocument();
      expect(screen.getByText("Restore Jobs")).toBeInTheDocument();
      expect(screen.getByText("Connected Bases")).toBeInTheDocument();
    });

    it("shows quick action buttons", () => {
      setup();
      expect(screen.getByRole("button", { name: "Create a new backup" })).toBeInTheDocument();
      expect(
        screen.getByRole("button", { name: "Open an existing backup file" }),
      ).toBeInTheDocument();
      expect(screen.getByRole("button", { name: "Restore from a backup" })).toBeInTheDocument();
    });

    it("shows privacy notice about data staying local", () => {
      setup();
      expect(screen.getByText(/No data leaves your device/i)).toBeInTheDocument();
    });
  });

  // ── Connections page ───────────────────────────────────────────────────────

  describe("Connections page", () => {
    beforeEach(async () => {
      const { user } = setup();
      await navigateTo(user, "Connections");
    });

    it("shows Personal Access Token field", () => {
      expect(screen.getByLabelText("Personal Access Token")).toBeInTheDocument();
    });

    it("shows all four permission check rows", () => {
      expect(screen.getAllByText("Schema read").length).toBeGreaterThanOrEqual(1);
      expect(screen.getAllByText("Records read").length).toBeGreaterThanOrEqual(1);
      expect(screen.getAllByText("Schema write").length).toBeGreaterThanOrEqual(1);
      expect(screen.getAllByText("Records write").length).toBeGreaterThanOrEqual(1);
    });
  });

  // ── Restore page ───────────────────────────────────────────────────────────

  describe("Restore page", () => {
    beforeEach(async () => {
      const { user } = setup();
      await navigateTo(user, "Restore");
    });

    it("renders restore section heading", () => {
      expect(screen.getByRole("heading", { name: "Restore from Backup" })).toBeInTheDocument();
    });

    it("shows dry-run option", () => {
      expect(screen.getByLabelText("Enable dry-run mode")).toBeInTheDocument();
      expect(screen.getByText("Dry-run mode")).toBeInTheDocument();
    });

    it("shows restore execution gate", () => {
      expect(screen.getByTestId("restore-execution-gate-panel")).toBeInTheDocument();
      expect(screen.getByTestId("attempt-restore-button")).toBeInTheDocument();
    });

    it("shows compatibility section", () => {
      expect(screen.getByRole("heading", { name: "Compatibility" })).toBeInTheDocument();
    });
  });

  // ── Reports page ───────────────────────────────────────────────────────────

  describe("Reports page", () => {
    beforeEach(async () => {
      const { user } = setup();
      await navigateTo(user, "Reports");
    });

    it("renders all three report tab buttons", () => {
      expect(screen.getByRole("tab", { name: "Backup Reports" })).toBeInTheDocument();
      expect(screen.getByRole("tab", { name: "Restore Reports" })).toBeInTheDocument();
      expect(screen.getByRole("tab", { name: "Validation Reports" })).toBeInTheDocument();
    });

    it("switches tab content when clicking Restore Reports tab", async () => {
      const restoreTab = screen.getByRole("tab", { name: "Restore Reports" });
      const backupTab = screen.getByRole("tab", { name: "Backup Reports" });
      const user = userEvent.setup();
      await user.click(restoreTab);
      expect(restoreTab).toHaveAttribute("aria-selected", "true");
      expect(backupTab).toHaveAttribute("aria-selected", "false");
    });

    it("switches tab content when clicking Validation Reports tab", async () => {
      const validationTab = screen.getByRole("tab", { name: "Validation Reports" });
      const user = userEvent.setup();
      await user.click(validationTab);
      expect(validationTab).toHaveAttribute("aria-selected", "true");
    });
  });

  // ── Settings page ──────────────────────────────────────────────────────────

  describe("Settings page", () => {
    beforeEach(async () => {
      const { user } = setup();
      await navigateTo(user, "Settings");
    });

    it("shows Local Storage section", () => {
      expect(screen.getByRole("heading", { name: "Local Storage" })).toBeInTheDocument();
    });

    it("shows privacy/no telemetry statement", () => {
      expect(screen.getByText(/does not collect telemetry/i)).toBeInTheDocument();
    });

    it("shows Redaction Defaults section", () => {
      expect(screen.getByRole("heading", { name: "Redaction Defaults" })).toBeInTheDocument();
    });
  });

  // ── Logs page ──────────────────────────────────────────────────────────────

  describe("Logs page", () => {
    beforeEach(async () => {
      const { user } = setup();
      await navigateTo(user, "Logs");
    });

    it("shows Job Logs heading", () => {
      expect(screen.getByRole("heading", { name: "Job Logs" })).toBeInTheDocument();
    });

    it("shows filter buttons", () => {
      expect(screen.getByRole("button", { name: "All" })).toBeInTheDocument();
      expect(screen.getByRole("button", { name: "Warnings" })).toBeInTheDocument();
      expect(screen.getByRole("button", { name: "Errors" })).toBeInTheDocument();
    });

    it("shows log entries from state", () => {
      expect(screen.getByRole("list", { name: "Log entries" })).toBeInTheDocument();
    });
  });
});
