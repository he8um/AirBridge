# Error Handling

AirBridge errors should be actionable and safe.

## Error shape

Every structured error should include:

```text
code
message
technical_detail
affected_resource
retryable
suggested_action
```

## Error categories

Suggested categories:

```text
AUTH_INVALID_TOKEN
AUTH_MISSING_SCOPE
PERMISSION_CREATE_BASE_DENIED
RATE_LIMITED
NETWORK_TIMEOUT
PACKAGE_INVALID_MANIFEST
PACKAGE_CHECKSUM_FAILED
SCHEMA_FIELD_UNSUPPORTED
RESTORE_RECORD_BATCH_FAILED
LINKED_RECORD_MAPPING_FAILED
```

## User-facing messages

Messages should be clear and specific.

Bad:

```text
Request failed.
```

Good:

```text
AirBridge cannot create the field "Campaign ROI" because this field type is not automatically restorable in v0.1. The field was added to the restore report as a manual action.
```

## Retryable vs non-retryable

Retryable:

- Rate limit.
- Temporary network timeout.
- Temporary server error.

Non-retryable:

- Invalid token.
- Missing scope.
- Unsupported field type.
- Invalid backup package.
- Destination base is not empty.

## Logging

Logs should include enough technical detail for debugging while excluding secrets and sensitive payloads.
