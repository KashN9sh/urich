//! DDD: commands, queries, domain module.

pub mod commands;
pub mod domain_module;

pub use commands::{Command, Query};
pub use domain_module::DomainModule;
