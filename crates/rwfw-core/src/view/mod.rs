//! Server-side HTML view engine (MiniJinja) — the npm-free replacement for the
//! React/Inertia render path. Coexists with `inertia::Inertia` during migration.
//!
//! - [`ViewRenderer`] owns a MiniJinja environment with a multi-root loader
//!   (one root per module, namespaced by module name, plus an app-level root).
//! - [`View`] is the Axum extractor handlers use: `v.render("blog/index", props)`.
//! - [`turbo`] builds Turbo Stream fragments for partial / real-time updates.

mod extractor;
mod renderer;
pub mod tags;
pub mod turbo;

pub use extractor::View;
pub use renderer::{TemplateRoot, ViewRenderer};
