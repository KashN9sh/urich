//! Core: Application, Module, Container, HttpModule.

pub mod app;
pub mod container;
pub mod into_core_error;
pub mod module;
pub mod outbox;
pub mod routing;
pub mod service_discovery;

pub use app::{Application, Handler, Middleware};
pub use container::{Container, ContainerError};
pub use into_core_error::IntoCoreError;
pub use module::Module;
pub use routing::HttpModule;
pub use outbox::{OutboxPublisher, OutboxStorage};
pub use service_discovery::ServiceDiscovery;
