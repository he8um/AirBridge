pub mod airtable;
pub mod backup;
mod commands;
mod errors;
mod models;
pub mod restore;

use models::common::AppHealth;

#[tauri::command]
fn greet(name: &str) -> String {
    format!("Hello, {}! You've been greeted from Rust!", name)
}

#[tauri::command]
fn get_app_health() -> AppHealth {
    AppHealth {
        app_name: "AirBridge".to_string(),
        version: "0.1.0".to_string(),
        status: "ok".to_string(),
        backend: "tauri".to_string(),
    }
}

#[cfg_attr(mobile, tauri::mobile_entry_point)]
pub fn run() {
    tauri::Builder::default()
        .plugin(tauri_plugin_dialog::init())
        .plugin(tauri_plugin_opener::init())
        .invoke_handler(tauri::generate_handler![
            greet,
            get_app_health,
            commands::connection::check_connection,
            commands::catalog::list_workspaces,
            commands::catalog::list_bases,
            commands::catalog::list_accessible_bases,
            commands::catalog::get_base_schema,
            commands::backup::list_backup_packages,
            commands::backup::create_backup_plan,
            commands::backup::create_records_export_plan,
            commands::backup::inspect_backup_package,
            commands::backup_job::validate_backup_output_path,
            commands::backup_job::run_backup_job,
            commands::backup_job::cancel_backup_job,
            commands::restore::list_restore_plans,
            commands::restore::create_restore_dry_run_plan,
            commands::reports::list_reports,
            commands::logs::list_logs,
            commands::compatibility::list_compatibility_rules,
        ])
        .run(tauri::generate_context!())
        .expect("error while running tauri application");
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::commands::compatibility::list_compatibility_rules;
    use crate::commands::connection::check_connection;
    use crate::commands::logs::list_logs;
    use crate::commands::reports::list_reports;
    use crate::errors::{AirBridgeError, ErrorCode};
    use crate::models::compatibility::FieldRestoreSupport;

    #[test]
    fn health_returns_app_name() {
        let health = get_app_health();
        assert_eq!(health.app_name, "AirBridge");
    }

    #[test]
    fn health_returns_ok_status() {
        let health = get_app_health();
        assert_eq!(health.status, "ok");
    }

    #[test]
    fn check_connection_does_not_expose_token() {
        let result = check_connection("super-secret-token".to_string());
        let json = serde_json::to_string(&result).expect("serialization failed");
        assert!(!json.contains("super-secret-token"));
    }

    #[test]
    fn compatibility_rules_include_restorable() {
        let rules = list_compatibility_rules().expect("command failed");
        assert!(rules
            .iter()
            .any(|r| r.support == FieldRestoreSupport::Restorable));
    }

    #[test]
    fn compatibility_rules_include_unsupported() {
        let rules = list_compatibility_rules().expect("command failed");
        assert!(rules
            .iter()
            .any(|r| r.support == FieldRestoreSupport::UnsupportedForRestore));
    }

    #[test]
    fn reports_return_deterministic_data() {
        let reports = list_reports().expect("command failed");
        assert_eq!(reports.len(), 3);
        assert_eq!(reports[0].id, "report-001");
    }

    #[test]
    fn logs_return_deterministic_data() {
        let logs = list_logs().expect("command failed");
        assert_eq!(logs.len(), 6);
        assert_eq!(logs[0].id, "log-001");
    }

    #[test]
    fn error_serializes_with_code_and_message() {
        let err = AirBridgeError::new(ErrorCode::InternalError, "test");
        let json = serde_json::to_string(&err).expect("serialization failed");
        assert!(json.contains("INTERNAL_ERROR"));
        assert!(json.contains("test"));
    }
}
