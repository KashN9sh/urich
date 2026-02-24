//! Phase 1: test handle_request without HTTP.

use urich_core::{App, CoreError, RouteId};

#[test]
fn register_and_handle() {
    let mut app = App::new();
    let id = app
        .register_route("POST", "orders/commands/create_order", None)
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
