"""Application layer: commands, queries, handlers."""
from __future__ import annotations

from dataclasses import dataclass
from urich.ddd import Command, Query
from urich.domain import EventBus

from .domain import Order, OrderCreated, Inventory, StockReserved
from .infrastructure import IOrderRepository, IInventoryRepository


@dataclass
class CreateOrder(Command):
    order_id: str
    customer_id: str
    total_cents: int


@dataclass
class GetOrder(Query):
    order_id: str


@dataclass
class ReserveForOrder(Command):
    """Multi-aggregate: touches Order and Inventory."""
    order_id: str
    inventory_id: str
    sku: str
    quantity: int


class CreateOrderHandler:
    def __init__(self, order_repository: IOrderRepository, event_bus: EventBus):
        self._repo = order_repository
        self._event_bus = event_bus

    async def __call__(self, cmd: CreateOrder) -> str:
        order = Order(id=cmd.order_id, customer_id=cmd.customer_id, total_cents=cmd.total_cents)
        await self._repo.add(order)
        await self._event_bus.publish(OrderCreated(order_id=order.id, customer_id=order.customer_id, total_cents=order.total_cents))
        return order.id


class GetOrderHandler:
    def __init__(self, order_repository: IOrderRepository):
        self._repo = order_repository

    async def __call__(self, query: GetOrder):
        order = await self._repo.get(query.order_id)
        if order is None:
            return None
        return {"id": order.id, "customer_id": order.customer_id, "total_cents": order.total_cents}


class ReserveForOrderHandler:
    def __init__(
        self,
        order_repository: IOrderRepository,
        inventory_repository: IInventoryRepository,
        event_bus: EventBus,
    ):
        self._order_repo = order_repository
        self._inventory_repo = inventory_repository
        self._event_bus = event_bus

    async def __call__(self, cmd: ReserveForOrder) -> str:
        order = await self._order_repo.get(cmd.order_id)
        if order is None:
            raise ValueError("Order not found")
        inventory = await self._inventory_repo.get(cmd.inventory_id)
        if inventory is None:
            raise ValueError("Inventory not found")
        inventory.reserve(cmd.quantity, cmd.order_id)
        await self._inventory_repo.save(inventory)
        await self._event_bus.publish(StockReserved(sku=inventory.sku, quantity=cmd.quantity, order_id=cmd.order_id))
        return cmd.order_id
