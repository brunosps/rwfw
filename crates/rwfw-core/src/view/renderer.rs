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

/// Renders MiniJinja templates from disk with dev hot-reload (via
/// `minijinja-autoreload`). Cheap to clone (shared behind an `Arc`).
#[derive(Clone)]
pub struct ViewRenderer {
    reloader: Arc<AutoReloader>,
    roots: Arc<Vec<TemplateRoot>>,
}

impl ViewRenderer {
    pub fn new(roots: Vec<TemplateRoot>) -> Self {
        let stored_roots = Arc::new(roots.clone());
        let reloader = AutoReloader::new(move |notifier| {
            let mut env = Environment::new();
            // Forgiving for HTML authors: `undefined.attr` yields undefined (and
            // renders empty / works with the `default` filter) instead of erroring.
            env.set_undefined_behavior(minijinja::UndefinedBehavior::Chainable);
            for root in &roots {
                if root.dir.exists() {
                    notifier.watch_path(&root.dir, true);
                }
            }

            // Discover `<x-...>` components and wire the tag compiler + attrs filter.
            let registry = super::tags::ComponentRegistry::scan(&roots);
            env.add_filter("attrs", super::tags::attrs_filter);

            let loader_roots = roots.clone();
            env.set_loader(move |name| match load_template(&loader_roots, name) {
                Ok(Some(source)) => Ok(Some(super::tags::compile(&source, &registry))),
                other => other,
            });
            Ok(env)
        });
        Self {
            reloader: Arc::new(reloader),
            roots: stored_roots,
        }
    }

    /// List discovered `<x-...>` components and their declared props (for the
    /// `/components` catalog). Re-scans the roots on demand.
    pub fn components(&self) -> serde_json::Value {
        let registry = super::tags::ComponentRegistry::scan(&self.roots);
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

/// Multi-root loader: tries each root in order. Module roots strip their
/// namespace prefix; the app root matches any name. First file found wins.
fn load_template(roots: &[TemplateRoot], name: &str) -> Result<Option<String>, MjError> {
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
}
