//! RPC: RpcModule, RpcClient. Like Python rpc/.

mod protocol;

pub use protocol::{RpcError, RpcServerHandler, RpcTransport};

use serde_json::Value;
use urich_core::CoreError;

use crate::core::app::Application;
use crate::core::{Handler, Module, ServiceDiscovery};

/// RPC as object: .server(path, handler) and .client(discovery, transport). Like Python RpcModule.
pub struct RpcModule {
    server_path: Option<String>,
    server_handler: Option<Box<dyn RpcServerHandler>>,
    client_discovery: Option<Box<dyn ServiceDiscovery>>,
    client_transport: Option<Box<dyn RpcTransport>>,
}

impl RpcModule {
    pub fn new() -> Self {
        Self {
            server_path: None,
            server_handler: None,
            client_discovery: None,
            client_transport: None,
        }
    }

    /// Route for incoming RPC. Single route; method name comes from body["method"].
    pub fn server(mut self, path: &str, handler: Box<dyn RpcServerHandler>) -> Self {
        self.server_path = Some(path.trim_matches('/').to_string());
        self.server_handler = Some(handler);
        self
    }

    /// Client: discovery (resolve name -> URL) and transport.
    pub fn client(
        mut self,
        discovery: Box<dyn ServiceDiscovery>,
        transport: Box<dyn RpcTransport>,
    ) -> Self {
        self.client_discovery = Some(discovery);
        self.client_transport = Some(transport);
        self
    }
}

impl Default for RpcModule {
    fn default() -> Self {
        Self::new()
    }
}

impl Module for RpcModule {
    fn register_into(&mut self, app: &mut Application) -> Result<(), CoreError> {
        if let (Some(path), Some(handler)) = (self.server_path.take(), self.server_handler.take()) {
            let h: Handler = Box::new(move |body: Value| {
                let method = body
                    .get("method")
                    .and_then(|v| v.as_str())
                    .unwrap_or("");
                let params = body.get("params").cloned().unwrap_or(Value::Null);
                let payload = serde_json::to_vec(&params).unwrap_or_default();
                match handler.handle(method, &payload) {
                    Ok(bytes) => serde_json::from_slice(&bytes)
                        .map_err(|e| CoreError::Validation(e.to_string())),
                    Err(e) => Err(CoreError::Validation(e.to_string())),
                }
            });
            app.register_route("POST", &path, None, h, None)?;
        }
        if let (Some(discovery), Some(transport)) =
            (self.client_discovery.take(), self.client_transport.take())
        {
            app.container_mut()
                .register_instance(RpcClient::new(discovery, transport));
        }
        Ok(())
    }
}

/// Client: call(service_name, method, params) -> result. Like Python RpcClient.
pub struct RpcClient {
    discovery: Box<dyn ServiceDiscovery>,
    transport: Box<dyn RpcTransport>,
}

impl RpcClient {
    pub fn new(
        discovery: Box<dyn ServiceDiscovery>,
        transport: Box<dyn RpcTransport>,
    ) -> Self {
        Self {
            discovery,
            transport,
        }
    }

    /// Call remote method. Resolves service URL via discovery, then transport.
    pub fn call(
        &self,
        service_name: &str,
        method: &str,
        params: Value,
    ) -> Result<Value, RpcError> {
        call(
            self.discovery.as_ref(),
            self.transport.as_ref(),
            service_name,
            method,
            params,
        )
    }
}

/// Helper: call service by name using discovery. Pass discovery and transport (e.g. from app).
pub fn call(
    discovery: &dyn ServiceDiscovery,
    transport: &dyn RpcTransport,
    service_name: &str,
    method: &str,
    params: Value,
) -> Result<Value, RpcError> {
    let urls = discovery.resolve(service_name);
    let url = urls.first().ok_or_else(|| {
        RpcError::ServiceUnavailable(format!("service {:?} not found", service_name))
    })?;
    let body = serde_json::json!({ "method": method, "params": params });
    let req = serde_json::to_vec(&body).unwrap_or_default();
    let bytes = transport.call(url, method, &req)?;
    serde_json::from_slice(&bytes).map_err(|e| RpcError::Transport(e.to_string()))
}
