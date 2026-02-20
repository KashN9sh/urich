"""AggregateRoot — aggregate root; collects domain events until save."""
from typing import List

from urich.domain.entity import Entity
from urich.domain.events import DomainEvent


class AggregateRoot(Entity):
    """
    Aggregate root. Events raised during work are collected
    and handed to repository/dispatcher on save.
    """

    def __init__(self, id: str) -> None:
        super().__init__(id)
        self._pending_events: List[DomainEvent] = []

    def raise_event(self, event: DomainEvent) -> None:
        self._pending_events.append(event)

    def collect_pending_events(self) -> List[DomainEvent]:
        """Collect and clear pending events (called by repository/application layer)."""
        events = list(self._pending_events)
        self._pending_events.clear()
        return events
