# FAQ

## Is AirBridge affiliated with Airtable?

No. AirBridge is an independent open-source project.

## Does AirBridge create a perfect clone of an Airtable base?

No. AirBridge creates local backups and provides best-effort restore with transparent limitations.

## Can AirBridge restore automations?

No, not in v0.1.

## Can AirBridge restore interfaces?

No, not in v0.1.

## Can AirBridge restore attachments?

In v0.1, AirBridge backs up attachment metadata only. Full file restore is planned for later exploration.

## Can AirBridge restore into an existing base with data?

No, not in v0.1. Restore targets are new bases or empty existing bases.

## Why does AirBridge use JSONL for records?

JSONL is better for large backups because it can be streamed, validated line by line, and processed without loading all records into memory.

## Are backup files encrypted?

Not in v0.1. Optional encryption is planned after the backup format stabilizes.

## Does AirBridge send telemetry?

No telemetry is planned for v0.1.

## Where are tokens stored?

AirBridge should use operating system credential storage where available. If secure storage is unavailable, session-only usage should be supported.
