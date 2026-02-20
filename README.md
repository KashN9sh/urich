# Urich

Async DDD framework for microservices on Starlette.

**Documentation:** [kashn9sh.github.io/urich](https://kashn9sh.github.io/urich)

The application is composed from module objects via `app.register(module)` — similar to FastAPI with routers, but one consistent style for domain, events, RPC and discovery.

## Idea

- **One object = one building block:** DomainModule, EventBusModule, OutboxModule, DiscoveryModule, RpcModule. All configured via fluent API and attached with `app.register(module)`.
- **DDD:** Bounded context as DomainModule with `.aggregate()`, `.repository()`, `.command()`, `.query()`, `.on_event()`. Commands and queries get HTTP routes automatically.
- **No lock-in:** Protocols (EventBus, ServiceDiscovery, RpcTransport) in core; implementations (Redis, Consul, HTTP+JSON) supplied by the user or optional out-of-the-box adapters.

## Install

```bash
pip install urich
# CLI for generating skeletons:
pip install "urich[cli]"
```

## Quick start

```python
from urich import Application
from urich.ddd import DomainModule

# One object = full bounded context
from orders.module import orders_module

app = Application()
app.register(orders_module)

# Run: python -m uvicorn main:app --reload  (or: pip install uvicorn && uvicorn main:app --reload)
```

Routes by convention: `POST /orders/commands/create_order`, `GET /orders/queries/get_order`.

## OpenAPI / Swagger

After registering all modules, call `app.openapi(title="My API", version="0.1.0")`. Then:

- **GET /openapi.json** — OpenAPI 3.0 spec
- **GET /docs** — Swagger UI

```python
app = Application()
# ... app.register(module) ...
app.openapi(title="My API", version="0.1.0")
```

## CLI

```bash
urich create-app myapp
cd myapp
urich add-context orders --dir .
urich add-aggregate orders Order --dir .
# In main.py: from orders.module import orders_module; app.register(orders_module)
```

## Module structure (DomainModule)

- **domain** — aggregate (AggregateRoot), domain events (DomainEvent).
- **application** — commands/queries (Command/Query), handlers (one per command/query).
- **infrastructure** — repository interface and implementation (e.g. in-memory for prototypes).
- **module.py** — one object `DomainModule("orders").aggregate(...).repository(...).command(...).query(...).on_event(...)`; register in the app with `app.register(orders_module)`.

## Other modules

- **EventBusModule** — `.adapter(impl)` or `.in_memory()`; in container as EventBus.
- **OutboxModule** — `.storage(...)` and `.publisher(...)`; contracts in core.
- **DiscoveryModule** — `.static({"svc": "http://..."})` or `.adapter(impl)`; ServiceDiscovery protocol.
- **RpcModule** — `.server(path="/rpc")` and `.client(discovery=..., transport=...)`; optional JsonHttpRpcTransport (requires httpx).

Full composition example: `examples/ecommerce/main.py`.
