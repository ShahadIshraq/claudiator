//! Library target that re-exports the hook's internal modules.
//!
//! This exists solely to make the modules accessible from integration tests
//! in `tests/`. The binary entry point remains `src/main.rs`; all production
//! code lives in the modules below.

pub mod cli;
pub mod config;
pub mod error;
pub mod event;
pub mod logger;
pub mod payload;
pub mod sender;
