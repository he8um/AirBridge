# Field Compatibility

AirBridge uses a compatibility matrix to describe backup and restore behavior for Airtable field types.

## Compatibility statuses

| Status | Meaning |
| --- | --- |
| `restorable` | AirBridge can recreate the field and restore values automatically in v0.1 target scope. |
| `partially_restorable` | AirBridge can restore some configuration or values, but not full behavior. |
| `metadata_only` | AirBridge backs up metadata but does not recreate the field behavior automatically. |
| `unsupported_for_restore` | AirBridge backs up what it can but skips restore. |
| `manual_action_required` | The restore report tells the user what must be recreated manually. |

## Target v0.1 support

| Airtable field type | Backup | Restore target | Notes |
| --- | --- | --- | --- |
| Single line text | Yes | Restorable | Standard text values. |
| Long text | Yes | Restorable | Rich formatting may be limited depending on API representation. |
| Number | Yes | Restorable | Preserve numeric values and field options where possible. |
| Currency | Yes | Restorable | Preserve precision and symbol where supported. |
| Percent | Yes | Restorable | Preserve numeric values and formatting where supported. |
| Checkbox | Yes | Restorable | Boolean values. |
| Date | Yes | Restorable | Preserve date format where supported. |
| Date/time | Yes | Restorable | Preserve timezone-related options where supported. |
| Email | Yes | Restorable | Treated as string with email validation. |
| URL | Yes | Restorable | Treated as URL string. |
| Phone number | Yes | Restorable | Treated as phone string. |
| Single select | Yes | Restorable | Restore options before values. |
| Multiple select | Yes | Restorable | Restore options before values. |
| Rating | Yes | Restorable | Preserve max rating and icon where supported. |
| Duration | Yes | Restorable | Preserve duration format where supported. |
| Barcode | Yes | Restorable | Preserve barcode value shape where supported. |
| Linked records | Yes | Restorable in phases | Requires old-to-new record ID mapping. |
| Attachments | Metadata | Metadata-only in v0.1 | File restore is not guaranteed in v0.1. |
| Formula | Yes | Manual action | Formula behavior may require manual recreation. |
| Lookup | Yes | Partial/manual | Depends on linked fields and API support. |
| Rollup | Yes | Partial/manual | Depends on linked fields and API support. |
| Count | Yes | Partial/manual | Computed value cannot be preserved as source system value. |
| Created time | Yes | Metadata-only | Original system value cannot be preserved exactly. |
| Last modified time | Yes | Metadata-only | Original system value cannot be preserved exactly. |
| Created by | Yes | Metadata-only | Depends on user identity and workspace. |
| Last modified by | Yes | Metadata-only | Depends on user identity and workspace. |
| Autonumber | Yes | Manual action | Original sequence cannot be guaranteed. |
| Button | Metadata | Manual action | Button behavior may require manual configuration. |
| Collaborator | Yes | Partial/manual | Requires user mapping; v0.1 best effort only. |

## Reporting requirements

Every skipped or partial field must appear in the restore compatibility report.

Example:

```json
{
  "field": "Campaign ROI",
  "type": "formula",
  "status": "manual_action_required",
  "reason": "Formula fields are backed up but not guaranteed to be recreated automatically in v0.1."
}
```

## Policy

AirBridge must not silently drop fields. If a field is not restored, the user must see it in the report.
