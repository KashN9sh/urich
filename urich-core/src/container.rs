//! Minimal DI container: register instance or factory, resolve by type. Shared by Rust and Python facades.

use std::any::{Any, TypeId};
use std::collections::HashMap;
use thiserror::Error;

#[derive(Error, Debug)]
pub enum ContainerError {
    #[error("no registration for type")]
    NotFound,
}

type FactoryFn = Box<dyn Fn(&mut Container) -> Box<dyn Any + Send + Sync> + Send + Sync>;

/// Minimal DI container: register instance or factory by type or by string key.
pub struct Container {
    store: HashMap<TypeId, Box<dyn Any + Send + Sync>>,
    factories: HashMap<TypeId, FactoryFn>,
    keyed_store: HashMap<String, Box<dyn Any + Send + Sync>>,
    keyed_factories: HashMap<String, FactoryFn>,
}

impl Container {
    pub fn new() -> Self {
        Self {
            store: HashMap::new(),
            factories: HashMap::new(),
            keyed_store: HashMap::new(),
            keyed_factories: HashMap::new(),
        }
    }

    pub fn register_instance<T: Send + Sync + 'static>(&mut self, value: T) {
        self.store.insert(TypeId::of::<T>(), Box::new(value));
    }

    pub fn register_factory<T, F>(&mut self, f: F)
    where
        T: Send + Sync + 'static,
        F: Fn(&mut Container) -> T + Send + Sync + 'static,
    {
        let type_id = TypeId::of::<T>();
        let factory: FactoryFn = Box::new(move |c: &mut Container| {
            let value = f(c);
            Box::new(value) as Box<dyn Any + Send + Sync>
        });
        self.factories.insert(type_id, factory);
    }

    pub fn resolve<T: 'static>(&mut self) -> Result<&T, ContainerError> {
        let type_id = TypeId::of::<T>();
        if self.store.get(&type_id).is_none() {
            if let Some(factory) = self.factories.remove(&type_id) {
                let value = factory(self);
                self.store.insert(type_id, value);
            }
        }
        self.store
            .get(&type_id)
            .and_then(|b| b.downcast_ref::<T>())
            .ok_or(ContainerError::NotFound)
    }

    pub fn resolve_mut<T: 'static>(&mut self) -> Result<&mut T, ContainerError> {
        self.store
            .get_mut(&TypeId::of::<T>())
            .and_then(|boxed| boxed.downcast_mut::<T>())
            .ok_or(ContainerError::NotFound)
    }

    pub fn register_instance_with_key<K: Into<String>, T: Send + Sync + 'static>(&mut self, key: K, value: T) {
        self.keyed_store.insert(key.into(), Box::new(value));
    }

    pub fn register_factory_with_key<K, T, F>(&mut self, key: K, f: F)
    where
        K: Into<String>,
        T: Send + Sync + 'static,
        F: Fn(&mut Container) -> T + Send + Sync + 'static,
    {
        let key = key.into();
        let factory: FactoryFn = Box::new(move |c: &mut Container| {
            let value = f(c);
            Box::new(value) as Box<dyn Any + Send + Sync>
        });
        self.keyed_factories.insert(key, factory);
    }

    pub fn resolve_by_key<T: 'static>(&mut self, key: &str) -> Result<&T, ContainerError> {
        if !self.keyed_store.contains_key(key) {
            if let Some(factory) = self.keyed_factories.remove(key) {
                let value = factory(self);
                self.keyed_store.insert(key.to_string(), value);
            }
        }
        self.keyed_store
            .get(key)
            .and_then(|b| b.downcast_ref::<T>())
            .ok_or(ContainerError::NotFound)
    }
}

impl Default for Container {
    fn default() -> Self {
        Self::new()
    }
}
