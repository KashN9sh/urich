//! DDD: commands, queries, domain module, handler-as-type.

pub mod command_handler;
pub mod commands;
pub mod domain_module;

pub use command_handler::{CommandHandler, QueryHandler};
pub use commands::{Command, Query};
pub use domain_module::DomainModule;
