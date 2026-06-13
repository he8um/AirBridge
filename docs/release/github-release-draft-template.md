# GitHub Release Draft Template

Copy the block below into the GitHub Release body when creating the v0.1.0-alpha release.

---

```
## AirBridge v0.1.0-alpha

**This is an alpha release. It is intended for development review and early testing only. It is not production-ready and is not recommended for use with production Airtable bases.**

---

### What is included in v0.1.0-alpha

- **Base backup** — Connect to an Airtable base, inspect its schema and records, and write a local `.airbridge` backup package containing schema metadata, record data, and attachment metadata.
- **Backup inspection** — Review the structure and contents of a backup package without connecting to Airtable.
- **Restore planning** — Generate a full restore plan (schema creation plan and record import plan) from a backup package without writing to Airtable. The plan output documents what would be created.
- **Connection check** — Verify an Airtable personal access token and inspect available bases.
- **Activity history** — View a summary of recent operations on the Reports page. History is in-memory only and does not persist between sessions.

---

### What is not included in v0.1.0-alpha

- **Restore write execution is disabled.** Restore planning runs end-to-end but no data is written to Airtable. See the known limitations section.
- **Credential storage is not implemented.** Tokens must be entered for each operation.
- **Attachment file backup is not supported.** Attachment metadata is captured; file bytes are not downloaded.
- **macOS notarization and Windows code signing are not configured** for this release build.

---

### Known limitations

See [docs/release/known-limitations.md](docs/release/known-limitations.md) for the full list.

Key limitations:
- Restore write execution not yet enabled
- No credential storage between sessions
- Attachment files not downloaded or uploaded
- Job history does not persist between application restarts
- Computed field types (formula, rollup, lookup) must be recreated manually after restore
- macOS `.dmg` is not notarized; Windows `.msi` is not code-signed

---

### Platform artifacts

| Platform | Artifact                         | Notes                               |
| -------- | -------------------------------- | ----------------------------------- |
| macOS    | `airbridge-v0.1.0-alpha-macOS`   | Not notarized. Gatekeeper may warn. |
| Linux    | `airbridge-v0.1.0-alpha-Linux`   | AppImage and/or .deb                |
| Windows  | `airbridge-v0.1.0-alpha-Windows` | Not code-signed. SmartScreen may warn. |

---

### Installation notes

**macOS:** Mount the `.dmg` and drag AirBridge to Applications. If Gatekeeper blocks launch, right-click the application and choose Open.

**Linux:** Mark the `.AppImage` as executable (`chmod +x`) and run it directly. For the `.deb` package, install with `sudo dpkg -i <package>.deb`.

**Windows:** Run the `.msi` installer. If Windows SmartScreen shows a warning, click "More info" and then "Run anyway."

---

### Testing notes

<!-- Add testing notes and steps here before publishing the release. -->

---

### Checksums

<!-- Add SHA-256 checksums for each artifact here before publishing the release. -->
```
