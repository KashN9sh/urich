"""
RpcModule — building block for RPC: server (accept calls) and client (call other services).
Configure via .server(...) and .client(...); register with app.register(rpc).
Transport and serialization — user's choice; optional minimal HTTP+JSON out of the box.
"""
from __future__ import annotations

from typing import Any, Callable

from starlette.requests import Request
from starlette.responses import Response

from urich.core.app import Application
from urich.core.module import Module
from urich.discovery.protocol import ServiceDiscovery
from urich.rpc.protocol import RpcServerHandler, RpcTransport


class RpcModule(Module):
    """
    RPC as object: .server(path, transport) and .client(discovery, transport).
    One object describes both accepting calls and calling other services.
    """

    def __init__(self) -> None:
        self._server_path: str | None = None
        self._server_transport: Any = None
        self._server_handler: RpcServerHandler | None = None
        self._client_discovery: ServiceDiscovery | None = None
        self._client_transport: RpcTransport | None = None

    def server(
        self,
        path: str = "/rpc",
        transport: Any = None,
        handler: RpcServerHandler | None = None,
    ) -> RpcModule:
        """Route for incoming RPC; transport and handler optional (minimal built-in)."""
        self._server_path = path.rstrip("/")
        self._server_transport = transport
        self._server_handler = handler
        return self

    def client(
        self,
        discovery: ServiceDiscovery | None = None,
        transport: RpcTransport | None = None,
    ) -> RpcModule:
        """Client: discovery (resolve name -> URL) and transport."""
        self._client_discovery = discovery
        self._client_transport = transport
        return self

    def register_into(self, app: Application) -> None:
        if self._server_path is not None:
            app.add_route(
                f"{self._server_path}/{{path:path}}",
                self._make_rpc_endpoint(app),
                methods=["POST"],
            )
        if self._client_discovery is not None:
            app.container.register_instance(ServiceDiscovery, self._client_discovery)
        if self._client_transport is not None:
            app.container.register_instance(RpcTransport, self._client_transport)

    def _make_rpc_endpoint(self, app: Application) -> Callable:
        """Minimal endpoint: POST body = JSON {method, params}; response = JSON."""
        import json

        async def endpoint(request: Request) -> Response:
            method = request.path_params.get("path", "") if request.path_params else ""
            try:
                body = await request.json()
            except Exception:
                body = {}
            params = (body.get("params", {}) if isinstance(body, dict) else {})
            payload_bytes = json.dumps(params).encode()
            if self._server_handler is not None:
                result = await self._server_handler.handle(method, payload_bytes)
            else:
                result = json.dumps({"error": "no handler"}).encode()
            return Response(
                content=result,
                media_type="application/json",
            )
        return endpoint


class JsonHttpRpcTransport:
    """Minimal transport out of the box: HTTP + JSON for quick start."""

    def __init__(self, discovery: ServiceDiscovery, base_path: str = "/rpc") -> None:
        self._discovery = discovery
        self._base_path = base_path

    async def call(self, url: str, method: str, payload: bytes) -> bytes:
        import json
        try:
            import httpx
        except ImportError:
            raise RuntimeError("JsonHttpRpcTransport requires httpx; pip install httpx")
        full_url = url.rstrip("/") + self._base_path + "/" + method
        body = {"method": method, "params": json.loads(payload.decode() or "{}")}
        async with httpx.AsyncClient() as client:
            r = await client.post(full_url, json=body)
            return r.content
