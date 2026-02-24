//! Module: register into app. Same idea as Python DomainModule.

use urich_core::CoreError;

use super::app::Application;

/// Module: register into app (commands, queries). Same idea as Python DomainModule.
pub trait Module {
    fn register_into(&mut self, app: &mut Application) -> Result<(), CoreError>;
}
