# Base Catalog and Schema Flow

## Status

Implemented. Two read-only Tauri commands provide base catalog and schema data.
No token is persisted. No write operations are performed.

## What happens

### Listing accessible bases

1. The frontend calls `listAccessibleBases({ token })` on `AirBridgeService`.
2. The live service forwards the token to `commands.listAccessibleBases(token)`.
3. The Rust command validates the token is non-empty, wraps it in `AirtableToken`,
   and immediately drops the raw string.
4. `AirtableClient.list_accessible_bases()` calls `GET /v0/meta/bases`.
5. The response is parsed into `Vec<AccessibleBaseSummary>` containing only `id` and `name`.
6. The result is returned to the frontend. The token is not in the response.

### Reading a base schema

1. The frontend calls `getBaseSchema({ token, baseId })` on `AirBridgeService`.
2. The live service forwards both values to `commands.getBaseSchema(token, baseId)`.
3. The Rust command validates inputs, wraps the token in `AirtableToken`, drops the raw string.
4. `AirtableClient.get_base_schema(base_id)` calls `GET /v0/meta/bases/{baseId}/tables`.
5. The response is passed to `schema::summarize_schema(base_id, &tables)`, producing
   a `BaseSchemaSummary` with table/field counts and compatibility breakdowns.
6. The summary is returned to the frontend. The token is not in the response.

## Token flow

Both commands use the session-local pattern: the token arrives as a command parameter,
is wrapped immediately in `AirtableToken`, and the raw `String` is `drop()`-ed. The token
is never stored, cached, logged, or returned in any field.

This is the same pattern used by `check_connection`.

## Read-only guarantee

Both commands call only Airtable metadata endpoints:

```
GET https://api.airtable.com/v0/meta/bases
GET https://api.airtable.com/v0/meta/bases/{baseId}/tables
```

No records are read, written, or deleted. No schema modifications are made.

## Response shapes

### `AccessibleBaseSummary`

```typescript
{ id: string; name: string }
```

### `BaseSchemaSummary`

```typescript
{
  baseId: string;
  tableCount: number;
  tables: Array<{
    id: string;
    name: string;
    fieldCount: number;
    fieldTypeCounts: Array<{ fieldType: string; count: number }>;
    compatibility: SchemaCompatibilitySummary;
  }>;
  compatibility: SchemaCompatibilitySummary;
}

interface SchemaCompatibilitySummary {
  restorableCount: number;
  metadataOnlyCount: number;
  unknownCount: number;
  totalCount: number;
}
```

## Compatibility classification

Field compatibility is determined by `schema::classify_field`:

| Classification  | Field types                                                                |
|-----------------|----------------------------------------------------------------------------|
| `Restorable`    | singleLineText, multilineText, number, currency, percent, singleSelect,    |
|                 | multipleSelects, checkbox, date, dateTime, duration, email, url,           |
|                 | phoneNumber, rating                                                        |
| `MetadataOnly`  | formula, rollup, count, lookup, createdTime, lastModifiedTime, createdBy,  |
|                 | lastModifiedBy, autoNumber, externalSyncSource                             |
| `Unknown`       | Any type not in the above lists                                            |

`MetadataOnly` fields are captured in schema backups but their computed values
cannot be restored via the API. `Unknown` types are treated conservatively.

## UI integration

- `ConnectionForm` shows accessible base names after a successful connection check.
  `ConnectionCheckResult` includes `accessibleBases` (populated on success, absent on failure).
- `BackupsPage` shows an "Available Bases" catalog card from app state.
  The base select dropdown is enabled when bases are loaded.

## Test strategy

- All Rust tests use `MockHttpTransport` — no live network calls.
- `list_accessible_bases_with_result` and `get_base_schema_with_result` are
  `#[cfg(test)]` helpers that bypass the transport for unit testing command logic.
- `schema::summarize_schema` tests verify counts, sorted field type histograms,
  and that no token sentinel appears in serialized output.
- Frontend tests inject `mockAirBridgeService` via service props — Tauri IPC is
  never invoked from tests.
- Token safety tests verify that neither the JSON result nor the DOM contains
  the token sentinel after a successful call.
