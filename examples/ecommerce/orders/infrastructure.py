"""Infrastructure: repository implementation and adapters (user-provided)."""
from typing import Optional
from urich.domain import Repository

from .domain import Order, Inventory


class IOrderRepository(Repository[Order]):
    pass


class IInventoryRepository(Repository[Inventory]):
    pass


class OrderRepositoryImpl(IOrderRepository):
    def __init__(self):
        self._store: dict[str, Order] = {}

    async def get(self, id: str) -> Optional[Order]:
        return self._store.get(id)

    async def add(self, aggregate: Order) -> None:
        self._store[aggregate.id] = aggregate

    async def save(self, aggregate: Order) -> None:
        self._store[aggregate.id] = aggregate


class InventoryRepositoryImpl(IInventoryRepository):
    def __init__(self):
        self._store: dict[str, Inventory] = {}

    async def get(self, id: str) -> Optional[Inventory]:
        return self._store.get(id)

    async def add(self, aggregate: Inventory) -> None:
        self._store[aggregate.id] = aggregate

    async def save(self, aggregate: Inventory) -> None:
        self._store[aggregate.id] = aggregate
