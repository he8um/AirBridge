import { useState } from "react";
import type { ConnectionCheckResult } from "../../backend/types";
import type { AirBridgeService } from "../../services/airBridgeService";
import { validateConnectionForm } from "./connectionValidation";
import { sanitizeConnectionError, hasSecretLeak } from "./connectionSecurity";
import { PermissionCheckList } from "./PermissionCheckList";
import { StatusBadge } from "../../components/StatusBadge";
import { liveAirBridgeService } from "../../services/liveAirBridgeService";

interface ConnectionFormProps {
  onSuccess?: (result: ConnectionCheckResult) => void;
  onError?: (message: string) => void;
  /** Service used for the connection check. Defaults to the live service.
   *  Inject a mock service in tests or jsdom environments. */
  service?: Pick<AirBridgeService, "checkConnection">;
}

export function ConnectionForm({ onSuccess, onError, service }: ConnectionFormProps) {
  const [name, setName] = useState("");
  const [token, setToken] = useState("");
  const [isChecking, setIsChecking] = useState(false);
  const [result, setResult] = useState<ConnectionCheckResult | null>(null);
  const [errorMessage, setErrorMessage] = useState<string | null>(null);

  const canSubmit = validateConnectionForm({ name, token }).valid;

  const checkService = service ?? liveAirBridgeService;

  async function handleSubmit(event: React.FormEvent<HTMLFormElement>) {
    event.preventDefault();

    const validation = validateConnectionForm({ name, token });
    if (!validation.valid) {
      const firstError = validation.errors[0];
      setErrorMessage(firstError?.message ?? "Please correct the form errors.");
      return;
    }

    setIsChecking(true);
    setResult(null);
    setErrorMessage(null);

    // Capture token for sanitization before it is cleared
    const capturedToken = token;

    try {
      const response = await checkService.checkConnection({ token: capturedToken });

      if (hasSecretLeak(JSON.stringify(response), capturedToken)) {
        setErrorMessage("Connection check returned unexpected data.");
        setResult(null);
        onError?.("Connection check returned unexpected data.");
        return;
      }

      setResult(response);
      onSuccess?.(response);
    } catch (err) {
      const message = sanitizeConnectionError(err, capturedToken);
      setErrorMessage(message);
      onError?.(message);
    } finally {
      setIsChecking(false);
      setToken("");
    }
  }

  function handleClear() {
    setName("");
    setToken("");
    setResult(null);
    setErrorMessage(null);
  }

  return (
    <form onSubmit={handleSubmit} aria-label="Connection setup form" noValidate>
      <div className="form-field">
        <label htmlFor="conn-name-input" className="form-label">
          Connection Name
        </label>
        <input
          id="conn-name-input"
          type="text"
          value={name}
          onChange={(e) => setName(e.target.value)}
          aria-label="Connection name"
          data-testid="name-input"
          className="form-input"
        />
      </div>

      <div className="form-field">
        <label htmlFor="conn-token-input" className="form-label">
          Personal Access Token
        </label>
        <input
          id="conn-token-input"
          type="password"
          value={token}
          onChange={(e) => setToken(e.target.value)}
          autoComplete="off"
          aria-label="Personal access token"
          data-testid="token-input"
          className="form-input"
        />
        <p className="form-hint">Token is used for verification only and is not stored.</p>
      </div>

      <div className="form-field">
        <label>
          <input type="checkbox" disabled aria-label="Remember connection" /> Remember connection
          (coming soon)
        </label>
      </div>

      {errorMessage && (
        <div role="alert" aria-live="assertive" className="form-error">
          {errorMessage}
        </div>
      )}

      {result && (
        <div className="form-result">
          <StatusBadge
            status={result.status === "connected" ? "connected" : "error"}
            label={result.status === "connected" ? "Connected" : "Failed"}
          />
        </div>
      )}

      {result?.permissions && result.permissions.length > 0 && (
        <PermissionCheckList checks={result.permissions} title="Permission Check Results" />
      )}

      <div style={{ display: "flex", gap: "var(--space-3)" }}>
        <button
          type="submit"
          className="btn btn-primary"
          disabled={isChecking || !canSubmit}
          aria-label="Test connection"
        >
          {isChecking ? "Checking…" : "Test Connection"}
        </button>
        <button
          type="button"
          className="btn btn-secondary"
          onClick={handleClear}
          aria-label="Clear connection form"
        >
          Clear
        </button>
      </div>
    </form>
  );
}
