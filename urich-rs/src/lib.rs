//! Urich Rust facade: Application, Module trait, register and run on urich-core.

use serde_json::Value;
use std::collections::HashMap;
use urich_core::{App, CoreError as CoreErrorInner, RouteId};

pub use urich_core::CoreError;

/// Handler: receives JSON value (validated), returns JSON value or error.
pub type Handler = Box<dyn Fn(Value) -> Result<Value, CoreErrorInner> + Send + Sync>;

/// Application: registers routes with core and dispatches to Rust handlers.
pub struct Application {
    core: App,
    handlers: HashMap<RouteId, Handler>,
    callback_installed: bool,
}

impl Application {
    pub fn new() -> Self {
        Self {
            core: App::new(),
            handlers: HashMap::new(),
            callback_installed: false,
        }
    }

    /// Register a route and handler. Path e.g. "orders/commands/create_order".
    pub fn register_route(
        &mut self,
        method: &str,
        path: &str,
        request_schema: Option<Value>,
        handler: Handler,
    ) -> Result<RouteId, CoreErrorInner> {
        let id = self.core.register_route(method, path, request_schema)?;
        self.handlers.insert(id, handler);
        Ok(id)
    }

    /// Register a domain module (bounded context). Like Python: app.register(employees_module).
    pub fn register(&mut self, module: &mut dyn Module) -> Result<(), CoreErrorInner> {
        module.register_into(self)
    }

    fn install_callback(&mut self) {
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

/// Module: register into app (commands, queries). Same idea as Python DomainModule.
pub trait Module {
    fn register_into(&mut self, app: &mut Application) -> Result<(), CoreErrorInner>;
}

/// Domain module (bounded context): .command(...).query(...) then app.register(module).
pub struct DomainModule {
    context: String,
    commands: Vec<(String, Handler)>,
    queries: Vec<(String, Handler)>,
}

impl DomainModule {
    pub fn new(context: &str) -> Self {
        Self {
            context: context.to_string(),
            commands: Vec::new(),
            queries: Vec::new(),
        }
    }

    /// Add command: POST {context}/commands/{name}. Handler can be a function or closure.
    pub fn command(
        mut self,
        name: &str,
        handler: impl Fn(Value) -> Result<Value, CoreErrorInner> + Send + Sync + 'static,
    ) -> Self {
        let path = format!("{}/commands/{}", self.context, name);
        self.commands.push((path, Box::new(handler)));
        self
    }

    /// Add query: GET {context}/queries/{name}. Handler can be a function or closure.
    pub fn query(
        mut self,
        name: &str,
        handler: impl Fn(Value) -> Result<Value, CoreErrorInner> + Send + Sync + 'static,
    ) -> Self {
        let path = format!("{}/queries/{}", self.context, name);
        self.queries.push((path, Box::new(handler)));
        self
    }
}

impl Module for DomainModule {
    fn register_into(&mut self, app: &mut Application) -> Result<(), CoreErrorInner> {
        for (path, handler) in self.commands.drain(..) {
            app.register_route("POST", &path, None, handler)?;
        }
        for (path, handler) in self.queries.drain(..) {
            app.register_route("GET", &path, None, handler)?;
        }
        Ok(())
    }
}
