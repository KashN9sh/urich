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
| **domain.py** | `Order`, `Inventory` (dataclasses), `OrderCreated`, `StockReserved` (DomainEvents). |
| **application.py** | `CreateOrder`, `GetOrder`, `ReserveForOrder` (multi-aggregate command), handlers with repo(s) + EventBus injection. |
| **infrastructure.py** | `IOrderRepository`, `IInventoryRepository`, in-memory implementations. |
| **module.py** | `DomainModule("orders").aggregate(Order).aggregate(Inventory).repository(...).repository(...).command(...).on_event(...)`. |

## Pricing context (domain service and strategy via .bind())

**pricing/** — Bounded context with no aggregate and no repository; only domain service and strategy registered via `.bind()`:

| File | Purpose |
|------|---------|
| **domain.py** | `IPricingService`, `IDiscountStrategy` (Protocols). |
| **infrastructure.py** | `PricingServiceImpl`, `PercentDiscountStrategy`. |
| **application.py** | `CalculatePrice` command, handler that injects `IPricingService`. |
| **module.py** | `DomainModule("pricing").bind(IDiscountStrategy, ...).bind(IPricingService, ...).command(...).query(...)`. |

## Stateless context (no persistence)

**stateless_module.py** — A module with only commands and queries (no `.aggregate()`, no `.repository()`): commission calculator and validator. See [Stateless context](../guide/stateless-context.md).
