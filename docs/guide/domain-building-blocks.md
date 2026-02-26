# Domain building blocks

Urich provides base types for the domain layer: **Entity**, **ValueObject**, **DomainEvent**, **Repository**, and **EventBus**. Import from `urich.domain`. You can also keep the domain **free of Urich** and use plain types; see [Domain without Urich](domain-without-framework.md).

---

## Entity

Identity-bearing object: equality and hash by `id`.

```python
from urich.domain import Entity

class Order(Entity):
    def __init__(self, id: str, customer_id: str):
        super().__init__(id=id)
        self.customer_id = customer_id
```

---

## ValueObject

Value without identity; equality by all fields. Uses a frozen dataclass.

```python
from urich.domain import ValueObject
from dataclasses import dataclass

@dataclass(frozen=True)
class Money(ValueObject):
    amount_cents: int
    currency: str
```

---

## DomainEvent

Base type for domain events. Subclass as dataclasses with fields.

```python
from urich.domain import DomainEvent
from dataclasses import dataclass

@dataclass
class OrderCreated(DomainEvent):
    order_id: str
    customer_id: str
    total_cents: int
```

---

## Repository

Abstract interface for aggregate persistence. Generic over the aggregate type.

```python
from urich.domain import Repository
from typing import Optional

class IOrderRepository(Repository[Order]):
    pass

class OrderRepositoryImpl(IOrderRepository):
    async def get(self, id: str) -> Optional[Order]: ...
    async def add(self, aggregate: Order) -> None: ...
    async def save(self, aggregate: Order) -> None: ...
```

- **get(id)** — Load by id; return `None` if not found.
- **add(aggregate)** — Persist a new aggregate.
- **save(aggregate)** — Update an existing aggregate.

DomainModule registers the implementation in the container and resolves the interface to it so handlers get the repo by type.

---

## EventBus

Protocol for publishing and subscribing to domain events. Provided by **EventBusModule** or by DomainModule (in-process) if none is registered.

```python
from urich.domain.events import EventBus

# In a handler:
await self._event_bus.publish(OrderCreated(...))
```

**Protocol:**

- **`async def publish(self, event: DomainEvent) -> None`**
- **`def subscribe(self, event_type: type[DomainEvent], handler: Any) -> None`**

**InProcessEventDispatcher** is the default implementation: subscribe by event type, publish invokes all registered handlers (sync or async).

---

## Imports summary

```python
from urich.domain import (
    Entity,
    ValueObject,
    DomainEvent,
    Repository,
    EventBus,
    InProcessEventDispatcher,
)
```
