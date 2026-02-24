//! HttpModule: route module by prefix. Like Python core/routing.

use std::sync::Arc;
use urich_core::CoreError;

use super::app::Application;
use super::{Handler, Module};

/// HTTP module (bounded context): name + routes. Like Python HttpModule.
/// Attach via app.register(module). Similar to include_router in FastAPI.
pub struct HttpModule {
    pub name: String,
    pub prefix: String,
    routes: Vec<(String, Arc<dyn Fn(serde_json::Value) -> Result<serde_json::Value, CoreError> + Send + Sync>, String)>,
}

impl HttpModule {
    pub fn new(name: &str, prefix: Option<&str>) -> Self {
        Self {
            name: name.to_string(),
            prefix: prefix.unwrap_or(&format!("/{}", name)).to_string(),
            routes: Vec::new(),
        }
    }

    /// Add a route. path without leading slash is under the module prefix.
    /// methods e.g. ["GET"], ["GET", "POST"].
    pub fn route(
        mut self,
        path: &str,
        handler: impl Fn(serde_json::Value) -> Result<serde_json::Value, CoreError> + Send + Sync + 'static,
        methods: &[&str],
    ) -> Self {
        let full_path = format!(
            "{}/{}",
            self.prefix.trim_end_matches('/'),
            path.trim_start_matches('/')
        );
        let arc = Arc::new(handler);
        for method in methods {
            self.routes
                .push((full_path.clone(), arc.clone(), method.to_string()));
        }
        self
    }
}

impl Module for HttpModule {
    fn register_into(&mut self, app: &mut Application) -> Result<(), CoreError> {
        for (path, arc, method) in self.routes.drain(..) {
            let handler: Handler = Box::new(move |v| arc(v));
            app.register_route(&method, &path, None, handler, None)?;
        }
        Ok(())
    }
}
