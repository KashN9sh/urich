//! Urich core: routing, validation, request handling, HTTP server.

pub mod http;
pub mod router;
pub mod schema;

pub use router::{Router, RouteId};
pub use schema::validate_json;

use std::collections::HashMap;
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

/// Registered route: method, path pattern, optional request body schema (JSON Schema).
#[derive(Clone, Debug)]
pub struct Route {
    pub id: RouteId,
    pub method: String,
    pub path: String,
    pub request_schema: Option<serde_json::Value>,
}

/// Request handler callback: (route_id, validated body bytes) -> response bytes.
/// The host (Python/Rust facade) implements this.
pub type RequestCallback = Box<dyn Fn(RouteId, &[u8]) -> Result<Vec<u8>, CoreError> + Send + Sync>;

/// Core app: routes and callback.
pub struct App {
    pub router: Router,
    pub routes: HashMap<RouteId, Route>,
    next_route_id: u32,
    callback: Option<RequestCallback>,
}

impl App {
    pub fn new() -> Self {
        Self {
            router: Router::new(),
            routes: HashMap::new(),
            next_route_id: 0,
            callback: None,
        }
    }

    /// Register a route. Path is exact (e.g. "/orders/commands/create_order").
    pub fn register_route(
        &mut self,
        method: &str,
        path: &str,
        request_schema: Option<serde_json::Value>,
    ) -> Result<RouteId, CoreError> {
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
            },
        );
        Ok(id)
    }

    pub fn set_callback(&mut self, cb: RequestCallback) {
        self.callback = Some(cb);
    }

    /// Handle a request without HTTP: match route, validate body, call callback. Used by tests and later by HTTP layer.
    pub fn handle_request(
        &self,
        method: &str,
        path: &str,
        body: &[u8],
    ) -> Result<Vec<u8>, CoreError> {
        let route_id = self
            .router
            .match_route(method, path)
            .ok_or_else(|| CoreError::NotFound(format!("{} {}", method, path)))?;
        let route = self
            .routes
            .get(&route_id)
            .ok_or_else(|| CoreError::NotFound(format!("route_id {:?}", route_id)))?;
        let validated = if let Some(ref schema) = route.request_schema {
            validate_json(body, schema)?
        } else {
            body.to_vec()
        };
        let cb = self
            .callback
            .as_ref()
            .ok_or_else(|| CoreError::Validation("no callback set".into()))?;
        cb(route_id, &validated)
    }

    /// Run HTTP server (blocks). Serves routes, GET /openapi.json, GET /docs. Requires callback to be set.
    pub fn run(
        self,
        host: &str,
        port: u16,
        openapi_title: &str,
        openapi_version: &str,
    ) -> Result<(), Box<dyn std::error::Error + Send + Sync>> {
        let app = std::sync::Arc::new(std::sync::RwLock::new(self));
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
                op.insert("tags".into(), serde_json::json!([]));
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
