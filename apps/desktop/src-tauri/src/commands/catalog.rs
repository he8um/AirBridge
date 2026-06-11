use crate::errors::AirBridgeResult;
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
