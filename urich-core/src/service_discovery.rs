//! Service discovery trait: resolve(service_name) -> URLs. Shared by Rust and Python facades.

/// How to resolve services by name. Implementations: static config, Consul, etc.
pub trait ServiceDiscovery: Send + Sync {
    fn resolve(&self, service_name: &str) -> Vec<String>;
}
