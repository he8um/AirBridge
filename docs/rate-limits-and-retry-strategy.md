# Rate Limits and Retry Strategy

AirBridge must handle Airtable API rate limits from the first implementation.

## Requirements

The API client should include:

- Per-base request limiter.
- Token-level limiter.
- Retry for transient network errors.
- 429 handling.
- Exponential backoff with jitter.
- Batch record operations where possible.
- Progress reporting during cooldown.

## Request limits

Design targets:

```text
Per-base limiter: 5 requests per second
Token-level limiter: 50 requests per second
Batch record writes: up to API-supported batch size
```

## 429 handling

When Airtable returns HTTP 429:

1. Stop issuing requests for the affected limiter.
2. Show a rate-limit state in the job progress UI.
3. Wait for the documented cooldown period.
4. Retry the failed request if it is safe to retry.
5. Preserve job logs.

## Retryable errors

Retry may be appropriate for:

- 429 rate limit.
- Network timeout.
- Temporary DNS failure.
- 502, 503, or 504 responses.

Retry is usually not appropriate for:

- Invalid token.
- Missing scope.
- Permission denied.
- Unsupported field type.
- Invalid schema payload.
- Malformed backup package.

## Backoff policy

Suggested backoff:

```text
attempt 1: 1 second
attempt 2: 2 seconds
attempt 3: 4 seconds
attempt 4: 8 seconds
attempt 5: fail with actionable error
```

Add jitter to avoid repeated collisions.

## User experience

The UI should show:

- Current operation.
- Retry count.
- Cooldown reason.
- Whether the operation is safe to cancel.
- Last error message.

## Logging

Do not log tokens. Logs should include request type, resource ID, status code, retry count, and sanitized error body.
