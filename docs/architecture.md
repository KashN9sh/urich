# Architecture & concepts

## Design principles

1. **One object = one building block** — DomainModule, EventBusModule, DiscoveryModule, etc. Each is configured via a fluent API and attached with `app.register(module)`. No scattered decorators or global state.
2. **Protocols over implementations** — EventBus, ServiceDiscovery, RpcTransport, OutboxStorage, OutboxPublisher are protocols. The framework and your code depend on abstractions; you plug in in-memory, Redis, Consul, HTTP+JSON, or your own impl.
3. **Bounded context as DomainModule** — One module per context (e.g. orders, payments). It declares aggregate, repository, commands, queries and event handlers. Routes and DI are wired from this single description.
4. **CQRS by convention** — Commands (write) and queries (read) are separate types and handlers. HTTP routes are generated: `POST /{context}/commands/{name}`, `GET|POST /{context}/queries/{name}`.

---

## Module protocol

Every building block implements **Module**:

```python
def register_into(self, app: Application) -> None: ...
```

When you call `app.register(module)`, the framework calls `module.register_into(app)`. The module then:

- Adds routes via `app.add_route()` or `app.mount()`,
- Registers types in `app.container` (repositories, EventBus, etc.),
- Or both.

This keeps composition explicit and testable: you can register a subset of modules or replace one with a test double.

---

## Dependency injection

The **container** is a registry keyed by type (or string). Resolution is by type: `container.resolve(IOrderRepository)` returns whatever was registered for that interface (typically the implementation class, registered so that the interface resolves to it).

- **register_instance(key, instance)** — Singleton instance.
- **register(key, factory, singleton=True)** — Factory; first resolve runs the factory and caches if singleton.
- **register_class(cls, singleton=True)** — Constructor injection: when resolving `cls`, the container instantiates it and fills constructor parameters by resolving their types from the container. So a handler that needs `IOrderRepository` and `EventBus` gets them automatically.

String annotations (e.g. from `from __future__ import annotations`) are resolved to the actual class in the same module when possible.

---

## Request flow (DomainModule)

1. HTTP request hits `POST /orders/commands/create_order`.
2. Starlette dispatches to the endpoint registered by DomainModule.
3. The endpoint parses JSON into the command dataclass (`CreateOrder`).
4. Handler is resolved from the container (if it’s a class) or used as-is (if it’s a function).
5. Handler is called with the command; it may load/save aggregates and publish domain events via EventBus.
6. Response is JSON: `{"ok": true, "result": ...}` for commands, or the query result directly for queries.

---

## Package layout (high level)

| Package | Role |
|---------|------|
| **urich** | Application, Container, Module, HttpModule, Config. |
| **urich.domain** | Entity, ValueObject, DomainEvent, Repository, EventBus, InProcessEventDispatcher. |
| **urich.ddd** | DomainModule, Command, Query. |
| **urich.events** | EventBusModule, EventBusAdapter, OutboxModule, OutboxStorage, OutboxPublisher. |
| **urich.discovery** | DiscoveryModule, ServiceDiscovery, static_discovery. |
| **urich.rpc** | RpcModule, RpcTransport, RpcServerHandler, JsonHttpRpcTransport. |
| **urich.core** | App, container, module, config, openapi, routing (HttpModule). |
| **urich.cli** | Typer CLI: create-app, add-context, add-aggregate. |
