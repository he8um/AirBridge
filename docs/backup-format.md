# Backup Format

AirBridge uses a `.airbridge` package as the backup artifact.

## Status

**Foundation implemented.** The package writer, reader, and validator are tested with synthetic data. Live record export is not yet connected — the UI does not create backup files. The format is stable at V0.1.

## File type

A `.airbridge` file is a standard ZIP archive with a documented internal layout. It can be inspected with any ZIP tool.

## Format goals

- Portable — no platform-specific paths or encodings.
- Inspectable — standard ZIP, JSON, and JSONL.
- Versioned — `format_version` in `manifest.json`.
- Stream-friendly — JSONL records are streamed line-by-line.
- Suitable for validation — SHA-256 checksums for all entries.
- Suitable for best-effort restore — manifest carries source metadata without tokens.

## Package layout

```text
example.airbridge
├── manifest.json                     ← required
├── base.json                         ← required
├── schema.json                       ← required
├── tables/
│   └── <table_id>/
│       └── records.jsonl
├── links/
│   └── linked-records.jsonl
├── attachments/
│   └── metadata.jsonl
├── reports/
│   ├── backup-report.json            ← required
│   ├── compatibility-report.json
│   └── validation-report.json
└── checksums/
    └── sha256.json                   ← required
```

**Required entries** (package is invalid without these): `manifest.json`, `base.json`, `schema.json`, `reports/backup-report.json`, `checksums/sha256.json`.

## manifest.json

Identifies the package, source, format version, counts, and security metadata. Contains no tokens, no local filesystem paths, and no real user credentials.

```json
{
  "format": "airbridge",
  "formatVersion": "0.1.0",
  "appVersion": "0.1.0",
  "createdAt": "2026-06-11T00:00:00Z",
  "source": {
    "provider": "airtable",
    "baseId": "appXXXXXXXXXXXXXX",
    "baseName": "Marketing Ops",
    "workspaceId": "wspXXXXXXXXXXXXXX"
  },
  "contents": {
    "tables": 8,
    "fields": 126,
    "records": 24392,
    "linkedRecordRelationships": 18,
    "attachments": 320
  },
  "security": {
    "containsRecordData": true,
    "containsAttachmentUrls": false,
    "encrypted": false,
    "redactionsApplied": ["emails", "collaborators"]
  },
  "package": {
    "generatedByApp": "airbridge",
    "packageId": "<uuid-v4>"
  }
}
```

## records.jsonl

Each line is one record. JSONL is used because it streams and validates better than a large JSON array.

```json
{"id":"recOld123","createdTime":"2026-06-01T10:00:00.000Z","fields":{"Campaign Name":"Summer Sale","Status":"Live"}}
```

## checksums/sha256.json

A JSON object mapping archive entry path → SHA-256 hex digest. Computed at write time. The checksum file itself is written last and is not checksummed. Validation fails if any checksummed entry has a mismatched hash.

```json
{
  "attachments/metadata.jsonl": "abc123...",
  "base.json": "def456...",
  "manifest.json": "ghi789...",
  "reports/backup-report.json": "jkl012...",
  "schema.json": "mno345..."
}
```

## Attachment policy (V0.1)

Attachment fields are captured as metadata only. File content is not exported. `security.containsAttachmentUrls` is `false` by default in V0.1.

## Linked record policy (V0.1)

Linked record fields capture the source record IDs. Restore will require remapping IDs to the destination base.

## Security invariants

- No API tokens appear inside the package.
- No local filesystem paths appear inside the archive entries.
- `encrypted` is always `false` in V0.1. Encryption is reserved for a future version.

## Versioning

The backup format is versioned independently from the app version via `formatVersion` in `manifest.json`. Pre-1.0 format changes may be breaking and will be documented in the changelog.

## Future path

When the record export engine is complete, it will feed data into the package writer. At that point the UI will expose a file path selector and a "Start Backup" button that creates a real `.airbridge` file.
