use serde::{Deserialize, Serialize};
use serde_json::Value;

use crate::airtable::models::AirtableRecord;

/// Attachment metadata for a single file within a record's attachment field.
///
/// V0.1 policy: full URLs are NOT stored. Only structural metadata is kept.
/// `url_present` records whether the API returned a URL without storing it.
#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct AttachmentMetadata {
    pub record_id: String,
    pub field_name: String,
    pub attachment_id: String,
    pub filename: String,
    /// MIME type, if present in the API response.
    #[serde(skip_serializing_if = "Option::is_none")]
    pub content_type: Option<String>,
    /// File size in bytes, if present in the API response.
    #[serde(skip_serializing_if = "Option::is_none")]
    pub size_bytes: Option<u64>,
    /// True if the API response included a URL. The URL itself is not stored.
    pub url_present: bool,
}

/// Extract attachment metadata from a set of records for the given field names.
///
/// `attachment_field_names` lists the fields known to be `multipleAttachments`
/// for this table (derived from the schema).
///
/// URL values are intentionally discarded — only `url_present` is recorded.
pub fn extract_attachment_metadata(
    records: &[AirtableRecord],
    attachment_field_names: &[&str],
) -> Vec<AttachmentMetadata> {
    let mut out = Vec::new();

    for record in records {
        for &field_name in attachment_field_names {
            if let Some(Value::Array(attachments)) = record.fields.get(field_name) {
                for att in attachments {
                    if let Some(obj) = att.as_object() {
                        let attachment_id = obj
                            .get("id")
                            .and_then(|v| v.as_str())
                            .unwrap_or("")
                            .to_string();
                        let filename = obj
                            .get("filename")
                            .and_then(|v| v.as_str())
                            .unwrap_or("")
                            .to_string();
                        let content_type = obj
                            .get("type")
                            .and_then(|v| v.as_str())
                            .map(|s| s.to_string());
                        let size_bytes = obj.get("size").and_then(|v| v.as_u64());
                        let url_present = obj.contains_key("url");

                        out.push(AttachmentMetadata {
                            record_id: record.id.as_str().to_string(),
                            field_name: field_name.to_string(),
                            attachment_id,
                            filename,
                            content_type,
                            size_bytes,
                            url_present,
                        });
                    }
                }
            }
        }
    }

    out
}

/// Serialise a slice of metadata entries to JSONL bytes (one object per line).
pub fn attachment_metadata_to_jsonl(entries: &[AttachmentMetadata]) -> Vec<u8> {
    let mut buf = Vec::new();
    for entry in entries {
        if let Ok(line) = serde_json::to_string(entry) {
            buf.extend_from_slice(line.as_bytes());
            buf.push(b'\n');
        }
    }
    buf
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::airtable::models::AirtableRecordId;
    use serde_json::json;
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

    fn attachment_value(id: &str, filename: &str, with_url: bool) -> Value {
        let mut obj = serde_json::Map::new();
        obj.insert("id".to_string(), json!(id));
        obj.insert("filename".to_string(), json!(filename));
        obj.insert("type".to_string(), json!("image/png"));
        obj.insert("size".to_string(), json!(1024u64));
        if with_url {
            obj.insert("url".to_string(), json!("https://dl.airtable.com/REDACTED"));
        }
        Value::Object(obj)
    }

    #[test]
    fn extracts_attachment_id() {
        let r = make_record(
            "recSrc01",
            vec![(
                "Files",
                Value::Array(vec![attachment_value("attAbc01", "photo.png", false)]),
            )],
        );
        let meta = extract_attachment_metadata(&[r], &["Files"]);
        assert_eq!(meta[0].attachment_id, "attAbc01");
    }

    #[test]
    fn extracts_filename() {
        let r = make_record(
            "recSrc01",
            vec![(
                "Files",
                Value::Array(vec![attachment_value("attAbc01", "photo.png", false)]),
            )],
        );
        let meta = extract_attachment_metadata(&[r], &["Files"]);
        assert_eq!(meta[0].filename, "photo.png");
    }

    #[test]
    fn url_not_stored_when_present() {
        let r = make_record(
            "recSrc01",
            vec![(
                "Files",
                Value::Array(vec![attachment_value("attAbc01", "photo.png", true)]),
            )],
        );
        let meta = extract_attachment_metadata(&[r], &["Files"]);
        let json = serde_json::to_string(&meta[0]).expect("serialize");
        assert!(!json.contains("dl.airtable.com"));
        assert!(!json.contains("https://"));
    }

    #[test]
    fn url_present_true_when_url_in_response() {
        let r = make_record(
            "recSrc01",
            vec![(
                "Files",
                Value::Array(vec![attachment_value("attAbc01", "photo.png", true)]),
            )],
        );
        let meta = extract_attachment_metadata(&[r], &["Files"]);
        assert!(meta[0].url_present);
    }

    #[test]
    fn url_present_false_when_no_url_in_response() {
        let r = make_record(
            "recSrc01",
            vec![(
                "Files",
                Value::Array(vec![attachment_value("attAbc01", "photo.png", false)]),
            )],
        );
        let meta = extract_attachment_metadata(&[r], &["Files"]);
        assert!(!meta[0].url_present);
    }

    #[test]
    fn extracts_content_type() {
        let r = make_record(
            "recSrc01",
            vec![(
                "Files",
                Value::Array(vec![attachment_value("attAbc01", "photo.png", false)]),
            )],
        );
        let meta = extract_attachment_metadata(&[r], &["Files"]);
        assert_eq!(meta[0].content_type.as_deref(), Some("image/png"));
    }

    #[test]
    fn extracts_size_bytes() {
        let r = make_record(
            "recSrc01",
            vec![(
                "Files",
                Value::Array(vec![attachment_value("attAbc01", "photo.png", false)]),
            )],
        );
        let meta = extract_attachment_metadata(&[r], &["Files"]);
        assert_eq!(meta[0].size_bytes, Some(1024));
    }

    #[test]
    fn skips_non_attachment_fields() {
        let r = make_record(
            "recSrc01",
            vec![
                ("Name", Value::String("Alice".to_string())),
                (
                    "Files",
                    Value::Array(vec![attachment_value("attAbc01", "photo.png", false)]),
                ),
            ],
        );
        let meta = extract_attachment_metadata(&[r], &["Files"]);
        assert_eq!(meta.len(), 1);
    }

    #[test]
    fn multiple_attachments_in_one_field() {
        let r = make_record(
            "recSrc01",
            vec![(
                "Files",
                Value::Array(vec![
                    attachment_value("att001", "a.png", false),
                    attachment_value("att002", "b.png", false),
                ]),
            )],
        );
        let meta = extract_attachment_metadata(&[r], &["Files"]);
        assert_eq!(meta.len(), 2);
    }

    #[test]
    fn empty_attachment_array_produces_no_entry() {
        let r = make_record("recSrc01", vec![("Files", Value::Array(vec![]))]);
        let meta = extract_attachment_metadata(&[r], &["Files"]);
        assert!(meta.is_empty());
    }

    #[test]
    fn attachment_metadata_to_jsonl_one_line_per_entry() {
        let entries = vec![
            AttachmentMetadata {
                record_id: "rec001".to_string(),
                field_name: "Files".to_string(),
                attachment_id: "att001".to_string(),
                filename: "a.png".to_string(),
                content_type: None,
                size_bytes: None,
                url_present: false,
            },
            AttachmentMetadata {
                record_id: "rec002".to_string(),
                field_name: "Files".to_string(),
                attachment_id: "att002".to_string(),
                filename: "b.png".to_string(),
                content_type: None,
                size_bytes: None,
                url_present: false,
            },
        ];
        let bytes = attachment_metadata_to_jsonl(&entries);
        let text = String::from_utf8(bytes).expect("utf8");
        assert_eq!(text.lines().count(), 2);
    }

    #[test]
    fn attachment_metadata_to_jsonl_each_line_is_valid_json() {
        let entries = vec![AttachmentMetadata {
            record_id: "rec001".to_string(),
            field_name: "Files".to_string(),
            attachment_id: "att001".to_string(),
            filename: "a.png".to_string(),
            content_type: Some("image/png".to_string()),
            size_bytes: Some(512),
            url_present: true,
        }];
        let bytes = attachment_metadata_to_jsonl(&entries);
        let text = String::from_utf8(bytes).expect("utf8");
        for line in text.lines() {
            let _: serde_json::Value = serde_json::from_str(line).expect("valid json");
        }
    }

    #[test]
    fn attachment_metadata_jsonl_does_not_contain_url() {
        let entries = vec![AttachmentMetadata {
            record_id: "rec001".to_string(),
            field_name: "Files".to_string(),
            attachment_id: "att001".to_string(),
            filename: "a.png".to_string(),
            content_type: None,
            size_bytes: None,
            url_present: true,
        }];
        let bytes = attachment_metadata_to_jsonl(&entries);
        let text = String::from_utf8(bytes).expect("utf8");
        assert!(!text.contains("https://"));
        assert!(!text.contains("dl.airtable.com"));
    }

    #[test]
    fn attachment_metadata_to_jsonl_empty_input() {
        let bytes = attachment_metadata_to_jsonl(&[]);
        assert!(bytes.is_empty());
    }

    #[test]
    fn jsonl_does_not_contain_token_sentinel() {
        const SENTINEL: &str = "pat_attach_sentinel_0123456789";
        let entries = vec![AttachmentMetadata {
            record_id: "rec001".to_string(),
            field_name: "Files".to_string(),
            attachment_id: "att001".to_string(),
            filename: "a.png".to_string(),
            content_type: None,
            size_bytes: None,
            url_present: false,
        }];
        let bytes = attachment_metadata_to_jsonl(&entries);
        let text = String::from_utf8(bytes).expect("utf8");
        assert!(!text.contains(SENTINEL));
    }
}
