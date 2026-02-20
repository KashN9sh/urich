# OpenAPI & Swagger

After registering all modules, call **`app.openapi(...)`** to expose the OpenAPI 3.0 spec and Swagger UI.

---

## Basic usage

```python
app = Application()
# ... app.register(module) ...
app.openapi(title="My API", version="0.1.0")
```

**Parameters:**

| Parameter | Default | Description |
|-----------|---------|-------------|
| `title` | `"API"` | API title in the spec and Swagger. |
| `version` | `"0.1.0"` | API version. |
| `docs_path` | `"/docs"` | Path for Swagger UI. |
| `openapi_path` | `"/openapi.json"` | Path for the OpenAPI JSON spec. |

This adds two GET routes:

- **GET /openapi.json** — OpenAPI 3.0 spec (paths, schemas, etc.).
- **GET /docs** — HTML page with Swagger UI that loads the spec from `openapi_path`.

---

## Request schemas for commands and queries

DomainModule registers routes with **request body** (commands, POST queries) or **query parameters** (GET queries). The framework builds OpenAPI schemas from your **dataclass** types so Swagger shows required fields and types.

- **Command** (POST): request body schema is derived from the command dataclass (e.g. `CreateOrder` → `order_id`, `customer_id`, `total_cents`).
- **Query** (GET): query parameters are derived from the query dataclass (e.g. `GetOrder` → `order_id`).
- **Query** (POST): same as command; body schema from the query dataclass.

So you don’t need to write OpenAPI by hand for standard command/query endpoints.

---

## Custom routes and OpenAPI

If you add routes manually with **`app.add_route()`**, you can pass optional OpenAPI metadata:

```python
app.add_route(
    "/custom",
    my_endpoint,
    methods=["POST"],
    openapi_body_schema={"type": "object", "properties": {"name": {"type": "string"}}, "required": ["name"]},
)
```

For GET with query parameters:

```python
app.add_route(
    "/search",
    search_endpoint,
    methods=["GET"],
    openapi_parameters=[
        {"name": "q", "in": "query", "required": True, "schema": {"type": "string"}},
    ],
)
```

Helper functions (from `urich.core.openapi`) if you use dataclasses:

- **`schema_from_dataclass(cls)`** — Returns a JSON Schema object for the dataclass (for request body).
- **`parameters_from_dataclass(cls)`** — Returns a list of OpenAPI parameter dicts for query (for GET).

---

## How the spec is built

`build_openapi_spec(routes, title=..., version=..., route_schemas=...)` walks the Starlette routes, and for each `(path, method)` that has an entry in `route_schemas` it merges `requestBody` and/or `parameters` into the operation. DomainModule fills `route_schemas` when it calls `app.add_route(..., openapi_body_schema=..., openapi_parameters=...)`. Other routes get generic placeholders (e.g. POST commands get a generic `object` body if no schema was provided).
