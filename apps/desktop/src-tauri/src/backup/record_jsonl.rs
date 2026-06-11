use serde_json::{json, Value};

use crate::airtable::models::AirtableRecord;

/// Convert an `AirtableRecord` to a single JSONL line.
///
/// Output shape:
/// ```json
/// {"id":"recXXX","createdTime":"2026-01-01T00:00:00.000Z","fields":{...}}
/// ```
///
/// No attachment URLs are written — callers must strip them via `attachments`
/// module before passing records here, or the fields blob is stored as-is.
pub fn record_to_jsonl_line(record: &AirtableRecord) -> Result<String, serde_json::Error> {
    let obj = json!({
        "id": record.id.as_str(),
        "createdTime": record.created_time,
        "fields": Value::Object(
            record.fields.iter()
                .map(|(k, v)| (k.clone(), v.clone()))
                .collect()
        ),
    });
    serde_json::to_string(&obj)
}

/// Convert a slice of records to a Vec of JSONL lines.
pub fn records_to_jsonl_lines(
    records: &[AirtableRecord],
) -> Result<Vec<String>, serde_json::Error> {
    records.iter().map(record_to_jsonl_line).collect()
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
            created_time: Some("2026-01-01T00:00:00.000Z".to_string()),
        }
    }

    #[test]
    fn jsonl_line_contains_record_id() {
        let r = make_record("recAbc001", vec![]);
        let line = record_to_jsonl_line(&r).expect("encode");
        assert!(line.contains("recAbc001"));
    }

    #[test]
    fn jsonl_line_contains_created_time() {
        let r = make_record("recAbc001", vec![]);
        let line = record_to_jsonl_line(&r).expect("encode");
        assert!(line.contains("2026-01-01T00:00:00.000Z"));
    }

    #[test]
    fn jsonl_line_contains_fields_key() {
        let r = make_record(
            "recAbc001",
            vec![("Name", Value::String("Alice".to_string()))],
        );
        let line = record_to_jsonl_line(&r).expect("encode");
        assert!(line.contains("\"fields\""));
    }

    #[test]
    fn jsonl_line_contains_field_value() {
        let r = make_record(
            "recAbc001",
            vec![("Name", Value::String("Alice".to_string()))],
        );
        let line = record_to_jsonl_line(&r).expect("encode");
        assert!(line.contains("Alice"));
    }

    #[test]
    fn jsonl_line_is_single_line() {
        let r = make_record(
            "recAbc001",
            vec![("Name", Value::String("Alice".to_string()))],
        );
        let line = record_to_jsonl_line(&r).expect("encode");
        assert!(!line.contains('\n'));
    }

    #[test]
    fn jsonl_line_deserializes_to_json_object() {
        let r = make_record("recAbc001", vec![("Score", Value::Number(42.into()))]);
        let line = record_to_jsonl_line(&r).expect("encode");
        let parsed: Value = serde_json::from_str(&line).expect("parse");
        assert_eq!(parsed["id"], "recAbc001");
        assert_eq!(parsed["fields"]["Score"], 42);
    }

    #[test]
    fn records_to_jsonl_lines_returns_one_per_record() {
        let records = vec![
            make_record("rec001", vec![]),
            make_record("rec002", vec![]),
            make_record("rec003", vec![]),
        ];
        let lines = records_to_jsonl_lines(&records).expect("encode");
        assert_eq!(lines.len(), 3);
    }

    #[test]
    fn records_to_jsonl_lines_empty_input() {
        let lines = records_to_jsonl_lines(&[]).expect("encode");
        assert!(lines.is_empty());
    }

    #[test]
    fn jsonl_line_null_created_time_serializes() {
        let mut r = make_record("recAbc001", vec![]);
        r.created_time = None;
        let line = record_to_jsonl_line(&r).expect("encode");
        let parsed: Value = serde_json::from_str(&line).expect("parse");
        assert!(parsed["createdTime"].is_null());
    }

    #[test]
    fn jsonl_line_does_not_contain_token_sentinel() {
        const SENTINEL: &str = "pat_record_jsonl_sentinel_0123456789";
        let r = make_record("recAbc001", vec![]);
        let line = record_to_jsonl_line(&r).expect("encode");
        assert!(!line.contains(SENTINEL));
    }
}
