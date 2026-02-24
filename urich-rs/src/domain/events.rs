//! Domain event marker. Like Python DomainEvent.

/// Domain event marker. Handlers subscribe by type via .on_event::<E>(handler). Like Python DomainEvent.
pub trait DomainEvent: Send + Sync {}
