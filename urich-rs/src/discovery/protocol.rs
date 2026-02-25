//! Service Discovery implementations. Trait is in core::service_discovery.

use urich_core::ServiceDiscovery;

/// Discovery from static config (name -> URL map). Like Python StaticDiscovery.
#[derive(Clone, Default)]
pub struct StaticDiscovery {
    services: std::collections::HashMap<String, String>,
}

impl StaticDiscovery {
    pub fn new(services: std::collections::HashMap<String, String>) -> Self {
        Self { services }
    }

    /// Build from a slice of (name, url) pairs.
    pub fn from_slice(pairs: &[(&str, &str)]) -> Self {
        Self {
            services: pairs
                .iter()
                .map(|(k, v)| ((*k).to_string(), (*v).to_string()))
                .collect(),
        }
    }
}

impl ServiceDiscovery for StaticDiscovery {
    fn resolve(&self, service_name: &str) -> Vec<String> {
        self.services
            .get(service_name)
            .map(|u| vec![u.clone()])
            .unwrap_or_default()
    }
}
