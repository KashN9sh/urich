//! Example: domain module (bounded context) — type-based API like Python DomainModule.

use serde_json::{json, Value};
use urich_rs::{Application, Command, CoreError, DomainModule, Query};

// Command/query types: route name comes from the type (like Python CreateOrder → create_order).
struct CreateOrder;
impl Command for CreateOrder {
    fn name() -> &'static str {
        "create_order"
    }
}

struct GetOrder;
impl Query for GetOrder {
    fn name() -> &'static str {
        "get_order"
    }
}

fn create_order(body: Value) -> Result<Value, CoreError> {
    let id = body.get("order_id").and_then(|v| v.as_str()).unwrap_or("?");
    Ok(json!({ "ok": true, "order_id": id }))
}

fn get_order(body: Value) -> Result<Value, CoreError> {
    let id = body.get("order_id").and_then(|v| v.as_str()).unwrap_or("?");
    Ok(json!({ "order_id": id, "status": "created" }))
}

fn main() -> Result<(), Box<dyn std::error::Error + Send + Sync>> {
    let mut app = Application::new();
    let mut orders = DomainModule::new("orders")
        .command_type::<CreateOrder>(create_order)
        .query_type::<GetOrder>(get_order);
    app.register(&mut orders)?;

    println!("Listening on http://127.0.0.1:8000");
    println!("  POST /orders/commands/create_order");
    println!("  GET  /orders/queries/get_order");
    println!("  GET  /openapi.json  GET  /docs");
    app.run("127.0.0.1", 8000, "Orders API", "0.1.0")
}
