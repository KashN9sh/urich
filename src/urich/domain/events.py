"""Domain events: base type and pending list when raised from aggregate."""
from __future__ import annotations

from dataclasses import dataclass
from typing import Any, Callable, Protocol, runtime_checkable


@runtime_checkable
class EventBus(Protocol):
    """Event bus protocol: publish and subscribe. Implementation by user or EventBusModule."""

    async def publish(self, event: object) -> None:
        ...

    def subscribe(self, event_type: type, handler: Callable[..., Any]) -> None:
        ...


@dataclass
class DomainEvent:
    """Base domain event type. Subclasses are dataclasses with fields."""
    pass


def in_process_dispatcher() -> "InProcessEventDispatcher":
    """Factory for in-process event dispatcher."""
    return InProcessEventDispatcher()


class InProcessEventDispatcher:
    """Dispatcher: subscribe by event type, publish invokes handlers."""

    def __init__(self) -> None:
        self._handlers: dict[type, list[Callable[..., Any]]] = {}

    def subscribe(self, event_type: type, handler: Callable[..., Any]) -> None:
        if event_type not in self._handlers:
            self._handlers[event_type] = []
        self._handlers[event_type].append(handler)

    async def publish(self, event: object) -> None:
        event_type = type(event)
        for handler in self._handlers.get(event_type, []):
            if callable(handler):
                result = handler(event)
                if hasattr(result, "__await__"):
                    await result
