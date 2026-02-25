//! Application: registers routes and dispatches to handlers. Shared engine for Rust and Python facades.

use serde_json::Value;
use std::any::TypeId;
use std::collections::HashMap;
use std::future::Future;
use std::pin::Pin;
use std::sync::{Arc, Mutex};

use crate::container::Container;
use crate::outbox::{OutboxPublisher, OutboxStorage};
use crate::service_discovery::ServiceDiscovery;
use crate::{App, CoreError, RequestContext, Response, RouteId};

use crate::module::Module;

/// Async handler: (body, container). Lock container inside handler when resolving.
pub type Handler = Box<
    dyn Fn(Value, Arc<Mutex<Container>>) -> Pin<Box<dyn Future<Output = Result<Value, CoreError>> + Send>>
        + Send
        + Sync,
>;

/// Event handler: receives event as JSON value.
pub type EventHandler = Box<dyn Fn(Value) -> Result<(), CoreError> + Send + Sync>;

/// Async middleware: receives request context, returns Some(response) to short-circuit or None to continue.
pub type Middleware = Box<
    dyn Fn(&RequestContext) -> Pin<Box<dyn Future<Output = Option<Response>> + Send>> + Send + Sync,
>;

/// Single callback for all routes (e.g. Python facade: one callable receives route_id, body, context).
pub type ExternalCallback = Arc<
    dyn Fn(RouteId, &[u8], &RequestContext) -> Pin<Box<dyn Future<Output = Result<Response, CoreError>> + Send>>
        + Send
        + Sync,
>;

/// Application: registers routes with core and dispatches to handlers; holds middlewares, Container, discovery, outbox.
/// Either per-route handlers (Rust) or one external_callback (e.g. Python).
pub struct Application {
    pub(crate) core: App,
    pub(crate) handlers: HashMap<RouteId, Handler>,
    pub(crate) callback_installed: bool,
    pub(crate) middlewares: Vec<Middleware>,
    pub(crate) event_handlers: HashMap<TypeId, Vec<EventHandler>>,
    pub(crate) container: Arc<Mutex<Container>>,
    pub(crate) discovery: Option<Box<dyn ServiceDiscovery>>,
    pub(crate) outbox_storage: Option<Box<dyn OutboxStorage>>,
    pub(crate) outbox_publisher: Option<Box<dyn OutboxPublisher>>,
    /// When set (e.g. by Python facade), used instead of handlers map.
    pub(crate) external_callback: Option<ExternalCallback>,
}

impl Application {
    pub fn new() -> Self {
        Self {
            core: App::new(),
            handlers: HashMap::new(),
            callback_installed: false,
            middlewares: Vec::new(),
            event_handlers: HashMap::new(),
            container: Arc::new(Mutex::new(Container::new())),
            discovery: None,
            outbox_storage: None,
            outbox_publisher: None,
            external_callback: None,
        }
    }

    /// Set one callback for all routes (used by Python and other facades). When set, install_callback uses it instead of the handlers map.
    pub fn set_external_callback(&mut self, cb: ExternalCallback) {
        self.external_callback = Some(cb);
    }

    /// Register command route only (no handler). Returns route_id. Use with set_external_callback for Python.
    pub fn add_command_route(
        &mut self,
        context: &str,
        name: &str,
        request_schema: Option<Value>,
    ) -> Result<RouteId, CoreError> {
        self.core.add_command(context, name, request_schema)
    }

    /// Register query route only (no handler). Returns route_id.
    pub fn add_query_route(
        &mut self,
        context: &str,
        name: &str,
        request_schema: Option<Value>,
    ) -> Result<RouteId, CoreError> {
        self.core.add_query(context, name, request_schema)
    }

    /// Register RPC method route only (no handler). For use with set_external_callback (e.g. Python).
    pub fn add_rpc_method_route(
        &mut self,
        name: &str,
        request_schema: Option<Value>,
    ) -> Result<RouteId, CoreError> {
        self.core.add_rpc_method(name, request_schema)
    }

    /// Subscribe to event type (route only). Returns handler_id. For use with set_external_callback.
    pub fn subscribe_event_route(&mut self, event_type_id: &str) -> RouteId {
        self.core.subscribe_event(event_type_id)
    }

    /// Register a route only (no handler). For use with set_external_callback (e.g. Python).
    pub fn register_route_only(
        &mut self,
        method: &str,
        path: &str,
        request_schema: Option<Value>,
        openapi_tag: Option<&str>,
    ) -> Result<RouteId, CoreError> {
        self.core.register_route(method, path, request_schema, openapi_tag)
    }

    /// Publish event by string type id (core convention). For Python and other facades. Blocks on async.
    pub fn publish_event_by_name(
        &self,
        event_type_id: &str,
        payload: &[u8],
    ) -> Result<(), CoreError> {
        let run = self.core.publish_event(event_type_id, payload);
        match tokio::runtime::Handle::try_current() {
            Ok(handle) => handle.block_on(run),
            Err(_) => tokio::runtime::Runtime::new()
                .map_err(|e| CoreError::Validation(e.to_string()))?
                .block_on(run),
        }
    }

    pub fn add_middleware<F, Fut>(&mut self, mw: F) -> &mut Self
    where
        F: Fn(&RequestContext) -> Fut + Send + Sync + 'static,
        Fut: Future<Output = Option<Response>> + Send + 'static,
    {
        self.middlewares.push(Box::new(move |ctx| Box::pin(mw(ctx))));
        self
    }

    pub fn set_outbox_storage(&mut self, s: Box<dyn OutboxStorage>) {
        self.outbox_storage = Some(s);
    }

    pub fn set_outbox_publisher(&mut self, p: Box<dyn OutboxPublisher>) {
        self.outbox_publisher = Some(p);
    }

    pub fn set_discovery(&mut self, adapter: Box<dyn ServiceDiscovery>) {
        self.discovery = Some(adapter);
    }

    pub fn discovery(&self) -> Option<&dyn ServiceDiscovery> {
        self.discovery.as_deref()
    }

    pub fn with_container<F, R>(&self, f: F) -> R
    where
        F: FnOnce(&Container) -> R,
    {
        f(&*self.container.lock().unwrap())
    }

    pub fn with_container_mut<F, R>(&self, f: F) -> R
    where
        F: FnOnce(&mut Container) -> R,
    {
        f(&mut *self.container.lock().unwrap())
    }

    pub fn subscribe_event(&mut self, type_id: TypeId, handler: EventHandler) {
        self.event_handlers
            .entry(type_id)
            .or_default()
            .push(handler);
    }

    pub fn publish_event(&self, type_id: TypeId, payload: Value) -> Result<(), CoreError> {
        if let Some(handlers) = self.event_handlers.get(&type_id) {
            for h in handlers {
                h(payload.clone())?;
            }
        }
        Ok(())
    }

    pub fn register_route(
        &mut self,
        method: &str,
        path: &str,
        request_schema: Option<Value>,
        handler: Handler,
        openapi_tag: Option<&str>,
    ) -> Result<RouteId, CoreError> {
        let id = self
            .core
            .register_route(method, path, request_schema, openapi_tag)?;
        self.handlers.insert(id, handler);
        Ok(id)
    }

    pub fn add_command(
        &mut self,
        context: &str,
        name: &str,
        request_schema: Option<Value>,
        handler: Handler,
        _openapi_tag: Option<&str>,
    ) -> Result<RouteId, CoreError> {
        let id = self.core.add_command(context, name, request_schema)?;
        self.handlers.insert(id, handler);
        Ok(id)
    }

    pub fn add_query(
        &mut self,
        context: &str,
        name: &str,
        request_schema: Option<Value>,
        handler: Handler,
        _openapi_tag: Option<&str>,
    ) -> Result<RouteId, CoreError> {
        let id = self.core.add_query(context, name, request_schema)?;
        self.handlers.insert(id, handler);
        Ok(id)
    }

    pub fn add_rpc_route(&mut self, path: &str) -> Result<(), CoreError> {
        self.core.add_rpc_route(path)
    }

    pub fn add_rpc_method(
        &mut self,
        name: &str,
        request_schema: Option<Value>,
        handler: Handler,
    ) -> Result<RouteId, CoreError> {
        let id = self.core.add_rpc_method(name, request_schema)?;
        self.handlers.insert(id, handler);
        Ok(id)
    }

    pub fn register(&mut self, module: &mut dyn Module) -> Result<(), CoreError> {
        module.register_into(self)
    }

    pub(crate) fn install_callback(&mut self) {
        if self.callback_installed {
            return;
        }
        self.callback_installed = true;
        if let Some(ext) = std::mem::take(&mut self.external_callback) {
            let middlewares = Arc::new(std::mem::take(&mut self.middlewares));
            self.core.set_callback(Box::new(move |route_id, body, ctx: &RequestContext| {
                let ctx = ctx.clone();
                let body = body.to_vec();
                let middlewares = Arc::clone(&middlewares);
                let ext = Arc::clone(&ext);
                Box::pin(async move {
                    for mw in middlewares.iter() {
                        if let Some(resp) = mw(&ctx).await {
                            return Ok(resp);
                        }
                    }
                    ext(route_id, &body, &ctx).await
                })
            }));
            return;
        }
        let handlers = Arc::new(std::mem::take(&mut self.handlers));
        let middlewares = Arc::new(std::mem::take(&mut self.middlewares));
        let container = Arc::clone(&self.container);
        self.core.set_callback(Box::new(move |route_id, body, ctx: &RequestContext| {
            let ctx = ctx.clone();
            let body = body.to_vec();
            let handlers = Arc::clone(&handlers);
            let middlewares = Arc::clone(&middlewares);
            let container = Arc::clone(&container);
            Box::pin(async move {
                for mw in middlewares.iter() {
                    if let Some(resp) = mw(&ctx).await {
                        return Ok(resp);
                    }
                }
                let value: Value = if body.is_empty() {
                    Value::Null
                } else {
                    serde_json::from_slice(&body).map_err(|e| CoreError::Validation(e.to_string()))?
                };
                let handler = handlers
                    .get(&route_id)
                    .ok_or_else(|| CoreError::NotFound(format!("route_id {:?}", route_id)))?;
                let result = handler(value, container).await?;
                let body = serde_json::to_vec(&result).map_err(CoreError::from)?;
                Ok(Response {
                    status_code: 200,
                    body,
                    content_type: None,
                })
            })
        }));
    }

    pub fn handle_request(
        &mut self,
        method: &str,
        path: &str,
        body: &[u8],
    ) -> Result<Vec<u8>, CoreError> {
        if !self.handlers.is_empty() || self.external_callback.is_some() {
            self.install_callback();
        }
        let ctx = RequestContext {
            method: method.to_string(),
            path: path.to_string(),
            headers: vec![],
            body: body.to_vec(),
        };
        let run = async { self.core.handle_request(&ctx).await };
        let result = match tokio::runtime::Handle::try_current() {
            Ok(handle) => handle.block_on(run),
            Err(_) => tokio::runtime::Runtime::new()
                .map_err(|e| CoreError::Validation(e.to_string()))?
                .block_on(run),
        };
        result.map(|r| r.body)
    }

    pub fn openapi_spec(&self, title: &str, version: &str) -> Value {
        self.core.openapi_spec(title, version)
    }

    pub fn run(
        mut self,
        host: &str,
        port: u16,
        openapi_title: &str,
        openapi_version: &str,
    ) -> Result<(), Box<dyn std::error::Error + Send + Sync>> {
        if !self.handlers.is_empty() || self.external_callback.is_some() {
            self.install_callback();
        }
        self.core.run(host, port, openapi_title, openapi_version)
    }

    pub fn run_from_env(
        mut self,
        default_host: &str,
        default_port: u16,
        openapi_title: &str,
        openapi_version: &str,
    ) -> Result<(), Box<dyn std::error::Error + Send + Sync>> {
        if !self.handlers.is_empty() || self.external_callback.is_some() {
            self.install_callback();
        }
        self.core
            .run_from_env(default_host, default_port, openapi_title, openapi_version)
    }
}

impl Default for Application {
    fn default() -> Self {
        Self::new()
    }
}
