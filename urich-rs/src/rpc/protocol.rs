//! RPC protocols: RpcError, RpcTransport, RpcServerHandler. Like Python rpc/protocol.

use async_trait::async_trait;
use thiserror::Error;

#[derive(Error, Debug)]
pub enum RpcError {
    #[error("[{code}] {message}")]
    Server { code: String, message: String },
    #[error("service unavailable: {0}")]
    ServiceUnavailable(String),
    #[error("transport error: {0}")]
    Transport(String),
}

/// RPC transport: send request, get response. User implements (HTTP, etc.). Async so it does not block the runtime.
#[async_trait]
pub trait RpcTransport: Send + Sync {
    async fn call(&self, url: &str, method: &str, payload: &[u8]) -> Result<Vec<u8>, RpcError>;
}

/// Incoming RPC handler: method + payload + container -> response bytes. Lock container, resolve deps, clone, then await (do not hold lock across await). Like Python.
#[async_trait]
pub trait RpcServerHandler: Send + Sync {
    async fn handle(
        &self,
        method: &str,
        payload: &[u8],
        container: std::sync::Arc<std::sync::Mutex<urich_core::Container>>,
    ) -> Result<Vec<u8>, RpcError>;
}
