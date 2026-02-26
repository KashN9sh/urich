# Multi-aggregate operations

A single command or query can work with **several aggregates** by injecting multiple repositories (and other dependencies) into the handler.

## Registering multiple repositories

Call `.repository(interface, impl)` once per repository. The container will resolve each interface to its implementation, so the handler constructor can request all of them:

```python
orders_module = (
    DomainModule("orders")
    .aggregate(Order)
    .aggregate(Inventory)
    .repository(IOrderRepository, OrderRepositoryImpl)
    .repository(IInventoryRepository, InventoryRepositoryImpl)
    .command(ReserveForOrder, ReserveForOrderHandler)
    ...
)
```

## Handler with multiple repositories

```python
class ReserveForOrderHandler:
    def __init__(
        self,
        order_repository: IOrderRepository,
        inventory_repository: IInventoryRepository,
        event_bus: EventBus,
    ):
        self._order_repo = order_repository
        self._inventory_repo = inventory_repository
        self._event_bus = event_bus

    async def __call__(self, cmd: ReserveForOrder) -> str:
        order = await self._order_repo.get(cmd.order_id)
        if order is None:
            raise ValueError("Order not found")
        inventory = await self._inventory_repo.get(cmd.inventory_id)
        if inventory is None:
            raise ValueError("Inventory not found")
        inventory.reserve(cmd.quantity, cmd.order_id)
        await self._inventory_repo.save(inventory)
        await self._event_bus.publish(StockReserved(sku=inventory.sku, quantity=cmd.quantity, order_id=cmd.order_id))
        return cmd.order_id
```

You coordinate loading, modifying, and saving aggregates inside the handler. For transactional boundaries across multiple repositories, use a unit-of-work or shared session in your infrastructure layer.

See the [ecommerce example](../examples/ecommerce.md): orders context with `ReserveForOrder` command and `Order` + `Inventory` aggregates.
