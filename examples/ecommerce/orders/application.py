"""Application layer: commands, queries, handlers."""
from __future__ import annotations

from dataclasses import dataclass
from urich.ddd import Command, Query
from urich.domain import EventBus

from .domain import Order, OrderCreated
from .infrastructure import IOrderRepository


@dataclass
class CreateOrder(Command):
    order_id: str
    customer_id: str
    total_cents: int


@dataclass
class GetOrder(Query):
    order_id: str


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


class GetOrderHandler:
    def __init__(self, order_repository: IOrderRepository):
        self._repo = order_repository

    async def __call__(self, query: GetOrder):
        order = await self._repo.get(query.order_id)
        if order is None:
            return None
        return {"id": order.id, "customer_id": order.customer_id, "total_cents": order.total_cents}
