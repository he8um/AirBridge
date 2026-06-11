# Backup Format

AirBridge uses a `.airbridge` package as the backup artifact.

## File type

A `.airbridge` file is a standard ZIP archive with a documented internal layout.

## Format goals

The format should be:

- Portable.
- Inspectable.
- Versioned.
- Stream-friendly.
- Suitable for large record sets.
- Suitable for validation.
- Suitable for best-effort restore.

## Package layout

```text
example.airbridge
├── manifest.json
├── base.json
├── schema.json
├── tables/
│   ├── tbl_xxx/
│   │   ├── table.json
│   │   ├── fields.json
│   │   ├── records.jsonl
│   │   └── records.csv
├── links/
│   └── linked-records.jsonl
├── attachments/
│   └── metadata.jsonl
├── reports/
│   ├── backup-report.json
│   ├── compatibility-report.json
│   └── validation-report.json
└── checksums/
    └── sha256.json
```

## manifest.json

The manifest identifies the package, source, format version, counts, and security metadata.

Example:

```json
{
  "format": "airbridge",
  "format_version": "0.1.0",
  "app_version": "0.1.0",
  "created_at": "2026-06-11T00:00:00Z",
  "source": {
    "provider": "airtable",
    "base_id": "appXXXXXXXXXXXXXX",
    "base_name": "Marketing Ops",
    "workspace_id": "wspXXXXXXXXXXXXXX"
  },
  "contents": {
    "tables": 8,
    "fields": 126,
    "records": 24392,
    "linked_record_relationships": 18,
    "attachments": 320
  },
  "security": {
    "contains_record_data": true,
    "contains_attachment_urls": false,
    "encrypted": false,
    "redactions_applied": ["emails", "collaborators"]
  }
}
```

## records.jsonl

Each line is one record. JSONL is used because it is easier to stream and validate than a single large JSON array.

Example:

```json
{"id":"recOld123","createdTime":"2026-06-01T10:00:00.000Z","fields":{"Campaign Name":"Summer Sale","Status":"Live"}}
```

## records.csv

CSV is included for human inspection. It is not the restore source of truth because CSV cannot preserve all Airtable value structures reliably.

## checksums

`checksums/sha256.json` contains hashes for package files. Validation should fail if required file checksums do not match.

## Versioning

The backup format is versioned separately from the app version. Pre-1.0 format changes may be breaking and must be documented.
