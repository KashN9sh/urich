"""
OutboxModule — contract: write in same transaction + fetch and publish.
Implementations (DB schema, transport) — user or separate package.
"""
from __future__ import annotations

from typing import TYPE_CHECKING, Any, Protocol, runtime_checkable

from urich.domain.events import DomainEvent

if TYPE_CHECKING:
    from urich.core.app import Application


@runtime_checkable
class OutboxStorage(Protocol):
    """
    Write events to outbox in the same transaction as aggregate save.
    User implements for their DB.
    """

    async def append(self, events: list[DomainEvent], *, connection: Any = None) -> None:
        ...


@runtime_checkable
class OutboxPublisher(Protocol):
    """
    Fetch unpublished records and send to transport.
    User's worker/cron calls this; transport is user's choice.
    """

    async def fetch_pending(self) -> list[Any]:
        ...

    async def mark_published(self, ids: list[Any]) -> None:
        ...


class OutboxModule:
    """
    Outbox building block: configure via .storage(...) and .publisher(...).
    Register via app.register(outbox). Implementations by user.
    """

    def __init__(self) -> None:
        self._storage: OutboxStorage | None = None
        self._publisher: OutboxPublisher | None = None

    def storage(self, impl: OutboxStorage) -> OutboxModule:
        self._storage = impl
        return self

    def publisher(self, impl: OutboxPublisher) -> OutboxModule:
        self._publisher = impl
        return self

    def register_into(self, app: Application) -> None:
        if self._storage is not None:
            app.container.register_instance(OutboxStorage, self._storage)
        if self._publisher is not None:
            app.container.register_instance(OutboxPublisher, self._publisher)
