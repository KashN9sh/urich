//! Python bindings for urich-core. Exposes register_route, set_handler, handle_request, openapi_spec, run.

use pyo3::prelude::*;
use std::sync::Mutex;
use urich_core::{App, CoreError, RequestContext, Response as CoreResponse, RouteId};

#[pyclass]
struct CoreApp {
    inner: Mutex<Option<App>>,
    handlers: Mutex<Option<pyo3::Py<pyo3::PyAny>>>,
}

#[pymethods]
impl CoreApp {
    #[new]
    fn new() -> Self {
        Self {
            inner: Mutex::new(Some(App::new())),
            handlers: Mutex::new(None),
        }
    }

    /// Register a route. Returns route_id (int). openapi_tag optional (e.g. context name for OpenAPI tags).
    #[pyo3(signature = (method, path, request_schema=None, openapi_tag=None))]
    fn register_route(
        &self,
        method: &str,
        path: &str,
        request_schema: Option<&str>,
        openapi_tag: Option<&str>,
    ) -> PyResult<u32> {
        let schema = request_schema.and_then(|s| serde_json::from_str(s).ok());
        let mut guard = self
            .inner
            .lock()
            .map_err(|e| pyo3::exceptions::PyRuntimeError::new_err(e.to_string()))?;
        let app = guard
            .as_mut()
            .ok_or_else(|| pyo3::exceptions::PyRuntimeError::new_err("already run"))?;
        let id = app
            .register_route(method, path, schema, openapi_tag)
            .map_err(|e| pyo3::exceptions::PyValueError::new_err(e.to_string()))?;
        Ok(id.0)
    }

    /// Add command: POST {context}/commands/{name}. Returns handler_id (int).
    #[pyo3(signature = (context, name, request_schema=None))]
    fn add_command(
        &self,
        context: &str,
        name: &str,
        request_schema: Option<&str>,
    ) -> PyResult<u32> {
        let schema = request_schema.and_then(|s| serde_json::from_str(s).ok());
        let mut guard = self
            .inner
            .lock()
            .map_err(|e| pyo3::exceptions::PyRuntimeError::new_err(e.to_string()))?;
        let app = guard
            .as_mut()
            .ok_or_else(|| pyo3::exceptions::PyRuntimeError::new_err("already run"))?;
        let id = app
            .add_command(context, name, schema)
            .map_err(|e| pyo3::exceptions::PyValueError::new_err(e.to_string()))?;
        Ok(id.0)
    }

    /// Add query: GET {context}/queries/{name}. Returns handler_id (int).
    #[pyo3(signature = (context, name, request_schema=None))]
    fn add_query(
        &self,
        context: &str,
        name: &str,
        request_schema: Option<&str>,
    ) -> PyResult<u32> {
        let schema = request_schema.and_then(|s| serde_json::from_str(s).ok());
        let mut guard = self
            .inner
            .lock()
            .map_err(|e| pyo3::exceptions::PyRuntimeError::new_err(e.to_string()))?;
        let app = guard
            .as_mut()
            .ok_or_else(|| pyo3::exceptions::PyRuntimeError::new_err("already run"))?;
        let id = app
            .add_query(context, name, schema)
            .map_err(|e| pyo3::exceptions::PyValueError::new_err(e.to_string()))?;
        Ok(id.0)
    }

    /// Add RPC route (one POST route at path). Body format: { "method": str, "params": object }.
    fn add_rpc_route(&self, path: &str) -> PyResult<()> {
        let mut guard = self
            .inner
            .lock()
            .map_err(|e| pyo3::exceptions::PyRuntimeError::new_err(e.to_string()))?;
        let app = guard
            .as_mut()
            .ok_or_else(|| pyo3::exceptions::PyRuntimeError::new_err("already run"))?;
        app.add_rpc_route(path)
            .map_err(|e| pyo3::exceptions::PyValueError::new_err(e.to_string()))
    }

    /// Add RPC method. Returns handler_id (int). Callback receives params bytes.
    #[pyo3(signature = (name, request_schema=None))]
    fn add_rpc_method(
        &self,
        name: &str,
        request_schema: Option<&str>,
    ) -> PyResult<u32> {
        let schema = request_schema.and_then(|s| serde_json::from_str(s).ok());
        let mut guard = self
            .inner
            .lock()
            .map_err(|e| pyo3::exceptions::PyRuntimeError::new_err(e.to_string()))?;
        let app = guard
            .as_mut()
            .ok_or_else(|| pyo3::exceptions::PyRuntimeError::new_err("already run"))?;
        let id = app
            .add_rpc_method(name, schema)
            .map_err(|e| pyo3::exceptions::PyValueError::new_err(e.to_string()))?;
        Ok(id.0)
    }

    /// Subscribe to event type. Returns handler_id (int).
    fn subscribe_event(&self, event_type_id: &str) -> PyResult<u32> {
        let mut guard = self
            .inner
            .lock()
            .map_err(|e| pyo3::exceptions::PyRuntimeError::new_err(e.to_string()))?;
        let app = guard
            .as_mut()
            .ok_or_else(|| pyo3::exceptions::PyRuntimeError::new_err("already run"))?;
        let id = app.subscribe_event(event_type_id);
        Ok(id.0)
    }

    /// Publish event: core calls execute(handler_id, payload) for each subscriber.
    fn publish_event(&self, event_type_id: &str, payload: &[u8]) -> PyResult<()> {
        let guard = self
            .inner
            .lock()
            .map_err(|e| pyo3::exceptions::PyRuntimeError::new_err(e.to_string()))?;
        let app = guard
            .as_ref()
            .ok_or_else(|| pyo3::exceptions::PyRuntimeError::new_err("already run"))?;
        app.publish_event(event_type_id, payload)
            .map_err(|e| pyo3::exceptions::PyValueError::new_err(e.to_string()))
    }

    /// Set the Python callable invoked as (route_id: int, body_bytes: bytes, context: dict) -> (status_code: int, response_bytes: bytes).
    /// context has "method", "path", "headers" (list of [name, value]), "body" (bytes).
    fn set_handler(&self, handler: pyo3::Py<pyo3::PyAny>) -> PyResult<()> {
        let handlers = Python::with_gil(|py| handler.clone_ref(py));
        *self.handlers.lock().unwrap() = Some(handler);
        let mut guard = self
            .inner
            .lock()
            .map_err(|e| pyo3::exceptions::PyRuntimeError::new_err(e.to_string()))?;
        let app = guard
            .as_mut()
            .ok_or_else(|| pyo3::exceptions::PyRuntimeError::new_err("already run"))?;
        app.set_callback(Box::new(move |route_id: RouteId, body: &[u8], ctx: &RequestContext| {
            Python::with_gil(|py| {
                let cb = handlers.bind(py);
                let body_bytes = pyo3::types::PyBytes::new_bound(py, body);
                let headers_list = pyo3::types::PyList::empty_bound(py);
                for (k, v) in &ctx.headers {
                    let pair = pyo3::types::PyList::new_bound(py, [k.as_str(), v.as_str()]);
                    headers_list.append(pair)?;
                }
                let context = pyo3::types::PyDict::new_bound(py);
                context.set_item("method", ctx.method.as_str())?;
                context.set_item("path", ctx.path.as_str())?;
                context.set_item("headers", headers_list)?;
                context.set_item("body", pyo3::types::PyBytes::new_bound(py, &ctx.body))?;
                let result = cb.call1((route_id.0, body_bytes, context))?;
                let tuple = result.downcast::<pyo3::types::PyTuple>()?;
                let status: u16 = tuple.get_item(0)?.extract()?;
                let body_item = tuple.get_item(1)?;
                let bytes = body_item.downcast::<pyo3::types::PyBytes>()?;
                Ok(CoreResponse {
                    status_code: status,
                    body: bytes.as_bytes().to_vec(),
                })
            })
            .map_err(|e: pyo3::PyErr| CoreError::Validation(e.to_string()))
        }));
        Ok(())
    }

    /// Handle request without HTTP (for testing). Returns (status_code, response_bytes).
    fn handle_request(&self, method: &str, path: &str, body: &[u8]) -> PyResult<(u16, Vec<u8>)> {
        let guard = self
            .inner
            .lock()
            .map_err(|e| pyo3::exceptions::PyRuntimeError::new_err(e.to_string()))?;
        let app = guard
            .as_ref()
            .ok_or_else(|| pyo3::exceptions::PyRuntimeError::new_err("already run"))?;
        let ctx = RequestContext {
            method: method.to_string(),
            path: path.to_string(),
            headers: vec![],
            body: body.to_vec(),
        };
        let resp = app
            .handle_request(&ctx)
            .map_err(|e| pyo3::exceptions::PyValueError::new_err(e.to_string()))?;
        Ok((resp.status_code, resp.body))
    }

    /// OpenAPI spec as JSON string.
    fn openapi_spec(&self, title: &str, version: &str) -> PyResult<String> {
        let guard = self
            .inner
            .lock()
            .map_err(|e| pyo3::exceptions::PyRuntimeError::new_err(e.to_string()))?;
        let app = guard
            .as_ref()
            .ok_or_else(|| pyo3::exceptions::PyRuntimeError::new_err("already run"))?;
        let spec = app.openapi_spec(title, version);
        serde_json::to_string(&spec)
            .map_err(|e| pyo3::exceptions::PyValueError::new_err(e.to_string()))
    }

    /// Run HTTP server (blocks). Serves routes, GET /openapi.json, GET /docs. Call set_handler before run. After run, this CoreApp is consumed.
    fn run(
        &self,
        host: &str,
        port: u16,
        openapi_title: &str,
        openapi_version: &str,
    ) -> PyResult<()> {
        let mut guard = self
            .inner
            .lock()
            .map_err(|e| pyo3::exceptions::PyRuntimeError::new_err(e.to_string()))?;
        let app = guard
            .take()
            .ok_or_else(|| pyo3::exceptions::PyRuntimeError::new_err("already run"))?;
        drop(guard);
        app.run(host, port, openapi_title, openapi_version)
            .map_err(|e| pyo3::exceptions::PyRuntimeError::new_err(e.to_string()))
    }
}

#[pymodule]
fn urich_core_native(m: &Bound<'_, PyModule>) -> PyResult<()> {
    m.add_class::<CoreApp>()?;
    Ok(())
}
