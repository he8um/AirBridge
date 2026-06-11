import { render, screen, waitFor } from "@testing-library/react";
import userEvent from "@testing-library/user-event";
import { describe, it, expect, vi, beforeEach } from "vitest";
import { PackageInspectionPanel } from "../features/backups/PackageInspectionPanel";
import { mockAirBridgeService } from "../services/mockAirBridgeService";
import type { AirBridgeService } from "../services/airBridgeService";
import type { BackupPackageInspectionResult } from "../backend/types";

// ── Mock the file picker ───────────────────────────────────────────────────

vi.mock("../features/backups/PackageInspectionPicker", () => ({
  pickBackupPackagePath: vi.fn().mockResolvedValue(null),
}));

import { pickBackupPackagePath } from "../features/backups/PackageInspectionPicker";
const mockPicker = vi.mocked(pickBackupPackagePath);

// ── Helpers ────────────────────────────────────────────────────────────────

function makeValidResult(filename = "backup.airbridge"): BackupPackageInspectionResult {
  return {
    filename,
    validationStatus: "valid",
    manifest: {
      format: "airbridge",
      formatVersion: "0.1.0",
      appVersion: "0.1.0",
      createdAt: "2026-06-11T00:00:00Z",
      provider: "airtable",
      baseId: "appTest01",
      baseName: "Test Base",
    },
    contents: {
      tableCount: 2,
      fieldCount: 9,
      recordCount: 47,
      linkedRecordRelationshipCount: 1,
      attachmentCount: 0,
    },
    security: {
      encrypted: false,
      containsRecordData: true,
      containsAttachmentUrls: false,
      redactionsApplied: [],
    },
    checksums: {
      checksumCount: 5,
      allValid: true,
    },
    entryCount: 8,
    warnings: [],
    errors: [],
  };
}

function makeInvalidResult(filename = "corrupt.airbridge"): BackupPackageInspectionResult {
  return {
    filename,
    validationStatus: "invalid",
    entryCount: 0,
    warnings: [],
    errors: [{ code: "CANNOT_OPEN", message: "package could not be opened" }],
  };
}

async function renderAndSelect(service: AirBridgeService, path: string) {
  render(<PackageInspectionPanel service={service} />);
  mockPicker.mockResolvedValueOnce(path);
  const btn = screen.getByTestId("select-package-button");
  await userEvent.click(btn);
}

// ── Type model tests ───────────────────────────────────────────────────────

describe("BackupPackageInspectionResult type model", () => {
  it("valid result has required fields", () => {
    const result = makeValidResult();
    expect(result.filename).toBe("backup.airbridge");
    expect(result.validationStatus).toBe("valid");
    expect(result.entryCount).toBe(8);
    expect(result.errors).toHaveLength(0);
  });

  it("invalid result has errors and no manifest", () => {
    const result = makeInvalidResult();
    expect(result.validationStatus).toBe("invalid");
    expect(result.errors[0].code).toBe("CANNOT_OPEN");
    expect(result.manifest).toBeUndefined();
  });

  it("manifest summary has required fields", () => {
    const result = makeValidResult();
    expect(result.manifest?.provider).toBe("airtable");
    expect(result.manifest?.baseId).toBe("appTest01");
    expect(result.manifest?.format).toBe("airbridge");
  });

  it("contents summary has all count fields", () => {
    const result = makeValidResult();
    expect(result.contents?.tableCount).toBe(2);
    expect(result.contents?.fieldCount).toBe(9);
    expect(result.contents?.recordCount).toBe(47);
    expect(result.contents?.linkedRecordRelationshipCount).toBe(1);
    expect(result.contents?.attachmentCount).toBe(0);
  });

  it("security summary has flag fields", () => {
    const result = makeValidResult();
    expect(result.security?.encrypted).toBe(false);
    expect(result.security?.containsRecordData).toBe(true);
    expect(result.security?.containsAttachmentUrls).toBe(false);
    expect(result.security?.redactionsApplied).toHaveLength(0);
  });

  it("checksum summary has count and validity", () => {
    const result = makeValidResult();
    expect(result.checksums?.checksumCount).toBe(5);
    expect(result.checksums?.allValid).toBe(true);
  });
});

// ── Mock service tests ─────────────────────────────────────────────────────

describe("mockAirBridgeService inspection", () => {
  it("returns valid result for normal path", async () => {
    const result = await mockAirBridgeService.inspectBackupPackage("/tmp/backup.airbridge");
    expect(result.validationStatus).toBe("valid");
    expect(result.filename).toBe("backup.airbridge");
  });

  it("returns invalid result when path contains 'invalid'", async () => {
    const result = await mockAirBridgeService.inspectBackupPackage(
      "/tmp/invalid_package.airbridge",
    );
    expect(result.validationStatus).toBe("invalid");
    expect(result.errors.length).toBeGreaterThan(0);
  });

  it("returns invalid result when path contains 'corrupt'", async () => {
    const result = await mockAirBridgeService.inspectBackupPackage("/tmp/corrupt.airbridge");
    expect(result.validationStatus).toBe("invalid");
  });

  it("filename does not contain directory separators", async () => {
    const result = await mockAirBridgeService.inspectBackupPackage(
      "/tmp/some/path/backup.airbridge",
    );
    expect(result.filename).toBe("backup.airbridge");
    expect(result.filename).not.toContain("/");
    expect(result.filename).not.toContain("\\");
  });

  it("valid result does not include absolute path in any field", async () => {
    const result = await mockAirBridgeService.inspectBackupPackage("/tmp/backup.airbridge");
    const json = JSON.stringify(result);
    expect(json).not.toContain("/tmp/");
    expect(json).not.toContain("/Users/");
  });
});

// ── Panel idle state ───────────────────────────────────────────────────────

describe("PackageInspectionPanel idle state", () => {
  beforeEach(() => {
    mockPicker.mockResolvedValue(null);
  });

  it("shows idle notice on mount", () => {
    render(<PackageInspectionPanel service={mockAirBridgeService} />);
    expect(screen.getByTestId("inspection-idle-notice")).not.toBeNull();
  });

  it("shows select button on mount", () => {
    render(<PackageInspectionPanel service={mockAirBridgeService} />);
    expect(screen.getByTestId("select-package-button")).not.toBeNull();
  });

  it("does not show result panel on mount", () => {
    render(<PackageInspectionPanel service={mockAirBridgeService} />);
    expect(screen.queryByTestId("inspection-result-panel")).toBeNull();
  });

  it("picker not called on mount", () => {
    render(<PackageInspectionPanel service={mockAirBridgeService} />);
    expect(mockPicker).not.toHaveBeenCalled();
  });

  it("stays idle when picker returns null (user cancelled)", async () => {
    render(<PackageInspectionPanel service={mockAirBridgeService} />);
    mockPicker.mockResolvedValueOnce(null);
    const btn = screen.getByTestId("select-package-button");
    await userEvent.click(btn);
    expect(screen.queryByTestId("inspection-result-panel")).toBeNull();
    expect(screen.getByTestId("inspection-idle-notice")).not.toBeNull();
  });
});

// ── Panel result state ─────────────────────────────────────────────────────

describe("PackageInspectionPanel result state", () => {
  beforeEach(() => {
    mockPicker.mockResolvedValue(null);
  });

  it("shows result panel after successful inspection", async () => {
    await renderAndSelect(mockAirBridgeService, "/tmp/backup.airbridge");
    await waitFor(() => screen.getByTestId("inspection-result-panel"));
    expect(screen.getByTestId("inspection-result-panel")).not.toBeNull();
  });

  it("shows filename from result (not full path)", async () => {
    await renderAndSelect(mockAirBridgeService, "/tmp/backup.airbridge");
    await waitFor(() => screen.getByTestId("inspection-filename"));
    const filenameEl = screen.getByTestId("inspection-filename");
    expect(filenameEl.textContent).toBe("backup.airbridge");
    expect(filenameEl.textContent).not.toContain("/tmp/");
  });

  it("shows validation status badge", async () => {
    await renderAndSelect(mockAirBridgeService, "/tmp/backup.airbridge");
    await waitFor(() => screen.getByTestId("inspection-validation-status"));
    const badge = screen.getByTestId("inspection-validation-status");
    expect(badge.getAttribute("data-validation-status")).toBe("valid");
  });

  it("shows read-only notice with required copy", async () => {
    await renderAndSelect(mockAirBridgeService, "/tmp/backup.airbridge");
    await waitFor(() => screen.getByTestId("inspection-readonly-notice"));
    const notice = screen.getByTestId("inspection-readonly-notice");
    expect(notice.textContent).toContain("Inspection is read-only");
    expect(notice.textContent).toContain("No files are extracted");
    expect(notice.textContent).toContain("Restore is not started");
  });

  it("shows manifest summary section", async () => {
    await renderAndSelect(mockAirBridgeService, "/tmp/backup.airbridge");
    await waitFor(() => screen.getByTestId("inspection-manifest-summary"));
    expect(screen.getByTestId("inspection-manifest-summary")).not.toBeNull();
  });

  it("shows contents summary section", async () => {
    await renderAndSelect(mockAirBridgeService, "/tmp/backup.airbridge");
    await waitFor(() => screen.getByTestId("inspection-contents-summary"));
    expect(screen.getByTestId("inspection-contents-summary")).not.toBeNull();
  });

  it("shows security summary section", async () => {
    await renderAndSelect(mockAirBridgeService, "/tmp/backup.airbridge");
    await waitFor(() => screen.getByTestId("inspection-security-summary"));
    expect(screen.getByTestId("inspection-security-summary")).not.toBeNull();
  });

  it("shows errors for invalid package", async () => {
    await renderAndSelect(mockAirBridgeService, "/tmp/invalid_pkg.airbridge");
    await waitFor(() => screen.getByTestId("inspection-errors"));
    expect(screen.getByTestId("inspection-errors")).not.toBeNull();
  });

  it("does not expose absolute path in rendered output", async () => {
    const { container } = render(<PackageInspectionPanel service={mockAirBridgeService} />);
    mockPicker.mockResolvedValueOnce("/Users/amirhesampiri/backups/my-backup.airbridge");
    const btn = screen.getByTestId("select-package-button");
    await userEvent.click(btn);
    await waitFor(() => screen.getByTestId("inspection-result-panel"));
    expect(container.textContent).not.toContain("/Users/amirhesampiri/");
    expect(container.textContent).not.toContain("/backups/");
  });

  it("shows checksum mismatch warning when checksums invalid", async () => {
    const serviceWithBadChecksums: AirBridgeService = {
      ...mockAirBridgeService,
      inspectBackupPackage: async () => ({
        ...makeValidResult(),
        checksums: { checksumCount: 3, allValid: false },
        validationStatus: "invalid",
        errors: [
          { code: "CHECKSUM_MISMATCH", message: "checksum mismatch for entry 'manifest.json'" },
        ],
      }),
    };
    await renderAndSelect(serviceWithBadChecksums, "/tmp/tampered.airbridge");
    await waitFor(() => screen.getByTestId("inspection-checksum-warning"));
    expect(screen.getByTestId("inspection-checksum-warning")).not.toBeNull();
  });
});
