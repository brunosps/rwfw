//! Compile-time embedded baseline of the framework's web assets, flattened by
//! `build.rs` into `$OUT_DIR`. Injected into `RwfwApp` as the packaged fallback
//! for templates and static assets.

use rust_embed::RustEmbed;
use rwfw_core::view::{AssetEmbed, TemplateEmbed};

/// Flattened, canonical-keyed templates (app at root, modules under `<module>/`).
#[derive(RustEmbed)]
#[folder = "$OUT_DIR/embed_templates"]
struct Templates;

/// Vendored JS (Turbo, Stimulus). Locked — never overridable by the disk overlay.
#[derive(RustEmbed)]
#[folder = "$OUT_DIR/embed_vendor"]
struct Vendor;

/// Compiled CSS and other static assets.
#[derive(RustEmbed)]
#[folder = "$OUT_DIR/embed_assets"]
struct Assets;

/// The embedded template baseline passed to `RwfwApp::template_embed`.
pub fn template_embed() -> TemplateEmbed {
    TemplateEmbed::new(
        |name| Templates::get(name).map(|f| f.data),
        || Templates::iter().collect(),
    )
}

/// The embedded static assets passed to `RwfwApp::asset_embed`.
pub fn asset_embed() -> AssetEmbed {
    AssetEmbed::new(
        |path| Vendor::get(path).map(|f| f.data),
        |path| Assets::get(path).map(|f| f.data),
    )
}
