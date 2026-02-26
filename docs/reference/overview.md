# API reference (overview)

Main public types and where to import them from. For full signatures, see the source or your IDE.

---

## Core (`urich` / `urich.core`)

| Symbol | Description |
|--------|-------------|
| `Application` | Main app; `register(module)`, `add_route()`, `openapi()`, `container`, `starlette`. |
| `Container` | DI: `register()`, `register_instance()`, `register_class()`, `resolve()`. |
| `Module` | Protocol: `register_into(app)`. |
| `HttpModule` | Plain HTTP routes under a prefix; `.route(path, endpoint, methods)`. |
| `Config` | Base config; `load_from_env(prefix, **defaults)` returns a dict. |

---

## Domain (`urich.domain`)

| Symbol | Description |
|--------|-------------|
| `Entity` | Base for entities; equality by `id`. |
| `ValueObject` | Frozen dataclass base; equality by fields. |
| `DomainEvent` | Base for domain events (dataclass subclasses). |
| `Repository[T]` | Abstract: `get(id)`, `add(aggregate)`, `save(aggregate)`. |
| `EventBus` | Protocol: `publish(event)`, `subscribe(event_type, handler)`. |
| `InProcessEventDispatcher` | Default in-process EventBus implementation. |

---

## DDD (`urich.ddd`)

| Symbol | Description |
|--------|-------------|
| `DomainModule` | Bounded context: `.aggregate()`, `.repository()`, `.command()`, `.query()`, `.on_event()`. |
| `Command` | Base dataclass for commands. |
| `Query` | Base dataclass for queries. |

---

## Events (`urich.events`)

| Symbol | Description |
|--------|-------------|
| `EventBusModule` | `.in_memory()` or `.adapter(impl)`; registers EventBus. |
| `EventBusAdapter` | Protocol: `publish`, `subscribe`. |
| `OutboxModule` | `.storage(impl)`, `.publisher(impl)`. |
| `OutboxStorage` | Protocol: `append(events, *, connection)`. |
| `OutboxPublisher` | Protocol: `fetch_pending()`, `mark_published(ids)`. |

---

## Discovery (`urich.discovery`)

| Symbol | Description |
|--------|-------------|
| `DiscoveryModule` | `.static(services)` or `.adapter(impl)`; registers ServiceDiscovery. |
| `ServiceDiscovery` | Protocol: `resolve(service_name) -> list[str]`. |
| `static_discovery(services)` | Returns StaticDiscovery (name → URL map). |

---

## RPC (`urich.rpc`)

| Symbol | Description |
|--------|-------------|
| `RpcModule` | `.server(path, handler)`, `.client(discovery, transport)`. |
| `RpcTransport` | Protocol: `call(url, method, payload) -> bytes`. |
| `RpcServerHandler` | Protocol: `handle(method, payload) -> bytes`. |
| `JsonHttpRpcTransport` | Built-in HTTP+JSON transport (requires httpx). |

---

## OpenAPI (`urich.core.openapi`)

| Symbol | Description |
|--------|-------------|
| `schema_from_dataclass(cls)` | JSON Schema dict for a dataclass. |
| `parameters_from_dataclass(cls)` | OpenAPI query parameters list for a dataclass. |
| `build_openapi_spec(routes, ...)` | Build full OpenAPI 3.0 spec dict. |

---

## CLI

Entry point: `urich` (after `pip install "urich[cli]"`). Commands: `create-app`, `add-context`, `add-aggregate`. See [CLI](../cli.md).
