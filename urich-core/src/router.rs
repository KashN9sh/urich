//! Simple router: exact path match. (Radix tree can be added later.)

use std::collections::HashMap;

#[derive(Clone, Copy, Debug, Hash, Eq, PartialEq)]
pub struct RouteId(pub u32);

/// Maps (method, path) -> RouteId. Path is stored as given (e.g. "/orders/commands/create_order").
pub struct Router {
    table: HashMap<(String, String), RouteId>,
}

impl Router {
    pub fn new() -> Self {
        Self {
            table: HashMap::new(),
        }
    }

    pub fn add(&mut self, method: &str, path: &str, id: RouteId) {
        let path = path.trim_start_matches('/');
        self.table
            .insert((method.to_uppercase(), path.to_owned()), id);
    }

    pub fn match_route(&self, method: &str, path: &str) -> Option<RouteId> {
        let path = path.trim_matches('/');
        self.table
            .get(&(method.to_uppercase(), path.to_owned()))
            .copied()
    }
}

impl Default for Router {
    fn default() -> Self {
        Self::new()
    }
}
