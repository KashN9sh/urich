"""Route module: one object per context; routes are attached to it."""
from __future__ import annotations

from typing import Any, Callable

from urich.core.app import Application
from urich.core.module import Module


class HttpModule(Module):
    """
    HTTP module (bounded context): name + routes.
    Attach via app.register(module). Similar to include_router in FastAPI.
    """

    def __init__(self, name: str, prefix: str | None = None) -> None:
        self.name = name
        self.prefix = prefix or f"/{name}"
        self._routes: list[tuple[str, Any, list[str]]] = []

    def route(self, path: str, endpoint: Callable[..., Any], methods: list[str] | None = None) -> HttpModule:
        """Add a route. path without leading slash is under the module prefix."""
        if methods is None:
            methods = ["GET"]
        p = path if path.startswith("/") else f"/{path}"
        self._routes.append((p, endpoint, methods))
        return self

    def register_into(self, app: Application) -> None:
        for path, endpoint, methods in self._routes:
            full_path = self.prefix.rstrip("/") + path
            app.add_route(full_path, endpoint, methods)
