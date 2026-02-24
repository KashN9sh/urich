//! OutboxModule: configure storage and publisher. Like Python events/outbox.

use urich_core::CoreError;

use crate::core::app::Application;
use crate::core::Module;

use super::protocol::{OutboxPublisher, OutboxStorage};

/// Outbox building block: configure via .storage(...) and .publisher(...). Like Python OutboxModule.
pub struct OutboxModule {
    storage: Option<Box<dyn OutboxStorage>>,
    publisher: Option<Box<dyn OutboxPublisher>>,
}

impl OutboxModule {
    pub fn new() -> Self {
        Self {
            storage: None,
            publisher: None,
        }
    }

    pub fn storage(mut self, impl_: Box<dyn OutboxStorage>) -> Self {
        self.storage = Some(impl_);
        self
    }

    pub fn publisher(mut self, impl_: Box<dyn OutboxPublisher>) -> Self {
        self.publisher = Some(impl_);
        self
    }
}

impl Default for OutboxModule {
    fn default() -> Self {
        Self::new()
    }
}

impl Module for OutboxModule {
    fn register_into(&mut self, app: &mut Application) -> Result<(), CoreError> {
        if let Some(s) = self.storage.take() {
            app.set_outbox_storage(s);
        }
        if let Some(p) = self.publisher.take() {
            app.set_outbox_publisher(p);
        }
        Ok(())
    }
}
