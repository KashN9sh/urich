# Domain module

A **DomainModule** is one object that describes a bounded context: aggregate, repository(ies), commands, queries and domain event handlers. Register it with `app.register(orders_module)`.

## Fluent API

```python
from urich.ddd import DomainModule

orders_module = (
    DomainModule("orders")
    .aggregate(Order)
    .repository(IOrderRepository, OrderRepositoryImpl)
    .command(CreateOrder, CreateOrderHandler)
    .query(GetOrder, GetOrderHandler)
    .on_event(OrderCreated, on_order_created)
)
```

- **`DomainModule(name, prefix=None)`** — `name` is the context name; `prefix` defaults to `"/{name}"` (e.g. `/orders`).
- **`.aggregate(root)`** — Registers the aggregate root type (optional metadata). The framework does **not** publish events from the aggregate; the command handler publishes events via EventBus. The aggregate can have any shape. See [Domain without Urich](domain-without-framework.md).
- **`.repository(interface, impl)`** — Registers the repository: interface in the container resolves to the implementation. Can be called multiple times for different repositories.
- **`.bind(interface, impl)`** — Registers any interface → implementation for DI (e.g. domain services, strategies, adapters). Handlers can request these types in their constructor.
- **`.command(cmd_type, handler)`** — One command type (dataclass) and one handler (class or callable). Adds `POST /{prefix}/commands/{snake_case(cmd_type.__name__)}`.
- **`.query(query_type, handler)`** — One query type and one handler. Adds `GET` and `POST` for `/{prefix}/queries/{snake_case(query_type.__name__)}`.
- **`.on_event(event_type, handler)`** — Subscribes the handler to the EventBus for this domain event. If no EventBus is registered, an in-process dispatcher is used automatically.

**Event flow:** Register an EventBus (e.g. via EventBusModule) or rely on the automatic InProcess one. In the command handler, after persisting the aggregate, call `await event_bus.publish(...)`. In the module, subscribe with `.on_event(EventType, handler)`. Import: `from urich.domain import EventBus`.

---

## Project structure

| Layer | Role |
|-------|------|
| **domain** | Aggregate root (any type), domain events (any type; optional: subclass of `DomainEvent`). Event publishing is done in the handler. |
| **application** | Command/query dataclasses (subclass of `Command` / `Query`), handler classes or functions. |
| **infrastructure** | Repository interface (e.g. `IOrderRepository`) and implementation (in-memory, DB, etc.). |
| **module.py** | Single `DomainModule(...)` instance; import and pass to `app.register()`. |

---

## Routes

| Kind | HTTP | Path pattern | Body (POST) / GET params |
|------|------|---------------|---------------------------|
| Command | POST | `/{prefix}/commands/{command_name}` | JSON → command dataclass |
| Query | GET, POST | `/{prefix}/queries/{query_name}` | GET: query params; POST: JSON → query dataclass |

Command/query names are derived from the dataclass name in snake_case (e.g. `CreateOrder` → `create_order`).

---

## Handlers

Handlers can be:

1. **A class** — Registered in the container and instantiated with constructor injection. The framework calls the instance with the command/query (handler must be callable: `__call__(self, cmd)` or `async __call__(self, cmd)`).
2. **A function** — Called directly with the command/query. Can be async.

Example class handler with DI. Import EventBus from `urich.domain`. The handler publishes domain events via EventBus and returns the value that will be sent in the HTTP response:

```python
from urich.domain import EventBus

class CreateOrderHandler:
    def __init__(self, order_repository: IOrderRepository, event_bus: EventBus):
        self._repo = order_repository
        self._event_bus = event_bus

    async def __call__(self, cmd: CreateOrder) -> str:
        order = Order(id=cmd.order_id, customer_id=cmd.customer_id, total_cents=cmd.total_cents)
        await self._repo.add(order)
        await self._event_bus.publish(OrderCreated(order_id=order.id, customer_id=order.customer_id, total_cents=order.total_cents))
        return order.id
```

The container resolves constructor dependencies (repositories, EventBus, etc.) by type.

---

## EventBus and event handlers

Import the EventBus type from `urich.domain`: `from urich.domain import EventBus`.

- If you registered **EventBusModule** (or another module that registers `EventBus`), that instance is used.
- If not, DomainModule registers an **InProcessEventDispatcher** as the EventBus automatically.
- `.on_event(OrderCreated, handler)` subscribes `handler` to `OrderCreated`. Handlers are invoked when you call `await event_bus.publish(event)` (e.g. from a command handler after saving the aggregate).

---

## Multiple aggregates

One DomainModule can declare several aggregates: call `.aggregate()`, `.repository()`, `.command()`, `.query()`, `.on_event()` for each. When you add a second (or later) aggregate with the CLI (`urich add-aggregate <context> <AggregateName> --dir ...`), the command **appends** to the existing files instead of overwriting them. See [CLI](../cli.md) for details.

---

## Optional aggregate and repository

You can build a module with only `.command()` and `.query()` (and optionally `.bind()`, `.on_event()`). No `.aggregate()` or `.repository()` required. Use this for stateless contexts (calculators, validators, gateways). See [Stateless context](stateless-context.md).

---

## Response format

- **Command** endpoint returns JSON: `{"ok": true, "result": <handler return value>}` or `{"ok": true}` if the handler returns `None`.
- **Query** endpoint returns JSON: the handler’s return value directly (or `{}` if `None`).

Errors in handlers are not caught by the framework; let them bubble so your ASGI server or middleware can handle them.
