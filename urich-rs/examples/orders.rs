//! Example: domain module — same style as Python (struct = command/query shape, handler receives typed value).

use serde::Deserialize;
use serde_json::{json, Value};
use urich_rs::{Application, Command, CoreError, DomainModule, Query};

// Command/query: just the struct, like Python. Name from type (CreateOrder → create_order).
#[derive(Debug, Deserialize, Command)]
struct CreateOrder {
    order_id: String,
}

#[derive(Debug, Deserialize, Query)]
struct GetOrder {
    order_id: String,
}

fn create_order(cmd: CreateOrder, _container: &urich_rs::Container) -> Result<Value, CoreError> {
    Ok(json!({ "ok": true, "order_id": cmd.order_id }))
}

fn get_order(query: GetOrder, _container: &urich_rs::Container) -> Result<Value, CoreError> {
    Ok(json!({ "order_id": query.order_id, "status": "created" }))
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
