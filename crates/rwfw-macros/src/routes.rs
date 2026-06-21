use proc_macro::TokenStream;
use proc_macro2::Span;
use quote::quote;
use std::collections::BTreeMap;
use std::path::Path;
use syn::{Ident, ItemStruct, LitStr, parse_macro_input};
use walkdir::WalkDir;

struct RouteEntry {
    /// Axum route path, e.g. "/", "/posts", "/posts/{id}", "/posts/{id}/edit"
    axum_path: String,
    /// Flat module name used with #[path], e.g. "route_index", "route_posts_index"
    flat_mod_name: String,
    /// Relative file path from the routes dir, e.g. "index.rs", "posts/[id].rs"
    relative_path: String,
}

pub fn rwfw_routes_impl(attr: TokenStream, item: TokenStream) -> TokenStream {
    let routes_dir_lit = parse_macro_input!(attr as LitStr);
    let input = parse_macro_input!(item as ItemStruct);
    let name = &input.ident;
    let vis = &input.vis;
    let routes_dir = routes_dir_lit.value();

    let manifest_dir = std::env::var("CARGO_MANIFEST_DIR").expect("CARGO_MANIFEST_DIR not set");
    let routes_path = Path::new(&manifest_dir).join(&routes_dir);

    if !routes_path.exists() {
        return syn::Error::new(
            routes_dir_lit.span(),
            format!("Routes directory not found: {}", routes_path.display()),
        )
        .to_compile_error()
        .into();
    }

    let entries = discover_routes(&routes_path);

    // Group entries by axum_path (multiple entries shouldn't map to same path,
    // but index.rs could)
    let mut path_to_entry: BTreeMap<String, &RouteEntry> = BTreeMap::new();
    for entry in &entries {
        path_to_entry.insert(entry.axum_path.clone(), entry);
    }

    // Generate flat module declarations with #[path = "..."] attributes
    let mod_declarations: Vec<proc_macro2::TokenStream> = entries
        .iter()
        .map(|entry| {
            let mod_name = Ident::new(&entry.flat_mod_name, Span::call_site());
            // Inside `mod routes { ... }`, Rust resolves inner module paths
            // relative to a `routes/` subdir of the parent file's directory.
            // Since we're generating code inside `mod routes { }` which expands
            // in lib.rs (at src/), inner paths are relative to src/routes/.
            // So we just use the filename/relative path directly.
            let rel_path = &entry.relative_path;
            quote! {
                #[path = #rel_path]
                pub mod #mod_name;
            }
        })
        .collect();

    // Generate route registrations
    let route_registrations: Vec<proc_macro2::TokenStream> = path_to_entry
        .iter()
        .map(|(axum_path, entry)| {
            let mod_name = Ident::new(&entry.flat_mod_name, Span::call_site());
            let path_str = axum_path.as_str();
            quote! {
                .route(#path_str, routes::#mod_name::route())
            }
        })
        .collect();

    // Collect all relative directories for cargo:rerun-if-changed
    let expanded = quote! {
        #vis struct #name;

        mod routes {
            #(#mod_declarations)*
        }

        impl #name {
            pub fn generated_routes() -> axum::Router<rwfw_core::app::AppState> {
                axum::Router::new()
                    #(#route_registrations)*
            }
        }
    };

    expanded.into()
}

fn discover_routes(routes_path: &Path) -> Vec<RouteEntry> {
    let mut entries = Vec::new();

    for entry in WalkDir::new(routes_path)
        .sort_by_file_name()
        .into_iter()
        .filter_map(|e| e.ok())
    {
        let path = entry.path();
        if !path.is_file() || path.extension().and_then(|e| e.to_str()) != Some("rs") {
            continue;
        }

        // Skip mod.rs files — they're only for module organization. Route files
        // are ASCII by convention; skip (rather than panic the build on) any path
        // that isn't valid UTF-8.
        let Some(file_stem) = path.file_stem().and_then(|s| s.to_str()) else {
            continue;
        };
        if file_stem == "mod" {
            continue;
        }

        let Ok(relative) = path.strip_prefix(routes_path) else {
            continue;
        };
        let Some(relative_str) = relative.to_str().map(str::to_string) else {
            continue;
        };

        // Build URL path segments from the directory components
        let mut url_segments: Vec<String> = Vec::new();
        let mut name_parts: Vec<String> = Vec::new();

        // Process directory components
        if let Some(parent) = relative.parent() {
            for component in parent.components() {
                let Some(s) = component.as_os_str().to_str() else {
                    continue;
                };
                url_segments.push(segment_to_axum(s));
                name_parts.push(segment_to_ident(s));
            }
        }

        // Process file name
        if file_stem == "index" {
            name_parts.push("index".to_string());
            // index.rs → maps to parent path (no extra segment)
        } else {
            url_segments.push(segment_to_axum(file_stem));
            name_parts.push(segment_to_ident(file_stem));
        }

        let axum_path = if url_segments.is_empty() {
            "/".to_string()
        } else {
            format!("/{}", url_segments.join("/"))
        };

        let flat_mod_name = format!("route_{}", name_parts.join("_"));

        entries.push(RouteEntry {
            axum_path,
            flat_mod_name,
            relative_path: relative_str,
        });
    }

    entries
}

/// Convert a filesystem segment to an Axum route segment
/// "[id]" → "{id}"
/// "[...slug]" → "{*slug}"
/// "posts" → "posts"
fn segment_to_axum(s: &str) -> String {
    if s.starts_with('[') && s.ends_with(']') {
        let inner = &s[1..s.len() - 1];
        if inner.starts_with("...") {
            format!("{{*{}}}", &inner[3..])
        } else {
            format!("{{{inner}}}")
        }
    } else {
        s.to_string()
    }
}

/// Convert a filesystem segment to a valid Rust identifier
/// "[id]" → "param_id"
/// "[...slug]" → "catch_slug"
/// "posts" → "posts"
fn segment_to_ident(s: &str) -> String {
    if s.starts_with('[') && s.ends_with(']') {
        let inner = &s[1..s.len() - 1];
        if inner.starts_with("...") {
            format!("catch_{}", &inner[3..])
        } else {
            format!("param_{inner}")
        }
    } else {
        s.replace('-', "_")
    }
}
