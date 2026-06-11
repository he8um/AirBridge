# Getting Started

This guide explains the first successful AirBridge workflow from connection to backup inspection.

## Prerequisites

You need:

- An Airtable account.
- Access to at least one Airtable base.
- An Airtable Personal Access Token with the required scopes.
- AirBridge installed for your operating system.

## Recommended first workflow

For your first run, use a small non-critical Airtable base. This helps verify connection, backup, inspection, and validation without handling sensitive production data.

## Step 1: Install AirBridge

Download the latest release for your operating system from GitHub Releases.

Expected release artifacts:

- macOS: `.dmg`
- Windows: `.msi` or `.exe`
- Linux: `.AppImage` or `.deb`

See `installation.md` for platform-specific guidance.

## Step 2: Create an Airtable token

Create a Personal Access Token with the scopes required for your workflow.

For backup only:

```text
data.records:read
schema.bases:read
```

For backup and restore:

```text
data.records:read
data.records:write
schema.bases:read
schema.bases:write
```

See `airtable-token-setup.md` for details.

## Step 3: Add a connection

Open AirBridge and navigate to **Connections**.

Enter your token and test the connection. AirBridge should verify:

- Token is accepted.
- Base schema can be read.
- Records can be read.
- Restore-related write permissions are available if needed.

## Step 4: Create a backup

Navigate to **Backups** and start a new backup.

Choose:

- Connection.
- Base.
- Backup scope.
- Redaction options.
- Output path.

Run the backup. AirBridge writes a `.airbridge` package to the selected location.

## Step 5: Inspect and validate

Open the generated backup package in AirBridge.

Review:

- Table count.
- Field count.
- Record count.
- Attachment metadata count.
- Linked-record relationships.
- Compatibility warnings.
- Checksum validation.

## Step 6: Restore only after review

Do not run restore until you have reviewed the restore plan. In v0.1, restore should target a new base or an empty existing base only.
