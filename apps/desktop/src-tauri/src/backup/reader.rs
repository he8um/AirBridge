use std::io::Read;
use std::path::Path;

use crate::backup::checksums::{checksums_from_json, ChecksumMap};
use crate::backup::format::{PATH_CHECKSUMS, PATH_MANIFEST, PATH_SCHEMA, REQUIRED_ENTRIES};
use crate::backup::manifest::PackageManifest;

/// Error type for package read failures.
#[derive(Debug)]
pub struct ReadError(pub String);

impl std::fmt::Display for ReadError {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "package read error: {}", self.0)
    }
}

impl From<zip::result::ZipError> for ReadError {
    fn from(e: zip::result::ZipError) -> Self {
        ReadError(e.to_string())
    }
}

impl From<std::io::Error> for ReadError {
    fn from(e: std::io::Error) -> Self {
        ReadError(e.to_string())
    }
}

/// Opens a `.airbridge` package at `path` and provides read access to its entries.
///
/// Files are never extracted to disk; content is returned as `Vec<u8>`.
pub struct BackupPackageReader {
    archive: zip::ZipArchive<std::fs::File>,
}

impl BackupPackageReader {
    /// Opens the package at `path`.
    pub fn open(path: &Path) -> Result<Self, ReadError> {
        let file = std::fs::File::open(path).map_err(|e| ReadError(e.to_string()))?;
        let archive = zip::ZipArchive::new(file)?;
        Ok(BackupPackageReader { archive })
    }

    /// Returns the names of all entries in the archive.
    pub fn entry_names(&mut self) -> Vec<String> {
        (0..self.archive.len())
            .filter_map(|i| self.archive.by_index(i).ok().map(|e| e.name().to_string()))
            .collect()
    }

    /// Returns the number of entries in the archive.
    pub fn entry_count(&mut self) -> usize {
        self.archive.len()
    }

    /// Reads a named entry and returns its bytes. Returns an error if not found.
    pub fn read_entry(&mut self, name: &str) -> Result<Vec<u8>, ReadError> {
        let mut entry = self
            .archive
            .by_name(name)
            .map_err(|_| ReadError(format!("entry not found: {name}")))?;
        let mut buf = Vec::new();
        entry.read_to_end(&mut buf)?;
        Ok(buf)
    }

    /// Reads and deserializes `manifest.json`.
    pub fn read_manifest(&mut self) -> Result<PackageManifest, ReadError> {
        let data = self.read_entry(PATH_MANIFEST)?;
        serde_json::from_slice(&data).map_err(|e| ReadError(format!("manifest parse error: {e}")))
    }

    /// Reads `checksums/sha256.json` and returns the checksum map.
    pub fn read_checksums(&mut self) -> Result<ChecksumMap, ReadError> {
        let data = self.read_entry(PATH_CHECKSUMS)?;
        checksums_from_json(&data).map_err(ReadError)
    }

    /// Reads `schema.json` as raw bytes.
    pub fn read_schema(&mut self) -> Result<Vec<u8>, ReadError> {
        self.read_entry(PATH_SCHEMA)
    }

    /// Returns true if all required entries are present.
    pub fn has_required_entries(&mut self) -> bool {
        let names = self.entry_names();
        REQUIRED_ENTRIES
            .iter()
            .all(|req| names.contains(&req.to_string()))
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::backup::format::{FORMAT_NAME, FORMAT_VERSION};
    use crate::backup::package::{PackageInput, TableRecords};
    use crate::backup::writer::write_package;
    use tempfile::tempdir;

    fn write_minimal(dir: &tempfile::TempDir) -> std::path::PathBuf {
        let path = dir.path().join("test.airbridge");
        let input = PackageInput {
            manifest_json: serde_json::to_vec(&serde_json::json!({
                "format": FORMAT_NAME,
                "formatVersion": FORMAT_VERSION,
                "appVersion": "0.1.0",
                "createdAt": "2026-06-11T00:00:00Z",
                "source": {
                    "provider": "airtable",
                    "baseId": "appSyn01",
                    "baseName": "Synthetic"
                },
                "contents": {
                    "tables": 1, "fields": 2, "records": 2,
                    "linkedRecordRelationships": 0, "attachments": 0
                },
                "security": {
                    "containsRecordData": true,
                    "containsAttachmentUrls": false,
                    "encrypted": false,
                    "redactionsApplied": []
                },
                "package": {
                    "generatedByApp": "airbridge",
                    "packageId": "00000000-0000-0000-0000-000000000001"
                }
            }))
            .unwrap(),
            base_json: br#"{"baseId":"appSyn01"}"#.to_vec(),
            schema_json: br#"{"tables":[]}"#.to_vec(),
            backup_report_json: br#"{"status":"ok"}"#.to_vec(),
            tables: vec![TableRecords {
                table_id: "tblSyn01".to_string(),
                lines: vec![
                    r#"{"id":"rec001","fields":{"Name":"Alpha"}}"#.to_string(),
                    r#"{"id":"rec002","fields":{"Name":"Beta"}}"#.to_string(),
                ],
            }],
            ..Default::default()
        };
        write_package(&path, &input).expect("write");
        path
    }

    #[test]
    fn reader_opens_valid_package() {
        let dir = tempdir().expect("tempdir");
        let path = write_minimal(&dir);
        let _reader = BackupPackageReader::open(&path).expect("open");
    }

    #[test]
    fn reader_reads_manifest() {
        let dir = tempdir().expect("tempdir");
        let path = write_minimal(&dir);
        let mut reader = BackupPackageReader::open(&path).expect("open");
        let manifest = reader.read_manifest().expect("read manifest");
        assert_eq!(manifest.format, FORMAT_NAME);
        assert_eq!(manifest.format_version, FORMAT_VERSION);
    }

    #[test]
    fn reader_reads_checksums() {
        let dir = tempdir().expect("tempdir");
        let path = write_minimal(&dir);
        let mut reader = BackupPackageReader::open(&path).expect("open");
        let checksums = reader.read_checksums().expect("read checksums");
        assert!(!checksums.is_empty());
    }

    #[test]
    fn reader_entry_names_contains_manifest() {
        let dir = tempdir().expect("tempdir");
        let path = write_minimal(&dir);
        let mut reader = BackupPackageReader::open(&path).expect("open");
        let names = reader.entry_names();
        assert!(names.contains(&PATH_MANIFEST.to_string()));
    }

    #[test]
    fn reader_has_required_entries() {
        let dir = tempdir().expect("tempdir");
        let path = write_minimal(&dir);
        let mut reader = BackupPackageReader::open(&path).expect("open");
        assert!(reader.has_required_entries());
    }

    #[test]
    fn reader_reads_schema_bytes() {
        let dir = tempdir().expect("tempdir");
        let path = write_minimal(&dir);
        let mut reader = BackupPackageReader::open(&path).expect("open");
        let schema = reader.read_schema().expect("read schema");
        assert!(!schema.is_empty());
    }

    #[test]
    fn reader_entry_count_is_nonzero() {
        let dir = tempdir().expect("tempdir");
        let path = write_minimal(&dir);
        let mut reader = BackupPackageReader::open(&path).expect("open");
        assert!(reader.entry_count() > 0);
    }

    #[test]
    fn reader_missing_entry_returns_error() {
        let dir = tempdir().expect("tempdir");
        let path = write_minimal(&dir);
        let mut reader = BackupPackageReader::open(&path).expect("open");
        let result = reader.read_entry("does-not-exist.json");
        assert!(result.is_err());
    }

    #[test]
    fn reader_manifest_source_base_id_matches() {
        let dir = tempdir().expect("tempdir");
        let path = write_minimal(&dir);
        let mut reader = BackupPackageReader::open(&path).expect("open");
        let manifest = reader.read_manifest().expect("read manifest");
        assert_eq!(manifest.source.base_id, "appSyn01");
    }
}
