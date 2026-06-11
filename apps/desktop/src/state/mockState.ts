import type { AppState } from "./appState";

export const MOCK_STATE: AppState = {
  connections: [
    {
      id: "conn-001",
      label: "Example Workspace (disconnected)",
      status: "disconnected",
      permissions: [
        { key: "schema-read", label: "Schema read", status: "unknown" },
        { key: "records-read", label: "Records read", status: "unknown" },
        { key: "schema-write", label: "Schema write", status: "unknown" },
        { key: "records-write", label: "Records write", status: "unknown" },
      ],
    },
    {
      id: "conn-002",
      label: "Example Workspace (connected)",
      status: "connected",
      connectedAt: "2025-01-15T09:00:00Z",
      permissions: [
        { key: "schema-read", label: "Schema read", status: "passed" },
        { key: "records-read", label: "Records read", status: "passed" },
        {
          key: "schema-write",
          label: "Schema write",
          status: "failed",
          detail: "Token lacks write scope",
        },
        {
          key: "records-write",
          label: "Records write",
          status: "failed",
          detail: "Token lacks write scope",
        },
      ],
    },
  ],

  workspaces: [
    {
      id: "wsExampleWorkspace01",
      name: "Example Workspace",
      bases: [
        {
          id: "appExampleBase01",
          workspaceId: "wsExampleWorkspace01",
          name: "Example Projects & Tasks",
          tableCount: 2,
        },
        {
          id: "appExampleBase02",
          workspaceId: "wsExampleWorkspace01",
          name: "Example Contacts",
          tableCount: 1,
        },
      ],
    },
  ],

  bases: [
    {
      id: "appExampleBase01",
      workspaceId: "wsExampleWorkspace01",
      name: "Example Projects & Tasks",
      tableCount: 2,
      tables: [
        {
          id: "tblProjects01",
          name: "Projects",
          fieldCount: 5,
          recordCount: 32,
          fields: [
            { id: "fldProjName", name: "Project Name", type: "singleLineText", primary: true },
            { id: "fldProjStatus", name: "Status", type: "singleSelect" },
            { id: "fldProjDue", name: "Due Date", type: "date" },
            { id: "fldProjOwner", name: "Owner", type: "singleLineText" },
            { id: "fldProjFormula", name: "Formula Result", type: "formula" },
          ],
        },
        {
          id: "tblTasks01",
          name: "Tasks",
          fieldCount: 4,
          recordCount: 15,
          fields: [
            { id: "fldTaskName", name: "Task Name", type: "singleLineText", primary: true },
            { id: "fldTaskProject", name: "Project", type: "multipleRecordLinks" },
            { id: "fldTaskDone", name: "Done", type: "checkbox" },
            { id: "fldTaskRollup", name: "Rollup Count", type: "rollup" },
          ],
        },
      ],
    },
    {
      id: "appExampleBase02",
      workspaceId: "wsExampleWorkspace01",
      name: "Example Contacts",
      tableCount: 1,
      tables: [
        {
          id: "tblContacts01",
          name: "Contacts",
          fieldCount: 4,
          recordCount: 28,
          fields: [
            { id: "fldContactName", name: "Full Name", type: "singleLineText", primary: true },
            { id: "fldContactEmail", name: "Email", type: "email" },
            { id: "fldContactPhone", name: "Phone", type: "phoneNumber" },
            { id: "fldContactCreated", name: "Created", type: "createdTime" },
          ],
        },
      ],
    },
  ],

  backupPackages: [
    {
      id: "pkg-001",
      connectionId: "conn-002",
      baseId: "appExampleBase01",
      workspaceId: "wsExampleWorkspace01",
      baseName: "Example Projects & Tasks",
      scope: "full",
      status: "succeeded",
      tableCount: 2,
      recordCount: 47,
      fileSizeBytes: 18432,
      createdAt: "2025-01-14T14:22:10Z",
      outputPath: "/Users/example/airbridge/backups/pkg-001.zip",
    },
    {
      id: "pkg-002",
      connectionId: "conn-002",
      baseId: "appExampleBase02",
      workspaceId: "wsExampleWorkspace01",
      baseName: "Example Contacts",
      scope: "schema_only",
      status: "succeeded",
      tableCount: 1,
      recordCount: 0,
      fileSizeBytes: 3072,
      createdAt: "2025-01-13T11:05:44Z",
      outputPath: "/Users/example/airbridge/backups/pkg-002.zip",
    },
    {
      id: "pkg-003",
      connectionId: "conn-002",
      baseId: "appExampleBase01",
      workspaceId: "wsExampleWorkspace01",
      baseName: "Example Projects & Tasks",
      scope: "full",
      status: "failed",
      tableCount: 0,
      recordCount: 0,
      fileSizeBytes: 0,
      createdAt: "2025-01-12T08:47:30Z",
      outputPath: "",
    },
  ],

  backupJobs: [
    {
      id: "job-001",
      connectionId: "conn-002",
      baseId: "appExampleBase01",
      baseName: "Example Projects & Tasks",
      scope: "full",
      status: "succeeded",
      startedAt: "2025-01-14T14:21:55Z",
      completedAt: "2025-01-14T14:22:10Z",
      packageId: "pkg-001",
      tablesProcessed: 2,
      totalTables: 2,
      recordsProcessed: 47,
    },
  ],

  restorePlans: [
    {
      id: "plan-001",
      packageId: "pkg-001",
      connectionId: "conn-002",
      mode: "new_base",
      status: "ready",
      warnings: [
        {
          fieldId: "fldProjFormula",
          fieldName: "Formula Result",
          fieldType: "formula",
          message:
            "Formula fields cannot be restored via the API. The field must be recreated manually in the target base.",
          severity: "warning",
        },
        {
          fieldId: "fldTaskRollup",
          fieldName: "Rollup Count",
          fieldType: "rollup",
          message:
            "Rollup configuration is captured in the schema backup but computed values will not be restored.",
          severity: "info",
        },
      ],
      createdAt: "2025-01-14T15:00:00Z",
    },
  ],

  restoreJobs: [
    {
      id: "rjob-001",
      planId: "plan-001",
      connectionId: "conn-002",
      isDryRun: true,
      status: "dry_run_complete",
      startedAt: "2025-01-14T15:01:00Z",
      completedAt: "2025-01-14T15:01:18Z",
      tablesRestored: 2,
      totalTables: 2,
      recordsRestored: 47,
      skippedFields: ["Formula Result", "Rollup Count"],
    },
  ],

  reports: [
    {
      id: "report-001",
      type: "backup",
      title: "Backup Report: Example Projects & Tasks",
      createdAt: "2025-01-14T14:22:12Z",
      severity: "info",
      itemCount: 1,
      relatedJobId: "job-001",
      relatedBaseId: "appExampleBase01",
      relatedBaseName: "Example Projects & Tasks",
      items: [
        {
          id: "ritem-001",
          severity: "info",
          title: "Backup completed successfully",
          detail: "2 tables and 47 records written to pkg-001.zip",
        },
      ],
    },
    {
      id: "report-002",
      type: "compatibility",
      title: "Compatibility Report: pkg-001",
      createdAt: "2025-01-14T14:22:14Z",
      severity: "warning",
      itemCount: 2,
      relatedBaseId: "appExampleBase01",
      relatedBaseName: "Example Projects & Tasks",
      items: [
        {
          id: "ritem-002",
          severity: "warning",
          fieldName: "Formula Result",
          tableName: "Projects",
          title: "Formula field is unsupported for restore",
          detail:
            "Formula expressions are stored in the schema backup but computed values cannot be restored via the API.",
        },
        {
          id: "ritem-003",
          severity: "info",
          fieldName: "Rollup Count",
          tableName: "Tasks",
          title: "Rollup field backed up as metadata only",
          detail:
            "Rollup configuration is captured. Computed values will not be present after restore.",
        },
      ],
    },
    {
      id: "report-003",
      type: "restore",
      title: "Dry-Run Report: plan-001",
      createdAt: "2025-01-14T15:01:20Z",
      severity: "warning",
      itemCount: 1,
      relatedJobId: "rjob-001",
      relatedBaseId: "appExampleBase01",
      relatedBaseName: "Example Projects & Tasks",
      items: [
        {
          id: "ritem-004",
          severity: "warning",
          title: "2 fields skipped during dry run",
          detail:
            "Fields skipped: Formula Result, Rollup Count. These fields require manual recreation in the target base.",
        },
      ],
    },
  ],

  logs: [
    {
      id: "log-001",
      timestamp: "2025-01-14T14:21:55Z",
      level: "debug",
      jobId: "job-001",
      jobType: "backup",
      message: "Initializing backup job",
    },
    {
      id: "log-002",
      timestamp: "2025-01-14T14:21:56Z",
      level: "info",
      jobId: "job-001",
      jobType: "backup",
      message: "Connected to Airtable API",
    },
    {
      id: "log-003",
      timestamp: "2025-01-14T14:22:00Z",
      level: "info",
      jobId: "job-001",
      jobType: "backup",
      message: "Backing up table: Projects (32 records)",
    },
    {
      id: "log-004",
      timestamp: "2025-01-14T14:22:04Z",
      level: "info",
      jobId: "job-001",
      jobType: "backup",
      message: "Backing up table: Tasks (15 records)",
    },
    {
      id: "log-005",
      timestamp: "2025-01-14T14:22:06Z",
      level: "warning",
      jobId: "job-001",
      jobType: "backup",
      message: "Rate limit reached, backing off 30s",
      detail: "HTTP 429 received from API endpoint. Retrying after delay.",
    },
    {
      id: "log-006",
      timestamp: "2025-01-14T14:22:10Z",
      level: "info",
      jobId: "job-001",
      jobType: "backup",
      message: "Backup complete: 47 records written",
    },
  ],

  compatibilityRules: [
    {
      fieldType: "singleLineText",
      support: "restorable",
      backupSupport: "full",
      note: "Restored as plain text field.",
    },
    {
      fieldType: "number",
      support: "restorable",
      backupSupport: "full",
      note: "Restored with original precision settings where supported.",
    },
    {
      fieldType: "singleSelect",
      support: "restorable",
      backupSupport: "full",
      note: "Options are recreated in the target base.",
    },
    {
      fieldType: "multipleRecordLinks",
      support: "partially_restorable",
      backupSupport: "full",
      note: "Link targets are remapped by record ID during restore. Unresolved links are skipped.",
    },
    {
      fieldType: "formula",
      support: "unsupported_for_restore",
      backupSupport: "metadata_only",
      note: "Formula expressions are stored in the schema backup. Computed values are not restored; the field must be recreated manually.",
    },
    {
      fieldType: "rollup",
      support: "metadata_only",
      backupSupport: "metadata_only",
      note: "Rollup configuration is captured in schema. Values are not restored.",
    },
    {
      fieldType: "createdTime",
      support: "metadata_only",
      backupSupport: "metadata_only",
      note: "Original creation timestamps cannot be restored via the API.",
    },
    {
      fieldType: "multipleAttachments",
      support: "partially_restorable",
      backupSupport: "partial",
      note: "Attachment metadata is backed up. File content is not re-uploaded; original attachment URLs are stored as reference only.",
    },
  ],

  selectedConnectionId: "conn-002",
  selectedBaseId: "appExampleBase01",
};
