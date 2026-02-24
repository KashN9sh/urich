//! Command and Query: type-driven name + structure. Like Python Command/Query dataclasses.

use std::any::TypeId;
use std::collections::HashMap;
use std::sync::Mutex;

use serde::de::DeserializeOwned;

/// PascalCase → snake_case. E.g. "CreateOrder" → "create_order".
fn type_name_to_snake_case(name: &str) -> String {
    let short = name.rsplit("::").next().unwrap_or(name);
    let mut out = String::with_capacity(short.len());
    for (i, c) in short.chars().enumerate() {
        if c.is_uppercase() {
            if i > 0 {
                out.push('_');
            }
            out.extend(c.to_lowercase());
        } else {
            out.push(c);
        }
    }
    out
}

static COMMAND_QUERY_NAMES: std::sync::OnceLock<Mutex<HashMap<TypeId, &'static str>>> =
    std::sync::OnceLock::new();

fn cached_type_name<T: ?Sized + 'static>() -> &'static str {
    let cache = COMMAND_QUERY_NAMES.get_or_init(|| Mutex::new(HashMap::new()));
    let id = TypeId::of::<T>();
    {
        let guard = cache.lock().unwrap();
        if let Some(&s) = guard.get(&id) {
            return s;
        }
    }
    let name = type_name_to_snake_case(std::any::type_name::<T>());
    let leaked: &'static str = Box::leak(name.into_boxed_str());
    cache.lock().unwrap().insert(id, leaked);
    leaked
}

/// Command: type describes route name and body shape. Like Python `@dataclass class CreateOrder(Command): order_id: str`.
/// Name defaults to snake_case of the type name (e.g. `CreateOrder` → `create_order`). Override `name()` to customize.
pub trait Command: DeserializeOwned
where
    Self: 'static,
{
    fn name() -> &'static str
    where
        Self: Sized,
    {
        cached_type_name::<Self>()
    }
}

/// Query: type describes route name and params shape. Like Python `@dataclass class GetOrder(Query): order_id: str`.
/// Name defaults to snake_case of the type name. Override `name()` to customize.
pub trait Query: DeserializeOwned
where
    Self: 'static,
{
    fn name() -> &'static str
    where
        Self: Sized,
    {
        cached_type_name::<Self>()
    }
}
