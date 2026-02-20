"""Module protocol: any object with register_into(app) can be registered in the application."""
from __future__ import annotations

from typing import TYPE_CHECKING, Protocol, runtime_checkable

if TYPE_CHECKING:
    from urich.core.app import Application


@runtime_checkable
class Module(Protocol):
    """Building block: configured externally, attached via app.register(module)."""

    def register_into(self, app: Application) -> None:
        """Attach the module to the app: routes, DI, subscriptions, etc."""
        ...
