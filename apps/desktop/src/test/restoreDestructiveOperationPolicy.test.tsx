import { describe, it, expect, vi } from "vitest";
import { render, screen, fireEvent } from "@testing-library/react";
import { RestoreDestructiveOperationPolicyPanel } from "../features/backups/RestoreDestructiveOperationPolicyPanel";
import { mockAirBridgeService } from "../services/mockAirBridgeService";
import type {
  DestructiveOperationPolicyRequest,
  DestructiveOperationPolicyResult,
  DeclaredOperation,
} from "../backend/types";

// ── Helpers ───────────────────────────────────────────────────────────────────

function makeOp(kind: DeclaredOperation["kind"], label: string): DeclaredOperation {
  return { kind, label };
}

const CREATE_ONLY_OPS: DeclaredOperation[] = [
  makeOp("createTable", "create-table-Projects"),
  makeOp("createField", "create-field-Name"),
  makeOp("createRecord", "create-record-batch-1"),
  makeOp("updateLinkedRecordReference", "update-linked-refs"),
  makeOp("preserveAttachmentMetadata", "preserve-attachment"),
  makeOp("checkpoint", "checkpoint-1"),
  makeOp("skipField", "skip-formula"),
  makeOp("manualAction", "manual-link"),
  makeOp("deferLinkedField", "defer-linked"),
];

function req(ops: DeclaredOperation[], name?: string): DestructiveOperationPolicyRequest {
  return { declaredOperations: ops, targetDisplayName: name ?? "My Base" };
}

// ── Mock service contract ─────────────────────────────────────────────────────

describe("mockAirBridgeService — verifyDestructiveOperationPolicy contract", () => {
  it("create-only operations return compliant", async () => {
    const result = await mockAirBridgeService.verifyDestructiveOperationPolicy(
      req(CREATE_ONLY_OPS),
    );
    expect(result.status).toBe("compliant");
  });

  it("empty operations list returns compliant", async () => {
    const result = await mockAirBridgeService.verifyDestructiveOperationPolicy(req([]));
    expect(result.status).toBe("compliant");
  });

  it("deleteTable returns blocked", async () => {
    const result = await mockAirBridgeService.verifyDestructiveOperationPolicy(
      req([makeOp("deleteTable", "drop-t")]),
    );
    expect(result.status).toBe("blocked");
  });

  it("deleteRecord returns blocked", async () => {
    const result = await mockAirBridgeService.verifyDestructiveOperationPolicy(
      req([makeOp("deleteRecord", "drop-r")]),
    );
    expect(result.status).toBe("blocked");
  });

  it("updateExistingRecord returns blocked", async () => {
    const result = await mockAirBridgeService.verifyDestructiveOperationPolicy(
      req([makeOp("updateExistingRecord", "upd-r")]),
    );
    expect(result.status).toBe("blocked");
  });

  it("overwriteField returns blocked", async () => {
    const result = await mockAirBridgeService.verifyDestructiveOperationPolicy(
      req([makeOp("overwriteField", "ovw-f")]),
    );
    expect(result.status).toBe("blocked");
  });

  it("overwriteTable returns blocked", async () => {
    const result = await mockAirBridgeService.verifyDestructiveOperationPolicy(
      req([makeOp("overwriteTable", "ovw-t")]),
    );
    expect(result.status).toBe("blocked");
  });

  it("attachmentUpload returns blocked", async () => {
    const result = await mockAirBridgeService.verifyDestructiveOperationPolicy(
      req([makeOp("attachmentUpload", "upload-a")]),
    );
    expect(result.status).toBe("blocked");
  });

  it("deleteBase returns blocked", async () => {
    const result = await mockAirBridgeService.verifyDestructiveOperationPolicy(
      req([makeOp("deleteBase", "drop-b")]),
    );
    expect(result.status).toBe("blocked");
  });

  it("five checks present for compliant result", async () => {
    const result = await mockAirBridgeService.verifyDestructiveOperationPolicy(
      req(CREATE_ONLY_OPS),
    );
    expect(result.checks).toHaveLength(5);
  });

  it("check IDs are DOP-01 through DOP-05", async () => {
    const result = await mockAirBridgeService.verifyDestructiveOperationPolicy(
      req(CREATE_ONLY_OPS),
    );
    const ids = result.checks.map((c) => c.checkId);
    expect(ids).toContain("DOP-01");
    expect(ids).toContain("DOP-02");
    expect(ids).toContain("DOP-03");
    expect(ids).toContain("DOP-04");
    expect(ids).toContain("DOP-05");
  });

  it("DOP-01 always passes", async () => {
    const result = await mockAirBridgeService.verifyDestructiveOperationPolicy(req([]));
    const dop01 = result.checks.find((c) => c.checkId === "DOP-01");
    expect(dop01?.status).toBe("passed");
  });

  it("DOP-05 passes for create-only ops", async () => {
    const result = await mockAirBridgeService.verifyDestructiveOperationPolicy(
      req(CREATE_ONLY_OPS),
    );
    const dop05 = result.checks.find((c) => c.checkId === "DOP-05");
    expect(dop05?.status).toBe("passed");
  });

  it("blockedOperations empty for compliant", async () => {
    const result = await mockAirBridgeService.verifyDestructiveOperationPolicy(
      req(CREATE_ONLY_OPS),
    );
    expect(result.blockedOperations).toHaveLength(0);
  });

  it("blockedOperations contains label for blocked", async () => {
    const result = await mockAirBridgeService.verifyDestructiveOperationPolicy(
      req([makeOp("deleteTable", "drop-my-table")]),
    );
    expect(result.blockedOperations).toContain("drop-my-table");
  });

  it("noChangesMade always true for all statuses", async () => {
    const cases = [req(CREATE_ONLY_OPS), req([makeOp("deleteTable", "d")]), req([])];
    for (const r of cases) {
      const result = await mockAirBridgeService.verifyDestructiveOperationPolicy(r);
      expect(result.noChangesMade).toBe(true);
    }
  });

  it("writesEnabled always false", async () => {
    const cases = [req(CREATE_ONLY_OPS), req([makeOp("deleteTable", "d")])];
    for (const r of cases) {
      const result = await mockAirBridgeService.verifyDestructiveOperationPolicy(r);
      expect(result.writesEnabled).toBe(false);
    }
  });

  it("networkWritesAttempted always false", async () => {
    const result = await mockAirBridgeService.verifyDestructiveOperationPolicy(
      req(CREATE_ONLY_OPS),
    );
    expect(result.networkWritesAttempted).toBe(false);
  });

  it("no token in message for any status", async () => {
    const cases = [req(CREATE_ONLY_OPS), req([makeOp("deleteTable", "d")])];
    for (const r of cases) {
      const result = await mockAirBridgeService.verifyDestructiveOperationPolicy(r);
      expect(result.message).not.toContain("token");
      expect(result.message).not.toContain("pat");
    }
  });

  it("no full path in message", async () => {
    const result = await mockAirBridgeService.verifyDestructiveOperationPolicy(
      req(CREATE_ONLY_OPS),
    );
    expect(result.message).not.toContain("/Users/");
    expect(result.message).not.toContain("/home/");
  });

  it("compliant message says writes remain disabled", async () => {
    const result = await mockAirBridgeService.verifyDestructiveOperationPolicy(
      req(CREATE_ONLY_OPS),
    );
    expect(result.message).toContain("disabled");
  });

  it("blocked message names blocked op label", async () => {
    const result = await mockAirBridgeService.verifyDestructiveOperationPolicy(
      req([makeOp("deleteTable", "drop-projects")]),
    );
    expect(result.message).toContain("drop-projects");
  });

  it("IPC fallback shape (null result) has safety invariants", () => {
    const fallback: DestructiveOperationPolicyResult = {
      status: "blocked",
      checks: [],
      message: "Destructive operation policy check is not available in this context.",
      blockedOperations: [],
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
  result: DestructiveOperationPolicyResult | null,
  loading = false,
  onVerify = vi.fn(),
) {
  return render(
    <RestoreDestructiveOperationPolicyPanel
      result={result}
      loading={loading}
      onVerify={onVerify}
    />,
  );
}

const COMPLIANT_RESULT: DestructiveOperationPolicyResult = {
  status: "compliant",
  checks: [
    {
      checkId: "DOP-01",
      label: "write-gate-disabled",
      status: "passed",
      message: "Gate disabled.",
    },
    { checkId: "DOP-02", label: "no-delete-operations", status: "passed", message: "No deletes." },
    {
      checkId: "DOP-03",
      label: "no-update-overwrite-operations",
      status: "passed",
      message: "No updates.",
    },
    { checkId: "DOP-04", label: "no-attachment-upload", status: "passed", message: "No uploads." },
    {
      checkId: "DOP-05",
      label: "create-only-operations",
      status: "passed",
      message: "All create-only.",
    },
  ],
  message: "All declared operations for My Base are create-only. Restore writes remain disabled.",
  blockedOperations: [],
  noChangesMade: true,
  networkWritesAttempted: false,
  writesEnabled: false,
};

const BLOCKED_RESULT: DestructiveOperationPolicyResult = {
  status: "blocked",
  checks: [
    {
      checkId: "DOP-01",
      label: "write-gate-disabled",
      status: "passed",
      message: "Gate disabled.",
    },
    {
      checkId: "DOP-02",
      label: "no-delete-operations",
      status: "failed",
      message: "Delete ops detected.",
      remediation: "Remove deletes.",
    },
    {
      checkId: "DOP-03",
      label: "no-update-overwrite-operations",
      status: "passed",
      message: "No updates.",
    },
    { checkId: "DOP-04", label: "no-attachment-upload", status: "passed", message: "No uploads." },
    {
      checkId: "DOP-05",
      label: "create-only-operations",
      status: "passed",
      message: "All create-only.",
    },
  ],
  message: "Blocked operations detected for My Base: drop-table.",
  blockedOperations: ["drop-table"],
  noChangesMade: true,
  networkWritesAttempted: false,
  writesEnabled: false,
};

describe("RestoreDestructiveOperationPolicyPanel rendering", () => {
  it("panel testid present", () => {
    renderPanel(null);
    expect(screen.getByTestId("restore-dop-panel")).toBeInTheDocument();
  });

  it("writes disabled notice always shown", () => {
    renderPanel(null);
    expect(screen.getByTestId("dop-writes-disabled-notice")).toBeInTheDocument();
  });

  it("verify button present", () => {
    renderPanel(null);
    expect(screen.getByTestId("dop-verify-button")).toBeInTheDocument();
  });

  it("button disabled when loading", () => {
    renderPanel(null, true);
    expect(screen.getByTestId("dop-verify-button")).toBeDisabled();
  });

  it("button shows re-verify label when result present", () => {
    renderPanel(COMPLIANT_RESULT);
    expect(screen.getByTestId("dop-verify-button")).toHaveTextContent("Re-verify");
  });

  it("onVerify callback fired on button click", () => {
    const onVerify = vi.fn();
    renderPanel(null, false, onVerify);
    fireEvent.click(screen.getByTestId("dop-verify-button"));
    expect(onVerify).toHaveBeenCalledOnce();
  });

  it("no result area before verify", () => {
    renderPanel(null);
    expect(screen.queryByTestId("dop-result")).not.toBeInTheDocument();
  });

  it("result area shown after verify", () => {
    renderPanel(COMPLIANT_RESULT);
    expect(screen.getByTestId("dop-result")).toBeInTheDocument();
  });

  it("compliant badge shown for compliant status", () => {
    renderPanel(COMPLIANT_RESULT);
    expect(screen.getByTestId("dop-compliant-badge")).toBeInTheDocument();
  });

  it("blocked badge shown for blocked status", () => {
    renderPanel(BLOCKED_RESULT);
    expect(screen.getByTestId("dop-blocked-badge")).toBeInTheDocument();
  });

  it("warning badge shown for warning status", () => {
    const warnResult: DestructiveOperationPolicyResult = {
      ...COMPLIANT_RESULT,
      status: "warning",
      message: "Some ops could not be classified.",
    };
    renderPanel(warnResult);
    expect(screen.getByTestId("dop-warning-badge")).toBeInTheDocument();
  });

  it("message text rendered", () => {
    renderPanel(COMPLIANT_RESULT);
    expect(screen.getByTestId("dop-message")).toHaveTextContent("create-only");
  });

  it("check rows rendered", () => {
    renderPanel(COMPLIANT_RESULT);
    const rows = screen.getAllByTestId("dop-check-row");
    expect(rows).toHaveLength(5);
  });

  it("safety summary rendered", () => {
    renderPanel(COMPLIANT_RESULT);
    expect(screen.getByTestId("dop-safety-summary")).toBeInTheDocument();
  });

  it("no-changes notice rendered", () => {
    renderPanel(COMPLIANT_RESULT);
    expect(screen.getByTestId("dop-no-changes-notice")).toBeInTheDocument();
  });

  it("compliant-notice shown only for compliant", () => {
    renderPanel(COMPLIANT_RESULT);
    expect(screen.getByTestId("dop-compliant-notice")).toBeInTheDocument();
    expect(screen.queryByTestId("dop-warning-notice")).not.toBeInTheDocument();
    expect(screen.queryByTestId("dop-blocked-notice")).not.toBeInTheDocument();
  });

  it("blocked-notice shown only for blocked", () => {
    renderPanel(BLOCKED_RESULT);
    expect(screen.getByTestId("dop-blocked-notice")).toBeInTheDocument();
    expect(screen.queryByTestId("dop-compliant-notice")).not.toBeInTheDocument();
    expect(screen.queryByTestId("dop-warning-notice")).not.toBeInTheDocument();
  });

  it("compliant notice says writes remain disabled", () => {
    renderPanel(COMPLIANT_RESULT);
    const notice = screen.getByTestId("dop-compliant-notice");
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

  it("blocked operations list shown when present", () => {
    renderPanel(BLOCKED_RESULT);
    expect(screen.getByTestId("dop-blocked-ops-list")).toBeInTheDocument();
    expect(screen.getByTestId("dop-blocked-op-item")).toHaveTextContent("drop-table");
  });

  it("blocked operations list not shown when empty", () => {
    renderPanel(COMPLIANT_RESULT);
    expect(screen.queryByTestId("dop-blocked-ops-list")).not.toBeInTheDocument();
  });
});
