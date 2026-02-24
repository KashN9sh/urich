//! Domain: entity, aggregate, repository, events.

pub mod aggregate;
pub mod entity;
pub mod events;
pub mod repository;

pub use aggregate::AggregateRoot;
pub use entity::Entity;
pub use events::DomainEvent;
pub use repository::Repository;
