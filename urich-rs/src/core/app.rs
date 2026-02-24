//! Application: registers routes with core and dispatches to Rust handlers.

use serde_json::Value;
use std::any::TypeId;
use std::collections::HashMap;
use urich_core::{App, CoreError as CoreErrorInner, RouteId};

use super::container::Container;
use super::outbox::{OutboxPublisher, OutboxStorage};
use super::service_discovery::ServiceDiscovery;

/// Handler: receives JSON value (validated), returns JSON value or error.
pub type Handler = Box<dyn Fn(Value) -> Result<Value, CoreErrorInner> + Send + Sync>;

/// Event handler: receives event as JSON value. Used by EventBus subscribe.
pub(crate) type EventHandler = Box<dyn Fn(Value) -> Result<(), CoreErrorInner> + Send + Sync>;

/// Application: registers routes with core and dispatches to Rust handlers; holds optional EventBus and Container.
pub struct Application {
    pub(crate) core: App,
    pub(crate) handlers: HashMap<RouteId, Handler>,
    pub(crate) callback_installed: bool,
    /// In-process event bus: type_id -> list of handlers.
    pub(crate) event_handlers: HashMap<TypeId, Vec<EventHandler>>,
    /// DI container for modules (Discovery, RPC, etc.).
    pub(crate) container: Container,
    /// Optional service discovery (set by DiscoveryModule).
    pub(crate) discovery: Option<Box<dyn ServiceDiscovery>>,
    /// Optional outbox storage (set by OutboxModule).
    pub(crate) outbox_storage: Option<Box<dyn OutboxStorage>>,
    /// Optional outbox publisher (set by OutboxModule).
    pub(crate) outbox_publisher: Option<Box<dyn OutboxPublisher>>,
}

impl Application {
    pub fn new() -> Self {
        Self {
            core: App::new(),
            handlers: HashMap::new(),
            callback_installed: false,
            event_handlers: HashMap::new(),
            container: Container::new(),
            discovery: None,
            outbox_storage: None,
            outbox_publisher: None,
        }
    }

    /// Set outbox storage (called by OutboxModule).
    pub fn set_outbox_storage(&mut self, s: Box<dyn OutboxStorage>) {
        self.outbox_storage = Some(s);
    }

    /// Set outbox publisher (called by OutboxModule).
    pub fn set_outbox_publisher(&mut self, p: Box<dyn OutboxPublisher>) {
        self.outbox_publisher = Some(p);
    }

    /// Set service discovery (called by DiscoveryModule). Like Python container.register_instance(ServiceDiscovery, ...).
    pub fn set_discovery(&mut self, adapter: Box<dyn ServiceDiscovery>) {
        self.discovery = Some(adapter);
    }

    /// Service discovery if registered. Like Python container.resolve(ServiceDiscovery).
    pub fn discovery(&self) -> Option<&dyn ServiceDiscovery> {
        self.discovery.as_deref()
    }

    /// DI container: register and resolve dependencies. Like Python app.container.
    pub fn container(&self) -> &Container {
        &self.container
    }

    /// DI container (mutable).
    pub fn container_mut(&mut self) -> &mut Container {
        &mut self.container
    }

    /// Subscribe to a domain event type. Called by DomainModule::register_into.
    pub fn subscribe_event(&mut self, type_id: TypeId, handler: EventHandler) {
        self.event_handlers
            .entry(type_id)
            .or_default()
            .push(handler);
    }

    /// Publish event (payload as JSON) to all handlers registered for the given type.
    pub fn publish_event(&self, type_id: TypeId, payload: Value) -> Result<(), CoreErrorInner> {
        if let Some(handlers) = self.event_handlers.get(&type_id) {
            for h in handlers {
                h(payload.clone())?;
            }
        }
        Ok(())
    }

    /// Register a route and handler. Path e.g. "orders/commands/create_order".
    pub fn register_route(
        &mut self,
        method: &str,
        path: &str,
        request_schema: Option<Value>,
        handler: Handler,
        openapi_tag: Option<&str>,
    ) -> Result<RouteId, CoreErrorInner> {
        let id = self
            .core
            .register_route(method, path, request_schema, openapi_tag)?;
        self.handlers.insert(id, handler);
        Ok(id)
    }

    /// Add command: POST {context}/commands/{name}. Core builds path.
    pub fn add_command(
        &mut self,
        context: &str,
        name: &str,
        request_schema: Option<Value>,
        handler: Handler,
        _openapi_tag: Option<&str>,
    ) -> Result<RouteId, CoreErrorInner> {
        let id = self.core.add_command(context, name, request_schema)?;
        self.handlers.insert(id, handler);
        Ok(id)
    }

    /// Add query: GET {context}/queries/{name}. Core builds path.
    pub fn add_query(
        &mut self,
        context: &str,
        name: &str,
        request_schema: Option<Value>,
        handler: Handler,
        _openapi_tag: Option<&str>,
    ) -> Result<RouteId, CoreErrorInner> {
        let id = self.core.add_query(context, name, request_schema)?;
        self.handlers.insert(id, handler);
        Ok(id)
    }

    /// Add RPC route (one POST). Then use add_rpc_method for each method.
    pub fn add_rpc_route(&mut self, path: &str) -> Result<(), CoreErrorInner> {
        self.core.add_rpc_route(path)
    }

    /// Add RPC method. Callback receives params as JSON value.
    pub fn add_rpc_method(
        &mut self,
        name: &str,
        request_schema: Option<Value>,
        handler: Handler,
    ) -> Result<RouteId, CoreErrorInner> {
        let id = self.core.add_rpc_method(name, request_schema)?;
        self.handlers.insert(id, handler);
        Ok(id)
    }

    /// Register a domain module (bounded context). Like Python: app.register(employees_module).
    pub fn register(&mut self, module: &mut dyn crate::core::Module) -> Result<(), CoreErrorInner> {
        module.register_into(self)
    }

    pub(crate) fn install_callback(&mut self) {
        if self.callback_installed {
            return;
        }
        self.callback_installed = true;
        let handlers = std::mem::take(&mut self.handlers);
        self.core.set_callback(Box::new(move |route_id, body| {
            let value: Value = if body.is_empty() {
                Value::Null
            } else {
                serde_json::from_slice(body).map_err(|e| CoreErrorInner::Validation(e.to_string()))?
            };
            let handler = handlers
                .get(&route_id)
                .ok_or_else(|| CoreErrorInner::NotFound(format!("route_id {:?}", route_id)))?;
            let result = handler(value)?;
            serde_json::to_vec(&result).map_err(Into::into)
        }));
    }

    /// Handle one request (for tests or when HTTP is external).
    pub fn handle_request(
        &mut self,
        method: &str,
        path: &str,
        body: &[u8],
    ) -> Result<Vec<u8>, CoreErrorInner> {
        if !self.handlers.is_empty() {
            self.install_callback();
        }
        self.core.handle_request(method, path, body)
    }

    /// OpenAPI spec as JSON value.
    pub fn openapi_spec(&self, title: &str, version: &str) -> Value {
        self.core.openapi_spec(title, version)
    }

    /// Run HTTP server (blocks). Serves routes, /openapi.json, /docs.
    pub fn run(
        mut self,
        host: &str,
        port: u16,
        openapi_title: &str,
        openapi_version: &str,
    ) -> Result<(), Box<dyn std::error::Error + Send + Sync>> {
        if !self.handlers.is_empty() {
            self.install_callback();
        }
        self.core.run(host, port, openapi_title, openapi_version)
    }
}

impl Default for Application {
    fn default() -> Self {
        Self::new()
    }
}
