//! HttpModule: route module by prefix. Shared by facades.

use std::sync::Arc;

use crate::application::{Application, Handler};
use crate::module::Module;
use crate::CoreError;

/// HTTP module (bounded context): name + routes. Attach via app.register(module).
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
            let handler: Handler = Box::new(move |v, _c| Box::pin(std::future::ready(arc(v))));
            app.register_route(&method, &path, None, handler, None)?;
        }
        Ok(())
    }
}
