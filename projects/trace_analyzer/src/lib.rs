//! trace_analyzer - Core library for analyzing pytest coverage traces
//!
//! This library provides the core functionality for parsing coverage data,
//! scenario metadata, and building queryable indexes. It is designed to be
//! independent of the execution context (CLI, MCP, etc.).

pub mod coverage;
pub mod error;
pub mod index;
pub mod models;
pub mod scenarios;
