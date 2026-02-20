# Other modules

Besides **DomainModule** and **HttpModule**, Urich provides modules for events, outbox, service discovery and RPC. All use the same pattern: configure with a fluent API, then `app.register(module)`.

---

## EventBusModule

Registers an **EventBus** in the container. Domain event handlers (from DomainModule `.on_event(...)`) use it to publish and subscribe.

### In-memory (single process)

```python
from urich.events import EventBusModule

event_bus_module = EventBusModule().in_memory()
app.register(event_bus_module)
```

### Custom adapter

Implement the **EventBusAdapter** protocol (`publish`, `subscribe`) and pass it:

```python
event_bus_module = EventBusModule().adapter(my_redis_event_bus)
app.register(event_bus_module)
```

**Protocol** (in `urich.domain.events` and `urich.events.protocol`):

- `async def publish(self, event: DomainEvent) -> None`
- `def subscribe(self, event_type: type[DomainEvent], handler: Any) -> None`

---

## OutboxModule

Transactional outbox: persist events in the same transaction as the aggregate; a separate publisher sends them later. Core defines the contracts; you implement storage and publisher.

### Protocols

- **OutboxStorage** — `async def append(self, events: list[DomainEvent], *, connection=None) -> None`. Write events (e.g. to a table) in the same transaction as the aggregate.
- **OutboxPublisher** — `async def fetch_pending(self) -> list[Any]` and `async def mark_published(self, ids: list[Any]) -> None`. A worker/cron calls these to send events and mark them published.

### Usage

```python
from urich.events import OutboxModule, OutboxStorage, OutboxPublisher

outbox_module = (
    OutboxModule()
    .storage(my_storage_impl)
    .publisher(my_publisher_impl)
)
app.register(outbox_module)
```

Storage and publisher are then available in the container for your code to use.

---

## DiscoveryModule

**ServiceDiscovery** resolves a service name to one or more URLs. Used by RPC client and any code that needs to call another service.

### Static map

```python
from urich.discovery import DiscoveryModule

discovery_module = DiscoveryModule().static({
    "orders": "http://orders:8000",
    "payments": "http://payments:8000",
})
app.register(discovery_module)
```

### Custom adapter

Implement **ServiceDiscovery**: `def resolve(self, service_name: str) -> list[str]` (return list of URLs).

```python
discovery_module = DiscoveryModule().adapter(my_consul_discovery)
app.register(discovery_module)
```

Helper: `static_discovery(services: dict[str, str])` returns a `StaticDiscovery` instance (same as `.static(...)` inside the module).

---

## RpcModule

Provides both **server** (accept RPC calls) and **client** (call other services by name).

### Server

```python
from urich.rpc import RpcModule

rpc_module = RpcModule().server(path="/rpc", handler=my_rpc_handler)
app.register(rpc_module)
```

- **path** — Route prefix (e.g. `/rpc`). Incoming requests: `POST /rpc/{method}`.
- **handler** — Optional **RpcServerHandler**: `async def handle(self, method: str, payload: bytes) -> bytes`. If omitted, the built-in endpoint returns a placeholder response.

### Client

```python
from urich.rpc import RpcModule, JsonHttpRpcTransport
from urich.discovery import static_discovery

discovery = static_discovery({"orders": "http://orders:8000"})
transport = JsonHttpRpcTransport(discovery, base_path="/rpc")

rpc_module = (
    RpcModule()
    .server(path="/rpc")
    .client(discovery=discovery, transport=transport)
)
app.register(rpc_module)
# Optionally also register DiscoveryModule(discovery) if other code needs ServiceDiscovery in the container
```

**JsonHttpRpcTransport** requires **httpx** (`pip install httpx`). Constructor: `JsonHttpRpcTransport(discovery: ServiceDiscovery, base_path="/rpc")`. It uses `discovery.resolve(service_name)` to get the base URL and sends HTTP POST with JSON body `{ "method": method, "params": ... }`.

**RpcTransport** protocol: `async def call(self, url: str, method: str, payload: bytes) -> bytes`. You can implement your own (e.g. gRPC, MessagePack).

---

## Full composition example

See [examples/ecommerce](https://github.com/KashN9sh/urich/tree/main/examples/ecommerce) for an app that composes DomainModule, EventBus, Discovery and RPC.
