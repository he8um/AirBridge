use std::path::Path;

use crate::airtable::client::AirtableClient;
use crate::airtable::http::HttpTransport;
use crate::backup::cancellation::CancellationToken;
use crate::backup::export_engine::{run_export, ExportEngineError};
use crate::backup::export_result::build_package_input;
use crate::backup::format::FORMAT_VERSION;
use crate::backup::job::{
    BackupJobError, BackupJobId, BackupJobPackageSummary, BackupJobPhase, BackupJobProgress,
    BackupJobRequest, BackupJobResult, BackupJobStatus, BackupJobTableResult, BackupJobWarning,
};
use crate::backup::job_events::BackupJobEvent;
use crate::backup::job_result::{
    build_cancelled_result, build_failed_result, build_succeeded_result,
    validation_summary_from_report,
};
use crate::backup::manifest::{
    ManifestContents, ManifestPackage, ManifestSecurity, ManifestSource, PackageManifest,
};
use crate::backup::reader::BackupPackageReader;
use crate::backup::validation::{validate_package, ValidationStatus};
use crate::backup::writer::write_package;

/// Orchestrates the full backup pipeline for a single run.
///
/// The orchestrator is synchronous and test-friendly — the caller supplies
/// the transport (mock or live), the output path (always a tempdir in tests),
/// and an optional cancellation token.
///
/// No token is stored in the orchestrator or emitted in results/events.
/// No absolute user paths appear in results or events.
pub struct BackupJobOrchestrator<T: HttpTransport> {
    client: AirtableClient<T>,
    cancellation: CancellationToken,
    /// Receives events as the job progresses (collected for tests, could be streamed).
    events: Vec<BackupJobEvent>,
}

impl<T: HttpTransport> BackupJobOrchestrator<T> {
    pub fn new(client: AirtableClient<T>, cancellation: CancellationToken) -> Self {
        BackupJobOrchestrator {
            client,
            cancellation,
            events: Vec::new(),
        }
    }

    /// Returns all emitted events after the job has run.
    pub fn events(&self) -> &[BackupJobEvent] {
        &self.events
    }

    fn emit(&mut self, event: BackupJobEvent) {
        self.events.push(event);
    }

    fn current_progress(phase: BackupJobPhase, message: &str) -> BackupJobProgress {
        BackupJobProgress {
            phase,
            message: message.to_string(),
            tables_completed: 0,
            total_tables: None,
        }
    }

    /// Run the full backup pipeline, writing the package to `output_path`.
    ///
    /// `output_path` must point into a temp directory in tests.
    /// The path is used locally only — it is never included in the returned result.
    pub fn run(&mut self, request: &BackupJobRequest, output_path: &Path) -> BackupJobResult {
        let job_id = request.job_id.clone();
        let base_id = request.base_id.as_str();
        let base_name = request.base_name.as_str();
        let mut warnings: Vec<BackupJobWarning> = Vec::new();

        // ── Planning phase ─────────────────────────────────────────────────
        self.emit(BackupJobEvent::JobStarted {
            job_id: job_id.clone(),
            base_id: base_id.to_string(),
            base_name: base_name.to_string(),
            table_count: request.table_specs.len(),
        });
        self.emit(BackupJobEvent::PhaseStarted {
            job_id: job_id.clone(),
            phase: BackupJobPhase::Planning,
        });

        if self.cancellation.is_cancelled() {
            self.emit(BackupJobEvent::JobCancelled {
                job_id: job_id.clone(),
                at_phase: BackupJobPhase::Planning,
            });
            return build_cancelled_result(job_id, base_id, base_name, warnings);
        }

        // ── Records export phase ───────────────────────────────────────────
        self.emit(BackupJobEvent::PhaseStarted {
            job_id: job_id.clone(),
            phase: BackupJobPhase::RecordsExport,
        });

        for spec in &request.table_specs {
            self.emit(BackupJobEvent::TableExportStarted {
                job_id: job_id.clone(),
                table_id: spec.table_id.clone(),
                table_name: spec.table_name.clone(),
            });
        }

        if self.cancellation.is_cancelled() {
            self.emit(BackupJobEvent::JobCancelled {
                job_id: job_id.clone(),
                at_phase: BackupJobPhase::RecordsExport,
            });
            return build_cancelled_result(job_id, base_id, base_name, warnings);
        }

        let export_result = match run_export(
            &self.client,
            base_id,
            base_name,
            &request.table_specs,
            request.page_size,
        ) {
            Ok(r) => r,
            Err(err) => {
                let (code, message, recoverable) = engine_error_to_job_error(&err);
                self.emit(BackupJobEvent::JobFailed {
                    job_id: job_id.clone(),
                    error_code: code.clone(),
                    message: message.clone(),
                });
                return build_failed_result(
                    job_id,
                    base_id,
                    base_name,
                    vec![BackupJobError {
                        code,
                        message,
                        recoverable,
                    }],
                    warnings,
                );
            }
        };

        for t in &export_result.tables {
            self.emit(BackupJobEvent::TableExportCompleted {
                job_id: job_id.clone(),
                table_id: t.table_id.clone(),
                table_name: t.table_name.clone(),
                record_count: t.record_count,
                pages_fetched: t.pages_fetched,
            });
        }

        let table_results: Vec<BackupJobTableResult> = export_result
            .tables
            .iter()
            .map(|t| BackupJobTableResult {
                table_id: t.table_id.clone(),
                table_name: t.table_name.clone(),
                record_count: t.record_count,
                pages_fetched: t.pages_fetched,
            })
            .collect();

        // ── Package build phase ────────────────────────────────────────────
        self.emit(BackupJobEvent::PhaseStarted {
            job_id: job_id.clone(),
            phase: BackupJobPhase::PackageBuild,
        });

        if self.cancellation.is_cancelled() {
            self.emit(BackupJobEvent::JobCancelled {
                job_id: job_id.clone(),
                at_phase: BackupJobPhase::PackageBuild,
            });
            return build_cancelled_result(job_id, base_id, base_name, warnings);
        }

        self.emit(BackupJobEvent::PackageWriteStarted {
            job_id: job_id.clone(),
        });

        let total_records = export_result.total_records();
        let table_count = export_result.tables.len();

        let manifest = PackageManifest::new(
            "0.1.0",
            "2026-06-11T00:00:00Z",
            ManifestSource {
                provider: "airtable".to_string(),
                base_id: base_id.to_string(),
                base_name: base_name.to_string(),
                workspace_id: None,
            },
            ManifestContents {
                tables: table_count,
                fields: 0,
                records: total_records,
                linked_record_relationships: 0,
                attachments: 0,
            },
            ManifestSecurity {
                contains_record_data: total_records > 0,
                contains_attachment_urls: false,
                encrypted: false,
                redactions_applied: vec![],
            },
            ManifestPackage {
                generated_by_app: "airbridge".to_string(),
                package_id: job_id.as_str().to_string(),
            },
        );

        let manifest_json = match serde_json::to_vec(&manifest) {
            Ok(b) => b,
            Err(e) => {
                let message = format!("manifest serialisation failed: {e}");
                self.emit(BackupJobEvent::JobFailed {
                    job_id: job_id.clone(),
                    error_code: "MANIFEST_SERIALISATION_ERROR".to_string(),
                    message: message.clone(),
                });
                return build_failed_result(
                    job_id,
                    base_id,
                    base_name,
                    vec![BackupJobError {
                        code: "MANIFEST_SERIALISATION_ERROR".to_string(),
                        message,
                        recoverable: false,
                    }],
                    warnings,
                );
            }
        };

        let backup_report_json =
            serde_json::to_vec(&serde_json::json!({"status":"ok","tableCount":table_count,"recordCount":total_records}))
                .unwrap_or_default();
        let compat_report_json =
            serde_json::to_vec(&serde_json::json!({"status":"ok"})).unwrap_or_default();

        let package_input = build_package_input(
            &export_result,
            manifest_json,
            request.base_json.clone(),
            request.schema_json.clone(),
            backup_report_json,
            compat_report_json,
        );

        if let Err(e) = write_package(output_path, &package_input) {
            let message = format!("package write failed: {e}");
            self.emit(BackupJobEvent::JobFailed {
                job_id: job_id.clone(),
                error_code: "PACKAGE_WRITE_ERROR".to_string(),
                message: message.clone(),
            });
            return build_failed_result(
                job_id,
                base_id,
                base_name,
                vec![BackupJobError {
                    code: "PACKAGE_WRITE_ERROR".to_string(),
                    message,
                    recoverable: false,
                }],
                warnings,
            );
        }

        // Count entries and checksums for the package summary
        let (entry_count, checksum_count) =
            count_package_entries_and_checksums(output_path).unwrap_or((0, 0));

        self.emit(BackupJobEvent::PackageWriteCompleted {
            job_id: job_id.clone(),
            entry_count,
        });

        // ── Validation phase ───────────────────────────────────────────────
        self.emit(BackupJobEvent::PhaseStarted {
            job_id: job_id.clone(),
            phase: BackupJobPhase::Validation,
        });
        self.emit(BackupJobEvent::ValidationStarted {
            job_id: job_id.clone(),
        });

        let validation_report = validate_package(output_path);

        let validation_status_str = match &validation_report.status {
            ValidationStatus::Valid => "valid",
            ValidationStatus::Warning => "warning",
            ValidationStatus::Invalid => "invalid",
        };

        self.emit(BackupJobEvent::ValidationCompleted {
            job_id: job_id.clone(),
            status: validation_status_str.to_string(),
            error_count: validation_report.errors.len(),
            warning_count: validation_report.warnings.len(),
        });

        if validation_report.status == ValidationStatus::Invalid {
            let error_codes: Vec<String> = validation_report
                .errors
                .iter()
                .map(|e| e.code.clone())
                .collect();
            let message = format!("package validation failed: {}", error_codes.join(", "));
            self.emit(BackupJobEvent::JobFailed {
                job_id: job_id.clone(),
                error_code: "VALIDATION_FAILED".to_string(),
                message: message.clone(),
            });
            return build_failed_result(
                job_id,
                base_id,
                base_name,
                vec![BackupJobError {
                    code: "VALIDATION_FAILED".to_string(),
                    message,
                    recoverable: false,
                }],
                warnings,
            );
        }

        if validation_report.status == ValidationStatus::Warning {
            for w in &validation_report.warnings {
                warnings.push(BackupJobWarning {
                    code: w.code.clone(),
                    message: w.message.clone(),
                    table_id: None,
                });
            }
        }

        let package_summary = BackupJobPackageSummary {
            package_id: job_id.as_str().to_string(),
            format_version: FORMAT_VERSION.to_string(),
            table_count,
            record_count: total_records,
            entry_count,
            checksum_count,
            encrypted: false,
            attachment_policy: "metadataOnly".to_string(),
        };

        let validation_summary = validation_summary_from_report(&validation_report);

        // ── Completed ──────────────────────────────────────────────────────
        self.emit(BackupJobEvent::PhaseStarted {
            job_id: job_id.clone(),
            phase: BackupJobPhase::Completed,
        });
        self.emit(BackupJobEvent::JobSucceeded {
            job_id: job_id.clone(),
            total_records,
            table_count,
        });

        build_succeeded_result(
            job_id,
            base_id,
            base_name,
            table_results,
            package_summary,
            validation_summary,
            warnings,
        )
    }
}

/// Map an `ExportEngineError` to a (code, sanitised_message, recoverable) triple.
fn engine_error_to_job_error(err: &ExportEngineError) -> (String, String, bool) {
    match err {
        ExportEngineError::InvalidToken => (
            "AUTH_FAILED".to_string(),
            "authentication failed: invalid or expired token".to_string(),
            false,
        ),
        ExportEngineError::RateLimited => (
            "RATE_LIMITED".to_string(),
            "request was rate limited by the API".to_string(),
            true,
        ),
        ExportEngineError::PermissionDenied => (
            "PERMISSION_DENIED".to_string(),
            "permission denied for base or table".to_string(),
            false,
        ),
        ExportEngineError::MissingScope => (
            "MISSING_SCOPE".to_string(),
            "token is missing a required scope".to_string(),
            false,
        ),
        ExportEngineError::NotFound => (
            "NOT_FOUND".to_string(),
            "base or table not found".to_string(),
            false,
        ),
        ExportEngineError::MalformedResponse(_) => (
            "MALFORMED_RESPONSE".to_string(),
            "received an unexpected response from the API".to_string(),
            false,
        ),
        ExportEngineError::TransientServerError(s) => (
            "TRANSIENT_SERVER_ERROR".to_string(),
            format!("transient server error (HTTP {s})"),
            true,
        ),
        ExportEngineError::SerialisationError(_) => (
            "SERIALISATION_ERROR".to_string(),
            "record serialisation failed".to_string(),
            false,
        ),
        ExportEngineError::PageLimitReached { table_id, pages } => (
            "PAGE_LIMIT_REACHED".to_string(),
            format!("page limit reached for table {table_id} after {pages} pages"),
            false,
        ),
    }
}

/// Open the written package and count total entries and checksum entries.
fn count_package_entries_and_checksums(path: &Path) -> Option<(usize, usize)> {
    let mut reader = BackupPackageReader::open(path).ok()?;
    let entry_count = reader.entry_count();
    let checksums = reader.read_checksums().ok()?;
    Some((entry_count, checksums.len()))
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::airtable::auth::AirtableToken;
    use crate::airtable::http::{MockHttpTransport, SequentialMockTransport};
    use crate::backup::export_engine::TableExportSpec;
    use crate::backup::validation::ValidationStatus;
    use tempfile::tempdir;

    const SENTINEL: &str = "pat_orchestrator_test_sentinel_0123456789";

    fn base_json() -> Vec<u8> {
        serde_json::to_vec(&serde_json::json!({"baseId":"appSyn01","name":"Synthetic"}))
            .expect("base_json")
    }

    fn schema_json() -> Vec<u8> {
        serde_json::to_vec(&serde_json::json!({"tables":[]})).expect("schema_json")
    }

    fn spec(table_id: &str) -> TableExportSpec {
        TableExportSpec {
            table_id: table_id.to_string(),
            table_name: format!("Table {table_id}"),
            linked_field_names: vec![],
            attachment_field_names: vec![],
        }
    }

    fn spec_with(table_id: &str, linked: Vec<&str>, attachments: Vec<&str>) -> TableExportSpec {
        TableExportSpec {
            table_id: table_id.to_string(),
            table_name: format!("Table {table_id}"),
            linked_field_names: linked.into_iter().map(|s| s.to_string()).collect(),
            attachment_field_names: attachments.into_iter().map(|s| s.to_string()).collect(),
        }
    }

    fn request(job_id: &str, specs: Vec<TableExportSpec>) -> BackupJobRequest {
        BackupJobRequest {
            job_id: BackupJobId(job_id.to_string()),
            base_id: "appSyn01".to_string(),
            base_name: "Synthetic".to_string(),
            base_json: base_json(),
            schema_json: schema_json(),
            table_specs: specs,
            page_size: 100,
        }
    }

    fn orchestrator_mock(transport: MockHttpTransport) -> BackupJobOrchestrator<MockHttpTransport> {
        let client = AirtableClient::new(AirtableToken::new(SENTINEL), transport);
        BackupJobOrchestrator::new(client, CancellationToken::new())
    }

    fn orchestrator_seq(
        transport: SequentialMockTransport,
        token: CancellationToken,
    ) -> BackupJobOrchestrator<SequentialMockTransport> {
        let client = AirtableClient::new(AirtableToken::new(SENTINEL), transport);
        BackupJobOrchestrator::new(client, token)
    }

    // ── Successful pipeline ────────────────────────────────────────────────

    #[test]
    fn successful_orchestration_returns_succeeded() {
        let body = r#"{"records":[{"id":"rec001","fields":{"Name":"Alpha"},"createdTime":"2026-01-01T00:00:00.000Z"}]}"#;
        let mut orch = orchestrator_mock(MockHttpTransport::ok(body));
        let dir = tempdir().expect("tempdir");
        let pkg = dir.path().join("job.airbridge");
        let req = request("job-orch-001", vec![spec("tbl01")]);
        let result = orch.run(&req, &pkg);
        assert_eq!(result.status, BackupJobStatus::Succeeded);
    }

    #[test]
    fn successful_orchestration_package_validates() {
        let body = r#"{"records":[{"id":"rec001","fields":{},"createdTime":null}]}"#;
        let mut orch = orchestrator_mock(MockHttpTransport::ok(body));
        let dir = tempdir().expect("tempdir");
        let pkg = dir.path().join("job.airbridge");
        let req = request("job-orch-002", vec![spec("tbl01")]);
        let result = orch.run(&req, &pkg);
        let vs = result.validation_summary.expect("validation summary");
        assert_eq!(vs.status, ValidationStatus::Valid);
    }

    #[test]
    fn two_page_export_succeeds() {
        let page1 =
            r#"{"records":[{"id":"rec001","fields":{},"createdTime":null}],"offset":"cursor2"}"#;
        let page2 = r#"{"records":[{"id":"rec002","fields":{},"createdTime":null}]}"#;
        let transport = SequentialMockTransport::new(vec![(200, page1), (200, page2)]);
        let mut orch = orchestrator_seq(transport, CancellationToken::new());
        let dir = tempdir().expect("tempdir");
        let pkg = dir.path().join("job.airbridge");
        let req = request("job-orch-003", vec![spec("tbl01")]);
        let result = orch.run(&req, &pkg);
        assert_eq!(result.status, BackupJobStatus::Succeeded);
        assert_eq!(result.tables[0].record_count, 2);
        assert_eq!(result.tables[0].pages_fetched, 2);
    }

    #[test]
    fn two_table_export_succeeds() {
        let body = r#"{"records":[{"id":"rec001","fields":{},"createdTime":null}]}"#;
        let mut orch = orchestrator_mock(MockHttpTransport::ok(body));
        let dir = tempdir().expect("tempdir");
        let pkg = dir.path().join("job.airbridge");
        let req = request("job-orch-004", vec![spec("tbl01"), spec("tbl02")]);
        let result = orch.run(&req, &pkg);
        assert_eq!(result.status, BackupJobStatus::Succeeded);
        assert_eq!(result.tables.len(), 2);
    }

    // ── Event order ────────────────────────────────────────────────────────

    #[test]
    fn event_order_includes_required_phases() {
        let body = r#"{"records":[]}"#;
        let mut orch = orchestrator_mock(MockHttpTransport::ok(body));
        let dir = tempdir().expect("tempdir");
        let pkg = dir.path().join("job.airbridge");
        let req = request("job-orch-005", vec![spec("tbl01")]);
        orch.run(&req, &pkg);
        let kinds: Vec<&str> = orch.events().iter().map(|e| e.kind_str()).collect();
        assert!(kinds.contains(&"jobStarted"), "missing jobStarted");
        assert!(
            kinds.contains(&"tableExportCompleted"),
            "missing tableExportCompleted"
        );
        assert!(
            kinds.contains(&"packageWriteCompleted"),
            "missing packageWriteCompleted"
        );
        assert!(
            kinds.contains(&"validationCompleted"),
            "missing validationCompleted"
        );
        assert!(kinds.contains(&"jobSucceeded"), "missing jobSucceeded");
    }

    #[test]
    fn job_started_is_first_event() {
        let body = r#"{"records":[]}"#;
        let mut orch = orchestrator_mock(MockHttpTransport::ok(body));
        let dir = tempdir().expect("tempdir");
        let pkg = dir.path().join("job.airbridge");
        let req = request("job-orch-006", vec![]);
        orch.run(&req, &pkg);
        assert_eq!(orch.events()[0].kind_str(), "jobStarted");
    }

    #[test]
    fn job_succeeded_is_last_event_on_success() {
        let body = r#"{"records":[]}"#;
        let mut orch = orchestrator_mock(MockHttpTransport::ok(body));
        let dir = tempdir().expect("tempdir");
        let pkg = dir.path().join("job.airbridge");
        let req = request("job-orch-007", vec![]);
        orch.run(&req, &pkg);
        let last = orch.events().last().expect("events");
        assert_eq!(last.kind_str(), "jobSucceeded");
    }

    // ── Package summary ────────────────────────────────────────────────────

    #[test]
    fn package_summary_has_positive_entry_count() {
        let body = r#"{"records":[]}"#;
        let mut orch = orchestrator_mock(MockHttpTransport::ok(body));
        let dir = tempdir().expect("tempdir");
        let pkg = dir.path().join("job.airbridge");
        let req = request("job-orch-008", vec![spec("tbl01")]);
        let result = orch.run(&req, &pkg);
        let ps = result.package_summary.expect("package summary");
        assert!(ps.entry_count > 0);
        assert!(ps.checksum_count > 0);
    }

    #[test]
    fn package_summary_encrypted_false() {
        let body = r#"{"records":[]}"#;
        let mut orch = orchestrator_mock(MockHttpTransport::ok(body));
        let dir = tempdir().expect("tempdir");
        let pkg = dir.path().join("job.airbridge");
        let req = request("job-orch-009", vec![]);
        let result = orch.run(&req, &pkg);
        let ps = result.package_summary.expect("package summary");
        assert!(!ps.encrypted);
        assert_eq!(ps.attachment_policy, "metadataOnly");
    }

    #[test]
    fn package_summary_record_count_matches_tables() {
        let body = r#"{"records":[{"id":"rec001","fields":{},"createdTime":null},{"id":"rec002","fields":{},"createdTime":null}]}"#;
        let mut orch = orchestrator_mock(MockHttpTransport::ok(body));
        let dir = tempdir().expect("tempdir");
        let pkg = dir.path().join("job.airbridge");
        let req = request("job-orch-010", vec![spec("tbl01")]);
        let result = orch.run(&req, &pkg);
        let ps = result.package_summary.expect("package summary");
        assert_eq!(ps.record_count, 2);
    }

    // ── Token / path safety ────────────────────────────────────────────────

    #[test]
    fn result_does_not_contain_token_sentinel() {
        let body = r#"{"records":[]}"#;
        let mut orch = orchestrator_mock(MockHttpTransport::ok(body));
        let dir = tempdir().expect("tempdir");
        let pkg = dir.path().join("job.airbridge");
        let req = request("job-orch-011", vec![]);
        let result = orch.run(&req, &pkg);
        let json = serde_json::to_string(&result).expect("serialize");
        assert!(!json.contains(SENTINEL));
    }

    #[test]
    fn result_does_not_contain_absolute_path() {
        let body = r#"{"records":[]}"#;
        let mut orch = orchestrator_mock(MockHttpTransport::ok(body));
        let dir = tempdir().expect("tempdir");
        let pkg = dir.path().join("job.airbridge");
        let req = request("job-orch-012", vec![]);
        let result = orch.run(&req, &pkg);
        let json = serde_json::to_string(&result).expect("serialize");
        assert!(!json.contains("/Users/"), "result contains absolute path");
        assert!(!json.contains("/home/"), "result contains home path");
    }

    #[test]
    fn events_do_not_contain_token_sentinel() {
        let body = r#"{"records":[]}"#;
        let mut orch = orchestrator_mock(MockHttpTransport::ok(body));
        let dir = tempdir().expect("tempdir");
        let pkg = dir.path().join("job.airbridge");
        let req = request("job-orch-013", vec![]);
        orch.run(&req, &pkg);
        for ev in orch.events() {
            let json = serde_json::to_string(ev).expect("serialize");
            assert!(!json.contains(SENTINEL), "event contains token sentinel");
        }
    }

    #[test]
    fn no_full_attachment_url_in_result() {
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
        let transport = MockHttpTransport::ok(body);
        let client = AirtableClient::new(AirtableToken::new(SENTINEL), transport);
        let mut orch = BackupJobOrchestrator::new(client, CancellationToken::new());
        let dir = tempdir().expect("tempdir");
        let pkg = dir.path().join("job.airbridge");
        let req = BackupJobRequest {
            job_id: BackupJobId("job-orch-014".to_string()),
            base_id: "appSyn01".to_string(),
            base_name: "Synthetic".to_string(),
            base_json: base_json(),
            schema_json: schema_json(),
            table_specs: vec![spec_with("tbl01", vec![], vec!["Files"])],
            page_size: 100,
        };
        let result = orch.run(&req, &pkg);
        let json = serde_json::to_string(&result).expect("serialize");
        assert!(
            !json.contains("dl.airtable.com"),
            "result contains attachment URL"
        );
        assert!(!json.contains("https://"), "result contains https URL");
    }

    // ── Error paths ────────────────────────────────────────────────────────

    #[test]
    fn error_401_returns_failed_status() {
        let transport = MockHttpTransport::with_status(401, r#"{"error":"UNAUTHORIZED"}"#);
        let mut orch = orchestrator_mock(transport);
        let dir = tempdir().expect("tempdir");
        let pkg = dir.path().join("job.airbridge");
        let req = request("job-orch-015", vec![spec("tbl01")]);
        let result = orch.run(&req, &pkg);
        assert_eq!(result.status, BackupJobStatus::Failed);
        assert!(result.errors.iter().any(|e| e.code == "AUTH_FAILED"));
    }

    #[test]
    fn error_403_returns_permission_denied() {
        let transport = MockHttpTransport::with_status(403, r#"{"error":"forbidden"}"#);
        let mut orch = orchestrator_mock(transport);
        let dir = tempdir().expect("tempdir");
        let pkg = dir.path().join("job.airbridge");
        let req = request("job-orch-016", vec![spec("tbl01")]);
        let result = orch.run(&req, &pkg);
        assert_eq!(result.status, BackupJobStatus::Failed);
        assert!(result.errors.iter().any(|e| e.code == "PERMISSION_DENIED"));
    }

    #[test]
    fn error_429_returns_rate_limited() {
        let transport = MockHttpTransport::with_status(429, r#"{"error":"RATE_LIMITED"}"#);
        let mut orch = orchestrator_mock(transport);
        let dir = tempdir().expect("tempdir");
        let pkg = dir.path().join("job.airbridge");
        let req = request("job-orch-017", vec![spec("tbl01")]);
        let result = orch.run(&req, &pkg);
        assert_eq!(result.status, BackupJobStatus::Failed);
        assert!(result.errors.iter().any(|e| e.code == "RATE_LIMITED"));
    }

    #[test]
    fn error_message_does_not_expose_token() {
        let transport = MockHttpTransport::with_status(401, r#"{"error":"UNAUTHORIZED"}"#);
        let mut orch = orchestrator_mock(transport);
        let dir = tempdir().expect("tempdir");
        let pkg = dir.path().join("job.airbridge");
        let req = request("job-orch-018", vec![spec("tbl01")]);
        let result = orch.run(&req, &pkg);
        let json = serde_json::to_string(&result).expect("serialize");
        assert!(!json.contains(SENTINEL));
    }

    // ── Cancellation ───────────────────────────────────────────────────────

    #[test]
    fn cancellation_before_export_returns_cancelled() {
        let token = CancellationToken::new();
        token.cancel();
        let transport = SequentialMockTransport::new(vec![(200, r#"{"records":[]}"#)]);
        let mut orch = orchestrator_seq(transport, token);
        let dir = tempdir().expect("tempdir");
        let pkg = dir.path().join("job.airbridge");
        let req = request("job-orch-019", vec![spec("tbl01")]);
        let result = orch.run(&req, &pkg);
        assert_eq!(result.status, BackupJobStatus::Cancelled);
    }

    #[test]
    fn cancellation_emits_job_cancelled_event() {
        let token = CancellationToken::new();
        token.cancel();
        let transport = SequentialMockTransport::new(vec![(200, r#"{"records":[]}"#)]);
        let mut orch = orchestrator_seq(transport, token);
        let dir = tempdir().expect("tempdir");
        let pkg = dir.path().join("job.airbridge");
        let req = request("job-orch-020", vec![spec("tbl01")]);
        orch.run(&req, &pkg);
        let kinds: Vec<&str> = orch.events().iter().map(|e| e.kind_str()).collect();
        assert!(
            kinds.contains(&"jobCancelled"),
            "missing jobCancelled event"
        );
    }

    #[test]
    fn cancellation_before_package_build_returns_cancelled() {
        struct CancelAfterExport {
            inner: MockHttpTransport,
            token: CancellationToken,
        }
        impl crate::airtable::http::HttpTransport for CancelAfterExport {
            fn send(
                &self,
                req: crate::airtable::http::HttpRequest,
            ) -> Result<crate::airtable::http::HttpResponse, String> {
                let resp = self.inner.send(req)?;
                self.token.cancel();
                Ok(resp)
            }
        }
        let token = CancellationToken::new();
        let transport = CancelAfterExport {
            inner: MockHttpTransport::ok(r#"{"records":[]}"#),
            token: token.clone(),
        };
        let client = AirtableClient::new(AirtableToken::new(SENTINEL), transport);
        let mut orch = BackupJobOrchestrator::new(client, token);
        let dir = tempdir().expect("tempdir");
        let pkg = dir.path().join("job.airbridge");
        let req = request("job-orch-021", vec![spec("tbl01")]);
        let result = orch.run(&req, &pkg);
        assert_eq!(result.status, BackupJobStatus::Cancelled);
    }
}
