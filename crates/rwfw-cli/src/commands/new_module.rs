use anyhow::Context;
use std::fs;
use std::path::{Component, Path, PathBuf};

pub async fn run(name: &str) -> anyhow::Result<()> {
    let module = to_kebab(name);
    let crate_name = format!("mod-{module}");
    let crate_ident = crate_name.replace('-', "_");
    let pascal = to_pascal(&module);
    let title = to_title(&module);
    let app_dir = crate::commands::app::crate_dir();
    let modules_dir = crate::commands::app::modules_dir();
    let module_dir = modules_dir.join(&module);
    let framework_deps = framework_deps(&app_dir, &modules_dir, &module_dir)?;

    if module_dir.exists() {
        anyhow::bail!("Module already exists: {}", module_dir.display());
    }

    fs::create_dir_all(module_dir.join("src/routes"))?;
    fs::create_dir_all(module_dir.join("src/migrations"))?;
    fs::create_dir_all(module_dir.join("web/pages"))?;

    fs::write(
        module_dir.join("Cargo.toml"),
        cargo_toml(&crate_name, &framework_deps),
    )?;
    fs::write(
        module_dir.join("src/lib.rs"),
        lib_rs(&module, &pascal, &title),
    )?;
    fs::write(
        module_dir.join("src/migrations/mod.rs"),
        migrations_mod_rs(),
    )?;
    fs::write(
        module_dir.join("src/routes/index.rs"),
        index_route_rs(&module, &title),
    )?;
    fs::write(
        module_dir.join("web/pages/Index.tsx"),
        index_page_tsx(&title),
    )?;

    insert_once(
        Path::new("Cargo.toml"),
        "]\n\n[workspace.package]",
        &format!("    \"{}\",\n", path_for_toml(&module_dir)),
    )?;
    append_once(
        &app_dir.join("Cargo.toml"),
        &format!(
            "{crate_name} = {{ path = \"{}\" }}\n",
            path_for_toml(&relative_path(&app_dir, &module_dir))
        ),
    )?;
    append_once(
        &app_dir.join("src/lib.rs"),
        &format!("extern crate {crate_ident};\n"),
    )?;

    println!("Created module `{module}` at {}", module_dir.display());
    Ok(())
}

struct FrameworkDeps {
    core: String,
    shared: String,
    macros: String,
}

fn framework_deps(
    app_dir: &Path,
    modules_dir: &Path,
    module_dir: &Path,
) -> anyhow::Result<FrameworkDeps> {
    let manifests = dependency_manifest_candidates(app_dir, modules_dir, module_dir)?;

    Ok(FrameworkDeps {
        core: find_dependency_line(&manifests, "rwfw-core")?
            .unwrap_or_else(|| default_framework_dependency_line(module_dir, "rwfw-core")),
        shared: find_dependency_line(&manifests, "rwfw-shared")?
            .unwrap_or_else(|| default_framework_dependency_line(module_dir, "rwfw-shared")),
        macros: find_dependency_line(&manifests, "rwfw-macros")?
            .unwrap_or_else(|| default_framework_dependency_line(module_dir, "rwfw-macros")),
    })
}

fn dependency_manifest_candidates(
    app_dir: &Path,
    modules_dir: &Path,
    module_dir: &Path,
) -> anyhow::Result<Vec<PathBuf>> {
    let mut manifests = Vec::new();

    if modules_dir.exists() {
        let mut module_entries = fs::read_dir(modules_dir)?
            .filter_map(|entry| entry.ok())
            .map(|entry| entry.path())
            .filter(|path| path != module_dir)
            .collect::<Vec<_>>();
        module_entries.sort();

        for path in module_entries {
            let manifest = path.join("Cargo.toml");
            if manifest.exists() {
                manifests.push(manifest);
            }
        }
    }

    let app_manifest = app_dir.join("Cargo.toml");
    if app_manifest.exists() {
        manifests.push(app_manifest);
    }

    Ok(manifests)
}

fn find_dependency_line(manifests: &[PathBuf], crate_name: &str) -> anyhow::Result<Option<String>> {
    let prefix = format!("{crate_name} =");

    for manifest in manifests {
        let content = fs::read_to_string(manifest)
            .with_context(|| format!("reading {}", manifest.display()))?;
        if let Some(line) = content
            .lines()
            .find(|line| line.trim_start().starts_with(&prefix))
        {
            return Ok(Some(format!("{}\n", line.trim())));
        }
    }

    Ok(None)
}

fn default_framework_dependency_line(module_dir: &Path, crate_name: &str) -> String {
    let local_crate_dir = Path::new("crates").join(crate_name);
    if local_crate_dir.join("Cargo.toml").exists() {
        return format!(
            "{crate_name} = {{ path = \"{}\" }}\n",
            path_for_toml(&relative_path(module_dir, &local_crate_dir))
        );
    }

    format!("{crate_name} = \"{}\"\n", env!("CARGO_PKG_VERSION"))
}

fn cargo_toml(crate_name: &str, framework_deps: &FrameworkDeps) -> String {
    format!(
        r#"[package]
name = "{crate_name}"
version.workspace = true
edition.workspace = true

[dependencies]
{core}{shared}{macros}axum = {{ workspace = true }}
async-trait = {{ workspace = true }}
inventory = {{ workspace = true }}
serde_json = {{ workspace = true }}
tracing = {{ workspace = true }}
"#,
        core = framework_deps.core,
        shared = framework_deps.shared,
        macros = framework_deps.macros
    )
}

fn lib_rs(module: &str, pascal: &str, title: &str) -> String {
    format!(
        r#"pub mod migrations;

use axum::Router;
use rwfw_core::app::AppState;
use rwfw_core::module::{{Module, ModuleRegistration, NavItem}};

#[rwfw_macros::rwfw_routes("src/routes")]
pub struct {pascal}Routes;

pub struct {pascal}Module;

impl {pascal}Module {{
    pub fn new() -> Self {{
        Self
    }}
}}

#[async_trait::async_trait]
impl Module for {pascal}Module {{
    fn name(&self) -> &str {{
        "{module}"
    }}

    fn routes(&self) -> Router<AppState> {{
        {pascal}Routes::generated_routes()
    }}

    fn migrations(&self) -> Vec<rwfw_core::migration::Migration> {{
        migrations::migrations()
    }}

    fn permissions(&self) -> Vec<rwfw_core::auth::Permission> {{
        vec![
        ]
    }}

    fn nav_items(&self) -> Vec<NavItem> {{
        vec![NavItem {{
            label: "{title}".to_string(),
            href: "/{module}".to_string(),
            icon: None,
        }}]
    }}
}}

inventory::submit! {{
    ModuleRegistration::new("{module}", || Box::new({pascal}Module::new()))
}}
"#
    )
}

fn migrations_mod_rs() -> String {
    r#"use rwfw_core::migration::Migration;

pub fn migrations() -> Vec<Migration> {
    vec![
    ]
}
"#
    .to_string()
}

fn index_route_rs(module: &str, title: &str) -> String {
    format!(
        r#"use axum::response::IntoResponse;
use axum::routing;
use rwfw_core::app::AppState;
use rwfw_core::inertia::Inertia;

pub fn route() -> axum::routing::MethodRouter<AppState> {{
    routing::get(get)
}}

async fn get(i: Inertia) -> impl IntoResponse {{
    i.render_with_ssr(
        "{module}/Index",
        serde_json::json!({{
            "title": "{title}"
        }}),
    )
    .await
}}
"#
    )
}

fn index_page_tsx(title: &str) -> String {
    format!(
        r#"import AppLayout from '@app/layouts/AppLayout'

interface Props {{
  title: string
}}

export default function Index({{ title }}: Props) {{
  return (
    <AppLayout>
      <div className="max-w-4xl">
        <h1 className="text-4xl font-bold text-gray-900 mb-4">{{title ?? '{title}'}}</h1>
      </div>
    </AppLayout>
  )
}}
"#
    )
}

fn insert_once(path: &Path, marker: &str, line: &str) -> anyhow::Result<()> {
    let mut content =
        fs::read_to_string(path).with_context(|| format!("reading {}", path.display()))?;
    if content.contains(line) {
        return Ok(());
    }

    let index = content
        .find(marker)
        .with_context(|| format!("marker not found in {}", path.display()))?;
    content.insert_str(index, line);
    fs::write(path, content).with_context(|| format!("writing {}", path.display()))?;
    Ok(())
}

fn append_once(path: &Path, line: &str) -> anyhow::Result<()> {
    let mut content =
        fs::read_to_string(path).with_context(|| format!("reading {}", path.display()))?;
    if content.contains(line) {
        return Ok(());
    }

    if !content.ends_with('\n') {
        content.push('\n');
    }
    content.push_str(line);
    fs::write(path, content).with_context(|| format!("writing {}", path.display()))?;
    Ok(())
}

fn relative_path(from_dir: &Path, to_path: &Path) -> PathBuf {
    let from = normal_components(from_dir);
    let to = normal_components(to_path);

    let common = from
        .iter()
        .zip(to.iter())
        .take_while(|(left, right)| left == right)
        .count();

    let mut relative = PathBuf::new();
    for _ in common..from.len() {
        relative.push("..");
    }
    for component in &to[common..] {
        relative.push(component);
    }

    if relative.as_os_str().is_empty() {
        PathBuf::from(".")
    } else {
        relative
    }
}

fn normal_components(path: &Path) -> Vec<String> {
    path.components()
        .filter_map(|component| match component {
            Component::Normal(value) => Some(value.to_string_lossy().to_string()),
            Component::ParentDir => Some("..".to_string()),
            Component::CurDir => None,
            Component::RootDir | Component::Prefix(_) => {
                Some(component.as_os_str().to_string_lossy().to_string())
            }
        })
        .collect()
}

fn path_for_toml(path: &Path) -> String {
    path.to_string_lossy().replace('\\', "/")
}

fn to_kebab(value: &str) -> String {
    value
        .trim()
        .chars()
        .map(|c| {
            if c.is_ascii_alphanumeric() {
                c.to_ascii_lowercase()
            } else {
                '-'
            }
        })
        .collect::<String>()
        .split('-')
        .filter(|part| !part.is_empty())
        .collect::<Vec<_>>()
        .join("-")
}

fn to_pascal(value: &str) -> String {
    value
        .split('-')
        .filter(|part| !part.is_empty())
        .map(|part| {
            let mut chars = part.chars();
            match chars.next() {
                Some(first) => format!("{}{}", first.to_ascii_uppercase(), chars.as_str()),
                None => String::new(),
            }
        })
        .collect()
}

fn to_title(value: &str) -> String {
    value
        .split('-')
        .filter(|part| !part.is_empty())
        .map(|part| {
            let mut chars = part.chars();
            match chars.next() {
                Some(first) => format!("{}{}", first.to_ascii_uppercase(), chars.as_str()),
                None => String::new(),
            }
        })
        .collect::<Vec<_>>()
        .join(" ")
}
