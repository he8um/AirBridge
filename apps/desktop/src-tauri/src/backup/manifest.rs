use serde::{Deserialize, Serialize};

use crate::backup::format::{FORMAT_NAME, FORMAT_VERSION};

/// Source metadata for the backup — identifies origin without embedding a token.
#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct ManifestSource {
    pub provider: String,
    pub base_id: String,
    pub base_name: String,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub workspace_id: Option<String>,
}

/// Record/field/attachment counts captured at backup time.
#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct ManifestContents {
    pub tables: usize,
    pub fields: usize,
    pub records: usize,
    pub linked_record_relationships: usize,
    pub attachments: usize,
}

/// Security and privacy metadata for the package.
#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct ManifestSecurity {
    pub contains_record_data: bool,
    pub contains_attachment_urls: bool,
    /// Always false in V0.1.
    pub encrypted: bool,
    /// List of field-category labels that were redacted, or empty.
    pub redactions_applied: Vec<String>,
}

/// Package identity metadata.
#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct ManifestPackage {
    pub generated_by_app: String,
    /// Unique identifier for this package instance (UUID v4).
    pub package_id: String,
}

/// Root manifest.json structure.
///
/// No tokens, no local filesystem paths, no real user credentials.
#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct PackageManifest {
    pub format: String,
    pub format_version: String,
    pub app_version: String,
    /// ISO-8601 timestamp string, e.g. "2026-06-11T00:00:00Z".
    pub created_at: String,
    pub source: ManifestSource,
    pub contents: ManifestContents,
    pub security: ManifestSecurity,
    pub package: ManifestPackage,
}

impl PackageManifest {
    /// Constructs a manifest pre-filled with format constants.
    /// Caller must supply `app_version`, `created_at`, `source`, `contents`,
    /// `security`, and `package`.
    pub fn new(
        app_version: impl Into<String>,
        created_at: impl Into<String>,
        source: ManifestSource,
        contents: ManifestContents,
        security: ManifestSecurity,
        package: ManifestPackage,
    ) -> Self {
        PackageManifest {
            format: FORMAT_NAME.to_string(),
            format_version: FORMAT_VERSION.to_string(),
            app_version: app_version.into(),
            created_at: created_at.into(),
            source,
            contents,
            security,
            package,
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    fn sample_manifest() -> PackageManifest {
        PackageManifest::new(
            "0.1.0",
            "2026-06-11T00:00:00Z",
            ManifestSource {
                provider: "airtable".to_string(),
                base_id: "appSynthetic0001".to_string(),
                base_name: "Synthetic Test Base".to_string(),
                workspace_id: Some("wspSynthetic0001".to_string()),
            },
            ManifestContents {
                tables: 2,
                fields: 8,
                records: 50,
                linked_record_relationships: 1,
                attachments: 3,
            },
            ManifestSecurity {
                contains_record_data: true,
                contains_attachment_urls: false,
                encrypted: false,
                redactions_applied: vec![],
            },
            ManifestPackage {
                generated_by_app: "airbridge".to_string(),
                package_id: "00000000-0000-0000-0000-000000000001".to_string(),
            },
        )
    }

    #[test]
    fn manifest_serializes_to_json() {
        let m = sample_manifest();
        let json = serde_json::to_string(&m).expect("serialize manifest");
        assert!(json.contains("\"format\""));
        assert!(json.contains("airbridge"));
        assert!(json.contains("0.1.0"));
    }

    #[test]
    fn manifest_deserializes_from_json() {
        let m = sample_manifest();
        let json = serde_json::to_string(&m).expect("serialize");
        let m2: PackageManifest = serde_json::from_str(&json).expect("deserialize");
        assert_eq!(m2.format, FORMAT_NAME);
        assert_eq!(m2.format_version, FORMAT_VERSION);
        assert_eq!(m2.source.base_id, "appSynthetic0001");
    }

    #[test]
    fn manifest_format_field_equals_constant() {
        let m = sample_manifest();
        assert_eq!(m.format, FORMAT_NAME);
    }

    #[test]
    fn manifest_format_version_equals_constant() {
        let m = sample_manifest();
        assert_eq!(m.format_version, FORMAT_VERSION);
    }

    #[test]
    fn manifest_encrypted_false_in_v01() {
        let m = sample_manifest();
        assert!(!m.security.encrypted);
    }

    #[test]
    fn manifest_does_not_contain_token_sentinel() {
        const SENTINEL: &str = "pat_manifest_test_sentinel_0123456789";
        let m = sample_manifest();
        let json = serde_json::to_string(&m).expect("serialize");
        assert!(!json.contains(SENTINEL));
    }

    #[test]
    fn manifest_redactions_can_be_empty() {
        let m = sample_manifest();
        let json = serde_json::to_string(&m).expect("serialize");
        let m2: PackageManifest = serde_json::from_str(&json).expect("deserialize");
        assert!(m2.security.redactions_applied.is_empty());
    }

    #[test]
    fn manifest_redactions_can_be_populated() {
        let mut m = sample_manifest();
        m.security.redactions_applied = vec!["emails".to_string(), "collaborators".to_string()];
        let json = serde_json::to_string(&m).expect("serialize");
        let m2: PackageManifest = serde_json::from_str(&json).expect("deserialize");
        assert_eq!(m2.security.redactions_applied.len(), 2);
    }

    #[test]
    fn manifest_roundtrip_preserves_all_counts() {
        let m = sample_manifest();
        let json = serde_json::to_string(&m).expect("serialize");
        let m2: PackageManifest = serde_json::from_str(&json).expect("deserialize");
        assert_eq!(m2.contents.tables, 2);
        assert_eq!(m2.contents.fields, 8);
        assert_eq!(m2.contents.records, 50);
    }

    #[test]
    fn manifest_workspace_id_is_optional() {
        let mut m = sample_manifest();
        m.source.workspace_id = None;
        let json = serde_json::to_string(&m).expect("serialize");
        assert!(!json.contains("workspaceId"));
    }

    #[test]
    fn manifest_source_does_not_include_token_field() {
        let m = sample_manifest();
        let json = serde_json::to_string(&m).expect("serialize");
        assert!(!json.contains("token"));
        assert!(!json.contains("apiKey"));
        assert!(!json.contains("pat"));
    }
}
