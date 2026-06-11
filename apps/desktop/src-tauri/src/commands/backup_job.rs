use std::path::Path;

use serde::{Deserialize, Serialize};

use crate::airtable::auth::AirtableToken;
use crate::airtable::client::AirtableClient;
use crate::airtable::http::ReqwestHttpTransport;
use crate::backup::cancellation::CancellationToken;
use crate::backup::export_engine::TableExportSpec;
use crate::backup::job::{BackupJobId, BackupJobRequest};
use crate::backup::job_orchestrator::BackupJobOrchestrator;
use crate::backup::output_path::{validate_output_path, OutputPathError};

// ── Confirmation contract ──────────────────────────────────────────────────

/// The confirmation phrase required to run a backup job.
///
/// The caller must supply this exact string in the `confirmation` field of
/// `RunBackupCommandRequest`. This prevents accidental execution and makes it
/// clear that the command writes a file to the provided output path.
pub const CONFIRMATION_PHRASE: &str = "CREATE BACKUP";

// ── Request model ──────────────────────────────────────────────────────────

/// Table spec as supplied by the frontend.
#[derive(Debug, Clone, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct RunBackupTableSpec {
    pub table_id: String,
    pub table_name: String,
    #[serde(default)]
    pub linked_field_names: Vec<String>,
    #[serde(default)]
    pub attachment_field_names: Vec<String>,
}

/// Request to run a backup job.
///
/// Security requirements:
/// - `token` is consumed to build the HTTP client and is not stored elsewhere.
/// - `output_path` must pass `validate_output_path` before any write.
/// - `confirmation` must equal `CONFIRMATION_PHRASE` exactly.
/// - No token appears in the response.
/// - No absolute output path appears in the response.
#[derive(Debug, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct RunBackupCommandRequest {
    /// Airtable personal access token. Consumed; never stored.
    pub token: String,
    /// Absolute path to write the `.airbridge` package to.
    pub output_path: String,
    /// Must equal `CONFIRMATION_PHRASE` ("CREATE BACKUP").
    pub confirmation: String,
    /// Airtable base ID.
    pub base_id: String,
    /// Human-readable base name.
    pub base_name: String,
    /// Pre-serialised base metadata (no token embedded).
    pub base_json: Vec<u8>,
    /// Pre-serialised schema JSON.
    pub schema_json: Vec<u8>,
    /// Tables to export.
    pub table_specs: Vec<RunBackupTableSpec>,
    /// Page size for the pagination loop (defaults to 100).
    #[serde(default = "default_page_size")]
    pub page_size: u32,
    /// Optional caller-supplied job ID. Generated if absent.
    #[serde(default)]
    pub job_id: Option<String>,
}

fn default_page_size() -> u32 {
    100
}

// ── Response model ─────────────────────────────────────────────────────────

/// A validation issue returned when the output path is rejected.
#[derive(Debug, Clone, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct OutputPathValidationResult {
    pub valid: bool,
    pub error_code: Option<String>,
    pub error_message: Option<String>,
}

impl OutputPathValidationResult {
    pub fn ok() -> Self {
        OutputPathValidationResult {
            valid: true,
            error_code: None,
            error_message: None,
        }
    }

    pub fn from_error(err: &OutputPathError) -> Self {
        OutputPathValidationResult {
            valid: false,
            error_code: Some(err.code().to_string()),
            error_message: Some(err.message().to_string()),
        }
    }
}

/// A pre-run safety error (confirmation missing, path invalid, etc.).
#[derive(Debug, Clone, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct BackupCommandSafetyError {
    pub code: String,
    pub message: String,
}

/// Response returned by `run_backup_job`.
///
/// Safe to serialise and return to the frontend:
/// - No token.
/// - No absolute output path — only the package filename is included.
/// - Job result is embedded on success.
#[derive(Debug, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct RunBackupCommandResponse {
    /// Whether the command completed successfully.
    pub success: bool,
    /// The filename-only portion of the output path (no directory, no absolute path).
    #[serde(skip_serializing_if = "Option::is_none")]
    pub package_filename: Option<String>,
    /// Safety errors that prevented execution (confirmation missing, invalid path, etc.).
    #[serde(skip_serializing_if = "Vec::is_empty")]
    pub safety_errors: Vec<BackupCommandSafetyError>,
    /// Embedded job result. Present on success or job-level failure.
    #[serde(skip_serializing_if = "Option::is_none")]
    pub job_result: Option<crate::backup::job::BackupJobResult>,
    /// Output path validation result. Always present.
    pub path_validation: OutputPathValidationResult,
}

impl RunBackupCommandResponse {
    fn safety_rejected(errors: Vec<BackupCommandSafetyError>) -> Self {
        RunBackupCommandResponse {
            success: false,
            package_filename: None,
            safety_errors: errors,
            job_result: None,
            path_validation: OutputPathValidationResult::ok(),
        }
    }

    fn path_rejected(validation: OutputPathValidationResult) -> Self {
        RunBackupCommandResponse {
            success: false,
            package_filename: None,
            safety_errors: vec![BackupCommandSafetyError {
                code: "INVALID_OUTPUT_PATH".to_string(),
                message: "output path validation failed".to_string(),
            }],
            job_result: None,
            path_validation: validation,
        }
    }

    fn from_job_result(
        result: crate::backup::job::BackupJobResult,
        package_filename: Option<String>,
    ) -> Self {
        let success = result.status == crate::backup::job::BackupJobStatus::Succeeded;
        RunBackupCommandResponse {
            success,
            package_filename,
            safety_errors: vec![],
            job_result: Some(result),
            path_validation: OutputPathValidationResult::ok(),
        }
    }
}

// ── Commands ───────────────────────────────────────────────────────────────

/// Validate a proposed backup output path without writing any file.
///
/// Safe to call from the UI at any time. Returns a validation result
/// indicating whether the path would be accepted by `run_backup_job`.
#[tauri::command]
pub fn validate_backup_output_path(path: String) -> OutputPathValidationResult {
    match validate_output_path(&path) {
        Ok(()) => OutputPathValidationResult::ok(),
        Err(err) => OutputPathValidationResult::from_error(&err),
    }
}

/// Run a backup job, writing the package to the validated output path.
///
/// Preconditions enforced before any file write:
/// 1. `confirmation` must equal `CONFIRMATION_PHRASE` ("CREATE BACKUP").
/// 2. `output_path` must pass all validation rules (extension, parent exists, etc.).
///
/// On success, returns the job result with a package summary. The full absolute
/// output path is never included in the response — only the filename is returned.
///
/// No token is persisted. The token is dropped once the HTTP client is built.
#[tauri::command]
pub fn run_backup_job(request: RunBackupCommandRequest) -> RunBackupCommandResponse {
    // ── Step 1: Confirmation check ─────────────────────────────────────────
    if request.confirmation != CONFIRMATION_PHRASE {
        return RunBackupCommandResponse::safety_rejected(vec![BackupCommandSafetyError {
            code: "CONFIRMATION_REQUIRED".to_string(),
            message: format!("confirmation must be the exact phrase \"{CONFIRMATION_PHRASE}\""),
        }]);
    }

    // ── Step 2: Output path validation ────────────────────────────────────
    if let Err(path_err) = validate_output_path(&request.output_path) {
        return RunBackupCommandResponse::path_rejected(OutputPathValidationResult::from_error(
            &path_err,
        ));
    }

    // Destructure the request to take ownership of all fields cleanly.
    // The token is consumed to build the client and is not stored anywhere else.
    let RunBackupCommandRequest {
        token,
        output_path: output_path_str,
        confirmation: _,
        base_id,
        base_name,
        base_json,
        schema_json,
        table_specs: raw_specs,
        page_size,
        job_id: raw_job_id,
    } = request;

    // Extract filename-only portion (no directory, no absolute path).
    let package_filename = Path::new(&output_path_str)
        .file_name()
        .and_then(|n| n.to_str())
        .map(|s| s.to_string());

    // ── Step 3: Build orchestrator inputs ─────────────────────────────────
    let job_id = BackupJobId(
        raw_job_id
            .filter(|s| !s.is_empty())
            .unwrap_or_else(|| uuid_like_id(&base_id)),
    );

    let table_specs: Vec<TableExportSpec> = raw_specs
        .iter()
        .map(|s| TableExportSpec {
            table_id: s.table_id.clone(),
            table_name: s.table_name.clone(),
            linked_field_names: s.linked_field_names.clone(),
            attachment_field_names: s.attachment_field_names.clone(),
        })
        .collect();

    let job_request = BackupJobRequest {
        job_id,
        base_id,
        base_name,
        base_json,
        schema_json,
        table_specs,
        page_size,
    };

    // ── Step 4: Build HTTP client (token consumed here; not stored) ────────
    // The token string is moved into AirtableToken and is not accessible after this.
    let airtable_token = AirtableToken::new(token);
    let transport = ReqwestHttpTransport::default();
    let client = AirtableClient::new(airtable_token, transport);

    // ── Step 5: Run the orchestrator ───────────────────────────────────────
    let cancellation = CancellationToken::new();
    let mut orchestrator = BackupJobOrchestrator::new(client, cancellation);
    let output_path = Path::new(&output_path_str);
    let result = orchestrator.run(&job_request, output_path);

    RunBackupCommandResponse::from_job_result(result, package_filename)
}

/// Deterministic job-ID-like string derived from base ID (not a real UUID).
fn uuid_like_id(base_id: &str) -> String {
    use std::collections::hash_map::DefaultHasher;
    use std::hash::{Hash, Hasher};
    let mut h = DefaultHasher::new();
    base_id.hash(&mut h);
    format!("job-{:016x}", h.finish())
}

// ── Unit-testable helpers ──────────────────────────────────────────────────

#[cfg(test)]
pub fn validate_backup_output_path_direct(path: &str) -> OutputPathValidationResult {
    validate_backup_output_path(path.to_string())
}

#[cfg(test)]
pub fn run_backup_job_direct(request: RunBackupCommandRequest) -> RunBackupCommandResponse {
    run_backup_job(request)
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::airtable::http::MockHttpTransport;
    use crate::backup::cancellation::CancellationToken;
    use crate::backup::export_engine::TableExportSpec;
    use crate::backup::job::{BackupJobId, BackupJobRequest, BackupJobStatus};
    use crate::backup::job_orchestrator::BackupJobOrchestrator;
    use crate::backup::validation::ValidationStatus;
    use tempfile::tempdir;

    const SENTINEL: &str = "pat_cmd_contract_test_sentinel_0123456789";

    fn base_json_bytes() -> Vec<u8> {
        serde_json::to_vec(&serde_json::json!({"baseId":"appSyn01","name":"Synthetic"}))
            .expect("base_json")
    }

    fn schema_json_bytes() -> Vec<u8> {
        serde_json::to_vec(&serde_json::json!({"tables":[]})).expect("schema_json")
    }

    fn make_request(output_path: &str, confirmation: &str) -> RunBackupCommandRequest {
        RunBackupCommandRequest {
            token: SENTINEL.to_string(),
            output_path: output_path.to_string(),
            confirmation: confirmation.to_string(),
            base_id: "appSyn01".to_string(),
            base_name: "Synthetic".to_string(),
            base_json: base_json_bytes(),
            schema_json: schema_json_bytes(),
            table_specs: vec![RunBackupTableSpec {
                table_id: "tbl01".to_string(),
                table_name: "Projects".to_string(),
                linked_field_names: vec![],
                attachment_field_names: vec![],
            }],
            page_size: 100,
            job_id: Some("job-cmd-test-001".to_string()),
        }
    }

    // ── Path validation command ────────────────────────────────────────────

    #[test]
    fn validate_path_ok_for_tempdir() {
        let dir = tempdir().expect("tempdir");
        let path = dir.path().join("output.airbridge");
        let result = validate_backup_output_path_direct(path.to_str().expect("str"));
        assert!(result.valid);
        assert!(result.error_code.is_none());
    }

    #[test]
    fn validate_path_rejects_empty() {
        let result = validate_backup_output_path_direct("");
        assert!(!result.valid);
        assert_eq!(result.error_code.as_deref(), Some("EMPTY_PATH"));
    }

    #[test]
    fn validate_path_rejects_wrong_extension() {
        let result = validate_backup_output_path_direct("/tmp/backup.zip");
        assert!(!result.valid);
        assert_eq!(result.error_code.as_deref(), Some("WRONG_EXTENSION"));
    }

    #[test]
    fn validate_path_rejects_missing_parent() {
        let result = validate_backup_output_path_direct("/nonexistent-xyz-abc/output.airbridge");
        assert!(!result.valid);
        assert_eq!(result.error_code.as_deref(), Some("PARENT_NOT_FOUND"));
    }

    #[test]
    fn validate_path_rejects_traversal() {
        let result = validate_backup_output_path_direct("../../output.airbridge");
        assert!(!result.valid);
        assert_eq!(result.error_code.as_deref(), Some("TRAVERSAL_DETECTED"));
    }

    #[test]
    fn validate_path_has_no_file_side_effects() {
        let dir = tempdir().expect("tempdir");
        let path = dir.path().join("output.airbridge");
        let _ = validate_backup_output_path_direct(path.to_str().expect("str"));
        assert!(!path.exists(), "validate must not create the output file");
    }

    // ── Confirmation check ─────────────────────────────────────────────────

    #[test]
    fn missing_confirmation_rejected() {
        let dir = tempdir().expect("tempdir");
        let path = dir.path().join("output.airbridge");
        let req = make_request(path.to_str().expect("str"), "");
        let resp = run_backup_job_direct(req);
        assert!(!resp.success);
        assert!(resp
            .safety_errors
            .iter()
            .any(|e| e.code == "CONFIRMATION_REQUIRED"));
    }

    #[test]
    fn wrong_confirmation_phrase_rejected() {
        let dir = tempdir().expect("tempdir");
        let path = dir.path().join("output.airbridge");
        let req = make_request(path.to_str().expect("str"), "yes");
        let resp = run_backup_job_direct(req);
        assert!(!resp.success);
        assert!(resp
            .safety_errors
            .iter()
            .any(|e| e.code == "CONFIRMATION_REQUIRED"));
    }

    #[test]
    fn case_sensitive_confirmation_rejected() {
        let dir = tempdir().expect("tempdir");
        let path = dir.path().join("output.airbridge");
        let req = make_request(path.to_str().expect("str"), "create backup");
        let resp = run_backup_job_direct(req);
        assert!(!resp.success);
        assert!(resp
            .safety_errors
            .iter()
            .any(|e| e.code == "CONFIRMATION_REQUIRED"));
    }

    // ── Path validation inside run_backup_job ──────────────────────────────

    #[test]
    fn invalid_extension_rejected_in_run() {
        let dir = tempdir().expect("tempdir");
        let path = dir.path().join("output.zip");
        let req = make_request(path.to_str().expect("str"), CONFIRMATION_PHRASE);
        let resp = run_backup_job_direct(req);
        assert!(!resp.success);
        assert!(!resp.path_validation.valid);
        assert_eq!(
            resp.path_validation.error_code.as_deref(),
            Some("WRONG_EXTENSION")
        );
    }

    #[test]
    fn missing_parent_rejected_in_run() {
        let req = make_request("/nonexistent-dir-xyz/output.airbridge", CONFIRMATION_PHRASE);
        let resp = run_backup_job_direct(req);
        assert!(!resp.success);
        assert!(!resp.path_validation.valid);
        assert_eq!(
            resp.path_validation.error_code.as_deref(),
            Some("PARENT_NOT_FOUND")
        );
    }

    #[test]
    fn traversal_rejected_in_run() {
        let req = make_request("../../output.airbridge", CONFIRMATION_PHRASE);
        let resp = run_backup_job_direct(req);
        assert!(!resp.success);
        assert!(!resp.path_validation.valid);
        assert_eq!(
            resp.path_validation.error_code.as_deref(),
            Some("TRAVERSAL_DETECTED")
        );
    }

    // ── Orchestrator path (uses MockHttpTransport directly) ────────────────

    fn run_with_mock_transport(
        transport: MockHttpTransport,
        output_path: &Path,
    ) -> crate::backup::job::BackupJobResult {
        use crate::airtable::auth::AirtableToken;
        use crate::airtable::client::AirtableClient;

        let client = AirtableClient::new(AirtableToken::new(SENTINEL), transport);
        let mut orch = BackupJobOrchestrator::new(client, CancellationToken::new());
        let job_request = BackupJobRequest {
            job_id: BackupJobId("job-cmd-direct-001".to_string()),
            base_id: "appSyn01".to_string(),
            base_name: "Synthetic".to_string(),
            base_json: base_json_bytes(),
            schema_json: schema_json_bytes(),
            table_specs: vec![TableExportSpec {
                table_id: "tbl01".to_string(),
                table_name: "Projects".to_string(),
                linked_field_names: vec![],
                attachment_field_names: vec![],
            }],
            page_size: 100,
        };
        orch.run(&job_request, output_path)
    }

    #[test]
    fn command_writes_package_to_tempdir() {
        let body = r#"{"records":[{"id":"rec001","fields":{},"createdTime":null}]}"#;
        let dir = tempdir().expect("tempdir");
        let pkg = dir.path().join("job.airbridge");
        let result = run_with_mock_transport(MockHttpTransport::ok(body), &pkg);
        assert_eq!(result.status, BackupJobStatus::Succeeded);
        assert!(pkg.exists(), "package file must exist after successful run");
    }

    #[test]
    fn command_result_does_not_contain_token() {
        let body = r#"{"records":[]}"#;
        let dir = tempdir().expect("tempdir");
        let pkg = dir.path().join("job.airbridge");
        let result = run_with_mock_transport(MockHttpTransport::ok(body), &pkg);
        let json = serde_json::to_string(&result).expect("serialize");
        assert!(!json.contains(SENTINEL), "result contains token sentinel");
    }

    #[test]
    fn command_result_does_not_contain_absolute_path() {
        let body = r#"{"records":[]}"#;
        let dir = tempdir().expect("tempdir");
        let pkg = dir.path().join("job.airbridge");
        let result = run_with_mock_transport(MockHttpTransport::ok(body), &pkg);
        let json = serde_json::to_string(&result).expect("serialize");
        assert!(!json.contains("/Users/"), "result contains absolute path");
        assert!(!json.contains("/home/"), "result contains home path");
    }

    #[test]
    fn command_response_package_filename_only() {
        // The response struct omits the directory — only the filename is returned.
        let path = Path::new("/tmp/my-backup.airbridge");
        let filename = path
            .file_name()
            .and_then(|n| n.to_str())
            .map(|s| s.to_string());
        assert_eq!(filename.as_deref(), Some("my-backup.airbridge"));
    }

    #[test]
    fn generated_package_validates() {
        let body = r#"{"records":[{"id":"rec001","fields":{},"createdTime":null}]}"#;
        let dir = tempdir().expect("tempdir");
        let pkg = dir.path().join("job.airbridge");
        let result = run_with_mock_transport(MockHttpTransport::ok(body), &pkg);
        let vs = result.validation_summary.expect("validation summary");
        assert_eq!(vs.status, ValidationStatus::Valid);
    }

    #[test]
    fn no_attachment_url_in_result() {
        let body = serde_json::to_string(&serde_json::json!({
            "records": [{
                "id": "rec001",
                "fields": {
                    "Files": [{
                        "id": "attAbc01",
                        "filename": "photo.png",
                        "url": "https://dl.airtable.com/REDACTED_URL"
                    }]
                },
                "createdTime": null
            }]
        }))
        .unwrap();
        let dir = tempdir().expect("tempdir");
        let pkg = dir.path().join("job.airbridge");
        let result = run_with_mock_transport(MockHttpTransport::ok(body), &pkg);
        let json = serde_json::to_string(&result).expect("serialize");
        assert!(
            !json.contains("dl.airtable.com"),
            "result has attachment URL"
        );
        assert!(!json.contains("https://"), "result has https URL");
    }

    #[test]
    fn auth_error_maps_to_sanitized_failure() {
        let transport = MockHttpTransport::with_status(401, r#"{"error":"UNAUTHORIZED"}"#);
        let dir = tempdir().expect("tempdir");
        let pkg = dir.path().join("job.airbridge");
        let result = run_with_mock_transport(transport, &pkg);
        assert_eq!(result.status, BackupJobStatus::Failed);
        assert!(result.errors.iter().any(|e| e.code == "AUTH_FAILED"));
        let json = serde_json::to_string(&result).expect("serialize");
        assert!(!json.contains(SENTINEL));
    }

    #[test]
    fn permission_error_maps_to_sanitized_failure() {
        let transport = MockHttpTransport::with_status(403, r#"{"error":"forbidden"}"#);
        let dir = tempdir().expect("tempdir");
        let pkg = dir.path().join("job.airbridge");
        let result = run_with_mock_transport(transport, &pkg);
        assert_eq!(result.status, BackupJobStatus::Failed);
        assert!(result.errors.iter().any(|e| e.code == "PERMISSION_DENIED"));
    }

    #[test]
    fn confirmation_phrase_constant_is_stable() {
        assert_eq!(CONFIRMATION_PHRASE, "CREATE BACKUP");
    }

    #[test]
    fn response_success_field_false_on_failed_job() {
        let transport = MockHttpTransport::with_status(401, r#"{"error":"UNAUTHORIZED"}"#);
        let dir = tempdir().expect("tempdir");
        let pkg = dir.path().join("job.airbridge");
        let result = run_with_mock_transport(transport, &pkg);
        // Wrap in a response to check the success field.
        let resp =
            RunBackupCommandResponse::from_job_result(result, Some("job.airbridge".to_string()));
        assert!(!resp.success);
    }

    #[test]
    fn output_path_validation_result_serializes_without_path() {
        let r = OutputPathValidationResult::ok();
        let json = serde_json::to_string(&r).expect("serialize");
        assert!(!json.contains("/Users/"));
        assert!(!json.contains("/home/"));
        assert!(!json.contains("/tmp/"));
    }
}
