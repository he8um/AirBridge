/// Returns the filename component of a path string (no directory).
/// Works for both Unix (`/a/b/c.airbridge`) and Windows (`C:\a\b\c.airbridge`) paths.
/// If no separator is found, returns the input unchanged.
pub fn redact_path_to_filename(path: &str) -> String {
    let last = path
        .rfind('/')
        .or_else(|| path.rfind('\\'))
        .map(|i| i + 1)
        .unwrap_or(0);
    path[last..].to_string()
}

/// Replaces token-like values (Airtable PAT pattern `pat…`) and bare Bearer
/// strings with `[redacted]` so they cannot surface in history summaries.
pub fn reject_or_redact_token_like_values(value: &str) -> String {
    let lower = value.to_lowercase();
    if lower.starts_with("pat") && value.len() > 12 && value.chars().all(|c| c.is_alphanumeric()) {
        return "[redacted]".to_string();
    }
    if lower.contains("bearer ") {
        return "[redacted]".to_string();
    }
    value.to_string()
}

/// Strips content that must not appear in history messages:
/// - full filesystem paths → filename only (last path component)
/// - attachment URLs → replaced with a safe placeholder
/// - token-like values → [redacted]
pub fn sanitize_history_message(message: &str) -> String {
    if message.contains("Bearer ") || message.contains("bearer ") {
        return "[message redacted — contained sensitive value]".to_string();
    }
    if message.starts_with("pat") && message.len() > 12 {
        return "[message redacted — contained sensitive value]".to_string();
    }
    // Redact attachment-style URLs (dl.airtable.com URLs)
    if message.contains("dl.airtable.com") || message.contains("v5.airtableusercontent.com") {
        return "[attachment URL redacted]".to_string();
    }
    // Redact full UNIX paths that look like user home dirs
    if message.contains("/Users/") || message.contains("/home/") {
        let last = message.rfind('/').map(|i| i + 1).unwrap_or(0);
        return message[last..].to_string();
    }
    // Redact full Windows paths
    if message.contains(":\\") {
        let last = message.rfind('\\').map(|i| i + 1).unwrap_or(0);
        return message[last..].to_string();
    }
    message.to_string()
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn unix_path_returns_filename() {
        let result = redact_path_to_filename("/Users/testuser/Documents/my-backup.airbridge");
        assert_eq!(result, "my-backup.airbridge");
    }

    #[test]
    fn windows_path_returns_filename() {
        let result = redact_path_to_filename("C:\\Users\\testuser\\Documents\\my-backup.airbridge");
        assert_eq!(result, "my-backup.airbridge");
    }

    #[test]
    fn filename_only_unchanged() {
        let result = redact_path_to_filename("my-backup.airbridge");
        assert_eq!(result, "my-backup.airbridge");
    }

    #[test]
    fn nested_unix_path_returns_filename() {
        let result = redact_path_to_filename("/home/runner/work/project/backup.airbridge");
        assert_eq!(result, "backup.airbridge");
    }

    #[test]
    fn token_like_value_redacted() {
        let result = reject_or_redact_token_like_values("patXXXXXXXXXXXXXXXXXXXXXXXX");
        assert_eq!(result, "[redacted]");
    }

    #[test]
    fn bearer_value_redacted() {
        let result = reject_or_redact_token_like_values("Bearer patXXXXXXXXXXXXXXXXXXXXXX");
        assert_eq!(result, "[redacted]");
    }

    #[test]
    fn normal_value_unchanged() {
        let result = reject_or_redact_token_like_values("hello world");
        assert_eq!(result, "hello world");
    }

    #[test]
    fn short_pat_prefix_unchanged() {
        let result = reject_or_redact_token_like_values("pat");
        assert_eq!(result, "pat");
    }

    #[test]
    fn message_with_unix_path_returns_filename() {
        let result =
            sanitize_history_message("Backup written to /Users/alice/docs/backup.airbridge");
        assert_eq!(result, "backup.airbridge");
    }

    #[test]
    fn message_with_windows_path_returns_filename() {
        let result = sanitize_history_message("Saved to C:\\Users\\bob\\backup.airbridge");
        assert_eq!(result, "backup.airbridge");
    }

    #[test]
    fn message_with_bearer_redacted() {
        let result = sanitize_history_message("Using Bearer patXXXXXXXXXXX");
        assert_eq!(result, "[message redacted — contained sensitive value]");
    }

    #[test]
    fn message_with_attachment_url_redacted() {
        let result = sanitize_history_message("Attachment at dl.airtable.com/abc123");
        assert_eq!(result, "[attachment URL redacted]");
    }

    #[test]
    fn safe_message_unchanged() {
        let result = sanitize_history_message("Backup plan generated for 3 tables.");
        assert_eq!(result, "Backup plan generated for 3 tables.");
    }
}
