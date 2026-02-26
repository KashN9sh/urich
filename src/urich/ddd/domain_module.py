"""
DomainModule — one object per bounded context.
Describes aggregate, repository, commands, queries, event subscriptions.
"""
from __future__ import annotations

import re
from typing import Any, Callable, Type

from starlette.requests import Request
from starlette.responses import JSONResponse, Response

from urich.core.app import Application
from urich.core.module import Module
from urich.core.openapi import parameters_from_dataclass, schema_from_dataclass
from urich.domain import Repository
from urich.domain.events import EventBus
from urich.ddd.commands import Command, Query


def _snake(name: str) -> str:
    return re.sub(r"(?<!^)(?=[A-Z])", "_", name).lower()


class DomainModule(Module):
    """
    One object = full bounded context.
    .aggregate() .repository() .command() .query() .on_event() .bind()
    Register via app.register(module).
    """

    def __init__(self, name: str, prefix: str | None = None) -> None:
        self.name = name
        self.prefix = prefix or f"/{name}"
        self._aggregate_root: Type[Any] | None = None
        self._repositories: list[tuple[Type[Repository[Any]], Type[Any]]] = []
        self._bindings: list[tuple[Type[Any], Type[Any]]] = []
        self._commands: list[tuple[Type[Command], Type[Any]]] = []
        self._queries: list[tuple[Type[Query], Type[Any]]] = []
        self._event_handlers: list[tuple[type, Any]] = []

    def aggregate(self, root: Type[Any]) -> DomainModule:
        """Register aggregate root type (optional metadata). Event publishing is done in the handler."""
        self._aggregate_root = root
        return self

    def repository(self, interface: Type[Repository[Any]], impl: Type[Any]) -> DomainModule:
        self._repositories.append((interface, impl))
        return self

    def bind(self, interface: Type[Any], impl: Type[Any]) -> DomainModule:
        """Register any interface → implementation for DI (e.g. domain services, strategies)."""
        self._bindings.append((interface, impl))
        return self

    def command(self, cmd_type: Type[Command], handler: Type[Any] | Callable[..., Any]) -> DomainModule:
        self._commands.append((cmd_type, handler))
        return self

    def query(self, query_type: Type[Query], handler: Type[Any] | Callable[..., Any]) -> DomainModule:
        self._queries.append((query_type, handler))
        return self

    def on_event(self, event_type: type, handler: Any) -> DomainModule:
        self._event_handlers.append((event_type, handler))
        return self

    def register_into(self, app: Application) -> None:
        container = app.container

        # Repositories: interface -> implementation
        for iface, impl in self._repositories:
            container.register_class(impl)
            container.register(iface, lambda c=container, i=impl: c.resolve(i))

        # Arbitrary bindings (domain services, strategies, adapters)
        for iface, impl in self._bindings:
            container.register_class(impl)
            container.register(iface, lambda c=container, i=impl: c.resolve(i))

        # EventBus: if already registered (e.g. EventBusModule), use it; else default in-process
        try:
            event_bus = container.resolve(EventBus)
        except KeyError:
            from urich.domain.events import InProcessEventDispatcher
            event_bus = InProcessEventDispatcher()
            container.register_instance(EventBus, event_bus)
            container.register_instance(InProcessEventDispatcher, event_bus)
        for event_type, handler in self._event_handlers:
            event_bus.subscribe(event_type, handler)

        # Command/query handlers: register class in container
        for cmd_type, handler in self._commands:
            if isinstance(handler, type):
                container.register_class(handler)
            path = f"{self.prefix.rstrip('/')}/commands/{_snake(cmd_type.__name__)}"
            app.add_route(
                path,
                self._make_command_endpoint(cmd_type, handler, container),
                methods=["POST"],
                openapi_body_schema=schema_from_dataclass(cmd_type),
                openapi_tags=[self.name],
            )

        for query_type, handler in self._queries:
            if isinstance(handler, type):
                container.register_class(handler)
            path = f"{self.prefix.rstrip('/')}/queries/{_snake(query_type.__name__)}"
            app.add_route(
                path,
                self._make_query_endpoint(query_type, handler, container),
                methods=["GET", "POST"],
                openapi_parameters=parameters_from_dataclass(query_type),
                openapi_body_schema=schema_from_dataclass(query_type),
                openapi_tags=[self.name],
            )

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
            response_result = getattr(result, "id", result) if result is not None else None
            return JSONResponse(
                {"ok": True, "result": response_result} if response_result is not None else {"ok": True}
            )
        return endpoint

    def _make_query_endpoint(
        self, query_type: Type[Query], handler: Type[Any] | Callable[..., Any], container: Any
    ) -> Callable:
        async def endpoint(request: Request) -> Response:
            if request.method == "POST":
                try:
                    body = await request.json()
                except Exception:
                    body = {}
            else:
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
