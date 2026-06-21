//! Dockerfile, compose, entrypoint and ignore-file emitters.
use super::*;

pub(super) fn gitignore() -> String {
    r#"/target
.env
.DS_Store
/data/*.db
/data/*.db-wal
/data/*.db-shm
"#
    .to_string()
}

pub(super) fn dockerignore() -> String {
    r#"/target
/.git
/.env
*.log
"#
    .to_string()
}

pub(super) fn env_example() -> String {
    r#"# RWFW local environment overrides
# SSO client credentials are appended by `rwfw auth sso add`.
"#
    .to_string()
}

pub(super) fn dockerfile(context: &AppTemplateContext) -> String {
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

pub(super) fn compose_yaml(context: &AppTemplateContext) -> String {
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

pub(super) fn compose_dev_yaml(context: &AppTemplateContext) -> String {
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

pub(super) fn docker_entrypoint_sh() -> String {
    r#"#!/usr/bin/env sh
set -eu

if [ "${RWFW_RUN_MIGRATIONS:-1}" = "1" ]; then
  rwfw-app __rwfw migrate
fi

exec rwfw-app
"#
    .to_string()
}
