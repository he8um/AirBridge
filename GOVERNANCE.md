# Governance

AirBridge is maintained as an open-source project with a conservative approach to data safety and restore behavior.

## Maintainer responsibilities

Maintainers are responsible for:

- Reviewing issues and pull requests.
- Protecting user data safety.
- Maintaining restore limitations documentation.
- Managing releases.
- Keeping the roadmap realistic.
- Enforcing community standards.
- Rejecting changes that introduce unsafe restore behavior.

## Decision principles

Project decisions should prioritize:

1. Data safety.
2. Clear user consent for write operations.
3. Transparent reporting of unsupported behavior.
4. Maintainable architecture.
5. Cross-platform reliability.
6. Clear documentation.

## Compatibility policy

AirBridge should not silently claim unsupported restore fidelity. Field types and Airtable features must be documented as one of:

- Restorable.
- Partially restorable.
- Metadata-only.
- Unsupported for restore.
- Manual action required.

## Release policy

Pre-1.0 releases may change behavior and package format. Breaking changes must be documented in the changelog and release notes.

## Maintainer authority

Maintainers may close issues or reject pull requests that:

- Are outside the project scope.
- Introduce unsafe write behavior.
- Lack sufficient testing for backup or restore changes.
- Add unnecessary remote dependencies.
- Include sensitive data.
- Conflict with documented security or privacy goals.
