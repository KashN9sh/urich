"""Orders domain: aggregate and events."""
from dataclasses import dataclass
from urich.domain import AggregateRoot, DomainEvent


@dataclass
class OrderCreated(DomainEvent):
    order_id: str
    customer_id: str
    total_cents: int


class Order(AggregateRoot):
    def __init__(self, id: str, customer_id: str, total_cents: int):
        super().__init__(id=id)
        self.customer_id = customer_id
        self.total_cents = total_cents
        self.raise_event(OrderCreated(order_id=id, customer_id=customer_id, total_cents=total_cents))
