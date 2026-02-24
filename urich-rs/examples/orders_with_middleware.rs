//! Example: middleware — как в Python add_middleware (JWT и т.п.).
//! Цепочка до handler: log_request всегда пропускает; require_header возвращает 401 без X-Demo-Key.

use serde::Deserialize;
use serde_json::{json, Value};
use urich_rs::{Application, Command, CoreError, CoreResponse, DomainModule, Query, RequestContext};

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

fn log_request(ctx: &RequestContext) -> Option<CoreResponse> {
    eprintln!("  {} {}", ctx.method, ctx.path);
    None
}

fn require_demo_key(ctx: &RequestContext) -> Option<CoreResponse> {
    let path = ctx.path.trim_start_matches('/');
    if path == "docs" || path == "openapi.json" || path.starts_with("docs/") {
        return None;
    }
    let has_key = ctx
        .headers
        .iter()
        .any(|(k, v)| k.eq_ignore_ascii_case("x-demo-key") && !v.is_empty());
    if has_key {
        return None;
    }
    let body = serde_json::to_vec(&json!({ "detail": "Missing X-Demo-Key header" })).unwrap_or_default();
    Some(CoreResponse {
        status_code: 401,
        body,
    })
}

fn main() -> Result<(), Box<dyn std::error::Error + Send + Sync>> {
    let mut app = Application::new();
    app.add_middleware(|ctx| {
        let ctx = ctx.clone();
        std::future::ready(log_request(&ctx))
    });
    app.add_middleware(|ctx| {
        let ctx = ctx.clone();
        std::future::ready(require_demo_key(&ctx))
    });
    let mut orders = DomainModule::new("orders")
        .command_type::<CreateOrder>(create_order)
        .query_type::<GetOrder>(get_order);
    app.register(&mut orders)?;

    println!("Listening on http://127.0.0.1:8000");
    println!("  POST /orders/commands/create_order  (нужен заголовок X-Demo-Key)");
    println!("  GET  /orders/queries/get_order");
    println!("  GET  /openapi.json  GET  /docs");
    app.run("127.0.0.1", 8000, "Orders API (with middleware)", "0.1.0")
}
