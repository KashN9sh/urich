//! Python bindings for urich-core. Uses Application (shared layer) with set_external_callback for Python handler.

use pyo3::prelude::*;
use std::sync::Mutex;
use urich_core::{
    Application, CoreError, ExternalCallback, RequestContext, Response as CoreResponse, RouteId,
};

#[pyclass]
struct CoreApp {
    inner: Mutex<Option<Application>>,
    handler: Mutex<Option<pyo3::Py<pyo3::PyAny>>>,
}

#[pymethods]
impl CoreApp {
    #[new]
    fn new() -> Self {
        Self {
            inner: Mutex::new(Some(Application::new())),
            handler: Mutex::new(None),
        }
    }

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
            .register_route_only(method, path, schema, openapi_tag)
            .map_err(|e| pyo3::exceptions::PyValueError::new_err(e.to_string()))?;
        Ok(id.0)
    }

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
            .add_command_route(context, name, schema)
            .map_err(|e| pyo3::exceptions::PyValueError::new_err(e.to_string()))?;
        Ok(id.0)
    }

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
            .add_query_route(context, name, schema)
            .map_err(|e| pyo3::exceptions::PyValueError::new_err(e.to_string()))?;
        Ok(id.0)
    }

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

    #[pyo3(signature = (name, request_schema=None))]
    fn add_rpc_method(&self, name: &str, request_schema: Option<&str>) -> PyResult<u32> {
        let schema = request_schema.and_then(|s| serde_json::from_str(s).ok());
        let mut guard = self
            .inner
            .lock()
            .map_err(|e| pyo3::exceptions::PyRuntimeError::new_err(e.to_string()))?;
        let app = guard
            .as_mut()
            .ok_or_else(|| pyo3::exceptions::PyRuntimeError::new_err("already run"))?;
        let id = app
            .add_rpc_method_route(name, schema)
            .map_err(|e| pyo3::exceptions::PyValueError::new_err(e.to_string()))?;
        Ok(id.0)
    }

    fn subscribe_event(&self, event_type_id: &str) -> PyResult<u32> {
        let mut guard = self
            .inner
            .lock()
            .map_err(|e| pyo3::exceptions::PyRuntimeError::new_err(e.to_string()))?;
        let app = guard
            .as_mut()
            .ok_or_else(|| pyo3::exceptions::PyRuntimeError::new_err("already run"))?;
        let id = app.subscribe_event_route(event_type_id);
        Ok(id.0)
    }

    fn publish_event(&self, event_type_id: &str, payload: &[u8]) -> PyResult<()> {
        let guard = self
            .inner
            .lock()
            .map_err(|e| pyo3::exceptions::PyRuntimeError::new_err(e.to_string()))?;
        let app = guard
            .as_ref()
            .ok_or_else(|| pyo3::exceptions::PyRuntimeError::new_err("already run"))?;
        app.publish_event_by_name(event_type_id, payload)
            .map_err(|e| pyo3::exceptions::PyValueError::new_err(e.to_string()))
    }

    fn set_handler(&self, handler: pyo3::Py<pyo3::PyAny>) -> PyResult<()> {
        let handler_arc = std::sync::Arc::new(Python::with_gil(|py| handler.clone_ref(py)));
        Python::with_gil(|py| *self.handler.lock().unwrap() = Some(handler.clone_ref(py)));
        let mut guard = self
            .inner
            .lock()
            .map_err(|e| pyo3::exceptions::PyRuntimeError::new_err(e.to_string()))?;
        let app = guard
            .as_mut()
            .ok_or_else(|| pyo3::exceptions::PyRuntimeError::new_err("already run"))?;
        let cb: ExternalCallback = std::sync::Arc::new(
            move |route_id: RouteId, body: &[u8], ctx: &RequestContext| {
                let handler_arc = std::sync::Arc::clone(&handler_arc);
                let body = body.to_vec();
                let ctx = ctx.clone();
                Box::pin(async move {
                    Python::with_gil(|py| {
                        let cb = handler_arc.bind(py);
                        let body_bytes = pyo3::types::PyBytes::new_bound(py, &body);
                        let headers_list = pyo3::types::PyList::empty_bound(py);
                        for (k, v) in &ctx.headers {
                            let pair =
                                pyo3::types::PyList::new_bound(py, [k.as_str(), v.as_str()]);
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
                            content_type: None,
                        })
                    })
                    .map_err(|e: pyo3::PyErr| CoreError::Validation(e.to_string()))
                })
            },
        );
        app.set_external_callback(cb);
        Ok(())
    }

    fn handle_request(&self, method: &str, path: &str, body: &[u8]) -> PyResult<(u16, Vec<u8>)> {
        let mut guard = self
            .inner
            .lock()
            .map_err(|e| pyo3::exceptions::PyRuntimeError::new_err(e.to_string()))?;
        let app = guard
            .as_mut()
            .ok_or_else(|| pyo3::exceptions::PyRuntimeError::new_err("already run"))?;
        let body_bytes = app
            .handle_request(method, path, body)
            .map_err(|e| pyo3::exceptions::PyValueError::new_err(e.to_string()))?;
        Ok((200, body_bytes))
    }

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

    fn run_from_env(
        &self,
        default_host: &str,
        default_port: u16,
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
        app.run_from_env(default_host, default_port, openapi_title, openapi_version)
            .map_err(|e| pyo3::exceptions::PyRuntimeError::new_err(e.to_string()))
    }
}

#[pymodule]
fn urich_core_native(m: &Bound<'_, PyModule>) -> PyResult<()> {
    m.add_class::<CoreApp>()?;
    Ok(())
}
