import { describe, it, expect, vi } from "vitest";
import { render, screen, fireEvent } from "@testing-library/react";
import { RestoreAttachmentUploadPolicyPanel } from "../features/backups/RestoreAttachmentUploadPolicyPanel";
import { mockAirBridgeService } from "../services/mockAirBridgeService";
import type {
  AttachmentUploadPolicyRequest,
  AttachmentUploadPolicyResult,
  DeclaredAttachmentField,
} from "../backend/types";

// ── Helpers ───────────────────────────────────────────────────────────────────

function makeField(
  intent: DeclaredAttachmentField["intent"],
  fieldName: string,
  tableName = "Table1",
): DeclaredAttachmentField {
  return { fieldName, tableName, intent };
}

const METADATA_ONLY_FIELDS: DeclaredAttachmentField[] = [
  makeField("metadataOnly", "Attachments", "Projects"),
  makeField("metadataOnly", "Docs", "Tasks"),
];

function req(fields: DeclaredAttachmentField[], name?: string): AttachmentUploadPolicyRequest {
  return { declaredAttachmentFields: fields, targetDisplayName: name ?? "My Base" };
}

// ── Mock service contract ─────────────────────────────────────────────────────

describe("mockAirBridgeService — verifyAttachmentUploadPolicy contract", () => {
  it("metadata-only fields return compliant", async () => {
    const result = await mockAirBridgeService.verifyAttachmentUploadPolicy(
      req(METADATA_ONLY_FIELDS),
    );
    expect(result.status).toBe("compliant");
  });

  it("empty fields list returns compliant", async () => {
    const result = await mockAirBridgeService.verifyAttachmentUploadPolicy(req([]));
    expect(result.status).toBe("compliant");
  });

  it("uploadRequested field returns blocked", async () => {
    const result = await mockAirBridgeService.verifyAttachmentUploadPolicy(
      req([makeField("uploadRequested", "Photos")]),
    );
    expect(result.status).toBe("blocked");
  });

  it("downloadRequested field returns warning", async () => {
    const result = await mockAirBridgeService.verifyAttachmentUploadPolicy(
      req([makeField("downloadRequested", "Files")]),
    );
    expect(result.status).toBe("warning");
  });

  it("unknown field returns warning", async () => {
    const result = await mockAirBridgeService.verifyAttachmentUploadPolicy(
      req([makeField("unknown", "Misc")]),
    );
    expect(result.status).toBe("warning");
  });

  it("mixed upload and metadata returns blocked (upload takes priority)", async () => {
    const result = await mockAirBridgeService.verifyAttachmentUploadPolicy(
      req([makeField("uploadRequested", "Photos"), makeField("metadataOnly", "Docs")]),
    );
    expect(result.status).toBe("blocked");
  });

  it("five checks present for any result", async () => {
    const result = await mockAirBridgeService.verifyAttachmentUploadPolicy(
      req(METADATA_ONLY_FIELDS),
    );
    expect(result.checks).toHaveLength(5);
  });

  it("check IDs are AUP-01 through AUP-05", async () => {
    const result = await mockAirBridgeService.verifyAttachmentUploadPolicy(
      req(METADATA_ONLY_FIELDS),
    );
    const ids = result.checks.map((c) => c.checkId);
    expect(ids).toContain("AUP-01");
    expect(ids).toContain("AUP-02");
    expect(ids).toContain("AUP-03");
    expect(ids).toContain("AUP-04");
    expect(ids).toContain("AUP-05");
  });

  it("AUP-01 always passes", async () => {
    const result = await mockAirBridgeService.verifyAttachmentUploadPolicy(req([]));
    const aup01 = result.checks.find((c) => c.checkId === "AUP-01");
    expect(aup01?.status).toBe("passed");
  });

  it("AUP-02 fails for uploadRequested", async () => {
    const result = await mockAirBridgeService.verifyAttachmentUploadPolicy(
      req([makeField("uploadRequested", "Photos")]),
    );
    const aup02 = result.checks.find((c) => c.checkId === "AUP-02");
    expect(aup02?.status).toBe("failed");
  });

  it("AUP-02 passes when no uploadRequested", async () => {
    const result = await mockAirBridgeService.verifyAttachmentUploadPolicy(
      req(METADATA_ONLY_FIELDS),
    );
    const aup02 = result.checks.find((c) => c.checkId === "AUP-02");
    expect(aup02?.status).toBe("passed");
  });

  it("AUP-03 warns for downloadRequested", async () => {
    const result = await mockAirBridgeService.verifyAttachmentUploadPolicy(
      req([makeField("downloadRequested", "Files")]),
    );
    const aup03 = result.checks.find((c) => c.checkId === "AUP-03");
    expect(aup03?.status).toBe("warning");
  });

  it("AUP-04 warns for unknown intents", async () => {
    const result = await mockAirBridgeService.verifyAttachmentUploadPolicy(
      req([makeField("unknown", "Misc")]),
    );
    const aup04 = result.checks.find((c) => c.checkId === "AUP-04");
    expect(aup04?.status).toBe("warning");
  });

  it("AUP-05 passes for all metadata-only", async () => {
    const result = await mockAirBridgeService.verifyAttachmentUploadPolicy(
      req(METADATA_ONLY_FIELDS),
    );
    const aup05 = result.checks.find((c) => c.checkId === "AUP-05");
    expect(aup05?.status).toBe("passed");
  });

  it("blockedFieldNames empty for compliant", async () => {
    const result = await mockAirBridgeService.verifyAttachmentUploadPolicy(
      req(METADATA_ONLY_FIELDS),
    );
    expect(result.blockedFieldNames).toHaveLength(0);
  });

  it("blockedFieldNames contains table.field for uploadRequested", async () => {
    const result = await mockAirBridgeService.verifyAttachmentUploadPolicy(
      req([makeField("uploadRequested", "Photos", "Projects")]),
    );
    expect(result.blockedFieldNames).toContain("Projects.Photos");
  });

  it("metadataOnlyFieldCount correct for two metadata fields", async () => {
    const result = await mockAirBridgeService.verifyAttachmentUploadPolicy(
      req(METADATA_ONLY_FIELDS),
    );
    expect(result.metadataOnlyFieldCount).toBe(2);
  });

  it("metadataOnlyFieldCount zero when no metadata fields", async () => {
    const result = await mockAirBridgeService.verifyAttachmentUploadPolicy(
      req([makeField("uploadRequested", "Photos")]),
    );
    expect(result.metadataOnlyFieldCount).toBe(0);
  });

  it("noChangesMade always true", async () => {
    const cases = [req(METADATA_ONLY_FIELDS), req([makeField("uploadRequested", "P")]), req([])];
    for (const r of cases) {
      const result = await mockAirBridgeService.verifyAttachmentUploadPolicy(r);
      expect(result.noChangesMade).toBe(true);
    }
  });

  it("writesEnabled always false", async () => {
    const cases = [req(METADATA_ONLY_FIELDS), req([makeField("uploadRequested", "P")])];
    for (const r of cases) {
      const result = await mockAirBridgeService.verifyAttachmentUploadPolicy(r);
      expect(result.writesEnabled).toBe(false);
    }
  });

  it("networkWritesAttempted always false", async () => {
    const result = await mockAirBridgeService.verifyAttachmentUploadPolicy(
      req(METADATA_ONLY_FIELDS),
    );
    expect(result.networkWritesAttempted).toBe(false);
  });

  it("no token in message for any status", async () => {
    const cases = [req(METADATA_ONLY_FIELDS), req([makeField("uploadRequested", "P")])];
    for (const r of cases) {
      const result = await mockAirBridgeService.verifyAttachmentUploadPolicy(r);
      expect(result.message).not.toContain("token");
      expect(result.message).not.toContain("pat");
    }
  });

  it("no full URL in message", async () => {
    const result = await mockAirBridgeService.verifyAttachmentUploadPolicy(
      req(METADATA_ONLY_FIELDS),
    );
    expect(result.message).not.toContain("dl.airtable.com");
    expect(result.message).not.toContain("airtableusercontent.com");
  });

  it("no full path in message", async () => {
    const result = await mockAirBridgeService.verifyAttachmentUploadPolicy(
      req(METADATA_ONLY_FIELDS),
    );
    expect(result.message).not.toContain("/Users/");
    expect(result.message).not.toContain("/home/");
  });

  it("compliant message says writes remain disabled", async () => {
    const result = await mockAirBridgeService.verifyAttachmentUploadPolicy(
      req(METADATA_ONLY_FIELDS),
    );
    expect(result.message).toContain("disabled");
  });

  it("blocked message names blocked field", async () => {
    const result = await mockAirBridgeService.verifyAttachmentUploadPolicy(
      req([makeField("uploadRequested", "Photos", "Projects")]),
    );
    expect(result.message).toContain("Projects.Photos");
  });

  it("IPC fallback shape (null result) has safety invariants", () => {
    const fallback: AttachmentUploadPolicyResult = {
      status: "blocked",
      checks: [],
      message: "Attachment upload policy check is not available in this context.",
      blockedFieldNames: [],
      metadataOnlyFieldCount: 0,
      noChangesMade: true,
      networkWritesAttempted: false,
      writesEnabled: false,
    };
    expect(fallback.writesEnabled).toBe(false);
    expect(fallback.noChangesMade).toBe(true);
    expect(fallback.networkWritesAttempted).toBe(false);
  });
});

// ── Panel rendering ───────────────────────────────────────────────────────────

function renderPanel(
  result: AttachmentUploadPolicyResult | null,
  loading = false,
  onVerify = vi.fn(),
) {
  return render(
    <RestoreAttachmentUploadPolicyPanel result={result} loading={loading} onVerify={onVerify} />,
  );
}

const COMPLIANT_RESULT: AttachmentUploadPolicyResult = {
  status: "compliant",
  checks: [
    {
      checkId: "AUP-01",
      label: "write-gate-disabled",
      status: "passed",
      message: "Gate disabled.",
    },
    {
      checkId: "AUP-02",
      label: "no-upload-requested",
      status: "passed",
      message: "No uploads.",
    },
    {
      checkId: "AUP-03",
      label: "no-download-requested",
      status: "passed",
      message: "No downloads.",
    },
    {
      checkId: "AUP-04",
      label: "no-unknown-intents",
      status: "passed",
      message: "All known.",
    },
    {
      checkId: "AUP-05",
      label: "metadata-only-confirmed",
      status: "passed",
      message: "All metadata-only.",
    },
  ],
  message:
    "All 2 declared attachment field(s) for My Base use metadata-only handling. Restore writes remain disabled.",
  blockedFieldNames: [],
  metadataOnlyFieldCount: 2,
  noChangesMade: true,
  networkWritesAttempted: false,
  writesEnabled: false,
};

const BLOCKED_RESULT: AttachmentUploadPolicyResult = {
  status: "blocked",
  checks: [
    {
      checkId: "AUP-01",
      label: "write-gate-disabled",
      status: "passed",
      message: "Gate disabled.",
    },
    {
      checkId: "AUP-02",
      label: "no-upload-requested",
      status: "failed",
      message: "Upload detected.",
      remediation: "Change to metadata-only.",
    },
    {
      checkId: "AUP-03",
      label: "no-download-requested",
      status: "passed",
      message: "No downloads.",
    },
    {
      checkId: "AUP-04",
      label: "no-unknown-intents",
      status: "passed",
      message: "All known.",
    },
    {
      checkId: "AUP-05",
      label: "metadata-only-confirmed",
      status: "failed",
      message: "Not metadata-only.",
    },
  ],
  message: "Attachment upload is not permitted for My Base: Projects.Photos.",
  blockedFieldNames: ["Projects.Photos"],
  metadataOnlyFieldCount: 0,
  noChangesMade: true,
  networkWritesAttempted: false,
  writesEnabled: false,
};

describe("RestoreAttachmentUploadPolicyPanel rendering", () => {
  it("panel testid present", () => {
    renderPanel(null);
    expect(screen.getByTestId("restore-aup-panel")).toBeInTheDocument();
  });

  it("writes disabled notice always shown", () => {
    renderPanel(null);
    expect(screen.getByTestId("aup-writes-disabled-notice")).toBeInTheDocument();
  });

  it("verify button present", () => {
    renderPanel(null);
    expect(screen.getByTestId("aup-verify-button")).toBeInTheDocument();
  });

  it("button disabled when loading", () => {
    renderPanel(null, true);
    expect(screen.getByTestId("aup-verify-button")).toBeDisabled();
  });

  it("button shows re-verify label when result present", () => {
    renderPanel(COMPLIANT_RESULT);
    expect(screen.getByTestId("aup-verify-button")).toHaveTextContent("Re-verify");
  });

  it("onVerify callback fired on button click", () => {
    const onVerify = vi.fn();
    renderPanel(null, false, onVerify);
    fireEvent.click(screen.getByTestId("aup-verify-button"));
    expect(onVerify).toHaveBeenCalledOnce();
  });

  it("no result area before verify", () => {
    renderPanel(null);
    expect(screen.queryByTestId("aup-result")).not.toBeInTheDocument();
  });

  it("result area shown after verify", () => {
    renderPanel(COMPLIANT_RESULT);
    expect(screen.getByTestId("aup-result")).toBeInTheDocument();
  });

  it("compliant badge shown for compliant status", () => {
    renderPanel(COMPLIANT_RESULT);
    expect(screen.getByTestId("aup-compliant-badge")).toBeInTheDocument();
  });

  it("blocked badge shown for blocked status", () => {
    renderPanel(BLOCKED_RESULT);
    expect(screen.getByTestId("aup-blocked-badge")).toBeInTheDocument();
  });

  it("warning badge shown for warning status", () => {
    const warnResult: AttachmentUploadPolicyResult = {
      ...COMPLIANT_RESULT,
      status: "warning",
      message: "Some attachment fields have deferred intent.",
    };
    renderPanel(warnResult);
    expect(screen.getByTestId("aup-warning-badge")).toBeInTheDocument();
  });

  it("message text rendered", () => {
    renderPanel(COMPLIANT_RESULT);
    expect(screen.getByTestId("aup-message")).toHaveTextContent("metadata-only");
  });

  it("check rows rendered", () => {
    renderPanel(COMPLIANT_RESULT);
    const rows = screen.getAllByTestId("aup-check-row");
    expect(rows).toHaveLength(5);
  });

  it("safety summary rendered", () => {
    renderPanel(COMPLIANT_RESULT);
    expect(screen.getByTestId("aup-safety-summary")).toBeInTheDocument();
  });

  it("no-changes notice rendered", () => {
    renderPanel(COMPLIANT_RESULT);
    expect(screen.getByTestId("aup-no-changes-notice")).toBeInTheDocument();
  });

  it("compliant-notice shown only for compliant", () => {
    renderPanel(COMPLIANT_RESULT);
    expect(screen.getByTestId("aup-compliant-notice")).toBeInTheDocument();
    expect(screen.queryByTestId("aup-warning-notice")).not.toBeInTheDocument();
    expect(screen.queryByTestId("aup-blocked-notice")).not.toBeInTheDocument();
  });

  it("blocked-notice shown only for blocked", () => {
    renderPanel(BLOCKED_RESULT);
    expect(screen.getByTestId("aup-blocked-notice")).toBeInTheDocument();
    expect(screen.queryByTestId("aup-compliant-notice")).not.toBeInTheDocument();
    expect(screen.queryByTestId("aup-warning-notice")).not.toBeInTheDocument();
  });

  it("compliant notice says writes remain disabled", () => {
    renderPanel(COMPLIANT_RESULT);
    const notice = screen.getByTestId("aup-compliant-notice");
    expect(notice).toHaveTextContent("disabled");
  });

  it("no execute button present", () => {
    renderPanel(COMPLIANT_RESULT);
    expect(screen.queryByText(/execute/i)).not.toBeInTheDocument();
    expect(screen.queryByText(/run restore/i)).not.toBeInTheDocument();
  });

  it("no token input present", () => {
    renderPanel(COMPLIANT_RESULT);
    expect(screen.queryByLabelText(/token/i)).not.toBeInTheDocument();
    expect(screen.queryByPlaceholderText(/token/i)).not.toBeInTheDocument();
  });

  it("no succeeded language in panel", () => {
    renderPanel(COMPLIANT_RESULT);
    expect(screen.queryByText(/succeeded/i)).not.toBeInTheDocument();
    expect(screen.queryByText(/restore complete/i)).not.toBeInTheDocument();
  });

  it("blocked fields list shown when present", () => {
    renderPanel(BLOCKED_RESULT);
    expect(screen.getByTestId("aup-blocked-fields-list")).toBeInTheDocument();
    expect(screen.getByTestId("aup-blocked-field-item")).toHaveTextContent("Projects.Photos");
  });

  it("blocked fields list not shown when empty", () => {
    renderPanel(COMPLIANT_RESULT);
    expect(screen.queryByTestId("aup-blocked-fields-list")).not.toBeInTheDocument();
  });

  it("safety summary shows metadataOnlyFields count", () => {
    renderPanel(COMPLIANT_RESULT);
    const summary = screen.getByTestId("aup-safety-summary");
    expect(summary).toHaveTextContent("metadataOnlyFields: 2");
  });

  it("no-changes notice mentions attachment bytes not uploaded", () => {
    renderPanel(COMPLIANT_RESULT);
    const notice = screen.getByTestId("aup-no-changes-notice");
    expect(notice).toHaveTextContent("Attachment file bytes have not been uploaded");
  });
});
