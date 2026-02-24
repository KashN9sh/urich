//! Repository: persistence for an aggregate. Like Python Repository[T].

use urich_core::CoreError;

/// Repository: persistence for an aggregate. Like Python Repository[T].
pub trait Repository<A>: Send {
    fn get(&self, id: &str) -> Result<Option<A>, CoreError>;
    fn add(&mut self, aggregate: A) -> Result<(), CoreError>;
    fn save(&mut self, aggregate: &A) -> Result<(), CoreError>;
}
