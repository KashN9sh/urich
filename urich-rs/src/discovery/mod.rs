//! Discovery: ServiceDiscovery, DiscoveryModule. Like Python discovery/.

mod protocol;

pub use protocol::StaticDiscovery;

use urich_core::{Application, CoreError, Module, ServiceDiscovery};

/// Discovery as object: one adapter (static, or custom). Like Python DiscoveryModule.
/// Register via app.register(discovery). Available on Application via .discovery().
pub struct DiscoveryModule {
    adapter: Option<Box<dyn ServiceDiscovery>>,
}

impl DiscoveryModule {
    pub fn new() -> Self {
        Self { adapter: None }
    }

    /// Static config: service name -> URL.
    pub fn static_discovery(mut self, services: std::collections::HashMap<String, String>) -> Self {
        self.adapter = Some(Box::new(StaticDiscovery::new(services)));
        self
    }

    /// Build static discovery from (name, url) pairs.
    pub fn static_slice(mut self, pairs: &[(&str, &str)]) -> Self {
        self.adapter = Some(Box::new(StaticDiscovery::from_slice(pairs)));
        self
    }

    /// Use custom implementation.
    pub fn adapter(mut self, impl_: impl ServiceDiscovery + 'static) -> Self {
        self.adapter = Some(Box::new(impl_));
        self
    }
}

impl Default for DiscoveryModule {
    fn default() -> Self {
        Self::new()
    }
}

impl Module for DiscoveryModule {
    fn register_into(&mut self, app: &mut Application) -> Result<(), CoreError> {
        let adapter = self.adapter.take().unwrap_or_else(|| {
            Box::new(StaticDiscovery::default()) as Box<dyn ServiceDiscovery>
        });
        app.set_discovery(adapter);
        Ok(())
    }
}
