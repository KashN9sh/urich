//! RPC protocols: RpcError, RpcTransport, RpcServerHandler. Like Python rpc/protocol.

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

/// RPC transport: send request, get response. User implements (HTTP, etc.).
pub trait RpcTransport: Send + Sync {
    fn call(&self, url: &str, method: &str, payload: &[u8]) -> Result<Vec<u8>, RpcError>;
}

/// Incoming RPC handler: method + payload -> response bytes.
pub trait RpcServerHandler: Send + Sync {
    fn handle(&self, method: &str, payload: &[u8]) -> Result<Vec<u8>, RpcError>;
}
