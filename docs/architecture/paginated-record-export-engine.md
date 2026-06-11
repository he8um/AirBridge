# Paginated Record Export Engine

## Overview

The record export engine fetches all records from an Airtable base using the existing `AirtableClient<T>` abstraction, converts them to JSONL, extracts linked-record references and attachment metadata, and builds a `PackageInput` ready for the package writer.

This is a **backend-only, test-only-wired** implementation in V0.1. There is no UI button to trigger live export. All tests use mocked HTTP transport — no live network calls are made.

## Module Map

| Module | Purpose |
|---|---|
| `backup/export_engine.rs` | Pagination loop, multi-table orchestration, error mapping |
| `backup/record_jsonl.rs` | `AirtableRecord` → JSONL line conversion |
| `backup/linked_records.rs` | Extract `LinkedRecordReference` from `multipleRecordLinks` fields |
| `backup/attachments.rs` | Extract `AttachmentMetadata` from `multipleAttachments` fields (no URLs) |
| `backup/export_result.rs` | `RecordExportEngineResult`, `TableExportResult`, `build_package_input()` |
| `airtable/http.rs` | Added `SequentialMockTransport` for multi-page pagination tests |

## Pagination Loop

The engine calls `client.list_records(base_id, table_id, &opts)` in a loop, following the `offset` cursor from each response until none is returned:

```
loop {
    resp = client.list_records(base_id, table_id, opts)
    accumulate resp.records
    if resp.offset is None → break
    opts.offset = resp.offset
    if pages >= MAX_PAGES_PER_TABLE → return PageLimitReached error
}
```

`MAX_PAGES_PER_TABLE = 10_000` guards against runaway loops on unexpectedly large tables.

## Error Mapping

`AirtableClientError` variants map 1:1 to `ExportEngineError`:

| Client Error | Engine Error |
|---|---|
| `InvalidToken` | `InvalidToken` |
| `RateLimited` | `RateLimited` |
| `PermissionDenied` | `PermissionDenied` |
| `MissingScope` | `MissingScope` |
| `NotFound` | `NotFound` |
| `MalformedResponse(msg)` | `MalformedResponse(msg)` |
| `TransientServerError(s)` | `TransientServerError(s)` |

## JSONL Format

Each record is serialised as a single-line JSON object:

```json
{"id":"recXXX","createdTime":"2026-01-01T00:00:00.000Z","fields":{...}}
```

No attachment URLs are written to JSONL lines. Field values are written as-is from the Airtable API response (callers should pre-strip attachment objects if full field removal is desired).

## Linked Record References

Fields with `type = "multipleRecordLinks"` are scanned. For each non-empty array, a `LinkedRecordReference` is emitted:

```json
{"sourceRecordId":"recSrc01","fieldName":"Tasks","linkedRecordIds":["recLink01","recLink02"]}
```

All references across all tables are merged into a single `linked-records.jsonl` blob in the `PackageInput`.

## Attachment Metadata Policy (V0.1)

**Full attachment URLs are never stored.** Only structural metadata is captured:

```json
{"recordId":"rec001","fieldName":"Files","attachmentId":"attAbc01","filename":"photo.png","contentType":"image/png","sizeBytes":1024,"urlPresent":true}
```

`urlPresent: true` records that the API returned a URL without preserving it. This allows future phases to identify which records had downloadable files without re-fetching.

## SequentialMockTransport

`MockHttpTransport` returns the same body for every call, which is sufficient for single-page tests. `SequentialMockTransport` pops responses from a queue, allowing multi-page pagination tests:

```rust
let transport = SequentialMockTransport::new(vec![
    (200, page1_body),
    (200, page2_body),
]);
```

Once the queue is exhausted to one entry, the last response is repeated (so the pagination loop correctly sees `offset: None` on the final page).

## Integration Test

`tests/export_engine_integration.rs` covers the full pipeline end-to-end:

1. Two-table, two-page mock export
2. Engine produces `RecordExportEngineResult`
3. `build_package_input()` wraps it in a `PackageInput`
4. `write_package()` writes to a tempdir path
5. `validate_package()` confirms `ValidationStatus::Valid`
6. Assertions: no token sentinel, no absolute paths, no attachment URLs

Packages are **only written to `tempfile::tempdir()`** — never to user-selected paths.

## Safety Constraints

- No live network calls in any test.
- No token persistence in any struct or output.
- No attachment URLs stored anywhere.
- Packages written only to temp directories.
- Generated `.airbridge` files are never committed to the repository.
- No UI production export flow in V0.1.

## Future Path

- Wire `run_export()` into a Tauri command (behind a flag) in a future session.
- Add retry logic for `RateLimited` responses.
- Implement `AttachmentPolicy::Download` for full attachment download in a later phase.
- Add progress event emission via Tauri events for UI progress tracking.
