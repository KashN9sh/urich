//! Map custom errors to CoreError in handlers. Shared by facades.

use crate::CoreError;

/// Convert any error to CoreError. Use in handlers: `.map_err(IntoCoreError::into_core_error)`.
pub trait IntoCoreError {
    fn into_core_error(self) -> CoreError;
}

impl<E: std::error::Error + Send + Sync + 'static> IntoCoreError for E {
    fn into_core_error(self) -> CoreError {
        CoreError::Validation(self.to_string())
    }
}
