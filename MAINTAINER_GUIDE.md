# Maintainer Guide

This guide describes routine maintainer responsibilities for AirBridge.

## Weekly maintenance checklist

- Review new issues.
- Label issues by type, area, priority, and status.
- Check for security-sensitive reports accidentally opened publicly.
- Review pull requests.
- Update roadmap status if scope changes.
- Keep field compatibility documentation aligned with implementation.
- Keep restore limitations visible and accurate.

## Issue triage

For each issue:

1. Confirm whether it is actionable.
2. Check whether it includes sensitive data.
3. Assign labels.
4. Ask for sanitized reproduction details if needed.
5. Decide whether it is accepted, blocked, out of scope, or needs more information.

## Pull request review

Review for:

- Correctness.
- Tests.
- Error handling.
- User-facing messaging.
- Documentation impact.
- Restore safety.
- Security and privacy impact.
- Cross-platform compatibility.

## Release preparation

Before a release:

- Confirm changelog updates.
- Run CI.
- Build release artifacts.
- Generate checksums.
- Test at least one backup and one validation workflow.
- Review restore limitation notes.
- Publish release notes with known limitations.

## Handling sensitive reports

If a user opens a public issue containing tokens or private data:

1. Hide or edit the content if possible.
2. Ask the user to rotate exposed tokens.
3. Move the discussion to a private security channel if needed.
4. Document the incident privately for release planning.
