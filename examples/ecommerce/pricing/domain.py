"""Pricing context: domain service and strategy (interfaces only, no Urich)."""
from typing import Protocol


class IPricingService(Protocol):
    def compute_price_cents(self, amount_cents: int, discount_key: str) -> int:
        ...


class IDiscountStrategy(Protocol):
    def discount_cents(self, amount_cents: int) -> int:
        ...
