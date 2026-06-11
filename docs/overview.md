# Overview

AirBridge is a local-first desktop application for backing up, inspecting, validating, and restoring Airtable bases.

## Problem

Many teams use Airtable as an operational database for campaigns, workflows, assets, CRM-like processes, internal tools, content operations, project tracking, and no-code systems. These bases often become critical infrastructure, but teams may lack a practical local backup and restore process.

Common problems include:

- No easy local archive of a base.
- Limited visibility into what a backup contains.
- Unclear restore behavior.
- Hard-to-reproduce schema and linked-record structures.
- Manual export workflows that lose metadata.
- Risky restore attempts without dry-run reports.

## AirBridge approach

AirBridge focuses on a conservative, transparent workflow:

1. Connect to Airtable with a Personal Access Token.
2. Select a base.
3. Create a local `.airbridge` backup package.
4. Inspect and validate the package.
5. Generate a restore plan.
6. Restore into a new or empty destination base.
7. Produce a restore report with created, skipped, partial, and failed items.

## Key design principles

- Local-first by default.
- No telemetry in v0.1.
- No destructive restore operations in v0.1.
- Explicit dry-run before restore.
- Transparent limitation reporting.
- Documented backup package format.
- Clear field compatibility matrix.
- Safe handling of credentials and logs.

## Primary users

AirBridge is designed for:

- Operations teams using Airtable as a workflow database.
- No-code builders maintaining business-critical bases.
- Internal tools teams that need exportable archives.
- Consultants and agencies who manage Airtable systems.
- Developers who want inspectable local backup packages.

## Scope summary

AirBridge v0.1 includes backup, inspect, validate, dry-run, and restore to new or empty base.

AirBridge v0.1 excludes scheduled backup, cloud sync, merge restore, overwrite restore, full attachment restore, automation restore, interface restore, and permission cloning.
