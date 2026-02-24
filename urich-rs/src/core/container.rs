//! Minimal DI container: register and resolve by type. Like Python core/container.

use std::any::{Any, TypeId};
use std::collections::HashMap;
use thiserror::Error;

#[derive(Error, Debug)]
pub enum ContainerError {
    #[error("no registration for type")]
    NotFound,
}

/// Minimal DI container: register instance by type, resolve by type.
/// Like Python Container; supports only register_instance + resolve (no factory/constructor injection).
pub struct Container {
    store: HashMap<TypeId, Box<dyn Any + Send + Sync>>,
}

impl Container {
    pub fn new() -> Self {
        Self {
            store: HashMap::new(),
        }
    }

    /// Register a ready-made instance. Like Python register_instance.
    pub fn register_instance<T: Send + Sync + 'static>(&mut self, value: T) {
        self.store.insert(TypeId::of::<T>(), Box::new(value));
    }

    /// Resolve an instance by type. Like Python resolve.
    pub fn resolve<T: 'static>(&self) -> Result<&T, ContainerError> {
        self.store
            .get(&TypeId::of::<T>())
            .and_then(|boxed| boxed.downcast_ref::<T>())
            .ok_or(ContainerError::NotFound)
    }

    /// Resolve an instance by type (mutable). For types that need mutability.
    pub fn resolve_mut<T: 'static>(&mut self) -> Result<&mut T, ContainerError> {
        self.store
            .get_mut(&TypeId::of::<T>())
            .and_then(|boxed| boxed.downcast_mut::<T>())
            .ok_or(ContainerError::NotFound)
    }
}

impl Default for Container {
    fn default() -> Self {
        Self::new()
    }
}
