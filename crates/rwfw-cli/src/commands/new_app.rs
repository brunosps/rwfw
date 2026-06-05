use anyhow::Context;
use std::fs;
use std::path::{Path, PathBuf};

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
        write_file(&app_dir.join("compose.dev.yaml"), compose_dev_yaml(&context))?;
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
    write_file(&app_dir.join("crates/app/web/vendor.lock"), VENDOR_LOCK.to_string())?;
    write_file(&app_dir.join("crates/app/web/assets/app.css"), APP_CSS.to_string())?;
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
    println!("Next steps:");
    println!("  cd {app_name}");
    println!("  docker compose -f compose.dev.yaml up -d");
    println!("  rwfw migrate");
    println!("  rwfw seed admin --email admin@example.com --password rwfw-admin-123");
    println!("  rwfw dev");
    println!(
        "  open http://localhost:3000{}",
        context.example.root_path()
    );
    println!("  docker compose up --build");
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
            "crates/modules/shop/web/templates/products",
            "crates/modules/shop/web/templates/cart",
            "crates/modules/shop/web/templates/checkout",
            "crates/modules/shop/web/templates/orders",
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

const TURBO_JS: &[u8] = include_bytes!("../../../rwfw-app/web/vendor/turbo.min.js");
const STIMULUS_JS: &[u8] = include_bytes!("../../../rwfw-app/web/vendor/stimulus.min.js");
const VENDOR_LOCK: &str = include_str!("../../../rwfw-app/web/vendor.lock");
const APP_CSS: &str = include_str!("../../../rwfw-app/web/assets/app.css");
const BASE_LAYOUT_TEMPLATE: &str =
    include_str!("../../../rwfw-app/web/templates/layouts/base.html.j2");
const APP_LAYOUT_TEMPLATE: &str =
    include_str!("../../../rwfw-app/web/templates/layouts/app.html.j2");
const AUTH_LAYOUT_TEMPLATE: &str =
    include_str!("../../../rwfw-app/web/templates/layouts/auth.html.j2");
const FLASH_COMPONENT_TEMPLATE: &str =
    include_str!("../../../rwfw-app/web/templates/components/flash/index.html.j2");
const FIELD_COMPONENT_TEMPLATE: &str =
    include_str!("../../../rwfw-app/web/templates/components/field/index.html.j2");
const COMPONENTS_CATALOG_TEMPLATE: &str =
    include_str!("../../../rwfw-app/web/templates/components_catalog.html.j2");
const AUTH_LOGIN_TEMPLATE: &str =
    include_str!("../../../modules/auth/web/templates/login.html.j2");
const AUTH_REGISTER_TEMPLATE: &str =
    include_str!("../../../modules/auth/web/templates/register.html.j2");

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

fn gitignore() -> String {
    r#"/target
.env
.DS_Store
/data/*.db
/data/*.db-wal
/data/*.db-shm
"#
    .to_string()
}

fn dockerignore() -> String {
    r#"/target
/.git
/.env
*.log
"#
    .to_string()
}

fn env_example() -> String {
    r#"# RWFW local environment overrides
# SSO client credentials are appended by `rwfw auth sso add`.
"#
    .to_string()
}

fn dockerfile(context: &AppTemplateContext) -> String {
    let framework_copy = match &context.dependency_source {
        DependencySource::Path(_) => {
            "COPY --from=rwfw-framework . ${RWFW_FRAMEWORK_PATH}\n".to_string()
        }
        DependencySource::Version(_) | DependencySource::Git { .. } => String::new(),
    };

    format!(
        r#"# syntax=docker/dockerfile:1.7

ARG APP_PACKAGE={}
ARG RWFW_FRAMEWORK_PATH=/opt/rwfw

FROM rust:1-bookworm AS rust-builder
ARG APP_PACKAGE
ARG RWFW_FRAMEWORK_PATH
WORKDIR /app
{framework_copy}COPY . .
RUN cargo build --release -p "${{APP_PACKAGE}}"

FROM debian:bookworm-slim AS runtime
ARG APP_PACKAGE
ENV APP_PACKAGE=${{APP_PACKAGE}}
ENV RWFW_ENV=production
ENV RWFW_RUN_MIGRATIONS=1
WORKDIR /app
RUN apt-get update \
    && apt-get install -y --no-install-recommends ca-certificates libssl3 libstdc++6 \
    && rm -rf /var/lib/apt/lists/*
COPY --from=rust-builder /app/target/release/${{APP_PACKAGE}} /usr/local/bin/rwfw-app
COPY --from=rust-builder /app/config ./config
# Web assets (Hotwire templates, vendored JS, compiled CSS) are loaded from disk
# at each crate's compile-time path, so the source web dirs are copied in.
COPY --from=rust-builder /app/crates ./crates
COPY docker/entrypoint.sh /usr/local/bin/rwfw-entrypoint
RUN chmod +x /usr/local/bin/rwfw-entrypoint
EXPOSE 8080
ENTRYPOINT ["rwfw-entrypoint"]
"#,
        context.app_name
    )
}

fn compose_yaml(context: &AppTemplateContext) -> String {
    let additional_contexts = match &context.dependency_source {
        DependencySource::Path(root) => format!(
            "      additional_contexts:\n        rwfw-framework: \"{}\"\n",
            path_for_yaml(root)
        ),
        DependencySource::Version(_) | DependencySource::Git { .. } => String::new(),
    };

    let framework_arg = match &context.dependency_source {
        DependencySource::Path(root) => {
            format!("        RWFW_FRAMEWORK_PATH: \"{}\"\n", path_for_yaml(root))
        }
        DependencySource::Version(_) | DependencySource::Git { .. } => String::new(),
    };

    format!(
        r#"services:
  db:
    image: postgres:16-alpine
    environment:
      POSTGRES_DB: "{database}"
      POSTGRES_USER: rwfw
      POSTGRES_PASSWORD: rwfw
    ports:
      - "54329:5432"
    healthcheck:
      test: ["CMD-SHELL", "pg_isready -U rwfw -d {database}"]
      interval: 5s
      timeout: 5s
      retries: 20
    volumes:
      - postgres-data:/var/lib/postgresql/data

  app:
    build:
      context: .
      dockerfile: Dockerfile.prod
{additional_contexts}      args:
        APP_PACKAGE: "{app_name}"
{framework_arg}    environment:
      RWFW_ENV: production
      RWFW_RUN_MIGRATIONS: "1"
      RWFW__DATABASE__URL: "postgres://rwfw:rwfw@db:5432/{database}"
    depends_on:
      db:
        condition: service_healthy
    ports:
      - "8080:8080"

volumes:
  postgres-data:
"#,
        app_name = context.app_name,
        database = context.docker_database_name,
        additional_contexts = additional_contexts,
        framework_arg = framework_arg
    )
}

fn compose_dev_yaml(context: &AppTemplateContext) -> String {
    format!(
        r#"services:
  db:
    image: postgres:16-alpine
    environment:
      POSTGRES_DB: "{database}"
      POSTGRES_USER: rwfw
      POSTGRES_PASSWORD: rwfw
    ports:
      - "${{RWFW_DEV_DB_PORT:-54329}}:5432"
    healthcheck:
      test: ["CMD-SHELL", "pg_isready -U rwfw -d {database}"]
      interval: 5s
      timeout: 5s
      retries: 20
    volumes:
      - postgres-data:/var/lib/postgresql/data

  pgadmin:
    image: dpage/pgadmin4:8
    environment:
      PGADMIN_DEFAULT_EMAIL: "${{PGADMIN_DEFAULT_EMAIL:-admin@example.com}}"
      PGADMIN_DEFAULT_PASSWORD: "${{PGADMIN_DEFAULT_PASSWORD:-rwfw}}"
      PGADMIN_CONFIG_SERVER_MODE: "False"
    depends_on:
      db:
        condition: service_healthy
    ports:
      - "${{RWFW_PGADMIN_PORT:-5053}}:80"
    volumes:
      - pgadmin-data:/var/lib/pgadmin

volumes:
  postgres-data:
  pgadmin-data:
"#,
        database = context.database_name
    )
}

fn docker_entrypoint_sh() -> String {
    r#"#!/usr/bin/env sh
set -eu

if [ "${RWFW_RUN_MIGRATIONS:-1}" = "1" ]; then
  rwfw-app __rwfw migrate
fi

exec rwfw-app
"#
    .to_string()
}

fn readme_md(context: &AppTemplateContext) -> String {
    format!(
        r#"# {title}

Generated by RWFW. This project is a runnable example application and a starting point for building modular Rust web apps with Axum, SeaORM, Hotwire (Turbo + Stimulus), MiniJinja templates, and PostgreSQL — with zero npm/node.

Example selected: `{example}` ({example_title}).

## Framework Dependency Source

{dependency_source}

## What Is In This Project

- `rwfw.toml`: RWFW project metadata used by the CLI to find the app crate and modules directory.
- `Cargo.toml`: Rust workspace with the app crate and local modules.
- `.env.example`: local environment variables for generated integrations such as SSO providers.
- `config/development.yaml`: local runtime config used by `rwfw dev`.
- `config/production.yaml`: production runtime config used by the Docker image.
- `compose.dev.yaml`: local development services: PostgreSQL and pgAdmin.
- `compose.yaml`: production-style Docker stack: PostgreSQL plus the compiled app container.
- `Dockerfile.prod`: single-stage Rust production image (no node); web assets are loaded from disk.
- `docker/entrypoint.sh`: production container entrypoint; optionally runs migrations before starting the app.
- `.rwfw/templates`: project-local code generation templates used by `rwfw generate`.
- `crates/app`: application composition crate. It wires modules, config, and migrations into one binary, and owns the shared `web/` (Hotwire layouts, vendored Turbo/Stimulus, `assets/app.css`).
- `crates/modules/home`: minimal example module with one page.
- `crates/modules/{example_module}`: generated `{example}` example module.
- `crates/modules/auth`: server-rendered auth pages for the framework auth module.

## Local Development

```bash
docker compose -f compose.dev.yaml up -d
rwfw migrate
rwfw seed admin --email admin@example.com --password rwfw-admin-123
rwfw dev
```

The dev database is exposed on `localhost:54329` by default:

```text
postgres://rwfw:rwfw@localhost:54329/{development_database}
```

pgAdmin is available at http://localhost:5053 with:

```text
email: admin@example.com
password: rwfw
```

Use these variables to avoid port conflicts:

```bash
RWFW_DEV_DB_PORT=54332 RWFW_PGADMIN_PORT=5055 docker compose -f compose.dev.yaml up -d
```

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

## Production Docker

```bash
docker compose up --build
```

The production compose stack builds `Dockerfile.prod` and listens on http://localhost:8080{root_path}.

Migrations run automatically when the app container starts. Set `RWFW_RUN_MIGRATIONS=0` to disable that behavior.

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
        development_database = context.database_name,
        example = context.example.label(),
        example_title = context.example.readme_title(),
        dependency_source = readme_dependency_source(&context.dependency_source),
        example_module = context.example.module_name(),
        root_path = context.example.root_path(),
        example_routes = readme_example_routes(context.example),
    )
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

auth:
  session_ttl: 86400
  oidc:
    redirect_base_url: "http://localhost:3000"
    providers: {{}}
"#,
        db_url = dev_database_url(context)
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
    write_file(&app_dir.join("src-tauri/src/main.rs"), tauri_main_rs(context))?;
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

fn write_blog_example(app_dir: &Path, context: &AppTemplateContext) -> anyhow::Result<()> {
    write_file(
        &app_dir.join("crates/modules/blog/Cargo.toml"),
        blog_cargo_toml(context),
    )?;
    write_file(
        &app_dir.join("crates/modules/blog/src/lib.rs"),
        blog_lib_rs(),
    )?;
    write_file(
        &app_dir.join("crates/modules/blog/src/migrations/mod.rs"),
        blog_migrations_mod_rs(),
    )?;
    write_file(
        &app_dir.join("crates/modules/blog/src/migrations/20260101000000_create_posts_table.sql"),
        blog_create_posts_table_sql(),
    )?;
    write_file(
        &app_dir.join("crates/modules/blog/src/models/mod.rs"),
        blog_models_mod_rs(),
    )?;
    write_file(
        &app_dir.join("crates/modules/blog/src/models/post.rs"),
        blog_post_model_rs(),
    )?;
    write_file(
        &app_dir.join("crates/modules/blog/src/repositories/mod.rs"),
        blog_repositories_mod_rs(),
    )?;
    write_file(
        &app_dir.join("crates/modules/blog/src/repositories/post_repo.rs"),
        blog_post_repository_rs(),
    )?;
    write_file(
        &app_dir.join("crates/modules/blog/src/use_cases/mod.rs"),
        blog_use_cases_mod_rs(),
    )?;
    write_file(
        &app_dir.join("crates/modules/blog/src/use_cases/save_post.rs"),
        blog_save_post_use_case_rs(),
    )?;
    write_file(
        &app_dir.join("crates/modules/blog/src/routes/index.rs"),
        blog_index_route_rs(),
    )?;
    write_file(
        &app_dir.join("crates/modules/blog/src/routes/posts/index.rs"),
        blog_posts_index_route_rs(),
    )?;
    write_file(
        &app_dir.join("crates/modules/blog/src/routes/posts/[slug].rs"),
        blog_post_show_route_rs(),
    )?;
    write_file(
        &app_dir.join("crates/modules/blog/src/routes/admin/posts/index.rs"),
        blog_admin_posts_index_route_rs(),
    )?;
    write_file(
        &app_dir.join("crates/modules/blog/src/routes/admin/posts/new.rs"),
        blog_admin_posts_new_route_rs(),
    )?;
    write_file(
        &app_dir.join("crates/modules/blog/src/routes/admin/posts/[id].rs"),
        blog_admin_post_item_route_rs(),
    )?;
    write_file(
        &app_dir.join("crates/modules/blog/src/routes/admin/posts/[id]/edit.rs"),
        blog_admin_post_edit_route_rs(),
    )?;
    write_file(
        &app_dir.join("crates/modules/blog/web/templates/index.html.j2"),
        blog_index_page_template(),
    )?;
    write_file(
        &app_dir.join("crates/modules/blog/web/templates/posts/index.html.j2"),
        blog_posts_index_page_template(),
    )?;
    write_file(
        &app_dir.join("crates/modules/blog/web/templates/posts/show.html.j2"),
        blog_post_show_page_template(),
    )?;
    write_file(
        &app_dir.join("crates/modules/blog/web/templates/admin/posts/index.html.j2"),
        blog_admin_posts_index_page_template(),
    )?;
    write_file(
        &app_dir.join("crates/modules/blog/web/templates/admin/posts/_form.html.j2"),
        blog_admin_post_form_template(),
    )?;
    write_file(
        &app_dir.join("crates/modules/blog/web/templates/admin/posts/new.html.j2"),
        blog_admin_post_new_page_template(),
    )?;
    write_file(
        &app_dir.join("crates/modules/blog/web/templates/admin/posts/edit.html.j2"),
        blog_admin_post_edit_page_template(),
    )?;

    Ok(())
}

fn blog_cargo_toml(context: &AppTemplateContext) -> String {
    format!(
        r#"[package]
name = "mod-blog"
version.workspace = true
edition.workspace = true

[dependencies]
rwfw-core = {}
rwfw-shared = {}
rwfw-macros = {}
axum = {{ workspace = true }}
async-trait = {{ workspace = true }}
inventory = {{ workspace = true }}
serde = {{ workspace = true }}
serde_json = {{ workspace = true }}
sea-orm = {{ workspace = true }}
chrono = {{ workspace = true }}
tracing = {{ workspace = true }}
anyhow = {{ workspace = true }}
uuid = {{ workspace = true }}
"#,
        context.rwfw_core_dep, context.rwfw_shared_dep, context.rwfw_macros_dep
    )
}

fn app_lib_rs(context: &AppTemplateContext) -> String {
    r#"use axum::response::Redirect;
use axum::Router;
use rwfw_core::config::AppConfig;
use rwfw_core::module::{Module, ModuleRegistration};

// Force linker to include module crates so inventory can discover them.
extern crate mod_auth;
extern crate __EXAMPLE_CRATE_IDENT__;
extern crate mod_home;

pub fn registered_modules() -> Vec<Box<dyn Module>> {
    inventory::iter::<ModuleRegistration>
        .into_iter()
        .map(|reg| {
            tracing::info!(module = %reg.name, "Discovered module");
            reg.create()
        })
        .collect()
}

pub async fn build_router(config: AppConfig) -> anyhow::Result<Router> {
    let mut app_builder = rwfw_core::app::RwfwApp::new(config)
        .web_root(std::path::Path::new(env!("CARGO_MANIFEST_DIR")).join("web"));

    for module in registered_modules() {
        app_builder = app_builder.module(module);
    }

    Ok(app_builder
        .build()
        .await?
        .route("/", axum::routing::get(root_redirect)))
}

async fn root_redirect() -> Redirect {
    Redirect::to("__ROOT_PATH__")
}

pub async fn run_migrations(
    config: &AppConfig,
) -> anyhow::Result<rwfw_core::migration::MigrationReport> {
    let db = rwfw_core::db::connect(config).await?;
    let modules = registered_modules();
    rwfw_core::migration::run_pending_migrations(&db, &modules).await
}
"#
    .replace(
        "__EXAMPLE_CRATE_IDENT__",
        context.example.module_crate_ident(),
    )
    .replace("__ROOT_PATH__", context.example.root_path())
}

fn app_main_rs(context: &AppTemplateContext) -> String {
    app_main_template().replace("__APP_CRATE_IDENT__", &context.app_crate_ident)
}

fn app_main_template() -> &'static str {
    r#"use rwfw_core::config::AppConfig;

#[tokio::main]
async fn main() -> anyhow::Result<()> {
    let config = AppConfig::load()?;
    init_logging(&config);

    let mut args = std::env::args().skip(1);
    if args.next().as_deref() == Some("__rwfw") {
        return run_rwfw_command(args.next().as_deref(), &config).await;
    }

    serve(config).await
}

fn init_logging(config: &AppConfig) {
    if let Ok(logging_config) = config.logging() {
        rwfw_core::logging::init(&logging_config);
    } else {
        rwfw_core::logging::init_default();
    }
}

async fn serve(config: AppConfig) -> anyhow::Result<()> {
    tracing::info!("Starting RWFW application");

    let router = __APP_CRATE_IDENT__::build_router(config.clone()).await?;

    let server_config = config
        .server()
        .unwrap_or_else(|_| rwfw_core::config::ServerConfig {
            host: "0.0.0.0".to_string(),
            port: 3000,
        });

    let addr = format!("{}:{}", server_config.host, server_config.port);
    tracing::info!(addr = %addr, "Server listening");

    let listener = tokio::net::TcpListener::bind(&addr).await?;
    axum::serve(listener, router).await?;

    Ok(())
}

async fn run_rwfw_command(command: Option<&str>, config: &AppConfig) -> anyhow::Result<()> {
    match command {
        Some("migrate") => {
            println!("Running migrations...");
            let report = __APP_CRATE_IDENT__::run_migrations(config).await?;

            for migration in &report.applied {
                println!(
                    "Applied {}:{} {}",
                    migration.module, migration.version, migration.name
                );
            }

            if report.applied.is_empty() {
                println!("No pending migrations.");
            } else {
                println!("Applied {} migration(s).", report.applied.len());
            }

            Ok(())
        }
        Some(other) => anyhow::bail!("Unknown RWFW app command: {other}"),
        None => anyhow::bail!("Missing RWFW app command"),
    }
}
"#
}

fn home_lib_rs() -> String {
    r#"pub mod migrations;

use axum::Router;
use rwfw_core::app::AppState;
use rwfw_core::module::{Module, ModuleRegistration, NavItem};

#[rwfw_macros::rwfw_routes("src/routes")]
pub struct HomeRoutes;

pub struct HomeModule;

impl HomeModule {
    pub fn new() -> Self {
        Self
    }
}

#[async_trait::async_trait]
impl Module for HomeModule {
    fn name(&self) -> &str {
        "home"
    }

    fn routes(&self) -> Router<AppState> {
        HomeRoutes::generated_routes()
    }

    fn migrations(&self) -> Vec<rwfw_core::migration::Migration> {
        migrations::migrations()
    }

    fn nav_items(&self) -> Vec<NavItem> {
        vec![NavItem {
            label: "Home".to_string(),
            href: "/home".to_string(),
            icon: Some("home".to_string()),
        }]
    }

    fn web_root(&self) -> Option<std::path::PathBuf> {
        Some(std::path::PathBuf::from(env!("CARGO_MANIFEST_DIR")).join("web"))
    }
}

inventory::submit! {
    ModuleRegistration::new("home", || Box::new(HomeModule::new()))
}
"#
    .to_string()
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

fn home_index_route_rs() -> String {
    r#"use axum::response::Response;
use axum::routing;
use rwfw_core::app::AppState;
use rwfw_core::view::View;

pub fn route() -> axum::routing::MethodRouter<AppState> {
    routing::get(get)
}

async fn get(v: View) -> Response {
    v.render(
        "home/index",
        serde_json::json!({
            "title": "Welcome to RWFW",
            "description": "A modular Rust web framework — Hotwire + MiniJinja, zero npm"
        }),
    )
}
"#
    .to_string()
}

fn home_index_page_template() -> &'static str {
    r#"{% extends "layouts/app.html.j2" %}
{% block title %}{{ title }}{% endblock %}
{% block content %}
<div class="max-w-4xl">
  <h1 class="text-4xl font-bold text-gray-900 mb-4">{{ title }}</h1>
  <p class="text-lg text-gray-600 mb-8">{{ description }}</p>
  <div class="grid grid-cols-1 md:grid-cols-2 gap-6">
    <div class="p-6 bg-white rounded-lg shadow-sm border">
      <h2 class="text-xl font-semibold mb-2">Modular Architecture</h2>
      <p class="text-gray-600">Each module is self-contained with its own routes, models, and templates.</p>
    </div>
    <div class="p-6 bg-white rounded-lg shadow-sm border">
      <h2 class="text-xl font-semibold mb-2">Hotwire + MiniJinja</h2>
      <p class="text-gray-600">Server-rendered HTML with Turbo navigation — no hydration, no npm.</p>
    </div>
    <div class="p-6 bg-white rounded-lg shadow-sm border">
      <h2 class="text-xl font-semibold mb-2">Turbo Streams</h2>
      <p class="text-gray-600">Real-time partial updates over SSE/WebSocket without a JS framework.</p>
    </div>
    <div class="p-6 bg-white rounded-lg shadow-sm border">
      <h2 class="text-xl font-semibold mb-2">Single Binary Deploy</h2>
      <p class="text-gray-600">All assets embedded in one Rust binary for simple deployment.</p>
    </div>
  </div>
</div>
{% endblock %}
"#
}

fn blog_lib_rs() -> String {
    r#"pub mod migrations;
pub mod models;
pub mod repositories;
pub mod use_cases;

use axum::Router;
use rwfw_core::app::AppState;
use rwfw_core::auth::Permission;
use rwfw_core::module::{Module, ModuleRegistration, NavItem};

#[rwfw_macros::rwfw_routes("src/routes")]
pub struct BlogRoutes;

pub struct BlogModule;

impl BlogModule {
    pub fn new() -> Self {
        Self
    }
}

#[async_trait::async_trait]
impl Module for BlogModule {
    fn name(&self) -> &str {
        "blog"
    }

    fn routes(&self) -> Router<AppState> {
        BlogRoutes::generated_routes()
    }

    fn migrations(&self) -> Vec<rwfw_core::migration::Migration> {
        migrations::migrations()
    }

    fn permissions(&self) -> Vec<Permission> {
        vec![Permission::new("blog.posts.manage", "Manage blog posts")]
    }

    fn nav_items(&self) -> Vec<NavItem> {
        vec![
            NavItem {
                label: "Blog".to_string(),
                href: "/blog".to_string(),
                icon: Some("newspaper".to_string()),
            },
            NavItem {
                label: "Blog Admin".to_string(),
                href: "/blog/admin/posts".to_string(),
                icon: Some("edit".to_string()),
            },
        ]
    }
}

inventory::submit! {
    ModuleRegistration::new("blog", || Box::new(BlogModule::new()))
}
"#
    .to_string()
}

fn blog_migrations_mod_rs() -> String {
    r#"use rwfw_core::migration::Migration;

pub fn migrations() -> Vec<Migration> {
    vec![
        Migration::new(
            "20260101000000",
            "create_posts_table",
            include_str!("20260101000000_create_posts_table.sql"),
        ),
    ]
}
"#
    .to_string()
}

fn blog_create_posts_table_sql() -> String {
    r#"CREATE SCHEMA IF NOT EXISTS blog;

CREATE TABLE IF NOT EXISTS blog.posts (
    id SERIAL PRIMARY KEY,
    title VARCHAR(255) NOT NULL,
    slug VARCHAR(255) NOT NULL UNIQUE,
    excerpt TEXT NOT NULL,
    body TEXT NOT NULL,
    cover_image_url VARCHAR(500),
    status VARCHAR(32) NOT NULL DEFAULT 'draft',
    published_at TIMESTAMPTZ,
    author_id UUID,
    created_at TIMESTAMPTZ NOT NULL DEFAULT NOW(),
    updated_at TIMESTAMPTZ NOT NULL DEFAULT NOW()
);

CREATE INDEX IF NOT EXISTS idx_blog_posts_status_published_at
    ON blog.posts (status, published_at DESC);

INSERT INTO blog.posts
    (title, slug, excerpt, body, cover_image_url, status, published_at, author_id, created_at, updated_at)
VALUES
    (
        'Building Rails-like conventions in Rust',
        'building-rails-like-conventions-in-rust',
        'A practical look at modules, migrations, routing and generated CRUD in RWFW.',
        'RWFW starts from the idea that internal software should be boring in the right places. Modules own their routes, migrations and UI, while the app crate only composes them. That keeps the framework explicit without forcing every team to rewrite the same glue.',
        'https://images.unsplash.com/photo-1515879218367-8466d910aaa4?auto=format&fit=crop&w=1200&q=80',
        'published',
        NOW() - INTERVAL '3 days',
        NULL,
        NOW() - INTERVAL '3 days',
        NOW() - INTERVAL '3 days'
    ),
    (
        'Why migrations belong to modules',
        'why-migrations-belong-to-modules',
        'Module-owned migrations make feature boundaries visible and distributable.',
        'A module should be able to explain the data it owns. Keeping migrations close to module code makes generated applications easier to inspect, copy and evolve. RWFW still keeps a central migration ledger so execution stays predictable.',
        'https://images.unsplash.com/photo-1451187580459-43490279c0fa?auto=format&fit=crop&w=1200&q=80',
        'published',
        NOW() - INTERVAL '1 day',
        NULL,
        NOW() - INTERVAL '1 day',
        NOW() - INTERVAL '1 day'
    )
ON CONFLICT (slug) DO NOTHING;
"#
    .to_string()
}

fn blog_models_mod_rs() -> String {
    r#"pub mod post;

pub use post::Post;
"#
    .to_string()
}

fn blog_post_model_rs() -> String {
    r#"use sea_orm::entity::prelude::*;
use serde::{Deserialize, Serialize};

#[derive(Clone, Debug, PartialEq, DeriveEntityModel, Serialize, Deserialize)]
#[sea_orm(table_name = "posts", schema_name = "blog")]
pub struct Model {
    #[sea_orm(primary_key)]
    pub id: i32,
    pub title: String,
    pub slug: String,
    #[sea_orm(column_type = "Text")]
    pub excerpt: String,
    #[sea_orm(column_type = "Text")]
    pub body: String,
    pub cover_image_url: Option<String>,
    pub status: String,
    pub published_at: Option<DateTimeWithTimeZone>,
    pub author_id: Option<Uuid>,
    pub created_at: DateTimeWithTimeZone,
    pub updated_at: DateTimeWithTimeZone,
}

#[derive(Copy, Clone, Debug, EnumIter, DeriveRelation)]
pub enum Relation {}

impl ActiveModelBehavior for ActiveModel {}

pub type Post = Model;
"#
    .to_string()
}

fn blog_repositories_mod_rs() -> String {
    r#"pub mod post_repo;
"#
    .to_string()
}

fn blog_post_repository_rs() -> String {
    r#"use crate::models::post::{self, ActiveModel, Column, Entity as PostEntity};
use sea_orm::*;

pub struct PostRepository {
    db: DatabaseConnection,
}

#[derive(Debug)]
pub struct SavePost {
    pub title: String,
    pub slug: String,
    pub excerpt: String,
    pub body: String,
    pub cover_image_url: Option<String>,
    pub status: String,
    pub published_at: Option<chrono::DateTime<chrono::FixedOffset>>,
    pub author_id: Option<uuid::Uuid>,
}

impl PostRepository {
    pub fn new(db: DatabaseConnection) -> Self {
        Self { db }
    }

    pub async fn find_published(&self) -> anyhow::Result<Vec<post::Model>> {
        Ok(PostEntity::find()
            .filter(Column::Status.eq("published"))
            .order_by_desc(Column::PublishedAt)
            .all(&self.db)
            .await?)
    }

    pub async fn find_all_for_admin(&self) -> anyhow::Result<Vec<post::Model>> {
        Ok(PostEntity::find()
            .order_by_desc(Column::UpdatedAt)
            .all(&self.db)
            .await?)
    }

    pub async fn find_by_id(&self, id: i32) -> anyhow::Result<Option<post::Model>> {
        Ok(PostEntity::find_by_id(id).one(&self.db).await?)
    }

    pub async fn find_published_by_slug(&self, slug: &str) -> anyhow::Result<Option<post::Model>> {
        Ok(PostEntity::find()
            .filter(Column::Slug.eq(slug))
            .filter(Column::Status.eq("published"))
            .one(&self.db)
            .await?)
    }

    pub async fn slug_exists(&self, slug: &str, except_id: Option<i32>) -> anyhow::Result<bool> {
        let mut query = PostEntity::find().filter(Column::Slug.eq(slug));
        if let Some(id) = except_id {
            query = query.filter(Column::Id.ne(id));
        }
        Ok(query.one(&self.db).await?.is_some())
    }

    pub async fn create(&self, data: SavePost) -> anyhow::Result<post::Model> {
        let now = chrono::Utc::now().fixed_offset();
        let model = ActiveModel {
            title: Set(data.title),
            slug: Set(data.slug),
            excerpt: Set(data.excerpt),
            body: Set(data.body),
            cover_image_url: Set(data.cover_image_url),
            status: Set(data.status),
            published_at: Set(data.published_at),
            author_id: Set(data.author_id),
            created_at: Set(now),
            updated_at: Set(now),
            ..Default::default()
        };
        Ok(model.insert(&self.db).await?)
    }

    pub async fn update(&self, id: i32, data: SavePost) -> anyhow::Result<post::Model> {
        let post = PostEntity::find_by_id(id)
            .one(&self.db)
            .await?
            .ok_or_else(|| anyhow::anyhow!("Post not found"))?;

        let mut active: ActiveModel = post.into();
        active.title = Set(data.title);
        active.slug = Set(data.slug);
        active.excerpt = Set(data.excerpt);
        active.body = Set(data.body);
        active.cover_image_url = Set(data.cover_image_url);
        active.status = Set(data.status);
        active.published_at = Set(data.published_at);
        active.author_id = Set(data.author_id);
        active.updated_at = Set(chrono::Utc::now().fixed_offset());
        Ok(active.update(&self.db).await?)
    }

    pub async fn delete(&self, id: i32) -> anyhow::Result<()> {
        PostEntity::delete_by_id(id).exec(&self.db).await?;
        Ok(())
    }
}
"#
    .to_string()
}

fn blog_use_cases_mod_rs() -> String {
    r#"pub mod save_post;
"#
    .to_string()
}

fn blog_save_post_use_case_rs() -> String {
    r#"use crate::models::post;
use crate::repositories::post_repo::{PostRepository, SavePost};
use rwfw_core::error::AppError;
use serde::Deserialize;
use std::collections::HashMap;

#[derive(Debug, Deserialize)]
pub struct SavePostInput {
    pub title: String,
    #[serde(default)]
    pub slug: Option<String>,
    pub excerpt: String,
    pub body: String,
    #[serde(default)]
    pub cover_image_url: Option<String>,
    #[serde(default = "default_status")]
    pub status: String,
}

fn default_status() -> String {
    "draft".to_string()
}

pub struct SavePostUseCase;

impl SavePostUseCase {
    pub async fn create(
        &self,
        repo: &PostRepository,
        author_id: uuid::Uuid,
        input: SavePostInput,
    ) -> Result<post::Model, AppError> {
        let data = self.prepare(repo, None, Some(author_id), input, None).await?;
        repo.create(data).await.map_err(AppError::Internal)
    }

    pub async fn update(
        &self,
        repo: &PostRepository,
        id: i32,
        author_id: uuid::Uuid,
        input: SavePostInput,
    ) -> Result<post::Model, AppError> {
        let existing = repo
            .find_by_id(id)
            .await
            .map_err(AppError::Internal)?
            .ok_or_else(|| AppError::NotFound(format!("Post not found: {id}")))?;
        let data = self
            .prepare(repo, Some(id), Some(author_id), input, Some(&existing))
            .await?;
        repo.update(id, data).await.map_err(AppError::Internal)
    }

    async fn prepare(
        &self,
        repo: &PostRepository,
        id: Option<i32>,
        author_id: Option<uuid::Uuid>,
        input: SavePostInput,
        existing: Option<&post::Model>,
    ) -> Result<SavePost, AppError> {
        let title = input.title.trim().to_string();
        let excerpt = input.excerpt.trim().to_string();
        let body = input.body.trim().to_string();
        let cover_image_url = input
            .cover_image_url
            .and_then(|value| optional_string(value.trim()));
        let status = match input.status.as_str() {
            "draft" | "published" => input.status,
            _ => "draft".to_string(),
        };

        let mut errors = HashMap::new();
        if title.is_empty() {
            errors.insert("title".to_string(), vec!["Title is required".to_string()]);
        }
        if excerpt.is_empty() {
            errors.insert("excerpt".to_string(), vec!["Excerpt is required".to_string()]);
        }
        if body.len() < 20 {
            errors.insert(
                "body".to_string(),
                vec!["Body must be at least 20 characters".to_string()],
            );
        }
        if !errors.is_empty() {
            return Err(AppError::Validation(errors));
        }

        let base_slug = input
            .slug
            .as_deref()
            .map(str::trim)
            .filter(|value| !value.is_empty())
            .map(slugify)
            .filter(|value| !value.is_empty())
            .unwrap_or_else(|| slugify(&title));
        let slug = unique_slug(repo, &base_slug, id).await?;
        let published_at = if status == "published" {
            existing
                .and_then(|post| post.published_at)
                .or_else(|| Some(chrono::Utc::now().fixed_offset()))
        } else {
            None
        };

        Ok(SavePost {
            title,
            slug,
            excerpt,
            body,
            cover_image_url,
            status,
            published_at,
            author_id: author_id.or_else(|| existing.and_then(|post| post.author_id)),
        })
    }
}

async fn unique_slug(
    repo: &PostRepository,
    base: &str,
    except_id: Option<i32>,
) -> Result<String, AppError> {
    let base = if base.is_empty() { "post" } else { base };
    let mut slug = base.to_string();
    let mut suffix = 2;

    while repo
        .slug_exists(&slug, except_id)
        .await
        .map_err(AppError::Internal)?
    {
        slug = format!("{base}-{suffix}");
        suffix += 1;
    }

    Ok(slug)
}

fn slugify(value: &str) -> String {
    let mut slug = String::new();
    let mut last_dash = false;

    for ch in value.chars().flat_map(char::to_lowercase) {
        if ch.is_ascii_alphanumeric() {
            slug.push(ch);
            last_dash = false;
        } else if !last_dash && !slug.is_empty() {
            slug.push('-');
            last_dash = true;
        }
    }

    slug.trim_matches('-').to_string()
}

fn optional_string(value: &str) -> Option<String> {
    if value.is_empty() {
        None
    } else {
        Some(value.to_string())
    }
}
"#
    .to_string()
}

fn blog_index_route_rs() -> String {
    r#"use crate::repositories::post_repo::PostRepository;
use axum::extract::State;
use axum::response::{IntoResponse, Response};
use axum::routing;
use rwfw_core::app::AppState;
use rwfw_core::error::AppError;
use rwfw_core::view::View;

pub fn route() -> axum::routing::MethodRouter<AppState> {
    routing::get(get)
}

async fn get(State(state): State<AppState>, v: View) -> Response {
    let repo = PostRepository::new(state.db.clone());
    let posts = match repo.find_published().await {
        Ok(posts) => posts,
        Err(error) => return AppError::Internal(error).into_response(),
    };

    v.render(
        "blog/index",
        serde_json::json!({
            "posts": posts,
        }),
    )
}
"#
    .to_string()
}

fn blog_posts_index_route_rs() -> String {
    r#"use crate::repositories::post_repo::PostRepository;
use axum::extract::State;
use axum::response::{IntoResponse, Response};
use axum::routing;
use rwfw_core::app::AppState;
use rwfw_core::error::AppError;
use rwfw_core::view::View;

pub fn route() -> axum::routing::MethodRouter<AppState> {
    routing::get(get)
}

async fn get(State(state): State<AppState>, v: View) -> Response {
    let repo = PostRepository::new(state.db.clone());
    match repo.find_published().await {
        Ok(posts) => v.render(
            "blog/posts/index",
            serde_json::json!({
                "posts": posts,
            }),
        ),
        Err(error) => AppError::Internal(error).into_response(),
    }
}
"#
    .to_string()
}

fn blog_post_show_route_rs() -> String {
    r#"use crate::repositories::post_repo::PostRepository;
use axum::extract::{Path, State};
use axum::response::{IntoResponse, Response};
use axum::routing;
use rwfw_core::app::AppState;
use rwfw_core::error::AppError;
use rwfw_core::view::View;

pub fn route() -> axum::routing::MethodRouter<AppState> {
    routing::get(get)
}

async fn get(
    State(state): State<AppState>,
    v: View,
    Path(slug): Path<String>,
) -> Response {
    let repo = PostRepository::new(state.db.clone());
    let post = match repo.find_published_by_slug(&slug).await {
        Ok(Some(post)) => post,
        Ok(None) => return AppError::NotFound(format!("Post not found: {slug}")).into_response(),
        Err(error) => return AppError::Internal(error).into_response(),
    };

    v.render(
        "blog/posts/show",
        serde_json::json!({
            "post": post,
        }),
    )
}
"#
    .to_string()
}

fn blog_admin_posts_index_route_rs() -> String {
    r#"use crate::repositories::post_repo::PostRepository;
use crate::use_cases::save_post::{SavePostInput, SavePostUseCase};
use axum::extract::{Form, State};
use axum::http::StatusCode;
use axum::response::{IntoResponse, Redirect, Response};
use axum::routing;
use rwfw_core::app::AppState;
use rwfw_core::auth::CurrentUser;
use rwfw_core::error::AppError;
use rwfw_core::view::View;

pub fn route() -> axum::routing::MethodRouter<AppState> {
    routing::get(get).post(post)
}

async fn get(State(state): State<AppState>, user: CurrentUser, v: View) -> Response {
    if !user.can("blog.posts.manage") {
        return AppError::Forbidden("Missing permission: blog.posts.manage".into()).into_response();
    }

    let repo = PostRepository::new(state.db.clone());
    let posts = match repo.find_all_for_admin().await {
        Ok(posts) => posts,
        Err(error) => return AppError::Internal(error).into_response(),
    };

    v.render(
        "blog/admin/posts/index",
        serde_json::json!({
            "posts": posts,
        }),
    )
}

async fn post(
    State(state): State<AppState>,
    user: CurrentUser,
    v: View,
    Form(input): Form<SavePostInput>,
) -> Response {
    if !user.can("blog.posts.manage") {
        return AppError::Forbidden("Missing permission: blog.posts.manage".into()).into_response();
    }

    let old = serde_json::json!({
        "title": input.title,
        "slug": input.slug,
        "excerpt": input.excerpt,
        "body": input.body,
        "cover_image_url": input.cover_image_url,
        "status": input.status,
    });

    let repo = PostRepository::new(state.db.clone());
    let use_case = SavePostUseCase;
    match use_case.create(&repo, user.id, input).await {
        Ok(post) => Redirect::to(&format!("/blog/admin/posts/{}/edit", post.id)).into_response(),
        Err(AppError::Validation(errors)) => v.render_status(
            StatusCode::UNPROCESSABLE_ENTITY,
            "blog/admin/posts/new",
            serde_json::json!({
                "post": serde_json::Value::Null,
                "old": old,
                "errors": rwfw_core::validation::first_messages(errors),
            }),
        ),
        Err(error) => error.into_response(),
    }
}
"#
    .to_string()
}

fn blog_admin_posts_new_route_rs() -> String {
    r#"use axum::response::{IntoResponse, Response};
use axum::routing;
use rwfw_core::app::AppState;
use rwfw_core::auth::CurrentUser;
use rwfw_core::error::AppError;
use rwfw_core::view::View;

pub fn route() -> axum::routing::MethodRouter<AppState> {
    routing::get(get)
}

async fn get(user: CurrentUser, v: View) -> Response {
    if !user.can("blog.posts.manage") {
        return AppError::Forbidden("Missing permission: blog.posts.manage".into()).into_response();
    }

    v.render(
        "blog/admin/posts/new",
        serde_json::json!({
            "post": serde_json::Value::Null,
        }),
    )
}
"#
    .to_string()
}

fn blog_admin_post_item_route_rs() -> String {
    r#"use crate::repositories::post_repo::PostRepository;
use axum::extract::{Path, State};
use axum::response::{IntoResponse, Redirect, Response};
use axum::routing;
use rwfw_core::app::AppState;
use rwfw_core::auth::CurrentUser;
use rwfw_core::error::AppError;

pub fn route() -> axum::routing::MethodRouter<AppState> {
    routing::delete(delete)
}

async fn delete(
    State(state): State<AppState>,
    user: CurrentUser,
    Path(id): Path<i32>,
) -> Response {
    if !user.can("blog.posts.manage") {
        return AppError::Forbidden("Missing permission: blog.posts.manage".into()).into_response();
    }

    let repo = PostRepository::new(state.db.clone());
    match repo.delete(id).await {
        Ok(()) => Redirect::to("/blog/admin/posts").into_response(),
        Err(error) => AppError::Internal(error).into_response(),
    }
}
"#
    .to_string()
}

fn blog_admin_post_edit_route_rs() -> String {
    r#"use crate::repositories::post_repo::PostRepository;
use crate::use_cases::save_post::{SavePostInput, SavePostUseCase};
use axum::extract::{Form, Path, State};
use axum::http::StatusCode;
use axum::response::{IntoResponse, Redirect, Response};
use axum::routing;
use rwfw_core::app::AppState;
use rwfw_core::auth::CurrentUser;
use rwfw_core::error::AppError;
use rwfw_core::view::View;

pub fn route() -> axum::routing::MethodRouter<AppState> {
    routing::get(get).post(put)
}

async fn get(
    State(state): State<AppState>,
    user: CurrentUser,
    v: View,
    Path(id): Path<i32>,
) -> Response {
    if !user.can("blog.posts.manage") {
        return AppError::Forbidden("Missing permission: blog.posts.manage".into()).into_response();
    }

    let repo = PostRepository::new(state.db.clone());
    let post = match repo.find_by_id(id).await {
        Ok(Some(post)) => post,
        Ok(None) => return AppError::NotFound(format!("Post not found: {id}")).into_response(),
        Err(error) => return AppError::Internal(error).into_response(),
    };

    v.render(
        "blog/admin/posts/edit",
        serde_json::json!({
            "post": post,
        }),
    )
}

// HTML forms can't issue PUT, so the edit form POSTs here.
async fn put(
    State(state): State<AppState>,
    user: CurrentUser,
    v: View,
    Path(id): Path<i32>,
    Form(input): Form<SavePostInput>,
) -> Response {
    if !user.can("blog.posts.manage") {
        return AppError::Forbidden("Missing permission: blog.posts.manage".into()).into_response();
    }

    let old = serde_json::json!({
        "id": id,
        "title": input.title,
        "slug": input.slug,
        "excerpt": input.excerpt,
        "body": input.body,
        "cover_image_url": input.cover_image_url,
        "status": input.status,
    });

    let repo = PostRepository::new(state.db.clone());
    let use_case = SavePostUseCase;
    match use_case.update(&repo, id, user.id, input).await {
        Ok(post) => Redirect::to(&format!("/blog/admin/posts/{}/edit", post.id)).into_response(),
        Err(AppError::Validation(errors)) => v.render_status(
            StatusCode::UNPROCESSABLE_ENTITY,
            "blog/admin/posts/edit",
            serde_json::json!({
                "post": old,
                "errors": rwfw_core::validation::first_messages(errors),
            }),
        ),
        Err(error) => error.into_response(),
    }
}
"#
    .to_string()
}

fn blog_index_page_template() -> String {
    r#"{% extends "layouts/app.html.j2" %}
{% block title %}Blog{% endblock %}
{% block content %}
<div class="max-w-4xl">
  <div class="flex justify-between items-center mb-8">
    <h1 class="text-3xl font-bold">Blog</h1>
    {% if auth and auth.user %}
    <a href="/blog/admin/posts" class="px-4 py-2 bg-blue-600 text-white rounded-md hover:bg-blue-700">Manage posts</a>
    {% endif %}
  </div>

  {% if posts | length == 0 %}
    <p class="text-gray-500">No published posts yet.</p>
  {% else %}
    <div class="space-y-4">
      {% for post in posts %}
      <article class="p-6 bg-white rounded-lg shadow-sm border">
        <a href="/blog/posts/{{ post.slug }}">
          <h2 class="text-xl font-semibold hover:text-blue-600 transition-colors">{{ post.title }}</h2>
        </a>
        <p class="mt-2 text-gray-600 line-clamp-2">{{ post.excerpt }}</p>
        <div class="mt-3 flex items-center gap-4 text-sm text-gray-400">
          <span>{{ post.published_at | default(post.created_at) }}</span>
        </div>
      </article>
      {% endfor %}
    </div>
  {% endif %}
</div>
{% endblock %}
"#
    .to_string()
}

fn blog_posts_index_page_template() -> String {
    r#"{% extends "layouts/app.html.j2" %}
{% block title %}Posts{% endblock %}
{% block content %}
<div class="max-w-4xl">
  <div class="flex justify-between items-center mb-8">
    <h1 class="text-3xl font-bold">Posts</h1>
    {% if auth and auth.user %}
    <a href="/blog/admin/posts/new" class="px-4 py-2 bg-blue-600 text-white rounded-md hover:bg-blue-700">New Post</a>
    {% endif %}
  </div>

  <div class="space-y-4">
    {% for post in posts %}
    <article class="p-6 bg-white rounded-lg shadow-sm border">
      <a href="/blog/posts/{{ post.slug }}">
        <h2 class="text-xl font-semibold hover:text-blue-600 transition-colors">{{ post.title }}</h2>
      </a>
      <p class="mt-2 text-gray-600 line-clamp-2">{{ post.excerpt }}</p>
      <div class="mt-3 flex items-center gap-4 text-sm text-gray-400">
        <span>{{ post.published_at | default(post.created_at) }}</span>
      </div>
    </article>
    {% endfor %}
  </div>
  {% if posts | length == 0 %}
    <p class="text-gray-500">No posts yet.</p>
  {% endif %}
</div>
{% endblock %}
"#
    .to_string()
}

fn blog_post_show_page_template() -> String {
    r#"{% extends "layouts/app.html.j2" %}
{% block title %}{{ post.title }}{% endblock %}
{% block content %}
<div class="max-w-3xl">
  <div class="mb-6">
    <a href="/blog/posts" class="text-blue-600 hover:underline text-sm">&larr; Back to posts</a>
  </div>

  <article class="bg-white rounded-lg shadow-sm border p-8">
    {% if post.cover_image_url %}
    <img src="{{ post.cover_image_url }}" alt="" class="mb-6 w-full rounded-lg object-cover">
    {% endif %}
    <h1 class="text-3xl font-bold mb-4">{{ post.title }}</h1>
    <div class="flex items-center gap-4 text-sm text-gray-400 mb-8">
      <span>{{ post.published_at | default(post.created_at) }}</span>
    </div>
    <p class="text-lg text-gray-600 mb-6">{{ post.excerpt }}</p>
    <div class="prose max-w-none">
      <p>{{ post.body }}</p>
    </div>
  </article>

  {% if auth and auth.user %}
  <div class="mt-6 flex gap-4">
    <a href="/blog/admin/posts/{{ post.id }}/edit" class="px-4 py-2 bg-gray-100 text-gray-700 rounded-md hover:bg-gray-200">Edit</a>
  </div>
  {% endif %}
</div>
{% endblock %}
"#
    .to_string()
}

fn blog_admin_posts_index_page_template() -> String {
    r#"{% extends "layouts/app.html.j2" %}
{% block title %}Manage Posts{% endblock %}
{% block content %}
<div class="max-w-4xl">
  <div class="flex justify-between items-center mb-8">
    <h1 class="text-3xl font-bold">Manage Posts</h1>
    <a href="/blog/admin/posts/new" class="px-4 py-2 bg-blue-600 text-white rounded-md hover:bg-blue-700">New Post</a>
  </div>

  {% if posts | length == 0 %}
    <p class="text-gray-500">No posts yet. Create the first one.</p>
  {% else %}
  <div class="overflow-hidden rounded-lg border bg-white shadow-sm">
    <table class="min-w-full divide-y divide-gray-200">
      <thead class="bg-gray-50">
        <tr>
          <th class="px-4 py-3 text-left text-xs font-medium uppercase tracking-wider text-gray-500">Title</th>
          <th class="px-4 py-3 text-left text-xs font-medium uppercase tracking-wider text-gray-500">Status</th>
          <th class="px-4 py-3 text-right text-xs font-medium uppercase tracking-wider text-gray-500">Actions</th>
        </tr>
      </thead>
      <tbody class="divide-y divide-gray-100">
        {% for post in posts %}
        <tr>
          <td class="px-4 py-3">
            <a href="/blog/admin/posts/{{ post.id }}/edit" class="font-medium text-gray-900 hover:text-blue-600">{{ post.title }}</a>
          </td>
          <td class="px-4 py-3 text-sm text-gray-500">{{ post.status }}</td>
          <td class="px-4 py-3 text-right">
            <a href="/blog/admin/posts/{{ post.id }}/edit" class="text-sm text-blue-600 hover:underline">Edit</a>
            <a href="/blog/admin/posts/{{ post.id }}" data-turbo-method="delete"
               data-turbo-confirm="Delete this post?"
               class="ml-4 text-sm text-red-600 hover:underline">Delete</a>
          </td>
        </tr>
        {% endfor %}
      </tbody>
    </table>
  </div>
  {% endif %}
</div>
{% endblock %}
"#
    .to_string()
}

fn blog_admin_post_form_template() -> String {
    r#"<form method="post" action="{{ action }}" class="bg-white rounded-lg shadow-sm border p-8 space-y-6">
  <input type="hidden" name="_csrf" value="{{ csrf_token }}">
  <x-field name="title" label="Title" :value="old.title | default(post.title) | default('')" />
  <x-field name="slug" label="Slug (optional)" :value="old.slug | default(post.slug) | default('')" :required="false" />
  <x-field name="excerpt" label="Excerpt" type="textarea" :value="old.excerpt | default(post.excerpt) | default('')" />
  <x-field name="body" label="Body" type="textarea" :value="old.body | default(post.body) | default('')" />
  <x-field name="cover_image_url" label="Cover image URL (optional)" :value="old.cover_image_url | default(post.cover_image_url) | default('')" :required="false" />
  <div>
    <label for="status" class="block text-sm font-medium text-gray-700">Status</label>
    {% set current_status = old.status | default(post.status) | default('draft') %}
    <select id="status" name="status"
            class="mt-1 block w-full rounded-md border border-gray-300 px-3 py-2 shadow-sm focus:border-blue-500 focus:outline-none focus:ring-1 focus:ring-blue-500">
      <option value="draft" {{ 'selected' if current_status == 'draft' }}>Draft</option>
      <option value="published" {{ 'selected' if current_status == 'published' }}>Published</option>
    </select>
    {% if errors.status %}<p class="mt-1 text-sm text-red-600">{{ errors.status }}</p>{% endif %}
  </div>
  <div class="flex gap-4">
    <button type="submit" data-turbo-submits-with="Saving..." class="px-6 py-2 bg-blue-600 text-white rounded-md hover:bg-blue-700 disabled:opacity-50">{{ submit_label }}</button>
    <a href="/blog/admin/posts" class="px-6 py-2 bg-gray-100 text-gray-700 rounded-md hover:bg-gray-200">Cancel</a>
  </div>
</form>
"#
    .to_string()
}

fn blog_admin_post_new_page_template() -> String {
    r#"{% extends "layouts/app.html.j2" %}
{% block title %}New Post{% endblock %}
{% block content %}
<div class="max-w-3xl">
  <div class="mb-6">
    <a href="/blog/admin/posts" class="text-blue-600 hover:underline text-sm">&larr; Back to posts</a>
  </div>
  <h1 class="text-3xl font-bold mb-8">New Post</h1>
  {% set action = "/blog/admin/posts" %}
  {% set submit_label = "Create Post" %}
  {% include "blog/admin/posts/_form.html.j2" %}
</div>
{% endblock %}
"#
    .to_string()
}

fn blog_admin_post_edit_page_template() -> String {
    r#"{% extends "layouts/app.html.j2" %}
{% block title %}Edit Post{% endblock %}
{% block content %}
<div class="max-w-3xl">
  <div class="mb-6">
    <a href="/blog/admin/posts" class="text-blue-600 hover:underline text-sm">&larr; Back to posts</a>
  </div>
  <h1 class="text-3xl font-bold mb-8">Edit Post</h1>
  {% set action = "/blog/admin/posts/" ~ post.id ~ "/edit" %}
  {% set submit_label = "Save Changes" %}
  {% include "blog/admin/posts/_form.html.j2" %}
</div>
{% endblock %}
"#
    .to_string()
}


// ---------------------------------------------------------------------------
// Ecommerce example (npm-free Hotwire): a `shop` module with products,
// categories, a cookie-backed server-rendered cart, and a fake checkout that
// persists orders. Mirrors the Hotwire blog example: `View` routes,
// `.html.j2` templates extending `layouts/app.html.j2`, form-based mutations
// with CSRF, and `data-turbo-method` links.
// ---------------------------------------------------------------------------

fn write_ecommerce_example(app_dir: &Path, context: &AppTemplateContext) -> anyhow::Result<()> {
    write_file(
        &app_dir.join("crates/modules/shop/Cargo.toml"),
        shop_cargo_toml(context),
    )?;
    write_file(
        &app_dir.join("crates/modules/shop/src/lib.rs"),
        shop_lib_rs(),
    )?;
    write_file(
        &app_dir.join("crates/modules/shop/src/cart.rs"),
        shop_cart_rs(),
    )?;
    write_file(
        &app_dir.join("crates/modules/shop/src/migrations/mod.rs"),
        shop_migrations_mod_rs(),
    )?;
    write_file(
        &app_dir.join("crates/modules/shop/src/migrations/20260101000000_create_shop_tables.sql"),
        shop_create_tables_sql(),
    )?;
    write_file(
        &app_dir.join("crates/modules/shop/src/models/mod.rs"),
        shop_models_mod_rs(),
    )?;
    write_file(
        &app_dir.join("crates/modules/shop/src/models/category.rs"),
        shop_category_model_rs(),
    )?;
    write_file(
        &app_dir.join("crates/modules/shop/src/models/product.rs"),
        shop_product_model_rs(),
    )?;
    write_file(
        &app_dir.join("crates/modules/shop/src/models/order.rs"),
        shop_order_model_rs(),
    )?;
    write_file(
        &app_dir.join("crates/modules/shop/src/models/order_item.rs"),
        shop_order_item_model_rs(),
    )?;
    write_file(
        &app_dir.join("crates/modules/shop/src/repositories/mod.rs"),
        shop_repositories_mod_rs(),
    )?;
    write_file(
        &app_dir.join("crates/modules/shop/src/repositories/shop_repo.rs"),
        shop_repository_rs(),
    )?;
    write_file(
        &app_dir.join("crates/modules/shop/src/use_cases/mod.rs"),
        shop_use_cases_mod_rs(),
    )?;
    write_file(
        &app_dir.join("crates/modules/shop/src/use_cases/checkout.rs"),
        shop_checkout_use_case_rs(),
    )?;
    write_file(
        &app_dir.join("crates/modules/shop/src/routes/index.rs"),
        shop_index_route_rs(),
    )?;
    write_file(
        &app_dir.join("crates/modules/shop/src/routes/products/[slug].rs"),
        shop_product_show_route_rs(),
    )?;
    write_file(
        &app_dir.join("crates/modules/shop/src/routes/cart/index.rs"),
        shop_cart_route_rs(),
    )?;
    write_file(
        &app_dir.join("crates/modules/shop/src/routes/cart/add.rs"),
        shop_cart_add_route_rs(),
    )?;
    write_file(
        &app_dir.join("crates/modules/shop/src/routes/cart/update.rs"),
        shop_cart_update_route_rs(),
    )?;
    write_file(
        &app_dir.join("crates/modules/shop/src/routes/checkout/index.rs"),
        shop_checkout_route_rs(),
    )?;
    write_file(
        &app_dir.join("crates/modules/shop/src/routes/orders/[number].rs"),
        shop_order_show_route_rs(),
    )?;
    write_file(
        &app_dir.join("crates/modules/shop/web/templates/index.html.j2"),
        shop_index_page_template(),
    )?;
    write_file(
        &app_dir.join("crates/modules/shop/web/templates/_product_card.html.j2"),
        shop_product_card_template(),
    )?;
    write_file(
        &app_dir.join("crates/modules/shop/web/templates/products/show.html.j2"),
        shop_product_show_page_template(),
    )?;
    write_file(
        &app_dir.join("crates/modules/shop/web/templates/cart/index.html.j2"),
        shop_cart_page_template(),
    )?;
    write_file(
        &app_dir.join("crates/modules/shop/web/templates/checkout/index.html.j2"),
        shop_checkout_page_template(),
    )?;
    write_file(
        &app_dir.join("crates/modules/shop/web/templates/orders/show.html.j2"),
        shop_order_show_page_template(),
    )?;

    Ok(())
}

fn shop_cargo_toml(context: &AppTemplateContext) -> String {
    format!(
        r#"[package]
name = "mod-shop"
version.workspace = true
edition.workspace = true

[dependencies]
rwfw-core = {}
rwfw-shared = {}
rwfw-macros = {}
axum = {{ workspace = true }}
async-trait = {{ workspace = true }}
inventory = {{ workspace = true }}
serde = {{ workspace = true }}
serde_json = {{ workspace = true }}
sea-orm = {{ workspace = true }}
chrono = {{ workspace = true }}
tracing = {{ workspace = true }}
anyhow = {{ workspace = true }}
uuid = {{ workspace = true }}
"#,
        context.rwfw_core_dep, context.rwfw_shared_dep, context.rwfw_macros_dep
    )
}

fn shop_lib_rs() -> String {
    r#"pub mod cart;
pub mod migrations;
pub mod models;
pub mod repositories;
pub mod use_cases;

use axum::Router;
use rwfw_core::app::AppState;
use rwfw_core::auth::Permission;
use rwfw_core::module::{Module, ModuleRegistration, NavItem};

#[rwfw_macros::rwfw_routes("src/routes")]
pub struct ShopRoutes;

pub struct ShopModule;

impl ShopModule {
    pub fn new() -> Self {
        Self
    }
}

#[async_trait::async_trait]
impl Module for ShopModule {
    fn name(&self) -> &str {
        "shop"
    }

    fn routes(&self) -> Router<AppState> {
        ShopRoutes::generated_routes()
    }

    fn migrations(&self) -> Vec<rwfw_core::migration::Migration> {
        migrations::migrations()
    }

    fn permissions(&self) -> Vec<Permission> {
        vec![Permission::new("shop.orders.view", "View shop orders")]
    }

    fn nav_items(&self) -> Vec<NavItem> {
        vec![
            NavItem {
                label: "Shop".to_string(),
                href: "/shop".to_string(),
                icon: Some("store".to_string()),
            },
            NavItem {
                label: "Cart".to_string(),
                href: "/shop/cart".to_string(),
                icon: Some("cart".to_string()),
            },
        ]
    }

    fn web_root(&self) -> Option<std::path::PathBuf> {
        Some(std::path::PathBuf::from(env!("CARGO_MANIFEST_DIR")).join("web"))
    }
}

inventory::submit! {
    ModuleRegistration::new("shop", || Box::new(ShopModule::new()))
}
"#
    .to_string()
}

fn shop_cart_rs() -> String {
    r#"//! Cookie-backed, server-rendered cart. The cart lives entirely on the server
//! side as a small `rwfw_cart` cookie holding `product_id:quantity` pairs, so
//! there is no client-side JS state (no localStorage, no React store). Every
//! mutation is a form POST that rewrites the cookie and redirects.

use axum::http::header::{COOKIE, SET_COOKIE};
use axum::http::{HeaderMap, HeaderValue};
use std::collections::BTreeMap;

pub const CART_COOKIE: &str = "rwfw_cart";
const MAX_QUANTITY: i32 = 20;

/// Parse the cart cookie into an ordered map of `product_id -> quantity`.
pub fn parse(headers: &HeaderMap) -> BTreeMap<i32, i32> {
    let mut cart = BTreeMap::new();
    let Some(raw) = cookie_value(headers, CART_COOKIE) else {
        return cart;
    };

    for pair in raw.split(',') {
        let Some((id, qty)) = pair.split_once(':') else {
            continue;
        };
        let (Ok(id), Ok(qty)) = (id.trim().parse::<i32>(), qty.trim().parse::<i32>()) else {
            continue;
        };
        if id <= 0 || qty <= 0 {
            continue;
        }
        cart.insert(id, qty.min(MAX_QUANTITY));
    }

    cart
}

/// Add (or increment) a product in the cart and return the updated map.
pub fn add(mut cart: BTreeMap<i32, i32>, product_id: i32, quantity: i32) -> BTreeMap<i32, i32> {
    if product_id <= 0 || quantity <= 0 {
        return cart;
    }
    let entry = cart.entry(product_id).or_insert(0);
    *entry = (*entry + quantity).min(MAX_QUANTITY);
    cart
}

/// Set an explicit quantity for a product. A quantity of `0` removes it.
pub fn set_quantity(
    mut cart: BTreeMap<i32, i32>,
    product_id: i32,
    quantity: i32,
) -> BTreeMap<i32, i32> {
    if product_id <= 0 {
        return cart;
    }
    if quantity <= 0 {
        cart.remove(&product_id);
    } else {
        cart.insert(product_id, quantity.min(MAX_QUANTITY));
    }
    cart
}

/// Serialize the cart back into a `Set-Cookie` header value.
pub fn cookie(cart: &BTreeMap<i32, i32>) -> Option<HeaderValue> {
    let value = if cart.is_empty() {
        format!("{CART_COOKIE}=; Path=/; Max-Age=0; SameSite=Lax")
    } else {
        let body = cart
            .iter()
            .map(|(id, qty)| format!("{id}:{qty}"))
            .collect::<Vec<_>>()
            .join(",");
        format!("{CART_COOKIE}={body}; Path=/; Max-Age=2592000; SameSite=Lax")
    };
    value.parse().ok()
}

/// Attach the serialized cart cookie to a response.
pub fn apply(response: &mut axum::response::Response, cart: &BTreeMap<i32, i32>) {
    if let Some(value) = cookie(cart) {
        response.headers_mut().append(SET_COOKIE, value);
    }
}

fn cookie_value<'a>(headers: &'a HeaderMap, name: &str) -> Option<&'a str> {
    let header = headers.get(COOKIE)?.to_str().ok()?;
    header.split(';').find_map(|cookie| {
        let (key, value) = cookie.trim().split_once('=')?;
        (key == name).then_some(value)
    })
}
"#
    .to_string()
}

fn shop_migrations_mod_rs() -> String {
    r#"use rwfw_core::migration::Migration;

pub fn migrations() -> Vec<Migration> {
    vec![
        Migration::new(
            "20260101000000",
            "create_shop_tables",
            include_str!("20260101000000_create_shop_tables.sql"),
        ),
    ]
}
"#
    .to_string()
}

fn shop_create_tables_sql() -> String {
    r#"CREATE SCHEMA IF NOT EXISTS shop;

CREATE TABLE IF NOT EXISTS shop.categories (
    id SERIAL PRIMARY KEY,
    name VARCHAR(255) NOT NULL,
    slug VARCHAR(255) NOT NULL UNIQUE,
    description TEXT NOT NULL,
    created_at TIMESTAMPTZ NOT NULL DEFAULT NOW(),
    updated_at TIMESTAMPTZ NOT NULL DEFAULT NOW()
);

CREATE TABLE IF NOT EXISTS shop.products (
    id SERIAL PRIMARY KEY,
    category_id INTEGER REFERENCES shop.categories(id) ON DELETE SET NULL,
    name VARCHAR(255) NOT NULL,
    slug VARCHAR(255) NOT NULL UNIQUE,
    description TEXT NOT NULL,
    price_cents INTEGER NOT NULL,
    currency VARCHAR(3) NOT NULL DEFAULT 'USD',
    image_url VARCHAR(500) NOT NULL,
    inventory INTEGER NOT NULL DEFAULT 0,
    featured BOOLEAN NOT NULL DEFAULT FALSE,
    active BOOLEAN NOT NULL DEFAULT TRUE,
    created_at TIMESTAMPTZ NOT NULL DEFAULT NOW(),
    updated_at TIMESTAMPTZ NOT NULL DEFAULT NOW()
);

CREATE TABLE IF NOT EXISTS shop.orders (
    id SERIAL PRIMARY KEY,
    number VARCHAR(64) NOT NULL UNIQUE,
    customer_name VARCHAR(255) NOT NULL,
    customer_email VARCHAR(255) NOT NULL,
    address_line VARCHAR(500) NOT NULL,
    city VARCHAR(255) NOT NULL,
    country VARCHAR(255) NOT NULL,
    total_cents INTEGER NOT NULL,
    currency VARCHAR(3) NOT NULL DEFAULT 'USD',
    status VARCHAR(64) NOT NULL DEFAULT 'paid_fake',
    created_at TIMESTAMPTZ NOT NULL DEFAULT NOW(),
    updated_at TIMESTAMPTZ NOT NULL DEFAULT NOW()
);

CREATE TABLE IF NOT EXISTS shop.order_items (
    id SERIAL PRIMARY KEY,
    order_id INTEGER NOT NULL REFERENCES shop.orders(id) ON DELETE CASCADE,
    product_id INTEGER,
    product_name VARCHAR(255) NOT NULL,
    product_slug VARCHAR(255) NOT NULL,
    unit_price_cents INTEGER NOT NULL,
    quantity INTEGER NOT NULL,
    subtotal_cents INTEGER NOT NULL,
    created_at TIMESTAMPTZ NOT NULL DEFAULT NOW()
);

CREATE INDEX IF NOT EXISTS idx_shop_products_active_featured
    ON shop.products (active, featured);

INSERT INTO shop.categories (name, slug, description)
VALUES
    ('Desk', 'desk', 'Objects for focused workspaces.'),
    ('Carry', 'carry', 'Bags and daily tools for moving between contexts.'),
    ('Sound', 'sound', 'Audio gear for deep work.')
ON CONFLICT (slug) DO NOTHING;

INSERT INTO shop.products
    (category_id, name, slug, description, price_cents, currency, image_url, inventory, featured, active)
VALUES
    (
        (SELECT id FROM shop.categories WHERE slug = 'desk'),
        'Machined Keyboard Tray',
        'machined-keyboard-tray',
        'A low-profile aluminum tray that keeps your keyboard and notes aligned for long work sessions.',
        12900,
        'USD',
        'https://images.unsplash.com/photo-1516321318423-f06f85e504b3?auto=format&fit=crop&w=1200&q=80',
        18,
        TRUE,
        TRUE
    ),
    (
        (SELECT id FROM shop.categories WHERE slug = 'desk'),
        'Task Lamp One',
        'task-lamp-one',
        'Warm directional light with a heavy base and a single mechanical hinge.',
        18900,
        'USD',
        'https://images.unsplash.com/photo-1507473885765-e6ed057f782c?auto=format&fit=crop&w=1200&q=80',
        9,
        TRUE,
        TRUE
    ),
    (
        (SELECT id FROM shop.categories WHERE slug = 'carry'),
        'Field Pack 24L',
        'field-pack-24l',
        'A weather resistant everyday pack with structured compartments for laptop, camera and cables.',
        24000,
        'USD',
        'https://images.unsplash.com/photo-1622560480605-d83c853bc5c3?auto=format&fit=crop&w=1200&q=80',
        12,
        TRUE,
        TRUE
    ),
    (
        (SELECT id FROM shop.categories WHERE slug = 'sound'),
        'Studio Monitor Headphones',
        'studio-monitor-headphones',
        'Closed-back headphones tuned for clear calls, editing and focused work.',
        16000,
        'USD',
        'https://images.unsplash.com/photo-1505740420928-5e560c06d30e?auto=format&fit=crop&w=1200&q=80',
        24,
        FALSE,
        TRUE
    )
ON CONFLICT (slug) DO NOTHING;
"#
    .to_string()
}

fn shop_models_mod_rs() -> String {
    r#"pub mod category;
pub mod order;
pub mod order_item;
pub mod product;

pub use category::Category;
pub use order::Order;
pub use order_item::OrderItem;
pub use product::Product;
"#
    .to_string()
}

fn shop_category_model_rs() -> String {
    r#"use sea_orm::entity::prelude::*;
use serde::{Deserialize, Serialize};

#[derive(Clone, Debug, PartialEq, DeriveEntityModel, Serialize, Deserialize)]
#[sea_orm(table_name = "categories", schema_name = "shop")]
pub struct Model {
    #[sea_orm(primary_key)]
    pub id: i32,
    pub name: String,
    pub slug: String,
    #[sea_orm(column_type = "Text")]
    pub description: String,
    pub created_at: DateTimeWithTimeZone,
    pub updated_at: DateTimeWithTimeZone,
}

#[derive(Copy, Clone, Debug, EnumIter, DeriveRelation)]
pub enum Relation {}

impl ActiveModelBehavior for ActiveModel {}

pub type Category = Model;
"#
    .to_string()
}

fn shop_product_model_rs() -> String {
    r#"use sea_orm::entity::prelude::*;
use serde::{Deserialize, Serialize};

#[derive(Clone, Debug, PartialEq, DeriveEntityModel, Serialize, Deserialize)]
#[sea_orm(table_name = "products", schema_name = "shop")]
pub struct Model {
    #[sea_orm(primary_key)]
    pub id: i32,
    pub category_id: Option<i32>,
    pub name: String,
    pub slug: String,
    #[sea_orm(column_type = "Text")]
    pub description: String,
    pub price_cents: i32,
    pub currency: String,
    pub image_url: String,
    pub inventory: i32,
    pub featured: bool,
    pub active: bool,
    pub created_at: DateTimeWithTimeZone,
    pub updated_at: DateTimeWithTimeZone,
}

#[derive(Copy, Clone, Debug, EnumIter, DeriveRelation)]
pub enum Relation {}

impl ActiveModelBehavior for ActiveModel {}

pub type Product = Model;
"#
    .to_string()
}

fn shop_order_model_rs() -> String {
    r#"use sea_orm::entity::prelude::*;
use serde::{Deserialize, Serialize};

#[derive(Clone, Debug, PartialEq, DeriveEntityModel, Serialize, Deserialize)]
#[sea_orm(table_name = "orders", schema_name = "shop")]
pub struct Model {
    #[sea_orm(primary_key)]
    pub id: i32,
    pub number: String,
    pub customer_name: String,
    pub customer_email: String,
    pub address_line: String,
    pub city: String,
    pub country: String,
    pub total_cents: i32,
    pub currency: String,
    pub status: String,
    pub created_at: DateTimeWithTimeZone,
    pub updated_at: DateTimeWithTimeZone,
}

#[derive(Copy, Clone, Debug, EnumIter, DeriveRelation)]
pub enum Relation {}

impl ActiveModelBehavior for ActiveModel {}

pub type Order = Model;
"#
    .to_string()
}

fn shop_order_item_model_rs() -> String {
    r#"use sea_orm::entity::prelude::*;
use serde::{Deserialize, Serialize};

#[derive(Clone, Debug, PartialEq, DeriveEntityModel, Serialize, Deserialize)]
#[sea_orm(table_name = "order_items", schema_name = "shop")]
pub struct Model {
    #[sea_orm(primary_key)]
    pub id: i32,
    pub order_id: i32,
    pub product_id: Option<i32>,
    pub product_name: String,
    pub product_slug: String,
    pub unit_price_cents: i32,
    pub quantity: i32,
    pub subtotal_cents: i32,
    pub created_at: DateTimeWithTimeZone,
}

#[derive(Copy, Clone, Debug, EnumIter, DeriveRelation)]
pub enum Relation {}

impl ActiveModelBehavior for ActiveModel {}

pub type OrderItem = Model;
"#
    .to_string()
}

fn shop_repositories_mod_rs() -> String {
    r#"pub mod shop_repo;
"#
    .to_string()
}

fn shop_repository_rs() -> String {
    r#"use crate::models::category::{self, Column as CategoryColumn, Entity as CategoryEntity};
use crate::models::order::{self, ActiveModel as OrderActiveModel, Column as OrderColumn, Entity as OrderEntity};
use crate::models::order_item::{self, ActiveModel as OrderItemActiveModel, Column as OrderItemColumn, Entity as OrderItemEntity};
use crate::models::product::{self, Column as ProductColumn, Entity as ProductEntity};
use sea_orm::*;

pub struct ShopRepository {
    db: DatabaseConnection,
}

#[derive(Debug)]
pub struct CheckoutProduct {
    pub product: product::Model,
    pub quantity: i32,
}

#[derive(Debug)]
pub struct CreateOrder {
    pub number: String,
    pub customer_name: String,
    pub customer_email: String,
    pub address_line: String,
    pub city: String,
    pub country: String,
    pub total_cents: i32,
    pub currency: String,
    pub items: Vec<CheckoutProduct>,
}

impl ShopRepository {
    pub fn new(db: DatabaseConnection) -> Self {
        Self { db }
    }

    pub async fn categories(&self) -> anyhow::Result<Vec<category::Model>> {
        Ok(CategoryEntity::find()
            .order_by_asc(CategoryColumn::Name)
            .all(&self.db)
            .await?)
    }

    pub async fn featured_products(&self) -> anyhow::Result<Vec<product::Model>> {
        Ok(ProductEntity::find()
            .filter(ProductColumn::Active.eq(true))
            .filter(ProductColumn::Featured.eq(true))
            .order_by_desc(ProductColumn::CreatedAt)
            .all(&self.db)
            .await?)
    }

    pub async fn products(
        &self,
        search: Option<&str>,
        category_slug: Option<&str>,
    ) -> anyhow::Result<Vec<product::Model>> {
        let mut query = ProductEntity::find()
            .filter(ProductColumn::Active.eq(true))
            .order_by_desc(ProductColumn::Featured)
            .order_by_asc(ProductColumn::Name);

        if let Some(search) = search.map(str::trim).filter(|value| !value.is_empty()) {
            query = query.filter(
                Condition::any()
                    .add(ProductColumn::Name.contains(search))
                    .add(ProductColumn::Description.contains(search)),
            );
        }

        if let Some(slug) = category_slug.map(str::trim).filter(|value| !value.is_empty()) {
            let Some(category) = CategoryEntity::find()
                .filter(CategoryColumn::Slug.eq(slug))
                .one(&self.db)
                .await?
            else {
                return Ok(Vec::new());
            };
            query = query.filter(ProductColumn::CategoryId.eq(category.id));
        }

        Ok(query.all(&self.db).await?)
    }

    pub async fn find_product_by_slug(&self, slug: &str) -> anyhow::Result<Option<product::Model>> {
        Ok(ProductEntity::find()
            .filter(ProductColumn::Slug.eq(slug))
            .filter(ProductColumn::Active.eq(true))
            .one(&self.db)
            .await?)
    }

    pub async fn find_products_by_ids(&self, ids: &[i32]) -> anyhow::Result<Vec<product::Model>> {
        if ids.is_empty() {
            return Ok(Vec::new());
        }

        Ok(ProductEntity::find()
            .filter(ProductColumn::Id.is_in(ids.to_vec()))
            .filter(ProductColumn::Active.eq(true))
            .all(&self.db)
            .await?)
    }

    pub async fn create_order(&self, data: CreateOrder) -> anyhow::Result<order::Model> {
        let txn = self.db.begin().await?;
        let now = chrono::Utc::now().fixed_offset();
        let order = OrderActiveModel {
            number: Set(data.number),
            customer_name: Set(data.customer_name),
            customer_email: Set(data.customer_email),
            address_line: Set(data.address_line),
            city: Set(data.city),
            country: Set(data.country),
            total_cents: Set(data.total_cents),
            currency: Set(data.currency),
            status: Set("paid_fake".to_string()),
            created_at: Set(now),
            updated_at: Set(now),
            ..Default::default()
        }
        .insert(&txn)
        .await?;

        for item in data.items {
            let subtotal = item.product.price_cents * item.quantity;
            OrderItemActiveModel {
                order_id: Set(order.id),
                product_id: Set(Some(item.product.id)),
                product_name: Set(item.product.name),
                product_slug: Set(item.product.slug),
                unit_price_cents: Set(item.product.price_cents),
                quantity: Set(item.quantity),
                subtotal_cents: Set(subtotal),
                created_at: Set(now),
                ..Default::default()
            }
            .insert(&txn)
            .await?;
        }

        txn.commit().await?;
        Ok(order)
    }

    pub async fn find_order(
        &self,
        number: &str,
    ) -> anyhow::Result<Option<(order::Model, Vec<order_item::Model>)>> {
        let Some(order) = OrderEntity::find()
            .filter(OrderColumn::Number.eq(number))
            .one(&self.db)
            .await?
        else {
            return Ok(None);
        };

        let items = OrderItemEntity::find()
            .filter(OrderItemColumn::OrderId.eq(order.id))
            .order_by_asc(OrderItemColumn::Id)
            .all(&self.db)
            .await?;

        Ok(Some((order, items)))
    }
}
"#
    .to_string()
}

fn shop_use_cases_mod_rs() -> String {
    r#"pub mod checkout;
"#
    .to_string()
}

fn shop_checkout_use_case_rs() -> String {
    r#"use crate::models::order;
use crate::repositories::shop_repo::{CheckoutProduct, CreateOrder, ShopRepository};
use rwfw_core::error::AppError;
use serde::Deserialize;
use std::collections::{BTreeMap, HashMap};

#[derive(Debug, Deserialize)]
pub struct CheckoutInput {
    pub customer_name: String,
    pub customer_email: String,
    pub address_line: String,
    pub city: String,
    pub country: String,
}

pub struct CheckoutUseCase;

impl CheckoutUseCase {
    /// Run a fake checkout: validate the buyer fields, re-price the cart against
    /// live product data, check inventory, and persist the order. The cart is the
    /// server-side cookie map (`product_id -> quantity`) the route already parsed.
    pub async fn execute(
        &self,
        repo: &ShopRepository,
        input: CheckoutInput,
        cart: &BTreeMap<i32, i32>,
    ) -> Result<order::Model, AppError> {
        let customer_name = input.customer_name.trim().to_string();
        let customer_email = input.customer_email.trim().to_string();
        let address_line = input.address_line.trim().to_string();
        let city = input.city.trim().to_string();
        let country = input.country.trim().to_string();

        let mut errors = HashMap::new();
        if customer_name.is_empty() {
            errors.insert("customer_name".to_string(), vec!["Name is required".to_string()]);
        }
        if !customer_email.contains('@') {
            errors.insert("customer_email".to_string(), vec!["Valid email is required".to_string()]);
        }
        if address_line.is_empty() {
            errors.insert("address_line".to_string(), vec!["Address is required".to_string()]);
        }
        if city.is_empty() {
            errors.insert("city".to_string(), vec!["City is required".to_string()]);
        }
        if country.is_empty() {
            errors.insert("country".to_string(), vec!["Country is required".to_string()]);
        }

        let mut quantities = BTreeMap::new();
        for (product_id, quantity) in cart {
            if *quantity > 0 {
                quantities.insert(*product_id, (*quantity).min(20));
            }
        }

        if quantities.is_empty() {
            errors.insert("items".to_string(), vec!["Cart is empty".to_string()]);
        }

        if !errors.is_empty() {
            return Err(AppError::Validation(errors));
        }

        let ids = quantities.keys().copied().collect::<Vec<_>>();
        let products = repo
            .find_products_by_ids(&ids)
            .await
            .map_err(AppError::Internal)?;

        if products.len() != ids.len() {
            return Err(AppError::BadRequest("Cart contains unavailable products".into()));
        }

        let mut checkout_items = Vec::new();
        let mut total_cents = 0;
        let mut currency = "USD".to_string();

        for product in products {
            let quantity = quantities.get(&product.id).copied().unwrap_or(0);
            if quantity <= 0 {
                continue;
            }
            if product.inventory < quantity {
                return Err(AppError::BadRequest(format!(
                    "{} has only {} item(s) in stock",
                    product.name, product.inventory
                )));
            }
            currency = product.currency.clone();
            total_cents += product.price_cents * quantity;
            checkout_items.push(CheckoutProduct { product, quantity });
        }

        let raw = uuid::Uuid::new_v4().simple().to_string();
        let number = format!("RW-{}", raw[..10].to_uppercase());

        repo.create_order(CreateOrder {
            number,
            customer_name,
            customer_email,
            address_line,
            city,
            country,
            total_cents,
            currency,
            items: checkout_items,
        })
        .await
        .map_err(AppError::Internal)
    }
}
"#
    .to_string()
}

fn scaffold_model_template() -> String {
    r#"use sea_orm::entity::prelude::*;
use serde::{Deserialize, Serialize};

#[derive(Clone, Debug, PartialEq, DeriveEntityModel, Serialize, Deserialize)]
#[sea_orm(table_name = "{{ table }}", schema_name = "{{ module }}")]
pub struct Model {
    #[sea_orm(primary_key)]
    pub id: i32,
{% for field in fields %}{% if field.is_text %}    #[sea_orm(column_type = "Text")]
{% endif %}    pub {{ field.name }}: {{ field.rust_type }},
{% endfor %}    pub created_at: DateTimeWithTimeZone,
    pub updated_at: DateTimeWithTimeZone,
}

#[derive(Copy, Clone, Debug, EnumIter, DeriveRelation)]
pub enum Relation {}

impl ActiveModelBehavior for ActiveModel {}

pub type {{ name_pascal }} = Model;
"#
    .to_string()
}

fn scaffold_migration_template() -> String {
    r#"CREATE SCHEMA IF NOT EXISTS {{ module }};

CREATE TABLE IF NOT EXISTS {{ module }}.{{ table }} (
    id SERIAL PRIMARY KEY,
{% for field in fields %}    {{ field.name }} {{ field.sql_type }},
{% endfor %}    created_at TIMESTAMPTZ NOT NULL DEFAULT NOW(),
    updated_at TIMESTAMPTZ NOT NULL DEFAULT NOW()
);
"#
    .to_string()
}

fn scaffold_repository_template() -> String {
    r#"use crate::models::{{ name_snake }}::{self, ActiveModel, Column, Entity as {{ name_pascal }}Entity};
use sea_orm::*;

pub struct {{ name_pascal }}Repository {
    db: DatabaseConnection,
}

#[derive(Debug)]
pub struct Create{{ name_pascal }} {
{% for field in fields %}    pub {{ field.name }}: {{ field.owned_rust_type }},
{% endfor %}}

#[derive(Debug)]
pub struct Update{{ name_pascal }} {
{% for field in fields %}    pub {{ field.name }}: {{ field.owned_rust_type }},
{% endfor %}}

impl {{ name_pascal }}Repository {
    pub fn new(db: DatabaseConnection) -> Self {
        Self { db }
    }

    pub async fn find_by_id(&self, id: i32) -> anyhow::Result<Option<{{ name_snake }}::Model>> {
        Ok({{ name_pascal }}Entity::find_by_id(id).one(&self.db).await?)
    }

    pub async fn find_all(
        &self,
        page: u64,
        per_page: u64,
        search: Option<&str>,
    ) -> anyhow::Result<(Vec<{{ name_snake }}::Model>, u64)> {
        let mut query = {{ name_pascal }}Entity::find().order_by_desc(Column::CreatedAt);
{% if has_searchable %}
        if let Some(search) = search.map(str::trim).filter(|value| !value.is_empty()) {
            let mut conditions = Condition::any();
{% for field in fields %}{% if field.is_searchable %}            conditions = conditions.add(Column::{{ field.column_variant }}.contains(search));
{% endif %}{% endfor %}            query = query.filter(conditions);
        }
{% else %}
        let _ = search;
{% endif %}
        let paginator = query.paginate(&self.db, per_page);

        let total = paginator.num_items().await?;
        let items = paginator.fetch_page(page.saturating_sub(1)).await?;
        Ok((items, total))
    }

    pub async fn create(&self, data: Create{{ name_pascal }}) -> anyhow::Result<{{ name_snake }}::Model> {
        let now = chrono::Utc::now().fixed_offset();
        let model = ActiveModel {
{% for field in fields %}            {{ field.name }}: Set(data.{{ field.name }}),
{% endfor %}            created_at: Set(now),
            updated_at: Set(now),
            ..Default::default()
        };
        Ok(model.insert(&self.db).await?)
    }

    pub async fn update(&self, id: i32, data: Update{{ name_pascal }}) -> anyhow::Result<{{ name_snake }}::Model> {
        let item = {{ name_pascal }}Entity::find_by_id(id)
            .one(&self.db)
            .await?
            .ok_or_else(|| anyhow::anyhow!("{{ name_pascal }} not found"))?;

        let mut active: ActiveModel = item.into();
{% for field in fields %}        active.{{ field.name }} = Set(data.{{ field.name }});
{% endfor %}        active.updated_at = Set(chrono::Utc::now().fixed_offset());
        Ok(active.update(&self.db).await?)
    }

    pub async fn delete(&self, id: i32) -> anyhow::Result<()> {
        {{ name_pascal }}Entity::delete_by_id(id).exec(&self.db).await?;
        Ok(())
    }
}
"#
    .to_string()
}

fn scaffold_use_case_create_template() -> String {
    r#"use crate::models::{{ name_snake }};
use crate::repositories::{{ name_snake }}_repo::{Create{{ name_pascal }}, {{ name_pascal }}Repository};
use rwfw_core::error::AppError;
use std::collections::HashMap;

#[derive(Debug, serde::Serialize, serde::Deserialize)]
pub struct Create{{ name_pascal }}Input {
{% for field in fields %}{% if field.is_bool %}    #[serde(default, deserialize_with = "rwfw_core::forms::checkbox")]
{% endif %}    pub {{ field.name }}: {{ field.input_rust_type }},
{% endfor %}}

pub struct Create{{ name_pascal }}Output {
    pub {{ name_snake }}: {{ name_snake }}::Model,
}

pub struct Create{{ name_pascal }}UseCase;

impl Create{{ name_pascal }}UseCase {
    pub async fn execute(
        &self,
        repo: &{{ name_pascal }}Repository,
        input: Create{{ name_pascal }}Input,
    ) -> Result<Create{{ name_pascal }}Output, AppError> {
        validate_input(&input)?;
{% for field in fields %}{% if field.optional and field.is_bool %}        let {{ field.name }} = Some(input.{{ field.name }});
{% elif field.optional and field.is_number %}        let {{ field.name }} = parse_optional_i32(&input.{{ field.name }}, "{{ field.name }}", "{{ field.title }}")?;
{% elif field.optional and field.is_decimal %}        let {{ field.name }} = parse_optional_f64(&input.{{ field.name }}, "{{ field.name }}", "{{ field.title }}")?;
{% elif field.optional and field.is_uuid %}        let {{ field.name }} = parse_optional_uuid(&input.{{ field.name }}, "{{ field.name }}", "{{ field.title }}")?;
{% elif field.optional and field.is_date %}        let {{ field.name }} = parse_optional_date(&input.{{ field.name }}, "{{ field.name }}", "{{ field.title }}")?;
{% elif field.optional and field.is_datetime %}        let {{ field.name }} = parse_optional_datetime(&input.{{ field.name }}, "{{ field.name }}", "{{ field.title }}")?;
{% elif field.optional %}        let {{ field.name }} = optional_string(&input.{{ field.name }});
{% elif field.is_bool %}        let {{ field.name }} = input.{{ field.name }};
{% elif field.is_number %}        let {{ field.name }} = parse_i32(&input.{{ field.name }}, "{{ field.name }}", "{{ field.title }}")?;
{% elif field.is_decimal %}        let {{ field.name }} = parse_f64(&input.{{ field.name }}, "{{ field.name }}", "{{ field.title }}")?;
{% elif field.is_uuid %}        let {{ field.name }} = parse_uuid(&input.{{ field.name }}, "{{ field.name }}", "{{ field.title }}")?;
{% elif field.is_date %}        let {{ field.name }} = parse_date(&input.{{ field.name }}, "{{ field.name }}", "{{ field.title }}")?;
{% elif field.is_datetime %}        let {{ field.name }} = parse_datetime(&input.{{ field.name }}, "{{ field.name }}", "{{ field.title }}")?;
{% else %}        let {{ field.name }} = input.{{ field.name }}.trim().to_string();
{% endif %}{% endfor %}
        let {{ name_snake }} = repo
            .create(Create{{ name_pascal }} {
{% for field in fields %}                {{ field.name }},
{% endfor %}            })
            .await
            .map_err(AppError::Internal)?;

        tracing::info!({{ name_snake }}_id = %{{ name_snake }}.id, "{{ name_pascal }} created");
        Ok(Create{{ name_pascal }}Output { {{ name_snake }} })
    }
}

fn validate_input(input: &Create{{ name_pascal }}Input) -> Result<(), AppError> {
    let mut errors = HashMap::new();
{% for field in fields %}{% if not field.optional and not field.is_bool %}    if input.{{ field.name }}.trim().is_empty() {
        errors.insert("{{ field.name }}".to_string(), vec!["{{ field.title }} is required".to_string()]);
    }
{% endif %}{% endfor %}
    if errors.is_empty() {
        Ok(())
    } else {
        Err(AppError::Validation(errors))
    }
}
{% if has_number or has_decimal or has_uuid or has_date or has_datetime %}
fn validation_error(field: &str, message: impl Into<String>) -> AppError {
    let mut errors = HashMap::new();
    errors.insert(field.to_string(), vec![message.into()]);
    AppError::Validation(errors)
}
{% endif %}{% if has_number %}
fn parse_i32(value: &str, field: &str, label: &str) -> Result<i32, AppError> {
    value
        .trim()
        .parse::<i32>()
        .map_err(|_| validation_error(field, format!("{label} must be a valid integer")))
}
{% endif %}{% if has_optional_number %}
fn parse_optional_i32(value: &str, field: &str, label: &str) -> Result<Option<i32>, AppError> {
    if value.trim().is_empty() {
        Ok(None)
    } else {
        parse_i32(value, field, label).map(Some)
    }
}
{% endif %}{% if has_decimal %}
fn parse_f64(value: &str, field: &str, label: &str) -> Result<f64, AppError> {
    value
        .trim()
        .parse::<f64>()
        .map_err(|_| validation_error(field, format!("{label} must be a valid decimal number")))
}
{% endif %}{% if has_optional_decimal %}
fn parse_optional_f64(value: &str, field: &str, label: &str) -> Result<Option<f64>, AppError> {
    if value.trim().is_empty() {
        Ok(None)
    } else {
        parse_f64(value, field, label).map(Some)
    }
}
{% endif %}{% if has_uuid %}
fn parse_uuid(value: &str, field: &str, label: &str) -> Result<uuid::Uuid, AppError> {
    value
        .trim()
        .parse::<uuid::Uuid>()
        .map_err(|_| validation_error(field, format!("{label} must be a valid UUID")))
}
{% endif %}{% if has_optional_uuid %}
fn parse_optional_uuid(value: &str, field: &str, label: &str) -> Result<Option<uuid::Uuid>, AppError> {
    if value.trim().is_empty() {
        Ok(None)
    } else {
        parse_uuid(value, field, label).map(Some)
    }
}
{% endif %}{% if has_date %}
fn parse_date(value: &str, field: &str, label: &str) -> Result<chrono::NaiveDate, AppError> {
    chrono::NaiveDate::parse_from_str(value.trim(), "%Y-%m-%d")
        .map_err(|_| validation_error(field, format!("{label} must be a valid date")))
}
{% endif %}{% if has_optional_date %}
fn parse_optional_date(
    value: &str,
    field: &str,
    label: &str,
) -> Result<Option<chrono::NaiveDate>, AppError> {
    if value.trim().is_empty() {
        Ok(None)
    } else {
        parse_date(value, field, label).map(Some)
    }
}
{% endif %}{% if has_datetime %}
fn parse_datetime(
    value: &str,
    field: &str,
    label: &str,
) -> Result<chrono::DateTime<chrono::FixedOffset>, AppError> {
    if let Ok(datetime) = chrono::DateTime::parse_from_rfc3339(value.trim()) {
        return Ok(datetime);
    }

    chrono::NaiveDateTime::parse_from_str(value.trim(), "%Y-%m-%dT%H:%M")
        .or_else(|_| chrono::NaiveDateTime::parse_from_str(value.trim(), "%Y-%m-%dT%H:%M:%S"))
        .map(|datetime| chrono::DateTime::<chrono::Utc>::from_naive_utc_and_offset(datetime, chrono::Utc).fixed_offset())
        .map_err(|_| validation_error(field, format!("{label} must be a valid date/time")))
}
{% endif %}{% if has_optional_datetime %}
fn parse_optional_datetime(
    value: &str,
    field: &str,
    label: &str,
) -> Result<Option<chrono::DateTime<chrono::FixedOffset>>, AppError> {
    if value.trim().is_empty() {
        Ok(None)
    } else {
        parse_datetime(value, field, label).map(Some)
    }
}
{% endif %}{% if has_optional %}
fn optional_string(value: &str) -> Option<String> {
    let value = value.trim();
    if value.is_empty() {
        None
    } else {
        Some(value.to_string())
    }
}
{% endif %}"#
    .to_string()
}

fn scaffold_use_case_update_template() -> String {
    r#"use crate::models::{{ name_snake }};
use crate::repositories::{{ name_snake }}_repo::{Update{{ name_pascal }}, {{ name_pascal }}Repository};
use rwfw_core::error::AppError;
use std::collections::HashMap;

#[derive(Debug, serde::Serialize, serde::Deserialize)]
pub struct Update{{ name_pascal }}Input {
{% for field in fields %}{% if field.is_bool %}    #[serde(default, deserialize_with = "rwfw_core::forms::checkbox")]
{% endif %}    pub {{ field.name }}: {{ field.input_rust_type }},
{% endfor %}}

pub struct Update{{ name_pascal }}Output {
    pub {{ name_snake }}: {{ name_snake }}::Model,
}

pub struct Update{{ name_pascal }}UseCase;

impl Update{{ name_pascal }}UseCase {
    pub async fn execute(
        &self,
        repo: &{{ name_pascal }}Repository,
        id: i32,
        input: Update{{ name_pascal }}Input,
    ) -> Result<Update{{ name_pascal }}Output, AppError> {
        validate_input(&input)?;
{% for field in fields %}{% if field.optional and field.is_bool %}        let {{ field.name }} = Some(input.{{ field.name }});
{% elif field.optional and field.is_number %}        let {{ field.name }} = parse_optional_i32(&input.{{ field.name }}, "{{ field.name }}", "{{ field.title }}")?;
{% elif field.optional and field.is_decimal %}        let {{ field.name }} = parse_optional_f64(&input.{{ field.name }}, "{{ field.name }}", "{{ field.title }}")?;
{% elif field.optional and field.is_uuid %}        let {{ field.name }} = parse_optional_uuid(&input.{{ field.name }}, "{{ field.name }}", "{{ field.title }}")?;
{% elif field.optional and field.is_date %}        let {{ field.name }} = parse_optional_date(&input.{{ field.name }}, "{{ field.name }}", "{{ field.title }}")?;
{% elif field.optional and field.is_datetime %}        let {{ field.name }} = parse_optional_datetime(&input.{{ field.name }}, "{{ field.name }}", "{{ field.title }}")?;
{% elif field.optional %}        let {{ field.name }} = optional_string(&input.{{ field.name }});
{% elif field.is_bool %}        let {{ field.name }} = input.{{ field.name }};
{% elif field.is_number %}        let {{ field.name }} = parse_i32(&input.{{ field.name }}, "{{ field.name }}", "{{ field.title }}")?;
{% elif field.is_decimal %}        let {{ field.name }} = parse_f64(&input.{{ field.name }}, "{{ field.name }}", "{{ field.title }}")?;
{% elif field.is_uuid %}        let {{ field.name }} = parse_uuid(&input.{{ field.name }}, "{{ field.name }}", "{{ field.title }}")?;
{% elif field.is_date %}        let {{ field.name }} = parse_date(&input.{{ field.name }}, "{{ field.name }}", "{{ field.title }}")?;
{% elif field.is_datetime %}        let {{ field.name }} = parse_datetime(&input.{{ field.name }}, "{{ field.name }}", "{{ field.title }}")?;
{% else %}        let {{ field.name }} = input.{{ field.name }}.trim().to_string();
{% endif %}{% endfor %}
        let {{ name_snake }} = repo
            .update(
                id,
                Update{{ name_pascal }} {
{% for field in fields %}                    {{ field.name }},
{% endfor %}                },
            )
            .await
            .map_err(AppError::Internal)?;

        tracing::info!({{ name_snake }}_id = %{{ name_snake }}.id, "{{ name_pascal }} updated");
        Ok(Update{{ name_pascal }}Output { {{ name_snake }} })
    }
}

fn validate_input(input: &Update{{ name_pascal }}Input) -> Result<(), AppError> {
    let mut errors = HashMap::new();
{% for field in fields %}{% if not field.optional and not field.is_bool %}    if input.{{ field.name }}.trim().is_empty() {
        errors.insert("{{ field.name }}".to_string(), vec!["{{ field.title }} is required".to_string()]);
    }
{% endif %}{% endfor %}
    if errors.is_empty() {
        Ok(())
    } else {
        Err(AppError::Validation(errors))
    }
}
{% if has_number or has_decimal or has_uuid or has_date or has_datetime %}
fn validation_error(field: &str, message: impl Into<String>) -> AppError {
    let mut errors = HashMap::new();
    errors.insert(field.to_string(), vec![message.into()]);
    AppError::Validation(errors)
}
{% endif %}{% if has_number %}
fn parse_i32(value: &str, field: &str, label: &str) -> Result<i32, AppError> {
    value
        .trim()
        .parse::<i32>()
        .map_err(|_| validation_error(field, format!("{label} must be a valid integer")))
}
{% endif %}{% if has_optional_number %}
fn parse_optional_i32(value: &str, field: &str, label: &str) -> Result<Option<i32>, AppError> {
    if value.trim().is_empty() {
        Ok(None)
    } else {
        parse_i32(value, field, label).map(Some)
    }
}
{% endif %}{% if has_decimal %}
fn parse_f64(value: &str, field: &str, label: &str) -> Result<f64, AppError> {
    value
        .trim()
        .parse::<f64>()
        .map_err(|_| validation_error(field, format!("{label} must be a valid decimal number")))
}
{% endif %}{% if has_optional_decimal %}
fn parse_optional_f64(value: &str, field: &str, label: &str) -> Result<Option<f64>, AppError> {
    if value.trim().is_empty() {
        Ok(None)
    } else {
        parse_f64(value, field, label).map(Some)
    }
}
{% endif %}{% if has_uuid %}
fn parse_uuid(value: &str, field: &str, label: &str) -> Result<uuid::Uuid, AppError> {
    value
        .trim()
        .parse::<uuid::Uuid>()
        .map_err(|_| validation_error(field, format!("{label} must be a valid UUID")))
}
{% endif %}{% if has_optional_uuid %}
fn parse_optional_uuid(value: &str, field: &str, label: &str) -> Result<Option<uuid::Uuid>, AppError> {
    if value.trim().is_empty() {
        Ok(None)
    } else {
        parse_uuid(value, field, label).map(Some)
    }
}
{% endif %}{% if has_date %}
fn parse_date(value: &str, field: &str, label: &str) -> Result<chrono::NaiveDate, AppError> {
    chrono::NaiveDate::parse_from_str(value.trim(), "%Y-%m-%d")
        .map_err(|_| validation_error(field, format!("{label} must be a valid date")))
}
{% endif %}{% if has_optional_date %}
fn parse_optional_date(
    value: &str,
    field: &str,
    label: &str,
) -> Result<Option<chrono::NaiveDate>, AppError> {
    if value.trim().is_empty() {
        Ok(None)
    } else {
        parse_date(value, field, label).map(Some)
    }
}
{% endif %}{% if has_datetime %}
fn parse_datetime(
    value: &str,
    field: &str,
    label: &str,
) -> Result<chrono::DateTime<chrono::FixedOffset>, AppError> {
    if let Ok(datetime) = chrono::DateTime::parse_from_rfc3339(value.trim()) {
        return Ok(datetime);
    }

    chrono::NaiveDateTime::parse_from_str(value.trim(), "%Y-%m-%dT%H:%M")
        .or_else(|_| chrono::NaiveDateTime::parse_from_str(value.trim(), "%Y-%m-%dT%H:%M:%S"))
        .map(|datetime| chrono::DateTime::<chrono::Utc>::from_naive_utc_and_offset(datetime, chrono::Utc).fixed_offset())
        .map_err(|_| validation_error(field, format!("{label} must be a valid date/time")))
}
{% endif %}{% if has_optional_datetime %}
fn parse_optional_datetime(
    value: &str,
    field: &str,
    label: &str,
) -> Result<Option<chrono::DateTime<chrono::FixedOffset>>, AppError> {
    if value.trim().is_empty() {
        Ok(None)
    } else {
        parse_datetime(value, field, label).map(Some)
    }
}
{% endif %}{% if has_optional %}
fn optional_string(value: &str) -> Option<String> {
    let value = value.trim();
    if value.is_empty() {
        None
    } else {
        Some(value.to_string())
    }
}
{% endif %}"#
    .to_string()
}

fn scaffold_use_case_delete_template() -> String {
    r#"use crate::repositories::{{ name_snake }}_repo::{{ name_pascal }}Repository;
use rwfw_core::error::AppError;

pub struct Delete{{ name_pascal }}UseCase;

impl Delete{{ name_pascal }}UseCase {
    pub async fn execute(&self, repo: &{{ name_pascal }}Repository, id: i32) -> Result<(), AppError> {
        repo.delete(id).await.map_err(AppError::Internal)?;
        tracing::info!({{ name_snake }}_id = %id, "{{ name_pascal }} deleted");
        Ok(())
    }
}
"#
    .to_string()
}

fn scaffold_route_index_template() -> String {
    super::generate::BUILTIN_ROUTE_INDEX_TEMPLATE.to_string()
}

fn scaffold_route_create_template() -> String {
    super::generate::BUILTIN_ROUTE_CREATE_TEMPLATE.to_string()
}

fn scaffold_route_edit_template() -> String {
    super::generate::BUILTIN_ROUTE_EDIT_TEMPLATE.to_string()
}

fn scaffold_route_item_template() -> String {
    super::generate::BUILTIN_ROUTE_ITEM_TEMPLATE.to_string()
}

fn scaffold_page_index_template() -> String {
    super::generate::BUILTIN_PAGE_INDEX_TEMPLATE.to_string()
}

fn scaffold_form_template() -> String {
    super::generate::BUILTIN_FORM_TEMPLATE.to_string()
}

fn scaffold_page_create_template() -> String {
    super::generate::BUILTIN_PAGE_CREATE_TEMPLATE.to_string()
}

fn scaffold_page_edit_template() -> String {
    super::generate::BUILTIN_PAGE_EDIT_TEMPLATE.to_string()
}

fn scaffold_page_show_template() -> String {
    super::generate::BUILTIN_PAGE_SHOW_TEMPLATE.to_string()
}

// ---------------------------------------------------------------------------
// Ecommerce example — Hotwire `View` routes (replace the Inertia handlers) and
// `.html.j2` MiniJinja pages (replace the `.tsx` pages). All server-rendered.
// ---------------------------------------------------------------------------

fn shop_index_route_rs() -> String {
    r#"use crate::repositories::shop_repo::ShopRepository;
use axum::extract::{Query, State};
use axum::response::{IntoResponse, Response};
use axum::routing;
use rwfw_core::app::AppState;
use rwfw_core::error::AppError;
use rwfw_core::view::View;
use serde::Deserialize;

#[derive(Debug, Deserialize)]
struct ProductFilters {
    q: Option<String>,
    category: Option<String>,
}

pub fn route() -> axum::routing::MethodRouter<AppState> {
    routing::get(get)
}

async fn get(
    State(state): State<AppState>,
    v: View,
    Query(filters): Query<ProductFilters>,
) -> Response {
    let repo = ShopRepository::new(state.db.clone());
    let categories = match repo.categories().await {
        Ok(categories) => categories,
        Err(error) => return AppError::Internal(error).into_response(),
    };
    let featured = match repo.featured_products().await {
        Ok(products) => products,
        Err(error) => return AppError::Internal(error).into_response(),
    };
    let products = match repo
        .products(filters.q.as_deref(), filters.category.as_deref())
        .await
    {
        Ok(products) => products,
        Err(error) => return AppError::Internal(error).into_response(),
    };

    v.render(
        "shop/index",
        serde_json::json!({
            "categories": categories,
            "featured": featured,
            "products": products,
            "filters": {
                "q": filters.q.unwrap_or_default(),
                "category": filters.category.unwrap_or_default(),
            }
        }),
    )
}
"#
    .to_string()
}

fn shop_product_show_route_rs() -> String {
    r#"use crate::repositories::shop_repo::ShopRepository;
use axum::extract::{Path, State};
use axum::response::{IntoResponse, Response};
use axum::routing;
use rwfw_core::app::AppState;
use rwfw_core::error::AppError;
use rwfw_core::view::View;

pub fn route() -> axum::routing::MethodRouter<AppState> {
    routing::get(get)
}

async fn get(
    State(state): State<AppState>,
    v: View,
    Path(slug): Path<String>,
) -> Response {
    let repo = ShopRepository::new(state.db.clone());
    let product = match repo.find_product_by_slug(&slug).await {
        Ok(Some(product)) => product,
        Ok(None) => return AppError::NotFound(format!("Product not found: {slug}")).into_response(),
        Err(error) => return AppError::Internal(error).into_response(),
    };

    v.render(
        "shop/products/show",
        serde_json::json!({
            "product": product,
        }),
    )
}
"#
    .to_string()
}

fn shop_cart_route_rs() -> String {
    r#"use crate::cart;
use crate::repositories::shop_repo::ShopRepository;
use axum::extract::State;
use axum::http::HeaderMap;
use axum::response::{IntoResponse, Response};
use axum::routing;
use rwfw_core::app::AppState;
use rwfw_core::error::AppError;
use rwfw_core::view::View;

pub fn route() -> axum::routing::MethodRouter<AppState> {
    routing::get(get)
}

async fn get(State(state): State<AppState>, headers: HeaderMap, v: View) -> Response {
    let cart = cart::parse(&headers);
    let repo = ShopRepository::new(state.db.clone());
    let ids = cart.keys().copied().collect::<Vec<_>>();
    let products = match repo.find_products_by_ids(&ids).await {
        Ok(products) => products,
        Err(error) => return AppError::Internal(error).into_response(),
    };

    let mut lines = Vec::new();
    let mut total_cents = 0;
    let mut currency = "USD".to_string();
    for product in products {
        let quantity = cart.get(&product.id).copied().unwrap_or(0);
        if quantity <= 0 {
            continue;
        }
        let subtotal = product.price_cents * quantity;
        total_cents += subtotal;
        currency = product.currency.clone();
        lines.push(serde_json::json!({
            "product": product,
            "quantity": quantity,
            "subtotal_cents": subtotal,
        }));
    }

    v.render(
        "shop/cart/index",
        serde_json::json!({
            "lines": lines,
            "total_cents": total_cents,
            "currency": currency,
        }),
    )
}
"#
    .to_string()
}

fn shop_cart_add_route_rs() -> String {
    r#"use crate::cart;
use axum::extract::Form;
use axum::http::HeaderMap;
use axum::response::{IntoResponse, Redirect, Response};
use axum::routing;
use rwfw_core::app::AppState;
use serde::Deserialize;

#[derive(Debug, Deserialize)]
struct AddToCartInput {
    product_id: i32,
    #[serde(default = "default_quantity")]
    quantity: i32,
}

fn default_quantity() -> i32 {
    1
}

pub fn route() -> axum::routing::MethodRouter<AppState> {
    routing::post(post)
}

async fn post(headers: HeaderMap, Form(input): Form<AddToCartInput>) -> Response {
    let cart = cart::parse(&headers);
    let cart = cart::add(cart, input.product_id, input.quantity);

    let mut response = Redirect::to("/shop/cart").into_response();
    cart::apply(&mut response, &cart);
    response
}
"#
    .to_string()
}

fn shop_cart_update_route_rs() -> String {
    r#"use crate::cart;
use axum::extract::Form;
use axum::http::HeaderMap;
use axum::response::{IntoResponse, Redirect, Response};
use axum::routing;
use rwfw_core::app::AppState;
use serde::Deserialize;

#[derive(Debug, Deserialize)]
struct UpdateCartInput {
    product_id: i32,
    quantity: i32,
}

pub fn route() -> axum::routing::MethodRouter<AppState> {
    routing::post(post)
}

async fn post(headers: HeaderMap, Form(input): Form<UpdateCartInput>) -> Response {
    let cart = cart::parse(&headers);
    let cart = cart::set_quantity(cart, input.product_id, input.quantity);

    let mut response = Redirect::to("/shop/cart").into_response();
    cart::apply(&mut response, &cart);
    response
}
"#
    .to_string()
}

fn shop_checkout_route_rs() -> String {
    r#"use crate::cart;
use crate::repositories::shop_repo::ShopRepository;
use crate::use_cases::checkout::{CheckoutInput, CheckoutUseCase};
use axum::extract::{Form, State};
use axum::http::{HeaderMap, StatusCode};
use axum::response::{IntoResponse, Response};
use axum::routing;
use rwfw_core::app::AppState;
use rwfw_core::error::AppError;
use rwfw_core::inertia::Inertia;
use rwfw_core::view::View;

pub fn route() -> axum::routing::MethodRouter<AppState> {
    routing::get(get).post(post)
}

async fn get(State(state): State<AppState>, headers: HeaderMap, v: View) -> Response {
    let cart = cart::parse(&headers);
    let repo = ShopRepository::new(state.db.clone());
    let ids = cart.keys().copied().collect::<Vec<_>>();
    let products = match repo.find_products_by_ids(&ids).await {
        Ok(products) => products,
        Err(error) => return AppError::Internal(error).into_response(),
    };

    let (lines, total_cents, currency) = summarize(&cart, products);

    v.render(
        "shop/checkout/index",
        serde_json::json!({
            "lines": lines,
            "total_cents": total_cents,
            "currency": currency,
            "old": serde_json::Value::Null,
        }),
    )
}

async fn post(
    State(state): State<AppState>,
    headers: HeaderMap,
    v: View,
    Form(input): Form<CheckoutInput>,
) -> Response {
    let cart = cart::parse(&headers);
    let repo = ShopRepository::new(state.db.clone());
    let use_case = CheckoutUseCase;

    let old = serde_json::json!({
        "customer_name": input.customer_name,
        "customer_email": input.customer_email,
        "address_line": input.address_line,
        "city": input.city,
        "country": input.country,
    });

    match use_case.execute(&repo, input, &cart).await {
        Ok(order) => {
            // Clear the cart cookie and redirect to the confirmation page.
            let mut response = Inertia::redirect_with_success(
                &format!("/shop/orders/{}", order.number),
                "Fake checkout completed",
            );
            cart::apply(&mut response, &std::collections::BTreeMap::new());
            response
        }
        Err(AppError::Validation(errors)) => {
            let ids = cart.keys().copied().collect::<Vec<_>>();
            let products = match repo.find_products_by_ids(&ids).await {
                Ok(products) => products,
                Err(error) => return AppError::Internal(error).into_response(),
            };
            let (lines, total_cents, currency) = summarize(&cart, products);
            v.render_status(
                StatusCode::UNPROCESSABLE_ENTITY,
                "shop/checkout/index",
                serde_json::json!({
                    "lines": lines,
                    "total_cents": total_cents,
                    "currency": currency,
                    "old": old,
                    "errors": rwfw_core::validation::first_messages(errors),
                }),
            )
        }
        Err(AppError::BadRequest(message)) => {
            Inertia::redirect_with_error("/shop/cart", message)
        }
        Err(error) => error.into_response(),
    }
}

fn summarize(
    cart: &std::collections::BTreeMap<i32, i32>,
    products: Vec<crate::models::product::Model>,
) -> (Vec<serde_json::Value>, i32, String) {
    let mut lines = Vec::new();
    let mut total_cents = 0;
    let mut currency = "USD".to_string();
    for product in products {
        let quantity = cart.get(&product.id).copied().unwrap_or(0);
        if quantity <= 0 {
            continue;
        }
        let subtotal = product.price_cents * quantity;
        total_cents += subtotal;
        currency = product.currency.clone();
        lines.push(serde_json::json!({
            "product": product,
            "quantity": quantity,
            "subtotal_cents": subtotal,
        }));
    }
    (lines, total_cents, currency)
}
"#
    .to_string()
}

fn shop_order_show_route_rs() -> String {
    r#"use crate::repositories::shop_repo::ShopRepository;
use axum::extract::{Path, State};
use axum::response::{IntoResponse, Response};
use axum::routing;
use rwfw_core::app::AppState;
use rwfw_core::error::AppError;
use rwfw_core::view::View;

pub fn route() -> axum::routing::MethodRouter<AppState> {
    routing::get(get)
}

async fn get(
    State(state): State<AppState>,
    v: View,
    Path(number): Path<String>,
) -> Response {
    let repo = ShopRepository::new(state.db.clone());
    let (order, items) = match repo.find_order(&number).await {
        Ok(Some(order)) => order,
        Ok(None) => return AppError::NotFound(format!("Order not found: {number}")).into_response(),
        Err(error) => return AppError::Internal(error).into_response(),
    };

    v.render(
        "shop/orders/show",
        serde_json::json!({
            "order": order,
            "items": items,
        }),
    )
}
"#
    .to_string()
}

fn shop_index_page_template() -> String {
    r#"{% extends "layouts/app.html.j2" %}
{% block title %}Shop{% endblock %}
{% block content %}
<div class="max-w-5xl">
  <div class="flex justify-between items-center mb-8">
    <h1 class="text-3xl font-bold">Shop</h1>
    <a href="/shop/cart" class="px-4 py-2 bg-gray-100 text-gray-700 rounded-md hover:bg-gray-200">View cart</a>
  </div>

  <form method="get" action="/shop" class="mb-8 flex flex-wrap gap-3 items-end">
    <div class="flex-1 min-w-[200px]">
      <label for="q" class="block text-sm font-medium text-gray-700">Search</label>
      <input id="q" name="q" type="search" value="{{ filters.q }}" placeholder="Search products"
             class="mt-1 block w-full rounded-md border border-gray-300 px-3 py-2 shadow-sm focus:border-blue-500 focus:outline-none focus:ring-1 focus:ring-blue-500">
    </div>
    <div>
      <label for="category" class="block text-sm font-medium text-gray-700">Category</label>
      <select id="category" name="category"
              class="mt-1 block rounded-md border border-gray-300 px-3 py-2 shadow-sm focus:border-blue-500 focus:outline-none focus:ring-1 focus:ring-blue-500">
        <option value="">All categories</option>
        {% for category in categories %}
        <option value="{{ category.slug }}" {{ 'selected' if filters.category == category.slug }}>{{ category.name }}</option>
        {% endfor %}
      </select>
    </div>
    <button type="submit" class="px-4 py-2 bg-blue-600 text-white rounded-md hover:bg-blue-700">Filter</button>
    {% if filters.q or filters.category %}
    <a href="/shop" class="px-4 py-2 text-sm text-gray-600 hover:underline">Reset</a>
    {% endif %}
  </form>

  {% if not filters.q and not filters.category and featured | length > 0 %}
  <section class="mb-10">
    <h2 class="text-xl font-semibold mb-4">Featured</h2>
    <div class="grid grid-cols-1 sm:grid-cols-2 lg:grid-cols-3 gap-6">
      {% for product in featured %}
      {% include "shop/_product_card.html.j2" %}
      {% endfor %}
    </div>
  </section>
  {% endif %}

  <section>
    <h2 class="text-xl font-semibold mb-4">All products</h2>
    {% if products | length == 0 %}
      <p class="text-gray-500">No products match your filters.</p>
    {% else %}
    <div class="grid grid-cols-1 sm:grid-cols-2 lg:grid-cols-3 gap-6">
      {% for product in products %}
      {% include "shop/_product_card.html.j2" %}
      {% endfor %}
    </div>
    {% endif %}
  </section>
</div>
{% endblock %}
"#
    .to_string()
}

fn shop_product_card_template() -> String {
    r#"<article class="flex flex-col bg-white rounded-lg shadow-sm border overflow-hidden">
  <a href="/shop/products/{{ product.slug }}" class="block">
    <img src="{{ product.image_url }}" alt="{{ product.name }}" class="h-48 w-full object-cover">
  </a>
  <div class="flex flex-1 flex-col p-4">
    <a href="/shop/products/{{ product.slug }}">
      <h3 class="text-lg font-semibold hover:text-blue-600 transition-colors">{{ product.name }}</h3>
    </a>
    <p class="mt-1 text-sm text-gray-500 line-clamp-2 flex-1">{{ product.description }}</p>
    <div class="mt-3 flex items-center justify-between">
      <span class="text-lg font-bold">{{ "%.2f" | format(product.price_cents / 100) }} {{ product.currency }}</span>
      <form method="post" action="/shop/cart/add">
        <input type="hidden" name="_csrf" value="{{ csrf_token }}">
        <input type="hidden" name="product_id" value="{{ product.id }}">
        <input type="hidden" name="quantity" value="1">
        <button type="submit" data-turbo-submits-with="Adding..."
                class="px-3 py-1.5 bg-blue-600 text-white text-sm rounded-md hover:bg-blue-700 disabled:opacity-50">Add to cart</button>
      </form>
    </div>
  </div>
</article>
"#
    .to_string()
}

fn shop_product_show_page_template() -> String {
    r#"{% extends "layouts/app.html.j2" %}
{% block title %}{{ product.name }}{% endblock %}
{% block content %}
<div class="max-w-4xl">
  <div class="mb-6">
    <a href="/shop" class="text-blue-600 hover:underline text-sm">&larr; Back to shop</a>
  </div>

  <div class="grid grid-cols-1 md:grid-cols-2 gap-8 bg-white rounded-lg shadow-sm border p-8">
    <img src="{{ product.image_url }}" alt="{{ product.name }}" class="w-full rounded-lg object-cover">
    <div class="flex flex-col">
      <h1 class="text-3xl font-bold mb-4">{{ product.name }}</h1>
      <p class="text-2xl font-semibold mb-4">{{ "%.2f" | format(product.price_cents / 100) }} {{ product.currency }}</p>
      <p class="text-gray-600 mb-6">{{ product.description }}</p>
      <p class="text-sm text-gray-400 mb-6">
        {% if product.inventory > 0 %}{{ product.inventory }} in stock{% else %}Out of stock{% endif %}
      </p>
      {% if product.inventory > 0 %}
      <form method="post" action="/shop/cart/add" class="flex items-end gap-4">
        <input type="hidden" name="_csrf" value="{{ csrf_token }}">
        <input type="hidden" name="product_id" value="{{ product.id }}">
        <div>
          <label for="quantity" class="block text-sm font-medium text-gray-700">Quantity</label>
          <input id="quantity" name="quantity" type="number" min="1" max="20" value="1"
                 class="mt-1 block w-24 rounded-md border border-gray-300 px-3 py-2 shadow-sm focus:border-blue-500 focus:outline-none focus:ring-1 focus:ring-blue-500">
        </div>
        <button type="submit" data-turbo-submits-with="Adding..."
                class="px-6 py-2 bg-blue-600 text-white rounded-md hover:bg-blue-700 disabled:opacity-50">Add to cart</button>
      </form>
      {% endif %}
    </div>
  </div>
</div>
{% endblock %}
"#
    .to_string()
}

fn shop_cart_page_template() -> String {
    r#"{% extends "layouts/app.html.j2" %}
{% block title %}Cart{% endblock %}
{% block content %}
<div class="max-w-4xl">
  <div class="flex justify-between items-center mb-8">
    <h1 class="text-3xl font-bold">Your cart</h1>
    <a href="/shop" class="text-blue-600 hover:underline text-sm">Continue shopping</a>
  </div>

  {% if lines | length == 0 %}
    <p class="text-gray-500">Your cart is empty. <a href="/shop" class="text-blue-600 hover:underline">Browse products</a>.</p>
  {% else %}
  <div class="overflow-hidden rounded-lg border bg-white shadow-sm">
    <table class="min-w-full divide-y divide-gray-200">
      <thead class="bg-gray-50">
        <tr>
          <th class="px-4 py-3 text-left text-xs font-medium uppercase tracking-wider text-gray-500">Product</th>
          <th class="px-4 py-3 text-left text-xs font-medium uppercase tracking-wider text-gray-500">Price</th>
          <th class="px-4 py-3 text-left text-xs font-medium uppercase tracking-wider text-gray-500">Quantity</th>
          <th class="px-4 py-3 text-right text-xs font-medium uppercase tracking-wider text-gray-500">Subtotal</th>
        </tr>
      </thead>
      <tbody class="divide-y divide-gray-100">
        {% for line in lines %}
        <tr>
          <td class="px-4 py-3">
            <a href="/shop/products/{{ line.product.slug }}" class="font-medium text-gray-900 hover:text-blue-600">{{ line.product.name }}</a>
          </td>
          <td class="px-4 py-3 text-sm text-gray-500">{{ "%.2f" | format(line.product.price_cents / 100) }} {{ line.product.currency }}</td>
          <td class="px-4 py-3">
            <form method="post" action="/shop/cart/update" class="flex items-center gap-2">
              <input type="hidden" name="_csrf" value="{{ csrf_token }}">
              <input type="hidden" name="product_id" value="{{ line.product.id }}">
              <input name="quantity" type="number" min="0" max="20" value="{{ line.quantity }}"
                     class="w-20 rounded-md border border-gray-300 px-2 py-1 shadow-sm focus:border-blue-500 focus:outline-none focus:ring-1 focus:ring-blue-500">
              <button type="submit" class="text-sm text-blue-600 hover:underline">Update</button>
            </form>
          </td>
          <td class="px-4 py-3 text-right text-sm text-gray-900">{{ "%.2f" | format(line.subtotal_cents / 100) }} {{ currency }}</td>
        </tr>
        {% endfor %}
      </tbody>
      <tfoot class="bg-gray-50">
        <tr>
          <td colspan="3" class="px-4 py-3 text-right font-semibold">Total</td>
          <td class="px-4 py-3 text-right font-bold">{{ "%.2f" | format(total_cents / 100) }} {{ currency }}</td>
        </tr>
      </tfoot>
    </table>
  </div>

  <div class="mt-6 flex justify-end">
    <a href="/shop/checkout" class="px-6 py-2 bg-blue-600 text-white rounded-md hover:bg-blue-700">Proceed to checkout</a>
  </div>
  {% endif %}
</div>
{% endblock %}
"#
    .to_string()
}

fn shop_checkout_page_template() -> String {
    r#"{% extends "layouts/app.html.j2" %}
{% block title %}Checkout{% endblock %}
{% block content %}
<div class="max-w-4xl">
  <div class="mb-6">
    <a href="/shop/cart" class="text-blue-600 hover:underline text-sm">&larr; Back to cart</a>
  </div>
  <h1 class="text-3xl font-bold mb-8">Checkout</h1>

  {% if lines | length == 0 %}
    <p class="text-gray-500">Your cart is empty. <a href="/shop" class="text-blue-600 hover:underline">Browse products</a>.</p>
  {% else %}
  <div class="grid grid-cols-1 md:grid-cols-3 gap-8">
    <form method="post" action="/shop/checkout" class="md:col-span-2 bg-white rounded-lg shadow-sm border p-8 space-y-6">
      <input type="hidden" name="_csrf" value="{{ csrf_token }}">
      <x-field name="customer_name" label="Full name" :value="old.customer_name | default('')" />
      <x-field name="customer_email" label="Email" type="email" :value="old.customer_email | default('')" />
      <x-field name="address_line" label="Address" :value="old.address_line | default('')" />
      <div class="grid grid-cols-2 gap-4">
        <x-field name="city" label="City" :value="old.city | default('')" />
        <x-field name="country" label="Country" :value="old.country | default('')" />
      </div>
      {% if errors.items %}<p class="text-sm text-red-600">{{ errors.items }}</p>{% endif %}
      <button type="submit" data-turbo-submits-with="Placing order..."
              class="w-full px-6 py-2 bg-blue-600 text-white rounded-md hover:bg-blue-700 disabled:opacity-50">Place order (fake)</button>
    </form>

    <aside class="bg-white rounded-lg shadow-sm border p-6 h-fit">
      <h2 class="text-lg font-semibold mb-4">Order summary</h2>
      <ul class="space-y-2 text-sm">
        {% for line in lines %}
        <li class="flex justify-between">
          <span>{{ line.quantity }} &times; {{ line.product.name }}</span>
          <span>{{ "%.2f" | format(line.subtotal_cents / 100) }}</span>
        </li>
        {% endfor %}
      </ul>
      <div class="mt-4 border-t pt-4 flex justify-between font-bold">
        <span>Total</span>
        <span>{{ "%.2f" | format(total_cents / 100) }} {{ currency }}</span>
      </div>
    </aside>
  </div>
  {% endif %}
</div>
{% endblock %}
"#
    .to_string()
}

fn shop_order_show_page_template() -> String {
    r#"{% extends "layouts/app.html.j2" %}
{% block title %}Order {{ order.number }}{% endblock %}
{% block content %}
<div class="max-w-3xl">
  <div class="mb-6">
    <a href="/shop" class="text-blue-600 hover:underline text-sm">&larr; Back to shop</a>
  </div>

  <div class="bg-white rounded-lg shadow-sm border p-8">
    <div class="mb-6">
      <p class="text-sm uppercase tracking-wider text-green-600 font-semibold">Order confirmed</p>
      <h1 class="text-3xl font-bold mt-1">Order {{ order.number }}</h1>
      <p class="mt-2 text-gray-500">Thanks, {{ order.customer_name }}. A confirmation was sent to {{ order.customer_email }}.</p>
    </div>

    <div class="grid grid-cols-1 sm:grid-cols-2 gap-4 mb-8 text-sm">
      <div>
        <p class="font-semibold text-gray-700">Shipping to</p>
        <p class="text-gray-600">{{ order.address_line }}</p>
        <p class="text-gray-600">{{ order.city }}, {{ order.country }}</p>
      </div>
      <div>
        <p class="font-semibold text-gray-700">Status</p>
        <p class="text-gray-600">{{ order.status }}</p>
      </div>
    </div>

    <div class="overflow-hidden rounded-lg border">
      <table class="min-w-full divide-y divide-gray-200">
        <thead class="bg-gray-50">
          <tr>
            <th class="px-4 py-3 text-left text-xs font-medium uppercase tracking-wider text-gray-500">Item</th>
            <th class="px-4 py-3 text-left text-xs font-medium uppercase tracking-wider text-gray-500">Qty</th>
            <th class="px-4 py-3 text-right text-xs font-medium uppercase tracking-wider text-gray-500">Subtotal</th>
          </tr>
        </thead>
        <tbody class="divide-y divide-gray-100">
          {% for item in items %}
          <tr>
            <td class="px-4 py-3 text-sm text-gray-900">{{ item.product_name }}</td>
            <td class="px-4 py-3 text-sm text-gray-500">{{ item.quantity }}</td>
            <td class="px-4 py-3 text-right text-sm text-gray-900">{{ "%.2f" | format(item.subtotal_cents / 100) }} {{ order.currency }}</td>
          </tr>
          {% endfor %}
        </tbody>
        <tfoot class="bg-gray-50">
          <tr>
            <td colspan="2" class="px-4 py-3 text-right font-semibold">Total</td>
            <td class="px-4 py-3 text-right font-bold">{{ "%.2f" | format(order.total_cents / 100) }} {{ order.currency }}</td>
          </tr>
        </tfoot>
      </table>
    </div>
  </div>
</div>
{% endblock %}
"#
    .to_string()
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
