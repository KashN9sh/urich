"""Pricing: commands and handlers (DI of domain service and strategy)."""
from __future__ import annotations

from dataclasses import dataclass
from urich.ddd import Command, Query

from .domain import IPricingService


@dataclass
class CalculatePrice(Command):
    amount_cents: int
    discount_key: str


@dataclass
class GetPriceInfo(Query):
    amount_cents: int


class CalculatePriceHandler:
    def __init__(self, pricing_service: IPricingService):
        self._pricing = pricing_service

    def __call__(self, cmd: CalculatePrice) -> int:
        return self._pricing.compute_price_cents(cmd.amount_cents, cmd.discount_key)


def get_price_info_handler(query: GetPriceInfo) -> dict:
    return {"amount_cents": query.amount_cents}
