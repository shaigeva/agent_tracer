//! Query functions for the trace index.
//!
//! This module provides the core query logic, independent of CLI/MCP concerns.
//! All functions take an Index reference and return data structures.

use crate::index::{Index, IndexError};
use crate::models::ScenarioOutcome;

/// A scenario with its metadata (returned from queries).
#[derive(Debug, Clone, serde::Serialize)]
pub struct ScenarioInfo {
    pub id: String,
    pub file: String,
    pub function: String,
    pub description: String,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub documentation: Option<String>,
    pub behaviors: Vec<String>,
    pub outcome: String,
}

/// Coverage context for a scenario.
#[derive(Debug, Clone, serde::Serialize)]
pub struct ScenarioContext {
    pub scenario: ScenarioInfo,
    pub coverage: Vec<FileCoverageInfo>,
}

/// Coverage info for a single file.
#[derive(Debug, Clone, serde::Serialize)]
pub struct FileCoverageInfo {
    pub path: String,
    pub lines: Vec<u32>,
}

/// A scenario that covers a specific file/line.
#[derive(Debug, Clone, serde::Serialize)]
pub struct AffectedScenario {
    pub scenario: ScenarioInfo,
    pub matching_lines: Vec<u32>,
}

/// List scenarios with optional filters.
pub fn list_scenarios(
    index: &Index,
    behavior: Option<&str>,
    errors_only: bool,
) -> Result<Vec<ScenarioInfo>, IndexError> {
    let conn = index.connection();

    let mut scenarios = Vec::new();

    // Build query based on filters
    let base_query = if let Some(behavior) = behavior {
        // Filter by behavior
        let mut stmt = conn.prepare(
            "SELECT DISTINCT s.id, s.file, s.function, s.description, s.documentation, s.outcome
             FROM scenarios s
             JOIN scenario_behaviors sb ON s.id = sb.scenario_id
             WHERE sb.behavior = ?1
             ORDER BY s.id",
        )?;

        let rows = stmt.query_map([behavior], |row| {
            Ok((
                row.get::<_, String>(0)?,
                row.get::<_, String>(1)?,
                row.get::<_, String>(2)?,
                row.get::<_, String>(3)?,
                row.get::<_, Option<String>>(4)?,
                row.get::<_, String>(5)?,
            ))
        })?;

        for row in rows {
            let (id, file, function, description, documentation, outcome) = row?;
            if errors_only && outcome != "error" {
                continue;
            }
            scenarios.push(ScenarioInfo {
                id: id.clone(),
                file,
                function,
                description,
                documentation,
                behaviors: get_behaviors(conn, &id)?,
                outcome,
            });
        }
        scenarios
    } else {
        // No behavior filter
        let query = if errors_only {
            "SELECT id, file, function, description, documentation, outcome
             FROM scenarios WHERE outcome = 'error' ORDER BY id"
        } else {
            "SELECT id, file, function, description, documentation, outcome
             FROM scenarios ORDER BY id"
        };

        let mut stmt = conn.prepare(query)?;
        let rows = stmt.query_map([], |row| {
            Ok((
                row.get::<_, String>(0)?,
                row.get::<_, String>(1)?,
                row.get::<_, String>(2)?,
                row.get::<_, String>(3)?,
                row.get::<_, Option<String>>(4)?,
                row.get::<_, String>(5)?,
            ))
        })?;

        for row in rows {
            let (id, file, function, description, documentation, outcome) = row?;
            scenarios.push(ScenarioInfo {
                id: id.clone(),
                file,
                function,
                description,
                documentation,
                behaviors: get_behaviors(conn, &id)?,
                outcome,
            });
        }
        scenarios
    };

    Ok(base_query)
}

/// Search scenarios by description text.
pub fn search_scenarios(index: &Index, query: &str) -> Result<Vec<ScenarioInfo>, IndexError> {
    let conn = index.connection();

    // Simple LIKE search on description and documentation
    let search_pattern = format!("%{}%", query);

    let mut stmt = conn.prepare(
        "SELECT id, file, function, description, documentation, outcome
         FROM scenarios
         WHERE description LIKE ?1 OR documentation LIKE ?1
         ORDER BY id",
    )?;

    let rows = stmt.query_map([&search_pattern], |row| {
        Ok((
            row.get::<_, String>(0)?,
            row.get::<_, String>(1)?,
            row.get::<_, String>(2)?,
            row.get::<_, String>(3)?,
            row.get::<_, Option<String>>(4)?,
            row.get::<_, String>(5)?,
        ))
    })?;

    let mut scenarios = Vec::new();
    for row in rows {
        let (id, file, function, description, documentation, outcome) = row?;
        scenarios.push(ScenarioInfo {
            id: id.clone(),
            file,
            function,
            description,
            documentation,
            behaviors: get_behaviors(conn, &id)?,
            outcome,
        });
    }

    Ok(scenarios)
}

/// Get full coverage context for a scenario.
pub fn get_scenario_context(
    index: &Index,
    scenario_id: &str,
) -> Result<ScenarioContext, IndexError> {
    let conn = index.connection();

    // Get scenario info
    let scenario: ScenarioInfo = conn
        .query_row(
            "SELECT id, file, function, description, documentation, outcome
             FROM scenarios WHERE id = ?1",
            [scenario_id],
            |row| {
                Ok(ScenarioInfo {
                    id: row.get(0)?,
                    file: row.get(1)?,
                    function: row.get(2)?,
                    description: row.get(3)?,
                    documentation: row.get(4)?,
                    behaviors: Vec::new(), // Will fill in below
                    outcome: row.get(5)?,
                })
            },
        )
        .map_err(|e| match e {
            rusqlite::Error::QueryReturnedNoRows => IndexError::ScenarioNotFound {
                id: scenario_id.to_string(),
            },
            _ => IndexError::Database(e),
        })?;

    // Get behaviors
    let behaviors = get_behaviors(conn, scenario_id)?;
    let scenario = ScenarioInfo {
        behaviors,
        ..scenario
    };

    // Get coverage grouped by file
    let mut stmt = conn.prepare(
        "SELECT file_path, line_number FROM coverage
         WHERE scenario_id = ?1
         ORDER BY file_path, line_number",
    )?;

    let rows = stmt.query_map([scenario_id], |row| {
        Ok((row.get::<_, String>(0)?, row.get::<_, u32>(1)?))
    })?;

    // Group by file
    let mut coverage_map: std::collections::HashMap<String, Vec<u32>> =
        std::collections::HashMap::new();
    for row in rows {
        let (path, line) = row?;
        coverage_map.entry(path).or_default().push(line);
    }

    let coverage: Vec<FileCoverageInfo> = coverage_map
        .into_iter()
        .map(|(path, lines)| FileCoverageInfo { path, lines })
        .collect();

    Ok(ScenarioContext { scenario, coverage })
}

/// Find scenarios that cover a specific file or line.
pub fn find_affected_scenarios(
    index: &Index,
    file_path: &str,
    line: Option<u32>,
) -> Result<Vec<AffectedScenario>, IndexError> {
    let conn = index.connection();

    // Normalize the file path for matching (handle both absolute and relative)
    let file_pattern = format!("%{}", file_path);

    let mut affected = Vec::new();

    if let Some(line_num) = line {
        // Find scenarios covering specific line
        let mut stmt = conn.prepare(
            "SELECT DISTINCT c.scenario_id, c.line_number
             FROM coverage c
             WHERE c.file_path LIKE ?1 AND c.line_number = ?2",
        )?;

        let rows = stmt.query_map((&file_pattern, line_num), |row| {
            Ok((row.get::<_, String>(0)?, row.get::<_, u32>(1)?))
        })?;

        let mut scenario_lines: std::collections::HashMap<String, Vec<u32>> =
            std::collections::HashMap::new();
        for row in rows {
            let (scenario_id, line) = row?;
            scenario_lines.entry(scenario_id).or_default().push(line);
        }

        for (scenario_id, lines) in scenario_lines {
            let context = get_scenario_context(index, &scenario_id)?;
            affected.push(AffectedScenario {
                scenario: context.scenario,
                matching_lines: lines,
            });
        }
    } else {
        // Find scenarios covering any line in file
        let mut stmt = conn.prepare(
            "SELECT DISTINCT c.scenario_id, c.line_number
             FROM coverage c
             WHERE c.file_path LIKE ?1
             ORDER BY c.scenario_id, c.line_number",
        )?;

        let rows = stmt.query_map([&file_pattern], |row| {
            Ok((row.get::<_, String>(0)?, row.get::<_, u32>(1)?))
        })?;

        let mut scenario_lines: std::collections::HashMap<String, Vec<u32>> =
            std::collections::HashMap::new();
        for row in rows {
            let (scenario_id, line) = row?;
            scenario_lines.entry(scenario_id).or_default().push(line);
        }

        for (scenario_id, lines) in scenario_lines {
            let context = get_scenario_context(index, &scenario_id)?;
            affected.push(AffectedScenario {
                scenario: context.scenario,
                matching_lines: lines,
            });
        }
    }

    Ok(affected)
}

/// Helper to get behaviors for a scenario.
fn get_behaviors(
    conn: &rusqlite::Connection,
    scenario_id: &str,
) -> Result<Vec<String>, rusqlite::Error> {
    let mut stmt = conn.prepare(
        "SELECT behavior FROM scenario_behaviors WHERE scenario_id = ?1 ORDER BY behavior",
    )?;
    let rows = stmt.query_map([scenario_id], |row| row.get(0))?;

    let mut behaviors = Vec::new();
    for row in rows {
        behaviors.push(row?);
    }
    Ok(behaviors)
}

/// Parse a target string like "file.py" or "file.py:25" into (path, optional line).
pub fn parse_target(target: &str) -> (String, Option<u32>) {
    if let Some(idx) = target.rfind(':') {
        let (path, line_str) = target.split_at(idx);
        if let Ok(line) = line_str[1..].parse::<u32>() {
            return (path.to_string(), Some(line));
        }
    }
    (target.to_string(), None)
}

impl From<ScenarioOutcome> for String {
    fn from(outcome: ScenarioOutcome) -> Self {
        match outcome {
            ScenarioOutcome::Success => "success".to_string(),
            ScenarioOutcome::Error => "error".to_string(),
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_parse_target_file_only() {
        let (path, line) = parse_target("src/auth.py");
        assert_eq!(path, "src/auth.py");
        assert_eq!(line, None);
    }

    #[test]
    fn test_parse_target_with_line() {
        let (path, line) = parse_target("src/auth.py:25");
        assert_eq!(path, "src/auth.py");
        assert_eq!(line, Some(25));
    }

    #[test]
    fn test_parse_target_with_colon_in_path() {
        // Windows-style path or other edge case
        let (path, line) = parse_target("C:/Users/test/file.py:10");
        assert_eq!(path, "C:/Users/test/file.py");
        assert_eq!(line, Some(10));
    }

    #[test]
    fn test_parse_target_invalid_line() {
        let (path, line) = parse_target("src/auth.py:abc");
        assert_eq!(path, "src/auth.py:abc");
        assert_eq!(line, None);
    }
}
