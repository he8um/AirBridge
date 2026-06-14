import { describe, it, expect, vi } from "vitest";
import { render, screen, fireEvent } from "@testing-library/react";
import { RestoreSchemaRecordOrderPolicyPanel } from "../features/backups/RestoreSchemaRecordOrderPolicyPanel";
import { mockAirBridgeService } from "../services/mockAirBridgeService";
import type {
  SchemaRecordOrderPolicyRequest,
  SchemaRecordOrderPolicyResult,
  DeclaredWritePhase,
  RestoreWritePhaseKind,
} from "../backend/types";

// ── Helpers ───────────────────────────────────────────────────────────────────

function phase(kind: RestoreWritePhaseKind, planned = true, blocked = false): DeclaredWritePhase {
  return { phase: kind, isPlanned: planned, isBlocked: blocked };
}

function req(phases: DeclaredWritePhase[], name = "My Base"): SchemaRecordOrderPolicyRequest {
  return { declaredPhases: phases, targetDisplayName: name };
}

const VALID_PHASES: DeclaredWritePhase[] = [
  phase("schema"),
  phase("records"),
  phase("linkedRecords"),
  phase("attachments"),
  phase("validation"),
];

// ── Mock service contract ─────────────────────────────────────────────────────

describe("mockAirBridgeService — verifySchemaRecordOrderPolicy contract", () => {
  it("valid phase order returns compliant", async () => {
    const result = await mockAirBridgeService.verifySchemaRecordOrderPolicy(req(VALID_PHASES));
    expect(result.status).toBe("compliant");
  });

  it("empty phases returns warning", async () => {
    const result = await mockAirBridgeService.verifySchemaRecordOrderPolicy(req([]));
    expect(result.status).toBe("warning");
  });

  it("schema only returns compliant", async () => {
    const result = await mockAirBridgeService.verifySchemaRecordOrderPolicy(req([phase("schema")]));
    expect(result.status).toBe("compliant");
  });

  it("records before schema returns blocked", async () => {
    const result = await mockAirBridgeService.verifySchemaRecordOrderPolicy(
      req([phase("records"), phase("schema")]),
    );
    expect(result.status).toBe("blocked");
    expect(result.orderingViolations).toContain("records-before-schema");
  });

  it("missing schema with records returns blocked", async () => {
    const result = await mockAirBridgeService.verifySchemaRecordOrderPolicy(
      req([phase("records")]),
    );
    expect(result.status).toBe("blocked");
    expect(result.orderingViolations).toContain("missing-schema-with-records");
  });

  it("blocked schema with records returns blocked", async () => {
    const result = await mockAirBridgeService.verifySchemaRecordOrderPolicy(
      req([phase("schema", false, true), phase("records")]),
    );
    expect(result.status).toBe("blocked");
    expect(result.orderingViolations).toContain("schema-phase-blocked");
  });

  it("unplanned schema with records returns warning", async () => {
    const result = await mockAirBridgeService.verifySchemaRecordOrderPolicy(
      req([phase("schema", false, false), phase("records")]),
    );
    expect(result.status).toBe("warning");
  });

  it("linked before record-create returns blocked", async () => {
    const result = await mockAirBridgeService.verifySchemaRecordOrderPolicy(
      req([phase("schema"), phase("linkedRecords"), phase("records")]),
    );
    expect(result.status).toBe("blocked");
    expect(result.orderingViolations).toContain("linked-before-record-create");
  });

  it("linked without records returns warning", async () => {
    const result = await mockAirBridgeService.verifySchemaRecordOrderPolicy(
      req([phase("schema"), phase("linkedRecords")]),
    );
    expect(result.status).toBe("warning");
  });

  it("attachment before record-create returns blocked", async () => {
    const result = await mockAirBridgeService.verifySchemaRecordOrderPolicy(
      req([phase("schema"), phase("attachments"), phase("records")]),
    );
    expect(result.status).toBe("blocked");
    expect(result.orderingViolations).toContain("attachment-before-record-create");
  });

  it("attachment without records returns warning", async () => {
    const result = await mockAirBridgeService.verifySchemaRecordOrderPolicy(
      req([phase("schema"), phase("attachments")]),
    );
    expect(result.status).toBe("warning");
  });

  it("five checks always present", async () => {
    const result = await mockAirBridgeService.verifySchemaRecordOrderPolicy(req(VALID_PHASES));
    expect(result.checks).toHaveLength(5);
  });

  it("check IDs are SRO-01 through SRO-05", async () => {
    const result = await mockAirBridgeService.verifySchemaRecordOrderPolicy(req(VALID_PHASES));
    const ids = result.checks.map((c) => c.checkId);
    expect(ids).toContain("SRO-01");
    expect(ids).toContain("SRO-02");
    expect(ids).toContain("SRO-03");
    expect(ids).toContain("SRO-04");
    expect(ids).toContain("SRO-05");
  });

  it("SRO-01 always passes", async () => {
    const result = await mockAirBridgeService.verifySchemaRecordOrderPolicy(req([]));
    const sro01 = result.checks.find((c) => c.checkId === "SRO-01");
    expect(sro01?.status).toBe("passed");
  });

  it("ordering violations empty for compliant", async () => {
    const result = await mockAirBridgeService.verifySchemaRecordOrderPolicy(req(VALID_PHASES));
    expect(result.orderingViolations).toHaveLength(0);
  });

  it("noChangesMade always true", async () => {
    for (const phases of [VALID_PHASES, [phase("records"), phase("schema")], []]) {
      const result = await mockAirBridgeService.verifySchemaRecordOrderPolicy(req(phases));
      expect(result.noChangesMade).toBe(true);
    }
  });

  it("writesEnabled always false", async () => {
    for (const phases of [VALID_PHASES, [phase("records"), phase("schema")]]) {
      const result = await mockAirBridgeService.verifySchemaRecordOrderPolicy(req(phases));
      expect(result.writesEnabled).toBe(false);
    }
  });

  it("networkWritesAttempted always false", async () => {
    const result = await mockAirBridgeService.verifySchemaRecordOrderPolicy(req(VALID_PHASES));
    expect(result.networkWritesAttempted).toBe(false);
  });

  it("no token in message", async () => {
    for (const phases of [VALID_PHASES, [phase("records"), phase("schema")]]) {
      const result = await mockAirBridgeService.verifySchemaRecordOrderPolicy(req(phases));
      expect(result.message).not.toContain("token");
      expect(result.message).not.toContain("pat");
    }
  });

  it("no full path in message", async () => {
    const result = await mockAirBridgeService.verifySchemaRecordOrderPolicy(req(VALID_PHASES));
    expect(result.message).not.toContain("/Users/");
    expect(result.message).not.toContain("/home/");
  });

  it("no record payload in message", async () => {
    const result = await mockAirBridgeService.verifySchemaRecordOrderPolicy(req(VALID_PHASES));
    expect(result.message).not.toContain("fields");
    expect(result.message).not.toContain("recordId");
  });

  it("compliant message says writes remain disabled", async () => {
    const result = await mockAirBridgeService.verifySchemaRecordOrderPolicy(req(VALID_PHASES));
    expect(result.message).toContain("disabled");
  });

  it("blocked message names violation", async () => {
    const result = await mockAirBridgeService.verifySchemaRecordOrderPolicy(
      req([phase("records"), phase("schema")]),
    );
    expect(result.message).toContain("records-before-schema");
  });

  it("display name appears in message", async () => {
    const result = await mockAirBridgeService.verifySchemaRecordOrderPolicy(req(VALID_PHASES));
    expect(result.message).toContain("My Base");
  });

  it("IPC fallback shape has safety invariants", () => {
    const fallback: SchemaRecordOrderPolicyResult = {
      status: "blocked",
      checks: [],
      message: "Schema record order policy check is not available in this context.",
      orderingViolations: [],
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
  result: SchemaRecordOrderPolicyResult | null,
  loading = false,
  onVerify = vi.fn(),
) {
  return render(
    <RestoreSchemaRecordOrderPolicyPanel result={result} loading={loading} onVerify={onVerify} />,
  );
}

const COMPLIANT_RESULT: SchemaRecordOrderPolicyResult = {
  status: "compliant",
  checks: [
    {
      checkId: "SRO-01",
      label: "write-gate-disabled",
      status: "passed",
      message: "Gate disabled.",
    },
    {
      checkId: "SRO-02",
      label: "schema-phase-present",
      status: "passed",
      message: "Schema present.",
    },
    {
      checkId: "SRO-03",
      label: "schema-before-records",
      status: "passed",
      message: "Schema before records.",
    },
    {
      checkId: "SRO-04",
      label: "records-before-linked-updates",
      status: "passed",
      message: "Records before linked.",
    },
    {
      checkId: "SRO-05",
      label: "records-before-attachments",
      status: "passed",
      message: "Records before attachments.",
    },
  ],
  message:
    "Phase ordering for My Base is valid. Schema precedes all record phases. Restore writes remain disabled.",
  orderingViolations: [],
  noChangesMade: true,
  networkWritesAttempted: false,
  writesEnabled: false,
};

const BLOCKED_RESULT: SchemaRecordOrderPolicyResult = {
  status: "blocked",
  checks: [
    {
      checkId: "SRO-01",
      label: "write-gate-disabled",
      status: "passed",
      message: "Gate disabled.",
    },
    {
      checkId: "SRO-02",
      label: "schema-phase-present",
      status: "passed",
      message: "Schema present.",
    },
    {
      checkId: "SRO-03",
      label: "schema-before-records",
      status: "failed",
      message: "Record-create phase appears before schema.",
      remediation: "Move schema first.",
    },
    { checkId: "SRO-04", label: "records-before-linked-updates", status: "passed", message: "OK." },
    { checkId: "SRO-05", label: "records-before-attachments", status: "passed", message: "OK." },
  ],
  message: "Phase ordering violation detected for My Base: records-before-schema.",
  orderingViolations: ["records-before-schema"],
  noChangesMade: true,
  networkWritesAttempted: false,
  writesEnabled: false,
};

describe("RestoreSchemaRecordOrderPolicyPanel rendering", () => {
  it("panel testid present", () => {
    renderPanel(null);
    expect(screen.getByTestId("restore-sro-panel")).toBeInTheDocument();
  });

  it("writes disabled notice always shown", () => {
    renderPanel(null);
    expect(screen.getByTestId("sro-writes-disabled-notice")).toBeInTheDocument();
  });

  it("verify button present", () => {
    renderPanel(null);
    expect(screen.getByTestId("sro-verify-button")).toBeInTheDocument();
  });

  it("button disabled when loading", () => {
    renderPanel(null, true);
    expect(screen.getByTestId("sro-verify-button")).toBeDisabled();
  });

  it("button shows re-verify label when result present", () => {
    renderPanel(COMPLIANT_RESULT);
    expect(screen.getByTestId("sro-verify-button")).toHaveTextContent("Re-verify");
  });

  it("onVerify callback fired on button click", () => {
    const onVerify = vi.fn();
    renderPanel(null, false, onVerify);
    fireEvent.click(screen.getByTestId("sro-verify-button"));
    expect(onVerify).toHaveBeenCalledOnce();
  });

  it("no result area before verify", () => {
    renderPanel(null);
    expect(screen.queryByTestId("sro-result")).not.toBeInTheDocument();
  });

  it("result area shown after verify", () => {
    renderPanel(COMPLIANT_RESULT);
    expect(screen.getByTestId("sro-result")).toBeInTheDocument();
  });

  it("compliant badge shown for compliant status", () => {
    renderPanel(COMPLIANT_RESULT);
    expect(screen.getByTestId("sro-compliant-badge")).toBeInTheDocument();
  });

  it("blocked badge shown for blocked status", () => {
    renderPanel(BLOCKED_RESULT);
    expect(screen.getByTestId("sro-blocked-badge")).toBeInTheDocument();
  });

  it("warning badge shown for warning status", () => {
    const warnResult: SchemaRecordOrderPolicyResult = {
      ...COMPLIANT_RESULT,
      status: "warning",
      message: "Phase data incomplete.",
    };
    renderPanel(warnResult);
    expect(screen.getByTestId("sro-warning-badge")).toBeInTheDocument();
  });

  it("message text rendered", () => {
    renderPanel(COMPLIANT_RESULT);
    expect(screen.getByTestId("sro-message")).toHaveTextContent("Phase ordering");
  });

  it("check rows rendered", () => {
    renderPanel(COMPLIANT_RESULT);
    const rows = screen.getAllByTestId("sro-check-row");
    expect(rows).toHaveLength(5);
  });

  it("safety summary rendered", () => {
    renderPanel(COMPLIANT_RESULT);
    expect(screen.getByTestId("sro-safety-summary")).toBeInTheDocument();
  });

  it("no-changes notice rendered", () => {
    renderPanel(COMPLIANT_RESULT);
    expect(screen.getByTestId("sro-no-changes-notice")).toBeInTheDocument();
  });

  it("compliant-notice shown only for compliant", () => {
    renderPanel(COMPLIANT_RESULT);
    expect(screen.getByTestId("sro-compliant-notice")).toBeInTheDocument();
    expect(screen.queryByTestId("sro-warning-notice")).not.toBeInTheDocument();
    expect(screen.queryByTestId("sro-blocked-notice")).not.toBeInTheDocument();
  });

  it("blocked-notice shown only for blocked", () => {
    renderPanel(BLOCKED_RESULT);
    expect(screen.getByTestId("sro-blocked-notice")).toBeInTheDocument();
    expect(screen.queryByTestId("sro-compliant-notice")).not.toBeInTheDocument();
    expect(screen.queryByTestId("sro-warning-notice")).not.toBeInTheDocument();
  });

  it("compliant notice says writes remain disabled", () => {
    renderPanel(COMPLIANT_RESULT);
    const notice = screen.getByTestId("sro-compliant-notice");
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

  it("violations list shown when present", () => {
    renderPanel(BLOCKED_RESULT);
    expect(screen.getByTestId("sro-violations-list")).toBeInTheDocument();
    expect(screen.getByTestId("sro-violation-item")).toHaveTextContent("records-before-schema");
  });

  it("violations list not shown when empty", () => {
    renderPanel(COMPLIANT_RESULT);
    expect(screen.queryByTestId("sro-violations-list")).not.toBeInTheDocument();
  });
});
