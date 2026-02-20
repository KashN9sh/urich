"""Service Discovery protocol: resolve(service_name) -> URL(s)."""
from typing import Protocol, runtime_checkable


@runtime_checkable
class ServiceDiscovery(Protocol):
    """
    How to resolve services by name. User chooses implementation (config, Consul, etcd).
    """

    def resolve(self, service_name: str) -> list[str]:
        """Return list of URLs (e.g. one for static)."""
        ...


def static_discovery(services: dict[str, str]) -> ServiceDiscovery:
    """Minimal out-of-the-box implementation: name -> URL map."""
    return StaticDiscovery(services)


class StaticDiscovery:
    """Discovery from static config (env/map)."""

    def __init__(self, services: dict[str, str]) -> None:
        self._services = dict(services)

    def resolve(self, service_name: str) -> list[str]:
        url = self._services.get(service_name)
        return [url] if url else []
