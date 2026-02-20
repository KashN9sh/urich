# Ecommerce example

The [examples/ecommerce](https://github.com/KashN9sh/urich/tree/main/examples/ecommerce) directory shows a full composition: DomainModule, EventBus, Outbox, Discovery and RPC.

---

## Minimal run (orders only)

**run_minimal.py** — Application + orders DomainModule + OpenAPI:

```python
from urich import Application
from orders.module import orders_module

app = Application()
app.register(orders_module)
app.openapi(title="Ecommerce API", version="0.1.0")
```

Run: `uvicorn run_minimal:app --reload`, then open `/docs`.

---

## Full composition (main.py)

**main.py** composes:

1. **Orders DomainModule** — Commands/queries and domain events for orders.
2. **EventBusModule** — Custom adapter (e.g. Redis) for publishing domain events.
3. **OutboxModule** — Storage and publisher (e.g. Postgres + Kafka) for transactional outbox.
4. **DiscoveryModule** — Static map of service names to URLs.
5. **RpcModule** — Server at `/rpc` and client with custom transport.

Order of registration: discovery and event bus first, then domain modules, then RPC (client may need ServiceDiscovery from the container).

---

## Orders context structure

| File | Purpose |
|------|---------|
| **domain.py** | `Order` (AggregateRoot), `OrderCreated` (DomainEvent). |
| **application.py** | `CreateOrder`, `GetOrder` (Command/Query), handlers with repo + EventBus injection. |
| **infrastructure.py** | `IOrderRepository`, in-memory `OrderRepositoryImpl`. |
| **module.py** | `DomainModule("orders").aggregate(...).repository(...).command(...).query(...).on_event(...)`. |

This is the same structure the CLI generates with `add-context` and `add-aggregate`.
