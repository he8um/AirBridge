use std::io::{self, Write};
use std::path::Path;

use zip::write::SimpleFileOptions;
use zip::CompressionMethod;

use crate::backup::checksums::{checksums_to_json, sha256_hex, ChecksumMap};
use crate::backup::format::{
    PATH_ATTACHMENT_METADATA, PATH_BACKUP_REPORT, PATH_BASE, PATH_CHECKSUMS,
    PATH_COMPATIBILITY_REPORT, PATH_LINKED_RECORDS, PATH_MANIFEST, PATH_SCHEMA, PATH_TABLES_PREFIX,
};
use crate::backup::package::PackageInput;

/// Error type for package write failures.
#[derive(Debug)]
pub struct WriteError(pub String);

impl std::fmt::Display for WriteError {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "package write error: {}", self.0)
    }
}

impl From<io::Error> for WriteError {
    fn from(e: io::Error) -> Self {
        WriteError(e.to_string())
    }
}

impl From<zip::result::ZipError> for WriteError {
    fn from(e: zip::result::ZipError) -> Self {
        WriteError(e.to_string())
    }
}

/// Writes a `.airbridge` ZIP package to `dest_path` from a `PackageInput`.
///
/// - Computes SHA-256 checksums for every written entry.
/// - Writes `checksums/sha256.json` as the final entry.
/// - Never embeds absolute local filesystem paths inside the archive.
/// - Never embeds tokens or secrets.
pub fn write_package(dest_path: &Path, input: &PackageInput) -> Result<(), WriteError> {
    if !input.is_complete() {
        return Err(WriteError(
            "incomplete package input: manifest, base, schema, and backup report are required"
                .to_string(),
        ));
    }

    let file = std::fs::File::create(dest_path)?;
    let mut zip = zip::ZipWriter::new(file);
    let opts = SimpleFileOptions::default().compression_method(CompressionMethod::Deflated);

    let mut checksums = ChecksumMap::new();

    macro_rules! write_entry {
        ($path:expr, $data:expr) => {{
            let data: &[u8] = $data;
            zip.start_file($path, opts)?;
            zip.write_all(data)?;
            checksums.insert($path.to_string(), sha256_hex(data));
        }};
    }

    write_entry!(PATH_MANIFEST, &input.manifest_json);
    write_entry!(PATH_BASE, &input.base_json);
    write_entry!(PATH_SCHEMA, &input.schema_json);

    // Per-table records JSONL
    for table in &input.tables {
        let entry_path = format!("{}{}/records.jsonl", PATH_TABLES_PREFIX, table.table_id);
        let data = table.lines.join("\n").into_bytes();
        zip.start_file(&entry_path, opts)?;
        zip.write_all(&data)?;
        checksums.insert(entry_path, sha256_hex(&data));
    }

    // Optional but encouraged entries
    if !input.attachment_metadata_jsonl.is_empty() {
        write_entry!(PATH_ATTACHMENT_METADATA, &input.attachment_metadata_jsonl);
    }
    if !input.linked_records_jsonl.is_empty() {
        write_entry!(PATH_LINKED_RECORDS, &input.linked_records_jsonl);
    }

    write_entry!(PATH_BACKUP_REPORT, &input.backup_report_json);

    if !input.compatibility_report_json.is_empty() {
        write_entry!(PATH_COMPATIBILITY_REPORT, &input.compatibility_report_json);
    }

    // Checksums entry written last so it covers all preceding entries.
    let checksum_json = checksums_to_json(&checksums);
    zip.start_file(PATH_CHECKSUMS, opts)?;
    zip.write_all(&checksum_json)?;

    zip.finish()?;
    Ok(())
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::backup::checksums::checksums_from_json;
    use crate::backup::format::REQUIRED_ENTRIES;
    use crate::backup::package::TableRecords;
    use std::io::Read;
    use tempfile::tempdir;

    fn minimal_input() -> PackageInput {
        PackageInput {
            manifest_json: br#"{"format":"airbridge","formatVersion":"0.1.0","appVersion":"0.1.0","createdAt":"2026-06-11T00:00:00Z","source":{"provider":"airtable","baseId":"appSyn01","baseName":"Synthetic"},"contents":{"tables":1,"fields":2,"records":3,"linkedRecordRelationships":0,"attachments":0},"security":{"containsRecordData":true,"containsAttachmentUrls":false,"encrypted":false,"redactionsApplied":[]},"package":{"generatedByApp":"airbridge","packageId":"00000000-0000-0000-0000-000000000001"}}"#.to_vec(),
            base_json: br#"{"baseId":"appSyn01","baseName":"Synthetic"}"#.to_vec(),
            schema_json: br#"{"tables":[{"id":"tblSyn01","name":"Items","fields":[]}]}"#.to_vec(),
            backup_report_json: br#"{"status":"ok","tableCount":1,"recordCount":3}"#.to_vec(),
            tables: vec![TableRecords {
                table_id: "tblSyn01".to_string(),
                lines: vec![
                    r#"{"id":"rec001","fields":{"Name":"Alpha"}}"#.to_string(),
                    r#"{"id":"rec002","fields":{"Name":"Beta"}}"#.to_string(),
                    r#"{"id":"rec003","fields":{"Name":"Gamma"}}"#.to_string(),
                ],
            }],
            ..Default::default()
        }
    }

    fn open_zip(path: &Path) -> zip::ZipArchive<std::fs::File> {
        let f = std::fs::File::open(path).expect("open zip");
        zip::ZipArchive::new(f).expect("parse zip")
    }

    fn read_zip_entry(archive: &mut zip::ZipArchive<std::fs::File>, name: &str) -> Vec<u8> {
        let mut entry = archive.by_name(name).expect("entry not found");
        let mut buf = Vec::new();
        entry.read_to_end(&mut buf).expect("read entry");
        buf
    }

    #[test]
    fn writer_creates_package_file() {
        let dir = tempdir().expect("tempdir");
        let path = dir.path().join("test.airbridge");
        write_package(&path, &minimal_input()).expect("write");
        assert!(path.exists(), "package file should exist");
    }

    #[test]
    fn writer_package_is_valid_zip() {
        let dir = tempdir().expect("tempdir");
        let path = dir.path().join("test.airbridge");
        write_package(&path, &minimal_input()).expect("write");
        let _archive = open_zip(&path);
    }

    #[test]
    fn writer_includes_manifest_entry() {
        let dir = tempdir().expect("tempdir");
        let path = dir.path().join("test.airbridge");
        write_package(&path, &minimal_input()).expect("write");
        let mut archive = open_zip(&path);
        let data = read_zip_entry(&mut archive, PATH_MANIFEST);
        assert!(!data.is_empty());
    }

    #[test]
    fn writer_includes_all_required_entries() {
        let dir = tempdir().expect("tempdir");
        let path = dir.path().join("test.airbridge");
        write_package(&path, &minimal_input()).expect("write");
        let mut archive = open_zip(&path);
        let names: Vec<String> = (0..archive.len())
            .map(|i| archive.by_index(i).unwrap().name().to_string())
            .collect();
        for req in REQUIRED_ENTRIES {
            assert!(
                names.contains(&req.to_string()),
                "required entry '{}' missing",
                req
            );
        }
    }

    #[test]
    fn writer_includes_checksum_entry() {
        let dir = tempdir().expect("tempdir");
        let path = dir.path().join("test.airbridge");
        write_package(&path, &minimal_input()).expect("write");
        let mut archive = open_zip(&path);
        let data = read_zip_entry(&mut archive, PATH_CHECKSUMS);
        assert!(!data.is_empty());
    }

    #[test]
    fn writer_checksums_are_valid_json() {
        let dir = tempdir().expect("tempdir");
        let path = dir.path().join("test.airbridge");
        write_package(&path, &minimal_input()).expect("write");
        let mut archive = open_zip(&path);
        let data = read_zip_entry(&mut archive, PATH_CHECKSUMS);
        let map = checksums_from_json(&data).expect("parse checksums");
        assert!(!map.is_empty());
    }

    #[test]
    fn writer_checksums_cover_manifest() {
        let dir = tempdir().expect("tempdir");
        let path = dir.path().join("test.airbridge");
        write_package(&path, &minimal_input()).expect("write");
        let mut archive = open_zip(&path);
        let checksum_data = read_zip_entry(&mut archive, PATH_CHECKSUMS);
        let map = checksums_from_json(&checksum_data).expect("parse checksums");
        assert!(
            map.contains_key(PATH_MANIFEST),
            "checksums should include manifest.json"
        );
    }

    #[test]
    fn writer_table_records_are_present() {
        let dir = tempdir().expect("tempdir");
        let path = dir.path().join("test.airbridge");
        write_package(&path, &minimal_input()).expect("write");
        let mut archive = open_zip(&path);
        let entry_path = format!("{}{}/records.jsonl", PATH_TABLES_PREFIX, "tblSyn01");
        let data = read_zip_entry(&mut archive, &entry_path);
        let text = String::from_utf8(data).expect("utf8");
        let lines: Vec<&str> = text.lines().collect();
        assert_eq!(lines.len(), 3, "should have 3 records");
        assert!(lines[0].contains("Alpha"));
        assert!(lines[1].contains("Beta"));
        assert!(lines[2].contains("Gamma"));
    }

    #[test]
    fn writer_does_not_embed_absolute_paths() {
        let dir = tempdir().expect("tempdir");
        let path = dir.path().join("test.airbridge");
        write_package(&path, &minimal_input()).expect("write");
        let mut archive = open_zip(&path);
        for i in 0..archive.len() {
            let entry = archive.by_index(i).expect("entry");
            let name = entry.name();
            assert!(
                !name.starts_with('/'),
                "entry '{}' must not start with /",
                name
            );
            assert!(
                !name.contains("Users/"),
                "entry '{}' must not contain absolute user path",
                name
            );
            assert!(
                !name.contains("home/"),
                "entry '{}' must not contain absolute home path",
                name
            );
        }
    }

    #[test]
    fn writer_package_does_not_contain_token_sentinel() {
        const SENTINEL: &str = "pat_writer_test_sentinel_0123456789";
        let dir = tempdir().expect("tempdir");
        let path = dir.path().join("test.airbridge");
        write_package(&path, &minimal_input()).expect("write");
        let bytes = std::fs::read(&path).expect("read file");
        assert!(!bytes
            .windows(SENTINEL.len())
            .any(|w| w == SENTINEL.as_bytes()));
    }

    #[test]
    fn writer_rejects_incomplete_input() {
        let dir = tempdir().expect("tempdir");
        let path = dir.path().join("test.airbridge");
        let result = write_package(&path, &PackageInput::default());
        assert!(result.is_err(), "incomplete input should fail");
    }

    #[test]
    fn writer_manifest_content_is_preserved() {
        let dir = tempdir().expect("tempdir");
        let path = dir.path().join("test.airbridge");
        let input = minimal_input();
        let expected_manifest = input.manifest_json.clone();
        write_package(&path, &input).expect("write");
        let mut archive = open_zip(&path);
        let data = read_zip_entry(&mut archive, PATH_MANIFEST);
        assert_eq!(data, expected_manifest);
    }

    #[test]
    fn writer_checksum_values_match_entry_content() {
        let dir = tempdir().expect("tempdir");
        let path = dir.path().join("test.airbridge");
        let input = minimal_input();
        write_package(&path, &input).expect("write");
        let mut archive = open_zip(&path);
        let checksum_data = read_zip_entry(&mut archive, PATH_CHECKSUMS);
        let map = checksums_from_json(&checksum_data).expect("parse checksums");

        let manifest_data = read_zip_entry(&mut archive, PATH_MANIFEST);
        let expected_hash = sha256_hex(&manifest_data);
        assert_eq!(map[PATH_MANIFEST], expected_hash);
    }

    #[test]
    fn writer_attachment_metadata_included_when_provided() {
        let dir = tempdir().expect("tempdir");
        let path = dir.path().join("test.airbridge");
        let mut input = minimal_input();
        input.attachment_metadata_jsonl =
            br#"{"fieldId":"fld01","url":"https://example.com/file.pdf","size":1024}"#.to_vec();
        write_package(&path, &input).expect("write");
        let mut archive = open_zip(&path);
        let data = read_zip_entry(&mut archive, PATH_ATTACHMENT_METADATA);
        assert!(!data.is_empty());
    }
}
