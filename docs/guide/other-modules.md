# Other modules

Besides **DomainModule**, Urich provides modules for events, outbox, service discovery and RPC. All are optional and follow the same pattern: configure with a fluent API, then `app.register(module)`.

## EventBusModule

Registers an **EventBus** in the container. Domain event handlers (from DomainModule `.on_event(...)`) use it to publish/subscribe.

- **In-memory** (single process):

```python
from urich.events import EventBusModule

event_bus_module = EventBusModule().in_memory()
app.register(event_bus_module)
```

- **Custom adapter** (e.g. Redis): `EventBusModule().adapter(your_impl)`.

## OutboxModule

Transactional outbox: store events in the same transaction as your aggregate, then a publisher sends them (e.g. to a message broker). Contracts (storage, publisher) are in core; you implement or supply adapters.

```python
from urich.outbox import OutboxModule

outbox_module = OutboxModule().storage(...).publisher(...)
app.register(outbox_module)
```

## DiscoveryModule

**ServiceDiscovery** — resolve service name to URL. Used by RpcModule client.

- **Static map**:

```python
from urich.discovery import DiscoveryModule

discovery_module = DiscoveryModule().static({
    "orders": "http://orders:8000",
    "payments": "http://payments:8000",
})
app.register(discovery_module)
```

- **Custom adapter** (e.g. Consul): `DiscoveryModule().adapter(your_impl)`.

## RpcModule

- **Server** — expose RPC over HTTP (e.g. `RpcModule().server(path="/rpc")`).
- **Client** — call other services by name using Discovery + Transport:

```python
from urich.rpc import RpcModule
from urich.rpc.transport import JsonHttpRpcTransport

rpc_module = (
    RpcModule()
    .client(discovery=discovery_module, transport=JsonHttpRpcTransport())
)
app.register(rpc_module)
```

`JsonHttpRpcTransport` requires **httpx** (add to your dependencies). You get a client that resolves service URLs via ServiceDiscovery and sends JSON over HTTP.

## Full composition

See [examples/ecommerce](https://github.com/KashN9sh/urich/tree/main/examples/ecommerce) for an app that composes DomainModule, EventBus, Discovery and RPC.
