"""
DomainModule — one object per bounded context.
Describes aggregate, repository, commands, queries, event subscriptions.
"""
from __future__ import annotations

import re
from typing import Any, Callable, Type

from urich.core.app import Application
from urich.core.request import Request
from urich.core.responses import JSONResponse, Response
from urich.core.module import Module
from urich.core.openapi import parameters_from_dataclass, schema_from_dataclass
from urich.domain import AggregateRoot, DomainEvent, Repository
from urich.domain.events import EventBus
from urich.ddd.commands import Command, Query


def _snake(name: str) -> str:
    return re.sub(r"(?<!^)(?=[A-Z])", "_", name).lower()


class DomainModule(Module):
    """
    One object = full bounded context.
    .aggregate() .repository() .command() .query() .on_event()
    Register via app.register(module).
    """

    def __init__(self, name: str, prefix: str | None = None) -> None:
        self.name = name
        self.prefix = prefix or f"/{name}"
        self._aggregate_root: Type[AggregateRoot] | None = None
        self._repositories: list[tuple[Type[Repository[Any]], Type[Any]]] = []
        self._commands: list[tuple[Type[Command], Type[Any]]] = []
        self._queries: list[tuple[Type[Query], Type[Any]]] = []
        self._event_handlers: list[tuple[Type[DomainEvent], Any]] = []

    def aggregate(self, root: Type[AggregateRoot]) -> DomainModule:
        self._aggregate_root = root
        return self

    def repository(self, interface: Type[Repository[Any]], impl: Type[Any]) -> DomainModule:
        self._repositories.append((interface, impl))
        return self

    def command(self, cmd_type: Type[Command], handler: Type[Any] | Callable[..., Any]) -> DomainModule:
        self._commands.append((cmd_type, handler))
        return self

    def query(self, query_type: Type[Query], handler: Type[Any] | Callable[..., Any]) -> DomainModule:
        self._queries.append((query_type, handler))
        return self

    def on_event(self, event_type: Type[DomainEvent], handler: Any) -> DomainModule:
        self._event_handlers.append((event_type, handler))
        return self

    def register_into(self, app: Application) -> None:
        container = app.container

        # Repositories: interface -> implementation
        for iface, impl in self._repositories:
            container.register_class(impl)
            container.register(iface, lambda c=container, i=impl: c.resolve(i))

        # Context for core paths (no leading slash)
        context = self.prefix.strip("/") or self.name

        # EventBus: if already registered (e.g. EventBusModule), use it; else default in-process wired to core
        try:
            event_bus = container.resolve(EventBus)
        except KeyError:
            from urich.domain.events import InProcessEventDispatcher
            event_bus = InProcessEventDispatcher(core_publish=app.publish_event)
            container.register_instance(EventBus, event_bus)
            container.register_instance(InProcessEventDispatcher, event_bus)
        for event_type, handler in self._event_handlers:
            app.subscribe_event(
                event_type.__name__,
                self._make_event_endpoint(event_type, handler, container),
            )

        # Command/query: core builds path; register handler_id -> endpoint
        for cmd_type, handler in self._commands:
            if isinstance(handler, type):
                container.register_class(handler)
            app.add_command(
                context,
                _snake(cmd_type.__name__),
                self._make_command_endpoint(cmd_type, handler, container),
                request_schema=schema_from_dataclass(cmd_type),
            )

        for query_type, handler in self._queries:
            if isinstance(handler, type):
                container.register_class(handler)
            app.add_query(
                context,
                _snake(query_type.__name__),
                self._make_query_endpoint(query_type, handler, container),
                request_schema=schema_from_dataclass(query_type),
            )

    def _make_event_endpoint(
        self, event_type: Type[DomainEvent], handler: Any, container: Any
    ) -> Callable:
        async def endpoint(request: Request) -> Response:
            try:
                body = await request.json()
            except Exception:
                body = {}
            if hasattr(event_type, "__dataclass_fields__"):
                event = event_type(**{k: body.get(k) for k in getattr(event_type, "__dataclass_fields__", {})})
            else:
                event = body
            if isinstance(handler, type):
                h = container.resolve(handler)
                result = await self._call_handler(h, event)
            else:
                result = await self._call_handler(handler, event)
            return JSONResponse({"ok": True} if result is None else {"ok": True, "result": result})

        return endpoint

    def _make_command_endpoint(
        self, cmd_type: Type[Command], handler: Type[Any] | Callable[..., Any], container: Any
    ) -> Callable:
        async def endpoint(request: Request) -> Response:
            try:
                body = await request.json()
            except Exception:
                body = {}
            cmd = cmd_type(**body)
            if isinstance(handler, type):
                h = container.resolve(handler)
                result = await self._call_handler(h, cmd)
            else:
                result = await self._call_handler(handler, cmd)
            return JSONResponse({"ok": True, "result": result} if result is not None else {"ok": True})
        return endpoint

    def _make_query_endpoint(
        self, query_type: Type[Query], handler: Type[Any] | Callable[..., Any], container: Any
    ) -> Callable:
        async def endpoint(request: Request) -> Response:
            try:
                body = await request.json()
            except Exception:
                body = {}
            if not body and request.method != "POST":
                body = dict(request.query_params)
            # string coercion for query params
            query = query_type(**body)
            if isinstance(handler, type):
                h = container.resolve(handler)
                result = await self._call_handler(h, query)
            else:
                result = await self._call_handler(handler, query)
            return JSONResponse(result if result is not None else {})
        return endpoint

    async def _call_handler(self, handler: Any, payload: Any) -> Any:
        result = handler(payload)
        if hasattr(result, "__await__"):
            return await result
        return result
