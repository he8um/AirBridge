use crate::restore::write_result::{RestoreWriteDisabledReason, RestoreWriteEngineStatus};

/// The decision returned by the write gate.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct RestoreWriteGateDecision {
    pub status: RestoreWriteEngineStatus,
    pub reason: RestoreWriteDisabledReason,
    pub message: String,
}

/// Evaluates the write engine gate.
///
/// This function is the single source of truth for whether restore writes
/// are permitted. In this version, writes are always disabled by product policy.
///
/// There is no branch that returns an enabled decision.
/// No Airtable API calls are made.
/// No files are written.
/// No token is required.
pub fn evaluate_write_gate() -> RestoreWriteGateDecision {
    RestoreWriteGateDecision {
        status: RestoreWriteEngineStatus::Disabled,
        reason: RestoreWriteDisabledReason::DisabledByProductPolicy,
        message: "Restore write execution is not enabled in this version. Schema creation and record import are planning-only operations. No Airtable changes are made.".to_string(),
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn gate_always_returns_disabled_status() {
        let decision = evaluate_write_gate();
        assert_eq!(decision.status, RestoreWriteEngineStatus::Disabled);
    }

    #[test]
    fn gate_always_returns_product_policy_reason() {
        let decision = evaluate_write_gate();
        assert_eq!(
            decision.reason,
            RestoreWriteDisabledReason::DisabledByProductPolicy
        );
    }

    #[test]
    fn gate_message_is_non_empty() {
        let decision = evaluate_write_gate();
        assert!(!decision.message.is_empty());
    }

    #[test]
    fn gate_never_returns_not_started() {
        let decision = evaluate_write_gate();
        assert_ne!(decision.status, RestoreWriteEngineStatus::NotStarted);
    }

    #[test]
    fn gate_never_returns_blocked_by_missing_confirmation() {
        let decision = evaluate_write_gate();
        assert_ne!(
            decision.reason,
            RestoreWriteDisabledReason::BlockedByMissingConfirmation
        );
    }

    #[test]
    fn gate_message_does_not_contain_token() {
        let decision = evaluate_write_gate();
        assert!(!decision.message.contains("pat_"));
        assert!(!decision.message.contains("apiKey"));
    }

    #[test]
    fn gate_message_does_not_contain_absolute_path() {
        let decision = evaluate_write_gate();
        assert!(!decision.message.contains("/Users/"));
        assert!(!decision.message.contains("/tmp/"));
    }
}
