use std::borrow::Cow;
use std::path::{Path, PathBuf};
use std::sync::Arc;

use minijinja::{Environment, Error as MjError, ErrorKind};
use minijinja_autoreload::AutoReloader;

/// A directory of templates, optionally namespaced under a module name.
///
/// `namespace = None` is the app-level root (e.g. `layouts/app.html.j2`).
/// `namespace = Some("blog")` resolves logical names like `blog/index` to
/// `<dir>/index.html.j2`.
#[derive(Clone, Debug)]
pub struct TemplateRoot {
    pub namespace: Option<String>,
    pub dir: PathBuf,
}

impl TemplateRoot {
    pub fn app(dir: impl AsRef<Path>) -> Self {
        Self {
            namespace: None,
            dir: dir.as_ref().to_path_buf(),
        }
    }

    pub fn module(name: impl Into<String>, dir: impl AsRef<Path>) -> Self {
        Self {
            namespace: Some(name.into()),
            dir: dir.as_ref().to_path_buf(),
        }
    }
}

/// An embedded, canonical-keyed template baseline supplied by the app crate (via
/// `rust-embed`). It is the last-resort source: in a packaged binary the source
/// `templates/` directories don't exist on disk, so lookups fall through to here.
///
/// The canonical key is exactly the MiniJinja lookup name (module pages keep
/// their `<module>/` prefix, app pages do not), e.g. `blog/index.html.j2`,
/// `layouts/base.html.j2`.
#[derive(Clone)]
pub struct TemplateEmbed {
    get: Arc<dyn Fn(&str) -> Option<Cow<'static, [u8]>> + Send + Sync>,
    iter: Arc<dyn Fn() -> Vec<Cow<'static, str>> + Send + Sync>,
}

impl TemplateEmbed {
    pub fn new(
        get: impl Fn(&str) -> Option<Cow<'static, [u8]>> + Send + Sync + 'static,
        iter: impl Fn() -> Vec<Cow<'static, str>> + Send + Sync + 'static,
    ) -> Self {
        Self {
            get: Arc::new(get),
            iter: Arc::new(iter),
        }
    }

    pub fn get(&self, name: &str) -> Option<Cow<'static, [u8]>> {
        (self.get)(name)
    }

    pub fn iter(&self) -> Vec<Cow<'static, str>> {
        (self.iter)()
    }
}

impl std::fmt::Debug for TemplateEmbed {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.write_str("TemplateEmbed")
    }
}

/// Embedded static assets (vendored JS + compiled CSS) supplied by the app
/// crate (via `rust-embed`). Packaged fallback when the on-disk web root is
/// absent. Keyed by the path under `vendor/` resp. `assets/`.
#[derive(Clone)]
pub struct AssetEmbed {
    vendor: Arc<dyn Fn(&str) -> Option<Cow<'static, [u8]>> + Send + Sync>,
    asset: Arc<dyn Fn(&str) -> Option<Cow<'static, [u8]>> + Send + Sync>,
}

impl AssetEmbed {
    pub fn new(
        vendor: impl Fn(&str) -> Option<Cow<'static, [u8]>> + Send + Sync + 'static,
        asset: impl Fn(&str) -> Option<Cow<'static, [u8]>> + Send + Sync + 'static,
    ) -> Self {
        Self {
            vendor: Arc::new(vendor),
            asset: Arc::new(asset),
        }
    }

    pub fn vendor(&self, path: &str) -> Option<Cow<'static, [u8]>> {
        (self.vendor)(path)
    }

    pub fn asset(&self, path: &str) -> Option<Cow<'static, [u8]>> {
        (self.asset)(path)
    }
}

impl std::fmt::Debug for AssetEmbed {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.write_str("AssetEmbed")
    }
}

/// Layered template sources, resolved in order:
///   1. disk overlay (`<overlay_root>/templates/...`) — runtime override
///   2. the source roots (live disk) — dev; misses fall through when packaged
///   3. the embedded baseline — always present, compiled into the binary
#[derive(Clone, Debug, Default)]
pub struct Layers {
    pub overlay_root: Option<PathBuf>,
    pub roots: Vec<TemplateRoot>,
    pub embed: Option<TemplateEmbed>,
}

/// Renders MiniJinja templates with dev hot-reload (via `minijinja-autoreload`)
/// across a layered source set (overlay → disk roots → embed). Cheap to clone.
#[derive(Clone)]
pub struct ViewRenderer {
    reloader: Arc<AutoReloader>,
    layers: Arc<Layers>,
}

impl ViewRenderer {
    /// Source-only renderer (no disk overlay, no embedded baseline). Used by
    /// tests and any pure-from-disk setup.
    pub fn new(roots: Vec<TemplateRoot>) -> Self {
        Self::layered(Layers {
            overlay_root: None,
            roots,
            embed: None,
        })
    }

    /// Layered renderer: disk overlay → source roots → embedded baseline.
    pub fn layered(layers: Layers) -> Self {
        let stored = Arc::new(layers.clone());
        let reloader = AutoReloader::new(move |notifier| {
            let mut env = Environment::new();
            // Forgiving for HTML authors: `undefined.attr` yields undefined.
            env.set_undefined_behavior(minijinja::UndefinedBehavior::Chainable);

            // Watch live source dirs + the disk overlay for autoreload. The
            // embedded baseline has nothing to watch.
            for root in &layers.roots {
                if root.dir.exists() {
                    notifier.watch_path(&root.dir, true);
                }
            }
            if let Some(overlay) = &layers.overlay_root {
                let tdir = overlay.join("templates");
                if tdir.exists() {
                    notifier.watch_path(&tdir, true);
                }
            }

            let registry = super::tags::ComponentRegistry::scan_layered(&layers);
            env.add_filter("attrs", super::tags::attrs_filter);

            let layers = layers.clone();
            env.set_loader(move |name| match load_template_layered(&layers, name) {
                Ok(Some(source)) => Ok(Some(super::tags::compile(&source, &registry))),
                other => other,
            });
            Ok(env)
        });
        Self {
            reloader: Arc::new(reloader),
            layers: stored,
        }
    }

    /// List discovered `<x-...>` components and their declared props (for the
    /// `/components` catalog). Re-scans all layers on demand.
    pub fn components(&self) -> serde_json::Value {
        let registry = super::tags::ComponentRegistry::scan_layered(&self.layers);
        let list: Vec<serde_json::Value> = registry
            .list()
            .into_iter()
            .map(|(name, props)| {
                let props: Vec<serde_json::Value> = props
                    .into_iter()
                    .map(|(pname, default)| serde_json::json!({ "name": pname, "default": default }))
                    .collect();
                serde_json::json!({ "name": name, "props": props })
            })
            .collect();
        serde_json::Value::Array(list)
    }

    /// Render the logical template `name` (e.g. `"blog/index"`) with `ctx`.
    pub fn render_to_string(&self, name: &str, ctx: minijinja::Value) -> Result<String, MjError> {
        let env = self
            .reloader
            .acquire_env()
            .map_err(|e| MjError::new(ErrorKind::InvalidOperation, e.to_string()))?;
        let template = env.get_template(&template_name(name))?;
        template.render(ctx)
    }
}

/// Map a logical name to a template file name, defaulting the extension.
fn template_name(name: &str) -> String {
    if name.ends_with(".j2") || name.ends_with(".html") {
        name.to_string()
    } else {
        format!("{name}.html.j2")
    }
}

/// Layered resolver. `name` is the canonical lookup name (already suffixed with
/// `.html.j2` by `template_name`, module pages prefixed with `<module>/`).
fn load_template_layered(layers: &Layers, name: &str) -> Result<Option<String>, MjError> {
    // 1) Disk overlay (sandboxed).
    if let Some(overlay) = &layers.overlay_root {
        if let Some(path) = safe_join(&overlay.join("templates"), name) {
            if path.is_file() {
                return std::fs::read_to_string(&path)
                    .map(Some)
                    .map_err(|e| MjError::new(ErrorKind::TemplateNotFound, e.to_string()));
            }
        }
    }
    // 2) Source roots (live disk; misses fall through in a packaged binary).
    if let Some(src) = load_template_from_roots(&layers.roots, name)? {
        return Ok(Some(src));
    }
    // 3) Embedded baseline (canonical key).
    if let Some(embed) = &layers.embed {
        if let Some(bytes) = embed.get(name) {
            let src = std::str::from_utf8(&bytes)
                .map_err(|e| MjError::new(ErrorKind::SyntaxError, e.to_string()))?
                .to_string();
            return Ok(Some(src));
        }
    }
    Ok(None)
}

/// Multi-root disk loader: tries each root in order. Module roots strip their
/// namespace prefix; the app root matches any name. First file found wins.
fn load_template_from_roots(roots: &[TemplateRoot], name: &str) -> Result<Option<String>, MjError> {
    for root in roots {
        let relative = match &root.namespace {
            Some(ns) => match name.strip_prefix(&format!("{ns}/")) {
                Some(rest) => rest,
                None => continue,
            },
            None => name,
        };
        let path = root.dir.join(relative);
        if path.is_file() {
            return std::fs::read_to_string(&path)
                .map(Some)
                .map_err(|e| MjError::new(ErrorKind::TemplateNotFound, e.to_string()));
        }
    }
    Ok(None)
}

/// Reject absolute, traversal, NUL, and backslash-smuggled relative keys.
/// Returns a normalized forward-slash relative key, or `None` if unsafe.
pub fn sanitize(path: &str) -> Option<String> {
    if path.is_empty() || path.contains('\0') {
        return None;
    }
    let mut out = Vec::new();
    for seg in path.split('/') {
        match seg {
            "" | "." => continue,
            ".." => return None,
            s if s.contains('\\') => return None,
            s => out.push(s),
        }
    }
    if out.is_empty() {
        return None;
    }
    Some(out.join("/"))
}

/// Join `rel` under `base` and canonicalize, ensuring the result does not escape
/// `base` (defends against `..` and symlinks). Returns `None` on escape/missing —
/// callers treat that as a miss and fall through to the next layer.
pub fn safe_join(base: &Path, rel: &str) -> Option<PathBuf> {
    let rel = sanitize(rel)?;
    let candidate = base.join(&rel);
    let base_c = base.canonicalize().ok()?;
    match candidate.canonicalize() {
        Ok(c) if c.starts_with(&base_c) => Some(c),
        _ => None,
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::sync::atomic::{AtomicU64, Ordering};

    static COUNTER: AtomicU64 = AtomicU64::new(0);

    fn temp_dir() -> PathBuf {
        let n = COUNTER.fetch_add(1, Ordering::Relaxed);
        let dir = std::env::temp_dir().join(format!("rwfw_view_{}_{}", std::process::id(), n));
        std::fs::create_dir_all(&dir).unwrap();
        dir
    }

    #[test]
    fn renders_app_template_with_context() {
        let dir = temp_dir();
        std::fs::write(dir.join("hello.html.j2"), "<p>Hello {{ name }}</p>").unwrap();

        let renderer = ViewRenderer::new(vec![TemplateRoot::app(&dir)]);
        let html = renderer
            .render_to_string("hello", minijinja::context! { name => "World" })
            .unwrap();

        assert_eq!(html, "<p>Hello World</p>");
    }

    #[test]
    fn resolves_module_namespace_with_app_layout_fallback() {
        let app_dir = temp_dir();
        let blog_dir = temp_dir();
        std::fs::create_dir_all(app_dir.join("layouts")).unwrap();
        std::fs::write(
            app_dir.join("layouts/base.html.j2"),
            "<main>{% block content %}{% endblock %}</main>",
        )
        .unwrap();
        std::fs::write(
            blog_dir.join("index.html.j2"),
            "{% extends \"layouts/base.html.j2\" %}{% block content %}{{ title }}{% endblock %}",
        )
        .unwrap();

        let renderer = ViewRenderer::new(vec![
            TemplateRoot::module("blog", &blog_dir),
            TemplateRoot::app(&app_dir),
        ]);
        let html = renderer
            .render_to_string("blog/index", minijinja::context! { title => "Posts" })
            .unwrap();

        assert_eq!(html, "<main>Posts</main>");
    }

    #[test]
    fn missing_template_is_an_error() {
        let dir = temp_dir();
        let renderer = ViewRenderer::new(vec![TemplateRoot::app(&dir)]);
        let err = renderer
            .render_to_string("nope", minijinja::context! {})
            .unwrap_err();
        assert_eq!(err.kind(), ErrorKind::TemplateNotFound);
    }

    #[test]
    fn renders_component_with_props_content_and_attrs() {
        let dir = temp_dir();
        std::fs::create_dir_all(dir.join("components/button")).unwrap();
        std::fs::write(
            dir.join("components/button/index.html.j2"),
            "{#def kind=\"primary\" #}\n<button {{ __attrs | attrs(class=\"btn btn-\" ~ kind) }}>{{ content }}</button>",
        )
        .unwrap();
        std::fs::write(
            dir.join("page.html.j2"),
            "<x-button kind=\"danger\" id=\"go\"><strong>Delete</strong></x-button>",
        )
        .unwrap();

        let renderer = ViewRenderer::new(vec![TemplateRoot::app(&dir)]);
        let html = renderer.render_to_string("page", minijinja::context! {}).unwrap();

        assert!(html.contains("class=\"btn btn-danger\""), "got: {html}");
        assert!(html.contains("id=\"go\""), "got: {html}");
        // HTML content is not double-escaped.
        assert!(html.contains("<strong>Delete</strong>"), "got: {html}");
    }

    #[test]
    fn renders_named_slot_with_html() {
        let dir = temp_dir();
        std::fs::create_dir_all(dir.join("components/card")).unwrap();
        std::fs::write(
            dir.join("components/card/index.html.j2"),
            "{#def title #}\n<div class=\"card\"><h2>{{ title }}</h2>{{ content }}{% if slots.footer %}<footer>{{ slots.footer }}</footer>{% endif %}</div>",
        )
        .unwrap();
        std::fs::write(
            dir.join("page.html.j2"),
            "<x-card title=\"Hello\"><p>Body</p><x-slot name=\"footer\"><a href=\"/x\">Foot</a></x-slot></x-card>",
        )
        .unwrap();

        let renderer = ViewRenderer::new(vec![TemplateRoot::app(&dir)]);
        let html = renderer.render_to_string("page", minijinja::context! {}).unwrap();

        assert!(html.contains("<h2>Hello</h2>"), "got: {html}");
        assert!(html.contains("<p>Body</p>"), "got: {html}");
        assert!(html.contains("<footer><a href=\"/x\">Foot</a></footer>"), "got: {html}");
    }

    #[test]
    fn disk_overlay_overrides_embedded_baseline() {
        // Embedded baseline serves "page" -> EMBED; an overlay file overrides it.
        let overlay = temp_dir();
        std::fs::create_dir_all(overlay.join("templates")).unwrap();
        std::fs::write(overlay.join("templates/page.html.j2"), "OVERLAY").unwrap();

        let embed = TemplateEmbed::new(
            |name| (name == "page.html.j2").then(|| Cow::Borrowed(&b"EMBED"[..])),
            || vec![Cow::Borrowed("page.html.j2")],
        );

        // With overlay present: overlay wins.
        let with_overlay = ViewRenderer::layered(Layers {
            overlay_root: Some(overlay.clone()),
            roots: vec![],
            embed: Some(embed.clone()),
        });
        assert_eq!(
            with_overlay
                .render_to_string("page", minijinja::context! {})
                .unwrap(),
            "OVERLAY"
        );

        // Without overlay: embedded baseline is used.
        let embed_only = ViewRenderer::layered(Layers {
            overlay_root: None,
            roots: vec![],
            embed: Some(embed),
        });
        assert_eq!(
            embed_only
                .render_to_string("page", minijinja::context! {})
                .unwrap(),
            "EMBED"
        );
    }

    #[test]
    fn sanitize_rejects_traversal() {
        assert_eq!(sanitize("a/b.css").as_deref(), Some("a/b.css"));
        assert_eq!(sanitize("./a//b.css").as_deref(), Some("a/b.css"));
        assert!(sanitize("../secret").is_none());
        assert!(sanitize("a/../../b").is_none());
        assert!(sanitize("").is_none());
    }
}
