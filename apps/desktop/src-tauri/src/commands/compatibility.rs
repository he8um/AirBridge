use crate::errors::AirBridgeResult;
use crate::models::compatibility::{FieldCompatibilityRule, FieldRestoreSupport};

#[tauri::command]
pub fn list_compatibility_rules() -> AirBridgeResult<Vec<FieldCompatibilityRule>> {
    Ok(vec![
        FieldCompatibilityRule {
            field_type: "singleLineText".to_string(),
            support: FieldRestoreSupport::Restorable,
            backup_support: "full".to_string(),
            note: "Restored as plain text field.".to_string(),
        },
        FieldCompatibilityRule {
            field_type: "number".to_string(),
            support: FieldRestoreSupport::Restorable,
            backup_support: "full".to_string(),
            note: "Restored with original precision settings where supported.".to_string(),
        },
        FieldCompatibilityRule {
            field_type: "singleSelect".to_string(),
            support: FieldRestoreSupport::Restorable,
            backup_support: "full".to_string(),
            note: "Options are recreated in the target base.".to_string(),
        },
        FieldCompatibilityRule {
            field_type: "multipleRecordLinks".to_string(),
            support: FieldRestoreSupport::PartiallyRestorable,
            backup_support: "full".to_string(),
            note: "Link targets are remapped by record ID during restore. Unresolved links are skipped.".to_string(),
        },
        FieldCompatibilityRule {
            field_type: "formula".to_string(),
            support: FieldRestoreSupport::UnsupportedForRestore,
            backup_support: "metadata_only".to_string(),
            note: "Formula expressions are stored in the schema backup. Computed values are not restored; the field must be recreated manually.".to_string(),
        },
        FieldCompatibilityRule {
            field_type: "rollup".to_string(),
            support: FieldRestoreSupport::MetadataOnly,
            backup_support: "metadata_only".to_string(),
            note: "Rollup configuration is captured in schema. Values are not restored.".to_string(),
        },
        FieldCompatibilityRule {
            field_type: "createdTime".to_string(),
            support: FieldRestoreSupport::MetadataOnly,
            backup_support: "metadata_only".to_string(),
            note: "Original creation timestamps cannot be restored via the API.".to_string(),
        },
        FieldCompatibilityRule {
            field_type: "multipleAttachments".to_string(),
            support: FieldRestoreSupport::PartiallyRestorable,
            backup_support: "partial".to_string(),
            note: "Attachment metadata is backed up. File content is not re-uploaded; original attachment URLs are stored as reference only.".to_string(),
        },
    ])
}
