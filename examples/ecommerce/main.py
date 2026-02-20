"""
App composition — everything via module objects and app.register().
Framework goal: application as a composition of such building blocks.
"""
from urich import Application
from urich.events import EventBusModule, OutboxModule
from urich.discovery import DiscoveryModule, ServiceDiscovery
from urich.rpc import RpcModule

from config import settings
from orders.module import orders_module
from adapters import (
    RedisEventAdapter,
    PostgresOutboxStorage,
    KafkaPublisher,
    JsonHttpTransport,
)

app = Application()

# Domain modules (bounded contexts)
app.register(orders_module)

# Event bus (adapter — user-provided or .in_memory())
event_bus = EventBusModule().adapter(RedisEventAdapter(settings.redis_url))
app.register(event_bus)

# Outbox — contract; storage/publisher implementations by user
outbox = (
    OutboxModule()
    .storage(PostgresOutboxStorage(settings.database_url))
    .publisher(KafkaPublisher(settings.kafka_brokers))
)
app.register(outbox)

# Discovery — how to find other services
discovery = DiscoveryModule().static({
    "orders": settings.orders_service_url,
    "notifications": settings.notifications_service_url,
})
app.register(discovery)

# RPC — server and client (transport — user or JsonHttpRpcTransport from urich.rpc)
rpc = RpcModule().server(path="/rpc")
rpc.client(discovery=app.container.resolve(ServiceDiscovery), transport=JsonHttpTransport())
app.register(rpc)
