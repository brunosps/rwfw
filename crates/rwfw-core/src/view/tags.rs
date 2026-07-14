//! `<x-name>` tag component compiler.
//!
//! A string -> string preprocessor (run by the loader before MiniJinja parses a
//! template) that rewrites HTML-like component tags into plain MiniJinja:
//!
//! ```html
//! <x-alert type="error" class="mt-4">
//!   Boom.
//!   <x-slot name="actions"><a href="/retry">Retry</a></x-slot>
//! </x-alert>
//! ```
//!
//! becomes a `{% with %}` + `{% include %}` of `components/alert/index.html.j2`,
//! passing declared props as variables, the default body as `content`, named
//! slots as `slots.*`, and any undeclared attributes as `__attrs` (rendered with
//! the `attrs` filter, which merges classes).
//!
//! Components declare their props with a leading `{#def ... #}` comment, e.g.
//! `{#def type="info", dismissible=false #}`.

use std::collections::BTreeMap;
use std::path::Path;
use std::sync::atomic::{AtomicU64, Ordering};

use super::renderer::TemplateRoot;

static COUNTER: AtomicU64 = AtomicU64::new(0);

fn next_id() -> u64 {
    COUNTER.fetch_add(1, Ordering::Relaxed)
}

/// A component's declared props: ordered (name, optional default-expression).
#[derive(Debug, Clone, Default)]
pub struct ComponentDef {
    pub props: Vec<(String, Option<String>)>,
}

impl ComponentDef {
    fn has_prop(&self, name: &str) -> bool {
        self.props.iter().any(|(p, _)| p == name)
    }
}

/// Registry of components discovered under each root's `components/<name>/index.html.j2`.
#[derive(Debug, Default, Clone)]
pub struct ComponentRegistry {
    defs: BTreeMap<String, ComponentDef>,
}

impl ComponentRegistry {
    /// Scan the given roots for `components/<name>/index.html.j2` files and parse
    /// their `{#def ... #}` declarations.
    pub fn scan(roots: &[TemplateRoot]) -> Self {
        let mut defs = BTreeMap::new();
        for root in roots {
            let components_dir = root.dir.join("components");
            scan_dir(&components_dir, &mut defs);
        }
        Self { defs }
    }

    /// Scan all layers (embedded baseline, source roots, disk overlay) for
    /// components, with later layers overriding earlier ones.
    pub fn scan_layered(layers: &super::renderer::Layers) -> Self {
        let mut defs = BTreeMap::new();

        // 1) Embedded baseline (lowest precedence).
        if let Some(embed) = &layers.embed {
            for key in embed.iter() {
                let Some(name) = key
                    .strip_prefix("components/")
                    .and_then(|r| r.strip_suffix("/index.html.j2"))
                else {
                    continue;
                };
                if name.contains('/') {
                    continue; // top-level component dirs only
                }
                if let Some(bytes) = embed.get(&key) {
                    if let Ok(src) = std::str::from_utf8(&bytes) {
                        defs.entry(name.to_string())
                            .or_insert_with(|| parse_def(src));
                    }
                }
            }
        }

        // 2) Source roots override the baseline.
        for root in &layers.roots {
            scan_dir_override(&root.dir.join("components"), &mut defs);
        }

        // 3) Disk overlay wins.
        if let Some(overlay) = &layers.overlay_root {
            scan_dir_override(&overlay.join("templates").join("components"), &mut defs);
        }

        Self { defs }
    }

    fn get(&self, name: &str) -> Option<&ComponentDef> {
        self.defs.get(name)
    }

    /// All components and their declared props, sorted by name.
    pub fn list(&self) -> Vec<(String, Vec<(String, Option<String>)>)> {
        self.defs
            .iter()
            .map(|(name, def)| (name.clone(), def.props.clone()))
            .collect()
    }
}

fn scan_dir(dir: &Path, defs: &mut BTreeMap<String, ComponentDef>) {
    let Ok(entries) = std::fs::read_dir(dir) else {
        return;
    };
    for entry in entries.flatten() {
        let path = entry.path();
        if !path.is_dir() {
            continue;
        }
        let Some(name) = path.file_name().and_then(|n| n.to_str()) else {
            continue;
        };
        let index = path.join("index.html.j2");
        if let Ok(source) = std::fs::read_to_string(&index) {
            defs.entry(name.to_string())
                .or_insert_with(|| parse_def(&source));
        }
    }
}

/// Like [`scan_dir`] but overrides existing entries (higher-precedence layer).
fn scan_dir_override(dir: &Path, defs: &mut BTreeMap<String, ComponentDef>) {
    let Ok(entries) = std::fs::read_dir(dir) else {
        return;
    };
    for entry in entries.flatten() {
        let path = entry.path();
        if !path.is_dir() {
            continue;
        }
        let Some(name) = path.file_name().and_then(|n| n.to_str()) else {
            continue;
        };
        if let Ok(source) = std::fs::read_to_string(path.join("index.html.j2")) {
            defs.insert(name.to_string(), parse_def(&source));
        }
    }
}

/// Parse a `{#def name, other="x", flag=false #}` declaration from a component.
pub fn parse_def(source: &str) -> ComponentDef {
    let mut def = ComponentDef::default();
    let Some(start) = source.find("{#def") else {
        return def;
    };
    let rest = &source[start + 5..];
    let Some(end) = rest.find("#}") else {
        return def;
    };
    let body = rest[..end].trim();
    for part in split_top_level(body, ',') {
        let part = part.trim();
        if part.is_empty() {
            continue;
        }
        if let Some((name, default)) = part.split_once('=') {
            def.props
                .push((name.trim().to_string(), Some(default.trim().to_string())));
        } else {
            def.props.push((part.to_string(), None));
        }
    }
    def
}

/// Compile all `<x-...>` tags in `src` into plain MiniJinja.
/// Name that no template context defines, so binding a prop to it yields undefined.
///
/// The renderer runs with `UndefinedBehavior::Chainable`, so reading an unknown name is undefined
/// rather than an error -- which is exactly the value an unpassed, undefaulted prop should hold.
const UNDEFINED_PROP: &str = "__rwfw_undefined_prop";

pub fn compile(src: &str, registry: &ComponentRegistry) -> String {
    let mut out = src.to_string();
    // Self-closing tags first (they can sit inside paired bodies).
    while let Some(tag) = find_self_closing(&out) {
        let replacement = emit(&tag.name, &tag.attrs, "", registry);
        out.replace_range(tag.start..tag.end, &replacement);
    }
    // Then paired tags, innermost first (match each first close to its open).
    while let Some(pair) = find_innermost_pair(&out) {
        let replacement = emit(&pair.name, &pair.attrs, &pair.body, registry);
        out.replace_range(pair.start..pair.end, &replacement);
    }
    out
}

struct SelfClosing {
    start: usize,
    end: usize,
    name: String,
    attrs: String,
}

struct Pair {
    start: usize,
    end: usize,
    name: String,
    attrs: String,
    body: String,
}

fn is_component_name_char(c: char) -> bool {
    c.is_ascii_alphanumeric() || c == '-' || c == '_'
}

/// Read a component tag name starting at `<x-` located at `lt` (index of '<').
/// Returns (name, index just after the name) if it is an `<x-NAME` opener and
/// NAME != "slot".
fn read_open_name(src: &str, lt: usize) -> Option<(String, usize)> {
    let bytes = src.as_bytes();
    if !src[lt..].starts_with("<x-") {
        return None;
    }
    let name_start = lt + 3;
    let mut i = name_start;
    while i < bytes.len() && is_component_name_char(bytes[i] as char) {
        i += 1;
    }
    let name = &src[name_start..i];
    if name.is_empty() || name == "slot" {
        return None;
    }
    Some((name.to_string(), i))
}

fn find_self_closing(src: &str) -> Option<SelfClosing> {
    let mut search = 0;
    while let Some(rel) = src[search..].find("<x-") {
        let lt = search + rel;
        if let Some((name, after_name)) = read_open_name(src, lt) {
            if let Some((attrs_end, self_closing)) = read_tag_end(src, after_name) {
                if self_closing {
                    return Some(SelfClosing {
                        start: lt,
                        end: attrs_end,
                        name,
                        attrs: src[after_name..attrs_end - 2].trim().to_string(),
                    });
                }
            }
        }
        search = lt + 3;
    }
    None
}

/// Find the first component close `</x-NAME>` and match it to the nearest
/// preceding `<x-NAME ...>` open — that pair is innermost.
fn find_innermost_pair(src: &str) -> Option<Pair> {
    let mut search = 0;
    loop {
        let rel = src[search..].find("</x-")?;
        let close_lt = search + rel;
        let after = close_lt + 4;
        let name_end = src[after..]
            .find('>')
            .map(|p| after + p)
            .unwrap_or(src.len());
        let name = src[after..name_end].trim().to_string();
        if name == "slot" {
            search = close_lt + 4;
            continue;
        }
        let close_end = name_end + 1;

        // Find the matching open: nearest `<x-NAME` before close_lt.
        let open_marker = format!("<x-{name}");
        let open_lt = src[..close_lt].rfind(&open_marker)?;
        let after_name = open_lt + open_marker.len();
        let (attrs_end, _self_closing) = read_tag_end(src, after_name)?;
        let attrs = src[after_name..attrs_end - 1].trim().to_string();
        let body = src[attrs_end..close_lt].to_string();
        return Some(Pair {
            start: open_lt,
            end: close_end,
            name,
            attrs,
            body,
        });
    }
}

/// Given index just after the tag name, find the end of the opening tag.
/// Returns (index just past `>`, self_closing). Respects quoted attribute values.
fn read_tag_end(src: &str, mut i: usize) -> Option<(usize, bool)> {
    let bytes = src.as_bytes();
    let mut quote: Option<u8> = None;
    while i < bytes.len() {
        let c = bytes[i];
        match quote {
            Some(q) => {
                if c == q {
                    quote = None;
                }
            }
            None => match c {
                b'"' | b'\'' => quote = Some(c),
                b'>' => {
                    let self_closing = i > 0 && bytes[i - 1] == b'/';
                    return Some((i + 1, self_closing));
                }
                _ => {}
            },
        }
        i += 1;
    }
    None
}

#[derive(Debug)]
struct Attr {
    name: String,
    value: String,
    is_expr: bool,
    is_boolean: bool,
}

fn parse_attrs(attrs: &str) -> Vec<Attr> {
    // Drop a trailing self-closing slash.
    let attrs = attrs.trim().trim_end_matches('/').trim();
    let bytes = attrs.as_bytes();
    let mut i = 0;
    let mut out = Vec::new();
    while i < bytes.len() {
        while i < bytes.len() && (bytes[i] as char).is_whitespace() {
            i += 1;
        }
        let name_start = i;
        while i < bytes.len() && !(bytes[i] as char).is_whitespace() && bytes[i] != b'=' {
            i += 1;
        }
        if i == name_start {
            break;
        }
        let raw_name = &attrs[name_start..i];
        if i < bytes.len() && bytes[i] == b'=' {
            i += 1; // skip '='
            let quote = bytes.get(i).copied();
            let (value, end) = if quote == Some(b'"') || quote == Some(b'\'') {
                let q = quote.unwrap();
                let vstart = i + 1;
                let mut j = vstart;
                while j < bytes.len() && bytes[j] != q {
                    j += 1;
                }
                (attrs[vstart..j].to_string(), j + 1)
            } else {
                let vstart = i;
                let mut j = vstart;
                while j < bytes.len() && !(bytes[j] as char).is_whitespace() {
                    j += 1;
                }
                (attrs[vstart..j].to_string(), j)
            };
            i = end;
            let is_expr = raw_name.starts_with(':');
            out.push(Attr {
                name: raw_name.trim_start_matches(':').to_string(),
                value,
                is_expr,
                is_boolean: false,
            });
        } else {
            // bare attribute -> boolean true
            out.push(Attr {
                name: raw_name.trim_start_matches(':').to_string(),
                value: "true".to_string(),
                is_expr: false,
                is_boolean: true,
            });
        }
    }
    out
}

fn jinja_string_literal(value: &str) -> String {
    format!("\"{}\"", value.replace('\\', "\\\\").replace('"', "\\\""))
}

/// Emit MiniJinja for a single component instance.
fn emit(name: &str, attrs_str: &str, body: &str, registry: &ComponentRegistry) -> String {
    let attrs = parse_attrs(attrs_str);
    let def = registry.get(name).cloned().unwrap_or_default();

    // Split passed attributes into declared props vs pass-through attrs.
    let mut passed_props: BTreeMap<String, String> = BTreeMap::new();
    let mut passthrough: Vec<(String, String)> = Vec::new();
    for attr in &attrs {
        let value_expr = if attr.is_boolean {
            "true".to_string()
        } else if attr.is_expr {
            attr.value.clone()
        } else {
            jinja_string_literal(&attr.value)
        };
        if def.has_prop(&attr.name) {
            passed_props.insert(attr.name.clone(), value_expr);
        } else {
            passthrough.push((attr.name.clone(), value_expr));
        }
    }

    // Resolve prop values: passed value, else declared default, else undefined.
    //
    // A declared prop that is neither passed nor defaulted must still ENTER the component's scope,
    // bound to undefined. Leaving it out of the `with` block is not the same as undefined: the name
    // then falls through to the CALLER's context, so a component that declares `label` and is used
    // on a page that happens to have a `label` renders the page's label instead of nothing. The
    // component silently shows someone else's data.
    let mut with_kv: Vec<String> = Vec::new();
    for (pname, default) in &def.props {
        let value = match (passed_props.get(pname), default) {
            (Some(passed), _) => passed.clone(),
            (None, Some(declared)) => declared.clone(),
            (None, None) => UNDEFINED_PROP.to_string(),
        };
        with_kv.push(format!("{pname}={value}"));
    }

    let attrs_dict = format!(
        "{{{}}}",
        passthrough
            .iter()
            .map(|(k, v)| format!("{}: {}", jinja_string_literal(k), v))
            .collect::<Vec<_>>()
            .join(", ")
    );

    // Extract named slots from the body; the remainder is the default content.
    let (default_body, slots) = extract_slots(body);
    let id = next_id();

    let mut prelude = String::new();
    let content_var = format!("__c{id}");
    prelude.push_str(&format!(
        "{{% set {content_var} %}}{}{{% endset %}}",
        compile(&default_body, registry)
    ));

    let mut slot_entries = Vec::new();
    for (idx, slot) in slots.iter().enumerate() {
        let var = format!("__s{id}_{idx}");
        prelude.push_str(&format!(
            "{{% set {var} %}}{}{{% endset %}}",
            compile(&slot.body, registry)
        ));
        slot_entries.push(format!("{}: {var}", jinja_string_literal(&slot.name)));
    }
    let slots_dict = format!("{{{}}}", slot_entries.join(", "));

    let mut with_vars = vec![
        format!("content={content_var}"),
        format!("__attrs={attrs_dict}"),
        format!("slots={slots_dict}"),
    ];
    with_vars.extend(with_kv);

    format!(
        "{prelude}{{% with {} %}}{{% include \"components/{name}/index.html.j2\" %}}{{% endwith %}}",
        with_vars.join(", ")
    )
}

struct Slot {
    name: String,
    body: String,
}

/// Pull `<x-slot name="..">..</x-slot>` blocks out of a component body; returns
/// the remaining default content plus the captured slots.
fn extract_slots(body: &str) -> (String, Vec<Slot>) {
    let mut remaining = String::new();
    let mut slots = Vec::new();
    let mut rest = body;
    while let Some(rel) = rest.find("<x-slot") {
        remaining.push_str(&rest[..rel]);
        let open = &rest[rel..];
        let Some((attrs_end, _)) = read_tag_end(open, 7) else {
            remaining.push_str(open);
            rest = "";
            break;
        };
        let attrs = parse_attrs(&open[7..attrs_end - 1]);
        let name = attrs
            .iter()
            .find(|a| a.name == "name")
            .map(|a| a.value.clone())
            .unwrap_or_default();
        let after_open = &open[attrs_end..];
        let Some(close_rel) = after_open.find("</x-slot>") else {
            remaining.push_str(open);
            rest = "";
            break;
        };
        let slot_body = &after_open[..close_rel];
        slots.push(Slot {
            name,
            body: slot_body.to_string(),
        });
        rest = &after_open[close_rel + "</x-slot>".len()..];
    }
    remaining.push_str(rest);
    (remaining, slots)
}

/// Split `s` on `sep`, ignoring separators inside quotes.
fn split_top_level(s: &str, sep: char) -> Vec<String> {
    let mut parts = Vec::new();
    let mut current = String::new();
    let mut quote: Option<char> = None;
    for c in s.chars() {
        match quote {
            Some(q) => {
                if c == q {
                    quote = None;
                }
                current.push(c);
            }
            None => {
                if c == '"' || c == '\'' {
                    quote = Some(c);
                    current.push(c);
                } else if c == sep {
                    parts.push(std::mem::take(&mut current));
                } else {
                    current.push(c);
                }
            }
        }
    }
    parts.push(current);
    parts
}

/// MiniJinja `attrs` filter: render a pass-through attribute map as HTML
/// attributes, merging `class` with an optional base passed as a keyword.
pub fn attrs_filter(
    value: minijinja::Value,
    kwargs: minijinja::value::Kwargs,
) -> Result<minijinja::Value, minijinja::Error> {
    use std::fmt::Write as _;

    let base_class: Option<String> = kwargs.get("class").ok();
    kwargs.assert_all_used()?;

    let mut classes: Vec<String> = Vec::new();
    if let Some(base) = &base_class {
        classes.extend(base.split_whitespace().map(String::from));
    }

    let mut rendered = String::new();
    if let Ok(iter) = value.try_iter() {
        for key in iter {
            let key_str = key.to_string();
            let val = value.get_item(&key).unwrap_or(minijinja::Value::UNDEFINED);
            if key_str == "class" {
                classes.extend(val.to_string().split_whitespace().map(String::from));
                continue;
            }
            if val.is_true() && val.as_str().is_none() {
                // boolean attribute
                let _ = write!(rendered, " {key_str}");
            } else {
                let _ = write!(rendered, " {key_str}=\"{}\"", html_escape(&val.to_string()));
            }
        }
    }

    if !classes.is_empty() {
        let merged = classes.join(" ");
        rendered = format!(" class=\"{}\"{rendered}", html_escape(&merged));
    }

    Ok(minijinja::Value::from_safe_string(rendered))
}

fn html_escape(input: &str) -> String {
    input
        .replace('&', "&amp;")
        .replace('"', "&quot;")
        .replace('<', "&lt;")
        .replace('>', "&gt;")
}

#[cfg(test)]
mod tests {
    use super::*;

    fn reg(name: &str, source: &str) -> ComponentRegistry {
        let mut defs = BTreeMap::new();
        defs.insert(name.to_string(), parse_def(source));
        ComponentRegistry { defs }
    }

    #[test]
    fn parses_def_with_defaults() {
        let def = parse_def("{#def type=\"info\", dismissible=false, title #}\n<div></div>");
        assert_eq!(def.props.len(), 3);
        assert_eq!(def.props[0], ("type".into(), Some("\"info\"".into())));
        assert_eq!(def.props[1], ("dismissible".into(), Some("false".into())));
        assert_eq!(def.props[2], ("title".into(), None));
    }

    #[test]
    fn compiles_props_content_and_attrs() {
        let registry = reg("alert", "{#def type=\"info\" #}");
        let out = compile(
            "<x-alert type=\"error\" class=\"mt-4\" id=\"a\">Boom</x-alert>",
            &registry,
        );
        assert!(out.contains("{% include \"components/alert/index.html.j2\" %}"));
        assert!(out.contains("type=\"error\""));
        // class/id are undeclared -> pass-through attrs.
        assert!(out.contains("\"class\": \"mt-4\""));
        assert!(out.contains("\"id\": \"a\""));
        assert!(out.contains("Boom"));
        assert!(out.contains("{% with "));
    }

    #[test]
    fn applies_default_for_missing_prop() {
        let registry = reg("alert", "{#def type=\"info\" #}");
        let out = compile("<x-alert>Hi</x-alert>", &registry);
        assert!(out.contains("type=\"info\""));
    }

    #[test]
    fn undefaulted_prop_still_enters_scope_so_it_cannot_read_the_caller() {
        let registry = reg("alert", "{#def title #}");
        let out = compile("<x-alert>Hi</x-alert>", &registry);

        // `title` is declared, was not passed, and has no default. It must still be bound in the
        // `with` block -- otherwise the name resolves against the caller's context and the
        // component renders the *page's* title as if it were its own.
        assert!(
            out.contains(&format!("title={UNDEFINED_PROP}")),
            "undeclared prop must shadow the caller: {out}"
        );
    }

    #[test]
    fn undefaulted_prop_renders_as_undefined_not_as_the_callers_value() -> Result<(), minijinja::Error>
    {
        let registry = reg("alert", "{#def title #}");
        let compiled = compile("<x-alert/>", &registry);

        let mut env = minijinja::Environment::new();
        env.set_undefined_behavior(minijinja::UndefinedBehavior::Chainable);
        env.add_template("components/alert/index.html.j2", "[{{ title }}]")?;
        env.add_template("page", &compiled)?;

        // The caller has a `title` of its own. The component must not see it.
        let out = env
            .get_template("page")?
            .render(minijinja::context! { title => "THE PAGE TITLE" })?;

        assert_eq!(out.trim(), "[]", "component leaked the caller's title: {out}");
        Ok(())
    }

    #[test]
    fn expression_props_pass_through_raw() {
        let registry = reg("card", "{#def title #}");
        let out = compile("<x-card :title=\"post.title\">x</x-card>", &registry);
        assert!(out.contains("title=post.title"));
    }

    #[test]
    fn self_closing_tag() {
        let registry = reg("icon", "{#def name #}");
        let out = compile("<x-icon name=\"star\" />", &registry);
        assert!(out.contains("{% include \"components/icon/index.html.j2\" %}"));
        assert!(out.contains("name=\"star\""));
    }

    #[test]
    fn named_slots_are_extracted() {
        let registry = reg("card", "{#def title #}");
        let out = compile(
            "<x-card title=\"T\">Body<x-slot name=\"footer\">F</x-slot></x-card>",
            &registry,
        );
        assert!(out.contains("\"footer\":"));
        assert!(out.contains("Body"));
        assert!(out.contains("F"));
        // slot markup is not left in the default content
        assert!(!out.contains("<x-slot"));
    }

    #[test]
    fn nested_components() {
        let mut defs = BTreeMap::new();
        defs.insert("outer".to_string(), parse_def("{#def #}"));
        defs.insert("inner".to_string(), parse_def("{#def #}"));
        let registry = ComponentRegistry { defs };
        let out = compile("<x-outer><x-inner>hi</x-inner></x-outer>", &registry);
        assert!(out.contains("components/outer/index.html.j2"));
        assert!(out.contains("components/inner/index.html.j2"));
        assert!(!out.contains("<x-"));
    }
}
