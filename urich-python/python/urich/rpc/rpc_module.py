"""
RpcModule — building block for RPC: server (accept calls) and client (call other services).
Configure via .server(...) and .client(...); register with app.register(rpc).
Transport and serialization — user's choice; optional minimal HTTP+JSON out of the box.
"""
from __future__ import annotations

from typing import Any, Callable

from urich.core.app import Application
from urich.core.request import Request
from urich.core.responses import Response
from urich.core.module import Module
from urich.discovery.protocol import ServiceDiscovery
from urich.rpc.protocol import RpcError, RpcServerHandler, RpcTransport


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
        handler: RpcServerHandler | type | None = None,
    ) -> RpcModule:
        """Route for incoming RPC. handler: instance or class (then registered and resolved from container)."""
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
            if self._server_handler is not None and isinstance(self._server_handler, type):
                app.container.register_class(self._server_handler)
            app.add_rpc_route(self._server_path)
            if self._server_handler is not None:
                h = self._server_handler
                methods = self._rpc_method_names(h)
                for method_name in methods:
                    app.add_rpc_method(
                        method_name,
                        self._make_rpc_method_endpoint(app, method_name),
                    )
        if self._client_discovery is not None:
            app.container.register_instance(ServiceDiscovery, self._client_discovery)
        if self._client_transport is not None:
            app.container.register_instance(RpcTransport, self._client_transport)
            app.container.register_class(RpcClient)

    def _rpc_method_names(self, handler: Any) -> list[str]:
        if hasattr(handler, "__rpc_methods__"):
            return list(handler.__rpc_methods__)
        obj = handler if not isinstance(handler, type) else handler
        return [
            m for m in dir(obj)
            if not m.startswith("_") and m != "handle" and callable(getattr(obj, m, None))
        ]

    def _make_rpc_method_endpoint(self, app: Application, method_name: str) -> Callable:
        import json

        async def endpoint(request: Request) -> Response:
            try:
                params = await request.json()
            except Exception:
                params = {}
            payload_bytes = json.dumps(params).encode()
            h = app.container.resolve(self._server_handler) if isinstance(self._server_handler, type) else self._server_handler
            result = await h.handle(method_name, payload_bytes)
            return Response(content=result, media_type="application/json")
        return endpoint


class RpcServer:
    """
    Server facade: implement methods like get_employee(self, employee_id: str) -> dict | None.
    handle() dispatches by method name, parses JSON params, calls self.<method>(**params), serializes result.
    Return None for "not found"; raise RpcError for errors (returned as standard error envelope).
    """

    async def handle(self, method: str, payload: bytes) -> bytes:
        import json

        params = {}
        if payload:
            try:
                params = json.loads(payload.decode() or "{}")
            except Exception:
                pass
        if not isinstance(params, dict):
            params = {}

        name = (method or "").replace("/", "_").strip()
        handler_fn = getattr(self, name, None) if name else None
        if not callable(handler_fn):
            return json.dumps({"error": {"code": "NOT_FOUND", "message": f"unknown method {method!r}"}}).encode()

        try:
            result = handler_fn(**params)
            if hasattr(result, "__await__"):
                result = await result
        except RpcError as e:
            return json.dumps({"error": {"code": e.code, "message": e.message}}).encode()
        except Exception as e:
            return json.dumps({"error": {"code": "INTERNAL", "message": str(e)}}).encode()

        return json.dumps(result).encode()


# Standard error envelope: {"error": {"code": "...", "message": "..."}} or {"error": "string"}
def _is_error_response(data: dict) -> bool:
    return isinstance(data, dict) and "error" in data


class RpcClient:
    """
    Facade: call(service_name, method, params) -> result dict or None.
    Uses ServiceDiscovery + RpcTransport; JSON encode/decode inside.
    On server error envelope or transport failure: return None or raise RpcError (see raise_on_error).
    """

    def __init__(self, discovery: ServiceDiscovery, transport: RpcTransport) -> None:
        self._discovery = discovery
        self._transport = transport

    async def call(
        self,
        service_name: str,
        method: str,
        params: dict,
        *,
        raise_on_error: bool = False,
    ) -> dict | Any | None:
        import json

        urls = self._discovery.resolve(service_name)
        if not urls:
            if raise_on_error:
                raise RpcError("SERVICE_UNAVAILABLE", f"Service {service_name!r} not found")
            return None
        try:
            payload = json.dumps(params).encode()
            result = await self._transport.call(urls[0], method, payload)
            data = json.loads(result.decode()) if result else None
        except Exception as e:
            if raise_on_error:
                raise RpcError("TRANSPORT_ERROR", str(e)) from e
            return None
        if _is_error_response(data):
            err = data["error"]
            if isinstance(err, dict):
                code = err.get("code", "UNKNOWN")
                msg = err.get("message", str(err))
            else:
                code = "UNKNOWN"
                msg = str(err)
            if raise_on_error:
                raise RpcError(code, msg)
            return None
        return data


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
