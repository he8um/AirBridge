/// Wraps a Personal Access Token for safe use in HTTP requests.
///
/// Debug is manually implemented to redact the value. The token is never
/// serialized or logged.
pub struct AirtableToken(String);

impl AirtableToken {
    pub fn new(raw: impl Into<String>) -> Self {
        AirtableToken(raw.into())
    }

    /// Returns the `Authorization` header value: `Bearer <token>`.
    /// Callers should use the returned string immediately and not store it.
    pub fn authorization_header_value(&self) -> String {
        format!("Bearer {}", self.0)
    }

    /// Returns the byte length of the token without exposing its value.
    pub fn len(&self) -> usize {
        self.0.len()
    }

    pub fn is_empty(&self) -> bool {
        self.0.is_empty()
    }
}

impl std::fmt::Debug for AirtableToken {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.debug_tuple("AirtableToken").field(&"[redacted]").finish()
    }
}

impl std::fmt::Display for AirtableToken {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "[redacted]")
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    const SENTINEL: &str = "pat_example_sentinel_token_0123456789";

    #[test]
    fn debug_does_not_expose_token() {
        let tok = AirtableToken::new(SENTINEL);
        let debug = format!("{tok:?}");
        assert!(!debug.contains(SENTINEL));
        assert!(debug.contains("[redacted]"));
    }

    #[test]
    fn display_does_not_expose_token() {
        let tok = AirtableToken::new(SENTINEL);
        let display = format!("{tok}");
        assert!(!display.contains(SENTINEL));
        assert_eq!(display, "[redacted]");
    }

    #[test]
    fn authorization_header_value_has_bearer_prefix() {
        let tok = AirtableToken::new(SENTINEL);
        let header = tok.authorization_header_value();
        assert!(header.starts_with("Bearer "));
        assert!(header.contains(SENTINEL));
    }

    #[test]
    fn len_returns_token_length_without_exposing_value() {
        let tok = AirtableToken::new(SENTINEL);
        assert_eq!(tok.len(), SENTINEL.len());
    }
}
