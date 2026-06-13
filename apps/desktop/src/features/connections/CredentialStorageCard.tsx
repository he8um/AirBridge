import { useState, useEffect } from "react";
import type { AirBridgeService } from "../../services/airBridgeService";
import type { CredentialStorageStatus } from "../../backend/types";

interface CredentialStorageCardProps {
  service: AirBridgeService;
}

/**
 * Lets the user optionally save their Airtable token to the OS keychain.
 *
 * Constraints:
 * - Token input is always type="password" — never rendered as plaintext.
 * - After a successful save, the token input is cleared.
 * - The saved token value is never rendered.
 * - No localStorage or sessionStorage is used.
 * - No Airtable write behavior.
 * - No restore write enablement.
 */
export function CredentialStorageCard({ service }: CredentialStorageCardProps) {
  const [storageStatus, setStorageStatus] = useState<CredentialStorageStatus | null>(null);
  const [tokenInput, setTokenInput] = useState("");
  const [isSaving, setIsSaving] = useState(false);
  const [isRemoving, setIsRemoving] = useState(false);
  const [feedbackMessage, setFeedbackMessage] = useState<string | null>(null);
  const [feedbackKind, setFeedbackKind] = useState<"success" | "error" | null>(null);

  // Load initial status on mount — never fetches or reveals the token value
  useEffect(() => {
    service
      .getCredentialStorageStatus({ kind: "airtablePersonalAccessToken" })
      .then((result) => {
        setStorageStatus(result.status);
      })
      .catch(() => {
        setStorageStatus("failed");
      });
  }, [service]);

  async function handleSave() {
    if (!tokenInput.trim()) return;
    setIsSaving(true);
    setFeedbackMessage(null);
    setFeedbackKind(null);
    try {
      const result = await service.saveAirtableTokenToKeychain({
        kind: "airtablePersonalAccessToken",
        token: tokenInput,
      });
      // Clear the token input immediately after forwarding — never keep it in state
      setTokenInput("");
      if (result.success) {
        setStorageStatus("saved");
        setFeedbackMessage("Token saved to OS keychain.");
        setFeedbackKind("success");
      } else {
        setStorageStatus(result.hasSavedToken ? "saved" : "notSaved");
        setFeedbackMessage(result.errorMessage ?? "Failed to save token.");
        setFeedbackKind("error");
      }
    } catch {
      setTokenInput("");
      setFeedbackMessage("An unexpected error occurred. Token was not saved.");
      setFeedbackKind("error");
    } finally {
      setIsSaving(false);
    }
  }

  async function handleRemove() {
    setIsRemoving(true);
    setFeedbackMessage(null);
    setFeedbackKind(null);
    try {
      const result = await service.removeAirtableTokenFromKeychain({
        kind: "airtablePersonalAccessToken",
      });
      if (result.success) {
        setStorageStatus("notSaved");
        setFeedbackMessage("Saved token removed from OS keychain.");
        setFeedbackKind("success");
      } else {
        setFeedbackMessage(result.errorMessage ?? "Failed to remove token.");
        setFeedbackKind("error");
      }
    } catch {
      setFeedbackMessage("An unexpected error occurred.");
      setFeedbackKind("error");
    } finally {
      setIsRemoving(false);
    }
  }

  const isUnavailable = storageStatus === "unavailable" || storageStatus === "failed";
  const hasSavedToken = storageStatus === "saved";

  return (
    <div
      data-testid="credential-storage-card"
      style={{ display: "flex", flexDirection: "column", gap: "var(--space-4)" }}
    >
      {/* Opt-in explanation */}
      <p style={{ fontSize: "var(--text-sm)", color: "var(--color-text-muted)", margin: 0 }}>
        Optionally save your Airtable Personal Access Token to the OS keychain so you do not need to
        enter it each session. Saving is not required. The token is stored only in the OS keychain —
        never in files, history, or logs.
      </p>

      {/* Unavailable notice */}
      {isUnavailable && (
        <div
          className="notice notice-warning"
          role="note"
          data-testid="credential-unavailable-notice"
          style={{ fontSize: "var(--text-xs)" }}
        >
          <span>OS keychain is not available on this system. Token saving is disabled.</span>
        </div>
      )}

      {/* Saved status badge */}
      {!isUnavailable && (
        <div
          style={{ display: "flex", alignItems: "center", gap: "var(--space-2)" }}
          data-testid="credential-status-row"
        >
          <span style={{ fontSize: "var(--text-xs)", color: "var(--color-text-muted)" }}>
            Status:
          </span>
          <span
            data-testid="credential-status-badge"
            data-status={storageStatus ?? "loading"}
            style={{
              fontSize: "var(--text-xs)",
              fontWeight: 600,
              color: hasSavedToken ? "var(--color-success)" : "var(--color-text-muted)",
            }}
          >
            {storageStatus === null
              ? "Checking…"
              : hasSavedToken
                ? "Saved token present"
                : "No saved token"}
          </span>
        </div>
      )}

      {/* Token input — disabled when unavailable or already saved */}
      {!isUnavailable && !hasSavedToken && (
        <div className="form-field">
          <label htmlFor="credential-token-input" className="form-label">
            Personal Access Token
          </label>
          <input
            id="credential-token-input"
            type="password"
            className="form-input"
            placeholder="Enter your Airtable token"
            value={tokenInput}
            onChange={(e) => setTokenInput(e.target.value)}
            disabled={isSaving || isUnavailable}
            autoComplete="new-password"
            data-testid="credential-token-input"
            aria-label="Airtable Personal Access Token"
          />
          <p style={{ fontSize: "var(--text-xs)", color: "var(--color-text-muted)" }}>
            The token is forwarded directly to the OS keychain. It is never stored in files or logs.
          </p>
        </div>
      )}

      {/* Save button */}
      {!isUnavailable && !hasSavedToken && (
        <button
          type="button"
          className="btn btn-primary"
          onClick={handleSave}
          disabled={isSaving || !tokenInput.trim() || isUnavailable}
          data-testid="credential-save-button"
          style={{ alignSelf: "flex-start" }}
        >
          {isSaving ? "Saving…" : "Save to Keychain"}
        </button>
      )}

      {/* Remove button — only shown when a token is saved */}
      {!isUnavailable && hasSavedToken && (
        <button
          type="button"
          className="btn btn-secondary"
          onClick={handleRemove}
          disabled={isRemoving}
          data-testid="credential-remove-button"
          style={{ alignSelf: "flex-start" }}
        >
          {isRemoving ? "Removing…" : "Remove Saved Token"}
        </button>
      )}

      {/* Feedback message — never contains the token value */}
      {feedbackMessage !== null && (
        <div
          className={`notice notice-${feedbackKind === "success" ? "info" : "warning"}`}
          role="status"
          data-testid="credential-feedback"
          data-feedback-kind={feedbackKind}
          style={{ fontSize: "var(--text-xs)" }}
        >
          <span>{feedbackMessage}</span>
        </div>
      )}
    </div>
  );
}
