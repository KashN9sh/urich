//! Urich core: routing, validation, request handling, async HTTP server.

pub mod http;
pub mod router;
pub mod schema;

pub use router::{Router, RouteId};
pub use schema::validate_json;

use std::collections::HashMap;
use std::future::Future;
use std::pin::Pin;
use std::sync::Arc;
use thiserror::Error;

#[derive(Error, Debug)]
pub enum CoreError {
    #[error("route not found: {0}")]
    NotFound(String),
    #[error("validation error: {0}")]
    Validation(String),
    #[error("invalid JSON: {0}")]
    Json(#[from] serde_json::Error),
}

/// Registered route: method, path pattern, optional request body schema (JSON Schema), optional OpenAPI tag.
#[derive(Clone, Debug)]
pub struct Route {
    pub id: RouteId,
    pub method: String,
    pub path: String,
    pub request_schema: Option<serde_json::Value>,
    pub openapi_tag: Option<String>,
}

/// Request context passed to the callback (and to middlewares in the facade): method, path, headers, raw body.
#[derive(Clone, Debug)]
pub struct RequestContext {
    pub method: String,
    pub path: String,
    pub headers: Vec<(String, String)>,
    pub body: Vec<u8>,
}

/// Response: status code and body (so middlewares can return 401, etc.).
#[derive(Clone, Debug)]
pub struct Response {
    pub status_code: u16,
    pub body: Vec<u8>,
}

/// Request handler callback: (route_id, payload, context) -> future of response. Stored as Arc so it can be called without holding App lock across await.
pub type RequestCallback = Arc<
    dyn Fn(RouteId, &[u8], &RequestContext) -> Pin<Box<dyn Future<Output = Result<Response, CoreError>> + Send>>
        + Send
        + Sync,
>;

/// Core app: routes, RPC, callback.
pub struct App {
    pub router: Router,
    pub routes: HashMap<RouteId, Route>,
    next_route_id: u32,
    callback: Option<RequestCallback>,
    /// Route id for the single RPC POST route; when a request matches this, body is parsed and dispatched by "method".
    rpc_route_id: Option<RouteId>,
    /// method_name -> (handler_id, optional request schema for params).
    rpc_methods: HashMap<String, (RouteId, Option<serde_json::Value>)>,
    /// event_type_id -> list of handler_ids (execute(handler_id, payload) on publish).
    event_subscriptions: HashMap<String, Vec<RouteId>>,
}

impl App {
    pub fn new() -> Self {
        Self {
            router: Router::new(),
            routes: HashMap::new(),
            next_route_id: 0,
            callback: None,
            rpc_route_id: None,
            rpc_methods: HashMap::new(),
            event_subscriptions: HashMap::new(),
        }
    }

    fn alloc_handler_id(&mut self) -> RouteId {
        let id = RouteId(self.next_route_id);
        self.next_route_id += 1;
        id
    }

    /// Register a route. Path is exact (e.g. "orders/commands/create_order"). Optional openapi_tag for OpenAPI tags (e.g. context name).
    pub fn register_route(
        &mut self,
        method: &str,
        path: &str,
        request_schema: Option<serde_json::Value>,
        openapi_tag: Option<&str>,
    ) -> Result<RouteId, CoreError> {
        let path = path.trim_start_matches('/');
        let id = RouteId(self.next_route_id);
        self.next_route_id += 1;
        self.router.add(method, path, id);
        self.routes.insert(
            id,
            Route {
                id,
                method: method.to_owned(),
                path: path.to_owned(),
                request_schema,
                openapi_tag: openapi_tag.map(String::from),
            },
        );
        Ok(id)
    }

    /// Add command: POST {context}/commands/{name}. Returns handler_id (RouteId) for execute(handler_id, body).
    pub fn add_command(
        &mut self,
        context: &str,
        name: &str,
        request_schema: Option<serde_json::Value>,
    ) -> Result<RouteId, CoreError> {
        let context = context.trim_matches('/');
        let path = format!("{}/commands/{}", context, name);
        self.register_route("POST", &path, request_schema, Some(context))
    }

    /// Add query: GET {context}/queries/{name}. Returns handler_id (RouteId) for execute(handler_id, body).
    pub fn add_query(
        &mut self,
        context: &str,
        name: &str,
        request_schema: Option<serde_json::Value>,
    ) -> Result<RouteId, CoreError> {
        let context = context.trim_matches('/');
        let path = format!("{}/queries/{}", context, name);
        self.register_route("GET", &path, request_schema, Some(context))
    }

    /// Add RPC route: one POST route at path. Body must be JSON { "method": string, "params": object }. Call add_rpc_method for each method.
    pub fn add_rpc_route(&mut self, path: &str) -> Result<(), CoreError> {
        let path = path.trim_matches('/');
        let id = self.register_route("POST", path, None, Some("RPC"))?;
        self.rpc_route_id = Some(id);
        Ok(())
    }

    /// Register RPC method; returns handler_id. Facade stores handler_id -> callable. When request hits RPC route, core parses body.method and calls execute(handler_id, params_bytes).
    pub fn add_rpc_method(
        &mut self,
        name: &str,
        request_schema: Option<serde_json::Value>,
    ) -> Result<RouteId, CoreError> {
        let id = self.alloc_handler_id();
        self.rpc_methods
            .insert(name.to_owned(), (id, request_schema));
        Ok(id)
    }

    /// Subscribe to event type; returns handler_id. Facade stores handler_id -> callable. On publish_event, core calls execute(handler_id, payload) for each subscriber.
    pub fn subscribe_event(&mut self, event_type_id: &str) -> RouteId {
        let id = self.alloc_handler_id();
        self.event_subscriptions
            .entry(event_type_id.to_owned())
            .or_default()
            .push(id);
        id
    }

    /// Publish event: call callback for each subscriber. Async, stops on first error.
    pub async fn publish_event(
        &self,
        event_type_id: &str,
        payload: &[u8],
    ) -> Result<(), CoreError> {
        let cb = self
            .callback
            .clone()
            .ok_or_else(|| CoreError::Validation("no callback set".into()))?;
        let ctx = RequestContext {
            method: "EVENT".to_string(),
            path: String::new(),
            headers: vec![],
            body: payload.to_vec(),
        };
        if let Some(ids) = self.event_subscriptions.get(event_type_id) {
            for &handler_id in ids {
                cb(handler_id, payload, &ctx).await?;
            }
        }
        Ok(())
    }

    pub fn set_callback(&mut self, cb: Box<dyn Fn(RouteId, &[u8], &RequestContext) -> Pin<Box<dyn Future<Output = Result<Response, CoreError>> + Send>> + Send + Sync>) {
        self.callback = Some(Arc::from(cb));
    }

    /// Clone of the callback (for HTTP layer to call without holding lock across await).
    pub fn get_callback(&self) -> Option<RequestCallback> {
        self.callback.clone()
    }

    /// Match route and validate body; returns (handler_id, payload). Used so HTTP layer can release App lock before calling async callback.
    pub fn match_route_and_validate(&self, context: &RequestContext) -> Result<(RouteId, Vec<u8>), CoreError> {
        let route_id = self
            .router
            .match_route(&context.method, &context.path)
            .ok_or_else(|| CoreError::NotFound(format!("{} {}", context.method, context.path)))?;

        let (handler_id, payload) = if self.rpc_route_id == Some(route_id) {
            let body_value: serde_json::Value =
                serde_json::from_slice(&context.body).unwrap_or(serde_json::Value::Null);
            let method_name = body_value
                .get("method")
                .and_then(|v| v.as_str())
                .unwrap_or("");
            let (handler_id, schema) = self
                .rpc_methods
                .get(method_name)
                .cloned()
                .ok_or_else(|| CoreError::NotFound(format!("rpc method {:?}", method_name)))?;
            let params = body_value.get("params").cloned().unwrap_or(serde_json::Value::Null);
            let params_bytes = serde_json::to_vec(&params)?;
            let validated = if let Some(s) = schema {
                validate_json(&params_bytes, &s)?;
                params_bytes
            } else {
                params_bytes
            };
            (handler_id, validated)
        } else {
            let route = self
                .routes
                .get(&route_id)
                .ok_or_else(|| CoreError::NotFound(format!("route_id {:?}", route_id)))?;
            let validated = if let Some(ref schema) = route.request_schema {
                validate_json(&context.body, schema)?
            } else {
                context.body.clone()
            };
            (route_id, validated)
        };
        Ok((handler_id, payload))
    }

    /// Handle a request: match route, validate, call callback. Async.
    pub async fn handle_request(&self, context: &RequestContext) -> Result<Response, CoreError> {
        let (handler_id, payload) = self.match_route_and_validate(context)?;
        let cb = self
            .callback
            .clone()
            .ok_or_else(|| CoreError::Validation("no callback set".into()))?;
        cb(handler_id, &payload, context).await
    }

    /// Run HTTP server (async, use from tokio). Serves routes, GET /openapi.json, GET /docs. Requires callback to be set.
    pub fn run(
        self,
        host: &str,
        port: u16,
        openapi_title: &str,
        openapi_version: &str,
    ) -> Result<(), Box<dyn std::error::Error + Send + Sync>> {
        let app = Arc::new(std::sync::RwLock::new(self));
        http::run(app, host, port, openapi_title, openapi_version)
    }

    /// OpenAPI spec from registered routes (minimal).
    pub fn openapi_spec(&self, title: &str, version: &str) -> serde_json::Value {
        let paths: serde_json::Map<String, serde_json::Value> = self
            .routes
            .values()
            .map(|r| {
                let key = format!("/{}", r.path.trim_start_matches('/'));
                let method = r.method.to_lowercase();
                let mut op = serde_json::Map::new();
                let tags = r
                    .openapi_tag
                    .as_ref()
                    .map(|t| serde_json::json!([t.as_str()]))
                    .unwrap_or(serde_json::json!([]));
                op.insert("tags".into(), tags);
                if let Some(ref s) = r.request_schema {
                    op.insert("requestBody".into(), serde_json::json!({
                        "content": { "application/json": { "schema": s } }
                    }));
                }
                (key, serde_json::json!({ method: op }))
            })
            .collect();
        serde_json::json!({
            "openapi": "3.0.0",
            "info": { "title": title, "version": version },
            "paths": paths
        })
    }
}

impl Default for App {
    fn default() -> Self {
        Self::new()
    }
}
