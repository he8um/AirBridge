# Live Connection Check

## Status

Implemented. The connection check performs a real read-only Airtable API call.
No token is persisted, logged, or returned.

## What happens

When a user submits a Personal Access Token in the connection form:

1. The token is held only in React component state until the form is submitted.
2. On submit, the token is passed to the Rust Tauri command `check_connection`.
3. The Rust command validates the token is non-empty, wraps it in `AirtableToken`,
   and immediately drops the raw string.
4. `AirtableClient` uses the wrapped token to build an `Authorization: Bearer`
   header for a single GET request to the Airtable list-bases endpoint.
5. The result is parsed, mapped to a `ConnectionCheckResult`, and returned.
6. The raw token is not present in the result, error, or any log output.
7. The frontend clears the token from component state in the `finally` block,
   regardless of success or failure.

## Read-only guarantee

The connection check calls only the list-bases metadata endpoint:

```
GET https://api.airtable.com/v0/meta/bases
```

No records are read, written, or deleted. No schema modifications are made.
This endpoint requires `schema.bases:read` scope at minimum.

## Token safety rules

- `AirtableToken`'s `Debug` and `Display` implementations print `[redacted]`.
- The type does not implement `Serialize` or `Deserialize`.
- `authorization_header_value()` is the only way to produce the bearer string.
- The Tauri command drops the raw `String` input immediately after wrapping.
- Error messages are mapped to generic text — they never echo the token value.
- The `hasSecretLeak` guard in the frontend verifies the returned JSON does not
  contain the token before accepting it.
- Tests use clearly synthetic sentinel strings, not real tokens.

## Permission check results

| Permission         | Status after success | Status after failure   |
|--------------------|---------------------|------------------------|
| schema.bases:read  | Passed              | Failed (with reason)   |
| data.records:read  | Passed              | Unknown                |
| schema.bases:write | Unknown             | Unknown                |
| data.records:write | Unknown             | Unknown                |

Write permissions are never verified destructively. They are always marked
`Unknown` (not verified) to avoid creating, updating, or deleting any data.

## Error mapping

| HTTP status | Displayed error                         |
|-------------|-----------------------------------------|
| 401         | Invalid or expired token                |
| 403 + scope | Token is missing required scopes        |
| 403         | Permission denied                       |
| 429         | Rate limited — try again later          |
| 5xx / IO    | Network or server error                 |
| Bad JSON    | Network or server error                 |

None of these messages include the token value.

## No persistence

Tokens are not written to disk, databases, OS keychains, environment variables,
or any persistent store. Connection profiles are not saved in this version.
Keychain integration is planned for a future session.

## Test strategy

- All Rust tests use `MockHttpTransport` — no live network calls.
- Frontend tests inject `mockAirBridgeService` via the `service` prop on
  `ConnectionForm` — Tauri IPC is never invoked from tests.
- Tests verify: token absent from serialized result, write permissions not
  marked `passed`, error messages do not contain token, DOM does not contain
  token after submit.
