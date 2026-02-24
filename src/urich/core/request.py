"""Minimal request object for core-backed endpoints (no Starlette)."""
from __future__ import annotations

import json
from typing import Any


class Request:
    """Request-like object: body (JSON), method, path, headers, query_params. Used by handlers and middlewares."""

    def __init__(
        self,
        body_bytes: bytes,
        method: str,
        *,
        path: str = "",
        query_params: dict[str, str] | None = None,
        headers: dict[str, str] | list[tuple[str, str]] | None = None,
    ) -> None:
        self._body = json.loads(body_bytes) if body_bytes else {}
        self.method = method
        self._query = query_params or {}
        self.path = path
        self.path_params = {"path": path} if path else {}
        if isinstance(headers, dict):
            self._headers = {k.lower(): v for k, v in headers.items()}
        elif isinstance(headers, list):
            self._headers = {str(k).lower(): str(v) for k, v in headers}
        else:
            self._headers = {}
        self.state: dict[str, Any] = {}  # for middlewares (e.g. request.state["user"])

    async def json(self) -> dict[str, Any]:
        return self._body

    @property
    def query_params(self) -> dict[str, str]:
        return self._query

    @property
    def headers(self) -> dict[str, str]:
        """Request headers (lowercased names). From core context for middlewares (e.g. Authorization)."""
        return self._headers
