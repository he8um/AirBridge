# AirBridge Test Fixtures

This directory contains synthetic test fixtures used during AirBridge development and automated testing.

## Purpose

These fixtures simulate `.airbridge` backup packages — the output format produced by AirBridge when backing up an Airtable base. They allow the test suite to run entirely offline without requiring a live Airtable connection or real API credentials.

**All data in this directory is fabricated.** No fixture represents a real Airtable base, real user records, or real credentials of any kind. Names, identifiers, and values are invented solely to exercise application logic.

## Directory Structure

```
fixtures/
└── airtable/
    ├── simple-base/              # A minimal two-table base with basic field types
    │   ├── manifest.json         # Backup metadata and summary counts
    │   ├── schema.json           # Table and field definitions
    │   └── records.jsonl         # Record data, one JSON object per line
    │
    ├── linked-records-base/      # A three-table base with linked record relationships
    │   ├── manifest.json
    │   ├── schema.json
    │   └── records.jsonl
    │
    ├── field-types-base/         # A single-table base covering many Airtable field types
    │   ├── manifest.json
    │   ├── schema.json
    │   └── records.jsonl
    │
    └── corrupted-backup/         # An intentionally empty/broken fixture for error-handling tests
        └── README.md
```

## Usage

- Unit and integration tests import these files directly from the `fixtures/` path.
- Fixtures are read-only — tests must not modify them. If a test needs a mutated version, copy the fixture into a temporary directory at test setup time.
- When adding a new fixture, follow the naming conventions and data requirements described in `docs/qa/test-fixtures.md`.

## Data Requirements

- All record field values must be obviously synthetic (e.g., "Example Base", "rec001", "example@example.com").
- No real personal information, credentials, or URLs pointing to real services.
- Base IDs and record IDs must use the `fixture-` or `Example` prefix to make their synthetic nature unambiguous.
