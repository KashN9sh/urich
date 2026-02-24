//! Async HTTP server: tokio + hyper. Вызывает ASGI-приложение (один протокол — любой сервер).
//! Хост/порт: env HOST/PORT и аргументы --host/--port (аргументы перекрывают env).

use crate::asgi::{AsgiApplication, AsgiError};
use crate::{CoreError, RequestContext, Response};
use bytes::Bytes;
use http_body_util::{BodyExt, Full};
use hyper::server::conn::http1;
use hyper::service::service_fn;
use hyper::{Request, Response as HyperResponse, StatusCode};
use hyper_util::rt::TokioIo;
use std::sync::{Arc, RwLock};
use tokio::net::TcpListener;

use crate::App;

/// Запуск сервера с ASGI-приложением. Один и тот же `AsgiApplication` можно отдать другому серверу (например Axum).
pub fn run_with_asgi(
    asgi: Arc<dyn AsgiApplication>,
    host: &str,
    port: u16,
) -> Result<(), Box<dyn std::error::Error + Send + Sync>> {
    let addr = format!("{}:{}", host, port);
    let rt = tokio::runtime::Builder::new_multi_thread()
        .enable_all()
        .build()?;
    rt.block_on(async move {
        let listener = TcpListener::bind(&addr).await?;
        loop {
            let (stream, _) = listener.accept().await?;
            let io = TokioIo::new(stream);
            let asgi = Arc::clone(&asgi);
            tokio::task::spawn(async move {
                let service = service_fn(move |req: Request<hyper::body::Incoming>| {
                    let asgi = Arc::clone(&asgi);
                    async move { asgi_request_to_hyper(asgi, req).await }
                });
                if let Err(e) = http1::Builder::new().serve_connection(io, service).await {
                    eprintln!("serve_connection error: {}", e);
                }
            });
        }
    })
}

/// Читает host и port: сначала из env HOST/PORT, затем из аргументов --host/--port (перекрывают env).
/// Для обоих фасадов (Python и Rust) — один способ запуска «как uvicorn».
pub fn host_port_from_env_and_args(default_host: &str, default_port: u16) -> (String, u16) {
    let mut host = std::env::var("HOST").unwrap_or_else(|_| default_host.to_string());
    let mut port = std::env::var("PORT")
        .ok()
        .and_then(|s| s.parse().ok())
        .unwrap_or(default_port);
    let args: Vec<String> = std::env::args().collect();
    let mut i = 1;
    while i < args.len() {
        if args[i] == "--host" && i + 1 < args.len() {
            host = args[i + 1].clone();
            i += 2;
            continue;
        }
        if args[i] == "--port" && i + 1 < args.len() {
            if let Ok(p) = args[i + 1].parse() {
                port = p;
            }
            i += 2;
            continue;
        }
        i += 1;
    }
    (host, port)
}

/// Запуск встроенного сервера с Urich App (удобная обёртка: создаёт UrichAsgi и вызывает run_with_asgi).
pub fn run(
    app: Arc<RwLock<App>>,
    host: &str,
    port: u16,
    openapi_title: &str,
    openapi_version: &str,
) -> Result<(), Box<dyn std::error::Error + Send + Sync>> {
    let asgi: Arc<dyn AsgiApplication> =
        Arc::new(crate::UrichAsgi::new(app, openapi_title, openapi_version));
    run_with_asgi(asgi, host, port)
}

async fn asgi_request_to_hyper(
    asgi: Arc<dyn AsgiApplication>,
    req: Request<hyper::body::Incoming>,
) -> Result<HyperResponse<Full<Bytes>>, std::convert::Infallible> {
    let ctx = match hyper_request_to_context(req).await {
        Ok(c) => c,
        Err(e) => return Ok(asgi_error_to_hyper(e)),
    };
    let result = asgi.call(ctx).await;
    Ok(match result {
        Ok(Response {
            status_code,
            body: resp_bytes,
            content_type,
        }) => {
            let ct = content_type.as_deref().unwrap_or("application/json");
            HyperResponse::builder()
                .status(status_code)
                .header("Content-Type", ct)
                .body(Full::new(Bytes::from(resp_bytes)))
                .unwrap()
        }
        Err(e) => asgi_error_to_hyper(e),
    })
}

async fn hyper_request_to_context(
    req: Request<hyper::body::Incoming>,
) -> Result<RequestContext, CoreError> {
    let method = req.method().to_string();
    let path = req.uri().path().trim_start_matches('/').to_string();
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
    let body_bytes = req
        .into_body()
        .collect()
        .await
        .map_err(|e| CoreError::Validation(e.to_string()))?
        .to_bytes();
    let body: Vec<u8> = if method.to_uppercase() == "GET" {
        if let Some(qs) = url.split('?').nth(1) {
            let params: std::collections::HashMap<String, String> = qs
                .split('&')
                .filter_map(|p| {
                    let mut it = p.splitn(2, '=');
                    let k = it.next()?.trim().to_string();
                    let v = it.next().unwrap_or("").trim().to_string();
                    if k.is_empty() {
                        None
                    } else {
                        Some((k, v))
                    }
                })
                .collect();
            serde_json::to_vec(&params).unwrap_or_default()
        } else {
            body_bytes.to_vec()
        }
    } else {
        body_bytes.to_vec()
    };
    Ok(RequestContext {
        method,
        path,
        headers,
        body,
    })
}

fn asgi_error_to_hyper(e: AsgiError) -> HyperResponse<Full<Bytes>> {
    let (status, msg) = match &e {
        CoreError::NotFound(_) => (StatusCode::NOT_FOUND, e.to_string()),
        _ => (StatusCode::BAD_REQUEST, e.to_string()),
    };
    let body = serde_json::json!({ "error": msg });
    HyperResponse::builder()
        .status(status)
        .header("Content-Type", "application/json")
        .body(Full::new(Bytes::from(body.to_string())))
        .unwrap()
}
