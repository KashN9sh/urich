# Urich

**Urich helps teams building Python microservices who want DDD/CQRS without a heavy framework** — one consistent style for domain, events, RPC and discovery, like FastAPI with routers.

Compose your app from module objects with `app.register(module)`.

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

Routes by convention: `POST /orders/commands/create_order`, `GET /orders/queries/get_order`. Add `app.openapi(title="My API", version="0.1.0")` — then open **/docs** for interactive Swagger UI with no extra setup.

---

## Next

- [Getting started](getting-started.md) — minimal app and first DomainModule
- [Why Urich](guide/why-urich.md) — for whom, and how it compares to FastAPI
- [Application & modules](guide/application.md) — Application, container, config, HttpModule
- [Domain module](guide/domain-module.md) — bounded context, commands, queries, events
- [Domain building blocks](guide/domain-building-blocks.md) — Entity, ValueObject, AggregateRoot, Repository, EventBus
- [Other modules](guide/other-modules.md) — EventBus, Outbox, Discovery, RPC
- [OpenAPI & Swagger](guide/openapi.md) — docs and request schemas
- [Architecture](architecture.md) — design, module protocol, DI, request flow
- [Reference](reference/overview.md) — API overview
- [Examples](examples/ecommerce.md) — ecommerce full composition
- [CLI](cli.md) — `urich create-app`, `add-context`, `add-aggregate`
