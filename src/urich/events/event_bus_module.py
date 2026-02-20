"""
EventBusModule — building block for the event bus.
Configure via .adapter(...) or .in_memory(); register with app.register(event_bus).
"""
from __future__ import annotations

from typing import Any

from urich.core.app import Application
from urich.core.module import Module
from urich.domain.events import EventBus, InProcessEventDispatcher
from urich.events.protocol import EventBusAdapter


class EventBusModule(Module):
    """
    Event bus as object: one adapter (in-memory, Redis, NATS — user's choice).
    Register via app.register(event_bus). Available in container as EventBus.
    """

    def __init__(self) -> None:
        self._adapter: EventBusAdapter | None = None

    def adapter(self, impl: EventBusAdapter) -> EventBusModule:
        """Use custom implementation (protocol: publish, subscribe)."""
        self._adapter = impl
        return self

    def in_memory(self) -> EventBusModule:
        """In-memory adapter out of the box for prototypes."""
        self._adapter = InProcessEventDispatcher()
        return self

    def register_into(self, app: Application) -> None:
        if self._adapter is None:
            self._adapter = InProcessEventDispatcher()
        app.container.register_instance(EventBus, self._adapter)
        # backward compat: also register by InProcessEventDispatcher type when in-memory
        if isinstance(self._adapter, InProcessEventDispatcher):
            app.container.register_instance(InProcessEventDispatcher, self._adapter)
