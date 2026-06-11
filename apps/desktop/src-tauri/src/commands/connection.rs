use crate::errors::AirBridgeResult;
use crate::models::connection::{
    ConnectionCheckResult, ConnectionStatus, PermissionCheck, PermissionCheckStatus,
};

#[tauri::command]
pub fn check_connection(token: String) -> AirBridgeResult<ConnectionCheckResult> {
    let _ = token;

    Ok(ConnectionCheckResult {
        connection_id: "conn-preview".to_string(),
        status: ConnectionStatus::Connected,
        permissions: vec![
            PermissionCheck {
                key: "schema:read".to_string(),
                label: "Read schema".to_string(),
                status: PermissionCheckStatus::Passed,
                detail: None,
            },
            PermissionCheck {
                key: "records:read".to_string(),
                label: "Read records".to_string(),
                status: PermissionCheckStatus::Passed,
                detail: None,
            },
            PermissionCheck {
                key: "schema:write".to_string(),
                label: "Write schema".to_string(),
                status: PermissionCheckStatus::Failed,
                detail: Some("Token scope does not include write access".to_string()),
            },
            PermissionCheck {
                key: "records:write".to_string(),
                label: "Write records".to_string(),
                status: PermissionCheckStatus::Failed,
                detail: Some("Token scope does not include write access".to_string()),
            },
        ],
    })
}
