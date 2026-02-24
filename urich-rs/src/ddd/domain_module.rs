//! Domain module (bounded context). Like Python DomainModule.

use serde_json::Value;
use std::any::TypeId;
use urich_core::CoreError as CoreErrorInner;

use crate::core::app::{Application, EventHandler, Handler};
use crate::core::Module;
use crate::domain::{AggregateRoot, DomainEvent};

/// Domain module (bounded context): .aggregate().command()/.command_type().query()/.query_type().on_event() then app.register(module).
pub struct DomainModule {
    pub(crate) context: String,
    pub(crate) commands: Vec<(String, Handler)>,
    pub(crate) queries: Vec<(String, Handler)>,
    pub(crate) aggregate_name: Option<String>,
    pub(crate) event_handlers: Vec<(TypeId, EventHandler)>,
}

impl DomainModule {
    pub fn new(context: &str) -> Self {
        Self {
            context: context.to_string(),
            commands: Vec::new(),
            queries: Vec::new(),
            aggregate_name: None,
            event_handlers: Vec::new(),
        }
    }

    /// Set aggregate root type. Like Python: .aggregate(Order).
    pub fn aggregate<A: AggregateRoot>(mut self) -> Self {
        self.aggregate_name = Some(A::name().to_string());
        self
    }

    /// Subscribe handler to domain event type. Like Python: .on_event(OrderCreated, handler).
    pub fn on_event<E: DomainEvent + 'static>(
        mut self,
        handler: impl Fn(Value) -> Result<(), CoreErrorInner> + Send + Sync + 'static,
    ) -> Self {
        self.event_handlers
            .push((TypeId::of::<E>(), Box::new(handler)));
        self
    }

    /// Declare repository for aggregate. Like Python: .repository(IOrderRepo, OrderRepoImpl).
    pub fn repository<A, R: crate::domain::Repository<A> + ?Sized>(self, _repo: &R) -> Self {
        self
    }

    /// Add command: POST {context}/commands/{name}.
    pub fn command(
        mut self,
        name: &str,
        handler: impl Fn(Value) -> Result<Value, CoreErrorInner> + Send + Sync + 'static,
    ) -> Self {
        let path = format!("{}/commands/{}", self.context, name);
        self.commands.push((path, Box::new(handler)));
        self
    }

    /// Add query: GET {context}/queries/{name}.
    pub fn query(
        mut self,
        name: &str,
        handler: impl Fn(Value) -> Result<Value, CoreErrorInner> + Send + Sync + 'static,
    ) -> Self {
        let path = format!("{}/queries/{}", self.context, name);
        self.queries.push((path, Box::new(handler)));
        self
    }

    /// Add command by type: path from `C::name()`. Like Python: .command(CreateOrder, handler).
    pub fn command_type<C: crate::ddd::Command>(
        self,
        handler: impl Fn(Value) -> Result<Value, CoreErrorInner> + Send + Sync + 'static,
    ) -> Self {
        self.command(C::name(), handler)
    }

    /// Add query by type: path from `Q::name()`. Like Python: .query(GetOrder, handler).
    pub fn query_type<Q: crate::ddd::Query>(
        self,
        handler: impl Fn(Value) -> Result<Value, CoreErrorInner> + Send + Sync + 'static,
    ) -> Self {
        self.query(Q::name(), handler)
    }
}

impl Module for DomainModule {
    fn register_into(&mut self, app: &mut Application) -> Result<(), urich_core::CoreError> {
        let tag = self.context.as_str();
        for (path, handler) in self.commands.drain(..) {
            app.register_route("POST", &path, None, handler, Some(tag))?;
        }
        for (path, handler) in self.queries.drain(..) {
            app.register_route("GET", &path, None, handler, Some(tag))?;
        }
        for (type_id, handler) in self.event_handlers.drain(..) {
            app.subscribe_event(type_id, handler);
        }
        Ok(())
    }
}
