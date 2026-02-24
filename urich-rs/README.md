# urich-rs

Urich for Rust: DDD/CQRS-style API on top of urich-core. No Axum or Tower in your dependencies — the core runs the HTTP server.

## Example

Как в Python: **один модуль = один bounded context**, команда/запрос — структуры с полями, хендлер получает типизированный тип; затем `app.register(module)`.

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

Строковый вариант: `.command("create_order", handler)` и `.query("get_order", handler)`. Низкоуровнево: `app.register_route(..., handler, openapi_tag)?`.

Run the full example:

```bash
cargo run -p urich-rs --example orders
# Then: curl -X POST http://127.0.0.1:8000/orders/commands/create_order -H "Content-Type: application/json" -d '{}'
# And: curl http://127.0.0.1:8000/openapi.json  or  open http://127.0.0.1:8000/docs
```
