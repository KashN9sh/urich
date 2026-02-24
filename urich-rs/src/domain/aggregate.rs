//! Aggregate root marker. Like Python AggregateRoot.

/// Aggregate root marker. Like Python AggregateRoot; name used for API consistency / OpenAPI.
pub trait AggregateRoot {
    fn name() -> &'static str
    where
        Self: Sized;
}
