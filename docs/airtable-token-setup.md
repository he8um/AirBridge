# Airtable Token Setup

AirBridge v0.1 uses Airtable Personal Access Tokens.

## Why Personal Access Tokens

Personal Access Tokens allow users to choose scopes and resources. This is safer than broad legacy credentials because access can be limited to selected bases or workspaces.

## Minimum scopes for backup

For backup-only use:

```text
data.records:read
schema.bases:read
```

These allow AirBridge to read records and base schema.

## Required scopes for restore

For restore workflows:

```text
data.records:read
data.records:write
schema.bases:read
schema.bases:write
```

These allow AirBridge to read source information and create destination schema and records.

## Resource access

When creating the token, choose the Airtable bases or workspaces AirBridge should access.

For restore to a new base, the token owner must also have sufficient permission in the destination workspace.

## Permission requirements

AirBridge should check:

- Can read base schema.
- Can read records.
- Can write records.
- Can create or update schema.
- Can create a base in the destination workspace.

If a permission check fails, AirBridge should show a precise error and suggest the missing scope or workspace permission.

## Token handling in AirBridge

AirBridge should:

- Never write tokens into `.airbridge` backup packages.
- Never show tokens in logs.
- Store tokens in the operating system credential store where available.
- Support session-only token usage when secure storage is unavailable.

## Recommended token naming

Use a descriptive name such as:

```text
AirBridge Local Backup
```

## Token rotation

If a token is exposed, delete or regenerate it immediately in Airtable and update AirBridge with the new token.
