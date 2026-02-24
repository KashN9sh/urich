//! Rust ASGI: протокол приложения (request → response), независимый от сервера.
//! Как ASGI в Python: одно приложение можно запускать нашим сервером, Axum, hyper и т.д.

use std::sync::{Arc, RwLock};

use async_trait::async_trait;

use crate::{App, CoreError, RequestContext, Response};

/// Ошибка вызова ASGI-приложения (маршрут не найден, валидация и т.д.).
pub type AsgiError = CoreError;

/// Протокол приложения: по запросу возвращает ответ (async).
/// Любой сервер (наш встроенный, Axum, hyper) конвертирует HTTP в `RequestContext`,
/// вызывает `call()`, конвертирует `Response` обратно в HTTP.
///
/// Аналог ASGI в Python: разделение приложения и сервера.
#[async_trait]
pub trait AsgiApplication: Send + Sync {
    /// Обработать один запрос. Сервер передаёт метод, путь, заголовки, тело.
    async fn call(&self, req: RequestContext) -> Result<Response, AsgiError>;
}

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
    async fn call(&self, req: RequestContext) -> Result<Response, AsgiError> {
        let path = req.path.trim_start_matches('/');
        let path_with_slash = format!("/{}", path);

        if path == "openapi.json" || path_with_slash == "/openapi.json" {
            let spec = {
                let guard = self.app.read().map_err(|e| CoreError::Validation(e.to_string()))?;
                guard.openapi_spec(&self.openapi_title, &self.openapi_version)
            };
            let body = serde_json::to_string(&spec).unwrap_or_default();
            return Ok(Response {
                status_code: 200,
                body: body.into_bytes(),
                content_type: Some("application/json".into()),
            });
        }

        if path == "docs" || path_with_slash == "/docs" {
            return Ok(Response {
                status_code: 200,
                body: SWAGGER_UI_HTML.as_bytes().to_vec(),
                content_type: Some("text/html".into()),
            });
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
        cb(handler_id, &payload, &req).await
    }
}
