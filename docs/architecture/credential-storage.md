# Credential Storage

## Overview

AirBridge provides optional OS keychain credential storage for the Airtable Personal Access
Token. Saving a token is never required — every operation that needs a token accepts one
directly from the user at the time of use.

When the user opts in, the token is forwarded to the OS keychain via the `keyring` crate. The
token is never stored in files, SQLite, `localStorage`, `sessionStorage`, the backup package,
history items, logs, or command results.

## Security Properties

- **Token never returned by commands.** The three credential commands (`get_credential_storage_status`,
  `save_airtable_token_to_keychain`, `remove_airtable_token_from_keychain`) never include the
  token in any result struct or error message.
- **No plaintext disk storage.** The token goes to the OS keychain API only. No file on disk
  contains the token.
- **Token never in history.** Job history summaries contain counts, filenames, and statuses —
  never token values.
- **Token never in logs.** No Rust or frontend code logs the token. Errors map to safe
  message strings before being returned.
- **Redaction helpers.** `credentials/redaction.rs` provides `is_token_like()`,
  `redact_token()`, and `ensure_no_token_in_message()` for sanitizing any message that could
  inadvertently contain a secret.
- **Input cleared after save.** The UI token input is cleared immediately after the token is
  forwarded to the Rust command. It is not retained in React state.
- **Explicit opt-in.** The UI explains that saving is optional before offering the input.
- **Restore write execution remains disabled.** Credential storage has no effect on the
  restore write gate — `evaluate_write_gate()` always returns `Disabled/DisabledByProductPolicy`.

## OS Keychain Backend

The `keyring` crate (v3) is used for OS keychain access:

- **macOS** — system Keychain (Keychain Access app)
- **Windows** — Windows Credential Store
- **Linux** — Secret Service protocol (e.g., GNOME Keyring, KWallet)

The service name is `"AirBridge"`. The account name (non-secret) is
`"airtable_personal_access_token"`.

If the keychain backend is not available (e.g., headless Linux without a secret service daemon),
`availability()` returns `Unavailable` and all three commands return safe unavailable results.
No error propagates to the user that would contain a secret value.

## Rust Modules

| Module | Path | Role |
|---|---|---|
| `credentials::errors` | `src/credentials/errors.rs` | `CredentialStorageError` — all variants have safe `safe_message()` |
| `credentials::models` | `src/credentials/models.rs` | Request/result types; `CredentialSaveRequest` is not `Serialize` |
| `credentials::redaction` | `src/credentials/redaction.rs` | `is_token_like`, `redact_token`, `ensure_no_token_in_message` |
| `credentials::store` | `src/credentials/store.rs` | `CredentialStore` trait; `InMemoryCredentialStore` (test-only) |
| `credentials::keychain` | `src/credentials/keychain.rs` | `OsKeychainStore` wrapping `keyring` crate |
| `commands::credentials` | `src/commands/credentials.rs` | Three `#[tauri::command]` functions |

## Tauri Commands

### `get_credential_storage_status`

Request: `{ kind: "airtablePersonalAccessToken" }`  
Result: `{ kind, status, availability, hasSavedToken, display }`  
- No token in result.
- `hasSavedToken` is a boolean presence indicator only.
- `display` is a safe human-readable string.

### `save_airtable_token_to_keychain`

Request: `{ kind, token }`  
Result: `{ kind, success, hasSavedToken, display, errorMessage }`  
- Token is accepted in the request and forwarded to the keychain. It is not returned.
- If the token is empty, returns `success: false` without making a keychain call.
- If the keychain is unavailable, returns `success: false` with a safe message.
- `CredentialSaveRequest` does not derive `Serialize` — the token cannot be serialized back.

### `remove_airtable_token_from_keychain`

Request: `{ kind }`  
Result: `{ kind, success, hasSavedToken, display, errorMessage }`  
- If no entry exists, returns `success: true` (idempotent).
- Never returns the token.

## TypeScript Layer

Types are in `src/backend/types.ts`. Commands are in `src/backend/commands.ts`.

Service interface (`AirBridgeService`) has three methods:
- `getCredentialStorageStatus(request)` — status check, no token in result
- `saveAirtableTokenToKeychain(request)` — accepts token, never returns it
- `removeAirtableTokenFromKeychain(request)` — no token in result

The live service converts `null` IPC results to safe unavailable fallbacks. The mock service
uses an in-memory `Map<CredentialKind, boolean>` that stores only presence — not the token value.

## UI

`CredentialStorageCard` (`src/features/connections/CredentialStorageCard.tsx`) is rendered in
the Settings page under "Saved Credentials".

Behavior:
- Explains that saving is optional before offering the input.
- Token input is always `type="password"`.
- After a successful save, the token input is cleared and hidden.
- The saved token value is never rendered in any state.
- No `localStorage` or `sessionStorage` is used.
- When the keychain is unavailable, an explanatory notice is shown and the input/save button
  are hidden.
- No execute button, no success message that implies Airtable write capability.
- No restore write enablement.

## What Is Not in This Version

- No automatic token retrieval for connection validation — the token must still be entered
  manually when checking a connection. Wiring saved token → connection check is deferred.
- No multi-account support — only one token per `CredentialKind` is stored.
- No token rotation or expiry detection.
