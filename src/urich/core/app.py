"""Application — composed from modules via app.register(module). Backed by urich-core (no Starlette)."""
from __future__ import annotations

import asyncio
import json
from typing import Any

import urich_core_native

from urich.core.container import Container
from urich.core.module import Module
from urich.core.request import Request


class Application:
    """
    Application. Composed from modules via register(module).
    Uses urich-core for HTTP and routing. No Starlette.
    """

    def __init__(self, config: Any = None) -> None:
        self._core = urich_core_native.CoreApp()
        self._modules: list[Module] = []
        self._container = Container()
        self._route_handlers: dict[int, tuple[Any, str]] = {}  # route_id -> (endpoint, method)
        self._openapi_title = "API"
        self._openapi_version = "0.1.0"
        self._handler_set = False
        if config is not None:
            self._container.register_instance(type(config), config)
            self._container.register_instance("config", config)

    def register(self, module: Module) -> Application:
        """Register a module (DomainModule, EventBusModule, etc.). Returns self for chaining."""
        module.register_into(self)
        self._modules.append(module)
        return self

    def add_route(
        self,
        path: str,
        endpoint: Any,
        methods: list[str] | None = None,
        *,
        openapi_body_schema: dict[str, Any] | None = None,
        openapi_parameters: list[dict[str, Any]] | None = None,
        openapi_tags: list[str] | None = None,
        openapi_security: list[dict[str, Any]] | None = None,
    ) -> None:
        """Add an HTTP route. Registers with core; openapi_* used for OpenAPI (tags, schema)."""
        if methods is None:
            methods = ["GET"]
        path_clean = path.lstrip("/")
        schema_str = json.dumps(openapi_body_schema) if openapi_body_schema else None
        tag = openapi_tags[0] if openapi_tags else None
        for method in methods:
            route_id = self._core.register_route(method, path_clean, schema_str, tag)
            self._route_handlers[route_id] = (endpoint, method)

    def mount(self, path: str, app: Any) -> None:
        """Not supported when using core backend."""
        raise NotImplementedError("mount() not supported; use core backend only")

    def openapi(
        self,
        *,
        title: str = "API",
        version: str = "0.1.0",
        docs_path: str = "/docs",
        openapi_path: str = "/openapi.json",
        security_schemes: dict[str, Any] | None = None,
        global_security: list[dict[str, Any]] | None = None,
    ) -> Application:
        """Store OpenAPI title/version for run(). /openapi.json and /docs are served by core."""
        self._openapi_title = title
        self._openapi_version = version
        return self

    def _make_dispatcher(self) -> Any:
        route_handlers = self._route_handlers

        def dispatcher(route_id: int, body_bytes: bytes) -> bytes:
            endpoint, method = route_handlers[route_id]
            req = Request(body_bytes, method)

            async def run_endpoint() -> bytes:
                response = await endpoint(req)
                return response.body

            return asyncio.run(run_endpoint())

        return dispatcher

    @property
    def container(self) -> Container:
        """DI container: registration and resolution of dependencies."""
        return self._container

    def run(self, host: str = "127.0.0.1", port: int = 8000) -> None:
        """Run HTTP server (blocks). Serves routes, GET /openapi.json, GET /docs. Call set_handler via dispatcher."""
        if not self._handler_set:
            self._core.set_handler(self._make_dispatcher())
            self._handler_set = True
        self._core.run(host, port, self._openapi_title, self._openapi_version)
