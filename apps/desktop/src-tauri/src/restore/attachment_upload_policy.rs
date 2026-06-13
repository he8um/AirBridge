use crate::restore::write_gate::evaluate_write_gate;
use serde::{Deserialize, Serialize};

// ── Public enums ──────────────────────────────────────────────────────────────

/// The overall verdict for an attachment upload policy check.
///
/// Safety invariants:
/// - `Compliant` does NOT enable restore writes.
/// - `writesEnabled` is always false regardless of status.
/// - Attachment file bytes are never uploaded in any status.
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub enum AttachmentUploadPolicyStatus {
    /// All attachment intents are metadata-only; no upload or download requested.
    Compliant,
    /// One or more attachment intents could not be classified or involve download.
    Warning,
    /// One or more attachment intents request upload, which is not permitted.
    Blocked,
}

/// The result of a single attachment policy check.
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub enum AttachmentUploadPolicyCheckStatus {
    Passed,
    Warning,
    Failed,
}

/// The declared intent for handling a set of attachment fields.
///
/// Only `MetadataOnly` is permitted in the current version.
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub enum AttachmentUploadIntent {
    /// Preserve attachment metadata (filename, MIME type, size) only.
    /// File bytes are not downloaded or re-uploaded. This is the only permitted intent.
    MetadataOnly,
    /// Request to upload attachment bytes to Airtable. Blocked in this version.
    UploadRequested,
    /// Request to download attachment bytes from a URL. Warning or blocked per policy.
    DownloadRequested,
    /// Intent is not known or could not be determined.
    Unknown,
}

impl AttachmentUploadIntent {
    /// Returns `true` if this intent is unconditionally blocked.
    pub fn is_blocked(&self) -> bool {
        matches!(self, AttachmentUploadIntent::UploadRequested)
    }

    /// Returns `true` if this intent produces a warning.
    pub fn is_warning(&self) -> bool {
        matches!(
            self,
            AttachmentUploadIntent::DownloadRequested | AttachmentUploadIntent::Unknown
        )
    }

    /// Returns `true` if this intent is permitted with no caveats.
    pub fn is_permitted(&self) -> bool {
        matches!(self, AttachmentUploadIntent::MetadataOnly)
    }
}

// ── Request / result types ────────────────────────────────────────────────────

/// A single declared attachment field to be policy-checked.
///
/// Safety: no token field, no path field, no full URL field.
#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct DeclaredAttachmentField {
    pub field_name: String,
    pub table_name: String,
    pub intent: AttachmentUploadIntent,
}

/// Request for Gate 5 attachment upload policy verification.
///
/// Safety: no token field, no path field, no full attachment URL.
#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct AttachmentUploadPolicyRequest {
    /// All attachment fields from the record import plan.
    pub declared_attachment_fields: Vec<DeclaredAttachmentField>,
    /// Human-readable description of the restore target (optional, for messages).
    pub target_display_name: Option<String>,
}

/// One individual check in the policy result.
#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct AttachmentUploadPolicyCheck {
    pub check_id: String,
    pub label: String,
    pub status: AttachmentUploadPolicyCheckStatus,
    pub message: String,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub remediation: Option<String>,
}

/// Result from `verify_attachment_upload_policy`.
///
/// Safety invariants:
/// - No token field.
/// - No filesystem path field.
/// - No full attachment URL field.
/// - `writes_enabled` is always `false`.
/// - `no_changes_made` is always `true`.
/// - `network_writes_attempted` is always `false`.
/// - `Compliant` does NOT enable restore writes.
/// - Attachment file bytes are never uploaded.
#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct AttachmentUploadPolicyResult {
    pub status: AttachmentUploadPolicyStatus,
    pub checks: Vec<AttachmentUploadPolicyCheck>,
    pub message: String,
    pub blocked_field_names: Vec<String>,
    pub metadata_only_field_count: usize,
    pub no_changes_made: bool,
    pub network_writes_attempted: bool,
    pub writes_enabled: bool,
}

// ── Core logic ────────────────────────────────────────────────────────────────

/// Verifies that no attachment upload operations are requested.
///
/// Check IDs:
/// - AUP-01: Write gate is disabled.
/// - AUP-02: No upload-requested intents.
/// - AUP-03: No download-requested intents (warning only).
/// - AUP-04: All unknown intents identified (warning only).
/// - AUP-05: All declared fields are metadata-only or accounted for.
///
/// No Airtable API calls are made.
/// No token is accepted or returned.
/// No filesystem path is accepted or returned.
/// No full attachment URL is accepted or returned.
pub fn verify_attachment_upload_policy(
    request: &AttachmentUploadPolicyRequest,
) -> AttachmentUploadPolicyResult {
    let mut checks: Vec<AttachmentUploadPolicyCheck> = Vec::new();
    let mut blocked_field_names: Vec<String> = Vec::new();

    // AUP-01: write gate always disabled
    let gate = evaluate_write_gate();
    checks.push(AttachmentUploadPolicyCheck {
        check_id: "AUP-01".to_string(),
        label: "write-gate-disabled".to_string(),
        status: AttachmentUploadPolicyCheckStatus::Passed,
        message: gate.message.clone(),
        remediation: None,
    });

    // Classify intents
    let upload_fields: Vec<&DeclaredAttachmentField> = request
        .declared_attachment_fields
        .iter()
        .filter(|f| f.intent == AttachmentUploadIntent::UploadRequested)
        .collect();

    let download_fields: Vec<&DeclaredAttachmentField> = request
        .declared_attachment_fields
        .iter()
        .filter(|f| f.intent == AttachmentUploadIntent::DownloadRequested)
        .collect();

    let unknown_fields: Vec<&DeclaredAttachmentField> = request
        .declared_attachment_fields
        .iter()
        .filter(|f| f.intent == AttachmentUploadIntent::Unknown)
        .collect();

    let metadata_only_count = request
        .declared_attachment_fields
        .iter()
        .filter(|f| f.intent == AttachmentUploadIntent::MetadataOnly)
        .count();

    for f in &upload_fields {
        blocked_field_names.push(format!("{}.{}", f.table_name, f.field_name));
    }

    // AUP-02: no upload-requested intents
    if !upload_fields.is_empty() {
        let names: Vec<String> = upload_fields
            .iter()
            .map(|f| format!("{}.{}", f.table_name, f.field_name))
            .collect();
        checks.push(AttachmentUploadPolicyCheck {
            check_id: "AUP-02".to_string(),
            label: "no-upload-requested".to_string(),
            status: AttachmentUploadPolicyCheckStatus::Failed,
            message: format!(
                "Attachment upload is not permitted in this version: {}.",
                names.join(", ")
            ),
            remediation: Some(
                "Change all attachment fields to MetadataOnly intent. File bytes cannot be uploaded during restore.".to_string(),
            ),
        });
    } else {
        checks.push(AttachmentUploadPolicyCheck {
            check_id: "AUP-02".to_string(),
            label: "no-upload-requested".to_string(),
            status: AttachmentUploadPolicyCheckStatus::Passed,
            message: "No attachment upload operations requested.".to_string(),
            remediation: None,
        });
    }

    // AUP-03: no download-requested intents (warning — download is deferred, not blocked)
    if !download_fields.is_empty() {
        let names: Vec<String> = download_fields
            .iter()
            .map(|f| format!("{}.{}", f.table_name, f.field_name))
            .collect();
        checks.push(AttachmentUploadPolicyCheck {
            check_id: "AUP-03".to_string(),
            label: "no-download-requested".to_string(),
            status: AttachmentUploadPolicyCheckStatus::Warning,
            message: format!(
                "Attachment download is deferred — file bytes will not be fetched: {}.",
                names.join(", ")
            ),
            remediation: Some(
                "Attachment download is not implemented in this version. Only metadata is preserved.".to_string(),
            ),
        });
    } else {
        checks.push(AttachmentUploadPolicyCheck {
            check_id: "AUP-03".to_string(),
            label: "no-download-requested".to_string(),
            status: AttachmentUploadPolicyCheckStatus::Passed,
            message: "No attachment download operations requested.".to_string(),
            remediation: None,
        });
    }

    // AUP-04: no unknown intents (warning)
    if !unknown_fields.is_empty() {
        let names: Vec<String> = unknown_fields
            .iter()
            .map(|f| format!("{}.{}", f.table_name, f.field_name))
            .collect();
        checks.push(AttachmentUploadPolicyCheck {
            check_id: "AUP-04".to_string(),
            label: "no-unknown-intents".to_string(),
            status: AttachmentUploadPolicyCheckStatus::Warning,
            message: format!(
                "Some attachment fields have unknown intent: {}.",
                names.join(", ")
            ),
            remediation: Some(
                "Classify all attachment fields before enabling live writes.".to_string(),
            ),
        });
    } else {
        checks.push(AttachmentUploadPolicyCheck {
            check_id: "AUP-04".to_string(),
            label: "no-unknown-intents".to_string(),
            status: AttachmentUploadPolicyCheckStatus::Passed,
            message: "All declared attachment fields have a known intent.".to_string(),
            remediation: None,
        });
    }

    // AUP-05: metadata-only confirmation
    let all_metadata_only =
        upload_fields.is_empty() && download_fields.is_empty() && unknown_fields.is_empty();

    if all_metadata_only {
        checks.push(AttachmentUploadPolicyCheck {
            check_id: "AUP-05".to_string(),
            label: "metadata-only-confirmed".to_string(),
            status: AttachmentUploadPolicyCheckStatus::Passed,
            message: format!(
                "All {} declared attachment field(s) use metadata-only handling. \
                File bytes are not uploaded or downloaded.",
                metadata_only_count
            ),
            remediation: None,
        });
    } else {
        let total_non_metadata = upload_fields.len() + download_fields.len() + unknown_fields.len();
        checks.push(AttachmentUploadPolicyCheck {
            check_id: "AUP-05".to_string(),
            label: "metadata-only-confirmed".to_string(),
            status: if !upload_fields.is_empty() {
                AttachmentUploadPolicyCheckStatus::Failed
            } else {
                AttachmentUploadPolicyCheckStatus::Warning
            },
            message: format!(
                "{} attachment field(s) are not confirmed as metadata-only.",
                total_non_metadata
            ),
            remediation: Some(
                "Set all attachment fields to MetadataOnly intent before enabling live writes."
                    .to_string(),
            ),
        });
    }

    // Determine overall status
    let has_blocked = !upload_fields.is_empty();
    let has_warning = !download_fields.is_empty() || !unknown_fields.is_empty();

    let status = if has_blocked {
        AttachmentUploadPolicyStatus::Blocked
    } else if has_warning {
        AttachmentUploadPolicyStatus::Warning
    } else {
        AttachmentUploadPolicyStatus::Compliant
    };

    let target_label = request
        .target_display_name
        .as_deref()
        .unwrap_or("the target base");

    let message = match status {
        AttachmentUploadPolicyStatus::Compliant => format!(
            "All {} declared attachment field(s) for {} use metadata-only handling. \
            Attachment file bytes are not uploaded or downloaded. Restore writes remain disabled.",
            metadata_only_count, target_label
        ),
        AttachmentUploadPolicyStatus::Warning => format!(
            "Some attachment fields for {} have deferred or unknown intent. \
            Attachment file bytes will not be uploaded or downloaded in this version.",
            target_label
        ),
        AttachmentUploadPolicyStatus::Blocked => format!(
            "Attachment upload is not permitted for {}: {}. \
            Change all attachment fields to MetadataOnly intent.",
            target_label,
            blocked_field_names.join(", ")
        ),
    };

    AttachmentUploadPolicyResult {
        status,
        checks,
        message,
        blocked_field_names,
        metadata_only_field_count: metadata_only_count,
        no_changes_made: true,
        network_writes_attempted: false,
        writes_enabled: false,
    }
}

// ── Tests ─────────────────────────────────────────────────────────────────────

#[cfg(test)]
mod tests {
    use super::*;

    fn field(table: &str, name: &str, intent: AttachmentUploadIntent) -> DeclaredAttachmentField {
        DeclaredAttachmentField {
            field_name: name.to_string(),
            table_name: table.to_string(),
            intent,
        }
    }

    fn req(
        fields: Vec<DeclaredAttachmentField>,
        name: Option<&str>,
    ) -> AttachmentUploadPolicyRequest {
        AttachmentUploadPolicyRequest {
            declared_attachment_fields: fields,
            target_display_name: name.map(|s| s.to_string()),
        }
    }

    fn metadata_fields() -> Vec<DeclaredAttachmentField> {
        vec![
            field("Projects", "Files", AttachmentUploadIntent::MetadataOnly),
            field("Tasks", "Attachments", AttachmentUploadIntent::MetadataOnly),
        ]
    }

    // ── Status paths ──────────────────────────────────────────────────────────

    #[test]
    fn metadata_only_is_compliant() {
        let result = verify_attachment_upload_policy(&req(metadata_fields(), Some("My Base")));
        assert_eq!(result.status, AttachmentUploadPolicyStatus::Compliant);
    }

    #[test]
    fn empty_fields_list_is_compliant() {
        let result = verify_attachment_upload_policy(&req(vec![], Some("My Base")));
        assert_eq!(result.status, AttachmentUploadPolicyStatus::Compliant);
    }

    #[test]
    fn upload_requested_is_blocked() {
        let result = verify_attachment_upload_policy(&req(
            vec![field(
                "Projects",
                "Files",
                AttachmentUploadIntent::UploadRequested,
            )],
            Some("My Base"),
        ));
        assert_eq!(result.status, AttachmentUploadPolicyStatus::Blocked);
    }

    #[test]
    fn download_requested_is_warning() {
        let result = verify_attachment_upload_policy(&req(
            vec![field(
                "Projects",
                "Files",
                AttachmentUploadIntent::DownloadRequested,
            )],
            Some("My Base"),
        ));
        assert_eq!(result.status, AttachmentUploadPolicyStatus::Warning);
    }

    #[test]
    fn unknown_intent_is_warning() {
        let result = verify_attachment_upload_policy(&req(
            vec![field("Projects", "Files", AttachmentUploadIntent::Unknown)],
            Some("My Base"),
        ));
        assert_eq!(result.status, AttachmentUploadPolicyStatus::Warning);
    }

    #[test]
    fn upload_with_metadata_is_blocked() {
        let mut fields = metadata_fields();
        fields.push(field(
            "Docs",
            "Scans",
            AttachmentUploadIntent::UploadRequested,
        ));
        let result = verify_attachment_upload_policy(&req(fields, Some("My Base")));
        assert_eq!(result.status, AttachmentUploadPolicyStatus::Blocked);
    }

    // ── Check IDs ─────────────────────────────────────────────────────────────

    #[test]
    fn all_five_check_ids_present_for_compliant() {
        let result = verify_attachment_upload_policy(&req(metadata_fields(), Some("My Base")));
        let ids: Vec<&str> = result.checks.iter().map(|c| c.check_id.as_str()).collect();
        assert!(ids.contains(&"AUP-01"));
        assert!(ids.contains(&"AUP-02"));
        assert!(ids.contains(&"AUP-03"));
        assert!(ids.contains(&"AUP-04"));
        assert!(ids.contains(&"AUP-05"));
    }

    #[test]
    fn aup_01_always_passes() {
        for fields in [vec![], metadata_fields()] {
            let result = verify_attachment_upload_policy(&req(fields, None));
            let aup01 = result
                .checks
                .iter()
                .find(|c| c.check_id == "AUP-01")
                .unwrap();
            assert_eq!(aup01.status, AttachmentUploadPolicyCheckStatus::Passed);
        }
    }

    #[test]
    fn aup_02_fails_on_upload_requested() {
        let result = verify_attachment_upload_policy(&req(
            vec![field("T", "F", AttachmentUploadIntent::UploadRequested)],
            None,
        ));
        let aup02 = result
            .checks
            .iter()
            .find(|c| c.check_id == "AUP-02")
            .unwrap();
        assert_eq!(aup02.status, AttachmentUploadPolicyCheckStatus::Failed);
    }

    #[test]
    fn aup_02_passes_on_metadata_only() {
        let result = verify_attachment_upload_policy(&req(metadata_fields(), None));
        let aup02 = result
            .checks
            .iter()
            .find(|c| c.check_id == "AUP-02")
            .unwrap();
        assert_eq!(aup02.status, AttachmentUploadPolicyCheckStatus::Passed);
    }

    #[test]
    fn aup_03_warns_on_download_requested() {
        let result = verify_attachment_upload_policy(&req(
            vec![field("T", "F", AttachmentUploadIntent::DownloadRequested)],
            None,
        ));
        let aup03 = result
            .checks
            .iter()
            .find(|c| c.check_id == "AUP-03")
            .unwrap();
        assert_eq!(aup03.status, AttachmentUploadPolicyCheckStatus::Warning);
    }

    #[test]
    fn aup_04_warns_on_unknown_intent() {
        let result = verify_attachment_upload_policy(&req(
            vec![field("T", "F", AttachmentUploadIntent::Unknown)],
            None,
        ));
        let aup04 = result
            .checks
            .iter()
            .find(|c| c.check_id == "AUP-04")
            .unwrap();
        assert_eq!(aup04.status, AttachmentUploadPolicyCheckStatus::Warning);
    }

    #[test]
    fn aup_05_passes_when_all_metadata_only() {
        let result = verify_attachment_upload_policy(&req(metadata_fields(), None));
        let aup05 = result
            .checks
            .iter()
            .find(|c| c.check_id == "AUP-05")
            .unwrap();
        assert_eq!(aup05.status, AttachmentUploadPolicyCheckStatus::Passed);
    }

    #[test]
    fn aup_05_fails_when_upload_requested() {
        let result = verify_attachment_upload_policy(&req(
            vec![field("T", "F", AttachmentUploadIntent::UploadRequested)],
            None,
        ));
        let aup05 = result
            .checks
            .iter()
            .find(|c| c.check_id == "AUP-05")
            .unwrap();
        assert_eq!(aup05.status, AttachmentUploadPolicyCheckStatus::Failed);
    }

    // ── blocked_field_names ───────────────────────────────────────────────────

    #[test]
    fn blocked_fields_empty_for_compliant() {
        let result = verify_attachment_upload_policy(&req(metadata_fields(), None));
        assert!(result.blocked_field_names.is_empty());
    }

    #[test]
    fn blocked_fields_contains_name_for_blocked() {
        let result = verify_attachment_upload_policy(&req(
            vec![field(
                "Projects",
                "Files",
                AttachmentUploadIntent::UploadRequested,
            )],
            None,
        ));
        assert!(result
            .blocked_field_names
            .contains(&"Projects.Files".to_string()));
    }

    #[test]
    fn metadata_only_count_matches_fields() {
        let result = verify_attachment_upload_policy(&req(metadata_fields(), None));
        assert_eq!(result.metadata_only_field_count, 2);
    }

    #[test]
    fn metadata_only_count_zero_when_all_upload() {
        let result = verify_attachment_upload_policy(&req(
            vec![field("T", "F", AttachmentUploadIntent::UploadRequested)],
            None,
        ));
        assert_eq!(result.metadata_only_field_count, 0);
    }

    // ── Safety invariants ─────────────────────────────────────────────────────

    #[test]
    fn no_changes_made_always_true() {
        for fields in [
            vec![],
            metadata_fields(),
            vec![field("T", "F", AttachmentUploadIntent::UploadRequested)],
        ] {
            let result = verify_attachment_upload_policy(&req(fields, None));
            assert!(result.no_changes_made);
        }
    }

    #[test]
    fn writes_enabled_always_false() {
        for fields in [
            vec![],
            metadata_fields(),
            vec![field("T", "F", AttachmentUploadIntent::UploadRequested)],
        ] {
            let result = verify_attachment_upload_policy(&req(fields, None));
            assert!(!result.writes_enabled);
        }
    }

    #[test]
    fn network_writes_attempted_always_false() {
        for fields in [
            vec![],
            metadata_fields(),
            vec![field("T", "F", AttachmentUploadIntent::UploadRequested)],
        ] {
            let result = verify_attachment_upload_policy(&req(fields, None));
            assert!(!result.network_writes_attempted);
        }
    }

    #[test]
    fn result_serialization_has_no_token() {
        let result = verify_attachment_upload_policy(&req(metadata_fields(), None));
        let json = serde_json::to_string(&result).unwrap();
        assert!(!json.contains("token"));
        assert!(!json.contains("pat"));
        assert!(!json.contains("apiKey"));
    }

    #[test]
    fn result_serialization_has_no_full_path() {
        let result = verify_attachment_upload_policy(&req(metadata_fields(), None));
        let json = serde_json::to_string(&result).unwrap();
        assert!(!json.contains("/Users/"));
        assert!(!json.contains("/home/"));
        assert!(!json.contains(":\\\\"));
    }

    #[test]
    fn result_serialization_has_no_attachment_url() {
        let result = verify_attachment_upload_policy(&req(metadata_fields(), None));
        let json = serde_json::to_string(&result).unwrap();
        // Full attachment URLs must not appear in any result field
        assert!(!json.contains("https://dl.airtable.com"));
        assert!(!json.contains("https://v5.airtableusercontent.com"));
    }

    #[test]
    fn message_does_not_contain_token() {
        for fields in [
            metadata_fields(),
            vec![field("T", "F", AttachmentUploadIntent::UploadRequested)],
        ] {
            let result = verify_attachment_upload_policy(&req(fields, None));
            assert!(!result.message.contains("pat"));
            assert!(!result.message.contains("token"));
        }
    }

    #[test]
    fn no_write_calls_are_made() {
        // Structural: verify_attachment_upload_policy takes no token, no HTTP client.
        // Confirmed at compile time — this test documents the intent.
        let result = verify_attachment_upload_policy(&req(metadata_fields(), None));
        assert!(!result.network_writes_attempted);
        assert!(!result.writes_enabled);
        assert!(result.no_changes_made);
    }

    #[test]
    fn compliant_message_says_writes_remain_disabled() {
        let result = verify_attachment_upload_policy(&req(metadata_fields(), Some("My Base")));
        assert!(result.message.contains("disabled"));
    }

    #[test]
    fn blocked_message_names_blocked_field() {
        let result = verify_attachment_upload_policy(&req(
            vec![field(
                "Projects",
                "Files",
                AttachmentUploadIntent::UploadRequested,
            )],
            Some("My Base"),
        ));
        assert!(result.message.contains("Projects.Files"));
    }

    #[test]
    fn display_name_in_message() {
        let result = verify_attachment_upload_policy(&req(metadata_fields(), Some("Archive Base")));
        assert!(result.message.contains("Archive Base"));
    }

    #[test]
    fn message_non_empty_for_all_statuses() {
        for fields in [
            vec![],
            metadata_fields(),
            vec![field("T", "F", AttachmentUploadIntent::UploadRequested)],
            vec![field("T", "F", AttachmentUploadIntent::DownloadRequested)],
            vec![field("T", "F", AttachmentUploadIntent::Unknown)],
        ] {
            let result = verify_attachment_upload_policy(&req(fields, None));
            assert!(!result.message.is_empty());
        }
    }

    // ── Intent helpers ────────────────────────────────────────────────────────

    #[test]
    fn upload_requested_is_blocked_intent() {
        assert!(AttachmentUploadIntent::UploadRequested.is_blocked());
    }

    #[test]
    fn metadata_only_is_not_blocked() {
        assert!(!AttachmentUploadIntent::MetadataOnly.is_blocked());
    }

    #[test]
    fn download_requested_is_warning_intent() {
        assert!(AttachmentUploadIntent::DownloadRequested.is_warning());
    }

    #[test]
    fn unknown_is_warning_intent() {
        assert!(AttachmentUploadIntent::Unknown.is_warning());
    }

    #[test]
    fn metadata_only_is_permitted() {
        assert!(AttachmentUploadIntent::MetadataOnly.is_permitted());
    }

    #[test]
    fn upload_requested_is_not_permitted() {
        assert!(!AttachmentUploadIntent::UploadRequested.is_permitted());
    }
}
