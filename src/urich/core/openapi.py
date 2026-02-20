"""Minimal OpenAPI 3.0 and Swagger UI: /openapi.json and /docs."""
from __future__ import annotations

import dataclasses
from typing import Any

# (path, method) -> OpenAPI request body schema or parameters
RouteSchemas = dict[tuple[str, str], dict[str, Any]]


def _path_to_openapi(path: str) -> str:
    """Convert Starlette path to OpenAPI path (e.g. {path:path} -> {path})."""
    if "{path:path}" in path:
        return path.replace("{path:path}", "{path}")
    return path


def _py_type_to_json_type(t: type) -> str:
    if t is str or (hasattr(t, "__origin__") and t is getattr(str, "__class__", str)):
        return "string"
    if t is int:
        return "integer"
    if t is float:
        return "number"
    if t is bool:
        return "boolean"
    if t is list or t is dict:
        return "object" if t is dict else "array"
    return "string"


def schema_from_dataclass(cls: type) -> dict[str, Any]:
    """Build JSON schema from a dataclass so Swagger shows required fields and types."""
    if not dataclasses.is_dataclass(cls):
        return {"type": "object"}
    props: dict[str, Any] = {}
    required: list[str] = []
    for f in dataclasses.fields(cls):
        if f.name.startswith("_"):
            continue
        t = f.type
        try:
            if getattr(t, "__origin__", None) is type(None) or (getattr(t, "__args__", ()) and type(None) in getattr(t, "__args__", ())):
                t = next((a for a in getattr(t, "__args__", ()) if a is not type(None)), str)
            elif getattr(t, "__args__", None) and type(None) not in getattr(t, "__args__", ()):
                t = getattr(t, "__args__", (str,))[0]
        except (IndexError, TypeError):
            pass
        props[f.name] = {"type": _py_type_to_json_type(t), "description": f.name.replace("_", " ")}
        if f.default is dataclasses.MISSING and f.default_factory is dataclasses.MISSING:
            required.append(f.name)
    return {"type": "object", "properties": props, "required": required}


def parameters_from_dataclass(cls: type) -> list[dict[str, Any]]:
    """Build OpenAPI query parameters from a dataclass (for GET queries)."""
    if not dataclasses.is_dataclass(cls):
        return []
    params: list[dict[str, Any]] = []
    for f in dataclasses.fields(cls):
        if f.name.startswith("_"):
            continue
        t = f.type
        try:
            if getattr(t, "__args__", None) and type(None) in getattr(t, "__args__", ()):
                t = next((a for a in getattr(t, "__args__", ()) if a is not type(None)), str)
        except (IndexError, TypeError):
            pass
        param: dict[str, Any] = {
            "name": f.name,
            "in": "query",
            "required": f.default is dataclasses.MISSING and f.default_factory is dataclasses.MISSING,
            "schema": {"type": _py_type_to_json_type(t)},
        }
        params.append(param)
    return params


def build_openapi_spec(
    routes: list[Any],
    *,
    title: str = "API",
    version: str = "0.1.0",
    route_schemas: RouteSchemas | None = None,
) -> dict[str, Any]:
    """Build OpenAPI 3.0 spec from Starlette routes and optional per-route request schemas."""
    from starlette.routing import Route

    route_schemas = route_schemas or {}
    paths: dict[str, Any] = {}
    for route in routes:
        if not isinstance(route, Route):
            continue
        path = _path_to_openapi(route.path)
        if path not in paths:
            paths[path] = {}
        for method in route.methods or ["GET"]:
            method_lower = method.lower()
            op: dict[str, Any] = {
                "summary": f"{method} {path}",
                "responses": {
                    "200": {"description": "OK", "content": {"application/json": {"schema": {"type": "object"}}}},
                },
            }
            key = (path, method_lower)
            if key in route_schemas:
                schema = route_schemas[key]
                if "requestBody" in schema:
                    op["requestBody"] = schema["requestBody"]
                if "parameters" in schema:
                    op["parameters"] = schema["parameters"]
                if "tags" in schema:
                    op["tags"] = schema["tags"]
            if "tags" not in op:
                op["tags"] = ["default"]
            if method_lower == "post" and "/commands/" in path and "requestBody" not in op:
                op["requestBody"] = {
                    "required": True,
                    "content": {"application/json": {"schema": {"type": "object"}}},
                }
            elif method_lower == "get" and "/queries/" in path and "parameters" not in op:
                op["parameters"] = [{"name": "query params", "in": "query", "schema": {"type": "object"}}]
            paths[path][method_lower] = op
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
