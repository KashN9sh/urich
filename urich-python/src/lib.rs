//! Python bindings for urich-core. Exposes register_route, set_handler, handle_request, openapi_spec, run.

use pyo3::prelude::*;
use std::sync::Mutex;
use urich_core::{App, CoreError, RouteId};

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

    /// Set the Python callable invoked as (route_id: int, body_bytes: bytes) -> response_bytes: bytes.
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
        app.set_callback(Box::new(move |route_id: RouteId, body: &[u8]| {
            Python::with_gil(|py| {
                let cb = handlers.bind(py);
                let body_bytes = pyo3::types::PyBytes::new_bound(py, body);
                let result = cb.call1((route_id.0, body_bytes))?;
                let bytes = result.downcast::<pyo3::types::PyBytes>()?;
                Ok(bytes.as_bytes().to_vec())
            })
            .map_err(|e: pyo3::PyErr| CoreError::Validation(e.to_string()))
        }));
        Ok(())
    }

    /// Handle request without HTTP (for testing). Returns response bytes.
    fn handle_request(&self, method: &str, path: &str, body: &[u8]) -> PyResult<Vec<u8>> {
        let guard = self
            .inner
            .lock()
            .map_err(|e| pyo3::exceptions::PyRuntimeError::new_err(e.to_string()))?;
        let app = guard
            .as_ref()
            .ok_or_else(|| pyo3::exceptions::PyRuntimeError::new_err("already run"))?;
        app.handle_request(method, path, body)
            .map_err(|e| pyo3::exceptions::PyValueError::new_err(e.to_string()))
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
