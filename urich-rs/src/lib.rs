//! Urich Rust facade: Application, Module trait, register and run on urich-core.

pub mod core;
pub mod ddd;
pub mod discovery;
pub mod domain;
pub mod events;
pub mod rpc;

pub use core::{Application, Container, ContainerError, Handler, HttpModule, Middleware, Module, OutboxPublisher, OutboxStorage, ServiceDiscovery};
pub use ddd::{DomainModule, Command, Query};
pub use urich_rs_macros::{Command, Query}; // derive macros (trait and macro share name in different namespaces)
pub use discovery::{DiscoveryModule, StaticDiscovery};
pub use domain::{AggregateRoot, DomainEvent, Entity, Repository};
pub use events::{EventBusAdapter, EventBusModule, OutboxModule};
pub use rpc::{call, RpcClient, RpcError, RpcModule, RpcServerHandler, RpcTransport};
pub use urich_core::{CoreError, RequestContext, Response as CoreResponse};
