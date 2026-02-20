# Urich

**Async DDD framework for microservices on Starlette.** Compose your app from module objects with `app.register(module)` — one consistent style for domain, events, RPC and discovery.

---

## Features

- **One object = one building block** — DomainModule, EventBusModule, OutboxModule, DiscoveryModule, RpcModule. Fluent API, attach with `app.register(module)`.
- **DDD out of the box** — Bounded context as DomainModule: `.aggregate()`, `.repository()`, `.command()`, `.query()`, `.on_event()`. Commands and queries get HTTP routes automatically.
- **No lock-in** — Protocols (EventBus, ServiceDiscovery, RpcTransport) in core; implementations (Redis, Consul, HTTP+JSON) by you or optional adapters.
- **OpenAPI / Swagger** — `app.openapi(...)` adds `/openapi.json` and `/docs`.

---

## Install

```bash
pip install urich
```

With CLI for generating app/context/aggregate skeletons:

```bash
pip install "urich[cli]"
```

---

## Quick start

```python
from urich import Application
from orders.module import orders_module

app = Application()
app.register(orders_module)
# Run: uvicorn main:app --reload
```

Routes by convention: `POST /orders/commands/create_order`, `GET /orders/queries/get_order`. Add `app.openapi(title="My API", version="0.1.0")` and open **GET /docs** for Swagger UI.

---

## Next

- [Getting started](getting-started.md) — minimal app and first DomainModule
- [Application & modules](guide/application.md) — Application, register, OpenAPI
- [Domain module](guide/domain-module.md) — aggregates, commands, queries, events
- [CLI](cli.md) — `urich create-app`, `add-context`, `add-aggregate`
