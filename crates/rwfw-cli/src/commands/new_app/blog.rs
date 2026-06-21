//! File emitters for the generated app + blog example module.
use super::*;

pub(super) fn write_blog_example(
    app_dir: &Path,
    context: &AppTemplateContext,
) -> anyhow::Result<()> {
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
        &app_dir.join(
            "crates/modules/blog/src/migrations/20260101000000_create_posts_table.sqlite.sql",
        ),
        blog_create_posts_table_sqlite_sql(),
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

pub(super) fn blog_cargo_toml(context: &AppTemplateContext) -> String {
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

pub(super) fn app_lib_rs(context: &AppTemplateContext) -> String {
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

pub(super) fn app_main_rs(context: &AppTemplateContext) -> String {
    app_main_template().replace("__APP_CRATE_IDENT__", &context.app_crate_ident)
}

pub(super) fn app_main_template() -> &'static str {
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

pub(super) fn home_lib_rs() -> String {
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

pub(super) fn migrations_mod_rs() -> String {
    r#"use rwfw_core::migration::Migration;

pub fn migrations() -> Vec<Migration> {
    vec![
    ]
}
"#
    .to_string()
}

pub(super) fn home_index_route_rs() -> String {
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

pub(super) fn home_index_page_template() -> &'static str {
    r#"{% extends "layouts/app.html.j2" %}
{% block title %}{{ title }}{% endblock %}
{% block content %}
<div class="max-w-3xl mx-auto px-6">
  <header class="pt-20 pb-16 border-b border-stone-200">
    <p class="text-xs font-semibold uppercase tracking-[0.2em] text-indigo-700 mb-6">Dispatches &amp; long reads</p>
    <h1 class="font-serif text-5xl sm:text-6xl leading-[1.05] tracking-tight text-stone-900">{{ title }}</h1>
    <p class="mt-8 text-lg leading-relaxed text-stone-600 max-w-2xl">{{ description }}</p>
  </header>

  <div class="grid grid-cols-1 sm:grid-cols-2 gap-x-12 gap-y-12 py-16">
    <section>
      <p class="text-xs font-semibold uppercase tracking-[0.2em] text-indigo-700 mb-3">Modular architecture</p>
      <p class="text-stone-600 leading-relaxed">Each module is self-contained with its own routes, models, and templates.</p>
    </section>
    <section>
      <p class="text-xs font-semibold uppercase tracking-[0.2em] text-indigo-700 mb-3">Hotwire + MiniJinja</p>
      <p class="text-stone-600 leading-relaxed">Server-rendered HTML with Turbo navigation — no hydration, no npm.</p>
    </section>
    <section>
      <p class="text-xs font-semibold uppercase tracking-[0.2em] text-indigo-700 mb-3">Turbo Streams</p>
      <p class="text-stone-600 leading-relaxed">Real-time partial updates over SSE/WebSocket without a JS framework.</p>
    </section>
    <section>
      <p class="text-xs font-semibold uppercase tracking-[0.2em] text-indigo-700 mb-3">Single binary deploy</p>
      <p class="text-stone-600 leading-relaxed">All assets embedded in one Rust binary for simple deployment.</p>
    </section>
  </div>
</div>
{% endblock %}
"#
}

pub(super) fn blog_lib_rs() -> String {
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

    fn web_root(&self) -> Option<std::path::PathBuf> {
        Some(std::path::PathBuf::from(env!("CARGO_MANIFEST_DIR")).join("web"))
    }
}

inventory::submit! {
    ModuleRegistration::new("blog", || Box::new(BlogModule::new()))
}
"#
    .to_string()
}

pub(super) fn blog_migrations_mod_rs() -> String {
    r#"use rwfw_core::migration::Migration;

pub fn migrations() -> Vec<Migration> {
    vec![
        Migration::with_sqlite(
            "20260101000000",
            "create_posts_table",
            include_str!("20260101000000_create_posts_table.sql"),
            include_str!("20260101000000_create_posts_table.sqlite.sql"),
        ),
    ]
}
"#
    .to_string()
}

pub(super) fn blog_create_posts_table_sql() -> String {
    r#"CREATE TABLE IF NOT EXISTS blog_posts (
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
    ON blog_posts (status, published_at DESC);

INSERT INTO blog_posts
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

pub(super) fn blog_create_posts_table_sqlite_sql() -> String {
    r#"CREATE TABLE IF NOT EXISTS blog_posts (
    id INTEGER PRIMARY KEY AUTOINCREMENT,
    title TEXT NOT NULL,
    slug TEXT NOT NULL UNIQUE,
    excerpt TEXT NOT NULL,
    body TEXT NOT NULL,
    cover_image_url TEXT,
    status TEXT NOT NULL DEFAULT 'draft',
    published_at TEXT,
    author_id TEXT,
    created_at TEXT NOT NULL DEFAULT (strftime('%Y-%m-%dT%H:%M:%f+00:00','now')),
    updated_at TEXT NOT NULL DEFAULT (strftime('%Y-%m-%dT%H:%M:%f+00:00','now'))
);

CREATE INDEX IF NOT EXISTS idx_blog_posts_status_published_at
    ON blog_posts (status, published_at DESC);

INSERT INTO blog_posts
    (title, slug, excerpt, body, cover_image_url, status, published_at, author_id, created_at, updated_at)
VALUES
    (
        'Building Rails-like conventions in Rust',
        'building-rails-like-conventions-in-rust',
        'A practical look at modules, migrations, routing and generated CRUD in RWFW.',
        'RWFW starts from the idea that internal software should be boring in the right places. Modules own their routes, migrations and UI, while the app crate only composes them. That keeps the framework explicit without forcing every team to rewrite the same glue.',
        'https://images.unsplash.com/photo-1515879218367-8466d910aaa4?auto=format&fit=crop&w=1200&q=80',
        'published',
        strftime('%Y-%m-%dT%H:%M:%f+00:00','now','-3 days'),
        NULL,
        strftime('%Y-%m-%dT%H:%M:%f+00:00','now','-3 days'),
        strftime('%Y-%m-%dT%H:%M:%f+00:00','now','-3 days')
    ),
    (
        'Why migrations belong to modules',
        'why-migrations-belong-to-modules',
        'Module-owned migrations make feature boundaries visible and distributable.',
        'A module should be able to explain the data it owns. Keeping migrations close to module code makes generated applications easier to inspect, copy and evolve. RWFW still keeps a central migration ledger so execution stays predictable.',
        'https://images.unsplash.com/photo-1451187580459-43490279c0fa?auto=format&fit=crop&w=1200&q=80',
        'published',
        strftime('%Y-%m-%dT%H:%M:%f+00:00','now','-1 days'),
        NULL,
        strftime('%Y-%m-%dT%H:%M:%f+00:00','now','-1 days'),
        strftime('%Y-%m-%dT%H:%M:%f+00:00','now','-1 days')
    )
ON CONFLICT (slug) DO NOTHING;
"#
    .to_string()
}

pub(super) fn blog_models_mod_rs() -> String {
    r#"pub mod post;

pub use post::Post;
"#
    .to_string()
}

pub(super) fn blog_post_model_rs() -> String {
    r#"use sea_orm::entity::prelude::*;
use serde::{Deserialize, Serialize};

#[derive(Clone, Debug, PartialEq, DeriveEntityModel, Serialize, Deserialize)]
#[sea_orm(table_name = "blog_posts")]
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

pub(super) fn blog_repositories_mod_rs() -> String {
    r#"pub mod post_repo;
"#
    .to_string()
}

pub(super) fn blog_post_repository_rs() -> String {
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

pub(super) fn blog_use_cases_mod_rs() -> String {
    r#"pub mod save_post;
"#
    .to_string()
}

pub(super) fn blog_save_post_use_case_rs() -> String {
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

pub(super) fn blog_index_route_rs() -> String {
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

pub(super) fn blog_posts_index_route_rs() -> String {
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

pub(super) fn blog_post_show_route_rs() -> String {
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

pub(super) fn blog_admin_posts_index_route_rs() -> String {
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

pub(super) fn blog_admin_posts_new_route_rs() -> String {
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

pub(super) fn blog_admin_post_item_route_rs() -> String {
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

pub(super) fn blog_admin_post_edit_route_rs() -> String {
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

pub(super) fn blog_index_page_template() -> String {
    r#"{% extends "layouts/reading.html.j2" %}
{% block title %}Blog{% endblock %}
{% block content %}
{% if posts | length > 0 and posts[0].cover_image_url %}
<a href="/blog/posts/{{ posts[0].slug }}" class="group relative block h-[60vh] w-full overflow-hidden">
  <img src="{{ posts[0].cover_image_url }}" alt="" class="absolute inset-0 h-full w-full object-cover transition-transform duration-700 group-hover:scale-105">
  <div class="absolute inset-0 bg-gradient-to-t from-stone-900/85 via-stone-900/30 to-transparent"></div>
  <div class="absolute inset-x-0 bottom-0">
    <div class="max-w-3xl mx-auto px-6 pb-14">
      <p class="text-xs font-semibold uppercase tracking-[0.2em] text-indigo-300 mb-4">{{ posts[0].published_at | default(posts[0].created_at) }}</p>
      <h2 class="font-serif text-4xl sm:text-5xl leading-[1.05] tracking-tight text-white">{{ posts[0].title }}</h2>
      <p class="mt-4 max-w-2xl text-lg leading-relaxed text-stone-200">{{ posts[0].excerpt }}</p>
    </div>
  </div>
</a>
{% endif %}

<div class="max-w-3xl mx-auto px-6">
  <div class="flex items-end justify-between pt-16 pb-10 border-b border-stone-200">
    <h1 class="font-serif text-4xl sm:text-5xl tracking-tight text-stone-900">The Journal</h1>
    {% if auth and auth.user %}
    <a href="/blog/admin/posts" class="text-xs font-semibold uppercase tracking-[0.2em] text-indigo-700 hover:text-indigo-800">Manage posts</a>
    {% endif %}
  </div>

  {% if posts | length == 0 %}
    <p class="py-16 text-stone-400 text-lg">No published posts yet.</p>
  {% else %}
    <div class="divide-y divide-stone-200">
      {% for post in posts %}
      <article class="py-12">
        <p class="text-xs font-semibold uppercase tracking-[0.2em] text-indigo-700 mb-4">{{ post.published_at | default(post.created_at) }}</p>
        <a href="/blog/posts/{{ post.slug }}" class="group block">
          <h2 class="font-serif text-2xl sm:text-3xl leading-snug tracking-tight text-stone-900 group-hover:text-indigo-700 transition-colors">{{ post.title }}</h2>
        </a>
        <p class="mt-3 text-stone-600 leading-relaxed">{{ post.excerpt }}</p>
      </article>
      {% endfor %}
    </div>
  {% endif %}
</div>
{% endblock %}
"#
    .to_string()
}

pub(super) fn blog_posts_index_page_template() -> String {
    r#"{% extends "layouts/reading.html.j2" %}
{% block title %}Posts{% endblock %}
{% block content %}
<div class="max-w-3xl mx-auto px-6">
  <div class="flex items-end justify-between pt-16 pb-10 border-b border-stone-200">
    <h1 class="font-serif text-4xl sm:text-5xl tracking-tight text-stone-900">All Posts</h1>
    {% if auth and auth.user %}
    <a href="/blog/admin/posts/new" class="text-xs font-semibold uppercase tracking-[0.2em] text-indigo-700 hover:text-indigo-800">New post</a>
    {% endif %}
  </div>

  {% if posts | length == 0 %}
    <p class="py-16 text-stone-400 text-lg">No posts yet.</p>
  {% else %}
    <div class="divide-y divide-stone-200">
      {% for post in posts %}
      <article class="py-12">
        <p class="text-xs font-semibold uppercase tracking-[0.2em] text-indigo-700 mb-4">{{ post.published_at | default(post.created_at) }}</p>
        <a href="/blog/posts/{{ post.slug }}" class="group block">
          <h2 class="font-serif text-2xl sm:text-3xl leading-snug tracking-tight text-stone-900 group-hover:text-indigo-700 transition-colors">{{ post.title }}</h2>
        </a>
        <p class="mt-3 text-stone-600 leading-relaxed">{{ post.excerpt }}</p>
      </article>
      {% endfor %}
    </div>
  {% endif %}
</div>
{% endblock %}
"#
    .to_string()
}

pub(super) fn blog_post_show_page_template() -> String {
    r#"{% extends "layouts/reading.html.j2" %}
{% block title %}{{ post.title }}{% endblock %}
{% block content %}
<article class="max-w-2xl mx-auto px-6 pt-16 pb-24">
  <div class="mb-10">
    <a href="/blog/posts" class="text-xs font-semibold uppercase tracking-[0.2em] text-indigo-700 hover:text-indigo-800">&larr; Back to posts</a>
  </div>

  <header class="text-center">
    <p class="text-xs font-semibold uppercase tracking-[0.2em] text-indigo-700 mb-6">{{ post.published_at | default(post.created_at) }}</p>
    <h1 class="font-serif text-4xl sm:text-5xl leading-[1.08] tracking-tight text-stone-900">{{ post.title }}</h1>
    <p class="mt-6 text-lg leading-relaxed text-stone-600">{{ post.excerpt }}</p>
  </header>

  {% if post.cover_image_url %}
  <img src="{{ post.cover_image_url }}" alt="" class="mt-12 w-full object-cover">
  {% endif %}

  <div class="prose prose-stone max-w-none mt-12 font-serif text-lg leading-relaxed text-stone-800">
    <p>{{ post.body }}</p>
  </div>

  {% if auth and auth.user %}
  <div class="mt-16 pt-8 border-t border-stone-200 text-center">
    <a href="/blog/admin/posts/{{ post.id }}/edit" class="text-xs font-semibold uppercase tracking-[0.2em] text-indigo-700 hover:text-indigo-800">Edit this post</a>
  </div>
  {% endif %}
</article>
{% endblock %}
"#
    .to_string()
}

pub(super) fn blog_admin_posts_index_page_template() -> String {
    r#"{% extends "layouts/app.html.j2" %}
{% block title %}Manage Posts{% endblock %}
{% block content %}
<div class="max-w-4xl">
  <div class="flex justify-between items-center mb-8">
    <div>
      <h1 class="font-serif text-3xl tracking-tight text-stone-900">Manage Posts</h1>
      <p class="mt-1 text-sm text-stone-500">Drafts, publishing, and editing.</p>
    </div>
    <a href="/blog/admin/posts/new" class="inline-flex items-center rounded-md bg-indigo-700 px-4 py-2 text-sm font-medium text-white shadow-sm hover:bg-indigo-800">New Post</a>
  </div>

  {% if posts | length == 0 %}
    <div class="rounded-lg border border-dashed border-stone-300 bg-white px-6 py-16 text-center">
      <p class="text-stone-500">No posts yet. Create the first one.</p>
    </div>
  {% else %}
  <div class="overflow-hidden rounded-lg border border-stone-200 bg-white">
    <table class="min-w-full divide-y divide-stone-200">
      <thead class="bg-stone-50">
        <tr>
          <th class="px-5 py-3 text-left text-xs font-semibold uppercase tracking-wider text-stone-500">Title</th>
          <th class="px-5 py-3 text-left text-xs font-semibold uppercase tracking-wider text-stone-500">Status</th>
          <th class="px-5 py-3 text-right text-xs font-semibold uppercase tracking-wider text-stone-500">Actions</th>
        </tr>
      </thead>
      <tbody class="divide-y divide-stone-100">
        {% for post in posts %}
        <tr class="hover:bg-stone-50/60">
          <td class="px-5 py-4">
            <a href="/blog/admin/posts/{{ post.id }}/edit" class="font-medium text-stone-900 hover:text-indigo-700">{{ post.title }}</a>
          </td>
          <td class="px-5 py-4">
            {% if post.status == 'published' %}
            <span class="inline-flex items-center rounded-full bg-emerald-50 px-2.5 py-0.5 text-xs font-medium text-emerald-700 ring-1 ring-inset ring-emerald-600/20">Published</span>
            {% else %}
            <span class="inline-flex items-center rounded-full bg-stone-100 px-2.5 py-0.5 text-xs font-medium text-stone-600 ring-1 ring-inset ring-stone-500/20">Draft</span>
            {% endif %}
          </td>
          <td class="px-5 py-4 text-right">
            <a href="/blog/admin/posts/{{ post.id }}/edit" class="text-sm font-medium text-indigo-700 hover:text-indigo-800">Edit</a>
            <a href="/blog/admin/posts/{{ post.id }}" data-turbo-method="delete"
               data-turbo-confirm="Delete this post?"
               class="ml-4 text-sm font-medium text-stone-400 hover:text-red-600">Delete</a>
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

pub(super) fn blog_admin_post_form_template() -> String {
    r#"<form method="post" action="{{ action }}" class="rounded-lg border border-stone-200 bg-white p-8 space-y-6">
  <input type="hidden" name="_csrf" value="{{ csrf_token }}">
  <x-field name="title" label="Title" :value="old.title | default(post.title) | default('')" />
  <x-field name="slug" label="Slug (optional)" :value="old.slug | default(post.slug) | default('')" :required="false" />
  <x-field name="excerpt" label="Excerpt" type="textarea" :value="old.excerpt | default(post.excerpt) | default('')" />
  <x-field name="body" label="Body" type="textarea" :value="old.body | default(post.body) | default('')" />
  <x-field name="cover_image_url" label="Cover image URL (optional)" :value="old.cover_image_url | default(post.cover_image_url) | default('')" :required="false" />
  <div>
    <label for="status" class="block text-sm font-medium text-stone-700">Status</label>
    {% set current_status = old.status | default(post.status) | default('draft') %}
    <select id="status" name="status"
            class="mt-1 block w-full rounded-md border border-stone-300 px-3 py-2 shadow-sm focus:border-indigo-500 focus:outline-none focus:ring-1 focus:ring-indigo-500">
      <option value="draft" {{ 'selected' if current_status == 'draft' }}>Draft</option>
      <option value="published" {{ 'selected' if current_status == 'published' }}>Published</option>
    </select>
    {% if errors.status %}<p class="mt-1 text-sm text-red-600">{{ errors.status }}</p>{% endif %}
  </div>
  <div class="flex gap-3 pt-2 border-t border-stone-100">
    <button type="submit" data-turbo-submits-with="Saving..." class="rounded-md bg-indigo-700 px-6 py-2 text-sm font-medium text-white shadow-sm hover:bg-indigo-800 disabled:opacity-50">{{ submit_label }}</button>
    <a href="/blog/admin/posts" class="rounded-md border border-stone-300 px-6 py-2 text-sm font-medium text-stone-700 hover:bg-stone-50">Cancel</a>
  </div>
</form>
"#
    .to_string()
}

pub(super) fn blog_admin_post_new_page_template() -> String {
    r#"{% extends "layouts/app.html.j2" %}
{% block title %}New Post{% endblock %}
{% block content %}
<div class="max-w-3xl">
  <div class="mb-6">
    <a href="/blog/admin/posts" class="text-xs font-semibold uppercase tracking-[0.2em] text-indigo-700 hover:text-indigo-800">&larr; Back to posts</a>
  </div>
  <h1 class="font-serif text-3xl tracking-tight text-stone-900 mb-8">New Post</h1>
  {% set action = "/blog/admin/posts" %}
  {% set submit_label = "Create Post" %}
  {% include "blog/admin/posts/_form.html.j2" %}
</div>
{% endblock %}
"#
    .to_string()
}

pub(super) fn blog_admin_post_edit_page_template() -> String {
    r#"{% extends "layouts/app.html.j2" %}
{% block title %}Edit Post{% endblock %}
{% block content %}
<div class="max-w-3xl">
  <div class="mb-6">
    <a href="/blog/admin/posts" class="text-xs font-semibold uppercase tracking-[0.2em] text-indigo-700 hover:text-indigo-800">&larr; Back to posts</a>
  </div>
  <h1 class="font-serif text-3xl tracking-tight text-stone-900 mb-8">Edit Post</h1>
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
