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
- **`.aggregate(root)`** — Registers the aggregate root type (used by convention; repository and handlers work with it).
- **`.repository(interface, impl)`** — Registers the repository: interface in the container resolves to the implementation. Can be called multiple times for different repositories.
- **`.command(cmd_type, handler)`** — One command type (dataclass) and one handler (class or callable). Adds `POST /{prefix}/commands/{snake_case(cmd_type.__name__)}`.
- **`.query(query_type, handler)`** — One query type and one handler. Adds `GET` and `POST` for `/{prefix}/queries/{snake_case(query_type.__name__)}`.
- **`.on_event(event_type, handler)`** — Subscribes the handler to the EventBus for this domain event. If no EventBus is registered, an in-process dispatcher is used automatically.

---

## Project structure

| Layer | Role |
|-------|------|
| **domain** | Aggregate root (subclass of `AggregateRoot`), domain events (subclass of `DomainEvent`). |
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

Example class handler with DI:

```python
class CreateOrderHandler:
    def __init__(self, order_repository: IOrderRepository, event_bus: EventBus):
        self._repo = order_repository
        self._event_bus = event_bus

    async def __call__(self, cmd: CreateOrder) -> str:
        order = Order(id=cmd.order_id, customer_id=cmd.customer_id, total_cents=cmd.total_cents)
        await self._repo.add(order)
        for event in order.collect_pending_events():
            await self._event_bus.publish(event)
        return order.id
```

The container resolves `IOrderRepository` and `EventBus` and injects them into the constructor.

---

## EventBus and event handlers

- If you registered **EventBusModule** (or another module that registers `EventBus`), that instance is used.
- If not, DomainModule registers an **InProcessEventDispatcher** as the EventBus automatically.
- `.on_event(OrderCreated, handler)` subscribes `handler` to `OrderCreated`. Handlers are invoked when you call `await event_bus.publish(event)` (e.g. from a command handler after saving the aggregate).

---

## Response format

- **Command** endpoint returns JSON: `{"ok": true, "result": <handler return value>}` or `{"ok": true}` if the handler returns `None`.
- **Query** endpoint returns JSON: the handler’s return value directly (or `{}` if `None`).

Errors in handlers are not caught by the framework; let them bubble so your ASGI server or middleware can handle them.
