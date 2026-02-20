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

    @property
    def container(self) -> Container:
        """DI container: registration and resolution of dependencies."""
        return self._container

    @property
    def starlette(self) -> Starlette:
        """ASGI app for uvicorn: uvicorn.run(app.starlette)."""
        return self._starlette
