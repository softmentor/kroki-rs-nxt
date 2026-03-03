//! kroki-core: Core domain logic for the kroki diagram platform.
//!
//! This crate contains pure business logic with zero infrastructure dependencies:
//! - `DiagramProvider` trait (the central port)
//! - `DiagramRegistry` for provider discovery and lookup
//! - Domain types: `DiagramRequest`, `DiagramResponse`, `DiagramError`
//! - Provider implementations: Command, Browser, Pipeline
//! - Configuration model
//! - Utility functions (decode, image conversion, font management)

pub mod config;
pub mod error;
pub mod ports;
pub mod providers;
pub mod services;
pub mod utils;

pub use error::{DiagramError, DiagramResult};
pub use ports::DiagramProvider;
pub use services::DiagramRegistry;
