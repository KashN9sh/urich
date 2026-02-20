"""Minimal OpenAPI 3.0 and Swagger UI: /openapi.json and /docs."""
from __future__ import annotations

from typing import Any


def _path_to_openapi(path: str) -> str:
    """Convert Starlette path to OpenAPI path (e.g. {path:path} -> {path})."""
    if "{path:path}" in path:
        return path.replace("{path:path}", "{path}")
    return path


def build_openapi_spec(routes: list[Any], *, title: str = "API", version: str = "0.1.0") -> dict[str, Any]:
    """Build OpenAPI 3.0 spec from Starlette routes list."""
    from starlette.routing import Route

    paths: dict[str, Any] = {}
    for route in routes:
        if not isinstance(route, Route):
            continue
        path = _path_to_openapi(route.path)
        if path not in paths:
            paths[path] = {}
        for method in route.methods or ["GET"]:
            method_lower = method.lower()
            paths[path][method_lower] = {
                "summary": f"{method} {path}",
                "responses": {
                    "200": {"description": "OK", "content": {"application/json": {"schema": {"type": "object"}}}},
                },
            }
            if method_lower == "post" and "/commands/" in path:
                paths[path][method_lower]["requestBody"] = {
                    "required": True,
                    "content": {"application/json": {"schema": {"type": "object"}}},
                }
            if method_lower == "get" and "/queries/" in path:
                paths[path][method_lower]["parameters"] = [
                    {"name": "query params", "in": "query", "schema": {"type": "object"}},
                ]
    return {
        "openapi": "3.0.0",
        "info": {"title": title, "version": version},
        "paths": paths,
    }


SWAGGER_UI_HTML = """<!DOCTYPE html>
<html>
<head>
  <link rel="stylesheet" type="text/css" href="https://unpkg.com/swagger-ui-dist@5/swagger-ui.css">
</head>
<body>
  <div id="swagger-ui"></div>
  <script src="https://unpkg.com/swagger-ui-dist@5/swagger-ui-bundle.js"></script>
  <script>
    SwaggerUIBundle({
      url: "/openapi.json",
      dom_id: "#swagger-ui",
    });
  </script>
</body>
</html>
"""
