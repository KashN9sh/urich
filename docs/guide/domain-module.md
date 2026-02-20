# Domain module

A **DomainModule** is one object that represents a bounded context: aggregate, repository, commands, queries and domain event handlers. Register it with `app.register(orders_module)`.

## Structure

| Layer | Role |
|-------|------|
| **domain** | Aggregate root (extends `AggregateRoot`), domain events (`DomainEvent`). |
| **application** | Commands and queries (dataclasses), one handler per command/query. Handlers get repository and EventBus from DI. |
| **infrastructure** | Repository interface (protocol) and implementation (e.g. in-memory or DB). |
| **module.py** | Single object: `DomainModule("orders").aggregate(...).repository(...).command(...).query(...).on_event(...)` |

## Example: module.py

```python
from urich.ddd import DomainModule
from .domain import Order, OrderCreated
from .application import CreateOrder, CreateOrderHandler, GetOrder, GetOrderHandler
from .infrastructure import IOrderRepository, OrderRepositoryImpl

def on_order_created(event: OrderCreated) -> None:
    ...

orders_module = (
    DomainModule("orders")
    .aggregate(Order)
    .repository(IOrderRepository, OrderRepositoryImpl)
    .command(CreateOrder, CreateOrderHandler)
    .query(GetOrder, GetOrderHandler)
    .on_event(OrderCreated, on_order_created)
)
```

## Routes

- **Commands** → `POST /{context}/commands/{command_name}`  
  Example: `POST /orders/commands/create_order`
- **Queries** → `GET` and `POST` / `{context}/queries/{query_name}`  
  Example: `GET /orders/queries/get_order`

Command/query names are derived from the dataclass name (e.g. `CreateOrder` → `create_order`).

## Aggregate and repository

- **Aggregate** — subclass `AggregateRoot`, emit `DomainEvent`s. The module needs one aggregate type.
- **Repository** — protocol (interface) + implementation. Registered in the container; handlers receive it by type. Typically `get(id)`, `add(aggregate)`, `save(aggregate)`.

## Commands and queries

- **Command** — dataclass (e.g. `CreateOrder(order_id: str, amount: float)`). One handler class that takes the command and performs changes, optionally emitting domain events via the aggregate.
- **Query** — dataclass (e.g. `GetOrder(order_id: str)`). One handler that returns data (e.g. from repository). No side effects.

Handlers are async; they receive repository and EventBus (if registered) via constructor injection.

## Domain events

`.on_event(OrderCreated, handler)` registers a handler that is called when the aggregate (or command handler) dispatches `OrderCreated` via the EventBus. Use the in-memory EventBus for single process or plug an adapter (e.g. Redis) for cross-service events.
