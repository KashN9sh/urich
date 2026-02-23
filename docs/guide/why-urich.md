# Why Urich

If you're used to `app.include_router()` in FastAPI, Urich gives you the same feel: **`app.register(module)`** — but each module is a full **bounded context**: aggregate, repository, commands, queries and event handlers. One object describes the whole context; routes and dependency injection are wired from it.

Same stack under the hood: **Starlette** (ASGI) and **Pydantic** (validation, OpenAPI). Urich adds a consistent, DDD-oriented way to compose domain, events, RPC and service discovery.

| | FastAPI | Urich |
|---|--------|------|
| **Composition** | Routers, dependency injection | Modules (domain + commands/queries + events + optional RPC) |
| **Unit of reuse** | Router + dependencies | DomainModule = one bounded context |
| **Foundation** | Starlette, Pydantic | Starlette, Pydantic |
| **Focus** | HTTP API, OpenAPI | DDD/CQRS, microservices, events, RPC |

When to choose Urich: you're building microservices or a modular backend and want **DDD/CQRS** with clear boundaries, domain events and optional RPC — without a heavy framework. When a simple REST API is enough, FastAPI alone is often sufficient.
