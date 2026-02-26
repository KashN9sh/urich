# DI: domain services, strategies, adapters

Besides repositories and EventBus, you can inject **any interface** into handlers by registering it with `.bind(interface, impl)` in the DomainModule.

## Registering services and strategies

Use `.bind()` the same way as `.repository()`: the container will resolve the interface to the implementation and inject it into handler constructors by type.

```python
pricing_module = (
    DomainModule("pricing")
    .bind(IDiscountStrategy, PercentDiscountStrategy)
    .bind(IPricingService, PricingServiceImpl)
    .command(CalculatePrice, CalculatePriceHandler)
    .query(GetPriceInfo, get_price_info_handler)
)
```

Define interfaces in the domain (or application) layer as `Protocol` or abstract base class; implement them in infrastructure or application. Handlers request the interface in the constructor:

```python
class CalculatePriceHandler:
    def __init__(self, pricing_service: IPricingService):
        self._pricing = pricing_service

    def __call__(self, cmd: CalculatePrice) -> int:
        return self._pricing.compute_price_cents(cmd.amount_cents, cmd.discount_key)
```

The container resolves `IPricingService` to `PricingServiceImpl`; if that implementation depends on `IDiscountStrategy`, it will resolve to `PercentDiscountStrategy` as well.

## When to use .bind()

- **Domain services** — e.g. pricing, availability, tax calculation.
- **Strategies** — e.g. discount algorithms, shipping cost rules.
- **Adapters** — any port interface (not only Repository or EventBus) that the bounded context needs.

See the [ecommerce example](../examples/ecommerce.md): **pricing** context with `IPricingService`, `IDiscountStrategy`, and `.bind()`.
