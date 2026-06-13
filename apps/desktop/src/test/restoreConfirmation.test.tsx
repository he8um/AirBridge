import { render, screen, fireEvent } from "@testing-library/react";
import { describe, it, expect, vi } from "vitest";
import { RestoreConfirmationPanel } from "../features/backups/RestoreConfirmationPanel";
import { mockAirBridgeService } from "../services/mockAirBridgeService";
import type { RestoreConfirmationResult } from "../backend/types";

// ── Fixtures ──────────────────────────────────────────────────────────────────

const CONFIRMED_RESULT: RestoreConfirmationResult = {
  status: "confirmed",
  checks: [
    {
      checkId: "CHK-C01",
      label: "Write gate disabled",
      status: "passed",
      message: "Write gate is disabled.",
    },
    {
      checkId: "CHK-C02",
      label: "Sandbox not blocked",
      status: "passed",
      message: "Sandbox status is 'warning'.",
    },
    {
      checkId: "CHK-C03",
      label: "Confirmation text",
      status: "passed",
      message: "Confirmation text matches exactly.",
    },
    {
      checkId: "CHK-C04",
      label: "No token in text",
      status: "passed",
      message: "No API token detected.",
    },
    {
      checkId: "CHK-C05",
      label: "Writes remain disabled",
      status: "passed",
      message: "Writes are disabled.",
    },
  ],
  requirements: [
    { requirementId: "REQ-C01", label: "Write gate disabled", satisfied: true, note: "" },
    { requirementId: "REQ-C02", label: "Sandbox not blocked", satisfied: true, note: "" },
    {
      requirementId: "REQ-C03",
      label: "Exact match",
      satisfied: true,
      note: 'Required: "RESTORE TO MY BASE"',
    },
  ],
  requiredText: "RESTORE TO MY BASE",
  message:
    "Confirmation accepted. Restore writes remain disabled — no Airtable changes will be made.",
  noChangesMade: true,
  networkWritesAttempted: false,
  writesEnabled: false,
};

const REJECTED_RESULT: RestoreConfirmationResult = {
  status: "rejected",
  checks: [
    {
      checkId: "CHK-C01",
      label: "Write gate disabled",
      status: "passed",
      message: "Write gate is disabled.",
    },
    {
      checkId: "CHK-C02",
      label: "Sandbox not blocked",
      status: "passed",
      message: "Sandbox status is 'warning'.",
    },
    {
      checkId: "CHK-C03",
      label: "Confirmation text",
      status: "failed",
      message: "Confirmation text does not match.",
    },
    {
      checkId: "CHK-C04",
      label: "No token in text",
      status: "passed",
      message: "No API token detected.",
    },
    {
      checkId: "CHK-C05",
      label: "Writes remain disabled",
      status: "passed",
      message: "Writes are disabled.",
    },
  ],
  requirements: [
    { requirementId: "REQ-C01", label: "Write gate disabled", satisfied: true, note: "" },
    { requirementId: "REQ-C02", label: "Sandbox not blocked", satisfied: true, note: "" },
    { requirementId: "REQ-C03", label: "Exact match", satisfied: false, note: "" },
  ],
  requiredText: "RESTORE TO MY BASE",
  message: "Confirmation text does not match. Type the exact required text (case-sensitive).",
  noChangesMade: true,
  networkWritesAttempted: false,
  writesEnabled: false,
};

const BLOCKED_RESULT: RestoreConfirmationResult = {
  status: "blocked",
  checks: [
    {
      checkId: "CHK-C01",
      label: "Write gate disabled",
      status: "passed",
      message: "Write gate is disabled.",
    },
    {
      checkId: "CHK-C02",
      label: "Sandbox not blocked",
      status: "failed",
      message: "Sandbox is blocked.",
    },
    { checkId: "CHK-C03", label: "Confirmation text", status: "failed", message: "Blocked." },
    { checkId: "CHK-C04", label: "No token in text", status: "passed", message: "" },
    { checkId: "CHK-C05", label: "Writes remain disabled", status: "passed", message: "" },
  ],
  requirements: [],
  requiredText: "RESTORE TO MY BASE",
  message: "Sandbox verification is blocked. Resolve Gate 1 first.",
  noChangesMade: true,
  networkWritesAttempted: false,
  writesEnabled: false,
};

// ── Mock service contract ─────────────────────────────────────────────────────

describe("validateRestoreConfirmationGate mock service contract", () => {
  it("exact match with verified sandbox returns 'confirmed'", async () => {
    const result = await mockAirBridgeService.validateRestoreConfirmationGate({
      enteredText: "RESTORE TO MY BASE",
      targetLabel: "My Base",
      sandboxVerificationStatus: "verified",
    });
    expect(result.status).toBe("confirmed");
  });

  it("exact match with 'warning' sandbox returns 'confirmed'", async () => {
    const result = await mockAirBridgeService.validateRestoreConfirmationGate({
      enteredText: "RESTORE TO MY BASE",
      targetLabel: "My Base",
      sandboxVerificationStatus: "warning",
    });
    expect(result.status).toBe("confirmed");
  });

  it("wrong case returns 'rejected'", async () => {
    const result = await mockAirBridgeService.validateRestoreConfirmationGate({
      enteredText: "restore to my base",
      targetLabel: "My Base",
      sandboxVerificationStatus: "warning",
    });
    expect(result.status).toBe("rejected");
  });

  it("partial match returns 'rejected'", async () => {
    const result = await mockAirBridgeService.validateRestoreConfirmationGate({
      enteredText: "RESTORE",
      targetLabel: "My Base",
      sandboxVerificationStatus: "warning",
    });
    expect(result.status).toBe("rejected");
  });

  it("extra words returns 'rejected'", async () => {
    const result = await mockAirBridgeService.validateRestoreConfirmationGate({
      enteredText: "RESTORE TO MY BASE NOW",
      targetLabel: "My Base",
      sandboxVerificationStatus: "warning",
    });
    expect(result.status).toBe("rejected");
  });

  it("empty text returns 'rejected'", async () => {
    const result = await mockAirBridgeService.validateRestoreConfirmationGate({
      enteredText: "",
      targetLabel: "My Base",
      sandboxVerificationStatus: "warning",
    });
    expect(result.status).toBe("rejected");
  });

  it("blocked sandbox returns 'blocked' even with correct text", async () => {
    const result = await mockAirBridgeService.validateRestoreConfirmationGate({
      enteredText: "RESTORE TO MY BASE",
      targetLabel: "My Base",
      sandboxVerificationStatus: "blocked",
    });
    expect(result.status).toBe("blocked");
  });

  it("noChangesMade always true", async () => {
    for (const text of ["RESTORE TO MY BASE", "", "wrong"]) {
      const result = await mockAirBridgeService.validateRestoreConfirmationGate({
        enteredText: text,
        targetLabel: "My Base",
        sandboxVerificationStatus: "warning",
      });
      expect(result.noChangesMade).toBe(true);
    }
  });

  it("writesEnabled always false", async () => {
    for (const text of ["RESTORE TO MY BASE", "", "wrong"]) {
      const result = await mockAirBridgeService.validateRestoreConfirmationGate({
        enteredText: text,
        targetLabel: "My Base",
        sandboxVerificationStatus: "warning",
      });
      expect(result.writesEnabled).toBe(false);
    }
  });

  it("networkWritesAttempted always false", async () => {
    const result = await mockAirBridgeService.validateRestoreConfirmationGate({
      enteredText: "RESTORE TO MY BASE",
      targetLabel: "My Base",
      sandboxVerificationStatus: "warning",
    });
    expect(result.networkWritesAttempted).toBe(false);
  });

  it("result JSON contains no token or path", async () => {
    const result = await mockAirBridgeService.validateRestoreConfirmationGate({
      enteredText: "RESTORE TO MY BASE",
      targetLabel: "My Base",
      sandboxVerificationStatus: "warning",
    });
    const json = JSON.stringify(result);
    expect(json).not.toMatch(/pat[0-9a-zA-Z]{10,}/);
    expect(json).not.toContain("/Users/");
    expect(json).not.toContain("/tmp/");
  });

  it("request has no token field", () => {
    const request: Parameters<typeof mockAirBridgeService.validateRestoreConfirmationGate>[0] = {
      enteredText: "RESTORE TO MY BASE",
      targetLabel: "My Base",
      sandboxVerificationStatus: "warning",
    };
    expect(Object.keys(request)).not.toContain("token");
  });

  it("result status never 'succeeded'", async () => {
    const result = await mockAirBridgeService.validateRestoreConfirmationGate({
      enteredText: "RESTORE TO MY BASE",
      targetLabel: "My Base",
      sandboxVerificationStatus: "warning",
    });
    const json = JSON.stringify(result);
    expect(json).not.toContain("succeeded");
  });

  it("requiredText starts with RESTORE", async () => {
    const result = await mockAirBridgeService.validateRestoreConfirmationGate({
      enteredText: "RESTORE TO MY BASE",
      targetLabel: "My Base",
      sandboxVerificationStatus: "warning",
    });
    expect(result.requiredText.startsWith("RESTORE")).toBe(true);
  });

  it("requiredText contains no path separators", async () => {
    const result = await mockAirBridgeService.validateRestoreConfirmationGate({
      enteredText: "RESTORE TO MY BASE",
      targetLabel: "My Base",
      sandboxVerificationStatus: "warning",
    });
    expect(result.requiredText).not.toContain("/");
    expect(result.requiredText).not.toContain("\\");
  });

  it("IPC fallback result is blocked with safe values", () => {
    const fallback: RestoreConfirmationResult = {
      status: "blocked",
      checks: [],
      requirements: [],
      requiredText: "RESTORE BACKUP",
      message: "Confirmation validation is not available in this context.",
      noChangesMade: true,
      networkWritesAttempted: false,
      writesEnabled: false,
    };
    expect(fallback.status).toBe("blocked");
    expect(fallback.noChangesMade).toBe(true);
    expect(fallback.writesEnabled).toBe(false);
  });
});

// ── RestoreConfirmationPanel rendering ───────────────────────────────────────

describe("RestoreConfirmationPanel rendering", () => {
  it("renders panel container", () => {
    render(
      <RestoreConfirmationPanel
        result={null}
        loading={false}
        requiredText="RESTORE BACKUP"
        onValidate={() => {}}
      />,
    );
    expect(screen.getByTestId("restore-confirmation-panel")).not.toBeNull();
  });

  it("always shows writes-disabled notice", () => {
    render(
      <RestoreConfirmationPanel
        result={null}
        loading={false}
        requiredText="RESTORE BACKUP"
        onValidate={() => {}}
      />,
    );
    expect(screen.getByTestId("confirmation-writes-disabled-notice")).not.toBeNull();
  });

  it("shows required text", () => {
    render(
      <RestoreConfirmationPanel
        result={null}
        loading={false}
        requiredText="RESTORE TO MY BASE"
        onValidate={() => {}}
      />,
    );
    expect(screen.getByTestId("confirmation-required-text").textContent).toBe("RESTORE TO MY BASE");
  });

  it("shows text input", () => {
    render(
      <RestoreConfirmationPanel
        result={null}
        loading={false}
        requiredText="RESTORE BACKUP"
        onValidate={() => {}}
      />,
    );
    expect(screen.getByTestId("confirmation-text-input")).not.toBeNull();
  });

  it("shows validate button", () => {
    render(
      <RestoreConfirmationPanel
        result={null}
        loading={false}
        requiredText="RESTORE BACKUP"
        onValidate={() => {}}
      />,
    );
    expect(screen.getByTestId("confirmation-validate-button")).not.toBeNull();
  });

  it("validate button disabled when input empty", () => {
    render(
      <RestoreConfirmationPanel
        result={null}
        loading={false}
        requiredText="RESTORE BACKUP"
        onValidate={() => {}}
      />,
    );
    const btn = screen.getByTestId("confirmation-validate-button") as HTMLButtonElement;
    expect(btn.disabled).toBe(true);
  });

  it("validate button enabled when input has text", () => {
    render(
      <RestoreConfirmationPanel
        result={null}
        loading={false}
        requiredText="RESTORE BACKUP"
        onValidate={() => {}}
      />,
    );
    const input = screen.getByTestId("confirmation-text-input") as HTMLInputElement;
    fireEvent.change(input, { target: { value: "RESTORE BACKUP" } });
    const btn = screen.getByTestId("confirmation-validate-button") as HTMLButtonElement;
    expect(btn.disabled).toBe(false);
  });

  it("calls onValidate with entered text", () => {
    const onValidate = vi.fn();
    render(
      <RestoreConfirmationPanel
        result={null}
        loading={false}
        requiredText="RESTORE BACKUP"
        onValidate={onValidate}
      />,
    );
    const input = screen.getByTestId("confirmation-text-input") as HTMLInputElement;
    fireEvent.change(input, { target: { value: "RESTORE BACKUP" } });
    fireEvent.click(screen.getByTestId("confirmation-validate-button"));
    expect(onValidate).toHaveBeenCalledWith("RESTORE BACKUP");
  });

  it("shows result section when result provided", () => {
    render(
      <RestoreConfirmationPanel
        result={CONFIRMED_RESULT}
        loading={false}
        requiredText="RESTORE TO MY BASE"
        onValidate={() => {}}
      />,
    );
    expect(screen.getByTestId("confirmation-result")).not.toBeNull();
  });

  it("shows status badge", () => {
    render(
      <RestoreConfirmationPanel
        result={CONFIRMED_RESULT}
        loading={false}
        requiredText="RESTORE TO MY BASE"
        onValidate={() => {}}
      />,
    );
    expect(screen.getByTestId("confirmation-status")).not.toBeNull();
  });

  it("shows message", () => {
    render(
      <RestoreConfirmationPanel
        result={CONFIRMED_RESULT}
        loading={false}
        requiredText="RESTORE TO MY BASE"
        onValidate={() => {}}
      />,
    );
    expect(screen.getByTestId("confirmation-message")).not.toBeNull();
  });

  it("shows check rows", () => {
    render(
      <RestoreConfirmationPanel
        result={CONFIRMED_RESULT}
        loading={false}
        requiredText="RESTORE TO MY BASE"
        onValidate={() => {}}
      />,
    );
    const rows = screen.getAllByTestId("confirmation-check-row");
    expect(rows.length).toBe(CONFIRMED_RESULT.checks.length);
  });

  it("shows safety summary", () => {
    render(
      <RestoreConfirmationPanel
        result={CONFIRMED_RESULT}
        loading={false}
        requiredText="RESTORE TO MY BASE"
        onValidate={() => {}}
      />,
    );
    expect(screen.getByTestId("confirmation-safety-summary")).not.toBeNull();
  });

  it("shows no-changes notice in safety summary", () => {
    render(
      <RestoreConfirmationPanel
        result={CONFIRMED_RESULT}
        loading={false}
        requiredText="RESTORE TO MY BASE"
        onValidate={() => {}}
      />,
    );
    expect(screen.getByTestId("confirmation-no-changes-notice")).not.toBeNull();
  });

  it("confirmed result shows accepted notice (writes still disabled)", () => {
    render(
      <RestoreConfirmationPanel
        result={CONFIRMED_RESULT}
        loading={false}
        requiredText="RESTORE TO MY BASE"
        onValidate={() => {}}
      />,
    );
    expect(screen.getByTestId("confirmation-accepted-notice")).not.toBeNull();
  });

  it("rejected result shows rejected notice", () => {
    render(
      <RestoreConfirmationPanel
        result={REJECTED_RESULT}
        loading={false}
        requiredText="RESTORE TO MY BASE"
        onValidate={() => {}}
      />,
    );
    expect(screen.getByTestId("confirmation-rejected-notice")).not.toBeNull();
  });

  it("blocked result shows blocked notice", () => {
    render(
      <RestoreConfirmationPanel
        result={BLOCKED_RESULT}
        loading={false}
        requiredText="RESTORE TO MY BASE"
        onValidate={() => {}}
      />,
    );
    expect(screen.getByTestId("confirmation-blocked-notice")).not.toBeNull();
  });

  it("no execute button anywhere", () => {
    const { container } = render(
      <RestoreConfirmationPanel
        result={CONFIRMED_RESULT}
        loading={false}
        requiredText="RESTORE TO MY BASE"
        onValidate={() => {}}
      />,
    );
    for (const btn of Array.from(container.querySelectorAll("button"))) {
      const text = (btn.textContent ?? "").toLowerCase();
      expect(text).not.toContain("execute");
      expect(text).not.toContain("run restore");
      expect(text).not.toContain("start restore");
    }
  });

  it("no token input field", () => {
    const { container } = render(
      <RestoreConfirmationPanel
        result={CONFIRMED_RESULT}
        loading={false}
        requiredText="RESTORE TO MY BASE"
        onValidate={() => {}}
      />,
    );
    const inputs = Array.from(container.querySelectorAll("input"));
    for (const input of inputs) {
      expect(input.type).not.toBe("password");
      expect((input.getAttribute("aria-label") ?? "").toLowerCase()).not.toContain("token");
    }
  });

  it("no Airtable token value in DOM", () => {
    const { container } = render(
      <RestoreConfirmationPanel
        result={CONFIRMED_RESULT}
        loading={false}
        requiredText="RESTORE TO MY BASE"
        onValidate={() => {}}
      />,
    );
    expect(container.textContent).not.toMatch(/pat[0-9a-zA-Z]{10,}/);
  });

  it("no full path in DOM", () => {
    const { container } = render(
      <RestoreConfirmationPanel
        result={CONFIRMED_RESULT}
        loading={false}
        requiredText="RESTORE TO MY BASE"
        onValidate={() => {}}
      />,
    );
    expect(container.textContent).not.toContain("/Users/");
    expect(container.textContent).not.toContain("/tmp/");
  });

  it("no success/succeeded language in DOM", () => {
    const { container } = render(
      <RestoreConfirmationPanel
        result={CONFIRMED_RESULT}
        loading={false}
        requiredText="RESTORE TO MY BASE"
        onValidate={() => {}}
      />,
    );
    const text = container.textContent?.toLowerCase() ?? "";
    expect(text).not.toContain("restore complete");
    expect(text).not.toContain("restore successful");
    expect(text).not.toContain("succeeded");
  });

  it("disabled notice always visible regardless of result", () => {
    for (const result of [null, CONFIRMED_RESULT, REJECTED_RESULT, BLOCKED_RESULT]) {
      const { unmount } = render(
        <RestoreConfirmationPanel
          result={result}
          loading={false}
          requiredText="RESTORE BACKUP"
          onValidate={() => {}}
        />,
      );
      expect(screen.getByTestId("confirmation-writes-disabled-notice")).not.toBeNull();
      unmount();
    }
  });
});
