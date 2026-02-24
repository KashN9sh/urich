# Roadmap

Public view of what’s done and what we’re aiming for.

## Done

- **Core:** Application, modules, DI container, config. OpenAPI 3.0 + Swagger UI.
- **DDD:** DomainModule (aggregate, repository, commands, queries, event handlers). CQRS by convention. Domain events, EventBus.
- **Events:** EventBusModule, OutboxModule (protocols in core; you plug storage/publisher).
- **Discovery & RPC:** DiscoveryModule, RpcModule. Static discovery; optional JsonHttpRpcTransport.
- **CLI:** `urich create-app`, `add-context`, `add-aggregate` — scaffold app and bounded context.
- **Docs & examples:** Getting started, architecture, ecommerce example.

## Planned

- **Event contracts** — guidance on versions and formats (e.g. Pydantic), no hard dependency.
- **CRUD endpoints** — optional generation per aggregate.
- **Documentation** — layers and DDD conventions within the framework.
- **Adapters** — more out-of-the-box or example adapters (e.g. Redis event bus, Consul discovery) as optional packages or in `examples/`.

**Multi-language (initial):** **urich-core** (Rust) — HTTP, routing, validation, OpenAPI. **urich-rs** — Rust facade (see `urich-rs/`, example: `cargo run -p urich-rs --example orders`). **urich-python** — PyO3 (build with maturin). See [Multi-language design](design-multi-language.md).

Ideas and contributions are welcome; open an issue or see [Contributing](https://github.com/KashN9sh/urich/blob/main/CONTRIBUTING.md).
