"""Skeleton templates for generating bounded context and aggregate."""

DOMAIN_PY = '''"""Domain {context}: aggregate and events."""
from dataclasses import dataclass
from urich.domain import AggregateRoot, DomainEvent


@dataclass
class {aggregate}Created(DomainEvent):
    """Event: {aggregate} created."""
    {aggregate_lower}_id: str
    # add fields


class {aggregate}(AggregateRoot):
    def __init__(self, id: str):
        super().__init__(id=id)
        self.raise_event({aggregate}Created({aggregate_lower}_id=id))
'''

APPLICATION_PY = '''"""Application layer: commands, queries, handlers."""
from __future__ import annotations

from dataclasses import dataclass
from urich.ddd import Command, Query
from urich.domain import EventBus

from .domain import {aggregate}, {aggregate}Created
from .infrastructure import I{aggregate}Repository


@dataclass
class Create{aggregate}(Command):
    {aggregate_lower}_id: str
    # add fields


@dataclass
class Get{aggregate}(Query):
    {aggregate_lower}_id: str


class Create{aggregate}Handler:
    def __init__(self, repo: I{aggregate}Repository, event_bus: EventBus):
        self._repo = repo
        self._event_bus = event_bus

    async def __call__(self, cmd: Create{aggregate}) -> str:
        agg = {aggregate}(id=cmd.{aggregate_lower}_id)
        await self._repo.add(agg)
        for e in agg.collect_pending_events():
            await self._event_bus.publish(e)
        return agg.id


class Get{aggregate}Handler:
    def __init__(self, repo: I{aggregate}Repository):
        self._repo = repo

    async def __call__(self, query: Get{aggregate}):
        agg = await self._repo.get(query.{aggregate_lower}_id)
        if agg is None:
            return None
        return {{"id": agg.id}}
'''

INFRASTRUCTURE_PY = '''"""Infrastructure: repository implementation."""
from __future__ import annotations

from typing import Optional
from urich.domain import Repository

from .domain import {aggregate}


class I{aggregate}Repository(Repository["{aggregate}"]):
    pass


class {aggregate}RepositoryImpl(I{aggregate}Repository):
    def __init__(self):
        self._store: dict[str, {aggregate}] = {{}}

    async def get(self, id: str) -> Optional[{aggregate}]:
        return self._store.get(id)

    async def add(self, aggregate: {aggregate}) -> None:
        self._store[aggregate.id] = aggregate

    async def save(self, aggregate: {aggregate}) -> None:
        self._store[aggregate.id] = aggregate
'''

MODULE_PY = '''"""One object = bounded context «{context}»."""
from urich.ddd import DomainModule

from .domain import {aggregate}, {aggregate}Created
from .application import Create{aggregate}, Create{aggregate}Handler, Get{aggregate}, Get{aggregate}Handler
from .infrastructure import I{aggregate}Repository, {aggregate}RepositoryImpl


def on_{aggregate_lower}_created(event: {aggregate}Created) -> None:
    """Handler: when {aggregate} is created."""
    ...


{context}_module = (
    DomainModule("{context}")
    .aggregate({aggregate})
    .repository(I{aggregate}Repository, {aggregate}RepositoryImpl)
    .command(Create{aggregate}, Create{aggregate}Handler)
    .query(Get{aggregate}, Get{aggregate}Handler)
    .on_event({aggregate}Created, on_{aggregate_lower}_created)
)
'''

MAIN_PY = '''"""Entry point: app is composed from modules."""
from urich import Application

# from {first_context}.module import {first_context}_module

app = Application()
# app.register({first_context}_module)

# Run: app.run()  # or app.run(host="0.0.0.0", port=8000)
'''

CONTEXT_SKELETON = '''"""Domain {context}."""
from urich.domain import AggregateRoot, DomainEvent

# Add aggregates and events or run: urich add-aggregate {context} <AggregateName>
'''

CONTEXT_APPLICATION_SKELETON = '''"""Application layer for context {context}."""
from urich.ddd import Command, Query

# Add commands, queries and handlers (or generate via add-aggregate)
'''

CONTEXT_INFRASTRUCTURE_SKELETON = '''"""Infrastructure for context {context}."""
from urich.domain import Repository

# Add repository interfaces and implementations
'''

CONTEXT_MODULE_SKELETON = '''"""Bounded context «{context}» — no aggregates yet."""
from urich.ddd import DomainModule

# Add .aggregate(), .repository(), .command(), .query(), .on_event() after add-aggregate
{context}_module = DomainModule("{context}")
'''
