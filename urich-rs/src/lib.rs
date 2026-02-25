//! Urich Rust facade: DDD, RPC, discovery, events on urich-core. Application, Container, Module live in urich-core (shared with Python).

pub mod ddd;
pub mod discovery;
pub mod domain;
pub mod events;
pub mod rpc;

pub use urich_core::{
    Application, Container, ContainerError, EventHandler, Handler, HttpModule, IntoCoreError,
    Middleware, Module, OutboxPublisher, OutboxStorage, ServiceDiscovery,
};
pub use ddd::{Command, CommandHandler, DomainModule, Query, QueryHandler};
pub use urich_rs_macros::{Command, Query}; // derive macros (trait and macro share name in different namespaces)
pub use discovery::{DiscoveryModule, StaticDiscovery};
pub use domain::{AggregateRoot, DomainEvent, Entity, Repository};
pub use events::{EventBusAdapter, EventBusModule, OutboxModule};
pub use rpc::{call, RpcClient, RpcError, RpcModule, RpcServerHandler, RpcTransport};
pub use urich_core::{host_port_from_env_and_args, CoreError, RequestContext, Response as CoreResponse};
