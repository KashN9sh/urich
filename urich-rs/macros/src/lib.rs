//! Derive macros for urich-rs. Use `#[derive(Command)]` and `#[derive(Query)]` so you don't need `impl Command for T {}`.

use proc_macro::TokenStream;
use quote::quote;
use syn::{parse_macro_input, DeriveInput};

/// Implements the `Command` trait. Name is derived from the type (e.g. `CreateOrder` → `create_order`).
/// Requires `Command` to be in scope (e.g. `use urich_rs::Command`).
#[proc_macro_derive(Command)]
pub fn derive_command(input: TokenStream) -> TokenStream {
    let input = parse_macro_input!(input as DeriveInput);
    let name = &input.ident;
    let expanded = quote! {
        impl Command for #name {}
    };
    TokenStream::from(expanded)
}

/// Implements the `Query` trait. Name is derived from the type (e.g. `GetOrder` → `get_order`).
/// Requires `Query` to be in scope (e.g. `use urich_rs::Query`).
#[proc_macro_derive(Query)]
pub fn derive_query(input: TokenStream) -> TokenStream {
    let input = parse_macro_input!(input as DeriveInput);
    let name = &input.ident;
    let expanded = quote! {
        impl Query for #name {}
    };
    TokenStream::from(expanded)
}
