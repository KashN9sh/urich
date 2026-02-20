"""Event bus adapter protocol: publish and subscribe by event type."""
from typing import Any, Protocol, runtime_checkable

from urich.domain.events import DomainEvent


@runtime_checkable
class EventBusAdapter(Protocol):
    """
    Event bus adapter. User supplies implementation (in-memory, Redis, NATS, etc.).
    Core only requires this protocol.
    """

    async def publish(self, event: DomainEvent) -> None:
        ...

    def subscribe(self, event_type: type[DomainEvent], handler: Any) -> None:
        ...
