use std::time::Duration;

/// Per-base request limit: 5 requests per second.
pub const DEFAULT_PER_BASE_RPS: u32 = 5;

/// Token-level request limit: 50 requests per second across all bases.
pub const DEFAULT_TOKEN_RPS: u32 = 50;

/// Cooldown applied after receiving a 429 response: 30 seconds.
pub const DEFAULT_COOLDOWN_SECS: u64 = 30;

/// Maximum retry attempts following a 429.
pub const DEFAULT_MAX_RETRIES: u32 = 3;

/// Initial backoff before the first retry.
pub const DEFAULT_INITIAL_BACKOFF_SECS: u64 = 1;

/// Multiplier applied to backoff duration on each successive retry.
pub const DEFAULT_BACKOFF_MULTIPLIER: f64 = 2.0;

/// Policy model for Airtable API rate limiting and retry behaviour.
///
/// This is a configuration/model type only. Enforcement is the
/// responsibility of the HTTP transport layer.
#[derive(Debug, Clone)]
pub struct AirtableRateLimitPolicy {
    /// Maximum requests per second per base.
    pub per_base_requests_per_second: u32,
    /// Maximum requests per second for the token across all bases.
    pub token_requests_per_second: u32,
    /// How long to wait after a 429 before any retry.
    pub cooldown_after_429: Duration,
    /// Maximum number of retry attempts following a 429.
    pub max_retries: u32,
    /// Initial backoff before the first retry attempt.
    pub initial_backoff: Duration,
    /// Multiplier applied to backoff on each retry (exponential).
    pub backoff_multiplier: f64,
}

impl AirtableRateLimitPolicy {
    /// Returns the backoff duration for a given retry index (0-based).
    pub fn backoff_for_retry(&self, retry: u32) -> Duration {
        let secs = self.initial_backoff.as_secs_f64() * self.backoff_multiplier.powi(retry as i32);
        Duration::from_secs_f64(secs)
    }
}

impl Default for AirtableRateLimitPolicy {
    fn default() -> Self {
        AirtableRateLimitPolicy {
            per_base_requests_per_second: DEFAULT_PER_BASE_RPS,
            token_requests_per_second: DEFAULT_TOKEN_RPS,
            cooldown_after_429: Duration::from_secs(DEFAULT_COOLDOWN_SECS),
            max_retries: DEFAULT_MAX_RETRIES,
            initial_backoff: Duration::from_secs(DEFAULT_INITIAL_BACKOFF_SECS),
            backoff_multiplier: DEFAULT_BACKOFF_MULTIPLIER,
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn default_per_base_rps_is_five() {
        let policy = AirtableRateLimitPolicy::default();
        assert_eq!(policy.per_base_requests_per_second, 5);
    }

    #[test]
    fn default_token_rps_is_fifty() {
        let policy = AirtableRateLimitPolicy::default();
        assert_eq!(policy.token_requests_per_second, 50);
    }

    #[test]
    fn default_cooldown_is_thirty_seconds() {
        let policy = AirtableRateLimitPolicy::default();
        assert_eq!(policy.cooldown_after_429, Duration::from_secs(30));
    }

    #[test]
    fn default_max_retries_is_three() {
        let policy = AirtableRateLimitPolicy::default();
        assert_eq!(policy.max_retries, 3);
    }

    #[test]
    fn backoff_doubles_each_retry() {
        let policy = AirtableRateLimitPolicy::default();
        let b0 = policy.backoff_for_retry(0).as_secs_f64();
        let b1 = policy.backoff_for_retry(1).as_secs_f64();
        let b2 = policy.backoff_for_retry(2).as_secs_f64();
        assert!((b1 / b0 - 2.0).abs() < 1e-9);
        assert!((b2 / b1 - 2.0).abs() < 1e-9);
    }

    #[test]
    fn backoff_for_retry_zero_equals_initial() {
        let policy = AirtableRateLimitPolicy::default();
        assert_eq!(policy.backoff_for_retry(0), policy.initial_backoff,);
    }
}
