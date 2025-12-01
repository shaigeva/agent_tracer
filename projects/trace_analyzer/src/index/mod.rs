//! Index building and storage.
//!
//! This module provides functionality to build a queryable SQLite index
//! from coverage data and scenario metadata.

pub mod builder;
pub mod schema;

pub use builder::IndexBuilder;
pub use schema::{Index, IndexError};
