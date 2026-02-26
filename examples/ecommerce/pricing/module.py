"""Pricing bounded context: domain service and strategy via .bind()."""
from urich.ddd import DomainModule

from .application import CalculatePrice, CalculatePriceHandler, GetPriceInfo, get_price_info_handler
from .domain import IPricingService, IDiscountStrategy
from .infrastructure import PricingServiceImpl, PercentDiscountStrategy


pricing_module = (
    DomainModule("pricing")
    .bind(IDiscountStrategy, PercentDiscountStrategy)
    .bind(IPricingService, PricingServiceImpl)
    .command(CalculatePrice, CalculatePriceHandler)
    .query(GetPriceInfo, get_price_info_handler)
)
