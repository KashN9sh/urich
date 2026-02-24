//! Rust ASGI: протокол приложения (scope + receive + send), независимый от сервера.
//! Один контракт для HTTP, WebSocket и Lifespan — как в Python ASGI.

use std::sync::{Arc, RwLock};

use async_trait::async_trait;

use crate::{App, CoreError, RequestContext, Response};

/// Ошибка вызова ASGI-приложения (маршрут не найден, валидация и т.д.).
pub type AsgiError = CoreError;

// -----------------------------------------------------------------------------
// Scope
// -----------------------------------------------------------------------------

/// Тип соединения и метаданные. Сервер передаёт приложению при вызове call().
#[derive(Clone, Debug)]
pub enum Scope {
    Http(HttpScope),
    WebSocket(WsScope),
    Lifespan(LifespanScope),
}

/// HTTP-запрос: метод, путь, заголовки, query. Тело приходит в receive как HttpRequest.
#[derive(Clone, Debug)]
pub struct HttpScope {
    pub method: String,
    pub path: String,
    pub headers: Vec<(String, String)>,
    /// Raw query string (e.g. "a=1&b=2").
    pub query_string: String,
}

/// WebSocket-подключение: путь, заголовки.
#[derive(Clone, Debug)]
pub struct WsScope {
    pub path: String,
    pub headers: Vec<(String, String)>,
}

/// Lifespan: событие startup или shutdown (один вызов call на каждое).
#[derive(Clone, Debug)]
pub enum LifespanScope {
    Startup,
    Shutdown,
}

// -----------------------------------------------------------------------------
// Receive / Send messages
// -----------------------------------------------------------------------------

/// События от сервера к приложению (receive).
#[derive(Clone, Debug)]
pub enum AsgiReceiveMessage {
    /// HTTP: тело запроса (одно сообщение на запрос).
    HttpRequest { body: Vec<u8> },
    /// Lifespan: запуск приложения.
    LifespanStartup,
    /// Lifespan: остановка приложения.
    LifespanShutdown,
    /// WebSocket: получены данные или закрытие.
    WsReceive {
        text: Option<String>,
        bytes: Option<Vec<u8>>,
        close_code: Option<u16>,
    },
}

/// События от приложения к серверу (send).
#[derive(Clone, Debug)]
pub enum AsgiSendMessage {
    /// HTTP: начало ответа (status + headers).
    HttpResponseStart {
        status: u16,
        headers: Vec<(String, String)>,
    },
    /// HTTP: часть тела (more = true если будут ещё части).
    HttpResponseBody { body: Vec<u8>, more: bool },
    /// Lifespan: startup завершён.
    LifespanStartupComplete,
    /// Lifespan: shutdown завершён.
    LifespanShutdownComplete,
    /// WebSocket: отправить текст или бинарные данные.
    WsSend {
        text: Option<String>,
        bytes: Option<Vec<u8>>,
    },
    /// WebSocket: закрыть соединение.
    WsClose { code: Option<u16> },
}

// -----------------------------------------------------------------------------
// Receive / Send traits
// -----------------------------------------------------------------------------

/// Канал приёма событий от сервера. Реализации создаёт сервер для каждого scope.
#[async_trait]
pub trait AsgiReceive: Send + Sync {
    /// Получить следующее сообщение. None — поток завершён (для HTTP — конец запроса).
    async fn recv(&mut self) -> Result<Option<AsgiReceiveMessage>, AsgiError>;
}

/// Канал отправки событий к серверу. Реализации создаёт сервер для каждого scope.
#[async_trait]
pub trait AsgiSend: Send + Sync {
    /// Отправить сообщение серверу.
    async fn send(&mut self, msg: AsgiSendMessage) -> Result<(), AsgiError>;
}

// -----------------------------------------------------------------------------
// AsgiApplication
// -----------------------------------------------------------------------------

/// Протокол приложения: один метод для всех типов соединений (HTTP, WebSocket, Lifespan).
/// Сервер строит scope и пару receive/send, вызывает call(scope, receive, send).
#[async_trait]
pub trait AsgiApplication: Send + Sync {
    /// Обработать соединение. Диспетчеризация по scope; приложение читает receive и пишет в send.
    async fn call(
        &self,
        scope: Scope,
        receive: &mut dyn AsgiReceive,
        send: &mut dyn AsgiSend,
    ) -> Result<(), AsgiError>;
}

// -----------------------------------------------------------------------------
// UrichAsgi
// -----------------------------------------------------------------------------

/// Urich-приложение как ASGI: роутинг, валидация, callback + GET /openapi.json, GET /docs.
pub struct UrichAsgi {
    app: Arc<RwLock<App>>,
    openapi_title: String,
    openapi_version: String,
}

const SWAGGER_UI_HTML: &str = r#"<!DOCTYPE html>
<html>
<head><title>Swagger UI</title><link rel="stylesheet" href="https://unpkg.com/swagger-ui-dist@5/swagger-ui.css"></head>
<body><div id="swagger-ui"></div>
<script src="https://unpkg.com/swagger-ui-dist@5/swagger-ui-bundle.js"></script>
<script>SwaggerUIBundle({ url: '/openapi.json', dom_id: '#swagger-ui' });</script>
</body>
</html>"#;

impl UrichAsgi {
    pub fn new(
        app: Arc<RwLock<App>>,
        openapi_title: impl Into<String>,
        openapi_version: impl Into<String>,
    ) -> Self {
        Self {
            app,
            openapi_title: openapi_title.into(),
            openapi_version: openapi_version.into(),
        }
    }
}

#[async_trait]
impl AsgiApplication for UrichAsgi {
    async fn call(
        &self,
        scope: Scope,
        receive: &mut dyn AsgiReceive,
        send: &mut dyn AsgiSend,
    ) -> Result<(), AsgiError> {
        match scope {
            Scope::Lifespan(LifespanScope::Startup) => {
                let _ = receive.recv().await?; // lifespan.startup
                send.send(AsgiSendMessage::LifespanStartupComplete).await?;
                Ok(())
            }
            Scope::Lifespan(LifespanScope::Shutdown) => {
                let _ = receive.recv().await?; // lifespan.shutdown
                send.send(AsgiSendMessage::LifespanShutdownComplete).await?;
                Ok(())
            }
            Scope::Http(http_scope) => {
                let body = match receive.recv().await? {
                    Some(AsgiReceiveMessage::HttpRequest { body }) => body,
                    _ => return Err(CoreError::Validation("expected http.request".into())),
                };
                let req = RequestContext {
                    method: http_scope.method.clone(),
                    path: http_scope.path.clone(),
                    headers: http_scope.headers.clone(),
                    body,
                };
                let path = req.path.trim_start_matches('/');
                let path_with_slash = format!("/{}", path);

                if path == "openapi.json" || path_with_slash == "/openapi.json" {
                    let spec = {
                        let guard = self.app.read().map_err(|e| CoreError::Validation(e.to_string()))?;
                        guard.openapi_spec(&self.openapi_title, &self.openapi_version)
                    };
                    let body = serde_json::to_string(&spec).unwrap_or_default();
                    send.send(AsgiSendMessage::HttpResponseStart {
                        status: 200,
                        headers: vec![("Content-Type".into(), "application/json".into())],
                    })
                    .await?;
                    send.send(AsgiSendMessage::HttpResponseBody {
                        body: body.into_bytes(),
                        more: false,
                    })
                    .await?;
                    return Ok(());
                }

                if path == "docs" || path_with_slash == "/docs" {
                    send.send(AsgiSendMessage::HttpResponseStart {
                        status: 200,
                        headers: vec![("Content-Type".into(), "text/html".into())],
                    })
                    .await?;
                    send.send(AsgiSendMessage::HttpResponseBody {
                        body: SWAGGER_UI_HTML.as_bytes().to_vec(),
                        more: false,
                    })
                    .await?;
                    return Ok(());
                }

                let (handler_id, payload) = {
                    let guard = self.app.read().map_err(|e| CoreError::Validation(e.to_string()))?;
                    guard.match_route_and_validate(&req)?
                };
                let cb = {
                    let guard = self.app.read().map_err(|e| CoreError::Validation(e.to_string()))?;
                    guard
                        .get_callback()
                        .ok_or_else(|| CoreError::Validation("no callback set".into()))?
                };
                let response: Response = cb(handler_id, &payload, &req).await?;
                let content_type = response
                    .content_type
                    .as_deref()
                    .unwrap_or("application/json");
                send.send(AsgiSendMessage::HttpResponseStart {
                    status: response.status_code,
                    headers: vec![("Content-Type".into(), content_type.to_string())],
                })
                .await?;
                send.send(AsgiSendMessage::HttpResponseBody {
                    body: response.body,
                    more: false,
                })
                .await?;
                Ok(())
            }
            Scope::WebSocket(_) => {
                // Пока минимальная обработка: закрываем с кодом "not supported"
                send.send(AsgiSendMessage::WsClose { code: Some(1008) })
                    .await?;
                Ok(())
            }
        }
    }
}
