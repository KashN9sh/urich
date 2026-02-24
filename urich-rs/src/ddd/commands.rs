//! Command and Query type markers. Like Python ddd/commands.

/// Command: type-driven route name. Like Python Command dataclass; name becomes path segment (snake_case).
pub trait Command {
    fn name() -> &'static str
    where
        Self: Sized;
}

/// Query: type-driven route name. Like Python Query dataclass; name becomes path segment (snake_case).
pub trait Query {
    fn name() -> &'static str
    where
        Self: Sized;
}
