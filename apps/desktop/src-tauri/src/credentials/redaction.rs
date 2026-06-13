/// Returns true if the value looks like an Airtable Personal Access Token
/// or another secret (long string with token-like prefix or length).
pub fn is_token_like(value: &str) -> bool {
    let trimmed = value.trim();
    // Airtable PATs start with "pat" and are typically 80+ chars
    if trimmed.starts_with("pat") && trimmed.len() >= 40 {
        return true;
    }
    // Generic long secrets (40+ chars, no whitespace)
    if trimmed.len() >= 40 && !trimmed.contains(' ') {
        return true;
    }
    false
}

/// Returns a redacted replacement for a token-like value.
pub fn redact_token(value: &str) -> String {
    if is_token_like(value) {
        "[redacted]".to_string()
    } else {
        value.to_string()
    }
}

/// Ensures no token-like substring appears in a message.
/// Replaces any token-like whitespace-free 40+ char segment with [redacted].
pub fn ensure_no_token_in_message(message: &str) -> String {
    let words: Vec<&str> = message.split_whitespace().collect();
    let cleaned: Vec<String> = words
        .into_iter()
        .map(|w| {
            if is_token_like(w) {
                "[redacted]".to_string()
            } else {
                w.to_string()
            }
        })
        .collect();
    cleaned.join(" ")
}

#[cfg(test)]
mod tests {
    use super::*;

    const AIRTABLE_PAT: &str = "pat_example_sentinel_0123456789abcdefghijklmnopqrstuvwxyz01234";
    const GENERIC_SECRET: &str = "generic_long_secret_abcdefghijklmnopqrstuvwxyz01234567890abcde";
    const SHORT_LABEL: &str = "Saved token";

    #[test]
    fn airtable_pat_is_detected_as_token_like() {
        assert!(is_token_like(AIRTABLE_PAT));
    }

    #[test]
    fn generic_long_secret_is_detected() {
        assert!(is_token_like(GENERIC_SECRET));
    }

    #[test]
    fn short_label_is_not_token_like() {
        assert!(!is_token_like(SHORT_LABEL));
    }

    #[test]
    fn empty_string_is_not_token_like() {
        assert!(!is_token_like(""));
    }

    #[test]
    fn redact_token_replaces_pat() {
        let result = redact_token(AIRTABLE_PAT);
        assert_eq!(result, "[redacted]");
        assert!(!result.contains(AIRTABLE_PAT));
    }

    #[test]
    fn redact_token_replaces_generic_secret() {
        let result = redact_token(GENERIC_SECRET);
        assert_eq!(result, "[redacted]");
    }

    #[test]
    fn redact_token_preserves_short_label() {
        let result = redact_token(SHORT_LABEL);
        assert_eq!(result, SHORT_LABEL);
    }

    #[test]
    fn ensure_no_token_in_message_cleans_embedded_token() {
        let message = format!("Error processing {AIRTABLE_PAT} in request");
        let cleaned = ensure_no_token_in_message(&message);
        assert!(!cleaned.contains(AIRTABLE_PAT));
        assert!(cleaned.contains("[redacted]"));
    }

    #[test]
    fn ensure_no_token_in_message_preserves_safe_message() {
        let message = "Keychain is not available";
        let cleaned = ensure_no_token_in_message(message);
        assert_eq!(cleaned, message);
    }

    #[test]
    fn serialization_of_redacted_result_has_no_sentinel() {
        let display = format!("Token: {}", redact_token(AIRTABLE_PAT));
        assert!(!display.contains(AIRTABLE_PAT));
        let json = serde_json::to_string(&display).expect("serialize");
        assert!(!json.contains(AIRTABLE_PAT));
    }
}
