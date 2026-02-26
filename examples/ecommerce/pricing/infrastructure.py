"""Pricing: implementations of domain service and strategy."""
from .domain import IPricingService, IDiscountStrategy


class PricingServiceImpl:
    def __init__(self, discount_strategy: IDiscountStrategy):
        self._strategy = discount_strategy

    def compute_price_cents(self, amount_cents: int, discount_key: str) -> int:
        if discount_key == "none":
            return amount_cents
        return max(0, amount_cents - self._strategy.discount_cents(amount_cents))


class PercentDiscountStrategy:
    def __init__(self):
        self._percent = 10.0

    def discount_cents(self, amount_cents: int) -> int:
        return int(amount_cents * self._percent / 100)
