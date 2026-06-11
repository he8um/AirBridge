# Airtable API Notes

This document summarizes Airtable API assumptions used by AirBridge. Validate these notes against Airtable's official documentation when implementing or updating API behavior.

## Official documentation references

- Airtable Web API getting started: https://support.airtable.com/docs/getting-started-with-airtables-web-api
- Airtable API call limits: https://support.airtable.com/docs/managing-api-call-limits-in-airtable
- Personal Access Tokens: https://support.airtable.com/docs/creating-personal-access-tokens
- Airtable Web API reference: https://airtable.com/developers/web/api

## Authentication

AirBridge v0.1 uses Airtable Personal Access Tokens.

Recommended scopes:

```text
data.records:read
data.records:write
schema.bases:read
schema.bases:write
```

## Create base

Airtable documents the create base endpoint as:

```text
POST https://api.airtable.com/v0/meta/bases
```

The endpoint creates a new base with provided tables and returns the schema for the newly created base. Restore should still handle permission failures because workspace permissions and token scope determine whether the request succeeds.

## List records

Record listing returns pages. The maximum page size is 100 records. AirBridge must follow offsets until no offset remains.

## Rate limits

Airtable enforces request limits. AirBridge must implement request queueing, per-base limiting, token-level limiting, retry behavior, and cooldown after 429 responses.

## Batch writes

Airtable supports batching for record operations. Restore should batch record creation and updates where possible to reduce request count while staying within API limits.

## Permission errors

Common restore failures may be caused by:

- Missing token scope.
- Token not granted access to the base or workspace.
- User lacking creator-level permission.
- Destination workspace restrictions.
- Unsupported field type creation.

## Implementation rule

Do not encode optimistic assumptions as silent behavior. If Airtable rejects a schema or record write, AirBridge should report the affected resource and suggested action.
