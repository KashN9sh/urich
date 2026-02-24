//! Outbox protocols: storage and publisher. Used by Application and events module.

use serde_json::Value;

/// Outbox storage: write events in same transaction as aggregate save.
pub trait OutboxStorage: Send + Sync {
    fn append(
        &mut self,
        events: &[Value],
        connection: Option<&dyn std::any::Any>,
    ) -> Result<(), Box<dyn std::error::Error + Send + Sync>>;
}

/// Outbox publisher: fetch unpublished and send to transport.
pub trait OutboxPublisher: Send + Sync {
    fn fetch_pending(&mut self)
        -> Result<Vec<(String, Value)>, Box<dyn std::error::Error + Send + Sync>>;
    fn mark_published(
        &mut self,
        ids: &[String],
    ) -> Result<(), Box<dyn std::error::Error + Send + Sync>>;
}
