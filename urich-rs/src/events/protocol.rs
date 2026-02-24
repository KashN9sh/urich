//! Event bus protocol. Outbox traits are in core::outbox.

use serde_json::Value;

pub use crate::core::{OutboxPublisher, OutboxStorage};

/// Event bus adapter. User supplies implementation (in-memory, Redis, etc.).
/// In Rust the default is in-app (Application::subscribe_event / publish_event).
pub trait EventBusAdapter: Send + Sync {
    /// Publish event (payload as JSON).
    fn publish(&self, event: Value) -> Result<(), Box<dyn std::error::Error + Send + Sync>>;
    /// Subscribe handler for event type (type_id identifies the event type).
    fn subscribe(
        &mut self,
        type_id: std::any::TypeId,
        handler: Box<dyn Fn(Value) -> Result<(), Box<dyn std::error::Error + Send + Sync>> + Send + Sync>,
    );
}
