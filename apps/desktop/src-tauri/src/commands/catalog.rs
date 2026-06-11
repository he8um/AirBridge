use crate::airtable::auth::AirtableToken;
use crate::airtable::client::AirtableClient;
use crate::airtable::errors::AirtableClientError;
use crate::airtable::http::ReqwestHttpTransport;
use crate::airtable::models::{AccessibleBaseSummary, BaseSchemaSummary};
use crate::airtable::schema::summarize_schema;
use crate::errors::{AirBridgeError, AirBridgeResult, ErrorCode};
use crate::models::catalog::{BaseSummary, FieldSummary, TableSummary, WorkspaceSummary};

#[tauri::command]
pub fn list_workspaces() -> AirBridgeResult<Vec<WorkspaceSummary>> {
    Ok(vec![WorkspaceSummary {
        id: "wsExampleWorkspace01".to_string(),
        name: "Example Workspace".to_string(),
        base_count: 2,
    }])
}

#[tauri::command]
pub fn list_bases() -> AirBridgeResult<Vec<BaseSummary>> {
    Ok(vec![
        BaseSummary {
            id: "appExampleBase01".to_string(),
            workspace_id: "wsExampleWorkspace01".to_string(),
            name: "Example Projects & Tasks".to_string(),
            table_count: 2,
            tables: vec![
                TableSummary {
                    id: "tblProjects01".to_string(),
                    name: "Projects".to_string(),
                    field_count: 5,
                    record_count: Some(32),
                    fields: vec![
                        FieldSummary {
                            id: "fldProjName".to_string(),
                            name: "ProjectName".to_string(),
                            field_type: "singleLineText".to_string(),
                            primary: true,
                        },
                        FieldSummary {
                            id: "fldProjStatus".to_string(),
                            name: "Status".to_string(),
                            field_type: "singleSelect".to_string(),
                            primary: false,
                        },
                        FieldSummary {
                            id: "fldProjDueDate".to_string(),
                            name: "DueDate".to_string(),
                            field_type: "date".to_string(),
                            primary: false,
                        },
                        FieldSummary {
                            id: "fldProjOwner".to_string(),
                            name: "Owner".to_string(),
                            field_type: "singleLineText".to_string(),
                            primary: false,
                        },
                        FieldSummary {
                            id: "fldProjFormula".to_string(),
                            name: "FormulaResult".to_string(),
                            field_type: "formula".to_string(),
                            primary: false,
                        },
                    ],
                },
                TableSummary {
                    id: "tblTasks01".to_string(),
                    name: "Tasks".to_string(),
                    field_count: 4,
                    record_count: Some(15),
                    fields: vec![
                        FieldSummary {
                            id: "fldTaskName".to_string(),
                            name: "TaskName".to_string(),
                            field_type: "singleLineText".to_string(),
                            primary: true,
                        },
                        FieldSummary {
                            id: "fldTaskProject".to_string(),
                            name: "Project".to_string(),
                            field_type: "multipleRecordLinks".to_string(),
                            primary: false,
                        },
                        FieldSummary {
                            id: "fldTaskDone".to_string(),
                            name: "Done".to_string(),
                            field_type: "checkbox".to_string(),
                            primary: false,
                        },
                        FieldSummary {
                            id: "fldTaskRollup".to_string(),
                            name: "RollupCount".to_string(),
                            field_type: "rollup".to_string(),
                            primary: false,
                        },
                    ],
                },
            ],
        },
        BaseSummary {
            id: "appExampleBase02".to_string(),
            workspace_id: "wsExampleWorkspace01".to_string(),
            name: "Example Contacts".to_string(),
            table_count: 1,
            tables: vec![TableSummary {
                id: "tblContacts01".to_string(),
                name: "Contacts".to_string(),
                field_count: 4,
                record_count: Some(28),
                fields: vec![
                    FieldSummary {
                        id: "fldContactName".to_string(),
                        name: "Name".to_string(),
                        field_type: "singleLineText".to_string(),
                        primary: true,
                    },
                    FieldSummary {
                        id: "fldContactEmail".to_string(),
                        name: "Email".to_string(),
                        field_type: "email".to_string(),
                        primary: false,
                    },
                    FieldSummary {
                        id: "fldContactPhone".to_string(),
                        name: "Phone".to_string(),
                        field_type: "phoneNumber".to_string(),
                        primary: false,
                    },
                    FieldSummary {
                        id: "fldContactNotes".to_string(),
                        name: "Notes".to_string(),
                        field_type: "multilineText".to_string(),
                        primary: false,
                    },
                ],
            }],
        },
    ])
}

// ── Live catalog commands ──────────────────────────────────────────────────

fn map_catalog_error(err: AirtableClientError) -> AirBridgeError {
    match err {
        AirtableClientError::InvalidToken => {
            AirBridgeError::new(ErrorCode::AuthInvalidToken, "Invalid or expired token")
        }
        AirtableClientError::MissingScope => AirBridgeError::new(
            ErrorCode::AuthMissingScope,
            "Token is missing required scopes",
        ),
        AirtableClientError::PermissionDenied => AirBridgeError::new(
            ErrorCode::PermissionDenied,
            "Permission denied for the requested resource",
        ),
        AirtableClientError::NotFound => {
            AirBridgeError::new(ErrorCode::InternalError, "Resource not found")
        }
        AirtableClientError::RateLimited => {
            AirBridgeError::new(ErrorCode::RateLimited, "Rate limited by Airtable API")
        }
        AirtableClientError::ValidationError(msg) => {
            AirBridgeError::new(ErrorCode::InternalError, format!("Validation error: {msg}"))
        }
        AirtableClientError::TransientServerError(_)
        | AirtableClientError::MalformedResponse(_) => {
            AirBridgeError::new(ErrorCode::NetworkUnavailable, "Network or server error")
        }
    }
}

/// Returns all bases accessible to the supplied token as lightweight summaries.
///
/// Read-only. Token is dropped immediately after wrapping. Never persisted.
#[tauri::command]
pub fn list_accessible_bases(token: String) -> AirBridgeResult<Vec<AccessibleBaseSummary>> {
    if token.trim().is_empty() {
        return Err(AirBridgeError::new(
            ErrorCode::AuthInvalidToken,
            "Token must not be empty",
        ));
    }

    let airtable_token = AirtableToken::new(&token);
    drop(token);

    let transport = ReqwestHttpTransport::new().map_err(|_| {
        AirBridgeError::new(
            ErrorCode::NetworkUnavailable,
            "Failed to initialize HTTP client",
        )
    })?;

    let client = AirtableClient::new(airtable_token, transport);
    client.list_accessible_bases().map_err(map_catalog_error)
}

/// Returns a schema summary for the given base id.
///
/// Read-only. Token is dropped immediately after wrapping. Never persisted.
#[tauri::command]
pub fn get_base_schema(token: String, base_id: String) -> AirBridgeResult<BaseSchemaSummary> {
    if token.trim().is_empty() {
        return Err(AirBridgeError::new(
            ErrorCode::AuthInvalidToken,
            "Token must not be empty",
        ));
    }
    if base_id.trim().is_empty() {
        return Err(AirBridgeError::new(
            ErrorCode::InternalError,
            "Base ID must not be empty",
        ));
    }

    let airtable_token = AirtableToken::new(&token);
    drop(token);

    let transport = ReqwestHttpTransport::new().map_err(|_| {
        AirBridgeError::new(
            ErrorCode::NetworkUnavailable,
            "Failed to initialize HTTP client",
        )
    })?;

    let client = AirtableClient::new(airtable_token, transport);
    let tables = client
        .get_base_schema(&base_id)
        .map_err(map_catalog_error)?;
    Ok(summarize_schema(&base_id, &tables))
}

// ── Unit-testable helpers ─────────────────────────────────────────────────

/// For testing: bypass transport and call list_accessible_bases logic directly.
#[cfg(test)]
pub fn list_accessible_bases_with_result(
    raw_token: &str,
    result: Result<Vec<crate::airtable::models::AccessibleBaseSummary>, AirtableClientError>,
) -> AirBridgeResult<Vec<AccessibleBaseSummary>> {
    if raw_token.trim().is_empty() {
        return Err(AirBridgeError::new(
            ErrorCode::AuthInvalidToken,
            "Token must not be empty",
        ));
    }
    result.map_err(map_catalog_error)
}

/// For testing: bypass transport and call get_base_schema logic directly.
#[cfg(test)]
pub fn get_base_schema_with_result(
    raw_token: &str,
    base_id: &str,
    result: Result<Vec<crate::airtable::models::AirtableTable>, AirtableClientError>,
) -> AirBridgeResult<BaseSchemaSummary> {
    if raw_token.trim().is_empty() {
        return Err(AirBridgeError::new(
            ErrorCode::AuthInvalidToken,
            "Token must not be empty",
        ));
    }
    let tables = result.map_err(map_catalog_error)?;
    Ok(summarize_schema(base_id, &tables))
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::airtable::models::{AirtableField, AirtableFieldId, AirtableTable, AirtableTableId};

    const SENTINEL: &str = "pat_example_catalog_sentinel_abcdef01";

    fn restorable_field(id: &str, name: &str) -> AirtableField {
        AirtableField {
            id: AirtableFieldId(id.to_string()),
            name: name.to_string(),
            field_type: "singleLineText".to_string(),
            options: None,
        }
    }

    fn formula_field(id: &str, name: &str) -> AirtableField {
        AirtableField {
            id: AirtableFieldId(id.to_string()),
            name: name.to_string(),
            field_type: "formula".to_string(),
            options: None,
        }
    }

    fn simple_table(id: &str, name: &str, fields: Vec<AirtableField>) -> AirtableTable {
        AirtableTable {
            id: AirtableTableId(id.to_string()),
            name: name.to_string(),
            primary_field_id: None,
            fields,
        }
    }

    // ── list_accessible_bases_with_result ─────────────────────────────────

    #[test]
    fn empty_token_returns_auth_error() {
        let result = list_accessible_bases_with_result("", Ok(vec![]));
        let err = result.unwrap_err();
        assert!(matches!(err.code, ErrorCode::AuthInvalidToken));
    }

    #[test]
    fn whitespace_token_returns_auth_error() {
        let result = list_accessible_bases_with_result("   ", Ok(vec![]));
        assert!(result.is_err());
    }

    #[test]
    fn list_bases_success_returns_summaries() {
        let bases = vec![
            AccessibleBaseSummary {
                id: "appExampleBase01".to_string(),
                name: "Base One".to_string(),
            },
            AccessibleBaseSummary {
                id: "appExampleBase02".to_string(),
                name: "Base Two".to_string(),
            },
        ];
        let result =
            list_accessible_bases_with_result(SENTINEL, Ok(bases)).expect("should succeed");
        assert_eq!(result.len(), 2);
        assert_eq!(result[0].id, "appExampleBase01");
    }

    #[test]
    fn list_bases_result_does_not_contain_token() {
        let bases = vec![AccessibleBaseSummary {
            id: "appExampleBase01".to_string(),
            name: "Example Base".to_string(),
        }];
        let result =
            list_accessible_bases_with_result(SENTINEL, Ok(bases)).expect("should succeed");
        let json = serde_json::to_string(&result).expect("serialize");
        assert!(!json.contains(SENTINEL));
    }

    #[test]
    fn list_bases_invalid_token_maps_correctly() {
        let result =
            list_accessible_bases_with_result(SENTINEL, Err(AirtableClientError::InvalidToken));
        let err = result.unwrap_err();
        assert!(matches!(err.code, ErrorCode::AuthInvalidToken));
        assert!(!err.message.contains(SENTINEL));
    }

    #[test]
    fn list_bases_rate_limited_maps_correctly() {
        let result =
            list_accessible_bases_with_result(SENTINEL, Err(AirtableClientError::RateLimited));
        let err = result.unwrap_err();
        assert!(matches!(err.code, ErrorCode::RateLimited));
    }

    // ── get_base_schema_with_result ────────────────────────────────────────

    #[test]
    fn get_schema_empty_token_returns_auth_error() {
        let result = get_base_schema_with_result("", "appExampleBase01", Ok(vec![]));
        assert!(result.is_err());
    }

    #[test]
    fn get_schema_empty_tables_returns_zero_count() {
        let result = get_base_schema_with_result(SENTINEL, "appExampleBase01", Ok(vec![]))
            .expect("should succeed");
        assert_eq!(result.table_count, 0);
        assert_eq!(result.base_id, "appExampleBase01");
    }

    #[test]
    fn get_schema_counts_restorable_and_metadata_only() {
        let tables = vec![simple_table(
            "tblTestTable01",
            "Projects",
            vec![
                restorable_field("fld001", "Name"),
                restorable_field("fld002", "Status"),
                formula_field("fld003", "ComputedValue"),
            ],
        )];
        let result = get_base_schema_with_result(SENTINEL, "appExampleBase01", Ok(tables))
            .expect("should succeed");
        assert_eq!(result.table_count, 1);
        assert_eq!(result.compatibility.restorable_count, 2);
        assert_eq!(result.compatibility.metadata_only_count, 1);
        assert_eq!(result.compatibility.total_count, 3);
    }

    #[test]
    fn get_schema_result_does_not_contain_token() {
        let tables = vec![simple_table(
            "tblTestTable01",
            "Table",
            vec![restorable_field("fld001", "Name")],
        )];
        let result = get_base_schema_with_result(SENTINEL, "appExampleBase01", Ok(tables))
            .expect("should succeed");
        let json = serde_json::to_string(&result).expect("serialize");
        assert!(!json.contains(SENTINEL));
    }

    #[test]
    fn get_schema_not_found_maps_correctly() {
        let result = get_base_schema_with_result(
            SENTINEL,
            "appExampleBase01",
            Err(AirtableClientError::NotFound),
        );
        let err = result.unwrap_err();
        assert!(matches!(err.code, ErrorCode::InternalError));
    }

    #[test]
    fn get_schema_error_never_contains_token() {
        let errors = vec![
            AirtableClientError::InvalidToken,
            AirtableClientError::MissingScope,
            AirtableClientError::PermissionDenied,
            AirtableClientError::RateLimited,
            AirtableClientError::TransientServerError(500),
            AirtableClientError::MalformedResponse("bad json".to_string()),
        ];
        for err in errors {
            let result = get_base_schema_with_result(SENTINEL, "appExampleBase01", Err(err));
            let err_msg = result.unwrap_err().message;
            assert!(
                !err_msg.contains(SENTINEL),
                "error message contained token: {err_msg}"
            );
        }
    }
}
