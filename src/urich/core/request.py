"""Minimal request object for core-backed endpoints (no Starlette)."""
from __future__ import annotations

import json
from typing import Any


class Request:
    """Request-like object: body (JSON), method, query_params, path_params."""

    def __init__(
        self,
        body_bytes: bytes,
        method: str,
        *,
        path: str = "",
        query_params: dict[str, str] | None = None,
    ) -> None:
        self._body = json.loads(body_bytes) if body_bytes else {}
        self.method = method
        self._query = query_params or {}
        self.path_params = {"path": path} if path else {}

    async def json(self) -> dict[str, Any]:
        return self._body

    @property
    def query_params(self) -> dict[str, str]:
        return self._query
