# Application & modules

## Application

`Application` wraps Starlette and composes the app from **modules** via `app.register(module)`.

```python
from urich import Application

app = Application()
app.register(orders_module)
app.register(events_module)
# ...
```

- The app is ASGI: use `uvicorn main:app` (or any ASGI server).
- Each module implements the **Module** protocol: `register_into(app)`. DomainModule, EventBusModule, RpcModule, etc. all register routes and/or container bindings.

## Registering modules

Order of registration can matter if one module depends on another (e.g. RPC client uses DiscoveryModule). Register infrastructure/global modules first, then domain modules.

```python
app = Application()
app.register(discovery_module)
app.register(event_bus_module)
app.register(orders_module)
app.register(payments_module)
```

## OpenAPI / Swagger

After registering all modules, call:

```python
app.openapi(title="My API", version="0.1.0")
```

This adds:

- **GET /openapi.json** — OpenAPI 3.0 spec
- **GET /docs** — Swagger UI

Command and query request bodies are derived from your dataclasses, so Swagger shows the correct required fields.

## Container (DI)

The application has a **container**: a singleton registry and constructor injection. Modules can register:

- `register_instance(EventBus, my_impl)` — one instance
- `register_class(IOrderRepository, OrderRepositoryImpl)` — one implementation, resolved by constructor annotations (e.g. `EventBus`, `Config`)

Handlers receive repositories and EventBus from the container.

## HttpModule

For plain HTTP routes (no DDD), use **HttpModule** and add routes manually. DomainModule is the main way to get command/query routes by convention.
