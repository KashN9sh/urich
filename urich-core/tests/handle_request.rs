//! Phase 1: test handle_request without HTTP (async).

use std::future::ready;
use urich_core::{App, CoreError, RequestContext, Response, RouteId};

fn ctx(method: &str, path: &str, body: &[u8]) -> RequestContext {
    RequestContext {
        method: method.to_string(),
        path: path.to_string(),
        headers: vec![],
        body: body.to_vec(),
    }
}

fn ok_body(body: &[u8]) -> Response {
    Response {
        status_code: 200,
        body: body.to_vec(),
        content_type: None,
    }
}

#[tokio::test]
async fn register_and_handle() {
    let mut app = App::new();
    let id = app
        .register_route("POST", "orders/commands/create_order", None, None)
        .unwrap();
    app.set_callback(Box::new(move |rid: RouteId, body: &[u8], _ctx: &RequestContext| {
        assert_eq!(rid.0, id.0);
        assert_eq!(body, b"{\"id\":\"x\"}");
        Box::pin(ready(Ok(ok_body(b"{\"ok\":true}"))))
    }));
    let out = app
        .handle_request(&ctx("POST", "orders/commands/create_order", b"{\"id\":\"x\"}"))
        .await
        .unwrap();
    assert_eq!(out.status_code, 200);
    assert_eq!(out.body, b"{\"ok\":true}");
}

#[tokio::test]
async fn not_found() {
    let app = App::new();
    let err = app.handle_request(&ctx("GET", "unknown", b"")).await.unwrap_err();
    match err {
        CoreError::NotFound(_) => {}
        _ => panic!("expected NotFound"),
    }
}

#[tokio::test]
async fn add_command_and_add_query() {
    let mut app = App::new();
    let cmd_id = app.add_command("orders", "create_order", None).unwrap();
    let q_id = app.add_query("orders", "get_order", None).unwrap();
    app.set_callback(Box::new(move |rid: RouteId, _body: &[u8], _ctx: &RequestContext| {
        let out: Vec<u8> = if rid.0 == cmd_id.0 {
            b"{\"created\":true}".to_vec()
        } else if rid.0 == q_id.0 {
            b"{\"id\":\"x\"}".to_vec()
        } else {
            return Box::pin(ready(Err(CoreError::NotFound("unknown".into()))));
        };
        Box::pin(ready(Ok(Response { status_code: 200, body: out, content_type: None })))
    }));
    let out = app.handle_request(&ctx("POST", "orders/commands/create_order", b"{}")).await.unwrap();
    assert_eq!(out.body, b"{\"created\":true}");
    let out = app.handle_request(&ctx("GET", "orders/queries/get_order", b"")).await.unwrap();
    assert_eq!(out.body, b"{\"id\":\"x\"}");
}

#[tokio::test]
async fn rpc_route_dispatch() {
    let mut app = App::new();
    app.add_rpc_route("rpc").unwrap();
    let get_foo_id = app.add_rpc_method("get_foo", None).unwrap();
    let add_id = app.add_rpc_method("add", None).unwrap();
    app.set_callback(Box::new(move |rid: RouteId, body: &[u8], _ctx: &RequestContext| {
        if rid.0 == get_foo_id.0 {
            Box::pin(ready(Ok(ok_body(b"{\"value\":42}"))))
        } else if rid.0 == add_id.0 {
            Box::pin(ready(Ok(ok_body(body))))
        } else {
            Box::pin(ready(Err(CoreError::NotFound("unknown method".into()))))
        }
    }));
    let body = br#"{"method":"get_foo","params":{}}"#;
    let out = app.handle_request(&ctx("POST", "rpc", body)).await.unwrap();
    assert_eq!(out.body, b"{\"value\":42}");
    let body2 = br#"{"method":"add","params":{"a":1,"b":2}}"#;
    let out2 = app.handle_request(&ctx("POST", "rpc", body2)).await.unwrap();
    let params_only: serde_json::Value = serde_json::from_slice(&out2.body).unwrap();
    assert_eq!(params_only, serde_json::json!({"a":1,"b":2}));
}

#[tokio::test]
async fn subscribe_and_publish_event() {
    let mut app = App::new();
    let id1 = app.subscribe_event("OrderCreated");
    let id2 = app.subscribe_event("OrderCreated");
    let received = std::sync::Arc::new(std::sync::Mutex::new(Vec::<(u32, Vec<u8>)>::new()));
    let rec = std::sync::Arc::clone(&received);
    app.set_callback(Box::new(move |rid: RouteId, payload: &[u8], _ctx: &RequestContext| {
        rec.lock().unwrap().push((rid.0, payload.to_vec()));
        Box::pin(ready(Ok(Response { status_code: 200, body: Vec::new(), content_type: None })))
    }));
    app.publish_event("OrderCreated", b"{\"id\":\"o1\"}").await.unwrap();
    let v = received.lock().unwrap();
    assert_eq!(v.len(), 2);
    assert_eq!(v[0].0, id1.0);
    assert_eq!(v[0].1, b"{\"id\":\"o1\"}");
    assert_eq!(v[1].0, id2.0);
    assert_eq!(v[1].1, b"{\"id\":\"o1\"}");
}
