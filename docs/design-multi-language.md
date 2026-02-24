# Multi-language design: core + facades

**Architecture:** one **Urich core** (shared implementation) and two **facades** — Python and Rust. There is no separate "main" or "default" package: the Python facade *is* the package `urich` for Python; the Rust facade is `urich-rs`. Both depend only on the core. You change the core once; the facades only plug the core into the host — they don't reimplement logic.

---

## Idea

**Urich depends only on the Urich core.** No Starlette, no Axum, no third-party web framework. The core *is* the server: it accepts HTTP, routes, parses, validates, calls your handler (via a callback into the host), serializes the response, and sends it. The Python and Rust "facades" are just the bindings to register your handlers with the core and start it — they don't implement transport themselves.

```
┌─────────────────────────────────────────────────────────────────┐
│  Urich Core (C or Rust)                                         │
│  — HTTP server (listen, accept, read request)                   │
│  — Routing (method + path → route id)                           │
│  — JSON parse + validate (schema per command/query)             │
│  — Call host handler (callback: route_id + payload → result)    │
│  — Response encode (JSON) + send                                │
│  — Conventions (path shapes, CQRS naming), OpenAPI             │
│  So: core = full stack; facades only register handlers & run   │
└─────────────────────────────────────────────────────────────────┘
           ▲                                      ▲
           │ FFI (register handler, start)        │ native (register handler, start)
           │                                      │
┌──────────┴──────────┐               ┌──────────┴──────────┐
│  Python (urich)     │               │  Rust (urich)       │
│  — Depends ONLY on │               │  — Depends ONLY on  │
│    Urich core       │               │    Urich core       │
│  — Register Python │               │  — Register Rust     │
│    callables; run   │               │    closures; run    │
│    core.run()       │               │    core.run()        │
│  No Starlette,      │               │  No Axum, no Tower  │
│  no ASGI lib        │               │  no hyper in your   │
│                     │               │  deps               │
└─────────────────────┘               └─────────────────────┘
```

- **Core** (`urich-core`) = HTTP + routing + parse + validate + serialize. It runs the server and invokes the host (Python or Rust) only to execute the handler for the matched route. One implementation for both languages.
- **Facades** = language bindings only: "register this handler for this route", "start the core". Your app depends only on `urich` (Python) or `urich-rs` (Rust). The Python package `urich` depends directly on the core (e.g. `urich-core-native` from the same repo); one wheel can be built with maturin (core + Python facade). No Starlette, no ASGI, no extra web framework.

When you change the core, both languages get it. Facades change only if the registration/start API of the core changes.

---

## What the core would expose (to the facades)

- **Registration:** register routes with (method, path pattern, schema for body). Core stores route id and schema. Optionally register a handler id per route (the facade maps that to a Python callable or Rust closure).
- **Run:** `run(host, port)` or equivalent. Core starts the HTTP server. On each request it: matches route, parses and validates body, calls the host via a registered callback (route_id + validated payload), gets the result, encodes and sends the response. The host (Python/Rust) only implements that callback — "given this payload, return this result".
- **OpenAPI:** core can produce the spec from registered routes and schemas so both languages serve the same docs.

So the facade does not run a server or implement HTTP. It only: (1) registers routes and schemas with the core, (2) registers the callback(s) the core will invoke for handler execution, (3) calls `run`. Your handler code receives the payload the core already validated; you can still use Pydantic/serde on the facade side to map that into nice types, but validation itself is in the core.

---

## Core language: C vs Rust

- **C:** maximum portability; Python (C API, ctypes) and Rust (extern "C") both consume it. You maintain one C codebase and build scripts.
- **Rust:** one codebase, safer and easier to extend. Rust facade uses the core as a crate. Python facade uses the same core via PyO3 (Rust → Python extension) or a small C ABI shim generated from Rust. Then "core" and "Rust facade" can even live in the same repo (workspace); Python is a separate package that depends on the core binary.

Either way, the **facades stay thin**: they only adapt between the core API and the host language and runtime.

---

## Workflow

1. Change the core (bug fix, new convention, new API).
2. Build the core (e.g. `make` or `cargo build`).
3. Update facades so they call the new core API (e.g. new function, changed struct). This can be manual or partly automated (e.g. codegen from a small IDL that the core and facades share).
4. Test Python and Rust apps; release.

So: **one place for everything (the core); facades are only "register handlers + run".**

---

## Declarative contract: facade describes, core assembles

**Two phases:**

1. **Construction:** The facade calls the core’s builder API; the core owns all structure (routes, OpenAPI, RPC methods, event subscriptions). The facade does not store route tables or subscription maps — only the mapping `handler_id → callable` for execution.
2. **Execution:** On each request (or on `publish_event`), the core determines `handler_id`, validates the payload, and calls the facade once: **execute(handler_id, body)**. The facade looks up the callable and returns the result.

So the only contract from core to facade is one callback: **execute(handler_id, body) → response bytes**. Commands, queries, RPC, and events all use it; the core decides how to choose `handler_id` (path, RPC method name, or event type).

**Unified behaviour:** Path conventions, RPC body format `{ "method", "params" }`, and event type identifiers (strings) are defined in the core so that Rust and Python behave the same.

---

## Callback and schema contract (implemented)

**Core API (Rust):**

- **Construction (facade → core):**
  - `add_command(context, name, request_schema) -> RouteId` — POST `{context}/commands/{name}`.
  - `add_query(context, name, request_schema) -> RouteId` — GET `{context}/queries/{name}`.
  - `add_rpc_route(path)` — one POST route for RPC; body = `{ "method": string, "params": object }`.
  - `add_rpc_method(name, request_schema) -> RouteId` — register RPC method; core dispatches by `method` and calls execute(handler_id, params_bytes).
  - `subscribe_event(event_type_id: &str) -> RouteId` — event type is a string (e.g. `"OrderCreated"`); core calls execute(handler_id, payload) on `publish_event`.
  - `publish_event(event_type_id, payload)` — core invokes execute for each subscriber.
  - `register_route(method, path, request_schema, openapi_tag) -> RouteId` — for custom routes only.
- **Execution (core → facade):**
  - `set_callback(cb)` — **execute(handler_id, payload, context):** callback receives `(RouteId, &[u8], &RequestContext)` and returns `Result<Response, CoreError>` where `Response { status_code, body }`. `RequestContext` has `method`, `path`, `headers`, `body` so the facade can run middlewares (e.g. JWT) before the route handler.
- **Other:**
  - `handle_request(context: &RequestContext) -> Result<Response>` — match route, validate, call callback; used by HTTP layer and tests.
  - **Middlewares:** The facade (e.g. Python) runs a chain of middlewares before the handler: each middleware gets `Request` (with `headers`, `path`) and returns `None` to continue or `Response` (e.g. 401) to short-circuit. Core passes full request context into the single callback; the facade implements the middleware chain.
  - `openapi_spec(title, version) -> Value` — minimal OpenAPI 3.0 from registered routes.

**Schema:** Core accepts optional JSON Schema for request body (and for RPC params). Facades pass schema when registering (Python: from Pydantic/dataclass; Rust: from serde/schemars).

**Implemented:** See repo — `urich-core/` (Rust), `urich-rs/` (Rust facade + example), `urich-python/` (PyO3 bindings; build with maturin). The Python facade in `src/urich/` uses `urich_core_native`; Application, DomainModule, RpcModule use the core’s add_command/add_query/add_rpc_*/subscribe_event; `app.run(host, port)` calls `core.run()`. Single package `urich` = core (native) + Python facade; dependency on the core is direct, not optional.

---

## How an application is built (Python and Rust)

The facade only **describes** the app to the core (via the core API) and keeps **handler_id → callable** for `execute(handler_id, body)`. The core owns routes, OpenAPI, RPC methods, event subscriptions.

### Python application

**What you write:**

```python
from urich import Application
from urich.ddd import DomainModule
from urich.ddd.commands import Command, Query
from dataclasses import dataclass

@dataclass
class CreateOrder(Command):
    order_id: str

@dataclass
class GetOrder(Query):
    order_id: str

async def create_order_handler(cmd: CreateOrder):
    return {"ok": True, "order_id": cmd.order_id}

async def get_order_handler(query: GetOrder):
    return {"order_id": query.order_id, "status": "created"}

app = Application()
orders = DomainModule("orders").command(CreateOrder, create_order_handler).query(GetOrder, get_order_handler)
app.register(orders)
app.openapi(title="Orders API", version="0.1.0")
app.run(host="127.0.0.1", port=8000)
```

**What happens under the hood:**

1. `app.register(orders)` → `orders.register_into(app)`.
2. DomainModule calls the **core** via app: `app.add_command("orders", "create_order", endpoint, request_schema=...)` → core `add_command("orders", "create_order", schema)` returns `handler_id`; facade stores `handler_id → create_order_handler`. Same for `add_query("orders", "get_order", ...)`.
3. Paths are **not** built in Python: the core builds `orders/commands/create_order` and `orders/queries/get_order`.
4. `app.run()` → facade sets **one** callback `execute(handler_id, body)` that looks up `handler_id` and runs the Python callable; then calls core `run(host, port, ...)`. Core starts HTTP; on each request it matches route → `handler_id` → calls facade's `execute(handler_id, body)`.

So: **Python** only describes (context + command/query names + handlers) and implements **execute**; the **core** owns routes and dispatch.

### Rust application

**What you write (same idea as Python: struct = shape of command/query, handler receives typed value):**

```rust
use serde::Deserialize;
use serde_json::{json, Value};
use urich_rs::{Application, Command, CoreError, DomainModule, Query};

#[derive(Deserialize, Command)]
struct CreateOrder {
    order_id: String,
}

#[derive(Deserialize, Query)]
struct GetOrder {
    order_id: String,
}

fn create_order(cmd: CreateOrder) -> Result<Value, CoreError> {
    Ok(json!({ "ok": true, "order_id": cmd.order_id }))
}

fn get_order(query: GetOrder) -> Result<Value, CoreError> {
    Ok(json!({ "order_id": query.order_id, "status": "created" }))
}

fn main() -> Result<(), Box<dyn std::error::Error + Send + Sync>> {
    let mut app = Application::new();
    let mut orders = DomainModule::new("orders")
        .command_type::<CreateOrder>(create_order)
        .query_type::<GetOrder>(get_order);
    app.register(&mut orders)?;
    app.run("127.0.0.1", 8000, "Orders API", "0.1.0")
}
```

**What happens under the hood:**

1. `app.register(&mut orders)` → `orders.register_into(app)`.
2. DomainModule calls the **core** via app: `app.add_command("orders", "create_order", None, handler, Some("orders"))` → core `add_command("orders", "create_order", schema)` returns `RouteId`; facade stores `RouteId → handler`. Same for `add_query("orders", "get_order", ...)`.
3. Paths are **not** built in Rust: the core builds `orders/commands/create_order` and `orders/queries/get_order`.
4. `app.run(...)` → facade installs **one** callback `execute(handler_id, body)` that looks up `handler_id` in its `handlers` map and runs the closure; then calls core `run(...)`. Core starts HTTP; on each request it matches route → `handler_id` → calls facade's callback.

So: **Rust** only describes (context + command/query names + handlers) and implements **execute**; the **core** owns routes and dispatch.

**Rust aligned with Python:** In both, the command/query type is a **struct/dataclass with fields** (e.g. `CreateOrder(order_id: str)`). The framework parses the request body and builds that instance, so the handler receives a typed object. In the minimal Rust example, the type is only a **name marker** (`Command::name() -> "create_order"`); the body is deserialized into the struct and the handler receives it (same as Python). In Rust you can describe the structure the same way: use a struct with fields and `serde::Deserialize`, deserialize the body in the handler (or in a wrapper), and pass a typed struct — then both languages “describe the command/query structure” and the handler is typed. The core only sees the route name and JSON; the facade can add this typed layer in both languages.

### Summary

| Step | Python | Rust | Core |
|------|--------|------|------|
| Describe context + commands/queries | `DomainModule("orders").command(...).query(...)` | `DomainModule::new("orders").command_type(...).query_type(...)` | — |
| Register with core | `app.register(orders)` → facade calls `core.add_command`, `core.add_query` | same | Stores routes, returns handler_id |
| Store handler | Facade: `handler_id → callable` | Facade: `handler_id → closure` | — |
| Run | `app.run()` → facade sets `execute(handler_id, body)`, then `core.run()` | same | HTTP, route → handler_id, calls execute(handler_id, body) |

RPC and events follow the same pattern: facade calls `add_rpc_route`, `add_rpc_method`, `subscribe_event`; core owns the registries; facade only keeps handler_id → callable and implements **execute(handler_id, body)**.
