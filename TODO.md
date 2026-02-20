# TODO: DDD framework for microservices

**Principle:** everything in the app is described by building-block objects, configured (fluent/builder) and attached via `app.register(module)` — DomainModule, EventBus, Outbox, Discovery, RPC. One style for all.

## 1. Core on Starlette

- [x] Project init: `pyproject.toml`, package layout `urich` (core, ddd, rpc, events, discovery, cli)
- [x] `Application` class — Starlette wrapper; app composed from modules via `app.register(module)` (like `include_router` in FastAPI)
- [x] Module object (bounded context): one object per context, routes and DDD set attached to it (`HttpModule`, `Module` protocol)
- [x] Minimal DI container: register by type/protocol, resolve dependencies
- [x] Single config object (env/file), available via DI
- [ ] Optional: basic OpenAPI generation from annotations

## 2. DDD structure and CQRS

- [x] Domain base classes: Entity, ValueObject, AggregateRoot
- [x] Domain events: base type + in-process dispatcher (EventBus, InProcessEventDispatcher)
- [x] Repository interface in domain (get, add, save)
- [x] In-memory repository implementation in infrastructure (in example)
- [x] Commands and queries: typed classes, one handler per command/query
- [x] **DDD module object** (`DomainModule`): `.aggregate()`, `.repository()`, `.command()`, `.query()`, `.on_event()`; register via `app.register(module)`, invoked from HTTP (POST /prefix/commands/..., GET|POST /prefix/queries/...)
- [x] Separation of read models (queries) and write models (commands → aggregates)

## 3. Events and microservices

- [x] **`EventBusModule` object:** configure via `.adapter(...)` or `.in_memory()`; register with `app.register(event_bus)`. `EventBusAdapter` protocol (publish/subscribe)
- [x] Optional in core: `EventBusModule().in_memory()` — in-process adapter out of the box
- [x] **`OutboxModule` object:** configure via `.storage(...)` and `.publisher(...)`; `OutboxStorage`, `OutboxPublisher` contracts in core; register with `app.register(outbox)`
- [ ] Event contracts (versions, formats) — recommendation (e.g. Pydantic), no hard dependency

## 4. RPC and discovery

- [x] **`DiscoveryModule` object:** configure via `.static(...)` or `.adapter(...)`; `ServiceDiscovery` protocol (resolve name → list[url]); `StaticDiscovery` implementation out of the box
- [x] **`RpcModule` object:** configure via `.server(path, ...)` and `.client(discovery=..., transport=...)`; `RpcTransport`, `RpcServerHandler` protocols; optional `JsonHttpRpcTransport` (requires httpx) for quick start

## 5. Prototyping (CLI)

- [x] CLI (typer): `urich create-app`, `add-context`, `add-aggregate` (pip install "urich[cli]")
- [x] Templates: domain (AggregateRoot, DomainEvent), application (Command/Query, handlers), infrastructure (Repository), module.py (DomainModule) skeletons
- [x] Generated code — `DomainModule` with .aggregate(), .repository(), .command(), .query(), .on_event(); in main.py — app.register(module)
- [ ] Optional: CRUD endpoint generation per aggregate

## 6. Documentation and examples

- [x] README with concept and quick start
- [x] Example app in `examples/ecommerce/` (bounded context orders, commands/queries, events)
- [ ] Documentation on layers and DDD conventions within the framework
