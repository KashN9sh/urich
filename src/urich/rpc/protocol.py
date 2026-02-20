"""RPC protocols: call by service name and method; transport and serialization — user's choice."""
from typing import Any, Protocol, runtime_checkable


@runtime_checkable
class RpcTransport(Protocol):
    """RPC transport: send request, get response. User implements (HTTP, gRPC)."""

    async def call(self, url: str, method: str, payload: bytes) -> bytes:
        ...


@runtime_checkable
class RpcServerHandler(Protocol):
    """Incoming RPC handler: method + body -> response."""

    async def handle(self, method: str, payload: bytes) -> bytes:
        ...
