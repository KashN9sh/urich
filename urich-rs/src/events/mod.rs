//! Events: EventBusModule, Outbox. Like Python events/.

mod outbox;
pub mod protocol;

pub use outbox::OutboxModule;
pub use protocol::{EventBusAdapter, OutboxPublisher, OutboxStorage};

use urich_core::CoreError;

use crate::core::app::Application;
use crate::core::Module;

/// Event bus as object. In Rust the default event bus is inside Application (subscribe_event / publish_event).
/// This module is a no-op placeholder for API consistency with Python; optionally register custom adapter in container.
pub struct EventBusModule;

impl EventBusModule {
    pub fn new() -> Self {
        Self
    }

    /// Use custom adapter (register in container as EventBusAdapter).
    pub fn adapter(self, impl_: Box<dyn EventBusAdapter>) -> EventBusModuleWithAdapter {
        EventBusModuleWithAdapter { adapter: Some(impl_) }
    }
}

impl Default for EventBusModule {
    fn default() -> Self {
        Self::new()
    }
}

impl Module for EventBusModule {
    fn register_into(&mut self, _app: &mut Application) -> Result<(), CoreError> {
        Ok(())
    }
}

/// EventBusModule with an adapter to register (returned by .adapter()).
pub struct EventBusModuleWithAdapter {
    adapter: Option<Box<dyn EventBusAdapter>>,
}

impl Module for EventBusModuleWithAdapter {
    fn register_into(&mut self, app: &mut Application) -> Result<(), CoreError> {
        if let Some(a) = self.adapter.take() {
            app.container_mut().register_instance(a);
        }
        Ok(())
    }
}
