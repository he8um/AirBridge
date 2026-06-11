# Airtable API Client

## Status

Active. The client skeleton is complete and the live connection check is wired
to the `check_connection` Tauri command via `ReqwestHttpTransport`. All tests
use `MockHttpTransport` — no live network calls in the test suite.

## Purpose

`apps/desktop/src-tauri/src/airtable/` provides the Rust-side foundation
for communicating with the Airtable REST API. It is designed so that:

- All interaction with Airtable is channelled through a single typed client.
- The HTTP transport is abstracted behind a trait, making tests independent
  of any real network.
- Token values are protected at the type level and cannot leak via `Debug`,
  `Display`, serialization, or error messages.

## Module Map

| Module         | Responsibility                                                  |
|----------------|-----------------------------------------------------------------|
| `auth`         | `AirtableToken` wrapper — safe header construction, no logging  |
| `client`       | `AirtableClient<T>` — typed methods over an HTTP transport      |
| `endpoints`    | URL builders for all API paths                                  |
| `errors`       | `AirtableClientError` enum and HTTP-status error mapping        |
| `http`         | `HttpTransport` trait, request/response types, `MockHttpTransport` |
| `models`       | Typed structs for bases, tables, fields, records, and requests  |
| `pagination`   | `PageSize`, `PaginationOffset`, `ListRecordsOptions`, query builders |
| `rate_limit`   | `AirtableRateLimitPolicy` — per-base and token-level limits     |
| `records`      | Batch-size constant and batch-splitting helpers                 |
| `schema`       | Field-compatibility classification helpers                      |

## Authentication

Airtable uses Personal Access Tokens (PATs) sent as `Authorization: Bearer <token>`.

Rules enforced by `AirtableToken`:

- `Debug` prints `[redacted]`, not the token value.
- `Display` prints `[redacted]`.
- The struct does not derive `Serialize` or `Deserialize`.
- `authorization_header_value()` is the only way to produce the header, and
  its return value should be consumed immediately, not stored.
- No test uses a real token. Tests use clearly synthetic sentinel strings.

Required PAT scopes for V0.1 operations:

| Scope                 | Required for          |
|-----------------------|-----------------------|
| `data.records:read`   | Record export         |
| `data.records:write`  | Record restore/write  |
| `schema.bases:read`   | Schema backup         |
| `schema.bases:write`  | Schema restore        |

## Endpoint Categories

- **Metadata** (`https://api.airtable.com/v0/meta/bases`): list accessible bases.
- **Schema** (`/meta/bases/{baseId}/tables`): retrieve table and field schema.
- **Records** (`/v0/{baseId}/{tableId}`): list, create, and update records.

All path builders are pure functions accepting IDs as parameters. No real
base or table IDs appear in the source.

## Pagination

- Record listing is paginated; pages contain up to 100 records.
- `PageSize` clamps values to `[1, 100]`.
- `PaginationOffset` holds the opaque cursor string returned by Airtable.
- `ListRecordsOptions` bundles page size, offset, field selection, and sort.
- `build_list_query_params` converts options to a URL query parameter map.

## Rate Limit Policy

`AirtableRateLimitPolicy` captures the limits documented by Airtable:

| Parameter                    | Default |
|------------------------------|---------|
| Per-base requests per second | 5       |
| Token-level requests per second | 50   |
| Cooldown after 429           | 30 s    |
| Max retries                  | 3       |
| Initial backoff              | 1 s     |
| Backoff multiplier           | 2×      |

This is a policy/configuration type only. Enforcement is the
responsibility of the HTTP transport layer (not yet implemented).

## Error Mapping

HTTP status codes are mapped to `AirtableClientError` variants:

| HTTP status | Error variant           |
|-------------|-------------------------|
| 401         | `InvalidToken`          |
| 403 + scope | `MissingScope`          |
| 403         | `PermissionDenied`      |
| 404         | `NotFound`              |
| 429         | `RateLimited`           |
| 400         | `ValidationError`       |
| 5xx         | `TransientServerError`  |
| Bad JSON    | `MalformedResponse`     |

## Test Strategy

- All tests are unit tests with no network calls.
- `MockHttpTransport` returns a pre-configured `HttpResponse`, simulating
  any status code and body.
- Token safety tests verify `Debug` and `Display` output does not contain
  the sentinel string.
- Pagination tests verify `PageSize` clamping and query param generation.
- Record batching tests verify correct splitting and total-count preservation.
- Error mapping tests cover every distinct HTTP status variant.
- Client tests verify JSON parsing and error propagation end-to-end using
  the mock transport.

## Schema Summary

`airtable/schema.rs` exposes `summarize_schema(base_id, tables)` which converts a
`Vec<AirtableTable>` into a `BaseSchemaSummary`. The summary contains:

- `base_id` — the base identifier
- `table_count` — number of tables
- `tables` — per-table summaries with field counts, sorted field type histograms,
  and per-table compatibility counts
- `compatibility` — aggregate counts: `restorableCount`, `metadataOnlyCount`,
  `unknownCount`, `totalCount`

Field compatibility classification uses `classify_field` from the same module.
The summary never contains field values or token material.

## Future Work

- **Live connection check**: Done — wired to `check_connection` Tauri command
  via `check_connection_for_token()` and `ReqwestHttpTransport`.
- **Base catalog**: Done — `list_accessible_bases()` on `AirtableClient` and
  `list_accessible_bases` Tauri command.
- **Schema read**: Done — `get_base_schema()` on `AirtableClient` + `summarize_schema()`
  produce a `BaseSchemaSummary` wired to the `get_base_schema` Tauri command.
- **Record export**: paginated record listing with continuation logic.
- **Write batching**: automatic splitting of large create/update payloads
  using `records::split_create_batches`.
- **Retry queue**: async retry loop respecting `AirtableRateLimitPolicy`.
- **Backoff/cooldown**: actual `tokio::time::sleep` enforcement after 429.
