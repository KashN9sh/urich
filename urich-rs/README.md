# urich-rs

Urich for Rust: DDD/CQRS-style API on top of urich-core. No Axum or Tower in your dependencies — the core runs the HTTP server.

## Example

Идея как в Python: **один модуль = один bounded context**, команды и запросы объявляются через `DomainModule`, затем `app.register(module)`.

```rust
use serde_json::{json, Value};
use urich_rs::{Application, CoreError, DomainModule};

fn create_order(body: Value) -> Result<Value, CoreError> {
    Ok(json!({ "ok": true }))
}

fn get_order(body: Value) -> Result<Value, CoreError> {
    Ok(json!({ "order_id": "1", "status": "created" }))
}

fn main() -> Result<(), Box<dyn std::error::Error + Send + Sync>> {
    let mut app = Application::new();
    let mut orders = DomainModule::new("orders")
        .command("create_order", create_order)
        .query("get_order", get_order);
    app.register(&mut orders)?;
    app.run("127.0.0.1", 8000, "Orders API", "0.1.0")
}
```

Низкоуровневая регистрация маршрута: `app.register_route("POST", "orders/commands/create_order", None, handler)?`.

Run the full example:

```bash
cargo run -p urich-rs --example orders
# Then: curl -X POST http://127.0.0.1:8000/orders/commands/create_order -H "Content-Type: application/json" -d '{}'
# And: curl http://127.0.0.1:8000/openapi.json  or  open http://127.0.0.1:8000/docs
```
