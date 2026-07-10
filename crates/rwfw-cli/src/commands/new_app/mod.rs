use anyhow::Context;
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
    let app_name = to_kebab(name);
    if app_name.is_empty() {
        anyhow::bail!("App name must contain at least one ASCII letter or number");
    }

    let app_dir = Path::new(&app_name);
    if app_dir.exists() {
        anyhow::bail!("App directory already exists: {}", app_dir.display());
    }

    let dependency_source = DependencySource::resolve(rwfw_path, rwfw_version, rwfw_git, rwfw_tag)?;
    let context = AppTemplateContext {
        app_name: app_name.clone(),
        app_crate_ident: app_name.replace('-', "_"),
        app_title: to_title(&app_name),
        database_name: format!("{}_dev", app_name.replace('-', "_")),
        docker_database_name: app_name.replace('-', "_"),
        rwfw_core_dep: dependency_source.dependency("rwfw-core"),
        rwfw_shared_dep: dependency_source.dependency("rwfw-shared"),
        rwfw_macros_dep: dependency_source.dependency("rwfw-macros"),
        mod_auth_dep: dependency_source.module_dependency("auth"),
        rwfw_tauri_dep: dependency_source.dependency_with_features("rwfw-tauri", &["desktop"]),
        example,
        database,
        tauri,
        dependency_source,
    };

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
    // The base layout's import map points at /assets/reactive.js; without the
    // file the whole Stimulus boot module would fail in generated apps.
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
    write_file(
        &app_dir.join("crates/modules/auth/web/templates/login.html.j2"),
        AUTH_LOGIN_TEMPLATE.to_string(),
    )?;
    write_file(
        &app_dir.join("crates/modules/auth/web/templates/register.html.j2"),
        AUTH_REGISTER_TEMPLATE.to_string(),
    )?;

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
    println!("  cd {app_name}");
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

    fn module_dependency(&self, module_name: &str) -> String {
        match self {
            Self::Path(root) => {
                let path = root.join("crates").join("modules").join(module_name);
                format!("{{ path = \"{}\" }}", path_for_toml(&path))
            }
            Self::Version(version) => format!("\"{version}\""),
            Self::Git { repository, tag } => git_dependency(repository, tag),
        }
    }

    /// Like [`dependency`] but with explicit cargo features (e.g. `desktop` for
    /// `rwfw-tauri`). Always emits an inline-table form.
    fn dependency_with_features(&self, crate_name: &str, features: &[&str]) -> String {
        let feats = features
            .iter()
            .map(|f| format!("\"{f}\""))
            .collect::<Vec<_>>()
            .join(", ");
        match self {
            Self::Path(root) => {
                let path = root.join("crates").join(crate_name);
                format!(
                    "{{ path = \"{}\", features = [{feats}] }}",
                    path_for_toml(&path)
                )
            }
            Self::Version(version) => {
                format!("{{ version = \"{version}\", features = [{feats}] }}")
            }
            Self::Git { repository, tag } => {
                format!("{{ git = \"{repository}\", tag = \"{tag}\", features = [{feats}] }}")
            }
        }
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
        "crates/modules/auth/web/templates",
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
- `crates/modules/auth`: server-rendered auth pages for the framework auth module.

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
sea-orm = { version = "1", features = ["sqlx-postgres", "sqlx-sqlite", "runtime-tokio-rustls", "macros"] }
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
    r#"database:
  url: "${RWFW__DATABASE__URL}"

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
  rate_limit:
    enabled: false
    max_requests: 10
    window_secs: 60

auth:
  session_ttl: 86400
  oidc:
    redirect_base_url: "http://localhost:8080"
    providers: {}
"#
    .to_string()
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
        // The base layout's import map maps `rwfw-reactive` to /assets/reactive.js
        // and the boot module imports it eagerly — if the scaffold ever stops
        // writing the asset, every generated app loses Stimulus entirely.
        assert!(
            BASE_LAYOUT_TEMPLATE.contains(r#""rwfw-reactive": "/assets/reactive.js""#),
            "base layout must map the reactive runtime in the import map"
        );
        assert!(
            BASE_LAYOUT_TEMPLATE.contains(r#"register("reactive""#),
            "base layout must register the reactive Stimulus controller"
        );
        assert!(
            REACTIVE_JS.contains("Controller"),
            "embedded reactive.js must be the Stimulus controller source"
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
