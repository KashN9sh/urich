"""
DiscoveryModule — building block for service discovery.
Configure via .static(...) or .adapter(...); register with app.register(discovery).
"""
from __future__ import annotations

from typing import Any

from urich.core.app import Application
from urich.core.module import Module
from urich.discovery.protocol import ServiceDiscovery, StaticDiscovery


class DiscoveryModule(Module):
    """
    Discovery as object: one adapter (static, Consul, etcd — user's choice).
    Register via app.register(discovery). Available in container as ServiceDiscovery.
    """

    def __init__(self) -> None:
        self._adapter: Any = None

    def static(self, services: dict[str, str]) -> DiscoveryModule:
        """Static config: service name -> URL."""
        self._adapter = StaticDiscovery(services)
        return self

    def adapter(self, impl: ServiceDiscovery) -> DiscoveryModule:
        """Use custom implementation (protocol: resolve(name) -> list[url])."""
        self._adapter = impl
        return self

    def register_into(self, app: Application) -> None:
        if self._adapter is None:
            self._adapter = StaticDiscovery({})
        app.container.register_instance(ServiceDiscovery, self._adapter)
