//! Async HTTP server: tokio + hyper. Вызывает ASGI call(scope, receive, send).
//! HTTP и Lifespan; WebSocket — при детекте Upgrade.
//! Хост/порт: env HOST/PORT и аргументы --host/--port (аргументы перекрывают env).

use crate::asgi::{
    AsgiApplication, AsgiReceive, AsgiReceiveMessage, AsgiSend, AsgiSendMessage, HttpScope,
    LifespanScope, Scope, WsScope,
};
use crate::CoreError;
use async_trait::async_trait;
use bytes::Bytes;
use http_body_util::{BodyExt, Full};
use hyper::server::conn::http1;
use hyper::service::service_fn;
use hyper::{Request, Response as HyperResponse, StatusCode};
use hyper_tungstenite::{is_upgrade_request, upgrade};
use hyper_tungstenite::tungstenite::Message;
use hyper_util::rt::TokioIo;
use std::sync::{Arc, RwLock};
use tokio::net::TcpListener;

use crate::App;

// -----------------------------------------------------------------------------
// HTTP receive/send drivers
// -----------------------------------------------------------------------------

/// Выдаёт одно сообщение HttpRequest с телом, затем None.
struct HttpReceiveDriver {
    body: Vec<u8>,
    sent: bool,
}

#[async_trait]
impl AsgiReceive for HttpReceiveDriver {
    async fn recv(&mut self) -> Result<Option<AsgiReceiveMessage>, crate::AsgiError> {
        if self.sent {
            return Ok(None);
        }
        self.sent = true;
        Ok(Some(AsgiReceiveMessage::HttpRequest {
            body: std::mem::take(&mut self.body),
        }))
    }
}

/// Собирает HttpResponseStart + HttpResponseBody, потом из него собирается hyper Response.
struct HttpSendDriver {
    status: Option<u16>,
    headers: Vec<(String, String)>,
    body: Vec<u8>,
}

impl HttpSendDriver {
    fn new() -> Self {
        Self {
            status: None,
            headers: Vec::new(),
            body: Vec::new(),
        }
    }
    fn into_hyper_response(self) -> HyperResponse<Full<Bytes>> {
        let status = self.status.unwrap_or(500);
        let mut b = HyperResponse::builder().status(status);
        for (k, v) in &self.headers {
            b = b.header(k.as_str(), v.as_str());
        }
        b.body(Full::new(Bytes::from(self.body))).unwrap()
    }
}

#[async_trait]
impl AsgiSend for HttpSendDriver {
    async fn send(&mut self, msg: AsgiSendMessage) -> Result<(), crate::AsgiError> {
        match msg {
            AsgiSendMessage::HttpResponseStart { status, headers } => {
                self.status = Some(status);
                self.headers = headers;
            }
            AsgiSendMessage::HttpResponseBody { body, .. } => {
                self.body.extend(body);
            }
            _ => {}
        }
        Ok(())
    }
}

// -----------------------------------------------------------------------------
// Lifespan receive/send drivers
// -----------------------------------------------------------------------------

struct LifespanReceiveDriver {
    event: Option<AsgiReceiveMessage>,
}

#[async_trait]
impl AsgiReceive for LifespanReceiveDriver {
    async fn recv(&mut self) -> Result<Option<AsgiReceiveMessage>, crate::AsgiError> {
        Ok(self.event.take())
    }
}

struct LifespanSendDriver;

#[async_trait]
impl AsgiSend for LifespanSendDriver {
    async fn send(&mut self, msg: AsgiSendMessage) -> Result<(), crate::AsgiError> {
        match msg {
            AsgiSendMessage::LifespanStartupComplete | AsgiSendMessage::LifespanShutdownComplete => {}
            _ => {}
        }
        Ok(())
    }
}

// -----------------------------------------------------------------------------
// WebSocket receive/send drivers
// -----------------------------------------------------------------------------

use hyper::upgrade::Upgraded;
use hyper_tungstenite::tungstenite::protocol::CloseFrame;
use hyper_tungstenite::WebSocketStream;
use tokio::sync::mpsc;
use tokio_stream::StreamExt;

type WsStream = WebSocketStream<hyper_util::rt::TokioIo<Upgraded>>;

struct WsReceiveDriver {
    rx: mpsc::Receiver<Result<Option<AsgiReceiveMessage>, crate::AsgiError>>,
}

struct WsSendDriver {
    tx: mpsc::Sender<AsgiSendMessage>,
}

#[async_trait]
impl AsgiReceive for WsReceiveDriver {
    async fn recv(&mut self) -> Result<Option<AsgiReceiveMessage>, crate::AsgiError> {
        match self.rx.recv().await {
            Some(Ok(msg)) => Ok(msg),
            Some(Err(e)) => Err(e),
            None => Ok(None),
        }
    }
}

#[async_trait]
impl AsgiSend for WsSendDriver {
    async fn send(&mut self, msg: AsgiSendMessage) -> Result<(), crate::AsgiError> {
        self.tx.send(msg).await.map_err(|_| crate::AsgiError::Validation("ws send channel closed".into()))
    }
}

async fn run_ws_stream_loop(
    mut stream: WsStream,
    tx_recv: mpsc::Sender<Result<Option<AsgiReceiveMessage>, crate::AsgiError>>,
    mut rx_send: mpsc::Receiver<AsgiSendMessage>,
) {
    use futures_util::SinkExt;
    loop {
        tokio::select! {
            msg = stream.next() => {
                let mapped = match msg {
                    None => {
                        let _ = tx_recv.send(Ok(None)).await;
                        break;
                    }
                    Some(Ok(Message::Text(s))) => Ok(Some(AsgiReceiveMessage::WsReceive {
                        text: Some(s.to_string()),
                        bytes: None,
                        close_code: None,
                    })),
                    Some(Ok(Message::Binary(b))) => Ok(Some(AsgiReceiveMessage::WsReceive {
                        text: None,
                        bytes: Some(b.to_vec()),
                        close_code: None,
                    })),
                    Some(Ok(Message::Close(c))) => Ok(Some(AsgiReceiveMessage::WsReceive {
                        text: None,
                        bytes: None,
                        close_code: c.map(|f: CloseFrame| f.code.into()),
                    })),
                    Some(Ok(Message::Ping(_) | Message::Pong(_) | Message::Frame(_))) => continue,
                    Some(Err(e)) => Err(crate::AsgiError::Validation(e.to_string())),
                };
                if tx_recv.send(mapped).await.is_err() {
                    break;
                }
            }
            msg = rx_send.recv() => {
                let Some(m) = msg else { break };
                let ws_msg = match m {
                    AsgiSendMessage::WsSend { text, bytes } => {
                        if let Some(t) = text {
                            Message::Text(t.into())
                        } else if let Some(b) = bytes {
                            Message::Binary(b.into())
                        } else {
                            continue;
                        }
                    }
                    AsgiSendMessage::WsClose { code } => {
                        let cf = code.map(|c| CloseFrame { code: c.into(), reason: String::new().into() });
                        Message::Close(cf)
                    }
                    _ => continue,
                };
                if stream.send(ws_msg).await.is_err() {
                    break;
                }
            }
        }
    }
}

/// Запуск сервера с ASGI-приложением (scope + receive + send).
/// Порядок: lifespan startup → accept loop; при shutdown — lifespan shutdown.
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
        // Lifespan: startup
        {
            let scope = Scope::Lifespan(LifespanScope::Startup);
            let mut recv = LifespanReceiveDriver {
                event: Some(AsgiReceiveMessage::LifespanStartup),
            };
            let mut send = LifespanSendDriver;
            if let Err(e) = asgi.call(scope, &mut recv, &mut send).await {
                eprintln!("lifespan startup error: {}", e);
            }
        }

        let listener = TcpListener::bind(&addr).await?;
        let shutdown = tokio::signal::ctrl_c();
        tokio::pin!(shutdown);

        let result = loop {
            tokio::select! {
                _ = &mut shutdown => {
                    // Lifespan: shutdown
                    let scope = Scope::Lifespan(LifespanScope::Shutdown);
                    let mut recv = LifespanReceiveDriver {
                        event: Some(AsgiReceiveMessage::LifespanShutdown),
                    };
                    let mut send = LifespanSendDriver;
                    let _ = asgi.call(scope, &mut recv, &mut send).await;
                    break Ok(());
                }
                accept_result = listener.accept() => {
                    let (stream, _) = match accept_result {
                        Ok(x) => x,
                        Err(e) => {
                            eprintln!("accept error: {}", e);
                            continue;
                        }
                    };
                    let io = TokioIo::new(stream);
                    let asgi = Arc::clone(&asgi);
                    tokio::task::spawn(async move {
                        let service = service_fn(move |req: Request<hyper::body::Incoming>| {
                            let asgi = Arc::clone(&asgi);
                            async move { asgi_http_or_ws_to_hyper(asgi, req).await }
                        });
                        if let Err(e) = http1::Builder::new().serve_connection(io, service).await {
                            eprintln!("serve_connection error: {}", e);
                        }
                    });
                }
            }
        };
        result
    })
}

async fn asgi_http_or_ws_to_hyper(
    asgi: Arc<dyn AsgiApplication>,
    req: Request<hyper::body::Incoming>,
) -> Result<HyperResponse<Full<Bytes>>, std::convert::Infallible> {
    if is_upgrade_request(&req) {
        return asgi_websocket_upgrade(asgi, req).await;
    }
    let (scope, body) = match hyper_request_to_scope_and_body(req).await {
        Ok(x) => x,
        Err(e) => return Ok(asgi_error_to_hyper(e)),
    };
    let mut recv = HttpReceiveDriver { body, sent: false };
    let mut send = HttpSendDriver::new();
    match asgi.call(scope, &mut recv, &mut send).await {
        Ok(()) => Ok(send.into_hyper_response()),
        Err(e) => Ok(asgi_error_to_hyper(e)),
    }
}

async fn asgi_websocket_upgrade(
    asgi: Arc<dyn AsgiApplication>,
    req: Request<hyper::body::Incoming>,
) -> Result<HyperResponse<Full<Bytes>>, std::convert::Infallible> {
    let path = req.uri().path().to_string();
    let headers: Vec<(String, String)> = req
        .headers()
        .iter()
        .map(|(k, v)| (k.as_str().to_string(), v.to_str().unwrap_or("").to_string()))
        .collect();
    let ws_scope = WsScope { path, headers };
    let (response, ws_future) = match upgrade(req, None) {
        Ok(x) => x,
        Err(e) => {
            return Ok(HyperResponse::builder()
                .status(StatusCode::BAD_REQUEST)
                .body(Full::new(Bytes::from(format!("upgrade error: {}", e))))
                .unwrap());
        }
    };
    let asgi_clone = Arc::clone(&asgi);
    tokio::spawn(async move {
        let stream = match ws_future.await {
            Ok(s) => s,
            Err(_) => return,
        };
        let (tx_recv, rx_recv) = mpsc::channel(16);
        let (tx_send, rx_send) = mpsc::channel(16);
        tokio::spawn(run_ws_stream_loop(stream, tx_recv, rx_send));
        let mut recv = WsReceiveDriver { rx: rx_recv };
        let mut send = WsSendDriver { tx: tx_send };
        let _ = asgi_clone
            .call(Scope::WebSocket(ws_scope), &mut recv, &mut send)
            .await;
    });
    let (parts, body) = response.into_parts();
    let bytes = body
        .collect()
        .await
        .map(|b| b.to_bytes())
        .unwrap_or_default();
    Ok(HyperResponse::from_parts(parts, Full::new(bytes)))
}

async fn hyper_request_to_scope_and_body(
    req: Request<hyper::body::Incoming>,
) -> Result<(Scope, Vec<u8>), CoreError> {
    let method = req.method().to_string();
    let path = req.uri().path().trim_start_matches('/').to_string();
    let query_string = req.uri().query().unwrap_or("").to_string();
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
    let url = req.uri().to_string();
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
    let scope = Scope::Http(HttpScope {
        method,
        path,
        headers,
        query_string,
    });
    Ok((scope, body))
}

fn asgi_error_to_hyper(e: crate::AsgiError) -> HyperResponse<Full<Bytes>> {
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
