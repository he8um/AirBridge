import { BACKUP_CONFIRMATION_TEXT } from "./backupExecutionHelpers";

interface BackupConfirmationBoxProps {
  value: string;
  onChange: (value: string) => void;
  disabled?: boolean;
}

export function BackupConfirmationBox({ value, onChange, disabled }: BackupConfirmationBoxProps) {
  const isConfirmed = value === BACKUP_CONFIRMATION_TEXT;

  return (
    <div
      className="form-field"
      style={{ display: "flex", flexDirection: "column", gap: "var(--space-2)" }}
    >
      <label htmlFor="backup-confirmation-input" className="form-label">
        Confirmation
      </label>
      <p
        style={{ fontSize: "var(--text-xs)", color: "var(--color-text-muted)", margin: 0 }}
        id="backup-confirmation-hint"
      >
        Type{" "}
        <span
          style={{ fontFamily: "var(--font-mono)", fontWeight: 600 }}
          aria-label="required confirmation text"
        >
          {BACKUP_CONFIRMATION_TEXT}
        </span>{" "}
        to confirm package creation.
      </p>
      <input
        id="backup-confirmation-input"
        type="text"
        className="form-input"
        value={value}
        onChange={(e) => onChange(e.target.value)}
        disabled={disabled}
        aria-describedby="backup-confirmation-hint"
        aria-label="Backup confirmation text"
        autoComplete="off"
        spellCheck={false}
        data-testid="backup-confirmation-input"
      />
      {value.length > 0 && !isConfirmed && (
        <p
          style={{
            fontSize: "var(--text-xs)",
            color: "var(--color-error, #c0392b)",
            margin: 0,
          }}
          role="alert"
          aria-live="polite"
        >
          Confirmation text does not match.
        </p>
      )}
    </div>
  );
}
