//! Parser for pytest-cov's .coverage SQLite database.
//!
//! The .coverage file is a SQLite database with the following schema:
//! - `file`: Maps file IDs to paths
//! - `context`: Maps context IDs to test identifiers (e.g., "test_foo.py::test_bar|run")
//! - `line_bits`: Stores coverage as compressed bitmaps (numbits format)
//!
//! This module parses the database and extracts per-test coverage data.

use std::collections::HashMap;
use std::path::Path;

use rusqlite::{Connection, OpenFlags};

use crate::error::CoverageError;
use crate::models::{CoverageMetadata, FileCoverage, TestCoverage};

/// Parser for pytest-cov coverage databases.
pub struct CoverageParser {
    conn: Connection,
}

impl CoverageParser {
    /// Open a coverage database file.
    pub fn open(path: &Path) -> Result<Self, CoverageError> {
        if !path.exists() {
            return Err(CoverageError::NotFound {
                path: path.display().to_string(),
            });
        }

        let conn = Connection::open_with_flags(path, OpenFlags::SQLITE_OPEN_READ_ONLY)?;

        // Verify this is a valid coverage database
        let has_schema: bool = conn
            .query_row(
                "SELECT COUNT(*) > 0 FROM sqlite_master WHERE type='table' AND name='coverage_schema'",
                [],
                |row| row.get(0),
            )
            .map_err(|e| CoverageError::InvalidSchema(e.to_string()))?;

        if !has_schema {
            return Err(CoverageError::InvalidSchema(
                "Missing coverage_schema table".to_string(),
            ));
        }

        Ok(Self { conn })
    }

    /// Read coverage metadata from the database.
    pub fn read_metadata(&self) -> Result<CoverageMetadata, CoverageError> {
        let mut stmt = self.conn.prepare("SELECT key, value FROM meta")?;
        let rows = stmt.query_map([], |row| {
            Ok((row.get::<_, String>(0)?, row.get::<_, String>(1)?))
        })?;

        let mut has_arcs = false;
        let mut version = None;
        let mut when = None;

        for row in rows {
            let (key, value) = row?;
            match key.as_str() {
                "has_arcs" => has_arcs = value == "true",
                "version" => version = Some(value),
                "when" => when = Some(value),
                _ => {}
            }
        }

        Ok(CoverageMetadata {
            has_arcs,
            version,
            when,
        })
    }

    /// Read all file paths from the database.
    pub fn read_files(&self) -> Result<HashMap<i64, String>, CoverageError> {
        let mut stmt = self.conn.prepare("SELECT id, path FROM file")?;
        let rows = stmt.query_map([], |row| {
            Ok((row.get::<_, i64>(0)?, row.get::<_, String>(1)?))
        })?;

        let mut files = HashMap::new();
        for row in rows {
            let (id, path) = row?;
            files.insert(id, path);
        }
        Ok(files)
    }

    /// Read all test contexts from the database.
    ///
    /// Returns a map of context ID to test identifier.
    /// Filters out empty contexts and strips the "|run" suffix.
    pub fn read_contexts(&self) -> Result<HashMap<i64, String>, CoverageError> {
        let mut stmt = self.conn.prepare("SELECT id, context FROM context")?;
        let rows = stmt.query_map([], |row| {
            Ok((row.get::<_, i64>(0)?, row.get::<_, String>(1)?))
        })?;

        let mut contexts = HashMap::new();
        for row in rows {
            let (id, context) = row?;
            // Skip empty context (global coverage)
            if context.is_empty() {
                continue;
            }
            // Strip "|run" suffix if present
            let test_id = context.strip_suffix("|run").unwrap_or(&context).to_string();
            contexts.insert(id, test_id);
        }
        Ok(contexts)
    }

    /// Read all coverage data from the database.
    ///
    /// Returns coverage data grouped by test context.
    pub fn read_coverage(&self) -> Result<Vec<TestCoverage>, CoverageError> {
        let files = self.read_files()?;
        let contexts = self.read_contexts()?;

        // Read line_bits and group by context
        let mut stmt = self
            .conn
            .prepare("SELECT file_id, context_id, numbits FROM line_bits")?;
        let rows = stmt.query_map([], |row| {
            Ok((
                row.get::<_, i64>(0)?,
                row.get::<_, i64>(1)?,
                row.get::<_, Vec<u8>>(2)?,
            ))
        })?;

        // Group coverage by context
        let mut coverage_by_context: HashMap<String, Vec<FileCoverage>> = HashMap::new();

        for row in rows {
            let (file_id, context_id, numbits) = row?;

            // Skip if we don't have the context (e.g., empty context)
            let Some(test_id) = contexts.get(&context_id) else {
                continue;
            };

            // Skip if we don't have the file
            let Some(file_path) = files.get(&file_id) else {
                continue;
            };

            let lines = decode_numbits(&numbits);

            coverage_by_context
                .entry(test_id.clone())
                .or_default()
                .push(FileCoverage {
                    path: file_path.clone(),
                    lines,
                });
        }

        // Convert to Vec<TestCoverage>
        let coverage: Vec<TestCoverage> = coverage_by_context
            .into_iter()
            .map(|(test_id, files)| TestCoverage { test_id, files })
            .collect();

        Ok(coverage)
    }
}

/// Decode coverage.py's numbits format to a sorted list of line numbers.
///
/// The numbits format is a byte array where each bit represents whether
/// a line is covered. The encoding is little-endian within each byte,
/// starting from line 0.
pub fn decode_numbits(numbits: &[u8]) -> Vec<u32> {
    let mut lines = Vec::new();
    for (byte_idx, &byte) in numbits.iter().enumerate() {
        for bit_idx in 0..8 {
            if byte & (1 << bit_idx) != 0 {
                let line_num = (byte_idx * 8 + bit_idx) as u32;
                lines.push(line_num);
            }
        }
    }
    lines
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_decode_numbits_empty() {
        assert_eq!(decode_numbits(&[]), Vec::<u32>::new());
    }

    #[test]
    fn test_decode_numbits_single_line() {
        // Line 0 is bit 0 of byte 0
        assert_eq!(decode_numbits(&[0x01]), vec![0]);
        // Line 7 is bit 7 of byte 0
        assert_eq!(decode_numbits(&[0x80]), vec![7]);
    }

    #[test]
    fn test_decode_numbits_multiple_lines_same_byte() {
        // Lines 0, 1, 2 are bits 0, 1, 2 of byte 0
        assert_eq!(decode_numbits(&[0x07]), vec![0, 1, 2]);
    }

    #[test]
    fn test_decode_numbits_multiple_bytes() {
        // Lines 0 (byte 0, bit 0) and 8 (byte 1, bit 0)
        assert_eq!(decode_numbits(&[0x01, 0x01]), vec![0, 8]);
    }

    #[test]
    fn test_decode_numbits_real_data() {
        // Real data from pytest-cov: bytes "00002031" hex
        // This should decode to lines [21, 24, 28, 29]
        let numbits = vec![0x00, 0x00, 0x20, 0x31];
        let lines = decode_numbits(&numbits);
        assert_eq!(lines, vec![21, 24, 28, 29]);
    }

    #[test]
    fn test_decode_numbits_sparse() {
        // Line 21 is byte 2 (21/8=2), bit 5 (21%8=5)
        // 0x20 = 0b00100000, bit 5 is set
        let mut numbits = vec![0x00, 0x00, 0x20];
        assert_eq!(decode_numbits(&numbits), vec![21]);

        // Add line 24: byte 3 (24/8=3), bit 0 (24%8=0)
        numbits.push(0x01);
        assert_eq!(decode_numbits(&numbits), vec![21, 24]);
    }
}
