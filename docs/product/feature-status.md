# Feature Status Matrix

**Version:** 0.1.0-alpha  
**Updated:** 2026-06-13

This matrix describes the current status of each feature area. Status values:

- **Complete** — implemented, tested, and available in the current build.
- **Partial** — core functionality implemented; some capabilities deferred.
- **Disabled** — contract and gate implemented; execution blocked.
- **Planned** — not yet implemented.

---

| Feature | Status | User-Facing Availability | Safety Notes | Next Work |
|---------|--------|--------------------------|-------------|-----------|
| Connection check | Complete | Available — Connections page | Token validated locally before API call; never stored; never in result | None |
| Base catalog / schema read | Complete | Available — Connections, Backups pages | Token consumed per call; never stored | None |
| Backup plan | Complete | Available — Backups page | No token required; no writes; read-only planning | None |
| Records export plan | Complete | Available — Backups page | No token required; no writes; read-only planning | None |
| Backup execution | Complete | Available — Backups page | Requires file picker + `CREATE BACKUP` confirmation; token consumed; path validated; filename-only in result | None |
| Backup progress / cancellation | Partial | Available — polling only | No destructive behavior on cancel | Streaming progress deferred |
| Package inspection | Complete | Available — Restore page | Read-only; no token; full path never in result | None |
| Restore dry-run | Complete | Available — Restore page | No token; no API calls; no writes; full path never in result | None |
| Restore execution gate | Complete | Available — Restore page | All preconditions validated; token checked for presence only; write engine disabled; `noChangesMade` always true | Enable write engine |
| Restore schema creation plan | Complete | Available — Restore page | No token; no Airtable calls; no writes; table-first ordering; field classification; dependency graph; `noChangesMade` always true | None |
| Restore record import plan | Complete | Available — Restore page | No token; no Airtable calls; no writes; batch planning (size 10); field import policies; linked record second-pass; attachment metadata; checkpoint; retry policy; `noChangesMade` always true | None |
| Restore write engine | Disabled | Not available — returns `readyButDisabled` | No Airtable writes in this version | Implement write engine, linked record remapping, post-restore verification |
| Credential / token storage | Planned | Not available | Tokens must be entered per-operation | OS keychain integration |
| Local job history | Complete | Available — Reports page | Memory-only; no tokens; no full paths; no record payloads; summary-level only | SQLite persistence deferred |
| Streaming progress events | Planned | Not available | — | Tauri event stream |
| Attachment file download | Planned | Not available — metadata only | Attachment URLs captured at backup time only; may expire | File download and storage engine |
| Attachment URL preservation | Partial | Metadata and URL in package | URL valid at backup time only | Link freshness / re-fetch strategy |
| Formula / rollup field restore | Not applicable | — | These fields cannot be created via the Airtable API; listed in compatibility report as unsupported | API limitation; manual recreation required |
| Automation / interface backup | Not applicable | — | Not part of v0.1 scope | Out of scope |
| Restore into non-empty base | Blocked by design | Not available | Prevents destructive overwrites | Not planned for v0.1 |
| Merge / overwrite restore | Blocked by design | Not available | Prevents data loss | Not planned for v0.1 |
| System field restore (created time, etc.) | Not applicable | — | Airtable API does not support writing system fields | API limitation |
| Cross-platform release builds | Planned | Not yet verified | — | CI pipeline verification |

---

## Detailed Notes

### Backup Execution

The backup execution contract (`run_backup_job`) is the only command that writes a file. It requires:

1. Explicit output path selection via the OS file picker.
2. Exact confirmation text `CREATE BACKUP` from the user.
3. Output path validation (extension, parent directory, no traversal, not an existing directory).

The token is consumed to build the Airtable HTTP client and discarded. It never appears in the result. The full output path never appears in the result — only the filename.

### Restore Execution Gate

The restore execution gate (`run_restore_execution`) validates seven preconditions in order:

1. Package inspected (non-empty filename).
2. Package validation status is `valid` or `warning`.
3. Dry-run plan exists (non-empty status).
4. Dry-run status is `ready` or `readyWithWarnings`.
5. Package path non-empty (target mode implicitly set).
6. Token non-empty.
7. Confirmation text equals `RESTORE BACKUP` exactly.

When all seven pass, the command returns `readyButDisabled` with `restoreWriteEngineNotEnabled`. No Airtable API call is made. `noChangesMade` is always `true`.

### Package Inspection

Package inspection (`inspect_backup_package`) is fully read-only. No token is required. The package is opened in memory, validated, and closed. No entries are extracted to disk. The result contains only the filename — the full path is stripped using `Path::file_name()` before the result is returned.

### Restore Dry-Run

Restore dry-run planning (`create_restore_dry_run_plan`) reads the inspected package, applies the compatibility engine, and returns a plan preview. No Airtable API calls are made. No token is required. The full path is not included in the result.

---

## Related Documents

- [Tauri Command Inventory](../architecture/tauri-command-inventory.md)
- [Known Limitations](../release/known-limitations.md)
- [v0.1.0-alpha Readiness](../release/v0.1.0-alpha-readiness.md)
- [Restore Execution Command Contract](../architecture/restore-execution-command-contract.md)
- [Safe Backup Command Contract](../architecture/safe-backup-command-contract.md)
