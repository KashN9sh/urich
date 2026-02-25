"""Minimal response types for core-backed endpoints (no Starlette)."""
from __future__ import annotations

import json
from typing import Any


class Response:
    """Response with .body (bytes) and .status_code (for middlewares and auth)."""

    def __init__(
        self,
        content: bytes,
        media_type: str = "application/json",
        status_code: int = 200,
    ) -> None:
        self.body = content if isinstance(content, bytes) else content.encode()
        self.media_type = media_type
        self.status_code = status_code


class JSONResponse(Response):
    """JSON response; content is serialized to bytes."""

    def __init__(
        self,
        content: dict[str, Any] | list[Any],
        status_code: int = 200,
        **kwargs: Any,
    ) -> None:
        body = json.dumps(content).encode()
        super().__init__(body, media_type="application/json", status_code=status_code)
