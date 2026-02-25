//! Module trait: register into Application. Shared by Rust and Python facades.

use crate::application::Application;
use crate::CoreError;

/// Module: register into app (commands, queries, routes).
pub trait Module {
    fn register_into(&mut self, app: &mut Application) -> Result<(), CoreError>;
}
