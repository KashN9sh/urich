"""Orders domain: aggregate and events."""
from dataclasses import dataclass
from urich.domain import DomainEvent


@dataclass
class OrderCreated(DomainEvent):
    order_id: str
    customer_id: str
    total_cents: int


@dataclass
class Order:
    id: str
    customer_id: str
    total_cents: int


@dataclass
class StockReserved(DomainEvent):
    sku: str
    quantity: int
    order_id: str


@dataclass
class Inventory:
    id: str
    sku: str
    quantity: int

    def reserve(self, quantity: int, order_id: str) -> None:
        if quantity > self.quantity:
            raise ValueError("Insufficient stock")
        self.quantity -= quantity
