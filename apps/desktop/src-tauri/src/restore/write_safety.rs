use serde::{Deserialize, Serialize};

/// Safety invariants for the restore write engine.
///
/// All fields are computed from the skeleton; no Airtable calls are made.
#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct RestoreWriteSafetyReport {
    /// Always false — write engine is not enabled.
    pub writes_enabled: bool,
    /// Always false — no network write calls are attempted.
    pub network_writes_attempted: bool,
    /// Always false — no token is required for the skeleton preview.
    pub token_required: bool,
    /// Always true — no Airtable changes were made.
    pub no_changes_made: bool,
    /// Always false — restore execution cannot succeed in this version.
    pub restore_success_possible: bool,
    /// Reason that writes are gated.
    pub gated_reason: String,
}

/// Returns the safety report for the write engine skeleton.
///
/// All invariants are enforced structurally — there is no branch
/// that produces a report with writes_enabled or restore_success_possible true.
pub fn build_write_safety_report() -> RestoreWriteSafetyReport {
    RestoreWriteSafetyReport {
        writes_enabled: false,
        network_writes_attempted: false,
        token_required: false,
        no_changes_made: true,
        restore_success_possible: false,
        gated_reason: "Restore write engine is not enabled in this version. Schema creation and record import are available as planning-only operations.".to_string(),
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn safety_report_writes_enabled_is_false() {
        let report = build_write_safety_report();
        assert!(!report.writes_enabled);
    }

    #[test]
    fn safety_report_network_writes_attempted_is_false() {
        let report = build_write_safety_report();
        assert!(!report.network_writes_attempted);
    }

    #[test]
    fn safety_report_token_required_is_false() {
        let report = build_write_safety_report();
        assert!(!report.token_required);
    }

    #[test]
    fn safety_report_no_changes_made_is_true() {
        let report = build_write_safety_report();
        assert!(report.no_changes_made);
    }

    #[test]
    fn safety_report_restore_success_possible_is_false() {
        let report = build_write_safety_report();
        assert!(!report.restore_success_possible);
    }

    #[test]
    fn safety_report_serialization_has_no_token_field() {
        let report = build_write_safety_report();
        let json = serde_json::to_string(&report).expect("serialize");
        assert!(!json.contains("\"token\""));
        assert!(!json.contains("\"apiKey\""));
    }

    #[test]
    fn safety_report_serialization_has_no_path_sentinel() {
        let report = build_write_safety_report();
        let json = serde_json::to_string(&report).expect("serialize");
        assert!(!json.contains("/Users/"));
        assert!(!json.contains("/tmp/"));
        assert!(!json.contains("/home/"));
    }

    #[test]
    fn safety_report_has_no_succeeded_term() {
        let report = build_write_safety_report();
        let json = serde_json::to_string(&report).expect("serialize");
        assert!(!json.contains("\"succeeded\""));
        assert!(!json.contains("\"Succeeded\""));
        assert!(!json.contains("success\":true"));
    }
}
