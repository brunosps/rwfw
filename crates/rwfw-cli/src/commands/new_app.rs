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

pub fn run(
    name: &str,
    example: ExampleKind,
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
        example,
        dependency_source,
    };

    create_dirs(app_dir)?;
    create_example_dirs(app_dir, example)?;
    write_file(&app_dir.join(".gitignore"), gitignore())?;
    write_file(&app_dir.join(".dockerignore"), dockerignore())?;
    write_file(&app_dir.join(".env.example"), env_example())?;
    write_file(&app_dir.join("Dockerfile"), dockerfile(&context))?;
    write_file(&app_dir.join("Dockerfile.prod"), dockerfile(&context))?;
    write_file(&app_dir.join("compose.yaml"), compose_yaml(&context))?;
    write_file(
        &app_dir.join("compose.dev.yaml"),
        compose_dev_yaml(&context),
    )?;
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
        &app_dir.join(".rwfw/templates/page_index.tsx.tera"),
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
        &app_dir.join(".rwfw/templates/form.tsx.tera"),
        scaffold_form_template(),
    )?;
    write_file(
        &app_dir.join(".rwfw/templates/page_create.tsx.tera"),
        scaffold_page_create_template(),
    )?;
    write_file(
        &app_dir.join(".rwfw/templates/page_edit.tsx.tera"),
        scaffold_page_edit_template(),
    )?;
    write_file(
        &app_dir.join(".rwfw/templates/page_show.tsx.tera"),
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
    write_file(&app_dir.join("package.json"), package_json(&context))?;
    write_file(&app_dir.join("tsconfig.json"), tsconfig_json())?;
    write_file(&app_dir.join("vite.config.ts"), vite_config_ts())?;
    write_file(&app_dir.join("postcss.config.js"), postcss_config_js())?;
    write_file(&app_dir.join("tailwind.config.ts"), tailwind_config_ts())?;
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
    write_file(&app_dir.join("crates/app/web/app.tsx"), app_tsx())?;
    write_file(&app_dir.join("crates/app/web/ssr.tsx"), ssr_tsx())?;
    write_file(&app_dir.join("crates/app/web/app.css"), app_css())?;
    write_file(
        &app_dir.join("crates/app/web/layouts/AppLayout.tsx"),
        app_layout_tsx(&context),
    )?;
    write_file(
        &app_dir.join("crates/app/web/layouts/AuthLayout.tsx"),
        auth_layout_tsx(&context),
    )?;
    write_file(
        &app_dir.join("crates/app/web/components/FlashMessages.tsx"),
        flash_messages_tsx(),
    )?;
    write_file(
        &app_dir.join("crates/app/web/hooks/useAuth.ts"),
        use_auth_ts(),
    )?;
    write_file(
        &app_dir.join("crates/app/web/types/inertia.d.ts"),
        inertia_types_ts(),
    )?;
    write_file(
        &app_dir.join("crates/modules/auth/web/pages/Login.tsx"),
        auth_login_page_tsx(),
    )?;
    write_file(
        &app_dir.join("crates/modules/auth/web/pages/Register.tsx"),
        auth_register_page_tsx(),
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
        &app_dir.join("crates/modules/home/web/pages/Index.tsx"),
        home_index_page_tsx(&context),
    )?;
    match example {
        ExampleKind::Blog => write_blog_example(app_dir, &context)?,
        ExampleKind::Ecommerce => write_ecommerce_example(app_dir, &context)?,
    }

    println!(
        "Created RWFW {example} app `{app_name}` at {}",
        app_dir.display(),
        example = context.example.label()
    );
    println!("Next steps:");
    println!("  cd {app_name}");
    println!("  docker compose -f compose.dev.yaml up -d");
    println!("  npm install");
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
    example: ExampleKind,
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
        "crates/app/web/components",
        "crates/app/web/hooks",
        "crates/app/web/layouts",
        "crates/app/web/types",
        "crates/modules/home/src/migrations",
        "crates/modules/home/src/routes",
        "crates/modules/home/web/pages",
        "crates/modules/auth/web/pages",
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
            "crates/modules/blog/web/pages/posts",
            "crates/modules/blog/web/pages/admin/posts",
        ],
        ExampleKind::Ecommerce => &[
            "crates/modules/shop/src/migrations",
            "crates/modules/shop/src/models",
            "crates/modules/shop/src/repositories",
            "crates/modules/shop/src/use_cases",
            "crates/modules/shop/src/routes/products",
            "crates/modules/shop/src/routes/cart",
            "crates/modules/shop/src/routes/checkout",
            "crates/modules/shop/src/routes/orders/[number]",
            "crates/modules/shop/web/pages/products",
            "crates/modules/shop/web/pages/cart",
            "crates/modules/shop/web/pages/checkout",
            "crates/modules/shop/web/pages/orders",
            "crates/modules/shop/web/components",
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
    fs::write(path, content).with_context(|| format!("writing {}", path.display()))?;
    Ok(())
}

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
/node_modules
/dist
.env
.DS_Store
"#
    .to_string()
}

fn dockerignore() -> String {
    r#"/target
/node_modules
/dist
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

FROM node:22-bookworm AS web-builder
WORKDIR /app
COPY package*.json ./
RUN if [ -f package-lock.json ]; then npm ci; else npm install; fi
COPY . .
RUN npm run build && npm run build:ssr

FROM rust:1-bookworm AS rust-builder
ARG APP_PACKAGE
ARG RWFW_FRAMEWORK_PATH
WORKDIR /app
{framework_copy}COPY . .
COPY --from=web-builder /app/dist ./dist
RUN cargo build --release -p "${{APP_PACKAGE}}"

FROM debian:bookworm-slim AS runtime
ARG APP_PACKAGE
ENV APP_PACKAGE=${{APP_PACKAGE}}
ENV RWFW_ENV=production
ENV RWFW_RUN_MIGRATIONS=1
ENV RWFW_VITE_ENTRY=crates/app/web/app.tsx
ENV RWFW_VITE_MANIFEST=dist/client/.vite/manifest.json
WORKDIR /app
RUN apt-get update \
    && apt-get install -y --no-install-recommends ca-certificates libssl3 libstdc++6 \
    && rm -rf /var/lib/apt/lists/*
COPY --from=rust-builder /app/target/release/${{APP_PACKAGE}} /usr/local/bin/rwfw-app
COPY --from=rust-builder /app/config ./config
COPY --from=rust-builder /app/dist ./dist
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

Generated by RWFW. This project is a runnable example application and a starting point for building modular Rust web apps with Axum, SeaORM, Inertia, React, Vite, SSR, and PostgreSQL.

Example selected: `{example}` ({example_title}).

## Framework Dependency Source

{dependency_source}

## What Is In This Project

- `rwfw.toml`: RWFW project metadata used by the CLI to find the app crate and modules directory.
- `Cargo.toml`: Rust workspace with the app crate and local modules.
- `package.json`: frontend scripts and React/Vite dependencies.
- `.env.example`: local environment variables for generated integrations such as SSO providers.
- `config/development.yaml`: local runtime config used by `rwfw dev`.
- `config/production.yaml`: production runtime config used by the Docker image.
- `compose.dev.yaml`: local development services: PostgreSQL and pgAdmin.
- `compose.yaml`: production-style Docker stack: PostgreSQL plus the compiled app container.
- `Dockerfile.prod`: multi-stage production image that builds frontend assets, SSR bundle, and the Rust binary.
- `docker/entrypoint.sh`: production container entrypoint; optionally runs migrations before starting the app.
- `.rwfw/templates`: project-local code generation templates used by `rwfw generate`.
- `crates/app`: application composition crate. It wires modules, SSR, config, and migrations into one binary.
- `crates/modules/home`: minimal example module with one page.
- `crates/modules/{example_module}`: generated `{example}` example module.
- `crates/modules/auth`: frontend auth pages for the framework auth module.

## Local Development

```bash
docker compose -f compose.dev.yaml up -d
npm install
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

The scaffold creates a SeaORM model, one SQL migration file, repository/use-case files, protected CRUD routes, searchable/paginated index pages, flash confirmations, and reusable React index/show/form/create/edit pages.

## Scaffold Templates

The generator reads templates from `.rwfw/templates` before using built-in defaults. Edit these files to customize generated code for this project:

- `model.rs.tera`: SeaORM model.
- `migration.sql.tera`: SQL migration.
- `repository.rs.tera`: SeaORM repository.
- `use_case_create.rs.tera`: create input, validation, and use-case.
- `use_case_update.rs.tera`: update input, validation, and use-case.
- `use_case_delete.rs.tera`: delete use-case.
- `route_index.rs.tera`: Axum/Inertia route.
- `route_create.rs.tera`: protected create page route.
- `route_edit.rs.tera`: protected edit/update route.
- `route_item.rs.tera`: protected show and item actions, including delete.
- `page_index.tsx.tera`: React page.
- `form.tsx.tera`: reusable form component.
- `page_create.tsx.tera`: React create page.
- `page_edit.tsx.tera`: React edit page.
- `page_show.tsx.tera`: React show page.

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
            r#"- `/shop` is the storefront.
- `/shop/products/:slug` shows a product detail page.
- `/shop/cart` is the local cart page.
- `/shop/checkout` posts a fake checkout to the backend.
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
    r#"[workspace]
resolver = "2"
members = [
    "crates/app",
    "crates/modules/home",
    "crates/modules/__EXAMPLE_MODULE__",
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
sea-orm = { version = "1", features = ["sqlx-postgres", "runtime-tokio-rustls", "macros"] }
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
}

fn package_json(context: &AppTemplateContext) -> String {
    format!(
        r#"{{
  "name": "{}",
  "private": true,
  "type": "module",
  "scripts": {{
    "dev": "vite",
    "build": "vite build",
    "build:ssr": "vite build --mode ssr",
    "preview": "vite preview"
  }},
  "dependencies": {{
    "@inertiajs/react": "^2.0.0",
    "react": "^19.0.0",
    "react-dom": "^19.0.0"
  }},
  "devDependencies": {{
    "@types/react": "^19.0.0",
    "@types/react-dom": "^19.0.0",
    "@vitejs/plugin-react": "^4.3.0",
    "autoprefixer": "^10.4.0",
    "postcss": "^8.4.0",
    "tailwindcss": "^3.4.0",
    "typescript": "^5.6.0",
    "vite": "^6.0.0"
  }}
}}
"#,
        context.app_name
    )
}

fn tsconfig_json() -> String {
    r#"{
  "compilerOptions": {
    "target": "ES2022",
    "lib": ["ES2022", "DOM", "DOM.Iterable"],
    "module": "ESNext",
    "moduleResolution": "bundler",
    "jsx": "react-jsx",
    "strict": true,
    "esModuleInterop": true,
    "skipLibCheck": true,
    "forceConsistentCasingInFileNames": true,
    "resolveJsonModule": true,
    "isolatedModules": true,
    "noEmit": true,
    "paths": {
      "@app/*": ["./crates/app/web/*"],
      "@modules/*": ["./crates/modules/*"]
    },
    "baseUrl": "."
  },
  "include": [
    "crates/app/web/**/*.ts",
    "crates/app/web/**/*.tsx",
    "crates/modules/*/web/**/*.ts",
    "crates/modules/*/web/**/*.tsx"
  ]
}
"#
    .to_string()
}

fn vite_config_ts() -> String {
    r#"import { defineConfig } from 'vite'
import react from '@vitejs/plugin-react'

const ssrPolyfills = `
(function () {
  const global = globalThis;

  if (typeof global.console === 'undefined') {
    global.console = { log() {}, info() {}, warn() {}, error() {}, debug() {} };
  }

  if (typeof global.performance === 'undefined') {
    global.performance = { now: () => Date.now() };
  }

  if (typeof global.queueMicrotask === 'undefined') {
    global.queueMicrotask = (callback) => Promise.resolve().then(callback);
  }

  if (typeof global.setTimeout === 'undefined') {
    global.setTimeout = (callback) => {
      global.queueMicrotask(callback);
      return 0;
    };
  }

  if (typeof global.clearTimeout === 'undefined') {
    global.clearTimeout = () => {};
  }

  if (typeof global.FormData === 'undefined') {
    global.FormData = class FormData {
      constructor() {
        this._entries = [];
      }

      append(name, value) {
        this._entries.push([name, value]);
      }

      forEach(callback) {
        this._entries.forEach(([name, value]) => callback(value, name, this));
      }
    };
  }

  if (typeof global.URLSearchParams === 'undefined') {
    global.URLSearchParams = class URLSearchParams {
      constructor(init = '') {
        this._params = [];
        const query = String(init).replace(/^\\?/, '');

        if (query) {
          query.split('&').forEach((part) => {
            if (!part) return;
            const [name, value = ''] = part.split('=');
            this.append(decodeURIComponent(name), decodeURIComponent(value));
          });
        }
      }

      append(name, value) {
        this._params.push([String(name), String(value)]);
      }

      set(name, value) {
        this.delete(name);
        this.append(name, value);
      }

      delete(name) {
        this._params = this._params.filter(([key]) => key !== String(name));
      }

      toString() {
        return this._params
          .map(([name, value]) => encodeURIComponent(name) + '=' + encodeURIComponent(value))
          .join('&');
      }
    };
  }

  if (typeof global.URL === 'undefined') {
    global.URL = class URL {
      constructor(input) {
        const raw = String(input || '/');
        const hashIndex = raw.indexOf('#');
        const withoutHash = hashIndex >= 0 ? raw.slice(0, hashIndex) : raw;

        this.hash = hashIndex >= 0 ? raw.slice(hashIndex) : '';

        const queryIndex = withoutHash.indexOf('?');
        this.pathname = queryIndex >= 0 ? withoutHash.slice(0, queryIndex) || '/' : withoutHash || '/';
        this.searchParams = new global.URLSearchParams(queryIndex >= 0 ? withoutHash.slice(queryIndex) : '');
      }

      get search() {
        const query = this.searchParams.toString();
        return query ? '?' + query : '';
      }

      set search(value) {
        this.searchParams = new global.URLSearchParams(value);
      }

      get href() {
        return this.pathname + this.search + this.hash;
      }

      toString() {
        return this.href;
      }
    };
  }

  if (typeof global.MessageChannel === 'undefined') {
    global.MessageChannel = class MessageChannel {
      constructor() {
        const port1 = { onmessage: null };
        const port2 = {
          postMessage(data) {
            global.queueMicrotask(() => {
              if (typeof port1.onmessage === 'function') {
                port1.onmessage({ data });
              }
            });
          },
        };

        this.port1 = port1;
        this.port2 = port2;
      }
    };
  }

  if (typeof global.TextEncoder === 'undefined') {
    global.TextEncoder = class TextEncoder {
      encode(input = '') {
        const text = String(input);
        const bytes = [];

        for (let index = 0; index < text.length; index += 1) {
          let codePoint = text.charCodeAt(index);

          if (codePoint >= 0xd800 && codePoint <= 0xdbff && index + 1 < text.length) {
            const next = text.charCodeAt(index + 1);
            if (next >= 0xdc00 && next <= 0xdfff) {
              codePoint = 0x10000 + ((codePoint - 0xd800) << 10) + (next - 0xdc00);
              index += 1;
            }
          }

          if (codePoint <= 0x7f) {
            bytes.push(codePoint);
          } else if (codePoint <= 0x7ff) {
            bytes.push(0xc0 | (codePoint >> 6), 0x80 | (codePoint & 0x3f));
          } else if (codePoint <= 0xffff) {
            bytes.push(0xe0 | (codePoint >> 12), 0x80 | ((codePoint >> 6) & 0x3f), 0x80 | (codePoint & 0x3f));
          } else {
            bytes.push(
              0xf0 | (codePoint >> 18),
              0x80 | ((codePoint >> 12) & 0x3f),
              0x80 | ((codePoint >> 6) & 0x3f),
              0x80 | (codePoint & 0x3f),
            );
          }
        }

        return new Uint8Array(bytes);
      }
    };
  }
})();
`;

export default defineConfig(({ mode }) => {
  const isSsrBuild = mode === 'ssr'
  const devServerPort = Number(process.env.RWFW_VITE_PORT ?? '5173')

  return {
    plugins: [react()],
    root: '.',
    define: {
      'process.env.NODE_ENV': JSON.stringify('production'),
    },
    build: {
      outDir: isSsrBuild ? 'dist/server' : 'dist/client',
      manifest: !isSsrBuild,
      lib: isSsrBuild
        ? {
            entry: 'crates/app/web/ssr.tsx',
            formats: ['iife'],
            name: 'RWFWSSR',
            fileName: () => 'ssr.js',
          }
        : undefined,
      rollupOptions: isSsrBuild
        ? {
            output: {
              banner: ssrPolyfills,
            },
          }
        : {
            input: 'crates/app/web/app.tsx',
          },
    },
    server: {
      port: devServerPort,
      strictPort: true,
    },
    resolve: {
      alias: {
        '@app': '/crates/app/web',
        '@modules': '/crates/modules',
      },
    },
  }
})
"#
    .to_string()
}

fn postcss_config_js() -> String {
    r#"export default {
  plugins: {
    tailwindcss: {},
    autoprefixer: {},
  },
}
"#
    .to_string()
}

fn tailwind_config_ts() -> String {
    r#"import type { Config } from 'tailwindcss'

export default {
  content: [
    'crates/app/web/**/*.tsx',
    'crates/modules/*/web/**/*.tsx',
  ],
  theme: {
    extend: {},
  },
  plugins: [],
} satisfies Config
"#
    .to_string()
}

fn development_yaml(context: &AppTemplateContext) -> String {
    format!(
        r#"database:
  url: "postgres://rwfw:rwfw@localhost:54329/{}"

server:
  host: "0.0.0.0"
  port: 3000

logging:
  level: "debug"
  format: "pretty"

vite:
  dev_server: "http://localhost:5173"
  manifest_path: "dist/client/.vite/manifest.json"
  ssr_bundle_path: "dist/server/ssr.js"

auth:
  session_ttl: 86400
  oidc:
    redirect_base_url: "http://localhost:3000"
    providers: {{}}
"#,
        context.database_name
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

vite:
  dev_server: null
  manifest_path: "dist/client/.vite/manifest.json"
  ssr_bundle_path: "dist/server/ssr.js"

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
        &app_dir.join("crates/modules/blog/web/pages/Index.tsx"),
        blog_index_page_tsx(),
    )?;
    write_file(
        &app_dir.join("crates/modules/blog/web/pages/posts/Index.tsx"),
        blog_posts_index_page_tsx(),
    )?;
    write_file(
        &app_dir.join("crates/modules/blog/web/pages/posts/Show.tsx"),
        blog_post_show_page_tsx(),
    )?;
    write_file(
        &app_dir.join("crates/modules/blog/web/pages/admin/posts/Index.tsx"),
        blog_admin_posts_index_page_tsx(),
    )?;
    write_file(
        &app_dir.join("crates/modules/blog/web/pages/admin/posts/Form.tsx"),
        blog_admin_post_form_tsx(),
    )?;
    write_file(
        &app_dir.join("crates/modules/blog/web/pages/admin/posts/New.tsx"),
        blog_admin_post_new_page_tsx(),
    )?;
    write_file(
        &app_dir.join("crates/modules/blog/web/pages/admin/posts/Edit.tsx"),
        blog_admin_post_edit_page_tsx(),
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
    init_ssr_if_available(&config);

    let mut app_builder = rwfw_core::app::RwfwApp::new(config);

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

fn init_ssr_if_available(config: &AppConfig) {
    if config.is_development() {
        return;
    }

    let ssr_bundle_path = config
        .vite()
        .ok()
        .and_then(|vite| vite.ssr_bundle_path)
        .unwrap_or_else(|| "dist/server/ssr.js".to_string());

    match std::fs::read_to_string(&ssr_bundle_path) {
        Ok(bundle) => rwfw_core::ssr::init(&bundle),
        Err(error) => {
            tracing::warn!(
                path = %ssr_bundle_path,
                error = %error,
                "SSR bundle not available; falling back to CSR shell"
            );
        }
    }
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

fn app_tsx() -> String {
    r#"import { createInertiaApp } from '@inertiajs/react'
import type { ComponentType } from 'react'
import { createRoot, hydrateRoot } from 'react-dom/client'
import './app.css'

createInertiaApp({
  resolve: (name) => {
    const pages = import.meta.glob<{ default: ComponentType<any> }>(
      '../../modules/*/web/pages/**/*.tsx',
      { eager: false }
    )

    const [module, ...rest] = name.split('/')
    const page = rest.join('/')
    const key = `../../modules/${module}/web/pages/${page}.tsx`

    if (!pages[key]) {
      throw new Error(`Page not found: ${name} (looked for ${key})`)
    }

    return pages[key]()
  },
  setup({ el, App, props }) {
    if (el.innerHTML) {
      hydrateRoot(el, <App {...props} />)
    } else {
      createRoot(el!).render(<App {...props} />)
    }
  },
})
"#
    .to_string()
}

fn ssr_tsx() -> String {
    r#"import { createInertiaApp } from '@inertiajs/react'
import type { ComponentType } from 'react'
import { renderToString } from 'react-dom/server.browser'

export async function render(pageJson: string): Promise<string> {
  const page = JSON.parse(pageJson)
  let html = ''

  await createInertiaApp({
    page,
    resolve: (name) => {
      const pages = import.meta.glob<{ default: ComponentType<any> }>(
        '../../modules/*/web/pages/**/*.tsx',
        { eager: true }
      )

      const [module, ...rest] = name.split('/')
      const pagePath = rest.join('/')
      const key = `../../modules/${module}/web/pages/${pagePath}.tsx`

      if (!pages[key]) {
        throw new Error(`SSR: Page not found: ${name} (looked for ${key})`)
      }

      return pages[key]
    },
    setup({ App, props }) {
      html = renderToString(<App {...props} />)
      return <App {...props} />
    },
  })

  return html
}
"#
    .to_string()
}

fn app_css() -> String {
    r#"@tailwind base;
@tailwind components;
@tailwind utilities;

body {
  @apply bg-slate-50 text-slate-950 antialiased;
}
"#
    .to_string()
}

fn app_layout_tsx(context: &AppTemplateContext) -> String {
    app_layout_template().replace("__APP_TITLE__", &context.app_title)
}

fn app_layout_template() -> &'static str {
    r#"import React from 'react'
import { Link, usePage } from '@inertiajs/react'
import FlashMessages from '@app/components/FlashMessages'

interface NavItem {
  label: string
  href: string
  icon?: string
}

interface ModuleNav {
  name: string
  nav_items: NavItem[]
}

interface SharedData {
  auth?: { user: { name: string; email: string } }
  modules?: ModuleNav[]
}

export default function AppLayout({ children }: { children: React.ReactNode }) {
  const { props } = usePage<SharedData & Record<string, unknown>>()
  const { auth, modules = [] } = props as SharedData

  return (
    <div className="min-h-screen bg-slate-50">
      <header className="border-b border-slate-200 bg-white">
        <div className="mx-auto flex max-w-6xl items-center justify-between px-6 py-4">
          <div>
            <h1 className="text-lg font-semibold text-slate-950">__APP_TITLE__</h1>
            <p className="text-sm text-slate-500">RWFW application</p>
          </div>
          {auth?.user && (
            <div className="text-right">
              <p className="text-sm font-medium text-slate-800">{auth.user.name}</p>
              <p className="text-xs text-slate-500">{auth.user.email}</p>
            </div>
          )}
        </div>
      </header>

      <div className="mx-auto grid max-w-6xl grid-cols-1 gap-8 px-6 py-8 md:grid-cols-[220px_1fr]">
        <aside>
          <nav className="space-y-1">
            {modules.map((mod) => (
              <div key={mod.name}>
                {mod.nav_items.map((item) => (
                  <Link
                    key={item.href}
                    href={item.href}
                    className="block rounded-lg px-3 py-2 text-sm font-medium text-slate-700 hover:bg-slate-200"
                  >
                    {item.label}
                  </Link>
                ))}
              </div>
            ))}
          </nav>
        </aside>

        <main>
          <FlashMessages />
          {children}
        </main>
      </div>
    </div>
  )
}
"#
}

fn auth_layout_tsx(context: &AppTemplateContext) -> String {
    auth_layout_template().replace("__APP_TITLE__", &context.app_title)
}

fn auth_layout_template() -> &'static str {
    r#"import React from 'react'
import { Link } from '@inertiajs/react'

export default function AuthLayout({ children }: { children: React.ReactNode }) {
  return (
    <div className="min-h-screen bg-slate-950 px-6 py-10 text-slate-950">
      <div className="mx-auto grid min-h-[calc(100vh-5rem)] max-w-6xl overflow-hidden rounded-[2rem] bg-white shadow-2xl lg:grid-cols-[1fr_440px]">
        <section className="hidden bg-[radial-gradient(circle_at_20%_20%,#fbbf24,transparent_28%),linear-gradient(135deg,#0f172a,#1e293b_55%,#334155)] p-10 text-white lg:flex lg:flex-col lg:justify-between">
          <div>
            <Link href="/home" className="text-sm font-semibold uppercase tracking-[0.3em] text-amber-200">
              __APP_TITLE__
            </Link>
            <h1 className="mt-16 max-w-lg text-5xl font-bold leading-tight">
              Auth, sessions, migrations and modules in one Rust app.
            </h1>
          </div>
          <p className="max-w-md text-sm leading-6 text-slate-300">
            This example protects write actions while keeping public pages readable.
          </p>
        </section>

        <main className="flex items-center justify-center p-6 sm:p-10">
          <div className="w-full max-w-md">{children}</div>
        </main>
      </div>
    </div>
  )
}
"#
}

fn auth_login_page_tsx() -> String {
    r#"import { Link, useForm, usePage } from '@inertiajs/react'
import AuthLayout from '@app/layouts/AuthLayout'

type SsoProvider = {
  name: string
  displayName: string
  loginUrl: string
}

export default function Login() {
  const { props } = usePage()
  const ssoProviders = ((props as any).ssoProviders ?? []) as SsoProvider[]
  const { data, setData, post, processing, errors } = useForm({
    email: '',
    password: '',
  })

  function submit(event: React.FormEvent) {
    event.preventDefault()
    post('/auth/login')
  }

  return (
    <AuthLayout>
      <div className="mb-8">
        <p className="text-sm font-semibold uppercase tracking-[0.2em] text-slate-500">Private routes</p>
        <h2 className="mt-3 text-3xl font-bold tracking-tight text-slate-950">Sign in</h2>
        <p className="mt-2 text-sm leading-6 text-slate-600">
          Sign in to create blog posts. Reading posts stays public.
        </p>
      </div>

      {ssoProviders.length > 0 && (
        <div className="mb-6 space-y-3">
          {ssoProviders.map((provider) => (
            <a
              key={provider.name}
              href={provider.loginUrl}
              className="block w-full rounded-xl border border-slate-300 px-4 py-3 text-center text-sm font-semibold text-slate-900 hover:bg-slate-50"
            >
              Continue with {provider.displayName}
            </a>
          ))}
          <div className="relative py-2">
            <div className="absolute inset-0 flex items-center">
              <div className="w-full border-t border-slate-200" />
            </div>
            <div className="relative flex justify-center text-xs uppercase tracking-[0.16em]">
              <span className="bg-white px-3 text-slate-500">or sign in locally</span>
            </div>
          </div>
        </div>
      )}

      <form onSubmit={submit} className="space-y-5">
        <div>
          <label htmlFor="email" className="mb-1 block text-sm font-medium text-slate-700">
            Email
          </label>
          <input
            id="email"
            type="email"
            value={data.email}
            onChange={(event) => setData('email', event.target.value)}
            className="w-full rounded-xl border border-slate-300 px-3 py-2 text-sm outline-none focus:border-slate-950"
            required
          />
          {errors.email && <p className="mt-1 text-sm text-red-600">{errors.email}</p>}
        </div>

        <div>
          <label htmlFor="password" className="mb-1 block text-sm font-medium text-slate-700">
            Password
          </label>
          <input
            id="password"
            type="password"
            value={data.password}
            onChange={(event) => setData('password', event.target.value)}
            className="w-full rounded-xl border border-slate-300 px-3 py-2 text-sm outline-none focus:border-slate-950"
            required
          />
          {errors.password && <p className="mt-1 text-sm text-red-600">{errors.password}</p>}
        </div>

        <button
          type="submit"
          disabled={processing}
          className="w-full rounded-xl bg-slate-950 px-4 py-3 text-sm font-semibold text-white hover:bg-slate-800 disabled:opacity-60"
        >
          {processing ? 'Signing in...' : 'Sign in'}
        </button>
      </form>

      <p className="mt-6 text-center text-sm text-slate-600">
        No account yet?{' '}
        <Link href="/auth/register" className="font-semibold text-slate-950 hover:underline">
          Create one
        </Link>
      </p>
    </AuthLayout>
  )
}
"#
    .to_string()
}

fn auth_register_page_tsx() -> String {
    r#"import { Link, useForm } from '@inertiajs/react'
import AuthLayout from '@app/layouts/AuthLayout'

export default function Register() {
  const { data, setData, post, processing, errors } = useForm({
    name: '',
    email: '',
    password: '',
    password_confirmation: '',
  })

  function submit(event: React.FormEvent) {
    event.preventDefault()
    post('/auth/register')
  }

  return (
    <AuthLayout>
      <div className="mb-8">
        <p className="text-sm font-semibold uppercase tracking-[0.2em] text-slate-500">Example user</p>
        <h2 className="mt-3 text-3xl font-bold tracking-tight text-slate-950">Create account</h2>
        <p className="mt-2 text-sm leading-6 text-slate-600">
          The first registered user is assigned the admin role by the auth module.
        </p>
      </div>

      <form onSubmit={submit} className="space-y-5">
        <div>
          <label htmlFor="name" className="mb-1 block text-sm font-medium text-slate-700">
            Name
          </label>
          <input
            id="name"
            value={data.name}
            onChange={(event) => setData('name', event.target.value)}
            className="w-full rounded-xl border border-slate-300 px-3 py-2 text-sm outline-none focus:border-slate-950"
            required
          />
          {errors.name && <p className="mt-1 text-sm text-red-600">{errors.name}</p>}
        </div>

        <div>
          <label htmlFor="email" className="mb-1 block text-sm font-medium text-slate-700">
            Email
          </label>
          <input
            id="email"
            type="email"
            value={data.email}
            onChange={(event) => setData('email', event.target.value)}
            className="w-full rounded-xl border border-slate-300 px-3 py-2 text-sm outline-none focus:border-slate-950"
            required
          />
          {errors.email && <p className="mt-1 text-sm text-red-600">{errors.email}</p>}
        </div>

        <div>
          <label htmlFor="password" className="mb-1 block text-sm font-medium text-slate-700">
            Password
          </label>
          <input
            id="password"
            type="password"
            value={data.password}
            onChange={(event) => setData('password', event.target.value)}
            className="w-full rounded-xl border border-slate-300 px-3 py-2 text-sm outline-none focus:border-slate-950"
            required
          />
          {errors.password && <p className="mt-1 text-sm text-red-600">{errors.password}</p>}
        </div>

        <div>
          <label htmlFor="password_confirmation" className="mb-1 block text-sm font-medium text-slate-700">
            Confirm password
          </label>
          <input
            id="password_confirmation"
            type="password"
            value={data.password_confirmation}
            onChange={(event) => setData('password_confirmation', event.target.value)}
            className="w-full rounded-xl border border-slate-300 px-3 py-2 text-sm outline-none focus:border-slate-950"
            required
          />
        </div>

        <button
          type="submit"
          disabled={processing}
          className="w-full rounded-xl bg-slate-950 px-4 py-3 text-sm font-semibold text-white hover:bg-slate-800 disabled:opacity-60"
        >
          {processing ? 'Creating...' : 'Create account'}
        </button>
      </form>

      <p className="mt-6 text-center text-sm text-slate-600">
        Already registered?{' '}
        <Link href="/auth/login" className="font-semibold text-slate-950 hover:underline">
          Sign in
        </Link>
      </p>
    </AuthLayout>
  )
}
"#
    .to_string()
}

fn flash_messages_tsx() -> String {
    r#"import { usePage } from '@inertiajs/react'

export default function FlashMessages() {
  const { props } = usePage()
  const flash = (props as any).flash

  if (!flash) return null

  return (
    <div className="mb-6 space-y-2">
      {flash.success && (
        <div className="rounded-lg bg-emerald-100 p-4 text-sm text-emerald-800">
          {flash.success}
        </div>
      )}
      {flash.error && (
        <div className="rounded-lg bg-red-100 p-4 text-sm text-red-800">
          {flash.error}
        </div>
      )}
      {flash.info && (
        <div className="rounded-lg bg-sky-100 p-4 text-sm text-sky-800">
          {flash.info}
        </div>
      )}
    </div>
  )
}
"#
    .to_string()
}

fn use_auth_ts() -> String {
    r#"import { usePage } from '@inertiajs/react'

export function useAuth() {
  const { props } = usePage()
  const auth = (props as any).auth

  return {
    user: auth?.user ?? null,
    roles: auth?.roles ?? [],
    permissions: auth?.permissions ?? [],
    isAuthenticated: !!auth?.user,
    can: (permission: string) =>
      (auth?.roles ?? []).includes('admin') || (auth?.permissions ?? []).includes(permission),
    hasRole: (role: string) => (auth?.roles ?? []).includes(role),
  }
}
"#
    .to_string()
}

fn inertia_types_ts() -> String {
    r#"import { PageProps } from '@inertiajs/react'

declare module '@inertiajs/react' {
  interface PageProps {
    auth?: {
      user: {
        id: string
        name: string
        email: string
      }
      roles: string[]
      permissions: string[]
    }
    flash?: {
      success?: string
      error?: string
      info?: string
    }
    errors?: Record<string, string>
    csrf_token?: string
    modules?: Array<{
      name: string
      nav_items: Array<{
        label: string
        href: string
        icon?: string
      }>
    }>
  }
}
"#
    .to_string()
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
    r#"use axum::response::IntoResponse;
use axum::routing;
use rwfw_core::app::AppState;
use rwfw_core::inertia::Inertia;

pub fn route() -> axum::routing::MethodRouter<AppState> {
    routing::get(get)
}

async fn get(i: Inertia) -> impl IntoResponse {
    i.render_with_ssr(
        "home/Index",
        serde_json::json!({
            "title": "Welcome to RWFW",
            "description": "A modular Rust web framework with React + Inertia + SSR"
        }),
    )
    .await
}
"#
    .to_string()
}

fn home_index_page_tsx(context: &AppTemplateContext) -> String {
    home_index_page_template().replace("__APP_TITLE__", &context.app_title)
}

fn home_index_page_template() -> &'static str {
    r#"import AppLayout from '@app/layouts/AppLayout'

interface Props {
  title: string
  description: string
}

export default function HomeIndex({ title, description }: Props) {
  return (
    <AppLayout>
      <section className="rounded-3xl border border-slate-200 bg-white p-8 shadow-sm">
        <p className="mb-3 text-sm font-semibold uppercase tracking-[0.2em] text-slate-500">
          __APP_TITLE__
        </p>
        <h2 className="mb-4 max-w-2xl text-4xl font-bold tracking-tight text-slate-950">
          {title}
        </h2>
        <p className="mb-8 max-w-2xl text-lg leading-8 text-slate-600">{description}</p>
        <div className="grid grid-cols-1 gap-4 md:grid-cols-3">
          {['Modules', 'Migrations', 'Single binary'].map((item) => (
            <div key={item} className="rounded-2xl bg-slate-100 p-5">
              <h3 className="font-semibold text-slate-900">{item}</h3>
              <p className="mt-2 text-sm text-slate-600">
                Convention-first primitives for internal products.
              </p>
            </div>
          ))}
        </div>
      </section>
    </AppLayout>
  )
}
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
    pub slug: Option<String>,
    pub excerpt: String,
    pub body: String,
    pub cover_image_url: Option<String>,
    pub status: String,
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
use rwfw_core::inertia::Inertia;

pub fn route() -> axum::routing::MethodRouter<AppState> {
    routing::get(get)
}

async fn get(State(state): State<AppState>, i: Inertia) -> Response {
    let repo = PostRepository::new(state.db.clone());
    let posts = match repo.find_published().await {
        Ok(posts) => posts,
        Err(error) => return AppError::Internal(error).into_response(),
    };

    i.render_with_ssr(
        "blog/Index",
        serde_json::json!({
            "posts": posts,
        }),
    )
    .await
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
use rwfw_core::inertia::Inertia;

pub fn route() -> axum::routing::MethodRouter<AppState> {
    routing::get(get)
}

async fn get(State(state): State<AppState>, i: Inertia) -> Response {
    let repo = PostRepository::new(state.db.clone());
    match repo.find_published().await {
        Ok(posts) => i
            .render_with_ssr(
                "blog/posts/Index",
                serde_json::json!({
                    "posts": posts,
                }),
            )
            .await,
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
use rwfw_core::inertia::Inertia;

pub fn route() -> axum::routing::MethodRouter<AppState> {
    routing::get(get)
}

async fn get(
    State(state): State<AppState>,
    i: Inertia,
    Path(slug): Path<String>,
) -> Response {
    let repo = PostRepository::new(state.db.clone());
    let post = match repo.find_published_by_slug(&slug).await {
        Ok(Some(post)) => post,
        Ok(None) => return AppError::NotFound(format!("Post not found: {slug}")).into_response(),
        Err(error) => return AppError::Internal(error).into_response(),
    };

    i.render_with_ssr(
        "blog/posts/Show",
        serde_json::json!({
            "post": post,
        }),
    )
    .await
}
"#
    .to_string()
}

fn blog_admin_posts_index_route_rs() -> String {
    r#"use crate::repositories::post_repo::PostRepository;
use crate::use_cases::save_post::{SavePostInput, SavePostUseCase};
use axum::extract::{Json, State};
use axum::response::{IntoResponse, Response};
use axum::routing;
use rwfw_core::app::AppState;
use rwfw_core::auth::CurrentUser;
use rwfw_core::error::AppError;
use rwfw_core::inertia::Inertia;

pub fn route() -> axum::routing::MethodRouter<AppState> {
    routing::get(get).post(post)
}

async fn get(State(state): State<AppState>, user: CurrentUser, i: Inertia) -> Response {
    if !user.can("blog.posts.manage") {
        return AppError::Forbidden("Missing permission: blog.posts.manage".into()).into_response();
    }

    let repo = PostRepository::new(state.db.clone());
    let posts = match repo.find_all_for_admin().await {
        Ok(posts) => posts,
        Err(error) => return AppError::Internal(error).into_response(),
    };

    i.render_with_ssr(
        "blog/admin/posts/Index",
        serde_json::json!({
            "posts": posts,
        }),
    )
    .await
}

async fn post(
    State(state): State<AppState>,
    user: CurrentUser,
    i: Inertia,
    Json(input): Json<SavePostInput>,
) -> Response {
    if !user.can("blog.posts.manage") {
        return AppError::Forbidden("Missing permission: blog.posts.manage".into()).into_response();
    }

    let repo = PostRepository::new(state.db.clone());
    let use_case = SavePostUseCase;
    match use_case.create(&repo, user.id, input).await {
        Ok(post) => Inertia::redirect_with_success(
            &format!("/blog/admin/posts/{}/edit", post.id),
            "Post created",
        ),
        Err(AppError::Validation(errors)) => i.redirect_back_with_errors(errors),
        Err(error) => error.into_response(),
    }
}
"#
    .to_string()
}

fn blog_admin_posts_new_route_rs() -> String {
    r#"use axum::response::IntoResponse;
use axum::routing;
use rwfw_core::app::AppState;
use rwfw_core::auth::CurrentUser;
use rwfw_core::error::AppError;
use rwfw_core::inertia::Inertia;

pub fn route() -> axum::routing::MethodRouter<AppState> {
    routing::get(get)
}

async fn get(user: CurrentUser, i: Inertia) -> impl IntoResponse {
    if !user.can("blog.posts.manage") {
        return AppError::Forbidden("Missing permission: blog.posts.manage".into()).into_response();
    }

    i.render_with_ssr(
        "blog/admin/posts/New",
        serde_json::json!({
            "post": serde_json::Value::Null,
        }),
    )
    .await
}
"#
    .to_string()
}

fn blog_admin_post_item_route_rs() -> String {
    r#"use crate::repositories::post_repo::PostRepository;
use axum::extract::{Path, State};
use axum::response::{IntoResponse, Response};
use axum::routing;
use rwfw_core::app::AppState;
use rwfw_core::auth::CurrentUser;
use rwfw_core::error::AppError;
use rwfw_core::inertia::Inertia;

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
        Ok(()) => Inertia::redirect_with_success("/blog/admin/posts", "Post deleted"),
        Err(error) => AppError::Internal(error).into_response(),
    }
}
"#
    .to_string()
}

fn blog_admin_post_edit_route_rs() -> String {
    r#"use crate::repositories::post_repo::PostRepository;
use crate::use_cases::save_post::{SavePostInput, SavePostUseCase};
use axum::extract::{Json, Path, State};
use axum::response::{IntoResponse, Response};
use axum::routing;
use rwfw_core::app::AppState;
use rwfw_core::auth::CurrentUser;
use rwfw_core::error::AppError;
use rwfw_core::inertia::Inertia;

pub fn route() -> axum::routing::MethodRouter<AppState> {
    routing::get(get).put(put)
}

async fn get(
    State(state): State<AppState>,
    user: CurrentUser,
    i: Inertia,
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

    i.render_with_ssr(
        "blog/admin/posts/Edit",
        serde_json::json!({
            "post": post,
        }),
    )
    .await
}

async fn put(
    State(state): State<AppState>,
    user: CurrentUser,
    i: Inertia,
    Path(id): Path<i32>,
    Json(input): Json<SavePostInput>,
) -> Response {
    if !user.can("blog.posts.manage") {
        return AppError::Forbidden("Missing permission: blog.posts.manage".into()).into_response();
    }

    let repo = PostRepository::new(state.db.clone());
    let use_case = SavePostUseCase;
    match use_case.update(&repo, id, user.id, input).await {
        Ok(post) => Inertia::redirect_with_success(
            &format!("/blog/admin/posts/{}/edit", post.id),
            "Post saved",
        ),
        Err(AppError::Validation(errors)) => i.redirect_back_with_errors(errors),
        Err(error) => error.into_response(),
    }
}
"#
    .to_string()
}

fn blog_index_page_tsx() -> String {
    r#"import { Link, usePage } from '@inertiajs/react'
import AppLayout from '@app/layouts/AppLayout'

interface Post {
  id: number
  title: string
  slug: string
  excerpt: string
  cover_image_url?: string
  published_at?: string
}

interface Props {
  posts: Post[]
}

interface SharedData {
  auth?: { user?: { name: string } }
}

export default function BlogIndex({ posts }: Props) {
  const featured = posts[0]
  const latest = posts.slice(1, 5)
  const { props } = usePage<SharedData & Record<string, unknown>>()
  const isSignedIn = Boolean(props.auth?.user)

  return (
    <AppLayout>
      <div className="-m-8 min-h-screen bg-[#f4efe6] px-6 py-8 text-stone-950 md:px-10">
        <section className="mx-auto max-w-6xl overflow-hidden rounded-[2rem] border border-stone-900/10 bg-[#fffaf0] shadow-[0_30px_80px_rgba(41,31,20,0.16)]">
          <div className="grid gap-0 lg:grid-cols-[1.1fr_0.9fr]">
            <div className="p-8 md:p-12">
              <p className="text-xs font-black uppercase tracking-[0.35em] text-orange-700">
                RWFW Journal
              </p>
              <h1 className="mt-6 max-w-3xl text-5xl font-black leading-[0.95] tracking-tight md:text-7xl">
                Um blog CMS com cara de produto real.
              </h1>
              <p className="mt-6 max-w-2xl text-lg leading-8 text-stone-700">
                Este exemplo demonstra rotas públicas, admin protegido, migrations por módulo,
                repository SeaORM e páginas Inertia com React.
              </p>
              <div className="mt-8 flex flex-wrap gap-3">
                <Link
                  href="/blog/posts"
                  className="rounded-full bg-stone-950 px-6 py-3 text-sm font-black text-white hover:bg-stone-800"
                >
                  Ler posts
                </Link>
                <Link
                  href={isSignedIn ? '/blog/admin/posts' : '/auth/login'}
                  className="rounded-full border border-stone-300 bg-white px-6 py-3 text-sm font-black text-stone-950 hover:border-stone-950"
                >
                  {isSignedIn ? 'Abrir admin' : 'Login para publicar'}
                </Link>
              </div>
            </div>

            <div className="relative min-h-[420px] bg-stone-950 p-8 text-white">
              <div className="absolute inset-0 bg-[radial-gradient(circle_at_20%_20%,rgba(251,146,60,0.5),transparent_30%),radial-gradient(circle_at_80%_20%,rgba(20,184,166,0.35),transparent_30%)]" />
              <div className="relative flex h-full flex-col justify-end">
                {featured ? (
                  <article>
                    {featured.cover_image_url && (
                      <img
                        src={featured.cover_image_url}
                        alt=""
                        className="mb-8 h-48 w-full rounded-3xl object-cover opacity-90"
                      />
                    )}
                    <p className="text-xs font-black uppercase tracking-[0.3em] text-orange-200">
                      Destaque
                    </p>
                    <h2 className="mt-3 text-3xl font-black leading-tight">{featured.title}</h2>
                    <p className="mt-4 text-sm leading-6 text-stone-200">{featured.excerpt}</p>
                    <Link
                      href={`/blog/posts/${featured.slug}`}
                      className="mt-6 inline-flex text-sm font-black text-orange-200 hover:text-white"
                    >
                      Continuar leitura
                    </Link>
                  </article>
                ) : (
                  <p className="text-stone-200">Publique o primeiro post no admin.</p>
                )}
              </div>
            </div>
          </div>
        </section>

        <section className="mx-auto mt-8 grid max-w-6xl gap-5 md:grid-cols-2">
          {latest.map((post) => (
            <Link
              key={post.id}
              href={`/blog/posts/${post.slug}`}
              className="rounded-[1.5rem] border border-stone-900/10 bg-white/70 p-6 shadow-sm transition hover:-translate-y-1 hover:bg-white"
            >
              <p className="text-xs font-black uppercase tracking-[0.25em] text-stone-500">
                Artigo
              </p>
              <h3 className="mt-3 text-2xl font-black tracking-tight">{post.title}</h3>
              <p className="mt-3 text-sm leading-6 text-stone-600">{post.excerpt}</p>
            </Link>
          ))}
        </section>
      </div>
    </AppLayout>
  )
}
"#
    .to_string()
}

fn blog_posts_index_page_tsx() -> String {
    r#"import { Link, usePage } from '@inertiajs/react'
import AppLayout from '@app/layouts/AppLayout'

interface Post {
  id: number
  title: string
  slug: string
  excerpt: string
  cover_image_url?: string
  published_at?: string
}

interface Props {
  posts: Post[]
}

interface SharedData {
  auth?: { user?: { name: string; email: string } }
}

export default function BlogPostsIndex({ posts }: Props) {
  const { props } = usePage<SharedData & Record<string, unknown>>()
  const isSignedIn = Boolean(props.auth?.user)

  return (
    <AppLayout>
      <div className="-m-8 min-h-screen bg-stone-950 px-6 py-10 text-white md:px-10">
        <div className="mx-auto max-w-6xl">
          <div className="mb-10 flex flex-col justify-between gap-6 md:flex-row md:items-end">
            <div>
              <p className="text-xs font-black uppercase tracking-[0.35em] text-orange-300">
                Public archive
              </p>
              <h1 className="mt-4 text-5xl font-black tracking-tight md:text-6xl">
                Posts publicados
              </h1>
              <p className="mt-4 max-w-2xl text-stone-300">
                A leitura é pública. A criação, edição e publicação ficam no admin protegido.
              </p>
            </div>
            <Link
              href={isSignedIn ? '/blog/admin/posts' : '/auth/login'}
              className="rounded-full bg-orange-300 px-6 py-3 text-center text-sm font-black text-stone-950 hover:bg-orange-200"
            >
              {isSignedIn ? 'Gerenciar posts' : 'Login para publicar'}
            </Link>
          </div>

          {posts.length === 0 ? (
            <div className="rounded-[2rem] border border-white/10 bg-white/5 p-10 text-stone-300">
              Nenhum post publicado ainda.
            </div>
          ) : (
            <div className="grid gap-5 md:grid-cols-2">
              {posts.map((post, index) => (
                <Link
                  key={post.id}
                  href={`/blog/posts/${post.slug}`}
                  className={`group overflow-hidden rounded-[2rem] border border-white/10 bg-white/[0.06] transition hover:-translate-y-1 hover:bg-white/[0.1] ${
                    index === 0 ? 'md:col-span-2 md:grid md:grid-cols-[0.9fr_1.1fr]' : ''
                  }`}
                >
                  {post.cover_image_url && (
                    <img
                      src={post.cover_image_url}
                      alt=""
                      className="h-56 w-full object-cover opacity-85 transition group-hover:opacity-100 md:h-full"
                    />
                  )}
                  <div className="p-7">
                    <p className="text-xs font-black uppercase tracking-[0.25em] text-orange-300">
                      {post.published_at ? new Date(post.published_at).toLocaleDateString() : 'Publicado'}
                    </p>
                    <h2 className="mt-4 text-3xl font-black tracking-tight">{post.title}</h2>
                    <p className="mt-4 leading-7 text-stone-300">{post.excerpt}</p>
                    <span className="mt-6 inline-flex text-sm font-black text-orange-200">
                      Ler artigo
                    </span>
                  </div>
                </Link>
              ))}
            </div>
          )}
        </div>
      </div>
    </AppLayout>
  )
}
"#
    .to_string()
}

fn blog_post_show_page_tsx() -> String {
    r#"import { Link } from '@inertiajs/react'
import AppLayout from '@app/layouts/AppLayout'

interface Post {
  id: number
  title: string
  slug: string
  excerpt: string
  body: string
  cover_image_url?: string
  published_at?: string
}

interface Props {
  post: Post
}

export default function BlogPostShow({ post }: Props) {
  return (
    <AppLayout>
      <article className="-m-8 min-h-screen bg-[#f7f0df] px-6 py-10 text-stone-950 md:px-10">
        <div className="mx-auto max-w-4xl">
          <Link href="/blog/posts" className="text-sm font-black text-orange-700 hover:text-stone-950">
            Voltar para posts
          </Link>
          <header className="mt-8">
            <p className="text-xs font-black uppercase tracking-[0.35em] text-stone-500">
              RWFW Journal
            </p>
            <h1 className="mt-5 text-5xl font-black leading-tight tracking-tight md:text-7xl">
              {post.title}
            </h1>
            <p className="mt-6 max-w-3xl text-xl leading-9 text-stone-700">{post.excerpt}</p>
          </header>

          {post.cover_image_url && (
            <img
              src={post.cover_image_url}
              alt=""
              className="mt-10 h-[420px] w-full rounded-[2rem] object-cover shadow-2xl"
            />
          )}

          <div className="mt-12 whitespace-pre-wrap rounded-[2rem] bg-white p-8 text-xl leading-10 text-stone-800 shadow-sm md:p-12">
            {post.body}
          </div>
        </div>
      </article>
    </AppLayout>
  )
}
"#
    .to_string()
}

fn blog_admin_posts_index_page_tsx() -> String {
    r#"import { Link, router } from '@inertiajs/react'
import AppLayout from '@app/layouts/AppLayout'

interface Post {
  id: number
  title: string
  slug: string
  excerpt: string
  status: 'draft' | 'published'
  updated_at: string
}

interface Props {
  posts: Post[]
}

export default function BlogAdminPostsIndex({ posts }: Props) {
  function deletePost(post: Post) {
    if (confirm(`Delete "${post.title}"?`)) {
      router.delete(`/blog/admin/posts/${post.id}`)
    }
  }

  return (
    <AppLayout>
      <div className="rounded-[2rem] border border-slate-200 bg-white p-8 shadow-sm">
        <div className="mb-8 flex flex-col justify-between gap-4 md:flex-row md:items-center">
          <div>
            <p className="text-xs font-black uppercase tracking-[0.25em] text-slate-500">
              Protected CMS
            </p>
            <h1 className="mt-3 text-4xl font-black tracking-tight text-slate-950">
              Blog admin
            </h1>
          </div>
          <Link
            href="/blog/admin/posts/new"
            className="rounded-full bg-slate-950 px-5 py-3 text-center text-sm font-black text-white hover:bg-slate-800"
          >
            New post
          </Link>
        </div>

        <div className="overflow-hidden rounded-2xl border border-slate-200">
          {posts.length === 0 ? (
            <div className="p-8 text-slate-500">No posts yet.</div>
          ) : (
            <table className="w-full text-left text-sm">
              <thead className="bg-slate-100 text-xs uppercase tracking-[0.2em] text-slate-500">
                <tr>
                  <th className="px-5 py-4">Post</th>
                  <th className="px-5 py-4">Status</th>
                  <th className="px-5 py-4 text-right">Actions</th>
                </tr>
              </thead>
              <tbody className="divide-y divide-slate-200">
                {posts.map((post) => (
                  <tr key={post.id} className="bg-white">
                    <td className="px-5 py-4">
                      <div className="font-black text-slate-950">{post.title}</div>
                      <div className="mt-1 text-xs text-slate-500">/{post.slug}</div>
                    </td>
                    <td className="px-5 py-4">
                      <span className={`rounded-full px-3 py-1 text-xs font-black ${
                        post.status === 'published'
                          ? 'bg-emerald-100 text-emerald-800'
                          : 'bg-amber-100 text-amber-800'
                      }`}>
                        {post.status}
                      </span>
                    </td>
                    <td className="px-5 py-4">
                      <div className="flex justify-end gap-2">
                        {post.status === 'published' && (
                          <Link
                            href={`/blog/posts/${post.slug}`}
                            className="rounded-full border border-slate-200 px-3 py-2 text-xs font-black text-slate-700 hover:border-slate-950"
                          >
                            View
                          </Link>
                        )}
                        <Link
                          href={`/blog/admin/posts/${post.id}/edit`}
                          className="rounded-full border border-slate-200 px-3 py-2 text-xs font-black text-slate-700 hover:border-slate-950"
                        >
                          Edit
                        </Link>
                        <button
                          type="button"
                          onClick={() => deletePost(post)}
                          className="rounded-full bg-red-50 px-3 py-2 text-xs font-black text-red-700 hover:bg-red-100"
                        >
                          Delete
                        </button>
                      </div>
                    </td>
                  </tr>
                ))}
              </tbody>
            </table>
          )}
        </div>
      </div>
    </AppLayout>
  )
}
"#
    .to_string()
}

fn blog_admin_post_form_tsx() -> String {
    r#"import { useForm } from '@inertiajs/react'
import type { FormEvent } from 'react'

export interface BlogPostFormValue {
  id?: number
  title: string
  slug?: string
  excerpt: string
  body: string
  cover_image_url?: string
  status: 'draft' | 'published'
}

interface Props {
  post?: BlogPostFormValue | null
  submitTo: string
  method: 'post' | 'put'
}

export default function BlogPostForm({ post, submitTo, method }: Props) {
  const { data, setData, post: create, put, processing, errors } = useForm({
    title: post?.title ?? '',
    slug: post?.slug ?? '',
    excerpt: post?.excerpt ?? '',
    body: post?.body ?? '',
    cover_image_url: post?.cover_image_url ?? '',
    status: post?.status ?? 'draft',
  })

  function submit(event: FormEvent) {
    event.preventDefault()
    if (method === 'post') {
      create(submitTo)
    } else {
      put(submitTo)
    }
  }

  return (
    <form onSubmit={submit} className="space-y-6">
      <div className="grid gap-5 md:grid-cols-2">
        <div>
          <label htmlFor="title" className="mb-2 block text-sm font-black text-slate-700">
            Title
          </label>
          <input
            id="title"
            value={data.title}
            onChange={(event) => setData('title', event.target.value)}
            className="w-full rounded-2xl border border-slate-300 px-4 py-3 outline-none focus:border-slate-950"
          />
          {errors.title && <p className="mt-2 text-sm text-red-600">{errors.title}</p>}
        </div>

        <div>
          <label htmlFor="slug" className="mb-2 block text-sm font-black text-slate-700">
            Slug
          </label>
          <input
            id="slug"
            value={data.slug}
            onChange={(event) => setData('slug', event.target.value)}
            placeholder="generated-from-title"
            className="w-full rounded-2xl border border-slate-300 px-4 py-3 outline-none focus:border-slate-950"
          />
        </div>
      </div>

      <div>
        <label htmlFor="excerpt" className="mb-2 block text-sm font-black text-slate-700">
          Excerpt
        </label>
        <textarea
          id="excerpt"
          rows={3}
          value={data.excerpt}
          onChange={(event) => setData('excerpt', event.target.value)}
          className="w-full rounded-2xl border border-slate-300 px-4 py-3 outline-none focus:border-slate-950"
        />
        {errors.excerpt && <p className="mt-2 text-sm text-red-600">{errors.excerpt}</p>}
      </div>

      <div>
        <label htmlFor="cover" className="mb-2 block text-sm font-black text-slate-700">
          Cover image URL
        </label>
        <input
          id="cover"
          value={data.cover_image_url}
          onChange={(event) => setData('cover_image_url', event.target.value)}
          className="w-full rounded-2xl border border-slate-300 px-4 py-3 outline-none focus:border-slate-950"
        />
      </div>

      <div>
        <label htmlFor="body" className="mb-2 block text-sm font-black text-slate-700">
          Body
        </label>
        <textarea
          id="body"
          rows={14}
          value={data.body}
          onChange={(event) => setData('body', event.target.value)}
          className="w-full rounded-2xl border border-slate-300 px-4 py-3 leading-7 outline-none focus:border-slate-950"
        />
        {errors.body && <p className="mt-2 text-sm text-red-600">{errors.body}</p>}
      </div>

      <div className="flex flex-col justify-between gap-4 rounded-2xl bg-slate-100 p-4 md:flex-row md:items-center">
        <select
          value={data.status}
          onChange={(event) => setData('status', event.target.value as 'draft' | 'published')}
          className="rounded-xl border border-slate-300 bg-white px-4 py-3 text-sm font-black"
        >
          <option value="draft">Draft</option>
          <option value="published">Published</option>
        </select>
        <button
          type="submit"
          disabled={processing}
          className="rounded-full bg-slate-950 px-6 py-3 text-sm font-black text-white hover:bg-slate-800 disabled:opacity-60"
        >
          {processing ? 'Saving...' : 'Save post'}
        </button>
      </div>
    </form>
  )
}
"#
    .to_string()
}

fn blog_admin_post_new_page_tsx() -> String {
    r#"import { Link } from '@inertiajs/react'
import AppLayout from '@app/layouts/AppLayout'
import BlogPostForm from './Form'

export default function BlogAdminPostNew() {
  return (
    <AppLayout>
      <div className="rounded-[2rem] border border-slate-200 bg-white p-8 shadow-sm">
        <Link href="/blog/admin/posts" className="text-sm font-black text-slate-500 hover:text-slate-950">
          Back to admin
        </Link>
        <h1 className="mt-4 text-4xl font-black tracking-tight text-slate-950">New post</h1>
        <div className="mt-8">
          <BlogPostForm submitTo="/blog/admin/posts" method="post" />
        </div>
      </div>
    </AppLayout>
  )
}
"#
    .to_string()
}

fn blog_admin_post_edit_page_tsx() -> String {
    r#"import { Link } from '@inertiajs/react'
import AppLayout from '@app/layouts/AppLayout'
import BlogPostForm, { type BlogPostFormValue } from './Form'

interface Props {
  post: BlogPostFormValue
}

export default function BlogAdminPostEdit({ post }: Props) {
  return (
    <AppLayout>
      <div className="rounded-[2rem] border border-slate-200 bg-white p-8 shadow-sm">
        <div className="flex flex-col justify-between gap-4 md:flex-row md:items-center">
          <div>
            <Link href="/blog/admin/posts" className="text-sm font-black text-slate-500 hover:text-slate-950">
              Back to admin
            </Link>
            <h1 className="mt-4 text-4xl font-black tracking-tight text-slate-950">Edit post</h1>
          </div>
          {post.status === 'published' && post.slug && (
            <Link
              href={`/blog/posts/${post.slug}`}
              className="rounded-full border border-slate-300 px-5 py-3 text-sm font-black text-slate-700 hover:border-slate-950"
            >
              View public post
            </Link>
          )}
        </div>
        <div className="mt-8">
          <BlogPostForm post={post} submitTo={`/blog/admin/posts/${post.id}/edit`} method="put" />
        </div>
      </div>
    </AppLayout>
  )
}
"#
    .to_string()
}

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
        &app_dir.join("crates/modules/shop/src/routes/checkout/index.rs"),
        shop_checkout_route_rs(),
    )?;
    write_file(
        &app_dir.join("crates/modules/shop/src/routes/orders/[number].rs"),
        shop_order_show_route_rs(),
    )?;
    write_file(
        &app_dir.join("crates/modules/shop/web/components/cart.ts"),
        shop_cart_ts(),
    )?;
    write_file(
        &app_dir.join("crates/modules/shop/web/components/ProductCard.tsx"),
        shop_product_card_tsx(),
    )?;
    write_file(
        &app_dir.join("crates/modules/shop/web/pages/Index.tsx"),
        shop_index_page_tsx(),
    )?;
    write_file(
        &app_dir.join("crates/modules/shop/web/pages/products/Show.tsx"),
        shop_product_show_page_tsx(),
    )?;
    write_file(
        &app_dir.join("crates/modules/shop/web/pages/cart/Index.tsx"),
        shop_cart_page_tsx(),
    )?;
    write_file(
        &app_dir.join("crates/modules/shop/web/pages/checkout/Index.tsx"),
        shop_checkout_page_tsx(),
    )?;
    write_file(
        &app_dir.join("crates/modules/shop/web/pages/orders/Show.tsx"),
        shop_order_show_page_tsx(),
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
    r#"pub mod migrations;
pub mod models;
pub mod repositories;
pub mod use_cases;

use axum::Router;
use rwfw_core::app::AppState;
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
}

inventory::submit! {
    ModuleRegistration::new("shop", || Box::new(ShopModule::new()))
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
    pub items: Vec<CheckoutLineInput>,
}

#[derive(Debug, Deserialize)]
pub struct CheckoutLineInput {
    pub product_id: i32,
    pub quantity: i32,
}

pub struct CheckoutUseCase;

impl CheckoutUseCase {
    pub async fn execute(
        &self,
        repo: &ShopRepository,
        input: CheckoutInput,
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
        for item in input.items {
            if item.quantity > 0 {
                let quantity = item.quantity.min(20);
                *quantities.entry(item.product_id).or_insert(0) += quantity;
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

fn shop_index_route_rs() -> String {
    r#"use crate::repositories::shop_repo::ShopRepository;
use axum::extract::{Query, State};
use axum::response::{IntoResponse, Response};
use axum::routing;
use rwfw_core::app::AppState;
use rwfw_core::error::AppError;
use rwfw_core::inertia::Inertia;
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
    i: Inertia,
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

    i.render_with_ssr(
        "shop/Index",
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
    .await
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
use rwfw_core::inertia::Inertia;

pub fn route() -> axum::routing::MethodRouter<AppState> {
    routing::get(get)
}

async fn get(
    State(state): State<AppState>,
    i: Inertia,
    Path(slug): Path<String>,
) -> Response {
    let repo = ShopRepository::new(state.db.clone());
    let product = match repo.find_product_by_slug(&slug).await {
        Ok(Some(product)) => product,
        Ok(None) => return AppError::NotFound(format!("Product not found: {slug}")).into_response(),
        Err(error) => return AppError::Internal(error).into_response(),
    };

    i.render_with_ssr(
        "shop/products/Show",
        serde_json::json!({
            "product": product,
        }),
    )
    .await
}
"#
    .to_string()
}

fn shop_cart_route_rs() -> String {
    r#"use crate::repositories::shop_repo::ShopRepository;
use axum::extract::State;
use axum::response::{IntoResponse, Response};
use axum::routing;
use rwfw_core::app::AppState;
use rwfw_core::error::AppError;
use rwfw_core::inertia::Inertia;

pub fn route() -> axum::routing::MethodRouter<AppState> {
    routing::get(get)
}

async fn get(State(state): State<AppState>, i: Inertia) -> Response {
    let repo = ShopRepository::new(state.db.clone());
    let products = match repo.products(None, None).await {
        Ok(products) => products,
        Err(error) => return AppError::Internal(error).into_response(),
    };

    i.render_with_ssr(
        "shop/cart/Index",
        serde_json::json!({
            "products": products,
        }),
    )
    .await
}
"#
    .to_string()
}

fn shop_checkout_route_rs() -> String {
    r#"use crate::repositories::shop_repo::ShopRepository;
use crate::use_cases::checkout::{CheckoutInput, CheckoutUseCase};
use axum::extract::{Json, State};
use axum::response::{IntoResponse, Response};
use axum::routing;
use rwfw_core::app::AppState;
use rwfw_core::error::AppError;
use rwfw_core::inertia::Inertia;

pub fn route() -> axum::routing::MethodRouter<AppState> {
    routing::get(get).post(post)
}

async fn get(State(state): State<AppState>, i: Inertia) -> Response {
    let repo = ShopRepository::new(state.db.clone());
    let products = match repo.products(None, None).await {
        Ok(products) => products,
        Err(error) => return AppError::Internal(error).into_response(),
    };

    i.render_with_ssr(
        "shop/checkout/Index",
        serde_json::json!({
            "products": products,
        }),
    )
    .await
}

async fn post(
    State(state): State<AppState>,
    i: Inertia,
    Json(input): Json<CheckoutInput>,
) -> Response {
    let repo = ShopRepository::new(state.db.clone());
    let use_case = CheckoutUseCase;

    match use_case.execute(&repo, input).await {
        Ok(order) => Inertia::redirect_with_success(
            &format!("/shop/orders/{}", order.number),
            "Fake checkout completed",
        ),
        Err(AppError::Validation(errors)) => i.redirect_back_with_errors(errors),
        Err(error) => error.into_response(),
    }
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
use rwfw_core::inertia::Inertia;

pub fn route() -> axum::routing::MethodRouter<AppState> {
    routing::get(get)
}

async fn get(
    State(state): State<AppState>,
    i: Inertia,
    Path(number): Path<String>,
) -> Response {
    let repo = ShopRepository::new(state.db.clone());
    let (order, items) = match repo.find_order(&number).await {
        Ok(Some(order)) => order,
        Ok(None) => return AppError::NotFound(format!("Order not found: {number}")).into_response(),
        Err(error) => return AppError::Internal(error).into_response(),
    };

    i.render_with_ssr(
        "shop/orders/Show",
        serde_json::json!({
            "order": order,
            "items": items,
        }),
    )
    .await
}
"#
    .to_string()
}

fn shop_cart_ts() -> String {
    r#"export interface CartItem {
  product_id: number
  quantity: number
}

const CART_KEY = 'rwfw_shop_cart'

export function readCart(): CartItem[] {
  if (typeof window === 'undefined') return []
  try {
    const parsed = JSON.parse(window.localStorage.getItem(CART_KEY) ?? '[]')
    if (!Array.isArray(parsed)) return []
    return parsed
      .map((item) => ({
        product_id: Number(item.product_id),
        quantity: Number(item.quantity),
      }))
      .filter((item) => Number.isInteger(item.product_id) && item.quantity > 0)
  } catch {
    return []
  }
}

export function writeCart(items: CartItem[]) {
  if (typeof window === 'undefined') return
  window.localStorage.setItem(CART_KEY, JSON.stringify(items.filter((item) => item.quantity > 0)))
  window.dispatchEvent(new Event('rwfw-shop-cart-changed'))
}

export function addToCart(productId: number, quantity = 1) {
  const cart = readCart()
  const existing = cart.find((item) => item.product_id === productId)
  if (existing) {
    existing.quantity += quantity
  } else {
    cart.push({ product_id: productId, quantity })
  }
  writeCart(cart)
}

export function clearCart() {
  writeCart([])
}
"#
    .to_string()
}

fn shop_product_card_tsx() -> String {
    r#"import { Link } from '@inertiajs/react'
import { addToCart } from './cart'

export interface Product {
  id: number
  name: string
  slug: string
  description: string
  price_cents: number
  currency: string
  image_url: string
  inventory: number
  featured: boolean
}

export function formatMoney(cents: number, currency = 'USD') {
  return new Intl.NumberFormat('en-US', {
    style: 'currency',
    currency,
  }).format(cents / 100)
}

export default function ProductCard({ product }: { product: Product }) {
  return (
    <article className="group overflow-hidden rounded-[1.8rem] border border-neutral-200 bg-white shadow-sm transition hover:-translate-y-1 hover:shadow-2xl">
      <Link href={`/shop/products/${product.slug}`}>
        <img
          src={product.image_url}
          alt=""
          className="h-72 w-full object-cover transition duration-500 group-hover:scale-105"
        />
      </Link>
      <div className="p-5">
        <div className="flex items-start justify-between gap-4">
          <div>
            <Link href={`/shop/products/${product.slug}`}>
              <h3 className="text-lg font-black tracking-tight text-neutral-950 hover:text-orange-700">
                {product.name}
              </h3>
            </Link>
            <p className="mt-2 line-clamp-2 text-sm leading-6 text-neutral-600">{product.description}</p>
          </div>
          <p className="shrink-0 text-sm font-black text-neutral-950">
            {formatMoney(product.price_cents, product.currency)}
          </p>
        </div>
        <button
          type="button"
          onClick={() => addToCart(product.id)}
          className="mt-5 w-full rounded-full bg-neutral-950 px-4 py-3 text-sm font-black text-white hover:bg-orange-700"
        >
          Add to cart
        </button>
      </div>
    </article>
  )
}
"#
    .to_string()
}

fn shop_index_page_tsx() -> String {
    r#"import { Link, router } from '@inertiajs/react'
import type { FormEvent } from 'react'
import { useState } from 'react'
import ProductCard, { type Product } from '../components/ProductCard'

interface Category {
  id: number
  name: string
  slug: string
  description: string
}

interface Props {
  categories: Category[]
  featured: Product[]
  products: Product[]
  filters: {
    q: string
    category: string
  }
}

export default function ShopIndex({ categories, featured, products, filters }: Props) {
  const [q, setQ] = useState(filters.q ?? '')
  const hero = featured[0] ?? products[0]

  function search(event: FormEvent) {
    event.preventDefault()
    router.get('/shop', { q, category: filters.category }, { preserveState: true })
  }

  return (
    <main className="min-h-screen bg-[#f6f1e7] text-neutral-950">
      <header className="mx-auto flex max-w-7xl items-center justify-between px-6 py-6">
        <Link href="/shop" className="text-xl font-black tracking-tight">
          RWFW Commerce
        </Link>
        <nav className="flex items-center gap-4 text-sm font-black">
          <Link href="/shop">Catalog</Link>
          <Link href="/shop/cart" className="rounded-full bg-neutral-950 px-4 py-2 text-white">
            Cart
          </Link>
        </nav>
      </header>

      <section className="mx-auto grid max-w-7xl gap-6 px-6 pb-10 lg:grid-cols-[0.9fr_1.1fr]">
        <div className="rounded-[2.5rem] bg-neutral-950 p-8 text-white md:p-12">
          <p className="text-xs font-black uppercase tracking-[0.35em] text-orange-300">
            Storefront example
          </p>
          <h1 className="mt-6 text-6xl font-black leading-[0.9] tracking-tight md:text-8xl">
            Commerce sem Next, em Rust.
          </h1>
          <p className="mt-6 max-w-xl text-lg leading-8 text-neutral-300">
            Catálogo, filtros, carrinho local e checkout falso persistido no PostgreSQL.
          </p>
          <form onSubmit={search} className="mt-8 flex rounded-full bg-white p-2 text-neutral-950">
            <input
              value={q}
              onChange={(event) => setQ(event.target.value)}
              placeholder="Search products"
              className="min-w-0 flex-1 rounded-full px-4 outline-none"
            />
            <button className="rounded-full bg-orange-400 px-5 py-3 text-sm font-black hover:bg-orange-300">
              Search
            </button>
          </form>
        </div>

        {hero && (
          <Link
            href={`/shop/products/${hero.slug}`}
            className="group relative min-h-[540px] overflow-hidden rounded-[2.5rem] bg-neutral-900"
          >
            <img
              src={hero.image_url}
              alt=""
              className="absolute inset-0 h-full w-full object-cover opacity-85 transition duration-700 group-hover:scale-105"
            />
            <div className="absolute inset-0 bg-gradient-to-t from-black/80 via-black/20 to-transparent" />
            <div className="absolute bottom-0 p-8 text-white md:p-10">
              <p className="text-xs font-black uppercase tracking-[0.3em] text-orange-200">Featured</p>
              <h2 className="mt-3 max-w-lg text-4xl font-black tracking-tight">{hero.name}</h2>
              <p className="mt-3 max-w-md text-sm leading-6 text-neutral-200">{hero.description}</p>
            </div>
          </Link>
        )}
      </section>

      <section className="mx-auto max-w-7xl px-6 pb-16">
        <div className="mb-8 flex flex-wrap gap-3">
          <Link
            href="/shop"
            className={`rounded-full px-4 py-2 text-sm font-black ${
              !filters.category ? 'bg-neutral-950 text-white' : 'bg-white text-neutral-700'
            }`}
          >
            All
          </Link>
          {categories.map((category) => (
            <Link
              key={category.id}
              href={`/shop?category=${category.slug}`}
              className={`rounded-full px-4 py-2 text-sm font-black ${
                filters.category === category.slug ? 'bg-neutral-950 text-white' : 'bg-white text-neutral-700'
              }`}
            >
              {category.name}
            </Link>
          ))}
        </div>

        <div className="grid gap-6 md:grid-cols-2 xl:grid-cols-3">
          {products.map((product) => (
            <ProductCard key={product.id} product={product} />
          ))}
        </div>
      </section>
    </main>
  )
}
"#
    .to_string()
}

fn shop_product_show_page_tsx() -> String {
    r#"import { Link } from '@inertiajs/react'
import { addToCart } from '../../components/cart'
import { formatMoney, type Product } from '../../components/ProductCard'

interface Props {
  product: Product
}

export default function ProductShow({ product }: Props) {
  return (
    <main className="min-h-screen bg-[#f6f1e7] text-neutral-950">
      <header className="mx-auto flex max-w-7xl items-center justify-between px-6 py-6">
        <Link href="/shop" className="text-xl font-black tracking-tight">
          RWFW Commerce
        </Link>
        <Link href="/shop/cart" className="rounded-full bg-neutral-950 px-4 py-2 text-sm font-black text-white">
          Cart
        </Link>
      </header>

      <section className="mx-auto grid max-w-7xl gap-8 px-6 pb-16 lg:grid-cols-[1.1fr_0.9fr]">
        <img
          src={product.image_url}
          alt=""
          className="h-[680px] w-full rounded-[2.5rem] object-cover shadow-2xl"
        />
        <div className="flex flex-col justify-center rounded-[2.5rem] bg-white p-8 shadow-sm md:p-12">
          <p className="text-xs font-black uppercase tracking-[0.35em] text-orange-700">
            Product detail
          </p>
          <h1 className="mt-5 text-5xl font-black leading-tight tracking-tight">{product.name}</h1>
          <p className="mt-5 text-2xl font-black">
            {formatMoney(product.price_cents, product.currency)}
          </p>
          <p className="mt-6 text-lg leading-8 text-neutral-600">{product.description}</p>
          <p className="mt-4 text-sm font-black text-neutral-500">{product.inventory} in stock</p>
          <button
            type="button"
            onClick={() => addToCart(product.id)}
            className="mt-8 rounded-full bg-neutral-950 px-6 py-4 text-sm font-black text-white hover:bg-orange-700"
          >
            Add to cart
          </button>
        </div>
      </section>
    </main>
  )
}
"#
    .to_string()
}

fn shop_cart_page_tsx() -> String {
    r#"import { Link } from '@inertiajs/react'
import { useEffect, useMemo, useState } from 'react'
import { readCart, writeCart, type CartItem } from '../../components/cart'
import { formatMoney, type Product } from '../../components/ProductCard'

interface Props {
  products: Product[]
}

export default function CartIndex({ products }: Props) {
  const [cart, setCart] = useState<CartItem[]>([])

  useEffect(() => {
    setCart(readCart())
  }, [])

  const rows = useMemo(
    () =>
      cart
        .map((item) => ({
          item,
          product: products.find((product) => product.id === item.product_id),
        }))
        .filter((row): row is { item: CartItem; product: Product } => Boolean(row.product)),
    [cart, products],
  )

  const total = rows.reduce((sum, row) => sum + row.product.price_cents * row.item.quantity, 0)

  function update(productId: number, quantity: number) {
    const next = cart
      .map((item) => (item.product_id === productId ? { ...item, quantity } : item))
      .filter((item) => item.quantity > 0)
    setCart(next)
    writeCart(next)
  }

  return (
    <main className="min-h-screen bg-[#f6f1e7] px-6 py-8 text-neutral-950">
      <div className="mx-auto max-w-5xl">
        <div className="mb-8 flex items-center justify-between">
          <Link href="/shop" className="text-xl font-black">RWFW Commerce</Link>
          <Link href="/shop" className="text-sm font-black text-orange-700">Continue shopping</Link>
        </div>

        <section className="rounded-[2rem] bg-white p-6 shadow-sm md:p-8">
          <h1 className="text-4xl font-black tracking-tight">Cart</h1>

          {rows.length === 0 ? (
            <div className="mt-8 rounded-2xl border border-dashed border-neutral-300 p-10 text-neutral-500">
              Your cart is empty.
            </div>
          ) : (
            <div className="mt-8 space-y-4">
              {rows.map(({ item, product }) => (
                <div key={product.id} className="grid gap-4 rounded-2xl border border-neutral-200 p-4 md:grid-cols-[120px_1fr_auto] md:items-center">
                  <img src={product.image_url} alt="" className="h-28 w-28 rounded-2xl object-cover" />
                  <div>
                    <h2 className="font-black">{product.name}</h2>
                    <p className="mt-1 text-sm text-neutral-500">{formatMoney(product.price_cents, product.currency)}</p>
                  </div>
                  <div className="flex items-center gap-3">
                    <input
                      type="number"
                      min={0}
                      value={item.quantity}
                      onChange={(event) => update(product.id, Number(event.target.value))}
                      className="w-20 rounded-xl border border-neutral-300 px-3 py-2"
                    />
                    <p className="w-24 text-right font-black">
                      {formatMoney(product.price_cents * item.quantity, product.currency)}
                    </p>
                  </div>
                </div>
              ))}
            </div>
          )}

          <div className="mt-8 flex flex-col justify-between gap-4 border-t border-neutral-200 pt-6 md:flex-row md:items-center">
            <p className="text-2xl font-black">Total {formatMoney(total)}</p>
            <Link
              href="/shop/checkout"
              className={`rounded-full px-6 py-3 text-center text-sm font-black ${
                rows.length === 0 ? 'pointer-events-none bg-neutral-200 text-neutral-500' : 'bg-neutral-950 text-white hover:bg-orange-700'
              }`}
            >
              Checkout
            </Link>
          </div>
        </section>
      </div>
    </main>
  )
}
"#
    .to_string()
}

fn shop_checkout_page_tsx() -> String {
    r#"import { Link, router, usePage } from '@inertiajs/react'
import type { FormEvent } from 'react'
import { useEffect, useMemo, useState } from 'react'
import { clearCart, readCart, type CartItem } from '../../components/cart'
import { formatMoney, type Product } from '../../components/ProductCard'

interface Props {
  products: Product[]
}

export default function CheckoutIndex({ products }: Props) {
  const [cart, setCart] = useState<CartItem[]>([])
  const [processing, setProcessing] = useState(false)
  const [form, setForm] = useState({
    customer_name: '',
    customer_email: '',
    address_line: '',
    city: '',
    country: 'US',
  })
  const { props } = usePage()
  const errors = (props as any).errors ?? {}

  useEffect(() => {
    setCart(readCart())
  }, [])

  const rows = useMemo(
    () =>
      cart
        .map((item) => ({
          item,
          product: products.find((product) => product.id === item.product_id),
        }))
        .filter((row): row is { item: CartItem; product: Product } => Boolean(row.product)),
    [cart, products],
  )
  const total = rows.reduce((sum, row) => sum + row.product.price_cents * row.item.quantity, 0)

  function submit(event: FormEvent) {
    event.preventDefault()
    setProcessing(true)
    router.post(
      '/shop/checkout',
      {
        ...form,
        items: cart,
      },
      {
        onSuccess: () => clearCart(),
        onFinish: () => setProcessing(false),
      },
    )
  }

  return (
    <main className="min-h-screen bg-[#f6f1e7] px-6 py-8 text-neutral-950">
      <div className="mx-auto max-w-6xl">
        <div className="mb-8 flex items-center justify-between">
          <Link href="/shop" className="text-xl font-black">RWFW Commerce</Link>
          <Link href="/shop/cart" className="text-sm font-black text-orange-700">Back to cart</Link>
        </div>

        <div className="grid gap-6 lg:grid-cols-[1fr_420px]">
          <form onSubmit={submit} className="rounded-[2rem] bg-white p-8 shadow-sm">
            <p className="text-xs font-black uppercase tracking-[0.3em] text-orange-700">Fake checkout</p>
            <h1 className="mt-3 text-4xl font-black tracking-tight">Delivery details</h1>

            <div className="mt-8 grid gap-5 md:grid-cols-2">
              {[
                ['customer_name', 'Name'],
                ['customer_email', 'Email'],
                ['address_line', 'Address'],
                ['city', 'City'],
                ['country', 'Country'],
              ].map(([key, label]) => (
                <label key={key} className={key === 'address_line' ? 'md:col-span-2' : ''}>
                  <span className="mb-2 block text-sm font-black text-neutral-700">{label}</span>
                  <input
                    value={(form as any)[key]}
                    onChange={(event) => setForm({ ...form, [key]: event.target.value })}
                    className="w-full rounded-2xl border border-neutral-300 px-4 py-3 outline-none focus:border-neutral-950"
                  />
                  {errors[key] && <p className="mt-2 text-sm text-red-600">{errors[key]}</p>}
                </label>
              ))}
            </div>

            {errors.items && <p className="mt-5 text-sm text-red-600">{errors.items}</p>}

            <button
              type="submit"
              disabled={processing || rows.length === 0}
              className="mt-8 rounded-full bg-neutral-950 px-6 py-4 text-sm font-black text-white hover:bg-orange-700 disabled:opacity-50"
            >
              {processing ? 'Creating order...' : 'Complete fake checkout'}
            </button>
          </form>

          <aside className="rounded-[2rem] bg-neutral-950 p-6 text-white shadow-sm">
            <h2 className="text-2xl font-black">Order summary</h2>
            <div className="mt-6 space-y-4">
              {rows.map(({ item, product }) => (
                <div key={product.id} className="flex gap-4">
                  <img src={product.image_url} alt="" className="h-20 w-20 rounded-2xl object-cover" />
                  <div className="flex-1">
                    <p className="font-black">{product.name}</p>
                    <p className="text-sm text-neutral-400">Qty {item.quantity}</p>
                  </div>
                  <p className="font-black">{formatMoney(product.price_cents * item.quantity, product.currency)}</p>
                </div>
              ))}
            </div>
            <div className="mt-6 border-t border-white/10 pt-6 text-2xl font-black">
              Total {formatMoney(total)}
            </div>
          </aside>
        </div>
      </div>
    </main>
  )
}
"#
    .to_string()
}

fn shop_order_show_page_tsx() -> String {
    r#"import { Link } from '@inertiajs/react'
import { formatMoney } from '../../components/ProductCard'

interface Order {
  number: string
  customer_name: string
  customer_email: string
  total_cents: number
  currency: string
  status: string
}

interface OrderItem {
  id: number
  product_name: string
  product_slug: string
  unit_price_cents: number
  quantity: number
  subtotal_cents: number
}

interface Props {
  order: Order
  items: OrderItem[]
}

export default function OrderShow({ order, items }: Props) {
  return (
    <main className="min-h-screen bg-neutral-950 px-6 py-10 text-white">
      <section className="mx-auto max-w-4xl rounded-[2.5rem] bg-white p-8 text-neutral-950 shadow-2xl md:p-12">
        <p className="text-xs font-black uppercase tracking-[0.35em] text-orange-700">
          Order confirmed
        </p>
        <h1 className="mt-4 text-5xl font-black tracking-tight">Pedido {order.number}</h1>
        <p className="mt-4 text-neutral-600">
          Checkout falso persistido no banco para {order.customer_name} ({order.customer_email}).
        </p>

        <div className="mt-8 space-y-4">
          {items.map((item) => (
            <div key={item.id} className="flex items-center justify-between rounded-2xl bg-neutral-100 p-4">
              <div>
                <Link href={`/shop/products/${item.product_slug}`} className="font-black hover:text-orange-700">
                  {item.product_name}
                </Link>
                <p className="text-sm text-neutral-500">Qty {item.quantity}</p>
              </div>
              <p className="font-black">{formatMoney(item.subtotal_cents, order.currency)}</p>
            </div>
          ))}
        </div>

        <div className="mt-8 flex items-center justify-between border-t border-neutral-200 pt-6">
          <span className="text-sm font-black uppercase tracking-[0.25em] text-neutral-500">{order.status}</span>
          <span className="text-3xl font-black">{formatMoney(order.total_cents, order.currency)}</span>
        </div>

        <Link
          href="/shop"
          className="mt-8 inline-flex rounded-full bg-neutral-950 px-6 py-3 text-sm font-black text-white hover:bg-orange-700"
        >
          Back to storefront
        </Link>
      </section>
    </main>
  )
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

#[derive(Debug, serde::Deserialize)]
pub struct Create{{ name_pascal }}Input {
{% for field in fields %}    pub {{ field.name }}: {{ field.input_rust_type }},
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

#[derive(Debug, serde::Deserialize)]
pub struct Update{{ name_pascal }}Input {
{% for field in fields %}    pub {{ field.name }}: {{ field.input_rust_type }},
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
    r#"use crate::repositories::{{ name_snake }}_repo::{{ name_pascal }}Repository;
use crate::use_cases::create_{{ name_snake }}::{Create{{ name_pascal }}Input, Create{{ name_pascal }}UseCase};
use axum::extract::{Json, Query, State};
use axum::response::{IntoResponse, Response};
use axum::routing;
use rwfw_core::app::AppState;
use rwfw_core::auth::CurrentUser;
use rwfw_core::error::AppError;
use rwfw_core::inertia::Inertia;
use serde::Deserialize;

#[derive(Debug, Deserialize)]
struct ListParams {
    page: Option<u64>,
    q: Option<String>,
}

pub fn route() -> axum::routing::MethodRouter<AppState> {
    routing::get(get).post(post)
}

async fn get(
    State(state): State<AppState>,
    user: CurrentUser,
    i: Inertia,
    Query(params): Query<ListParams>,
) -> Response {
    if !user.can("{{ module }}.{{ table }}.view") {
        return AppError::Forbidden("Missing permission: {{ module }}.{{ table }}.view".into()).into_response();
    }

    let per_page = 20_u64;
    let page = params.page.unwrap_or(1).max(1);
    let search = params.q.as_deref().map(str::trim).filter(|value| !value.is_empty());
    let repo = {{ name_pascal }}Repository::new(state.db.clone());
    let (items, total) = match repo.find_all(page, per_page, search).await {
        Ok(result) => result,
        Err(error) => return AppError::Internal(error).into_response(),
    };
    let total_pages = if total == 0 { 1 } else { total.div_ceil(per_page) };

    i.render_with_ssr(
        "{{ module }}/{{ table }}/Index",
        serde_json::json!({
            "items": items,
            "filters": {
                "q": search.unwrap_or("")
            },
            "pagination": {
                "page": page.min(total_pages),
                "per_page": per_page,
                "total": total,
                "total_pages": total_pages
            }
        }),
    )
    .await
}

async fn post(
    State(state): State<AppState>,
    user: CurrentUser,
    i: Inertia,
    Json(input): Json<Create{{ name_pascal }}Input>,
) -> Response {
    if !user.can("{{ module }}.{{ table }}.create") {
        return AppError::Forbidden("Missing permission: {{ module }}.{{ table }}.create".into()).into_response();
    }

    let repo = {{ name_pascal }}Repository::new(state.db.clone());
    let use_case = Create{{ name_pascal }}UseCase;

    match use_case.execute(&repo, input).await {
        Ok(_) => Inertia::redirect_with_success("/{{ module }}/{{ table }}", "{{ name_pascal }} created."),
        Err(AppError::Validation(errors)) => i.redirect_back_with_errors(errors),
        Err(error) => error.into_response(),
    }
}
"#
    .to_string()
}

fn scaffold_route_create_template() -> String {
    r#"use axum::response::IntoResponse;
use axum::routing;
use rwfw_core::app::AppState;
use rwfw_core::auth::CurrentUser;
use rwfw_core::error::AppError;
use rwfw_core::inertia::Inertia;

pub fn route() -> axum::routing::MethodRouter<AppState> {
    routing::get(get)
}

async fn get(user: CurrentUser, i: Inertia) -> impl IntoResponse {
    if !user.can("{{ module }}.{{ table }}.create") {
        return AppError::Forbidden("Missing permission: {{ module }}.{{ table }}.create".into()).into_response();
    }

    i.render_with_ssr("{{ module }}/{{ table }}/Create", serde_json::json!({}))
        .await
}
"#
    .to_string()
}

fn scaffold_route_edit_template() -> String {
    r#"use crate::repositories::{{ name_snake }}_repo::{{ name_pascal }}Repository;
use crate::use_cases::update_{{ name_snake }}::{Update{{ name_pascal }}Input, Update{{ name_pascal }}UseCase};
use axum::extract::{Json, Path, State};
use axum::response::{IntoResponse, Response};
use axum::routing;
use rwfw_core::app::AppState;
use rwfw_core::auth::CurrentUser;
use rwfw_core::error::AppError;
use rwfw_core::inertia::Inertia;

pub fn route() -> axum::routing::MethodRouter<AppState> {
    routing::get(get).put(put)
}

async fn get(
    State(state): State<AppState>,
    user: CurrentUser,
    i: Inertia,
    Path(id): Path<i32>,
) -> Response {
    if !user.can("{{ module }}.{{ table }}.update") {
        return AppError::Forbidden("Missing permission: {{ module }}.{{ table }}.update".into()).into_response();
    }

    let repo = {{ name_pascal }}Repository::new(state.db.clone());
    let item = match repo.find_by_id(id).await {
        Ok(Some(item)) => item,
        Ok(None) => return AppError::NotFound(format!("{{ name_pascal }} not found: {id}")).into_response(),
        Err(error) => return AppError::Internal(error).into_response(),
    };

    i.render_with_ssr(
        "{{ module }}/{{ table }}/Edit",
        serde_json::json!({
            "item": item
        }),
    )
    .await
}

async fn put(
    State(state): State<AppState>,
    user: CurrentUser,
    i: Inertia,
    Path(id): Path<i32>,
    Json(input): Json<Update{{ name_pascal }}Input>,
) -> Response {
    if !user.can("{{ module }}.{{ table }}.update") {
        return AppError::Forbidden("Missing permission: {{ module }}.{{ table }}.update".into()).into_response();
    }

    let repo = {{ name_pascal }}Repository::new(state.db.clone());
    let use_case = Update{{ name_pascal }}UseCase;

    match use_case.execute(&repo, id, input).await {
        Ok(_) => Inertia::redirect_with_success("/{{ module }}/{{ table }}", "{{ name_pascal }} updated."),
        Err(AppError::Validation(errors)) => i.redirect_back_with_errors(errors),
        Err(error) => error.into_response(),
    }
}
"#
    .to_string()
}

fn scaffold_route_item_template() -> String {
    r#"use crate::repositories::{{ name_snake }}_repo::{{ name_pascal }}Repository;
use crate::use_cases::delete_{{ name_snake }}::Delete{{ name_pascal }}UseCase;
use axum::extract::{Path, State};
use axum::response::{IntoResponse, Response};
use axum::routing;
use rwfw_core::app::AppState;
use rwfw_core::auth::CurrentUser;
use rwfw_core::error::AppError;
use rwfw_core::inertia::Inertia;

pub fn route() -> axum::routing::MethodRouter<AppState> {
    routing::get(get).delete(delete)
}

async fn get(
    State(state): State<AppState>,
    user: CurrentUser,
    i: Inertia,
    Path(id): Path<i32>,
) -> Response {
    if !user.can("{{ module }}.{{ table }}.view") {
        return AppError::Forbidden("Missing permission: {{ module }}.{{ table }}.view".into()).into_response();
    }

    let repo = {{ name_pascal }}Repository::new(state.db.clone());
    let item = match repo.find_by_id(id).await {
        Ok(Some(item)) => item,
        Ok(None) => return AppError::NotFound(format!("{{ name_pascal }} not found: {id}")).into_response(),
        Err(error) => return AppError::Internal(error).into_response(),
    };

    i.render_with_ssr(
        "{{ module }}/{{ table }}/Show",
        serde_json::json!({
            "item": item
        }),
    )
    .await
}

async fn delete(State(state): State<AppState>, user: CurrentUser, Path(id): Path<i32>) -> Response {
    if !user.can("{{ module }}.{{ table }}.delete") {
        return AppError::Forbidden("Missing permission: {{ module }}.{{ table }}.delete".into()).into_response();
    }

    let repo = {{ name_pascal }}Repository::new(state.db.clone());
    let use_case = Delete{{ name_pascal }}UseCase;

    match use_case.execute(&repo, id).await {
        Ok(()) => Inertia::redirect_with_success("/{{ module }}/{{ table }}", "{{ name_pascal }} deleted."),
        Err(error) => error.into_response(),
    }
}
"#
    .to_string()
}

fn scaffold_page_index_template() -> String {
    r#"import type { FormEvent } from 'react'
import { Link, router } from '@inertiajs/react'
import AppLayout from '@app/layouts/AppLayout'

interface Pagination {
  page: number
  per_page: number
  total: number
  total_pages: number
}

interface Props {
  items: Array<Record<string, unknown>>
  pagination: Pagination
  filters?: {
    q?: string
  }
}

function pageHref(page: number, q?: string) {
  const params = new URLSearchParams()
  if (q) params.set('q', q)
  if (page > 1) params.set('page', String(page))
  const query = params.toString()
  return query ? `/{{ module }}/{{ table }}?${query}` : '/{{ module }}/{{ table }}'
}

export default function {{ name_pascal }}Index({ items, pagination, filters = {} }: Props) {
  const q = filters.q ?? ''
  const canGoBack = pagination.page > 1
  const canGoForward = pagination.page < pagination.total_pages

  function submitSearch(event: FormEvent<HTMLFormElement>) {
    event.preventDefault()
    const form = new FormData(event.currentTarget)
    const q = String(form.get('q') ?? '').trim()
    const data = q ? { q } : {}
    router.get('/{{ module }}/{{ table }}', data, { preserveState: true, replace: true })
  }

  return (
    <AppLayout>
      <div className="max-w-5xl">
        <div className="mb-8 flex flex-col gap-4 sm:flex-row sm:items-center sm:justify-between">
          <div>
            <p className="text-sm font-semibold uppercase tracking-[0.2em] text-slate-500">{{ table }}</p>
            <h1 className="mt-2 text-3xl font-bold text-gray-900">{{ name_pascal }}</h1>
          </div>
          <Link
            href="/{{ module }}/{{ table }}/create"
            className="rounded-xl bg-slate-950 px-4 py-2 text-sm font-semibold text-white hover:bg-slate-800"
          >
            New {{ name_pascal }}
          </Link>
        </div>

        <form onSubmit={submitSearch} className="mb-4 flex flex-col gap-3 rounded-2xl border border-slate-200 bg-white p-4 shadow-sm sm:flex-row">
          <input
            name="q"
            type="search"
            defaultValue={q}
            placeholder="Search {{ table }}..."
            className="min-w-0 flex-1 rounded-xl border border-slate-300 px-3 py-2 text-sm outline-none focus:border-slate-950"
          />
          <div className="flex gap-2">
            <button
              type="submit"
              className="rounded-xl bg-slate-950 px-4 py-2 text-sm font-semibold text-white hover:bg-slate-800"
            >
              Search
            </button>
            {q ? (
              <Link
                href="/{{ module }}/{{ table }}"
                className="rounded-xl border border-slate-300 px-4 py-2 text-sm font-semibold text-slate-700 hover:bg-slate-50"
              >
                Clear
              </Link>
            ) : null}
          </div>
        </form>

        <div className="overflow-hidden rounded-lg border bg-white">
          <table className="w-full text-sm">
            <thead className="bg-gray-50">
              <tr>
{% for field in fields %}                <th className="px-4 py-2 text-left">{{ field.title }}</th>
{% endfor %}                <th className="px-4 py-2 text-right">Actions</th>
              </tr>
            </thead>
            <tbody>
              {items.length === 0 && (
                <tr>
                  <td className="px-4 py-6 text-gray-500" colSpan={ {{ fields_colspan }} }>
                    No records yet.
                  </td>
                </tr>
              )}
              {items.map((item, index) => (
                <tr key={index} className="border-t">
{% for field in fields %}                  <td className="px-4 py-2">{String(item['{{ field.name }}'] ?? '')}</td>
{% endfor %}                  <td className="px-4 py-2 text-right">
                    {item['id'] ? (
                      <div className="flex justify-end gap-3">
                      <Link
                        href={`/{{ module }}/{{ table }}/${String(item['id'])}`}
                        className="text-sm font-semibold text-slate-700 hover:text-slate-950"
                      >
                        View
                      </Link>
                      <Link
                        href={`/{{ module }}/{{ table }}/${String(item['id'])}/edit`}
                        className="text-sm font-semibold text-slate-700 hover:text-slate-950"
                      >
                        Edit
                      </Link>
                      <button
                        type="button"
                        onClick={() => {
                          if (window.confirm('Delete this {{ name_pascal }}?')) {
                            router.delete('/{{ module }}/{{ table }}/' + String(item['id']))
                          }
                        }}
                        className="text-sm font-semibold text-red-600 hover:text-red-700"
                      >
                        Delete
                      </button>
                      </div>
                    ) : null}
                  </td>
                </tr>
              ))}
            </tbody>
          </table>
        </div>

        <div className="mt-4 flex flex-col gap-3 text-sm text-slate-600 sm:flex-row sm:items-center sm:justify-between">
          <p>
            Showing {items.length} of {pagination.total} records
          </p>
          <div className="flex items-center gap-2">
            {canGoBack ? (
              <Link
                href={pageHref(pagination.page - 1, q)}
                className="rounded-xl border border-slate-300 px-3 py-2 font-semibold text-slate-700 hover:bg-slate-50"
              >
                Previous
              </Link>
            ) : (
              <span className="rounded-xl border border-slate-200 px-3 py-2 font-semibold text-slate-300">
                Previous
              </span>
            )}
            <span>
              Page {pagination.page} of {pagination.total_pages}
            </span>
            {canGoForward ? (
              <Link
                href={pageHref(pagination.page + 1, q)}
                className="rounded-xl border border-slate-300 px-3 py-2 font-semibold text-slate-700 hover:bg-slate-50"
              >
                Next
              </Link>
            ) : (
              <span className="rounded-xl border border-slate-200 px-3 py-2 font-semibold text-slate-300">
                Next
              </span>
            )}
          </div>
        </div>
      </div>
    </AppLayout>
  )
}
"#
    .to_string()
}

fn scaffold_form_template() -> String {
    r#"import type { FormEvent } from 'react'
import { Link } from '@inertiajs/react'

type FormData = Record<string, string | boolean>

interface Props {
  data: FormData
  setData: (field: string, value: string | boolean) => void
  processing: boolean
  errors: Record<string, string | undefined>
  submitLabel: string
  cancelHref: string
  onSubmit: (event: FormEvent<HTMLFormElement>) => void
}

export default function {{ name_pascal }}Form({
  data,
  setData,
  processing,
  errors,
  submitLabel,
  cancelHref,
  onSubmit,
}: Props) {
  return (
    <form onSubmit={onSubmit} className="space-y-5 rounded-2xl border border-slate-200 bg-white p-6 shadow-sm">
{% for field in fields %}{% if field.is_bool %}      <label className="flex items-center gap-3 rounded-xl border border-slate-200 px-4 py-3">
        <input
          id="{{ field.name }}"
          type="checkbox"
          checked={Boolean(data['{{ field.name }}'])}
          onChange={(event) => setData('{{ field.name }}', event.target.checked)}
          className="h-4 w-4 rounded border-slate-300 text-slate-950 focus:ring-slate-950"
        />
        <span className="text-sm font-medium text-slate-800">
          {{ field.title }}{% if field.optional %} <span className="text-xs font-normal text-slate-500">(optional)</span>{% endif %}
        </span>
      </label>
      {errors['{{ field.name }}'] && <p className="text-sm text-red-600">{errors['{{ field.name }}']}</p>}
{% else %}      <div>
        <label htmlFor="{{ field.name }}" className="mb-1 block text-sm font-medium text-slate-700">
          {{ field.title }}{% if field.optional %} <span className="text-xs font-normal text-slate-500">(optional)</span>{% endif %}
        </label>
{% if field.is_text %}        <textarea
          id="{{ field.name }}"
          value={String(data['{{ field.name }}'] ?? '')}
          onChange={(event) => setData('{{ field.name }}', event.target.value)}
          rows={6}
          className="w-full rounded-xl border border-slate-300 px-3 py-2 text-sm outline-none focus:border-slate-950"
        />
{% else %}        <input
          id="{{ field.name }}"
          type="{{ field.input_type }}"
{% if field.has_input_step %}          step="{{ field.input_step }}"
{% endif %}          value={String(data['{{ field.name }}'] ?? '')}
          onChange={(event) => setData('{{ field.name }}', event.target.value)}
          className="w-full rounded-xl border border-slate-300 px-3 py-2 text-sm outline-none focus:border-slate-950"
        />
{% endif %}        {errors['{{ field.name }}'] && <p className="mt-1 text-sm text-red-600">{errors['{{ field.name }}']}</p>}
      </div>
{% endif %}{% endfor %}      <div className="flex items-center gap-3 pt-2">
        <button
          type="submit"
          disabled={processing}
          className="rounded-xl bg-slate-950 px-4 py-2 text-sm font-semibold text-white hover:bg-slate-800 disabled:opacity-60"
        >
          {processing ? 'Saving...' : submitLabel}
        </button>
        <Link
          href={cancelHref}
          className="rounded-xl border border-slate-300 px-4 py-2 text-sm font-semibold text-slate-700 hover:bg-slate-50"
        >
          Cancel
        </Link>
      </div>
    </form>
  )
}
"#
    .to_string()
}

fn scaffold_page_create_template() -> String {
    r#"import type { FormEvent } from 'react'
import { useForm } from '@inertiajs/react'
import AppLayout from '@app/layouts/AppLayout'
import {{ name_pascal }}Form from './Form'

type FormData = Record<string, string | boolean>

export default function {{ name_pascal }}Create() {
  const { data, setData, post, processing, errors } = useForm({
{% for field in fields %}    '{{ field.name }}': {% if field.is_bool %}false{% else %}''{% endif %},
{% endfor %}  })
  const formData = data as FormData
  const setFormData = setData as (field: string, value: string | boolean) => void
  const formErrors = errors as Record<string, string | undefined>

  function submit(event: FormEvent<HTMLFormElement>) {
    event.preventDefault()
    post('/{{ module }}/{{ table }}')
  }

  return (
    <AppLayout>
      <div className="max-w-3xl">
        <div className="mb-8">
          <p className="text-sm font-semibold uppercase tracking-[0.2em] text-slate-500">{{ table }}</p>
          <h1 className="mt-2 text-3xl font-bold text-slate-950">New {{ name_pascal }}</h1>
        </div>
        <{{ name_pascal }}Form
          data={formData}
          setData={setFormData}
          processing={processing}
          errors={formErrors}
          submitLabel="Create {{ name_pascal }}"
          cancelHref="/{{ module }}/{{ table }}"
          onSubmit={submit}
        />
      </div>
    </AppLayout>
  )
}
"#
    .to_string()
}

fn scaffold_page_edit_template() -> String {
    r#"import type { FormEvent } from 'react'
import { useForm } from '@inertiajs/react'
import AppLayout from '@app/layouts/AppLayout'
import {{ name_pascal }}Form from './Form'

type FormData = Record<string, string | boolean>

interface Props {
  item: Record<string, unknown> & { id: number }
}
{% if has_datetime %}
function datetimeLocalValue(value: unknown) {
  return value === null || value === undefined ? '' : String(value).slice(0, 16)
}
{% endif %}

export default function {{ name_pascal }}Edit({ item }: Props) {
  const { data, setData, put, processing, errors } = useForm({
{% for field in fields %}    '{{ field.name }}': {% if field.is_bool %}Boolean(item['{{ field.name }}']){% elif field.is_datetime %}datetimeLocalValue(item['{{ field.name }}']){% else %}String(item['{{ field.name }}'] ?? ''){% endif %},
{% endfor %}  })
  const formData = data as FormData
  const setFormData = setData as (field: string, value: string | boolean) => void
  const formErrors = errors as Record<string, string | undefined>

  function submit(event: FormEvent<HTMLFormElement>) {
    event.preventDefault()
    put(`/{{ module }}/{{ table }}/${item.id}/edit`)
  }

  return (
    <AppLayout>
      <div className="max-w-3xl">
        <div className="mb-8">
          <p className="text-sm font-semibold uppercase tracking-[0.2em] text-slate-500">{{ table }}</p>
          <h1 className="mt-2 text-3xl font-bold text-slate-950">Edit {{ name_pascal }}</h1>
        </div>
        <{{ name_pascal }}Form
          data={formData}
          setData={setFormData}
          processing={processing}
          errors={formErrors}
          submitLabel="Save {{ name_pascal }}"
          cancelHref="/{{ module }}/{{ table }}"
          onSubmit={submit}
        />
      </div>
    </AppLayout>
  )
}
"#
    .to_string()
}

fn scaffold_page_show_template() -> String {
    r#"import { Link, router } from '@inertiajs/react'
import AppLayout from '@app/layouts/AppLayout'

interface Props {
  item: Record<string, unknown> & { id: number }
}

export default function {{ name_pascal }}Show({ item }: Props) {
  return (
    <AppLayout>
      <div className="max-w-3xl">
        <div className="mb-8 flex items-start justify-between gap-4">
          <div>
            <p className="text-sm font-semibold uppercase tracking-[0.2em] text-slate-500">{{ table }}</p>
            <h1 className="mt-2 text-3xl font-bold text-slate-950">{{ name_pascal }} #{item.id}</h1>
          </div>
          <div className="flex gap-3">
            <Link
              href="/{{ module }}/{{ table }}"
              className="rounded-xl border border-slate-300 px-4 py-2 text-sm font-semibold text-slate-700 hover:bg-slate-50"
            >
              Back
            </Link>
            <Link
              href={`/{{ module }}/{{ table }}/${item.id}/edit`}
              className="rounded-xl bg-slate-950 px-4 py-2 text-sm font-semibold text-white hover:bg-slate-800"
            >
              Edit
            </Link>
          </div>
        </div>

        <dl className="overflow-hidden rounded-2xl border border-slate-200 bg-white shadow-sm">
{% for field in fields %}          <div className="border-b border-slate-100 px-6 py-4 last:border-b-0">
            <dt className="text-xs font-semibold uppercase tracking-[0.2em] text-slate-500">{{ field.title }}</dt>
            <dd className="mt-2 whitespace-pre-wrap text-sm text-slate-900">{String(item['{{ field.name }}'] ?? '')}</dd>
          </div>
{% endfor %}        </dl>

        <button
          type="button"
          onClick={() => {
            if (window.confirm('Delete this {{ name_pascal }}?')) {
              router.delete('/{{ module }}/{{ table }}/' + item.id)
            }
          }}
          className="mt-6 rounded-xl border border-red-200 px-4 py-2 text-sm font-semibold text-red-600 hover:bg-red-50"
        >
          Delete {{ name_pascal }}
        </button>
      </div>
    </AppLayout>
  )
}
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
