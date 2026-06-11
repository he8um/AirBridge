use serde::{Deserialize, Serialize};

#[allow(dead_code)]
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
#[serde(rename_all = "camelCase")]
pub enum FieldRestoreSupport {
    Restorable,
    PartiallyRestorable,
    MetadataOnly,
    UnsupportedForRestore,
    ManualActionRequired,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct FieldCompatibilityRule {
    pub field_type: String,
    pub support: FieldRestoreSupport,
    pub note: String,
    pub backup_support: String,
}
