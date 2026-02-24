//! Service Discovery trait: resolve(service_name) -> URLs. Used by Application and discovery module.

/// How to resolve services by name. Implementations: static config, Consul, etc.
pub trait ServiceDiscovery: Send + Sync {
    /// Return list of URLs for the service.
    fn resolve(&self, service_name: &str) -> Vec<String>;
}
