"""One object = full bounded context «orders»."""
from urich.ddd import DomainModule

from .domain import Order, OrderCreated
from .application import CreateOrder, CreateOrderHandler, GetOrder, GetOrderHandler
from .infrastructure import IOrderRepository, OrderRepositoryImpl


def send_order_created_notification(event: OrderCreated) -> None:
    """Domain event handler (e.g. enqueue notification)."""
    ...


orders_module = (
    DomainModule("orders")
    .aggregate(Order)
    .repository(IOrderRepository, OrderRepositoryImpl)
    .command(CreateOrder, CreateOrderHandler)
    .query(GetOrder, GetOrderHandler)
    .on_event(OrderCreated, send_order_created_notification)
)
