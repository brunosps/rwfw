use proc_macro::TokenStream;

mod routes;

/// Proc macro that generates an Axum Router from the file structure in `src/routes/`.
///
/// Convention:
/// - `index.rs` maps to the directory's root path
/// - `[param].rs` maps to `/{param}` dynamic segments
/// - Function names map to HTTP methods: `get`, `post`, `put`, `delete`, `patch`
///
/// Example:
/// ```ignore
/// #[rwfw_routes("src/routes")]
/// pub struct MyModule;
/// ```
#[proc_macro_attribute]
pub fn rwfw_routes(attr: TokenStream, item: TokenStream) -> TokenStream {
    routes::rwfw_routes_impl(attr, item)
}
