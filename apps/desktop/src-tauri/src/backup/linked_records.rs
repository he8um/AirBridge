use serde::{Deserialize, Serialize};
use serde_json::Value;

use crate::airtable::models::AirtableRecord;

/// A single extracted linked-record reference.
#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct LinkedRecordReference {
    /// ID of the record that contains the link field.
    pub source_record_id: String,
    /// Name of the link field (human-readable).
    pub field_name: String,
    /// IDs of the linked records.
    pub linked_record_ids: Vec<String>,
}

/// Extract all `multipleRecordLinks` references from a set of records.
///
/// `linked_field_names` is the list of field names known to be link fields
/// for this table (derived from the schema).
pub fn extract_linked_references(
    records: &[AirtableRecord],
    linked_field_names: &[&str],
) -> Vec<LinkedRecordReference> {
    let mut out = Vec::new();

    for record in records {
        for &field_name in linked_field_names {
            if let Some(value) = record.fields.get(field_name) {
                let ids = extract_record_ids(value);
                if !ids.is_empty() {
                    out.push(LinkedRecordReference {
                        source_record_id: record.id.as_str().to_string(),
                        field_name: field_name.to_string(),
                        linked_record_ids: ids,
                    });
                }
            }
        }
    }

    out
}

/// Serialise a slice of references to JSONL bytes (one object per line).
pub fn references_to_jsonl(refs: &[LinkedRecordReference]) -> Vec<u8> {
    let mut buf = Vec::new();
    for r in refs {
        if let Ok(line) = serde_json::to_string(r) {
            buf.extend_from_slice(line.as_bytes());
            buf.push(b'\n');
        }
    }
    buf
}

fn extract_record_ids(value: &Value) -> Vec<String> {
    match value {
        Value::Array(arr) => arr
            .iter()
            .filter_map(|v| v.as_str().map(|s| s.to_string()))
            .collect(),
        Value::String(s) => vec![s.clone()],
        _ => vec![],
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::airtable::models::AirtableRecordId;
    use std::collections::HashMap;

    fn make_record(id: &str, fields: Vec<(&str, Value)>) -> AirtableRecord {
        let mut map = HashMap::new();
        for (k, v) in fields {
            map.insert(k.to_string(), v);
        }
        AirtableRecord {
            id: AirtableRecordId(id.to_string()),
            fields: map,
            created_time: None,
        }
    }

    #[test]
    fn extracts_linked_ids_from_array_field() {
        let r = make_record(
            "recSrc01",
            vec![(
                "Tasks",
                Value::Array(vec![
                    Value::String("recLink01".to_string()),
                    Value::String("recLink02".to_string()),
                ]),
            )],
        );
        let refs = extract_linked_references(&[r], &["Tasks"]);
        assert_eq!(refs.len(), 1);
        assert_eq!(refs[0].linked_record_ids.len(), 2);
        assert_eq!(refs[0].linked_record_ids[0], "recLink01");
    }

    #[test]
    fn extracts_source_record_id() {
        let r = make_record(
            "recSrc01",
            vec![(
                "Tasks",
                Value::Array(vec![Value::String("recLink01".to_string())]),
            )],
        );
        let refs = extract_linked_references(&[r], &["Tasks"]);
        assert_eq!(refs[0].source_record_id, "recSrc01");
    }

    #[test]
    fn extracts_field_name() {
        let r = make_record(
            "recSrc01",
            vec![(
                "Projects",
                Value::Array(vec![Value::String("recLink01".to_string())]),
            )],
        );
        let refs = extract_linked_references(&[r], &["Projects"]);
        assert_eq!(refs[0].field_name, "Projects");
    }

    #[test]
    fn skips_non_link_fields() {
        let r = make_record(
            "recSrc01",
            vec![
                ("Name", Value::String("Alice".to_string())),
                (
                    "Tasks",
                    Value::Array(vec![Value::String("recLink01".to_string())]),
                ),
            ],
        );
        let refs = extract_linked_references(&[r], &["Tasks"]);
        assert_eq!(refs.len(), 1);
    }

    #[test]
    fn empty_link_array_produces_no_ref() {
        let r = make_record("recSrc01", vec![("Tasks", Value::Array(vec![]))]);
        let refs = extract_linked_references(&[r], &["Tasks"]);
        assert!(refs.is_empty());
    }

    #[test]
    fn missing_field_produces_no_ref() {
        let r = make_record("recSrc01", vec![]);
        let refs = extract_linked_references(&[r], &["Tasks"]);
        assert!(refs.is_empty());
    }

    #[test]
    fn multiple_records_multiple_fields() {
        let records = vec![
            make_record(
                "recSrc01",
                vec![(
                    "A",
                    Value::Array(vec![Value::String("recLink01".to_string())]),
                )],
            ),
            make_record(
                "recSrc02",
                vec![(
                    "B",
                    Value::Array(vec![Value::String("recLink02".to_string())]),
                )],
            ),
        ];
        let refs = extract_linked_references(&records, &["A", "B"]);
        assert_eq!(refs.len(), 2);
    }

    #[test]
    fn references_to_jsonl_produces_one_line_per_ref() {
        let refs = vec![
            LinkedRecordReference {
                source_record_id: "recSrc01".to_string(),
                field_name: "Tasks".to_string(),
                linked_record_ids: vec!["recLink01".to_string()],
            },
            LinkedRecordReference {
                source_record_id: "recSrc02".to_string(),
                field_name: "Projects".to_string(),
                linked_record_ids: vec!["recLink02".to_string()],
            },
        ];
        let bytes = references_to_jsonl(&refs);
        let text = String::from_utf8(bytes).expect("utf8");
        let lines: Vec<&str> = text.lines().collect();
        assert_eq!(lines.len(), 2);
    }

    #[test]
    fn references_to_jsonl_each_line_is_valid_json() {
        let refs = vec![LinkedRecordReference {
            source_record_id: "recSrc01".to_string(),
            field_name: "Tasks".to_string(),
            linked_record_ids: vec!["recLink01".to_string()],
        }];
        let bytes = references_to_jsonl(&refs);
        let text = String::from_utf8(bytes).expect("utf8");
        for line in text.lines() {
            let _: serde_json::Value = serde_json::from_str(line).expect("valid json");
        }
    }

    #[test]
    fn references_to_jsonl_empty_input() {
        let bytes = references_to_jsonl(&[]);
        assert!(bytes.is_empty());
    }

    #[test]
    fn jsonl_does_not_contain_token_sentinel() {
        const SENTINEL: &str = "pat_linked_sentinel_0123456789";
        let refs = vec![LinkedRecordReference {
            source_record_id: "recSrc01".to_string(),
            field_name: "Tasks".to_string(),
            linked_record_ids: vec!["recLink01".to_string()],
        }];
        let bytes = references_to_jsonl(&refs);
        let text = String::from_utf8(bytes).expect("utf8");
        assert!(!text.contains(SENTINEL));
    }
}
