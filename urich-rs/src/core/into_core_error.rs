//! Map custom errors to CoreError in handlers without writing .map_err(|e| CoreError::Validation(e.to_string())).

use urich_core::CoreError;

/// Convert any error to CoreError. Use in handlers: `.map_err(IntoCoreError::into_core_error)` or `?` with `impl From<YourError> for CoreError` in your app.
pub trait IntoCoreError {
    fn into_core_error(self) -> CoreError;
}

impl<E: std::error::Error + Send + Sync + 'static> IntoCoreError for E {
    fn into_core_error(self) -> CoreError {
        CoreError::Validation(self.to_string())
    }
}
