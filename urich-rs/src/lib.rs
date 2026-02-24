//! Urich Rust facade: Application, Module trait, register and run on urich-core.

pub mod core;
pub mod ddd;
pub mod discovery;
pub mod domain;
pub mod events;
pub mod rpc;

pub use core::{Application, Container, ContainerError, Handler, HttpModule, Module, OutboxPublisher, OutboxStorage, ServiceDiscovery};
pub use ddd::{Command, DomainModule, Query};
pub use discovery::{DiscoveryModule, StaticDiscovery};
pub use domain::{AggregateRoot, DomainEvent, Entity, Repository};
pub use events::{EventBusAdapter, EventBusModule, OutboxModule};
pub use rpc::{call, RpcClient, RpcError, RpcModule, RpcServerHandler, RpcTransport};
pub use urich_core::CoreError;
