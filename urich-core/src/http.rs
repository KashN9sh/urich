//! Minimal HTTP server: accept request, call handle_request, send response. Serves /openapi.json and /docs.

use crate::{App, CoreError};
use std::sync::{Arc, RwLock};

const SWAGGER_UI_HTML: &str = r#"<!DOCTYPE html>
<html>
<head><title>Swagger UI</title><link rel="stylesheet" href="https://unpkg.com/swagger-ui-dist@5/swagger-ui.css"></head>
<body><div id="swagger-ui"></div>
<script src="https://unpkg.com/swagger-ui-dist@5/swagger-ui-bundle.js"></script>
<script>SwaggerUIBundle({ url: '/openapi.json', dom_id: '#swagger-ui' });</script>
</body>
</html>"#;

/// Run HTTP server; blocks. Serves POST/GET to registered routes, GET /openapi.json, GET /docs.
pub fn run(
    app: Arc<RwLock<App>>,
    host: &str,
    port: u16,
    openapi_title: &str,
    openapi_version: &str,
) -> Result<(), Box<dyn std::error::Error + Send + Sync>> {
    let addr = format!("{}:{}", host, port);
    let server = tiny_http::Server::http(&addr)?;
    for mut request in server.incoming_requests() {
        let method = format!("{}", request.method());
        let url = request.url().to_string();
        let path = url
            .split('?')
            .next()
            .unwrap_or(&url)
            .trim_start_matches('/')
            .to_string();
        let path_with_slash = format!("/{}", path);

        if path == "openapi.json" || path_with_slash == "/openapi.json" {
            let app_guard = app.read().map_err(|e| e.to_string())?;
            let spec = app_guard.openapi_spec(openapi_title, openapi_version);
            let body = serde_json::to_string(&spec).unwrap_or_default();
            let response = tiny_http::Response::from_string(body)
                .with_status_code(200)
                .with_header(tiny_http::Header::from_bytes(&b"Content-Type"[..], &b"application/json"[..]).unwrap());
            request.respond(response)?;
            continue;
        }
        if path == "docs" || path_with_slash == "/docs" {
            let response = tiny_http::Response::from_string(SWAGGER_UI_HTML)
                .with_status_code(200)
                .with_header(tiny_http::Header::from_bytes(&b"Content-Type"[..], &b"text/html"[..]).unwrap());
            request.respond(response)?;
            continue;
        }

        let mut body_vec = Vec::new();
        let _ = std::io::Read::read_to_end(&mut request.as_reader(), &mut body_vec);
        let body = body_vec.as_slice();

        let result = {
            let app_guard = app.read().map_err(|e| CoreError::Validation(e.to_string()))?;
            app_guard.handle_request(&method, &path, body)
        };

        match result {
            Ok(resp_bytes) => {
                let response = tiny_http::Response::from_data(resp_bytes)
                    .with_status_code(200)
                    .with_header(tiny_http::Header::from_bytes(&b"Content-Type"[..], &b"application/json"[..]).unwrap());
                request.respond(response)?;
            }
            Err(CoreError::NotFound(msg)) => {
                let body = serde_json::json!({ "error": msg });
                let response = tiny_http::Response::from_string(body.to_string())
                    .with_status_code(404)
                    .with_header(tiny_http::Header::from_bytes(&b"Content-Type"[..], &b"application/json"[..]).unwrap());
                request.respond(response)?;
            }
            Err(e) => {
                let body = serde_json::json!({ "error": e.to_string() });
                let response = tiny_http::Response::from_string(body.to_string())
                    .with_status_code(400)
                    .with_header(tiny_http::Header::from_bytes(&b"Content-Type"[..], &b"application/json"[..]).unwrap());
                request.respond(response)?;
            }
        }
    }
    Ok(())
}
