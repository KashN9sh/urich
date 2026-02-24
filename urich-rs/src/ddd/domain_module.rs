//! Domain module (bounded context). Like Python DomainModule. Handlers run in async context.

use serde_json::Value;
use std::any::TypeId;
use std::sync::{Arc, Mutex};
use urich_core::CoreError as CoreErrorInner;

use crate::core::app::{Application, EventHandler, Handler};
use crate::core::container::Container;
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

    /// Add command: POST {context}/commands/{name}. Handler is async (receives body, container Arc).
    pub fn command<F, Fut>(
        mut self,
        name: &str,
        handler: F,
    ) -> Self
    where
        F: Fn(Value, Arc<Mutex<Container>>) -> Fut + Send + Sync + 'static,
        Fut: std::future::Future<Output = Result<Value, CoreErrorInner>> + Send + 'static,
    {
        let path = format!("{}/commands/{}", self.context, name);
        self.commands.push((path, Box::new(move |body, container| Box::pin(handler(body, container)))));
        self
    }

    /// Add query: GET {context}/queries/{name}. Handler is async.
    pub fn query<F, Fut>(
        mut self,
        name: &str,
        handler: F,
    ) -> Self
    where
        F: Fn(Value, Arc<Mutex<Container>>) -> Fut + Send + Sync + 'static,
        Fut: std::future::Future<Output = Result<Value, CoreErrorInner>> + Send + 'static,
    {
        let path = format!("{}/queries/{}", self.context, name);
        self.queries.push((path, Box::new(move |body, container| Box::pin(handler(body, container)))));
        self
    }

    /// Add command by type: body deserialized into `C`, handler receives (command, &Container). Sync handler wrapped in async.
    pub fn command_type<C: crate::ddd::Command>(
        mut self,
        handler: impl Fn(C, &Container) -> Result<Value, CoreErrorInner> + Send + Sync + 'static,
    ) -> Self {
        let path = format!("{}/commands/{}", self.context, C::name());
        let handler_arc: Arc<dyn Fn(Value, &Container) -> Result<Value, CoreErrorInner> + Send + Sync> =
            Arc::new(Box::new(move |body: Value, guard: &Container| {
                let c: C = serde_json::from_value(body)
                    .map_err(|e| CoreErrorInner::Validation(e.to_string()))?;
                handler(c, guard)
            }) as Box<dyn Fn(Value, &Container) -> Result<Value, CoreErrorInner> + Send + Sync>);
        let h: Handler = Box::new(move |body: Value, container: Arc<Mutex<Container>>| {
            let handler = Arc::clone(&handler_arc);
            Box::pin(async move {
                let guard = container.lock().unwrap();
                handler(body, &*guard)
            })
        });
        self.commands.push((path, h));
        self
    }

    /// Add query by type: params deserialized into `Q`, handler receives (query, &Container). Sync handler wrapped in async.
    pub fn query_type<Q: crate::ddd::Query>(
        mut self,
        handler: impl Fn(Q, &Container) -> Result<Value, CoreErrorInner> + Send + Sync + 'static,
    ) -> Self {
        let path = format!("{}/queries/{}", self.context, Q::name());
        let handler_arc: Arc<dyn Fn(Value, &Container) -> Result<Value, CoreErrorInner> + Send + Sync> =
            Arc::new(Box::new(move |body: Value, guard: &Container| {
                let q: Q = serde_json::from_value(body)
                    .map_err(|e| CoreErrorInner::Validation(e.to_string()))?;
                handler(q, guard)
            }) as Box<dyn Fn(Value, &Container) -> Result<Value, CoreErrorInner> + Send + Sync>);
        let h: Handler = Box::new(move |body: Value, container: Arc<Mutex<Container>>| {
            let handler = Arc::clone(&handler_arc);
            Box::pin(async move {
                let guard = container.lock().unwrap();
                handler(body, &*guard)
            })
        });
        self.queries.push((path, h));
        self
    }
}

impl Module for DomainModule {
    fn register_into(&mut self, app: &mut Application) -> Result<(), urich_core::CoreError> {
        let tag = self.context.as_str();
        for (path, handler) in self.commands.drain(..) {
            let name = path
                .strip_prefix(&format!("{}/commands/", self.context))
                .unwrap_or(&path);
            app.add_command(&self.context, name, None, handler, Some(tag))?;
        }
        for (path, handler) in self.queries.drain(..) {
            let name = path
                .strip_prefix(&format!("{}/queries/", self.context))
                .unwrap_or(&path);
            app.add_query(&self.context, name, None, handler, Some(tag))?;
        }
        for (type_id, handler) in self.event_handlers.drain(..) {
            app.subscribe_event(type_id, handler);
        }
        Ok(())
    }
}
