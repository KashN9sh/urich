"""One object = full bounded context «orders»."""
from urich.ddd import DomainModule

from .domain import Order, OrderCreated, Inventory, StockReserved
from .application import (
    CreateOrder,
    CreateOrderHandler,
    GetOrder,
    GetOrderHandler,
    ReserveForOrder,
    ReserveForOrderHandler,
)
from .infrastructure import IOrderRepository, OrderRepositoryImpl, IInventoryRepository, InventoryRepositoryImpl


def send_order_created_notification(event: OrderCreated) -> None:
    """Domain event handler (e.g. enqueue notification)."""
    ...


def on_stock_reserved(event: StockReserved) -> None:
    """Domain event handler for stock reservation."""
    ...


orders_module = (
    DomainModule("orders")
    .aggregate(Order)
    .aggregate(Inventory)
    .repository(IOrderRepository, OrderRepositoryImpl)
    .repository(IInventoryRepository, InventoryRepositoryImpl)
    .command(CreateOrder, CreateOrderHandler)
    .command(ReserveForOrder, ReserveForOrderHandler)
    .query(GetOrder, GetOrderHandler)
    .on_event(OrderCreated, send_order_created_notification)
    .on_event(StockReserved, on_stock_reserved)
)
