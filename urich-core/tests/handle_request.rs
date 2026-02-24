//! Phase 1: test handle_request without HTTP.

use urich_core::{App, CoreError, RouteId};

#[test]
fn register_and_handle() {
    let mut app = App::new();
    let id = app
        .register_route("POST", "orders/commands/create_order", None, None)
        .unwrap();
    app.set_callback(Box::new(move |rid: RouteId, body: &[u8]| {
        assert_eq!(rid.0, id.0);
        assert_eq!(body, b"{\"id\":\"x\"}");
        Ok(b"{\"ok\":true}"[..].to_vec())
    }));
    let out = app
        .handle_request("POST", "orders/commands/create_order", b"{\"id\":\"x\"}")
        .unwrap();
    assert_eq!(out, b"{\"ok\":true}");
}

#[test]
fn not_found() {
    let app = App::new();
    let err = app.handle_request("GET", "unknown", b"").unwrap_err();
    match err {
        CoreError::NotFound(_) => {}
        _ => panic!("expected NotFound"),
    }
}

#[test]
fn add_command_and_add_query() {
    let mut app = App::new();
    let cmd_id = app.add_command("orders", "create_order", None).unwrap();
    let q_id = app.add_query("orders", "get_order", None).unwrap();
    app.set_callback(Box::new(move |rid: RouteId, _body: &[u8]| {
        let out: Vec<u8> = if rid.0 == cmd_id.0 {
            b"{\"created\":true}".to_vec()
        } else if rid.0 == q_id.0 {
            b"{\"id\":\"x\"}".to_vec()
        } else {
            return Err(CoreError::NotFound("unknown".into()));
        };
        Ok(out)
    }));
    let out = app
        .handle_request("POST", "orders/commands/create_order", b"{}")
        .unwrap();
    assert_eq!(out, b"{\"created\":true}");
    let out = app.handle_request("GET", "orders/queries/get_order", b"").unwrap();
    assert_eq!(out, b"{\"id\":\"x\"}");
}

#[test]
fn rpc_route_dispatch() {
    let mut app = App::new();
    app.add_rpc_route("rpc").unwrap();
    let get_foo_id = app.add_rpc_method("get_foo", None).unwrap();
    let add_id = app.add_rpc_method("add", None).unwrap();
    app.set_callback(Box::new(move |rid: RouteId, body: &[u8]| {
        if rid.0 == get_foo_id.0 {
            Ok(b"{\"value\":42}".to_vec())
        } else if rid.0 == add_id.0 {
            Ok(body.to_vec())
        } else {
            Err(CoreError::NotFound("unknown method".into()))
        }
    }));
    let body = br#"{"method":"get_foo","params":{}}"#;
    let out = app.handle_request("POST", "rpc", body).unwrap();
    assert_eq!(out, b"{\"value\":42}");
    let body2 = br#"{"method":"add","params":{"a":1,"b":2}}"#;
    let out2 = app.handle_request("POST", "rpc", body2).unwrap();
    let params_only: serde_json::Value = serde_json::from_slice(&out2).unwrap();
    assert_eq!(params_only, serde_json::json!({"a":1,"b":2}));
}

#[test]
fn subscribe_and_publish_event() {
    let mut app = App::new();
    let id1 = app.subscribe_event("OrderCreated");
    let id2 = app.subscribe_event("OrderCreated");
    let received = std::sync::Arc::new(std::sync::Mutex::new(Vec::<(u32, Vec<u8>)>::new()));
    let rec = std::sync::Arc::clone(&received);
    app.set_callback(Box::new(move |rid: RouteId, payload: &[u8]| {
        rec.lock().unwrap().push((rid.0, payload.to_vec()));
        Ok(Vec::new())
    }));
    app.publish_event("OrderCreated", b"{\"id\":\"o1\"}").unwrap();
    let v = received.lock().unwrap();
    assert_eq!(v.len(), 2);
    assert_eq!(v[0].0, id1.0);
    assert_eq!(v[0].1, b"{\"id\":\"o1\"}");
    assert_eq!(v[1].0, id2.0);
    assert_eq!(v[1].1, b"{\"id\":\"o1\"}");
}
