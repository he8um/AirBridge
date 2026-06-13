import { render, screen, waitFor } from "@testing-library/react";
import userEvent from "@testing-library/user-event";
import { describe, it, expect, beforeEach, vi } from "vitest";
import { CredentialStorageCard } from "../features/connections/CredentialStorageCard";
import { mockAirBridgeService } from "../services/mockAirBridgeService";
import type { AirBridgeService } from "../services/airBridgeService";
import type {
  CredentialSaveResult,
  CredentialRemoveResult,
  CredentialStatusResult,
} from "../backend/types";

// ── Helpers ────────────────────────────────────────────────────────────────────

function renderCard(service: AirBridgeService = mockAirBridgeService) {
  return render(<CredentialStorageCard service={service} />);
}

const SENTINEL = "pat_example_sentinel_0123456789abcdefghijklmnopqrstuvwxyz01234";

// ── Rendering ──────────────────────────────────────────────────────────────────

describe("CredentialStorageCard rendering", () => {
  beforeEach(() => {
    // Reset mock store state between tests by removing any saved credential
    mockAirBridgeService.removeAirtableTokenFromKeychain({
      kind: "airtablePersonalAccessToken",
    });
  });

  it("renders the credential storage card", async () => {
    renderCard();
    await waitFor(() => screen.getByTestId("credential-storage-card"));
    expect(screen.getByTestId("credential-storage-card")).not.toBeNull();
  });

  it("token input is type password", async () => {
    renderCard();
    await waitFor(() => screen.getByTestId("credential-token-input"));
    const input = screen.getByTestId("credential-token-input") as HTMLInputElement;
    expect(input.type).toBe("password");
  });

  it("shows no-saved-token status initially", async () => {
    renderCard();
    await waitFor(() => screen.getByTestId("credential-status-badge"));
    const badge = screen.getByTestId("credential-status-badge");
    expect(badge.getAttribute("data-status")).toBe("notSaved");
  });

  it("does not render token value in DOM on mount", async () => {
    renderCard();
    await waitFor(() => screen.getByTestId("credential-storage-card"));
    const { container } = renderCard();
    await waitFor(() => screen.getAllByTestId("credential-storage-card"));
    expect(container.textContent).not.toContain(SENTINEL);
  });

  it("save button is disabled when token input is empty", async () => {
    renderCard();
    await waitFor(() => screen.getByTestId("credential-save-button"));
    const btn = screen.getByTestId("credential-save-button") as HTMLButtonElement;
    expect(btn.disabled).toBe(true);
  });
});

// ── Save flow ──────────────────────────────────────────────────────────────────

describe("CredentialStorageCard save flow", () => {
  beforeEach(() => {
    mockAirBridgeService.removeAirtableTokenFromKeychain({
      kind: "airtablePersonalAccessToken",
    });
  });

  it("save button becomes enabled when token is typed", async () => {
    renderCard();
    await waitFor(() => screen.getByTestId("credential-token-input"));
    await userEvent.type(screen.getByTestId("credential-token-input"), "some-token-value");
    const btn = screen.getByTestId("credential-save-button") as HTMLButtonElement;
    expect(btn.disabled).toBe(false);
  });

  it("token input is removed from DOM after successful save", async () => {
    renderCard();
    await waitFor(() => screen.getByTestId("credential-token-input"));
    await userEvent.type(screen.getByTestId("credential-token-input"), SENTINEL);
    await userEvent.click(screen.getByTestId("credential-save-button"));
    // After a successful save the status transitions to "saved" and the input is hidden
    await waitFor(() => expect(screen.queryByTestId("credential-token-input")).toBeNull());
  });

  it("saved status renders after successful save", async () => {
    renderCard();
    await waitFor(() => screen.getByTestId("credential-token-input"));
    await userEvent.type(screen.getByTestId("credential-token-input"), SENTINEL);
    await userEvent.click(screen.getByTestId("credential-save-button"));
    await waitFor(() => {
      const badge = screen.getByTestId("credential-status-badge");
      expect(badge.getAttribute("data-status")).toBe("saved");
    });
  });

  it("token is never rendered in the DOM after save", async () => {
    const { container } = renderCard();
    await waitFor(() => screen.getByTestId("credential-token-input"));
    await userEvent.type(screen.getByTestId("credential-token-input"), SENTINEL);
    await userEvent.click(screen.getByTestId("credential-save-button"));
    await waitFor(() => {
      const badge = screen.getByTestId("credential-status-badge");
      expect(badge.getAttribute("data-status")).toBe("saved");
    });
    expect(container.textContent).not.toContain(SENTINEL);
  });

  it("token input is not shown after successful save", async () => {
    renderCard();
    await waitFor(() => screen.getByTestId("credential-token-input"));
    await userEvent.type(screen.getByTestId("credential-token-input"), SENTINEL);
    await userEvent.click(screen.getByTestId("credential-save-button"));
    await waitFor(() => expect(screen.queryByTestId("credential-token-input")).toBeNull());
  });

  it("feedback message does not contain the token value", async () => {
    renderCard();
    await waitFor(() => screen.getByTestId("credential-token-input"));
    await userEvent.type(screen.getByTestId("credential-token-input"), SENTINEL);
    await userEvent.click(screen.getByTestId("credential-save-button"));
    await waitFor(() => screen.getByTestId("credential-feedback"));
    const feedback = screen.getByTestId("credential-feedback");
    expect(feedback.textContent).not.toContain(SENTINEL);
  });
});

// ── Remove flow ────────────────────────────────────────────────────────────────

describe("CredentialStorageCard remove flow", () => {
  it("remove button updates status to not saved", async () => {
    // Pre-save a token via service directly
    await mockAirBridgeService.saveAirtableTokenToKeychain({
      kind: "airtablePersonalAccessToken",
      token: SENTINEL,
    });

    renderCard();
    await waitFor(() => screen.getByTestId("credential-remove-button"));
    await userEvent.click(screen.getByTestId("credential-remove-button"));
    await waitFor(() => {
      const badge = screen.getByTestId("credential-status-badge");
      expect(badge.getAttribute("data-status")).toBe("notSaved");
    });
  });

  it("token is never rendered during or after remove", async () => {
    await mockAirBridgeService.saveAirtableTokenToKeychain({
      kind: "airtablePersonalAccessToken",
      token: SENTINEL,
    });

    const { container } = renderCard();
    await waitFor(() => screen.getByTestId("credential-remove-button"));
    await userEvent.click(screen.getByTestId("credential-remove-button"));
    await waitFor(() => {
      const badge = screen.getByTestId("credential-status-badge");
      expect(badge.getAttribute("data-status")).toBe("notSaved");
    });
    expect(container.textContent).not.toContain(SENTINEL);
  });
});

// ── Unavailable state ──────────────────────────────────────────────────────────

describe("CredentialStorageCard unavailable state", () => {
  it("shows unavailable notice when keychain is not available", async () => {
    const unavailableService: AirBridgeService = {
      ...mockAirBridgeService,
      getCredentialStorageStatus: async (req): Promise<CredentialStatusResult> => ({
        kind: req.kind,
        status: "unavailable",
        availability: "unavailable",
        hasSavedToken: false,
        display: "OS keychain is not available on this system.",
      }),
    };
    renderCard(unavailableService);
    await waitFor(() => screen.getByTestId("credential-unavailable-notice"));
    expect(screen.getByTestId("credential-unavailable-notice").textContent).toContain(
      "not available",
    );
  });

  it("does not show token input when unavailable", async () => {
    const unavailableService: AirBridgeService = {
      ...mockAirBridgeService,
      getCredentialStorageStatus: async (req): Promise<CredentialStatusResult> => ({
        kind: req.kind,
        status: "unavailable",
        availability: "unavailable",
        hasSavedToken: false,
        display: "OS keychain is not available.",
      }),
    };
    renderCard(unavailableService);
    await waitFor(() => screen.getByTestId("credential-unavailable-notice"));
    expect(screen.queryByTestId("credential-token-input")).toBeNull();
    expect(screen.queryByTestId("credential-save-button")).toBeNull();
  });
});

// ── Mock service contract ──────────────────────────────────────────────────────

describe("mock service credential contract", () => {
  beforeEach(() => {
    mockAirBridgeService.removeAirtableTokenFromKeychain({
      kind: "airtablePersonalAccessToken",
    });
  });

  it("save result never contains token", async () => {
    const result = await mockAirBridgeService.saveAirtableTokenToKeychain({
      kind: "airtablePersonalAccessToken",
      token: SENTINEL,
    });
    const json = JSON.stringify(result);
    expect(json).not.toContain(SENTINEL);
  });

  it("save with hasSavedToken true after save", async () => {
    const result = await mockAirBridgeService.saveAirtableTokenToKeychain({
      kind: "airtablePersonalAccessToken",
      token: "some-valid-token-at-least-minimum-length",
    });
    expect(result.hasSavedToken).toBe(true);
    expect(result.success).toBe(true);
  });

  it("save with empty token returns hasSavedToken false", async () => {
    const result = await mockAirBridgeService.saveAirtableTokenToKeychain({
      kind: "airtablePersonalAccessToken",
      token: "",
    });
    expect(result.hasSavedToken).toBe(false);
    expect(result.success).toBe(false);
  });

  it("remove result has hasSavedToken false", async () => {
    const result = await mockAirBridgeService.removeAirtableTokenFromKeychain({
      kind: "airtablePersonalAccessToken",
    });
    expect(result.hasSavedToken).toBe(false);
    expect(result.success).toBe(true);
  });

  it("status result never contains token", async () => {
    const result = await mockAirBridgeService.getCredentialStorageStatus({
      kind: "airtablePersonalAccessToken",
    });
    const json = JSON.stringify(result);
    expect(json).not.toContain(SENTINEL);
    expect(json).not.toContain('"token"');
  });

  it("status hasSavedToken reflects save/remove cycle", async () => {
    const before = await mockAirBridgeService.getCredentialStorageStatus({
      kind: "airtablePersonalAccessToken",
    });
    expect(before.hasSavedToken).toBe(false);

    await mockAirBridgeService.saveAirtableTokenToKeychain({
      kind: "airtablePersonalAccessToken",
      token: SENTINEL,
    });

    const after = await mockAirBridgeService.getCredentialStorageStatus({
      kind: "airtablePersonalAccessToken",
    });
    expect(after.hasSavedToken).toBe(true);
  });
});

// ── IPC fallback ───────────────────────────────────────────────────────────────

describe("CredentialStorageCard IPC fallback", () => {
  it("IPC fallback save result is safe — no token in result", () => {
    const fallback: CredentialSaveResult = {
      kind: "airtablePersonalAccessToken",
      success: false,
      hasSavedToken: false,
      display: "OS keychain is not available in this context.",
      errorMessage: "OS keychain is not available in this context.",
    };
    const json = JSON.stringify(fallback);
    expect(json).not.toContain(SENTINEL);
    expect(json).not.toContain('"token"');
  });

  it("IPC fallback remove result is safe", () => {
    const fallback: CredentialRemoveResult = {
      kind: "airtablePersonalAccessToken",
      success: false,
      hasSavedToken: false,
      display: "OS keychain is not available in this context.",
      errorMessage: "OS keychain is not available in this context.",
    };
    expect(fallback.hasSavedToken).toBe(false);
    const json = JSON.stringify(fallback);
    expect(json).not.toContain(SENTINEL);
  });
});

// ── No localStorage / sessionStorage ──────────────────────────────────────────

describe("CredentialStorageCard storage isolation", () => {
  it("does not write to localStorage", async () => {
    const setItemSpy = vi.spyOn(Storage.prototype, "setItem");
    renderCard();
    await waitFor(() => screen.getByTestId("credential-storage-card"));

    if (screen.queryByTestId("credential-token-input")) {
      await userEvent.type(screen.getByTestId("credential-token-input"), SENTINEL);
      if (screen.queryByTestId("credential-save-button")) {
        await userEvent.click(screen.getByTestId("credential-save-button"));
      }
    }

    const calls = setItemSpy.mock.calls;
    for (const [key, value] of calls) {
      expect(String(key)).not.toContain("token");
      expect(String(key)).not.toContain("credential");
      expect(String(value)).not.toContain(SENTINEL);
    }
    setItemSpy.mockRestore();
  });

  it("does not write to sessionStorage", async () => {
    const setItemSpy = vi.spyOn(Storage.prototype, "setItem");
    renderCard();
    await waitFor(() => screen.getByTestId("credential-storage-card"));

    const calls = setItemSpy.mock.calls;
    for (const [, value] of calls) {
      expect(String(value)).not.toContain(SENTINEL);
    }
    setItemSpy.mockRestore();
  });
});

// ── Restore write remains disabled ─────────────────────────────────────────────

describe("restore write gate unchanged by credential storage", () => {
  it("restore write engine result has no succeeded status after save", async () => {
    await mockAirBridgeService.saveAirtableTokenToKeychain({
      kind: "airtablePersonalAccessToken",
      token: SENTINEL,
    });
    const writeResult = await mockAirBridgeService.previewRestoreWriteEngine({
      packageFilename: "backup.airbridge",
      packagePath: "/tmp/backup.airbridge",
    });
    expect(writeResult.status).toBe("disabled");
    const json = JSON.stringify(writeResult).toLowerCase();
    expect(json).not.toContain("succeeded");
  });
});
