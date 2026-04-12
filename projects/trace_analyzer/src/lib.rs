//! trace_analyzer - Core library for analyzing pytest coverage traces
//!
//! This library provides the core functionality for parsing coverage data,
//! scenario metadata, and building queryable indexes. It is designed to be
//! independent of the execution context (CLI, MCP, etc.).

pub mod call_trace;
pub mod coverage;
pub mod diagram;
pub mod error;
pub mod gallery;
pub mod index;
pub mod mcp;
pub mod models;
pub mod query;
pub mod run;
pub mod scenarios;
