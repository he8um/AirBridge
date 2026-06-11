use std::path::Path;

/// Result of validating a proposed backup output path.
#[derive(Debug, Clone, PartialEq, Eq)]
pub enum OutputPathError {
    /// Path string was empty.
    Empty,
    /// Extension is not `.airbridge`.
    WrongExtension,
    /// The path points to an existing directory, not a file.
    IsDirectory,
    /// The parent directory does not exist.
    ParentNotFound,
    /// The path contains traversal components (`..`).
    TraversalDetected,
    /// The path contains a null byte.
    NullByte,
}

impl OutputPathError {
    pub fn code(&self) -> &'static str {
        match self {
            OutputPathError::Empty => "EMPTY_PATH",
            OutputPathError::WrongExtension => "WRONG_EXTENSION",
            OutputPathError::IsDirectory => "IS_DIRECTORY",
            OutputPathError::ParentNotFound => "PARENT_NOT_FOUND",
            OutputPathError::TraversalDetected => "TRAVERSAL_DETECTED",
            OutputPathError::NullByte => "NULL_BYTE",
        }
    }

    pub fn message(&self) -> &'static str {
        match self {
            OutputPathError::Empty => "output path must not be empty",
            OutputPathError::WrongExtension => "output path must have a .airbridge extension",
            OutputPathError::IsDirectory => "output path must be a file, not a directory",
            OutputPathError::ParentNotFound => "the output directory does not exist",
            OutputPathError::TraversalDetected => {
                "output path must not contain traversal components (..)"
            }
            OutputPathError::NullByte => "output path must not contain null bytes",
        }
    }
}

impl std::fmt::Display for OutputPathError {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "{}: {}", self.code(), self.message())
    }
}

/// Validate a proposed backup output path without touching the filesystem
/// (except to check whether the parent directory exists and whether the path
/// is itself an existing directory).
///
/// Does not create, open, or write any file.
pub fn validate_output_path(path_str: &str) -> Result<(), OutputPathError> {
    if path_str.is_empty() {
        return Err(OutputPathError::Empty);
    }

    if path_str.contains('\0') {
        return Err(OutputPathError::NullByte);
    }

    let path = Path::new(path_str);

    // Reject paths that contain `..` components.
    for component in path.components() {
        if component == std::path::Component::ParentDir {
            return Err(OutputPathError::TraversalDetected);
        }
    }

    // Must end in `.airbridge`.
    match path.extension() {
        Some(ext) if ext == "airbridge" => {}
        _ => return Err(OutputPathError::WrongExtension),
    }

    // Must not be an existing directory.
    if path.is_dir() {
        return Err(OutputPathError::IsDirectory);
    }

    // Parent directory must exist.
    match path.parent() {
        Some(parent) if !parent.as_os_str().is_empty() => {
            if !parent.exists() {
                return Err(OutputPathError::ParentNotFound);
            }
        }
        // No parent component means a bare filename with no directory — treat as
        // relative to the current directory, which always "exists".
        _ => {}
    }

    Ok(())
}

#[cfg(test)]
mod tests {
    use super::*;
    use tempfile::tempdir;

    // ── Extension validation ───────────────────────────────────────────────

    #[test]
    fn empty_path_rejected() {
        assert_eq!(validate_output_path(""), Err(OutputPathError::Empty));
    }

    #[test]
    fn wrong_extension_rejected() {
        assert_eq!(
            validate_output_path("/tmp/backup.zip"),
            Err(OutputPathError::WrongExtension)
        );
    }

    #[test]
    fn no_extension_rejected() {
        assert_eq!(
            validate_output_path("/tmp/backup"),
            Err(OutputPathError::WrongExtension)
        );
    }

    #[test]
    fn correct_extension_in_existing_dir_accepted() {
        let dir = tempdir().expect("tempdir");
        let path = dir.path().join("output.airbridge");
        let path_str = path.to_str().expect("path to str");
        assert!(validate_output_path(path_str).is_ok());
    }

    // ── Directory detection ────────────────────────────────────────────────

    #[test]
    fn existing_directory_rejected() {
        let dir = tempdir().expect("tempdir");
        // The tempdir itself is a directory — rejected even with the right name shape.
        let path_str = dir.path().to_str().expect("path to str");
        // Only produces IsDirectory if the path looks like .airbridge; use a subdir named like one.
        let subdir = dir.path().join("backup.airbridge");
        std::fs::create_dir(&subdir).expect("create subdir");
        let path_str = subdir.to_str().expect("path to str");
        assert_eq!(
            validate_output_path(path_str),
            Err(OutputPathError::IsDirectory)
        );
    }

    // ── Parent directory existence ─────────────────────────────────────────

    #[test]
    fn missing_parent_dir_rejected() {
        assert_eq!(
            validate_output_path("/nonexistent-dir-abc123/output.airbridge"),
            Err(OutputPathError::ParentNotFound)
        );
    }

    // ── Traversal ─────────────────────────────────────────────────────────

    #[test]
    fn traversal_component_rejected() {
        let dir = tempdir().expect("tempdir");
        let path = dir
            .path()
            .join("..")
            .join(dir.path().file_name().unwrap_or_default())
            .join("output.airbridge");
        let path_str = path.to_str().expect("path to str");
        assert_eq!(
            validate_output_path(path_str),
            Err(OutputPathError::TraversalDetected)
        );
    }

    #[test]
    fn double_dot_in_string_rejected() {
        assert_eq!(
            validate_output_path("../../output.airbridge"),
            Err(OutputPathError::TraversalDetected)
        );
    }

    // ── Null byte ─────────────────────────────────────────────────────────

    #[test]
    fn null_byte_in_path_rejected() {
        assert_eq!(
            validate_output_path("/tmp/out\0put.airbridge"),
            Err(OutputPathError::NullByte)
        );
    }

    // ── Error codes ───────────────────────────────────────────────────────

    #[test]
    fn error_code_empty() {
        assert_eq!(OutputPathError::Empty.code(), "EMPTY_PATH");
    }

    #[test]
    fn error_code_wrong_extension() {
        assert_eq!(OutputPathError::WrongExtension.code(), "WRONG_EXTENSION");
    }

    #[test]
    fn error_code_is_directory() {
        assert_eq!(OutputPathError::IsDirectory.code(), "IS_DIRECTORY");
    }

    #[test]
    fn error_code_parent_not_found() {
        assert_eq!(OutputPathError::ParentNotFound.code(), "PARENT_NOT_FOUND");
    }

    #[test]
    fn error_code_traversal() {
        assert_eq!(
            OutputPathError::TraversalDetected.code(),
            "TRAVERSAL_DETECTED"
        );
    }

    #[test]
    fn error_code_null_byte() {
        assert_eq!(OutputPathError::NullByte.code(), "NULL_BYTE");
    }

    // ── No filesystem side effects ─────────────────────────────────────────

    #[test]
    fn validation_does_not_create_files() {
        let dir = tempdir().expect("tempdir");
        let path = dir.path().join("output.airbridge");
        let path_str = path.to_str().expect("path to str");
        let _ = validate_output_path(path_str);
        assert!(!path.exists(), "validation must not create the output file");
    }
}
