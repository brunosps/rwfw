use proc_macro::TokenStream;

mod reactive;
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

#[proc_macro_derive(ReactiveComponent, attributes(reactive))]
pub fn derive_reactive_component(item: TokenStream) -> TokenStream {
    reactive::derive_reactive_component(item)
}

#[proc_macro_attribute]
pub fn reactive_actions(attr: TokenStream, item: TokenStream) -> TokenStream {
    reactive::reactive_actions(attr, item)
}

#[proc_macro_attribute]
pub fn action(_attr: TokenStream, item: TokenStream) -> TokenStream {
    let item: proc_macro2::TokenStream = item.into();
    quote::quote! {
        compile_error!("#[action] can only be used inside an impl block annotated with #[reactive_actions]");
        #item
    }
    .into()
}
