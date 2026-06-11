# Security and Privacy QA

AirBridge is designed to operate entirely on the local machine. No user data, API tokens, or record content is transmitted to any server outside the device. This document describes the verification steps to confirm that these properties hold in a release build.

---

## No Network Egress During Backup/Restore

**Goal:** Confirm that AirBridge does not make any outbound network connection to third-party servers (outside of the Airtable API, which is the intentional data source/destination).

### Verification Steps

- [ ] Install a local network monitor (e.g., Little Snitch on macOS, Windows Firewall logging, or `ss`/`tcpdump` on Linux).
- [ ] Launch AirBridge with network monitoring active.
- [ ] Perform a full backup of a test base.
- [ ] Perform a dry-run restore from the backup.
- [ ] Review all outbound connections made by the AirBridge process.
- [ ] **Expected:** The only outbound connections are to `api.airtable.com` (or Airtable's documented API hostname). No connections to analytics endpoints, crash-reporting services, update servers (during operation, as opposed to explicit update checks), or any other third-party hostname.
- [ ] **Expected:** When no operation is in progress (idle state), no periodic outbound connections are made.

---

## Token Storage Security

**Goal:** Confirm that API tokens are stored only in the local app data directory and are never transmitted, logged, or embedded in backup packages.

### Verification Steps

- [ ] Add a connection with a known test token.
- [ ] Locate the app data directory (see `cross-platform-qa.md` for paths).
- [ ] Inspect the stored configuration file. Verify:
  - [ ] The token is stored in a local file, not in a browser localStorage or a cloud-synced location.
  - [ ] The token is not stored in plaintext alongside other application logs. (If it is encrypted, document the encryption method in the technical documentation.)
- [ ] Perform a backup. After completion, open every file in the backup package.
  - [ ] The token must not appear in `manifest.json`, `schema.json`, `records.jsonl`, or any other file in the package.
- [ ] Review the application log file after a backup.
  - [ ] The token string must not appear anywhere in the log.
- [ ] Simulate a failed backup (revoke the token mid-backup). Review the error messages shown in the UI and written to the log.
  - [ ] The token must not appear in the error message text.

---

## No Telemetry or Analytics

**Goal:** Confirm that the production build contains no telemetry, analytics, crash-reporting, or user-tracking code.

### Verification Steps

- [ ] Inspect the compiled frontend bundle (`dist/` or equivalent) for known analytics domain strings: `analytics`, `telemetry`, `sentry`, `datadog`, `mixpanel`, `amplitude`, `segment`, `gtag`, `ga.js`.
  - [ ] None of these strings should appear as URL fragments in the bundle.
- [ ] Inspect `Cargo.lock` and `package.json` for known telemetry crate or package names.
- [ ] Run the application with network monitoring active through a complete backup-and-restore cycle.
  - [ ] No connections to domains other than `api.airtable.com` should be observed.
- [ ] Review `tauri.conf.json` (or equivalent) for any `allowlist` entries that permit connections to non-Airtable hosts.

---

## Log File Content

**Goal:** Confirm that the log file contains operational information only, with no tokens, record field values, or personally identifiable information.

### Verification Steps

- [ ] Perform a backup of a test base containing known field values (e.g., a record with `email` field `test-check@example.com`).
- [ ] Open the application log file.
- [ ] Search the log for the token string used — **must not be present**.
- [ ] Search the log for the test email address or other known record field values — **must not be present**.
- [ ] Confirm that the log contains useful operational entries: operation start/stop timestamps, table names, field names, record counts, and error messages.
- [ ] Confirm that the log does not contain full record payloads or raw API responses that include record field values.
- [ ] Verify that the log file's file permissions restrict read access to the current user (e.g., mode `0600` on macOS/Linux).

---

## File Permissions on Output Packages

- [ ] On macOS/Linux: backup package files are created with mode `0644` (owner read/write, group and others read-only) or more restrictive. The containing directory is `0755` or more restrictive.
- [ ] On Windows: backup files are created in the user's home or chosen directory with standard user-owned permissions. They are not world-readable to other Windows users on the same machine.
- [ ] The application does not write backup files to system-wide directories (e.g., `/tmp`, `/var`, `C:\Windows\Temp`) without user consent.

---

## Input Validation (Backup File Tampering)

**Goal:** Confirm that a maliciously crafted backup file cannot cause the application to behave unsafely.

### Verification Steps

- [ ] Open a backup package whose `manifest.json` contains a very long string value (>10,000 characters) in `baseName`. Verify: the application handles this gracefully (truncates the display, shows an error, or ignores the oversized value) rather than crashing or hanging.
- [ ] Open a backup package whose `records.jsonl` contains a line that is valid JSON but has unexpected top-level keys. Verify: the application parses what it recognizes and ignores unknown keys, without crashing.
- [ ] Open a backup package whose `schema.json` references a `linkedTableId` that does not exist in the same schema. Verify: the application reports a validation warning, not a panic.
- [ ] Provide a file that claims to be a backup package but is actually a binary file (e.g., a PNG renamed to `.airbridge`). Verify: the application detects the invalid format and shows a clear error message.

---

## Redaction Policy Enforcement

If AirBridge supports user-configured field redaction:

- [ ] Configure a field (e.g., an email field) as redacted before performing a backup.
- [ ] After the backup, inspect `records.jsonl` and confirm that the redacted field's values are replaced with `null` or a placeholder, not their actual values.
- [ ] Confirm that the redaction is noted in `manifest.json` (e.g., a `redactedFields` list).
- [ ] Confirm that running a restore from a redacted backup correctly restores `null` for the redacted field, and does not attempt to fill in the original values.

---

## No Credentials in Build Artifacts or Logs

- [ ] The release binary does not contain any hardcoded API keys, tokens, or credentials. Verify by running `strings airbridge-binary | grep -i "Bearer\|patPersonal\|apikey"` — no matches.
- [ ] `tauri.conf.json` and `package.json` in the release artifact contain no embedded secrets.
- [ ] The CI build logs (if publicly visible) do not print the signing certificate password, token values, or any other secrets. Confirm that all sensitive environment variables are masked.
- [ ] The application's built-in default configuration does not reference any specific Airtable base ID, token scope, or user account.

---

## Summary Verification Table

| Property | Verification method | Expected result |
|---------|---------------------|----------------|
| No egress to non-Airtable hosts | Network monitor during full backup+restore | Only `api.airtable.com` connections seen |
| Token not in backup package | grep token string across all package files | Zero matches |
| Token not in log | grep token string in log file | Zero matches |
| No telemetry code in bundle | grep known analytics domains in frontend bundle | Zero matches |
| Record values not in log | grep known field value in log file | Zero matches |
| Backup files have restricted permissions | `ls -l` on output files | `0644` or more restrictive |
| Malformed input does not crash | Open crafted malformed package | Error message shown, no crash |
