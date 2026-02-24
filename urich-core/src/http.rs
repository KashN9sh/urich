//! Async HTTP server: tokio + hyper. Serves routes, GET /openapi.json, GET /docs.

use crate::{App, CoreError, RequestContext, Response};
use bytes::Bytes;
use http_body_util::{BodyExt, Full};
use hyper::server::conn::http1;
use hyper::service::service_fn;
use hyper::{Request, Response as HyperResponse, StatusCode};
use hyper_util::rt::TokioIo;
use std::sync::{Arc, RwLock};
use tokio::net::TcpListener;

const SWAGGER_UI_HTML: &str = r#"<!DOCTYPE html>
<html>
<head><title>Swagger UI</title><link rel="stylesheet" href="https://unpkg.com/swagger-ui-dist@5/swagger-ui.css"></head>
<body><div id="swagger-ui"></div>
<script src="https://unpkg.com/swagger-ui-dist@5/swagger-ui-bundle.js"></script>
<script>SwaggerUIBundle({ url: '/openapi.json', dom_id: '#swagger-ui' });</script>
</body>
</html>"#;

/// Run HTTP server; blocks by running tokio runtime. Serves POST/GET to registered routes, GET /openapi.json, GET /docs.
pub fn run(
    app: Arc<RwLock<App>>,
    host: &str,
    port: u16,
    openapi_title: &str,
    openapi_version: &str,
) -> Result<(), Box<dyn std::error::Error + Send + Sync>> {
    let addr = format!("{}:{}", host, port);
    let openapi_title = openapi_title.to_string();
    let openapi_version = openapi_version.to_string();

    let rt = tokio::runtime::Builder::new_multi_thread()
        .enable_all()
        .build()?;

    rt.block_on(async move {
        let listener = TcpListener::bind(&addr).await?;
        loop {
            let (stream, _) = listener.accept().await?;
            let io = TokioIo::new(stream);
            let app = Arc::clone(&app);
            let openapi_title = openapi_title.clone();
            let openapi_version = openapi_version.clone();
            tokio::task::spawn(async move {
                let service = service_fn(move |req: Request<hyper::body::Incoming>| {
                    let app = Arc::clone(&app);
                    let openapi_title = openapi_title.clone();
                    let openapi_version = openapi_version.clone();
                    async move { handle_request(app, req, openapi_title, openapi_version).await }
                });
                if let Err(e) = http1::Builder::new().serve_connection(io, service).await {
                    eprintln!("serve_connection error: {}", e);
                }
            });
        }
    })
}

async fn handle_request(
    app: Arc<RwLock<App>>,
    req: Request<hyper::body::Incoming>,
    openapi_title: String,
    openapi_version: String,
) -> Result<HyperResponse<Full<Bytes>>, CoreError> {
    let method = req.method().to_string();
    let path = req.uri().path().trim_start_matches('/').to_string();
    let path_with_slash = format!("/{}", path);

    if path == "openapi.json" || path_with_slash == "/openapi.json" {
        let spec = {
            let guard = app.read().map_err(|e| CoreError::Validation(e.to_string()))?;
            guard.openapi_spec(&openapi_title, &openapi_version)
        };
        let body = serde_json::to_string(&spec).unwrap_or_default();
        return Ok(HyperResponse::builder()
            .status(StatusCode::OK)
            .header("Content-Type", "application/json")
            .body(Full::new(Bytes::from(body)))
            .unwrap());
    }

    if path == "docs" || path_with_slash == "/docs" {
        return Ok(HyperResponse::builder()
            .status(StatusCode::OK)
            .header("Content-Type", "text/html")
            .body(Full::new(Bytes::from_static(SWAGGER_UI_HTML.as_bytes())))
            .unwrap());
    }

    let url = req.uri().to_string();
    let headers: Vec<(String, String)> = req
        .headers()
        .iter()
        .map(|(k, v)| {
            (
                k.as_str().to_string(),
                v.to_str().unwrap_or("").to_string(),
            )
        })
        .collect();

    let body_bytes = req.into_body().collect().await.map_err(|e| CoreError::Validation(e.to_string()))?.to_bytes();
    let body: Vec<u8> = if method.to_uppercase() == "GET" {
        if let Some(qs) = url.split('?').nth(1) {
            let params: std::collections::HashMap<String, String> = qs
                .split('&')
                .filter_map(|p| {
                    let mut it = p.splitn(2, '=');
                    let k = it.next()?.trim().to_string();
                    let v = it.next().unwrap_or("").trim().to_string();
                    if k.is_empty() { None } else { Some((k, v)) }
                })
                .collect();
            serde_json::to_vec(&params).unwrap_or_default()
        } else {
            body_bytes.to_vec()
        }
    } else {
        body_bytes.to_vec()
    };

    let context = RequestContext {
        method: method.clone(),
        path: path.clone(),
        headers,
        body: body.clone(),
    };

    let (handler_id, payload) = {
        let guard = app.read().map_err(|e| CoreError::Validation(e.to_string()))?;
        guard.match_route_and_validate(&context)?
    };
    let cb = {
        let guard = app.read().map_err(|e| CoreError::Validation(e.to_string()))?;
        guard.get_callback().ok_or_else(|| CoreError::Validation("no callback set".into()))?
    };

    let result = cb(handler_id, &payload, &context).await;

    match result {
        Ok(Response {
            status_code,
            body: resp_bytes,
        }) => Ok(HyperResponse::builder()
            .status(status_code)
            .header("Content-Type", "application/json")
            .body(Full::new(Bytes::from(resp_bytes)))
            .unwrap()),
        Err(CoreError::NotFound(msg)) => {
            let body = serde_json::json!({ "error": msg });
            Ok(HyperResponse::builder()
                .status(StatusCode::NOT_FOUND)
                .header("Content-Type", "application/json")
                .body(Full::new(Bytes::from(body.to_string())))
                .unwrap())
        }
        Err(e) => {
            let body = serde_json::json!({ "error": e.to_string() });
            Ok(HyperResponse::builder()
                .status(StatusCode::BAD_REQUEST)
                .header("Content-Type", "application/json")
                .body(Full::new(Bytes::from(body.to_string())))
                .unwrap())
        }
    }
}
