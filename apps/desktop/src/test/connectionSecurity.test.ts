import { describe, it, expect } from "vitest";
import {
  redactSecret,
  hasSecretLeak,
  sanitizeConnectionError,
} from "../features/connections/connectionSecurity";

describe("connectionSecurity", () => {
  describe("redactSecret", () => {
    it("returns [redacted] for a non-empty value", () => {
      expect(redactSecret("my-secret-token-value")).toBe("[redacted]");
    });

    it("returns empty string for empty input", () => {
      expect(redactSecret("")).toBe("");
    });

    it("never returns the original value for non-empty input", () => {
      const secret = "super-secret-value-12345";
      expect(redactSecret(secret)).not.toContain(secret);
    });
  });

  describe("hasSecretLeak", () => {
    it("returns true when text contains the secret", () => {
      expect(
        hasSecretLeak("error: token abc123def456ghi789jkl is invalid", "abc123def456ghi789jkl"),
      ).toBe(true);
    });

    it("returns false when text does not contain the secret", () => {
      expect(hasSecretLeak("connection failed", "abc123def456ghi789jkl")).toBe(false);
    });

    it("returns false for empty secret", () => {
      expect(hasSecretLeak("any text here", "")).toBe(false);
    });

    it("returns false for empty text", () => {
      expect(hasSecretLeak("", "secret")).toBe(false);
    });
  });

  describe("sanitizeConnectionError", () => {
    it("returns a generic message for unknown error type", () => {
      expect(sanitizeConnectionError(42)).toBe("Connection check failed. Please try again.");
    });

    it("extracts message from Error instances", () => {
      const err = new Error("rate limit exceeded");
      expect(sanitizeConnectionError(err)).toBe("rate limit exceeded");
    });

    it("uses string errors directly", () => {
      expect(sanitizeConnectionError("something went wrong")).toBe("something went wrong");
    });

    it("redacts secret from Error message", () => {
      const secret = "my-secret-token-value-xyz-123456";
      const err = new Error(`Token ${secret} was rejected`);
      const result = sanitizeConnectionError(err, secret);
      expect(result).not.toContain(secret);
      expect(result).toContain("[redacted]");
    });

    it("redacts secret from string error", () => {
      const secret = "my-secret-token-value-xyz-123456";
      const result = sanitizeConnectionError(`invalid token: ${secret}`, secret);
      expect(result).not.toContain(secret);
    });

    it("does not modify message when secret is not present", () => {
      const result = sanitizeConnectionError("rate limit exceeded", "some-other-secret-12345");
      expect(result).toBe("rate limit exceeded");
    });
  });
});
