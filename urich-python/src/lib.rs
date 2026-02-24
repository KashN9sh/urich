//! Python bindings for urich-core. Exposes register_route, set_handler, handle_request, openapi_spec.

use pyo3::prelude::*;
use std::sync::Mutex;
use urich_core::{App, CoreError, RouteId};

#[pyclass]
struct CoreApp {
    inner: Mutex<App>,
    handlers: Mutex<Option<pyo3::Py<pyo3::PyAny>>>,
}

#[pymethods]
impl CoreApp {
    #[new]
    fn new() -> Self {
        Self {
            inner: Mutex::new(App::new()),
            handlers: Mutex::new(None),
        }
    }

    /// Register a route. Returns route_id (int).
    #[pyo3(signature = (method, path, request_schema=None))]
    fn register_route(
        &self,
        method: &str,
        path: &str,
        request_schema: Option<&str>,
    ) -> PyResult<u32> {
        let schema = request_schema.and_then(|s| serde_json::from_str(s).ok());
        let mut app = self
            .inner
            .lock()
            .map_err(|e| pyo3::exceptions::PyRuntimeError::new_err(e.to_string()))?;
        let id = app
            .register_route(method, path, schema)
            .map_err(|e| pyo3::exceptions::PyValueError::new_err(e.to_string()))?;
        Ok(id.0)
    }

    /// Set the Python callable invoked as (route_id: int, body_bytes: bytes) -> response_bytes: bytes.
    fn set_handler(&self, handler: pyo3::Py<pyo3::PyAny>) -> PyResult<()> {
        let handlers = Python::with_gil(|py| handler.clone_ref(py));
        *self.handlers.lock().unwrap() = Some(handler);
        let mut app = self
            .inner
            .lock()
            .map_err(|e| pyo3::exceptions::PyRuntimeError::new_err(e.to_string()))?;
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
        let app = self
            .inner
            .lock()
            .map_err(|e| pyo3::exceptions::PyRuntimeError::new_err(e.to_string()))?;
        app.handle_request(method, path, body)
            .map_err(|e| pyo3::exceptions::PyValueError::new_err(e.to_string()))
    }

    /// OpenAPI spec as JSON string.
    fn openapi_spec(&self, title: &str, version: &str) -> PyResult<String> {
        let app = self
            .inner
            .lock()
            .map_err(|e| pyo3::exceptions::PyRuntimeError::new_err(e.to_string()))?;
        let spec = app.openapi_spec(title, version);
        serde_json::to_string(&spec)
            .map_err(|e| pyo3::exceptions::PyValueError::new_err(e.to_string()))
    }
}

#[pymodule]
fn urich_core_native(m: &Bound<'_, PyModule>) -> PyResult<()> {
    m.add_class::<CoreApp>()?;
    Ok(())
}
