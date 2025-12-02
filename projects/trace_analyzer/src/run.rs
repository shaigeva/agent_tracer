//! Run scenarios with coverage collection.
//!
//! This module provides functionality to execute pytest scenarios with
//! coverage enabled and capture the results.

use std::process::Command;

use serde::Serialize;

/// Result of running a scenario.
#[derive(Debug, Serialize)]
pub struct RunResult {
    /// Scenario ID that was run
    pub scenario_id: String,
    /// Whether the test passed
    pub passed: bool,
    /// Exit code from pytest
    pub exit_code: i32,
    /// Standard output from pytest
    pub stdout: String,
    /// Standard error from pytest
    pub stderr: String,
}

/// Run a pytest scenario with coverage enabled.
///
/// This executes pytest with:
/// - The specific test node ID
/// - Coverage collection enabled (`--cov`)
/// - Per-test coverage context (`--cov-context=test`)
///
/// # Arguments
/// * `scenario_id` - The pytest node ID (e.g., "tests/scenarios/test_auth.py::test_login")
///
/// # Returns
/// A `RunResult` with the test outcome and output.
pub fn run_scenario(scenario_id: &str) -> anyhow::Result<RunResult> {
    let output = Command::new("pytest")
        .arg(scenario_id)
        .arg("--cov")
        .arg("--cov-context=test")
        .arg("-v")
        .output()?;

    let exit_code = output.status.code().unwrap_or(-1);
    let passed = output.status.success();
    let stdout = String::from_utf8_lossy(&output.stdout).to_string();
    let stderr = String::from_utf8_lossy(&output.stderr).to_string();

    Ok(RunResult {
        scenario_id: scenario_id.to_string(),
        passed,
        exit_code,
        stdout,
        stderr,
    })
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_run_result_serialization() {
        let result = RunResult {
            scenario_id: "tests/test_foo.py::test_bar".to_string(),
            passed: true,
            exit_code: 0,
            stdout: "PASSED".to_string(),
            stderr: "".to_string(),
        };

        let json = serde_json::to_string(&result).unwrap();
        assert!(json.contains("test_foo.py"));
        assert!(json.contains("\"passed\":true"));
    }
}
