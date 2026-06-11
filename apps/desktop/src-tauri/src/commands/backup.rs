use crate::errors::AirBridgeResult;
use crate::models::backup::{BackupPackageSummary, BackupScope, BackupStatus};

#[tauri::command]
pub fn list_backup_packages() -> AirBridgeResult<Vec<BackupPackageSummary>> {
    Ok(vec![
        BackupPackageSummary {
            id: "pkg-001".to_string(),
            connection_id: "conn-002".to_string(),
            base_id: "appExampleBase01".to_string(),
            workspace_id: "wsExampleWorkspace01".to_string(),
            base_name: "Example Projects & Tasks".to_string(),
            scope: BackupScope::Full,
            status: BackupStatus::Succeeded,
            table_count: 2,
            record_count: 47,
            file_size_bytes: 18432,
            created_at: "2025-01-14T14:22:10Z".to_string(),
            output_path: "".to_string(),
        },
        BackupPackageSummary {
            id: "pkg-002".to_string(),
            connection_id: "conn-002".to_string(),
            base_id: "appExampleBase02".to_string(),
            workspace_id: "wsExampleWorkspace01".to_string(),
            base_name: "Example Contacts".to_string(),
            scope: BackupScope::SchemaOnly,
            status: BackupStatus::Succeeded,
            table_count: 1,
            record_count: 0,
            file_size_bytes: 3072,
            created_at: "2025-01-13T11:05:44Z".to_string(),
            output_path: "".to_string(),
        },
        BackupPackageSummary {
            id: "pkg-003".to_string(),
            connection_id: "conn-002".to_string(),
            base_id: "appExampleBase01".to_string(),
            workspace_id: "wsExampleWorkspace01".to_string(),
            base_name: "Example Projects & Tasks".to_string(),
            scope: BackupScope::Full,
            status: BackupStatus::Failed,
            table_count: 0,
            record_count: 0,
            file_size_bytes: 0,
            created_at: "2025-01-12T08:47:30Z".to_string(),
            output_path: "".to_string(),
        },
    ])
}
