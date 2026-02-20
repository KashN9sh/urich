"""Application — Starlette wrapper; app is composed from modules via app.register(module)."""
from __future__ import annotations

from typing import Any

from starlette.applications import Starlette
from starlette.routing import Route

from urich.core.container import Container
from urich.core.module import Module


class Application:
    """
    Application. Composed from modules via register(module).
    Each module is an object with register_into(app).
    """

    def __init__(self, config: Any = None) -> None:
        self._starlette = Starlette(routes=[])
        self._modules: list[Module] = []
        self._container = Container()
        if config is not None:
            self._container.register_instance(type(config), config)
            self._container.register_instance("config", config)

    def register(self, module: Module) -> Application:
        """Register a module (DomainModule, EventBusModule, routes, etc.). Returns self for chaining."""
        module.register_into(self)
        self._modules.append(module)
        return self

    def add_route(self, path: str, endpoint: Any, methods: list[str] | None = None) -> None:
        """Add an HTTP route. Called by modules from register_into."""
        if methods is None:
            methods = ["GET"]
        route = Route(path, endpoint, methods=methods)
        self._starlette.routes.append(route)

    def mount(self, path: str, app: Starlette) -> None:
        """Mount a sub-app at prefix. Called by modules from register_into."""
        from starlette.routing import Mount
        self._starlette.routes.append(Mount(path, app=app))

    def openapi(
        self,
        *,
        title: str = "API",
        version: str = "0.1.0",
        docs_path: str = "/docs",
        openapi_path: str = "/openapi.json",
    ) -> Application:
        """Add OpenAPI spec and Swagger UI. Call after all modules are registered. Returns self."""
        from urich.core.openapi import build_openapi_spec, SWAGGER_UI_HTML
        from starlette.responses import HTMLResponse, JSONResponse

        spec = build_openapi_spec(self._starlette.routes, title=title, version=version)
        self._openapi_spec = spec  # type: ignore[attr-defined]

        async def openapi_endpoint(request: Any) -> Any:
            return JSONResponse(spec)

        async def docs_endpoint(request: Any) -> Any:
            return HTMLResponse(SWAGGER_UI_HTML.replace("/openapi.json", openapi_path))

        self.add_route(openapi_path, openapi_endpoint, methods=["GET"])
        self.add_route(docs_path, docs_endpoint, methods=["GET"])
        return self

    @property
    def container(self) -> Container:
        """DI container: registration and resolution of dependencies."""
        return self._container

    @property
    def starlette(self) -> Starlette:
        """Underlying Starlette ASGI app (e.g. for middleware)."""
        return self._starlette

    async def __call__(self, scope: dict, receive: Any, send: Any) -> None:
        """ASGI: uvicorn.run(app) works directly."""
        await self._starlette(scope, receive, send)
