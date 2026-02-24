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

## Callback and schema contract (implemented)

**Core API (Rust):**

- `register_route(method, path, request_schema: Option<Value>, openapi_tag: Option<&str>) -> Result<RouteId>` — path is exact (e.g. `orders/commands/create_order`). Schema is JSON Schema (optional). openapi_tag used for OpenAPI tags (e.g. context name).
- `set_callback(cb: RequestCallback)` — `RequestCallback = Box<dyn Fn(RouteId, &[u8]) -> Result<Vec<u8>, CoreError> + Send + Sync>`. The core calls this with (route_id, validated body bytes); the host returns response bytes.
- `handle_request(method, path, body) -> Result<Vec<u8>>` — no HTTP; used by tests and by the HTTP layer when it receives a request.
- `openapi_spec(title, version) -> Value` — minimal OpenAPI 3.0 JSON from registered routes.

**Schema:** Core accepts optional JSON Schema for the request body. Validation is done in the core (currently parse-only; full jsonschema check can be added). Facades pass schema when registering (Python: from Pydantic/dataclass; Rust: from serde/schemars).

**Implemented:** See repo — `urich-core/` (Rust), `urich-rs/` (Rust facade + example), `urich-python/` (PyO3 bindings; build with maturin). The Python facade lives in `src/urich/`: Application, DomainModule, etc., use the core via `urich_core_native`; `app.run(host, port)` calls `core.run()`. Single package `urich` = core (native) + Python facade; dependency on the core is direct, not optional.
