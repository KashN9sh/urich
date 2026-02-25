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
        self._middlewares: list[Any] = []  # each: (request) -> None | Response
        self._openapi_title = "API"
        self._openapi_version = "0.1.0"
        self._handler_set = False
        if config is not None:
            self._container.register_instance(type(config), config)
            self._container.register_instance("config", config)

    def add_middleware(self, middleware: Any) -> Application:
        """Add a middleware. Middleware receives (request) and returns None to continue or Response to short-circuit (e.g. 401)."""
        self._middlewares.append(middleware)
        return self

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

    def add_command(
        self,
        context: str,
        name: str,
        endpoint: Any,
        *,
        request_schema: dict[str, Any] | None = None,
    ) -> None:
        """Add command: POST {context}/commands/{name}. Core builds path; store handler_id -> endpoint."""
        schema_str = json.dumps(request_schema) if request_schema else None
        route_id = self._core.add_command(context, name, schema_str)
        self._route_handlers[route_id] = (endpoint, "POST")

    def add_query(
        self,
        context: str,
        name: str,
        endpoint: Any,
        *,
        request_schema: dict[str, Any] | None = None,
        openapi_parameters: list[dict[str, Any]] | None = None,
    ) -> None:
        """Add query: GET {context}/queries/{name}. Core builds path; store handler_id -> endpoint."""
        schema_str = json.dumps(request_schema) if request_schema else None
        route_id = self._core.add_query(context, name, schema_str)
        self._route_handlers[route_id] = (endpoint, "GET")

    def add_rpc_route(self, path: str = "rpc") -> None:
        """Add single RPC POST route. Then use add_rpc_method for each method."""
        self._core.add_rpc_route(path)

    def add_rpc_method(
        self,
        name: str,
        endpoint: Any,
        *,
        request_schema: dict[str, Any] | None = None,
    ) -> None:
        """Register RPC method. Callback receives params as bytes (JSON)."""
        schema_str = json.dumps(request_schema) if request_schema else None
        route_id = self._core.add_rpc_method(name, schema_str)
        self._route_handlers[route_id] = (endpoint, "POST")

    def subscribe_event(self, event_type_id: str, endpoint: Any) -> None:
        """Subscribe to event type. Core returns handler_id; store handler_id -> endpoint."""
        route_id = self._core.subscribe_event(event_type_id)
        self._route_handlers[route_id] = (endpoint, "EVENT")

    def publish_event(self, event_type_id: str, payload: bytes) -> None:
        """Publish event: core calls execute(handler_id, payload) for each subscriber."""
        self._core.publish_event(event_type_id, payload)

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
        middlewares = self._middlewares

        def dispatcher(route_id: int, body_bytes: bytes, context: dict) -> tuple[int, bytes]:
            # Build request with headers from context (for middlewares)
            headers = context.get("headers") or []
            if isinstance(headers, list) and headers and isinstance(headers[0], (list, tuple)):
                headers = {str(k): str(v) for k, v in headers}
            elif isinstance(headers, dict):
                pass
            else:
                headers = {}
            req = Request(
                body_bytes,
                context.get("method", "GET"),
                path=context.get("path", ""),
                headers=headers,
            )

            async def run_middlewares() -> tuple[int, bytes] | None:
                for mw in middlewares:
                    result = mw(req)
                    if hasattr(result, "__await__"):
                        result = await result
                    if result is not None:
                        return (getattr(result, "status_code", 200), result.body)
                return None

            async def run_handler() -> tuple[int, bytes]:
                short = await run_middlewares() if middlewares else None
                if short is not None:
                    return short
                endpoint, _ = route_handlers[route_id]
                response = await endpoint(req)
                return (getattr(response, "status_code", 200), response.body)

            return asyncio.run(run_handler())

        return dispatcher

    @property
    def container(self) -> Container:
        """DI container: registration and resolution of dependencies."""
        return self._container

    def run(
        self,
        host: str = "127.0.0.1",
        port: int = 8000,
    ) -> None:
        """Run HTTP server (blocks). Serves routes, GET /openapi.json, GET /docs.
        host/port are defaults; env (HOST, PORT) and CLI (--host, --port) override, like uvicorn.
        """
        if not self._handler_set:
            self._core.set_handler(self._make_dispatcher())
            self._handler_set = True
        self._core.run_from_env(host, port, self._openapi_title, self._openapi_version)
