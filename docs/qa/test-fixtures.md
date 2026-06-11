# Test Fixtures

## Overview

The `fixtures/` directory at the repository root contains synthetic backup packages used by AirBridge's test suite. These files simulate `.airbridge` backup packages without requiring a live Airtable connection or real user data.

All fixture data is fabricated. No fixture file contains real base IDs, real record values, real credentials, or any information about real users or organizations.

---

## Directory Contents

```
fixtures/
└── airtable/
    ├── simple-base/
    ├── linked-records-base/
    ├── field-types-base/
    └── corrupted-backup/
```

### `simple-base`

A minimal two-table backup fixture. Designed as the baseline fixture for most unit and integration tests.

- **Tables:** `Projects` (4 fields), `Tasks` (4 fields)
- **Records:** 3 project records, 2 task records
- **Field types used:** singleLineText, singleSelect, date, checkbox
- **Use for:** manifest parsing, schema loading, basic record round-trips, record count assertions

### `linked-records-base`

A three-table fixture with linked record relationships between tables.

- **Tables:** `Clients`, `Projects`, `Tasks`
- **Records:** 3 clients, 3 projects, 2 tasks
- **Field types used:** singleLineText, email, multipleRecordLinks
- **Use for:** linked record restore tests, record ID remapping logic, dependency-order table creation

### `field-types-base`

A single-table fixture with one record per field type. Intended to ensure that every supported Airtable field type is parsed and serialized correctly.

- **Tables:** `Example Table` (15 fields)
- **Records:** 2 records
- **Field types used:** singleLineText, multilineText, number, currency, percent, checkbox, singleSelect, multipleSelects, date, dateTime, email, url, phoneNumber, rating, duration
- **Use for:** field type parser coverage, serialization round-trip tests, schema option preservation

### `corrupted-backup`

An intentionally empty directory with no valid backup files.

- **Use for:** error handling tests — verifies that the application shows a useful error state when a backup package cannot be opened, rather than crashing or silently failing.
- See `fixtures/airtable/corrupted-backup/README.md` for guidance on adding sub-scenarios.

---

## How to Use Fixtures in Tests

### Rust (cargo test)

Load fixture files using `std::path::PathBuf` relative to the `CARGO_MANIFEST_DIR` environment variable, which points to the crate root at compile time:

```rust
fn fixture_path(rel: &str) -> std::path::PathBuf {
    let root = std::path::PathBuf::from(env!("CARGO_MANIFEST_DIR"));
    root.join("../../fixtures").join(rel)
}

#[test]
fn test_parse_simple_manifest() {
    let path = fixture_path("airtable/simple-base/manifest.json");
    let content = std::fs::read_to_string(path).unwrap();
    let manifest: Manifest = serde_json::from_str(&content).unwrap();
    assert_eq!(manifest.backup_id, "fixture-simple-base-001");
}
```

### TypeScript / Vitest

Import fixture JSON directly using Vite's JSON import:

```typescript
import manifest from '../../fixtures/airtable/simple-base/manifest.json';

it('displays the correct base name', () => {
  render(<BackupSummary manifest={manifest} />);
  expect(screen.getByText('Example Base')).toBeInTheDocument();
});
```

For JSONL files, read and split by newline in the test setup:

```typescript
import { readFileSync } from 'fs';
import { join } from 'path';

function loadRecords(fixture: string): object[] {
  const filePath = join(__dirname, '../../fixtures', fixture);
  return readFileSync(filePath, 'utf-8')
    .split('\n')
    .filter(Boolean)
    .map(JSON.parse);
}
```

### Integration Tests

Integration tests that test the Tauri command layer can pass fixture file paths as inputs to commands under test. Fixture paths should be resolved relative to the repository root using an environment variable set in the test runner configuration.

---

## Fixture Naming Conventions

| Convention | Rationale |
|-----------|-----------|
| Base IDs: `appExample<Name>01` | Clearly fake, matches Airtable's `app` prefix |
| Table IDs: `tbl<Name>01` | Matches Airtable's `tbl` prefix |
| Field IDs: `fld<Name>01` | Matches Airtable's `fld` prefix |
| Record IDs: `rec<Name>001` or `rec<NNN>` | Clearly fake, matches Airtable's `rec` prefix |
| Backup IDs: `fixture-<name>-001` | Unambiguous synthetic prefix |
| Email addresses: `*@example.com` | RFC 2606 reserved domain — safe for synthetic data |
| URLs: `https://example.com/*` | RFC 2606 reserved domain — safe for synthetic data |
| Phone numbers: `+1-555-01NN` | 555 numbers are reserved for fictional use |

---

## How to Add New Fixtures

1. Create a new subdirectory under `fixtures/airtable/` with a descriptive name (e.g., `attachments-base`).
2. Add `manifest.json`, `schema.json`, and `records.jsonl` following the format of existing fixtures.
3. Ensure all values are synthetic (see naming conventions above).
4. Update the fixture set description in this document (`docs/qa/test-fixtures.md`).
5. Update `fixtures/README.md` with a one-line entry for the new fixture.

Do not commit fixture files that contain:
- Real Airtable base IDs or record IDs from production bases
- Real email addresses, phone numbers, or URLs
- Real API tokens or credentials of any kind
- Data that could identify a real person or organization

---

## Fixture Data Requirements Checklist

Before committing a new fixture, verify:

- [ ] All base IDs use the `appExample` prefix.
- [ ] All record IDs use the `rec` prefix with a clearly invented suffix.
- [ ] All email addresses use `@example.com`.
- [ ] All URLs use `https://example.com/`.
- [ ] All phone numbers use the `+1-555-01xx` range.
- [ ] The `backupId` in `manifest.json` starts with `fixture-`.
- [ ] `recordCount` in `manifest.json` equals the number of non-empty lines in `records.jsonl`.
- [ ] `tableCount` in `manifest.json` equals the number of entries in `tables` in `schema.json`.
- [ ] The fixture is documented in this file and in `fixtures/README.md`.
