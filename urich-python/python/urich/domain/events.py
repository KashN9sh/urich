"""Domain events: base type and pending list when raised from aggregate."""
from __future__ import annotations

import json
from dataclasses import asdict, dataclass
from typing import Any, Callable, Protocol, runtime_checkable


@runtime_checkable
class EventBus(Protocol):
    """Event bus protocol: publish and subscribe. Implementation by user or EventBusModule."""

    async def publish(self, event: "DomainEvent") -> None:
        ...

    def subscribe(self, event_type: type["DomainEvent"], handler: Any) -> None:
        ...


@dataclass
class DomainEvent:
    """Base domain event type. Subclasses are dataclasses with fields."""
    pass


def in_process_dispatcher() -> "InProcessEventDispatcher":
    """Factory for in-process event dispatcher."""
    return InProcessEventDispatcher()


class InProcessEventDispatcher:
    """Dispatcher: subscribe by event type, publish invokes handlers. Optional core_publish: (event_type_id, payload_bytes) -> None to route through core."""

    def __init__(self, core_publish: Callable[[str, bytes], None] | None = None) -> None:
        self._handlers: dict[type, list[Any]] = {}
        self._core_publish = core_publish

    def subscribe(self, event_type: type[DomainEvent], handler: Any) -> None:
        if event_type not in self._handlers:
            self._handlers[event_type] = []
        self._handlers[event_type].append(handler)

    async def publish(self, event: DomainEvent) -> None:
        event_type = type(event)
        if self._core_publish is not None:
            payload = json.dumps(asdict(event) if hasattr(event, "__dataclass_fields__") else event.__dict__).encode()
            self._core_publish(event_type.__name__, payload)
            return
        for handler in self._handlers.get(event_type, []):
            if callable(handler):
                result = handler(event)
                if hasattr(result, "__await__"):
                    await result
