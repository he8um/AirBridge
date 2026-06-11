# Contributing to AirBridge

Thank you for considering a contribution to AirBridge. The project aims to be useful, safe, and maintainable. Contributions should improve the product without weakening backup integrity, restore safety, security, or documentation quality.

## Ways to contribute

You can contribute by:

- Reporting bugs.
- Requesting features.
- Improving documentation.
- Testing backup and restore workflows.
- Adding field compatibility support.
- Improving UI and accessibility.
- Improving reliability, logging, and validation.
- Fixing packaging or platform-specific issues.

## Before opening an issue

Please check:

1. Existing open issues.
2. The roadmap.
3. The restore limitations documentation.
4. The field compatibility documentation.
5. The troubleshooting guide.

If your issue involves a real Airtable base, do not upload sensitive data. Provide sanitized examples whenever possible.

## Before opening a pull request

A good pull request should:

- Have a clear purpose.
- Be scoped to one change or one related group of changes.
- Include tests where practical.
- Update documentation when behavior changes.
- Avoid unrelated refactors.
- Avoid committing secrets, tokens, logs with sensitive data, or real backup packages.
- Explain restore or backup safety implications when relevant.

## Development principles

AirBridge prioritizes:

- Safety over convenience.
- Clear restore reports over silent best guesses.
- Local-first behavior over remote dependencies.
- Explicit user consent for write operations.
- Maintainable architecture over short-term shortcuts.
- Clear documentation for limitations.

## Commit message style

Use short, descriptive commit messages:

```text
feat: add backup package manifest validation
fix: handle Airtable rate limit responses
docs: document linked record restore behavior
refactor: split restore planner from importer
test: add package checksum validation tests
```

## Pull request checklist

Before submitting a pull request, confirm:

- [ ] I tested the change locally.
- [ ] I updated documentation if behavior changed.
- [ ] I did not include secrets, tokens, private logs, or sensitive Airtable data.
- [ ] I linked the relevant issue where applicable.
- [ ] I described any backup, restore, or security implications.

## Review expectations

Maintainers may ask for:

- Smaller scope.
- Additional tests.
- Clearer error handling.
- Documentation updates.
- Safer restore behavior.
- Better user-facing warnings.

This is normal for a project that handles user data and write operations.
