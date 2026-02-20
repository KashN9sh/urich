# Application & modules

## Application

`Application` is the entry point. It wraps Starlette and composes the app from **modules** via `app.register(module)`.

```python
from urich import Application

app = Application()
app.register(orders_module)
app.register(events_module)
```

- **ASGI**: use any ASGI server, e.g. `uvicorn main:app --reload`.
- **Config**: pass optional config so it is available in the container: `Application(config=my_config)`. Then handlers can depend on `Config` (or your config type) in the constructor.

### Main API

| Method / property | Description |
|------------------|-------------|
| `register(module)` | Registers a module (DomainModule, EventBusModule, etc.). Returns `self` for chaining. |
| `add_route(path, endpoint, methods=..., openapi_body_schema=..., openapi_parameters=...)` | Adds an HTTP route. Optional OpenAPI schema/parameters for Swagger. |
| `mount(path, app)` | Mounts a Starlette sub-app at a path prefix. |
| `openapi(title=..., version=..., docs_path="/docs", openapi_path="/openapi.json")` | Adds OpenAPI spec and Swagger UI. Call **after** all modules are registered. |
| `container` | The DI container (see below). |
| `starlette` | The underlying Starlette app (e.g. for custom middleware). |

---

## Module protocol

Any object that implements **Module** can be registered:

```python
class Module(Protocol):
    def register_into(self, app: Application) -> None: ...
```

When you call `app.register(module)`, the framework calls `module.register_into(app)`. The module then adds routes, registers types in the container, or both. Built-in modules: **DomainModule**, **HttpModule**, **EventBusModule**, **OutboxModule**, **DiscoveryModule**, **RpcModule**.

---

## Register order

Register infrastructure/global modules first, then domain modules that may depend on them:

```python
app = Application()
app.register(discovery_module)
app.register(event_bus_module)
app.register(orders_module)
app.register(payments_module)
app.openapi(title="My API", version="0.1.0")
```

---

## Container (DI)

The application has a **container**: a registry that resolves dependencies by type (or string key). Handlers receive repositories, EventBus, Config, etc. via **constructor injection**.

### Registering

| Method | Description |
|--------|-------------|
| `container.register_instance(key, instance)` | Register a ready-made instance (e.g. `EventBus`, `Config`). |
| `container.register(key, factory, singleton=True)` | Register a factory; on first resolve the result is cached if `singleton=True`. |
| `container.register_class(cls, singleton=True)` | Register a class; on resolve an instance is created with constructor parameters **resolved from the container**. |

### Resolving

- `container.resolve(SomeType)` returns the instance for `SomeType` (or the registered implementation of a protocol). Raises `KeyError` if not registered.

Modules typically register implementations (e.g. repository impl, EventBus); command/query handlers are registered as classes and get `IOrderRepository`, `EventBus`, etc. by type in `__init__`.

---

## Config

You can pass a config object when creating the app:

```python
from urich import Application, Config

config = MyConfig(**Config.load_from_env("APP_", host="localhost", port=8000))
app = Application(config=config)
```

`Config.load_from_env(prefix, **defaults)` returns a dict from environment variables with the given prefix (e.g. `APP_HOST` → `host`), plus defaults. Your handlers can then depend on `MyConfig` (or `Config`) in the constructor and get it from the container.

---

## HttpModule (plain HTTP routes)

For non-DDD HTTP routes, use **HttpModule**:

```python
from urich import Application, HttpModule

health = HttpModule("health").route("/ping", ping_handler, methods=["GET"])
app = Application()
app.register(health)
```

Routes are mounted under the module prefix (e.g. `/health/ping`). Use `path` with or without leading slash; it is appended to the prefix.
