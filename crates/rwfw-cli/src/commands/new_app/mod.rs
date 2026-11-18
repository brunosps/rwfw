use anyhow::Context;
use rwfw_core::config::STRICT_CONTENT_SECURITY_POLICY;
use std::fs;
use std::path::{Path, PathBuf};

mod blog;
mod docker;
mod scaffold;
mod shop;
use blog::*;
use docker::*;
use scaffold::*;
use shop::*;

#[derive(Debug, Clone, Copy, PartialEq, Eq, clap::ValueEnum)]
pub enum ExampleKind {
    Blog,
    Ecommerce,
}

impl ExampleKind {
    fn label(self) -> &'static str {
        match self {
            Self::Blog => "blog",
            Self::Ecommerce => "ecommerce",
        }
    }

    fn module_name(self) -> &'static str {
        match self {
            Self::Blog => "blog",
            Self::Ecommerce => "shop",
        }
    }

    fn module_crate_name(self) -> &'static str {
        match self {
            Self::Blog => "mod-blog",
            Self::Ecommerce => "mod-shop",
        }
    }

    fn module_crate_ident(self) -> &'static str {
        match self {
            Self::Blog => "mod_blog",
            Self::Ecommerce => "mod_shop",
        }
    }

    fn root_path(self) -> &'static str {
        match self {
            Self::Blog => "/blog",
            Self::Ecommerce => "/shop",
        }
    }

    fn readme_title(self) -> &'static str {
        match self {
            Self::Blog => "Blog CMS example",
            Self::Ecommerce => "Ecommerce storefront example",
        }
    }
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, clap::ValueEnum)]
pub enum DatabaseKind {
    Postgres,
    Sqlite,
}

impl DatabaseKind {
    /// Feature of rwfw-core / mod-auth / rwfw-tauri that compiles this driver.
    fn feature(self) -> &'static str {
        match self {
            Self::Postgres => "postgres",
            Self::Sqlite => "sqlite",
        }
    }

    /// sea-orm feature for this driver.
    fn sea_orm_feature(self) -> &'static str {
        match self {
            Self::Postgres => "sqlx-postgres",
            Self::Sqlite => "sqlx-sqlite",
        }
    }
}

pub fn run(
    name: &str,
    example: ExampleKind,
    database: DatabaseKind,
    tauri: bool,
    rwfw_path: Option<&Path>,
    rwfw_version: Option<&str>,
    rwfw_git: Option<&str>,
    rwfw_tag: Option<&str>,
) -> anyhow::Result<()> {
    let (app_name, app_dir) = resolve_target(name, &std::env::current_dir()?)?;
    ensure_target_is_usable(&app_dir)?;
    let in_place = app_dir == Path::new(".");
    let app_dir = app_dir.as_path();

    let dependency_source = DependencySource::resolve(rwfw_path, rwfw_version, rwfw_git, rwfw_tag)?;
    let context = AppTemplateContext::new(
        app_name.clone(),
        example,
        database,
        tauri,
        dependency_source,
    );

    create_dirs(app_dir)?;
    create_example_dirs(app_dir, example)?;
    write_file(&app_dir.join(".gitignore"), gitignore())?;
    write_file(&app_dir.join(".dockerignore"), dockerignore())?;
    write_file(&app_dir.join(".env.example"), env_example())?;
    write_file(&app_dir.join("Dockerfile"), dockerfile(&context))?;
    write_file(&app_dir.join("Dockerfile.prod"), dockerfile(&context))?;
    if matches!(context.database, DatabaseKind::Postgres) {
        write_file(&app_dir.join("compose.yaml"), compose_yaml(&context))?;
        write_file(
            &app_dir.join("compose.dev.yaml"),
            compose_dev_yaml(&context),
        )?;
    }
    write_file(
        &app_dir.join(".rwfw/templates/model.rs.tera"),
        scaffold_model_template(),
    )?;
    write_file(
        &app_dir.join(".rwfw/templates/migration.sql.tera"),
        scaffold_migration_template(),
    )?;
    write_file(
        &app_dir.join(".rwfw/templates/route_index.rs.tera"),
        scaffold_route_index_template(),
    )?;
    write_file(
        &app_dir.join(".rwfw/templates/page_index.html.j2.tera"),
        scaffold_page_index_template(),
    )?;
    write_file(
        &app_dir.join(".rwfw/templates/repository.rs.tera"),
        scaffold_repository_template(),
    )?;
    write_file(
        &app_dir.join(".rwfw/templates/use_case_create.rs.tera"),
        scaffold_use_case_create_template(),
    )?;
    write_file(
        &app_dir.join(".rwfw/templates/use_case_update.rs.tera"),
        scaffold_use_case_update_template(),
    )?;
    write_file(
        &app_dir.join(".rwfw/templates/use_case_delete.rs.tera"),
        scaffold_use_case_delete_template(),
    )?;
    write_file(
        &app_dir.join(".rwfw/templates/form.html.j2.tera"),
        scaffold_form_template(),
    )?;
    write_file(
        &app_dir.join(".rwfw/templates/page_create.html.j2.tera"),
        scaffold_page_create_template(),
    )?;
    write_file(
        &app_dir.join(".rwfw/templates/page_edit.html.j2.tera"),
        scaffold_page_edit_template(),
    )?;
    write_file(
        &app_dir.join(".rwfw/templates/page_show.html.j2.tera"),
        scaffold_page_show_template(),
    )?;
    write_file(
        &app_dir.join(".rwfw/templates/route_create.rs.tera"),
        scaffold_route_create_template(),
    )?;
    write_file(
        &app_dir.join(".rwfw/templates/route_edit.rs.tera"),
        scaffold_route_edit_template(),
    )?;
    write_file(
        &app_dir.join(".rwfw/templates/route_item.rs.tera"),
        scaffold_route_item_template(),
    )?;
    write_file(
        &app_dir.join("docker/entrypoint.sh"),
        docker_entrypoint_sh(),
    )?;
    write_file(&app_dir.join("README.md"), readme_md(&context))?;
    write_file(&app_dir.join("rwfw.toml"), rwfw_toml(&context))?;
    write_file(&app_dir.join("Cargo.toml"), workspace_cargo_toml(&context))?;
    write_file(
        &app_dir.join("config/development.yaml"),
        development_yaml(&context),
    )?;
    write_file(&app_dir.join("config/production.yaml"), production_yaml())?;

    write_file(
        &app_dir.join("crates/app/Cargo.toml"),
        app_cargo_toml(&context),
    )?;
    write_file(&app_dir.join("crates/app/src/lib.rs"), app_lib_rs(&context))?;
    write_file(
        &app_dir.join("crates/app/src/main.rs"),
        app_main_rs(&context),
    )?;
    // App-level web assets: Hotwire templates, vendored JS (Turbo + Stimulus),
    // compiled Tailwind CSS, and the `<x-...>` components used by the layouts.
    write_file(
        &app_dir.join("crates/app/web/templates/layouts/base.html.j2"),
        BASE_LAYOUT_TEMPLATE.to_string(),
    )?;
    write_file(
        &app_dir.join("crates/app/web/templates/layouts/app.html.j2"),
        APP_LAYOUT_TEMPLATE.to_string(),
    )?;
    write_file(
        &app_dir.join("crates/app/web/templates/layouts/auth.html.j2"),
        AUTH_LAYOUT_TEMPLATE.to_string(),
    )?;
    write_file(
        &app_dir.join("crates/app/web/templates/layouts/reading.html.j2"),
        READING_LAYOUT_TEMPLATE.to_string(),
    )?;
    write_file(
        &app_dir.join("crates/app/web/templates/components/flash/index.html.j2"),
        FLASH_COMPONENT_TEMPLATE.to_string(),
    )?;
    write_file(
        &app_dir.join("crates/app/web/templates/components/field/index.html.j2"),
        FIELD_COMPONENT_TEMPLATE.to_string(),
    )?;
    write_file(
        &app_dir.join("crates/app/web/templates/components_catalog.html.j2"),
        COMPONENTS_CATALOG_TEMPLATE.to_string(),
    )?;
    write_file(
        &app_dir.join("crates/app/web/vendor.lock"),
        VENDOR_LOCK.to_string(),
    )?;
    write_file(
        &app_dir.join("crates/app/web/assets/app.css"),
        APP_CSS.to_string(),
    )?;
    write_file(
        &app_dir.join("crates/app/web/assets/dev-livereload.js"),
        DEV_LIVERELOAD_JS.to_string(),
    )?;
    // The base layout loads /assets/boot.js, which imports /assets/reactive.js;
    // without either file Turbo/Stimulus never boot in generated apps.
    write_file(
        &app_dir.join("crates/app/web/assets/boot.js"),
        BOOT_JS.to_string(),
    )?;
    write_file(
        &app_dir.join("crates/app/web/assets/reactive.js"),
        REACTIVE_JS.to_string(),
    )?;
    write_binary_file(
        &app_dir.join("crates/app/web/vendor/turbo.min.js"),
        TURBO_JS,
    )?;
    write_binary_file(
        &app_dir.join("crates/app/web/vendor/stimulus.min.js"),
        STIMULUS_JS,
    )?;
    write_auth_templates(app_dir)?;

    write_file(
        &app_dir.join("crates/modules/home/Cargo.toml"),
        home_cargo_toml(&context),
    )?;
    write_file(
        &app_dir.join("crates/modules/home/src/lib.rs"),
        home_lib_rs(),
    )?;
    write_file(
        &app_dir.join("crates/modules/home/src/migrations/mod.rs"),
        migrations_mod_rs(),
    )?;
    write_file(
        &app_dir.join("crates/modules/home/src/routes/index.rs"),
        home_index_route_rs(),
    )?;
    write_file(
        &app_dir.join("crates/modules/home/web/templates/index.html.j2"),
        home_index_page_template().to_string(),
    )?;
    match example {
        ExampleKind::Blog => write_blog_example(app_dir, &context)?,
        ExampleKind::Ecommerce => write_ecommerce_example(app_dir, &context)?,
    }

    if context.tauri {
        write_tauri_shell(app_dir, &context)?;
    }

    println!(
        "Created RWFW {example} app `{app_name}` at {}",
        app_dir.display(),
        example = context.example.label()
    );
    let is_postgres = matches!(context.database, DatabaseKind::Postgres);
    println!("Next steps:");
    if !in_place {
        println!("  cd {app_name}");
    }
    if is_postgres {
        println!("  docker compose -f compose.dev.yaml up -d");
    }
    println!("  rwfw migrate");
    println!("  rwfw seed admin --email admin@example.com --password rwfw-admin-123");
    println!("  rwfw dev");
    println!(
        "  open http://localhost:3000{}",
        context.example.root_path()
    );
    if is_postgres {
        println!();
        println!("  # production-style run (Postgres + app in containers):");
        println!("  docker compose up --build");
    }
    println!();
    println!("If the CLI is not installed globally yet:");
    println!("  cargo run --manifest-path <rwfw-repo>/Cargo.toml -p rwfw-cli -- dev");

    Ok(())
}

#[derive(Debug, Clone)]
struct AppTemplateContext {
    app_name: String,
    app_crate_ident: String,
    app_title: String,
    database_name: String,
    docker_database_name: String,
    rwfw_core_dep: String,
    rwfw_shared_dep: String,
    rwfw_macros_dep: String,
    mod_auth_dep: String,
    rwfw_tauri_dep: String,
    example: ExampleKind,
    database: DatabaseKind,
    tauri: bool,
    dependency_source: DependencySource,
}

impl AppTemplateContext {
    fn new(
        app_name: String,
        example: ExampleKind,
        database: DatabaseKind,
        tauri: bool,
        dependency_source: DependencySource,
    ) -> Self {
        Self {
            app_name: app_name.clone(),
            app_crate_ident: app_name.replace('-', "_"),
            app_title: to_title(&app_name),
            database_name: format!("{}_dev", app_name.replace('-', "_")),
            docker_database_name: app_name.replace('-', "_"),
            // Only the chosen database driver is compiled in: default features are
            // off for every framework crate that carries a driver, otherwise Cargo
            // feature unification would bring the other one back.
            rwfw_core_dep: dependency_source
                .dependency_without_defaults("crates/rwfw-core", &[database.feature()]),
            rwfw_shared_dep: dependency_source.dependency("rwfw-shared"),
            rwfw_macros_dep: dependency_source.dependency("rwfw-macros"),
            mod_auth_dep: dependency_source
                .dependency_without_defaults("crates/modules/auth", &[database.feature(), "oidc"]),
            rwfw_tauri_dep: dependency_source
                .dependency_without_defaults("crates/rwfw-tauri", &["desktop", database.feature()]),
            example,
            database,
            tauri,
            dependency_source,
        }
    }
}

#[derive(Debug, Clone)]
enum DependencySource {
    Path(PathBuf),
    Version(String),
    Git { repository: String, tag: String },
}

impl DependencySource {
    fn resolve(
        rwfw_path: Option<&Path>,
        rwfw_version: Option<&str>,
        rwfw_git: Option<&str>,
        rwfw_tag: Option<&str>,
    ) -> anyhow::Result<Self> {
        let explicit_sources = [
            rwfw_path.is_some(),
            rwfw_version.is_some(),
            rwfw_git.is_some(),
        ]
        .into_iter()
        .filter(|is_set| *is_set)
        .count();
        if explicit_sources > 1 {
            anyhow::bail!("Use only one of --rwfw-path, --rwfw-version, or --rwfw-git");
        }

        if rwfw_git.is_none() && rwfw_tag.is_some() {
            anyhow::bail!("Use --rwfw-tag only together with --rwfw-git");
        }

        if let Some(path) = rwfw_path {
            return Ok(Self::Path(validate_framework_path(path)?));
        }

        if let Some(version) = rwfw_version {
            return Ok(Self::Version(version.to_string()));
        }

        if let Some(repository) = rwfw_git {
            if repository.trim().is_empty() {
                anyhow::bail!("--rwfw-git cannot be empty");
            }
            let tag = rwfw_tag
                .filter(|tag| !tag.trim().is_empty())
                .ok_or_else(|| anyhow::anyhow!("--rwfw-git requires --rwfw-tag"))?;
            return Ok(Self::Git {
                repository: repository.to_string(),
                tag: tag.to_string(),
            });
        }

        if let Ok(path) = std::env::var("RWFW_FRAMEWORK_PATH") {
            return Ok(Self::Path(validate_framework_path(Path::new(&path))?));
        }

        if let Some(path) = find_framework_root(&std::env::current_dir()?) {
            return Ok(Self::Path(validate_framework_path(&path)?));
        }

        if let Ok(repository) = std::env::var("RWFW_FRAMEWORK_GIT") {
            if repository.trim().is_empty() {
                anyhow::bail!("RWFW_FRAMEWORK_GIT cannot be empty");
            }
            let tag = std::env::var("RWFW_FRAMEWORK_TAG")
                .with_context(|| {
                    "RWFW_FRAMEWORK_GIT requires RWFW_FRAMEWORK_TAG for reproducible generated apps"
                })?
                .trim()
                .to_string();
            if tag.is_empty() {
                anyhow::bail!("RWFW_FRAMEWORK_TAG cannot be empty");
            }
            return Ok(Self::Git { repository, tag });
        }

        Ok(Self::Version(env!("CARGO_PKG_VERSION").to_string()))
    }

    fn dependency(&self, crate_name: &str) -> String {
        match self {
            Self::Path(root) => {
                let path = root.join("crates").join(crate_name);
                format!("{{ path = \"{}\" }}", path_for_toml(&path))
            }
            Self::Version(version) => format!("\"{version}\""),
            Self::Git { repository, tag } => git_dependency(repository, tag),
        }
    }

    /// Dependency on the framework crate at `crate_dir` (relative to the rwfw
    /// repo root) with default features off and exactly `features` on.
    fn dependency_without_defaults(&self, crate_dir: &str, features: &[&str]) -> String {
        let feats = features
            .iter()
            .map(|f| format!("\"{f}\""))
            .collect::<Vec<_>>()
            .join(", ");
        let source = match self {
            Self::Path(root) => format!("path = \"{}\"", path_for_toml(&root.join(crate_dir))),
            Self::Version(version) => format!("version = \"{version}\""),
            Self::Git { repository, tag } => format!("git = \"{repository}\", tag = \"{tag}\""),
        };
        format!("{{ {source}, default-features = false, features = [{feats}] }}")
    }
}

fn validate_framework_path(path: &Path) -> anyhow::Result<PathBuf> {
    let root = path
        .canonicalize()
        .with_context(|| format!("resolving RWFW framework path {}", path.display()))?;
    for crate_name in ["rwfw-core", "rwfw-shared", "rwfw-macros"] {
        let cargo_toml = root.join("crates").join(crate_name).join("Cargo.toml");
        if !cargo_toml.exists() {
            anyhow::bail!("RWFW framework path is missing {}", cargo_toml.display());
        }
    }
    let auth_cargo_toml = root.join("crates/modules/auth/Cargo.toml");
    if !auth_cargo_toml.exists() {
        anyhow::bail!(
            "RWFW framework path is missing {}",
            auth_cargo_toml.display()
        );
    }
    Ok(root)
}

fn find_framework_root(start: &Path) -> Option<PathBuf> {
    for dir in start.ancestors() {
        if dir.join("crates/rwfw-core/Cargo.toml").exists()
            && dir.join("crates/rwfw-cli/Cargo.toml").exists()
        {
            return Some(dir.to_path_buf());
        }
    }
    None
}

fn create_dirs(app_dir: &Path) -> anyhow::Result<()> {
    for dir in [
        "config",
        ".rwfw/templates",
        "docker",
        "crates/app/src",
        "crates/app/web/templates/layouts",
        "crates/app/web/templates/components/flash",
        "crates/app/web/templates/components/field",
        "crates/app/web/vendor",
        "crates/app/web/assets",
        "crates/modules/home/src/migrations",
        "crates/modules/home/src/routes",
        "crates/modules/home/web/templates",
        APP_AUTH_TEMPLATES_DIR,
        "data",
    ] {
        fs::create_dir_all(app_dir.join(dir))?;
    }
    Ok(())
}

fn create_example_dirs(app_dir: &Path, example: ExampleKind) -> anyhow::Result<()> {
    let dirs: &[&str] = match example {
        ExampleKind::Blog => &[
            "crates/modules/blog/src/migrations",
            "crates/modules/blog/src/models",
            "crates/modules/blog/src/repositories",
            "crates/modules/blog/src/use_cases",
            "crates/modules/blog/src/routes/posts/[slug]",
            "crates/modules/blog/src/routes/admin/posts/[id]",
            "crates/modules/blog/web/templates/posts",
            "crates/modules/blog/web/templates/admin/posts",
        ],
        ExampleKind::Ecommerce => &[
            "crates/modules/shop/src/migrations",
            "crates/modules/shop/src/models",
            "crates/modules/shop/src/repositories",
            "crates/modules/shop/src/use_cases",
            "crates/modules/shop/src/routes/products/[slug]",
            "crates/modules/shop/src/routes/cart",
            "crates/modules/shop/src/routes/checkout",
            "crates/modules/shop/src/routes/orders/[number]",
            "crates/modules/shop/src/routes/admin/products/[id]",
            "crates/modules/shop/src/routes/admin/orders",
            "crates/modules/shop/web/templates/products",
            "crates/modules/shop/web/templates/cart",
            "crates/modules/shop/web/templates/checkout",
            "crates/modules/shop/web/templates/orders",
            "crates/modules/shop/web/templates/admin/products",
            "crates/modules/shop/web/templates/admin/orders",
        ],
    };

    for dir in dirs {
        fs::create_dir_all(app_dir.join(dir))?;
    }

    Ok(())
}

fn write_file(path: &Path, content: String) -> anyhow::Result<()> {
    if path.exists() {
        anyhow::bail!("File already exists: {}", path.display());
    }
    if let Some(parent) = path.parent() {
        fs::create_dir_all(parent).with_context(|| format!("creating {}", parent.display()))?;
    }
    fs::write(path, content).with_context(|| format!("writing {}", path.display()))?;
    Ok(())
}

/// Where generated apps keep their copy of the auth pages.
///
/// The framework `mod-auth` crate serves templates from its own `web/` dir, so a
/// copy under a local `crates/modules/auth` (with no crate of its own) was never
/// read. The app template root is searched before every module root and matches
/// the `auth/` namespace, so files here override the framework pages.
const APP_AUTH_TEMPLATES_DIR: &str = "crates/app/web/templates/auth";

fn write_auth_templates(app_dir: &Path) -> anyhow::Result<()> {
    let dir = app_dir.join(APP_AUTH_TEMPLATES_DIR);
    write_file(&dir.join("login.html.j2"), AUTH_LOGIN_TEMPLATE.to_string())?;
    write_file(
        &dir.join("register.html.j2"),
        AUTH_REGISTER_TEMPLATE.to_string(),
    )?;
    Ok(())
}

fn write_binary_file(path: &Path, content: &[u8]) -> anyhow::Result<()> {
    if path.exists() {
        anyhow::bail!("File already exists: {}", path.display());
    }
    if let Some(parent) = path.parent() {
        fs::create_dir_all(parent).with_context(|| format!("creating {}", parent.display()))?;
    }
    fs::write(path, content).with_context(|| format!("writing {}", path.display()))?;
    Ok(())
}

// ---------------------------------------------------------------------------
// Embedded npm-free frontend assets (mirrors the dogfood `rwfw-app/web/*`).
// Baked into the CLI binary so generated apps need no npm/node and no network.
// ---------------------------------------------------------------------------

const TURBO_JS: &[u8] = include_bytes!("../../../../rwfw-app/web/vendor/turbo.min.js");
const STIMULUS_JS: &[u8] = include_bytes!("../../../../rwfw-app/web/vendor/stimulus.min.js");
const VENDOR_LOCK: &str = include_str!("../../../../rwfw-app/web/vendor.lock");
const APP_CSS: &str = include_str!("../../../../rwfw-app/web/assets/app.css");
const DEV_LIVERELOAD_JS: &str = include_str!("../../../../rwfw-app/web/assets/dev-livereload.js");
const BOOT_JS: &str = include_str!("../../../../rwfw-app/web/assets/boot.js");
const REACTIVE_JS: &str = include_str!("../../../../rwfw-app/web/assets/reactive.js");
const BASE_LAYOUT_TEMPLATE: &str =
    include_str!("../../../../rwfw-app/web/templates/layouts/base.html.j2");
const APP_LAYOUT_TEMPLATE: &str =
    include_str!("../../../../rwfw-app/web/templates/layouts/app.html.j2");
const AUTH_LAYOUT_TEMPLATE: &str =
    include_str!("../../../../rwfw-app/web/templates/layouts/auth.html.j2");
const READING_LAYOUT_TEMPLATE: &str =
    include_str!("../../../../rwfw-app/web/templates/layouts/reading.html.j2");
const FLASH_COMPONENT_TEMPLATE: &str =
    include_str!("../../../../rwfw-app/web/templates/components/flash/index.html.j2");
const FIELD_COMPONENT_TEMPLATE: &str =
    include_str!("../../../../rwfw-app/web/templates/components/field/index.html.j2");
const COMPONENTS_CATALOG_TEMPLATE: &str =
    include_str!("../../../../rwfw-app/web/templates/components_catalog.html.j2");
const AUTH_LOGIN_TEMPLATE: &str =
    include_str!("../../../../modules/auth/web/templates/login.html.j2");
const AUTH_REGISTER_TEMPLATE: &str =
    include_str!("../../../../modules/auth/web/templates/register.html.j2");

fn path_for_toml(path: &Path) -> String {
    path.to_string_lossy().replace('\\', "/")
}

fn git_dependency(repository: &str, tag: &str) -> String {
    format!(
        "{{ git = \"{}\", tag = \"{}\" }}",
        toml_string(repository),
        toml_string(tag)
    )
}

fn toml_string(value: &str) -> String {
    value.replace('\\', "\\\\").replace('"', "\\\"")
}

fn path_for_yaml(path: &Path) -> String {
    path.to_string_lossy()
        .replace('\\', "/")
        .replace('"', "\\\"")
}

fn readme_md(context: &AppTemplateContext) -> String {
    let db_label = if matches!(context.database, DatabaseKind::Postgres) {
        "PostgreSQL"
    } else {
        "SQLite (embedded, zero-infra)"
    };
    let project_db_files = readme_project_db_files(context);
    let local_dev_section = readme_local_dev_section(context);
    let production_section = readme_production_section(context);
    format!(
        r#"# {title}

Generated by RWFW. This project is a runnable example application and a starting point for building modular Rust web apps with Axum, SeaORM, Hotwire (Turbo + Stimulus), MiniJinja templates, and {db_label} — with zero npm/node.

Example selected: `{example}` ({example_title}).

## Framework Dependency Source

{dependency_source}

## What Is In This Project

- `rwfw.toml`: RWFW project metadata used by the CLI to find the app crate and modules directory.
- `Cargo.toml`: Rust workspace with the app crate and local modules.
- `.env.example`: local environment variables for generated integrations such as SSO providers.
- `config/development.yaml`: local runtime config used by `rwfw dev`.
- `config/production.yaml`: production runtime config used by the Docker image.
{project_db_files}
- `Dockerfile.prod`: single-stage Rust production image (no node); web assets are loaded from disk.
- `docker/entrypoint.sh`: production container entrypoint; optionally runs migrations before starting the app.
- `.rwfw/templates`: project-local code generation templates used by `rwfw generate`.
- `crates/app`: application composition crate. It wires modules, config, and migrations into one binary, and owns the shared `web/` (Hotwire layouts, vendored Turbo/Stimulus, `assets/app.css`).
- `crates/modules/home`: minimal example module with one page.
- `crates/modules/{example_module}`: generated `{example}` example module.
- `crates/app/web/templates/auth`: login and register pages for the framework auth module. The app template root is searched before module templates, so edits here replace the framework pages.

{local_dev_section}

## SSO / OIDC

Add a market SSO provider to this app:

```bash
rwfw auth sso add keycloak \
  --provider keycloak \
  --issuer-url http://localhost:8081/realms/rwfw \
  --client-id-env KEYCLOAK_CLIENT_ID \
  --client-secret-env KEYCLOAK_CLIENT_SECRET
```

The command updates `config/development.yaml`, `config/production.yaml`, and `.env.example`.
Set the generated env vars before starting the app. The redirect URI is:

```text
http://localhost:3000/auth/sso/keycloak/callback
```

Use `--provider azure-b2c` for Azure AD B2C or `--provider oidc` for any standards-compliant OpenID Connect provider.

{production_section}

## Example Routes

{example_routes}

## Migrations

- Module migrations live under `crates/modules/<module>/src/migrations`.
- Each migration should be one SQL file registered from `mod.rs` with `include_str!`.
- RWFW executes migration SQL through the SeaORM database connection.
- The central migration ledger is `rwfw.migrations`.

## Code Generation

Create a module:

```bash
rwfw new module admin
```

Generate a model and migration:

```bash
rwfw generate model Product --module admin --fields "name:string description:text active:bool price:decimal published_at:datetime"
```

Generate a CRUD scaffold slice:

```bash
rwfw generate scaffold Product --module admin name:string description:text active:bool price:decimal available_on:date published_at:datetime
```

Supported field types: `string`, `text`, `int`, `integer`, `decimal`, `float`, `bool`, `boolean`, `uuid`, `date`, `datetime`, and `timestamp`.
Append `?` to the field name or type for nullable columns, for example `summary?:text` or `published_at:datetime?`.

The scaffold creates a SeaORM model, one SQL migration file, repository/use-case files, protected CRUD routes, searchable/paginated index pages, flash confirmations, and reusable MiniJinja index/show/form/create/edit templates.

## Scaffold Templates

The generator reads templates from `.rwfw/templates` before using built-in defaults. Edit these files to customize generated code for this project:

- `model.rs.tera`: SeaORM model.
- `migration.sql.tera`: SQL migration.
- `repository.rs.tera`: SeaORM repository.
- `use_case_create.rs.tera`: create input, validation, and use-case.
- `use_case_update.rs.tera`: update input, validation, and use-case.
- `use_case_delete.rs.tera`: delete use-case.
- `route_index.rs.tera`: Axum route using the `View` engine.
- `route_create.rs.tera`: protected create page route.
- `route_edit.rs.tera`: protected edit/update route.
- `route_item.rs.tera`: protected show and item actions, including delete.
- `page_index.html.j2.tera`: MiniJinja index page.
- `form.html.j2.tera`: reusable MiniJinja form partial.
- `page_create.html.j2.tera`: MiniJinja create page.
- `page_edit.html.j2.tera`: MiniJinja edit page.
- `page_show.html.j2.tera`: MiniJinja show page.

Template context:

- `module`: module name, for example `blog`.
- `table`: plural table/resource name, for example `posts`.
- `name_snake`: singular snake_case name, for example `post`.
- `name_pascal`: singular PascalCase name, for example `Post`.
- `migration_version`: generated migration version.
- `fields`: field list with `name`, `kind`, `rust_type`, `sql_type`, `input_type`, `input_step`, `has_input_step`, `input_rust_type`, `owned_rust_type`, `title`, `column_variant`, `optional`, `is_text`, `is_bool`, `is_date`, `is_datetime`, `is_decimal`, `is_number`, `is_uuid`, and `is_searchable`.
- `fields_len`: number of fields, with minimum value `1` for empty table states.
- `fields_colspan`: number of fields plus the action column, with minimum value `2`.
- `has_number`, `has_decimal`, `has_uuid`, `has_date`, `has_datetime`, `has_optional`, `has_optional_number`, `has_optional_decimal`, `has_optional_uuid`, `has_optional_date`, `has_optional_datetime`, and `has_searchable`: convenience booleans for conditional helpers.

After editing templates, run `rwfw generate ...` again for new resources. Existing generated files are not overwritten.
"#,
        title = context.app_title,
        example = context.example.label(),
        example_title = context.example.readme_title(),
        dependency_source = readme_dependency_source(&context.dependency_source),
        example_module = context.example.module_name(),
        example_routes = readme_example_routes(context.example),
    )
}

fn readme_project_db_files(context: &AppTemplateContext) -> String {
    if matches!(context.database, DatabaseKind::Postgres) {
        "- `compose.dev.yaml`: local development services: PostgreSQL and pgAdmin.\n\
         - `compose.yaml`: production-style Docker stack: PostgreSQL plus the compiled app container."
            .to_string()
    } else {
        format!(
            "- `data/{}.db`: embedded SQLite database file, created by `rwfw migrate` (no external DB service or Docker needed for local dev).",
            context.database_name
        )
    }
}

fn readme_local_dev_section(context: &AppTemplateContext) -> String {
    if matches!(context.database, DatabaseKind::Postgres) {
        format!(
            r#"## Local Development

```bash
docker compose -f compose.dev.yaml up -d
rwfw migrate
rwfw seed admin --email admin@example.com --password rwfw-admin-123
rwfw dev
```

`rwfw dev` runs the app locally in Cargo's `dev` profile. Templates and CSS
live reload in the browser; Rust code auto-restarts when `cargo-watch` is
installed and falls back to plain `cargo run` otherwise.

The dev database is exposed on `localhost:54329` by default:

```text
postgres://rwfw:rwfw@localhost:54329/{db}
```

pgAdmin is available at http://localhost:5053 with:

```text
email: admin@example.com
password: rwfw
```

Use these variables to avoid port conflicts:

```bash
RWFW_DEV_DB_PORT=54332 RWFW_PGADMIN_PORT=5055 docker compose -f compose.dev.yaml up -d
```"#,
            db = context.database_name
        )
    } else {
        format!(
            r#"## Local Development

```bash
rwfw migrate
rwfw seed admin --email admin@example.com --password rwfw-admin-123
rwfw dev
```

`rwfw dev` runs the app locally in Cargo's `dev` profile. Templates and CSS
live reload in the browser; Rust code auto-restarts when `cargo-watch` is
installed and falls back to plain `cargo run` otherwise.

This app uses an embedded SQLite database at `data/{db}.db`, created on the first `rwfw migrate`. No external database service or Docker is required for local development."#,
            db = context.database_name
        )
    }
}

fn readme_production_section(context: &AppTemplateContext) -> String {
    let root = context.example.root_path();
    if matches!(context.database, DatabaseKind::Postgres) {
        format!(
            r#"## Production Docker

```bash
docker compose up --build
```

The production compose stack builds `Dockerfile.prod` and listens on http://localhost:8080{root}.

Migrations run automatically when the app container starts. Set `RWFW_RUN_MIGRATIONS=0` to disable that behavior."#
        )
    } else {
        format!(
            r#"## Production Docker

```bash
docker build -f Dockerfile.prod -t {app} .
docker run -p 8080:8080 -v "$(pwd)/data:/app/data" {app}
```

The image bundles the app and serves on http://localhost:8080{root}; the SQLite file persists in the mounted `data/` volume.

Migrations run automatically when the container starts. Set `RWFW_RUN_MIGRATIONS=0` to disable that behavior."#,
            app = context.app_name
        )
    }
}

fn readme_dependency_source(source: &DependencySource) -> String {
    match source {
        DependencySource::Path(root) => format!(
            "This app depends on a local RWFW checkout at `{}`. This is the recommended setup while developing the framework itself. Production Docker builds include this checkout as an extra build context.",
            path_for_toml(root)
        ),
        DependencySource::Git { repository, tag } => format!(
            "This app depends on RWFW crates from Git repository `{repository}` pinned to tag `{tag}`. This is the recommended alpha distribution mode before crates.io publishing."
        ),
        DependencySource::Version(version) => format!(
            "This app depends on published RWFW crates at version `{version}`. Use this once RWFW packages are published and the API is stable enough for versioned releases."
        ),
    }
}

fn readme_example_routes(example: ExampleKind) -> &'static str {
    match example {
        ExampleKind::Blog => {
            r#"- `/blog` is the editorial landing page.
- `/blog/posts` lists published posts.
- `/blog/posts/:slug` shows a public post.
- `/blog/admin/posts` is protected and manages drafts, publishing, editing, and deletion.
- `/auth/register` creates a user and signs in. The first user becomes admin."#
        }
        ExampleKind::Ecommerce => {
            r#"- `/shop` is the storefront with category and search filters.
- `/shop/products/:slug` shows a product detail page with an add-to-cart form.
- `/shop/cart` is the server-rendered cart (backed by a cookie, no client JS state).
- `/shop/checkout` renders the checkout form and posts a fake checkout to the backend.
- `/shop/orders/:number` shows the persisted order confirmation.
- `/auth/register` is included so you can test protected framework routes later, but storefront browsing and checkout are public in this example."#
        }
    }
}

fn rwfw_toml(context: &AppTemplateContext) -> String {
    format!(
        r#"app_package = "{}"
app_crate_dir = "crates/app"
modules_dir = "crates/modules"
"#,
        context.app_name
    )
}

fn workspace_cargo_toml(context: &AppTemplateContext) -> String {
    let tauri_member = if context.tauri {
        "\n    \"src-tauri\","
    } else {
        ""
    };
    r#"[workspace]
resolver = "2"
members = [
    "crates/app",
    "crates/modules/home",
    "crates/modules/__EXAMPLE_MODULE__",__TAURI_MEMBER__
]

[workspace.package]
version = "0.1.0"
edition = "2024"
license = "MIT"

[workspace.dependencies]
axum = { version = "0.8", features = ["macros"] }
axum-extra = { version = "0.10", features = ["cookie", "typed-header"] }
tokio = { version = "1", features = ["full"] }
tower = "0.5"
tower-http = { version = "0.6", features = ["fs", "trace", "cors"] }
tower-sessions = "0.14"
serde = { version = "1", features = ["derive"] }
serde_json = "1"
sea-orm = { version = "1", features = ["__SEA_ORM_DRIVER__", "runtime-tokio-rustls", "macros"] }
async-trait = "0.1"
inventory = "0.3"
tracing = "0.1"
tracing-subscriber = { version = "0.3", features = ["json", "env-filter"] }
dotenvy = "0.15"
config = "0.14"
anyhow = "1"
thiserror = "2"
chrono = { version = "0.4", features = ["serde"] }
uuid = { version = "1", features = ["v4", "serde"] }
"#
    .replace("__EXAMPLE_MODULE__", context.example.module_name())
    .replace("__SEA_ORM_DRIVER__", context.database.sea_orm_feature())
    .replace("__TAURI_MEMBER__", tauri_member)
}

fn dev_database_url(context: &AppTemplateContext) -> String {
    match context.database {
        DatabaseKind::Postgres => format!(
            "postgres://rwfw:rwfw@localhost:54329/{}",
            context.database_name
        ),
        DatabaseKind::Sqlite => format!("sqlite://data/{}.db?mode=rwc", context.database_name),
    }
}

fn development_yaml(context: &AppTemplateContext) -> String {
    format!(
        r#"database:
  url: "{db_url}"

server:
  host: "0.0.0.0"
  port: 3000

logging:
  level: "debug"
  format: "pretty"

security:
  headers_enabled: true
  hsts: false
  frame_options: "SAMEORIGIN"
  referrer_policy: "strict-origin-when-cross-origin"
  # The generated layout loads every script from a file, so scripts need no
  # 'unsafe-inline'. Keep it that way: an inline <script> will be blocked.
  content_security_policy: "{csp}"
  rate_limit:
    enabled: false
    max_requests: 10
    window_secs: 60

auth:
  session_ttl: 86400
  oidc:
    redirect_base_url: "http://localhost:3000"
    providers: {{}}

app:
  title: "{app_title}"
"#,
        db_url = dev_database_url(context),
        app_title = context.app_title,
        csp = STRICT_CONTENT_SECURITY_POLICY,
    )
}

fn create_tauri_dirs(app_dir: &Path) -> anyhow::Result<()> {
    fs::create_dir_all(app_dir.join("src-tauri/src"))?;
    Ok(())
}

/// Emit a `src-tauri/` desktop shell: a `<app>-desktop` crate that boots the
/// app's `build_router` in-process (via `rwfw-tauri`) and opens a Tauri window.
fn write_tauri_shell(app_dir: &Path, context: &AppTemplateContext) -> anyhow::Result<()> {
    create_tauri_dirs(app_dir)?;
    write_file(
        &app_dir.join("src-tauri/Cargo.toml"),
        tauri_cargo_toml(context),
    )?;
    write_file(&app_dir.join("src-tauri/build.rs"), tauri_build_rs())?;
    write_file(
        &app_dir.join("src-tauri/src/main.rs"),
        tauri_main_rs(context),
    )?;
    write_file(
        &app_dir.join("src-tauri/tauri.conf.json"),
        tauri_conf_json(context),
    )?;
    Ok(())
}

fn tauri_cargo_toml(context: &AppTemplateContext) -> String {
    format!(
        r#"[package]
name = "{app_name}-desktop"
version.workspace = true
edition.workspace = true

[[bin]]
name = "{app_name}-desktop"
path = "src/main.rs"

[build-dependencies]
tauri-build = {{ version = "2", features = [] }}

[dependencies]
{app_name} = {{ path = "../crates/app" }}
rwfw-core = {core}
rwfw-tauri = {tauri}
tauri = {{ version = "2", features = [] }}
tokio = {{ workspace = true }}
anyhow = {{ workspace = true }}
"#,
        app_name = context.app_name,
        core = context.rwfw_core_dep,
        tauri = context.rwfw_tauri_dep,
    )
}

fn tauri_build_rs() -> String {
    "fn main() {\n    tauri_build::build();\n}\n".to_string()
}

fn tauri_main_rs(context: &AppTemplateContext) -> String {
    r#"// Desktop shell for the RWFW app. Boots `build_router` in-process on
// 127.0.0.1:0 with graceful shutdown (owned by rwfw-tauri) and opens a native
// window pointed at the ephemeral local address.
#![cfg_attr(
    all(not(debug_assertions), target_os = "windows"),
    windows_subsystem = "windows"
)]

use rwfw_core::config::AppConfig;

fn main() -> anyhow::Result<()> {
    let config = AppConfig::load()?;
    rwfw_core::logging::init_default();

    // Apply migrations (creates the SQLite file on first run) before the window.
    tauri::async_runtime::block_on(__APP_CRATE_IDENT__::run_migrations(&config))?;

    tauri::Builder::default()
        .setup(move |app| {
            rwfw_tauri::desktop::attach(app, config, __APP_CRATE_IDENT__::build_router)
                .map_err(|e| e.to_string())?;
            Ok(())
        })
        .run(tauri::generate_context!())
        .map_err(|e| anyhow::anyhow!("tauri error: {e}"))?;
    Ok(())
}
"#
    .replace("__APP_CRATE_IDENT__", &context.app_crate_ident)
}

fn tauri_conf_json(context: &AppTemplateContext) -> String {
    format!(
        r#"{{
  "$schema": "https://schema.tauri.app/config/2",
  "productName": "{title}",
  "version": "0.1.0",
  "identifier": "com.rwfw.{ident}",
  "build": {{
    "beforeDevCommand": "",
    "beforeBuildCommand": ""
  }},
  "app": {{
    "windows": [
      {{
        "title": "{title}",
        "width": 1200,
        "height": 800,
        "resizable": true,
        "fullscreen": false
      }}
    ],
    "security": {{
      "csp": null
    }}
  }},
  "bundle": {{
    "active": false
  }}
}}
"#,
        title = context.app_title,
        ident = context.app_crate_ident,
    )
}

fn production_yaml() -> String {
    format!(
        r#"database:
  url: "${{RWFW__DATABASE__URL}}"

server:
  host: "0.0.0.0"
  port: 8080

logging:
  level: "info"
  format: "json"

security:
  headers_enabled: true
  hsts: true
  frame_options: "SAMEORIGIN"
  referrer_policy: "strict-origin-when-cross-origin"
  # The generated layout loads every script from a file, so scripts need no
  # 'unsafe-inline'. Keep it that way: an inline <script> will be blocked.
  content_security_policy: "{csp}"
  rate_limit:
    enabled: false
    max_requests: 10
    window_secs: 60

auth:
  session_ttl: 86400
  oidc:
    redirect_base_url: "http://localhost:8080"
    providers: {{}}
"#,
        csp = STRICT_CONTENT_SECURITY_POLICY,
    )
}

fn app_cargo_toml(context: &AppTemplateContext) -> String {
    format!(
        r#"[package]
name = "{}"
version.workspace = true
edition.workspace = true

[[bin]]
name = "{}"
path = "src/main.rs"

[dependencies]
rwfw-core = {}
rwfw-shared = {}
mod-auth = {}
mod-home = {{ path = "../modules/home" }}
{} = {{ path = "../modules/{}" }}
tokio = {{ workspace = true }}
tracing = {{ workspace = true }}
inventory = {{ workspace = true }}
axum = {{ workspace = true }}
anyhow = {{ workspace = true }}
"#,
        context.app_name,
        context.app_name,
        context.rwfw_core_dep,
        context.rwfw_shared_dep,
        context.mod_auth_dep,
        context.example.module_crate_name(),
        context.example.module_name()
    )
}

fn home_cargo_toml(context: &AppTemplateContext) -> String {
    format!(
        r#"[package]
name = "mod-home"
version.workspace = true
edition.workspace = true

[dependencies]
rwfw-core = {}
rwfw-shared = {}
rwfw-macros = {}
axum = {{ workspace = true }}
async-trait = {{ workspace = true }}
inventory = {{ workspace = true }}
serde_json = {{ workspace = true }}
tracing = {{ workspace = true }}
"#,
        context.rwfw_core_dep, context.rwfw_shared_dep, context.rwfw_macros_dep
    )
}

/// App name and target directory for `rwfw new app <name>`. `.` generates into
/// the current directory and takes the name from its folder; any other name
/// becomes `./<kebab-name>`.
fn resolve_target(name: &str, cwd: &Path) -> anyhow::Result<(String, PathBuf)> {
    let (app_name, app_dir) = if name.trim() == "." {
        let folder = cwd
            .file_name()
            .map(|folder| folder.to_string_lossy().into_owned())
            .unwrap_or_default();
        (to_kebab(&folder), PathBuf::from("."))
    } else {
        let app_name = to_kebab(name);
        let app_dir = PathBuf::from(&app_name);
        (app_name, app_dir)
    };
    if app_name.is_empty() {
        anyhow::bail!("App name must contain at least one ASCII letter or number");
    }
    Ok((app_name, app_dir))
}

/// The target may not exist yet, or be an empty directory (a fresh `git init`
/// or clone with only `.git` counts as empty). Anything else is refused up
/// front so a half-written app never mixes with existing files.
fn ensure_target_is_usable(dir: &Path) -> anyhow::Result<()> {
    if !dir.exists() {
        return Ok(());
    }
    if !dir.is_dir() {
        anyhow::bail!("{} exists and is not a directory", dir.display());
    }
    let mut others = fs::read_dir(dir)
        .with_context(|| format!("reading {}", dir.display()))?
        .filter_map(|entry| entry.ok())
        .map(|entry| entry.file_name().to_string_lossy().into_owned())
        .filter(|name| name != ".git")
        .collect::<Vec<_>>();
    if !others.is_empty() {
        others.sort();
        others.truncate(5);
        anyhow::bail!(
            "App directory {} is not empty (found: {}). Use a new or empty directory; \
             only a .git folder may already be there.",
            dir.display(),
            others.join(", ")
        );
    }
    Ok(())
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

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn generated_layout_ships_the_reactive_runtime_it_imports() {
        // The base layout loads /assets/boot.js, which imports reactive.js and
        // registers it — if the scaffold ever stops writing either asset, every
        // generated app loses Stimulus entirely.
        assert!(
            BASE_LAYOUT_TEMPLATE
                .contains(r#"<script type="module" src="/assets/boot.js"></script>"#),
            "base layout must load the boot module from a file"
        );
        assert!(
            BOOT_JS.contains(r#"from "/assets/reactive.js""#),
            "boot.js must import the reactive runtime"
        );
        assert!(
            BOOT_JS.contains(r#"register("reactive""#),
            "boot.js must register the reactive Stimulus controller"
        );
        assert!(
            REACTIVE_JS.contains("Controller"),
            "embedded reactive.js must be the Stimulus controller source"
        );
    }

    /// Every `<script>` start tag in `html` (lowercased).
    fn script_tags(html: &str) -> Vec<String> {
        let lower = html.to_ascii_lowercase();
        lower
            .match_indices("<script")
            .map(|(start, _)| {
                let end = lower[start..]
                    .find('>')
                    .map_or(lower.len(), |i| start + i + 1);
                lower[start..end].to_string()
            })
            .collect()
    }

    #[test]
    fn generated_layouts_have_no_inline_scripts() {
        // The generated config ships a CSP without 'unsafe-inline' for scripts,
        // so an inline <script> (or import map) in a layout would be blocked.
        for (name, template) in [
            ("base", BASE_LAYOUT_TEMPLATE),
            ("app", APP_LAYOUT_TEMPLATE),
            ("auth", AUTH_LAYOUT_TEMPLATE),
            ("reading", READING_LAYOUT_TEMPLATE),
        ] {
            for tag in script_tags(template) {
                assert!(
                    tag.contains(" src="),
                    "layouts/{name}.html.j2 has an inline script: {tag}"
                );
            }
            assert!(
                !template.contains("importmap"),
                "layouts/{name}.html.j2 has an inline import map"
            );
        }
        for (name, js) in [("boot.js", BOOT_JS), ("reactive.js", REACTIVE_JS)] {
            assert!(
                !js.contains("from \"@hotwired/"),
                "{name} uses a bare specifier, which needs an import map"
            );
        }
    }

    #[test]
    fn generated_config_uses_a_csp_without_inline_scripts() {
        for yaml in [development_yaml(&test_context()), production_yaml()] {
            let value: serde_yaml::Value = serde_yaml::from_str(&yaml).unwrap();
            let csp = value["security"]["content_security_policy"]
                .as_str()
                .expect("generated config must set security.content_security_policy");
            assert_eq!(csp, STRICT_CONTENT_SECURITY_POLICY);
        }
        // format! must keep the env placeholder literal.
        assert!(production_yaml().contains(r#"url: "${RWFW__DATABASE__URL}""#));
    }

    #[test]
    fn generated_dockerfile_does_not_install_openssl() {
        // rwfw-core uses rustls; the binary links no libssl.
        let dockerfile = dockerfile(&test_context());
        assert!(!dockerfile.contains("libssl"), "{dockerfile}");
    }

    fn scratch_dir(label: &str) -> PathBuf {
        let dir = std::env::temp_dir().join(format!(
            "rwfw-new-app-{label}-{}-{}",
            std::process::id(),
            std::time::SystemTime::now()
                .duration_since(std::time::UNIX_EPOCH)
                .unwrap()
                .as_nanos()
        ));
        fs::create_dir_all(&dir).unwrap();
        dir
    }

    #[test]
    fn dot_takes_the_app_name_from_the_current_folder() {
        let (name, dir) = resolve_target(".", Path::new("/home/me/My Shop_2")).unwrap();
        assert_eq!(name, "my-shop-2");
        assert_eq!(dir, PathBuf::from("."));

        let (name, dir) = resolve_target("Demo App", Path::new("/tmp")).unwrap();
        assert_eq!(name, "demo-app");
        assert_eq!(dir, PathBuf::from("demo-app"));

        assert!(resolve_target(".", Path::new("/")).is_err());
    }

    #[test]
    fn accepts_missing_empty_or_git_only_directories() {
        let root = scratch_dir("ok");
        ensure_target_is_usable(&root.join("missing")).unwrap();
        ensure_target_is_usable(&root).unwrap();
        fs::create_dir_all(root.join(".git/objects")).unwrap();
        ensure_target_is_usable(&root).unwrap();
        fs::remove_dir_all(&root).unwrap();
    }

    #[test]
    fn refuses_directories_with_files() {
        let root = scratch_dir("busy");
        fs::create_dir_all(root.join(".git")).unwrap();
        fs::write(root.join("README.md"), "hi").unwrap();
        let error = ensure_target_is_usable(&root).unwrap_err().to_string();
        fs::remove_dir_all(&root).unwrap();
        assert!(error.contains("is not empty"), "{error}");
        assert!(error.contains("README.md"), "{error}");
    }

    fn test_context() -> AppTemplateContext {
        context_for(DatabaseKind::Sqlite)
    }

    fn context_for(database: DatabaseKind) -> AppTemplateContext {
        AppTemplateContext::new(
            "demo-app".to_string(),
            ExampleKind::Blog,
            database,
            false,
            DependencySource::Git {
                repository: "https://github.com/brunosps/rwfw".to_string(),
                tag: "v0.0.0".to_string(),
            },
        )
    }

    #[test]
    fn generated_app_compiles_only_the_chosen_database_driver() {
        for (database, wanted, unwanted) in [
            (DatabaseKind::Postgres, "postgres", "sqlite"),
            (DatabaseKind::Sqlite, "sqlite", "postgres"),
        ] {
            let context = context_for(database);
            // Every framework crate that carries a driver must turn default
            // features off, or Cargo feature unification re-enables both.
            for dep in [
                &context.rwfw_core_dep,
                &context.mod_auth_dep,
                &context.rwfw_tauri_dep,
            ] {
                assert!(dep.contains("default-features = false"), "{dep}");
                assert!(dep.contains(&format!("\"{wanted}\"")), "{dep}");
                assert!(!dep.contains(&format!("\"{unwanted}\"")), "{dep}");
            }
            assert!(context.mod_auth_dep.contains("\"oidc\""));

            let workspace = workspace_cargo_toml(&context);
            let sea_orm = workspace
                .lines()
                .find(|line| line.starts_with("sea-orm ="))
                .unwrap();
            assert!(sea_orm.contains(&format!("sqlx-{wanted}")), "{sea_orm}");
            assert!(!sea_orm.contains(&format!("sqlx-{unwanted}")), "{sea_orm}");
        }
    }

    /// `key = "value"` from the `[section]` table of a vendor.lock.
    fn lock_value<'a>(lock: &'a str, section: &str, key: &str) -> &'a str {
        let header = format!("[{section}]");
        let body = lock
            .split_once(&header)
            .unwrap_or_else(|| panic!("vendor.lock has no {header}"))
            .1;
        let body = body.split("\n[").next().unwrap();
        body.lines()
            .find_map(|line| {
                let (k, v) = line.split_once('=')?;
                (k.trim() == key).then(|| v.trim().trim_matches('"'))
            })
            .unwrap_or_else(|| panic!("vendor.lock {header} has no {key}"))
    }

    #[test]
    fn layout_integrity_matches_vendor_lock_and_vendored_files() {
        use base64::Engine;
        use sha2::{Digest, Sha384};

        for (name, bytes) in [("turbo", TURBO_JS), ("stimulus", STIMULUS_JS)] {
            let file = lock_value(VENDOR_LOCK, name, "file");
            let locked = lock_value(VENDOR_LOCK, name, "integrity");

            let actual = format!(
                "sha384-{}",
                base64::engine::general_purpose::STANDARD.encode(Sha384::digest(bytes))
            );
            assert_eq!(locked, actual, "vendor.lock integrity for {file} is stale");

            let preload =
                format!(r#"<link rel="modulepreload" href="/{file}" integrity="{locked}">"#);
            assert!(
                BASE_LAYOUT_TEMPLATE.contains(&preload),
                "base layout must preload /{file} with the vendor.lock integrity"
            );
        }
    }

    #[test]
    fn auth_templates_land_in_the_app_template_root() {
        // The app root has no namespace, so `auth/login.html.j2` resolves there
        // before reaching mod-auth's own web root. A copy anywhere else is dead.
        let app_dir = std::env::temp_dir().join(format!(
            "rwfw-auth-templates-{}-{}",
            std::process::id(),
            std::time::SystemTime::now()
                .duration_since(std::time::UNIX_EPOCH)
                .unwrap()
                .as_nanos()
        ));
        write_auth_templates(&app_dir).unwrap();

        let auth_dir = app_dir.join("crates/app/web/templates/auth");
        let login = fs::read_to_string(auth_dir.join("login.html.j2")).unwrap();
        let register = fs::read_to_string(auth_dir.join("register.html.j2")).unwrap();
        let stray_module_dir_exists = app_dir.join("crates/modules/auth").exists();
        fs::remove_dir_all(&app_dir).unwrap();

        assert_eq!(login, AUTH_LOGIN_TEMPLATE);
        assert_eq!(register, AUTH_REGISTER_TEMPLATE);
        assert!(
            !stray_module_dir_exists,
            "auth templates must not go to a crate-less crates/modules/auth"
        );
    }

    /// Postgres-only tokens that must never appear in a SQLite migration body —
    /// these are exactly what broke `rwfw new app --database sqlite` before the
    /// example migrations gained a dedicated SQLite dialect.
    const PG_ONLY: &[&str] = &[
        "SERIAL",
        "TIMESTAMPTZ",
        "CREATE SCHEMA",
        "NOW()",
        "BOOLEAN",
        "INTERVAL",
        " UUID",
    ];

    fn assert_sqlite_dialect(sql: &str) {
        for token in PG_ONLY {
            assert!(
                !sql.contains(token),
                "SQLite migration body must not contain Postgres-only `{token}`"
            );
        }
        assert!(
            sql.contains("INTEGER PRIMARY KEY AUTOINCREMENT"),
            "SQLite migration body should use AUTOINCREMENT integer PKs"
        );
        assert!(
            sql.contains("strftime("),
            "SQLite migration body should default timestamps via strftime()"
        );
    }

    #[test]
    fn blog_example_emits_dual_body_sqlite_migration() {
        let mod_rs = blog_migrations_mod_rs();
        assert!(
            mod_rs.contains("Migration::with_sqlite"),
            "blog migrations mod must register a SQLite body"
        );
        assert!(mod_rs.contains("create_posts_table.sqlite.sql"));
        let sql = blog_create_posts_table_sqlite_sql();
        assert_sqlite_dialect(&sql);
        assert!(sql.contains("blog_posts"));
    }

    #[test]
    fn shop_example_emits_dual_body_sqlite_migration() {
        let mod_rs = shop_migrations_mod_rs();
        assert!(mod_rs.contains("Migration::with_sqlite"));
        assert!(mod_rs.contains("create_shop_tables.sqlite.sql"));
        let sql = shop_create_tables_sqlite_sql();
        assert_sqlite_dialect(&sql);
        for table in [
            "shop_categories",
            "shop_products",
            "shop_orders",
            "shop_order_items",
        ] {
            assert!(
                sql.contains(table),
                "shop SQLite body missing table {table}"
            );
        }
    }

    #[test]
    fn example_tables_are_schema_free_on_both_backends() {
        // Models drop `schema_name` and use prefixed table names so the same
        // entity works on Postgres and SQLite; the Postgres migration bodies must
        // agree (no schema, no schema-qualified table names).
        let blog_model = blog_post_model_rs();
        assert!(!blog_model.contains("schema_name"));
        assert!(blog_model.contains(r#"table_name = "blog_posts""#));

        for model in [
            shop_category_model_rs(),
            shop_product_model_rs(),
            shop_order_model_rs(),
            shop_order_item_model_rs(),
        ] {
            assert!(
                !model.contains("schema_name"),
                "shop model must not be schema-qualified"
            );
        }

        let blog_pg = blog_create_posts_table_sql();
        assert!(!blog_pg.contains("CREATE SCHEMA"));
        assert!(!blog_pg.contains("blog.posts"));

        let shop_pg = shop_create_tables_sql();
        assert!(!shop_pg.contains("CREATE SCHEMA"));
        assert!(!shop_pg.contains("shop.categories"));
    }
}
