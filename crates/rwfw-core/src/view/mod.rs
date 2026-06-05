//! Server-side HTML view engine (MiniJinja) — the npm-free replacement for the
//! React/Inertia render path. Coexists with `inertia::Inertia` during migration.
//!
//! - [`ViewRenderer`] owns a MiniJinja environment with a layered loader: a disk
//!   overlay (`<exe>/web`), the source roots (one per module, namespaced, plus
//!   an app-level root), and an embedded baseline compiled into the binary.
//! - [`View`] is the Axum extractor handlers use: `v.render("blog/index", props)`.
//! - [`turbo`] builds Turbo Stream fragments for partial / real-time updates.

mod extractor;
mod overlay;
mod renderer;
pub mod tags;
pub mod turbo;

pub use extractor::View;
pub use overlay::exe_overlay_root;
pub use renderer::{
    AssetEmbed, Layers, TemplateEmbed, TemplateRoot, ViewRenderer, safe_join, sanitize,
};
