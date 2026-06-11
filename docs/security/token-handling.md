# Token Handling

## Current Behavior (v0.1)

AirBridge accepts Airtable Personal Access Tokens for verifying connection permissions.
In the current version:

- Tokens are entered in a password input field and held only in local React component state.
- The token is passed to the connection check function and immediately discarded after the check completes.
- Tokens are **not** persisted to disk, stored in global application state, written to logs, included in reports, or returned in command results.
- The token input field uses `type="password"` to prevent the value from being read by browser extensions or screen readers in plain text mode.
- The `autoComplete="off"` attribute is set to discourage browser credential autofill from activating.
- Error messages are sanitized before display; any accidental token occurrence in an error string is replaced with `[redacted]`.
- After a connection check completes (whether successful or failed), the token is cleared from component state.

## Future Behavior

- Optional OS credential storage (macOS Keychain, Windows Credential Manager, Linux Secret Service) will be offered as an opt-in feature.
- A "session-only" mode will allow tokens to be held in process memory only for the duration of the application session.
- Permission scope checks will be run at connection creation and can be re-run on demand.

## Developer Rules

When contributing to AirBridge, follow these rules regarding token values:

1. **Never include tokens in error messages, logs, or reports.** Use `sanitizeConnectionError` from `connectionSecurity.ts` when handling errors that may have originated from token-bearing contexts.
2. **Never render tokens outside a `type="password"` input field.** Do not display tokens in status messages, success notices, or debug output.
3. **Never store tokens in global application state** (`AppState`, Zustand/Redux stores, session storage, local storage, or similar).
4. **Never include tokens in test fixtures, snapshots, or mock data.** Use synthetic placeholder values in tests that clearly do not resemble real credentials.
5. **Use `hasSecretLeak` in tests** to verify that rendered output and serialized results are free of token values.
6. **Drop tokens in Rust commands immediately.** The `check_connection` Tauri command must use `let _ = token;` or equivalent to discard the token before any further processing.

## Backup Execution Token Flow

When the user executes a backup from the Run Backup panel, a separate one-time token entry is used:

1. The user types a personal access token into a `type="password"` field inside `BackupExecutionPanel`.
2. The token is held only in `useState` within that component — never in global state.
3. On run, the token is included in the `RunBackupCommandRequest` object and forwarded to `service.runBackupJob()`.
4. The live service passes the request directly to the Tauri IPC (`run_backup_job` command).
5. On the Rust side, the token is moved into `AirtableToken::new(token)` and then into the HTTP client. The original string is consumed and not stored.
6. After the run completes or fails, `clearSensitiveState()` is called, setting the token state back to `""`.

The token does not appear in:
- `RunBackupCommandResponse`
- `BackupJobResult`
- Any backup job event
- Any rendered UI element outside the masked password input
- Any test fixture or snapshot

The existing connection-page token is not reused for backup execution. The user supplies a fresh token for each backup run.
