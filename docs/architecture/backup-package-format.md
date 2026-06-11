# Backup Package Format

## Status

Foundation implemented. Writer, reader, and validator are tested with synthetic data only. No live Airtable record export is connected yet.

## Overview

A `.airbridge` file is a ZIP-compatible archive. It is self-describing (manifest carries all metadata), portable (no local paths inside), and verifiable (SHA-256 checksums for all entries).

## Rust module layout

| Module | Responsibility |
|---|---|
| `backup/format.rs` | Format constants: `FORMAT_NAME`, `FORMAT_VERSION`, `PACKAGE_EXTENSION`, required entry paths. |
| `backup/manifest.rs` | `PackageManifest` struct with serde round-trip. Source, contents, security, package identity. |
| `backup/checksums.rs` | `sha256_hex()` helper, `ChecksumMap` type alias, JSON serialization helpers. |
| `backup/package.rs` | `PackageInput` (write-side input DTO) and `TableRecords` (per-table JSONL lines). |
| `backup/writer.rs` | `write_package(path, input)` — creates a ZIP, writes all entries, computes and appends checksums. |
| `backup/reader.rs` | `BackupPackageReader` — opens a ZIP, reads entries in memory, deserializes manifest and checksums. |
| `backup/validation.rs` | `validate_package(path)` — extension check, required-entry check, manifest validation, checksum verification. Returns `ValidationReport`. |

## Write flow

```
PackageInput { manifest_json, base_json, schema_json, tables[], ... }
    │
    ▼
write_package(dest_path, input)
    │  ├─ writes manifest.json
    │  ├─ writes base.json, schema.json
    │  ├─ writes tables/<id>/records.jsonl for each table
    │  ├─ writes attachments/metadata.jsonl (if provided)
    │  ├─ writes links/linked-records.jsonl (if provided)
    │  ├─ writes reports/backup-report.json
    │  ├─ writes reports/compatibility-report.json (if provided)
    │  └─ writes checksums/sha256.json (last, covers all previous entries)
    └─ returns Ok(()) or WriteError
```

## Read flow

```
BackupPackageReader::open(path)
    ├─ read_manifest()        → PackageManifest
    ├─ read_checksums()       → ChecksumMap
    ├─ read_schema()          → Vec<u8>
    ├─ read_entry(name)       → Vec<u8>
    ├─ entry_names()          → Vec<String>
    ├─ entry_count()          → usize
    └─ has_required_entries() → bool
```

## Validation flow

```
validate_package(path)
    ├─ Check file extension (.airbridge expected)
    ├─ Open archive
    ├─ Check all REQUIRED_ENTRIES are present
    ├─ Parse manifest.json
    │   ├─ Verify format == "airbridge"
    │   └─ Verify format_version == "0.1.0"
    ├─ Parse checksums/sha256.json
    └─ For each checksummed entry: sha256(content) == stored_hash
    └─ Returns ValidationReport { status, errors, warnings, entry_count, manifest_summary }
```

## Manifest invariants

- `format` is always `"airbridge"`.
- `format_version` is always `"0.1.0"` in this release.
- No token field exists on `ManifestSource` or anywhere in the manifest.
- No local filesystem paths exist inside the archive entries.
- `security.encrypted` is always `false` in V0.1.

## Checksum behavior

- `checksums/sha256.json` is written as the final archive entry.
- It covers all entries written before it (manifest, base, schema, tables, attachments, reports).
- It does not cover itself.
- Validation reads the checksum file, then verifies each covered entry.
- A single hash mismatch produces a `CHECKSUM_MISMATCH` error and status `Invalid`.

## JSONL record format

Per-table records are stored at `tables/<table_id>/records.jsonl`. Each line is a JSON object representing one record. JSONL is preferred over a JSON array because:
- Individual records can be read and validated without parsing the whole file.
- Partial writes are detectable (truncated lines fail JSON parse).
- Large record sets do not require full in-memory buffering.

## Attachment metadata (V0.1)

`attachments/metadata.jsonl` contains one JSON object per attachment field occurrence. File content is NOT exported. Each line captures: `fieldId`, `url`, `filename`, `size`, `mimeType`. The URL is the Airtable-provided URL at backup time — it may expire.

## TypeScript mirror types

Defined in `src/backend/types.ts`:

```typescript
export type PackageValidationStatus = "valid" | "invalid" | "warning";
export interface PackageValidationIssue { code, message }
export interface PackageManifestSummary { format, formatVersion, appVersion, createdAt, baseId, baseName, tableCount, recordCount }
export interface PackageValidationReport { status, errors, warnings, entryCount, manifestSummary? }
```

These are not yet connected to a Tauri command. They are available for future UI integration.

## Future path

1. Record export engine writes JSONL records into `PackageInput.tables`.
2. `write_package` creates the real `.airbridge` file at a user-selected path.
3. A `validate_backup_package` Tauri command is wired up for post-write verification.
4. The Backups page gains a file path selector and "Start Backup" button.
