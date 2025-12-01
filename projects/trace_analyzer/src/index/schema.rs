//! Index database schema and access.
//!
//! The index is a SQLite database containing:
//! - scenarios: Scenario metadata
//! - scenario_behaviors: Many-to-many behavior tags
//! - coverage: Per-scenario line coverage
//! - functions: Function definitions from Python AST (future)

use std::path::Path;

use rusqlite::{Connection, OpenFlags};
use thiserror::Error;

/// Errors that can occur when working with the index.
#[derive(Error, Debug)]
pub enum IndexError {
    #[error("Failed to create index directory: {0}")]
    CreateDir(#[from] std::io::Error),

    #[error("Database error: {0}")]
    Database(#[from] rusqlite::Error),

    #[error("Index not found at {path}")]
    NotFound { path: String },

    #[error("Scenario not found: {id}")]
    ScenarioNotFound { id: String },
}

/// Handle to the index database.
pub struct Index {
    conn: Connection,
}

impl Index {
    /// Create a new index at the given path.
    ///
    /// This will create the directory if it doesn't exist and initialize
    /// a new SQLite database with the index schema.
    pub fn create(index_dir: &Path) -> Result<Self, IndexError> {
        // Create directory if needed
        std::fs::create_dir_all(index_dir)?;

        let db_path = index_dir.join("index.db");
        let conn = Connection::open(&db_path)?;

        let index = Self { conn };
        index.init_schema()?;

        Ok(index)
    }

    /// Open an existing index.
    pub fn open(index_dir: &Path) -> Result<Self, IndexError> {
        let db_path = index_dir.join("index.db");

        if !db_path.exists() {
            return Err(IndexError::NotFound {
                path: db_path.display().to_string(),
            });
        }

        let conn = Connection::open_with_flags(&db_path, OpenFlags::SQLITE_OPEN_READ_WRITE)?;

        Ok(Self { conn })
    }

    /// Open an existing index in read-only mode.
    pub fn open_readonly(index_dir: &Path) -> Result<Self, IndexError> {
        let db_path = index_dir.join("index.db");

        if !db_path.exists() {
            return Err(IndexError::NotFound {
                path: db_path.display().to_string(),
            });
        }

        let conn = Connection::open_with_flags(&db_path, OpenFlags::SQLITE_OPEN_READ_ONLY)?;

        Ok(Self { conn })
    }

    /// Initialize the database schema.
    fn init_schema(&self) -> Result<(), IndexError> {
        self.conn.execute_batch(
            r#"
            -- Scenarios with their metadata
            CREATE TABLE IF NOT EXISTS scenarios (
                id TEXT PRIMARY KEY,
                file TEXT NOT NULL,
                function TEXT NOT NULL,
                description TEXT NOT NULL,
                documentation TEXT,
                outcome TEXT NOT NULL
            );

            -- Behavior tags (many-to-many)
            CREATE TABLE IF NOT EXISTS scenario_behaviors (
                scenario_id TEXT NOT NULL REFERENCES scenarios(id) ON DELETE CASCADE,
                behavior TEXT NOT NULL,
                PRIMARY KEY (scenario_id, behavior)
            );

            -- Coverage per scenario (each row is a covered line)
            CREATE TABLE IF NOT EXISTS coverage (
                scenario_id TEXT NOT NULL REFERENCES scenarios(id) ON DELETE CASCADE,
                file_path TEXT NOT NULL,
                line_number INTEGER NOT NULL,
                PRIMARY KEY (scenario_id, file_path, line_number)
            );

            -- Function definitions from AST (future)
            CREATE TABLE IF NOT EXISTS functions (
                file_path TEXT NOT NULL,
                name TEXT NOT NULL,
                start_line INTEGER NOT NULL,
                end_line INTEGER NOT NULL,
                docstring TEXT,
                PRIMARY KEY (file_path, name, start_line)
            );

            -- Indexes for query performance
            CREATE INDEX IF NOT EXISTS idx_coverage_file_line
                ON coverage(file_path, line_number);
            CREATE INDEX IF NOT EXISTS idx_behaviors_behavior
                ON scenario_behaviors(behavior);
            CREATE INDEX IF NOT EXISTS idx_scenarios_outcome
                ON scenarios(outcome);
            CREATE INDEX IF NOT EXISTS idx_functions_file
                ON functions(file_path);
            "#,
        )?;

        Ok(())
    }

    /// Clear all data from the index (but keep schema).
    pub fn clear(&self) -> Result<(), IndexError> {
        self.conn.execute_batch(
            r#"
            DELETE FROM coverage;
            DELETE FROM scenario_behaviors;
            DELETE FROM scenarios;
            DELETE FROM functions;
            "#,
        )?;
        Ok(())
    }

    /// Get a reference to the underlying connection for queries.
    pub fn connection(&self) -> &Connection {
        &self.conn
    }

    /// Get the number of scenarios in the index.
    pub fn scenario_count(&self) -> Result<usize, IndexError> {
        let count: i64 = self
            .conn
            .query_row("SELECT COUNT(*) FROM scenarios", [], |row| row.get(0))?;
        Ok(count as usize)
    }

    /// Get the number of coverage entries in the index.
    pub fn coverage_count(&self) -> Result<usize, IndexError> {
        let count: i64 = self
            .conn
            .query_row("SELECT COUNT(*) FROM coverage", [], |row| row.get(0))?;
        Ok(count as usize)
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use tempfile::TempDir;

    #[test]
    fn test_create_index() {
        let temp_dir = TempDir::new().unwrap();
        let index_dir = temp_dir.path().join(".trace-index");

        let index = Index::create(&index_dir).unwrap();

        // Verify tables exist
        let tables: Vec<String> = index
            .conn
            .prepare("SELECT name FROM sqlite_master WHERE type='table' ORDER BY name")
            .unwrap()
            .query_map([], |row| row.get(0))
            .unwrap()
            .filter_map(|r| r.ok())
            .collect();

        assert!(tables.contains(&"scenarios".to_string()));
        assert!(tables.contains(&"scenario_behaviors".to_string()));
        assert!(tables.contains(&"coverage".to_string()));
        assert!(tables.contains(&"functions".to_string()));
    }

    #[test]
    fn test_open_existing_index() {
        let temp_dir = TempDir::new().unwrap();
        let index_dir = temp_dir.path().join(".trace-index");

        // Create index
        let _index = Index::create(&index_dir).unwrap();

        // Open it again
        let index = Index::open(&index_dir).unwrap();
        assert_eq!(index.scenario_count().unwrap(), 0);
    }

    #[test]
    fn test_open_nonexistent_index() {
        let temp_dir = TempDir::new().unwrap();
        let index_dir = temp_dir.path().join(".trace-index");

        let result = Index::open(&index_dir);
        assert!(matches!(result, Err(IndexError::NotFound { .. })));
    }

    #[test]
    fn test_clear_index() {
        let temp_dir = TempDir::new().unwrap();
        let index_dir = temp_dir.path().join(".trace-index");

        let index = Index::create(&index_dir).unwrap();

        // Insert some data
        index
            .conn
            .execute(
                "INSERT INTO scenarios (id, file, function, description, outcome)
                 VALUES ('test::foo', 'test.py', 'foo', 'A test', 'success')",
                [],
            )
            .unwrap();

        assert_eq!(index.scenario_count().unwrap(), 1);

        // Clear
        index.clear().unwrap();
        assert_eq!(index.scenario_count().unwrap(), 0);
    }
}
