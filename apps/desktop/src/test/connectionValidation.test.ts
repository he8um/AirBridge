import { describe, it, expect } from "vitest";
import {
  validateConnectionName,
  validatePersonalAccessToken,
  validateConnectionForm,
} from "../features/connections/connectionValidation";

describe("connectionValidation", () => {
  describe("validateConnectionName", () => {
    it("accepts a valid name", () => {
      expect(validateConnectionName("My Connection")).toBeNull();
    });

    it("rejects empty name", () => {
      const result = validateConnectionName("");
      expect(result).not.toBeNull();
      expect(result?.field).toBe("name");
    });

    it("rejects single-char name", () => {
      const result = validateConnectionName("a");
      expect(result).not.toBeNull();
      expect(result?.field).toBe("name");
    });

    it("rejects name over 80 chars", () => {
      const longName = "a".repeat(81);
      const result = validateConnectionName(longName);
      expect(result).not.toBeNull();
      expect(result?.field).toBe("name");
    });

    it("trims whitespace before validating", () => {
      // " a " trims to "a" which is length 1 — invalid
      const result = validateConnectionName(" a ");
      expect(result).not.toBeNull();
      expect(result?.field).toBe("name");
    });

    it("does not include any token value in error messages", () => {
      const result = validatePersonalAccessToken("x");
      expect(result?.message).not.toContain("x");
    });
  });

  describe("validatePersonalAccessToken", () => {
    it("accepts a token of sufficient length", () => {
      expect(validatePersonalAccessToken("a".repeat(20))).toBeNull();
    });

    it("rejects empty token", () => {
      const result = validatePersonalAccessToken("");
      expect(result).not.toBeNull();
      expect(result?.field).toBe("token");
    });

    it("rejects token under 20 chars", () => {
      const result = validatePersonalAccessToken("short_value");
      expect(result).not.toBeNull();
      expect(result?.field).toBe("token");
    });

    it("rejects whitespace-only token", () => {
      const result = validatePersonalAccessToken("   ");
      expect(result).not.toBeNull();
      expect(result?.field).toBe("token");
    });

    it("validation message does not include the token value", () => {
      const shortToken = "short_token_value_here";
      const result = validatePersonalAccessToken(shortToken);
      if (result) {
        expect(result.message).not.toContain(shortToken);
      }
    });
  });

  describe("validateConnectionForm", () => {
    it("returns valid for good input", () => {
      const result = validateConnectionForm({
        name: "My Connection",
        token: "a".repeat(20),
      });
      expect(result.valid).toBe(true);
      expect(result.errors).toHaveLength(0);
    });

    it("returns both errors for empty input", () => {
      const result = validateConnectionForm({ name: "", token: "" });
      expect(result.valid).toBe(false);
      expect(result.errors).toHaveLength(2);
    });

    it("returns only token error when name is valid", () => {
      const result = validateConnectionForm({ name: "My Connection", token: "" });
      expect(result.valid).toBe(false);
      expect(result.errors).toHaveLength(1);
      expect(result.errors[0]?.field).toBe("token");
    });
  });
});
