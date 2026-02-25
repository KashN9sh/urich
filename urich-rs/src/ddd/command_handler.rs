//! Handler as type: resolve from container (DI), no lock/resolve in module code. Like Python handler class.

use async_trait::async_trait;
use serde_json::Value;
use urich_core::CoreError;

use crate::ddd::{Command, Query};

/// Handler for command `C`. Implement this and register with `register_factory`; then use `.command_with_handler::<C, Self>()`.
/// No manual lock/resolve in the module — the framework resolves the handler from the container and calls `handle(cmd)`.
#[async_trait]
pub trait CommandHandler<C>: Send + Sync
where
    C: Command,
{
    async fn handle(&self, cmd: C) -> Result<Value, CoreError>;
}

/// Handler for query `Q`. Implement and register with `register_factory`; then use `.query_with_handler::<Q, Self>()`.
#[async_trait]
pub trait QueryHandler<Q>: Send + Sync
where
    Q: Query,
{
    async fn handle(&self, query: Q) -> Result<Value, CoreError>;
}
