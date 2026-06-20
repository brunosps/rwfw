//! File emitters for the generated e-commerce (shop) example module.
use super::*;

pub(super) fn write_ecommerce_example(
    app_dir: &Path,
    context: &AppTemplateContext,
) -> anyhow::Result<()> {
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
        &app_dir.join(
            "crates/modules/shop/src/migrations/20260101000000_create_shop_tables.sqlite.sql",
        ),
        shop_create_tables_sqlite_sql(),
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
        &app_dir.join("crates/modules/shop/src/use_cases/save_product.rs"),
        shop_save_product_use_case_rs(),
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
    // admin (backoffice) routes
    write_file(
        &app_dir.join("crates/modules/shop/src/routes/admin/products/index.rs"),
        shop_admin_products_index_route_rs(),
    )?;
    write_file(
        &app_dir.join("crates/modules/shop/src/routes/admin/products/new.rs"),
        shop_admin_products_new_route_rs(),
    )?;
    write_file(
        &app_dir.join("crates/modules/shop/src/routes/admin/products/[id].rs"),
        shop_admin_products_item_route_rs(),
    )?;
    write_file(
        &app_dir.join("crates/modules/shop/src/routes/admin/products/[id]/edit.rs"),
        shop_admin_products_edit_route_rs(),
    )?;
    write_file(
        &app_dir.join("crates/modules/shop/src/routes/admin/orders/index.rs"),
        shop_admin_orders_index_route_rs(),
    )?;
    write_file(
        &app_dir.join("crates/modules/shop/src/routes/admin/orders/[id].rs"),
        shop_admin_orders_show_route_rs(),
    )?;
    // admin (backoffice) templates
    write_file(
        &app_dir.join("crates/modules/shop/web/templates/admin/products/index.html.j2"),
        shop_admin_products_index_template(),
    )?;
    write_file(
        &app_dir.join("crates/modules/shop/web/templates/admin/products/_form.html.j2"),
        shop_admin_products_form_template(),
    )?;
    write_file(
        &app_dir.join("crates/modules/shop/web/templates/admin/products/new.html.j2"),
        shop_admin_products_new_template(),
    )?;
    write_file(
        &app_dir.join("crates/modules/shop/web/templates/admin/products/edit.html.j2"),
        shop_admin_products_edit_template(),
    )?;
    write_file(
        &app_dir.join("crates/modules/shop/web/templates/admin/orders/index.html.j2"),
        shop_admin_orders_index_template(),
    )?;
    write_file(
        &app_dir.join("crates/modules/shop/web/templates/admin/orders/show.html.j2"),
        shop_admin_orders_show_template(),
    )?;

    Ok(())
}

pub(super) fn shop_cargo_toml(context: &AppTemplateContext) -> String {
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

pub(super) fn shop_lib_rs() -> String {
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
        vec![
            Permission::new("shop.products.manage", "Manage shop products"),
            Permission::new("shop.orders.view", "View shop orders"),
        ]
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
            NavItem {
                label: "Shop Admin".to_string(),
                href: "/shop/admin/products".to_string(),
                icon: Some("dashboard".to_string()),
            },
            NavItem {
                label: "Orders".to_string(),
                href: "/shop/admin/orders".to_string(),
                icon: Some("receipt".to_string()),
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

pub(super) fn shop_cart_rs() -> String {
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

pub(super) fn shop_migrations_mod_rs() -> String {
    r#"use rwfw_core::migration::Migration;

pub fn migrations() -> Vec<Migration> {
    vec![
        Migration::with_sqlite(
            "20260101000000",
            "create_shop_tables",
            include_str!("20260101000000_create_shop_tables.sql"),
            include_str!("20260101000000_create_shop_tables.sqlite.sql"),
        ),
    ]
}
"#
    .to_string()
}

pub(super) fn shop_create_tables_sql() -> String {
    r#"CREATE TABLE IF NOT EXISTS shop_categories (
    id SERIAL PRIMARY KEY,
    name VARCHAR(255) NOT NULL,
    slug VARCHAR(255) NOT NULL UNIQUE,
    description TEXT NOT NULL,
    created_at TIMESTAMPTZ NOT NULL DEFAULT NOW(),
    updated_at TIMESTAMPTZ NOT NULL DEFAULT NOW()
);

CREATE TABLE IF NOT EXISTS shop_products (
    id SERIAL PRIMARY KEY,
    category_id INTEGER REFERENCES shop_categories(id) ON DELETE SET NULL,
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

CREATE TABLE IF NOT EXISTS shop_orders (
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

CREATE TABLE IF NOT EXISTS shop_order_items (
    id SERIAL PRIMARY KEY,
    order_id INTEGER NOT NULL REFERENCES shop_orders(id) ON DELETE CASCADE,
    product_id INTEGER,
    product_name VARCHAR(255) NOT NULL,
    product_slug VARCHAR(255) NOT NULL,
    unit_price_cents INTEGER NOT NULL,
    quantity INTEGER NOT NULL,
    subtotal_cents INTEGER NOT NULL,
    created_at TIMESTAMPTZ NOT NULL DEFAULT NOW()
);

CREATE INDEX IF NOT EXISTS idx_shop_products_active_featured
    ON shop_products (active, featured);

INSERT INTO shop_categories (name, slug, description)
VALUES
    ('Desk', 'desk', 'Objects for focused workspaces.'),
    ('Carry', 'carry', 'Bags and daily tools for moving between contexts.'),
    ('Sound', 'sound', 'Audio gear for deep work.')
ON CONFLICT (slug) DO NOTHING;

INSERT INTO shop_products
    (category_id, name, slug, description, price_cents, currency, image_url, inventory, featured, active)
VALUES
    (
        (SELECT id FROM shop_categories WHERE slug = 'desk'),
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
        (SELECT id FROM shop_categories WHERE slug = 'desk'),
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
        (SELECT id FROM shop_categories WHERE slug = 'carry'),
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
        (SELECT id FROM shop_categories WHERE slug = 'sound'),
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

pub(super) fn shop_create_tables_sqlite_sql() -> String {
    r#"CREATE TABLE IF NOT EXISTS shop_categories (
    id INTEGER PRIMARY KEY AUTOINCREMENT,
    name TEXT NOT NULL,
    slug TEXT NOT NULL UNIQUE,
    description TEXT NOT NULL,
    created_at TEXT NOT NULL DEFAULT (strftime('%Y-%m-%dT%H:%M:%f+00:00','now')),
    updated_at TEXT NOT NULL DEFAULT (strftime('%Y-%m-%dT%H:%M:%f+00:00','now'))
);

CREATE TABLE IF NOT EXISTS shop_products (
    id INTEGER PRIMARY KEY AUTOINCREMENT,
    category_id INTEGER REFERENCES shop_categories(id) ON DELETE SET NULL,
    name TEXT NOT NULL,
    slug TEXT NOT NULL UNIQUE,
    description TEXT NOT NULL,
    price_cents INTEGER NOT NULL,
    currency TEXT NOT NULL DEFAULT 'USD',
    image_url TEXT NOT NULL,
    inventory INTEGER NOT NULL DEFAULT 0,
    featured INTEGER NOT NULL DEFAULT 0,
    active INTEGER NOT NULL DEFAULT 1,
    created_at TEXT NOT NULL DEFAULT (strftime('%Y-%m-%dT%H:%M:%f+00:00','now')),
    updated_at TEXT NOT NULL DEFAULT (strftime('%Y-%m-%dT%H:%M:%f+00:00','now'))
);

CREATE TABLE IF NOT EXISTS shop_orders (
    id INTEGER PRIMARY KEY AUTOINCREMENT,
    number TEXT NOT NULL UNIQUE,
    customer_name TEXT NOT NULL,
    customer_email TEXT NOT NULL,
    address_line TEXT NOT NULL,
    city TEXT NOT NULL,
    country TEXT NOT NULL,
    total_cents INTEGER NOT NULL,
    currency TEXT NOT NULL DEFAULT 'USD',
    status TEXT NOT NULL DEFAULT 'paid_fake',
    created_at TEXT NOT NULL DEFAULT (strftime('%Y-%m-%dT%H:%M:%f+00:00','now')),
    updated_at TEXT NOT NULL DEFAULT (strftime('%Y-%m-%dT%H:%M:%f+00:00','now'))
);

CREATE TABLE IF NOT EXISTS shop_order_items (
    id INTEGER PRIMARY KEY AUTOINCREMENT,
    order_id INTEGER NOT NULL REFERENCES shop_orders(id) ON DELETE CASCADE,
    product_id INTEGER,
    product_name TEXT NOT NULL,
    product_slug TEXT NOT NULL,
    unit_price_cents INTEGER NOT NULL,
    quantity INTEGER NOT NULL,
    subtotal_cents INTEGER NOT NULL,
    created_at TEXT NOT NULL DEFAULT (strftime('%Y-%m-%dT%H:%M:%f+00:00','now'))
);

CREATE INDEX IF NOT EXISTS idx_shop_products_active_featured
    ON shop_products (active, featured);

INSERT INTO shop_categories (name, slug, description)
VALUES
    ('Desk', 'desk', 'Objects for focused workspaces.'),
    ('Carry', 'carry', 'Bags and daily tools for moving between contexts.'),
    ('Sound', 'sound', 'Audio gear for deep work.')
ON CONFLICT (slug) DO NOTHING;

INSERT INTO shop_products
    (category_id, name, slug, description, price_cents, currency, image_url, inventory, featured, active)
VALUES
    (
        (SELECT id FROM shop_categories WHERE slug = 'desk'),
        'Machined Keyboard Tray',
        'machined-keyboard-tray',
        'A low-profile aluminum tray that keeps your keyboard and notes aligned for long work sessions.',
        12900,
        'USD',
        'https://images.unsplash.com/photo-1516321318423-f06f85e504b3?auto=format&fit=crop&w=1200&q=80',
        18,
        1,
        1
    ),
    (
        (SELECT id FROM shop_categories WHERE slug = 'desk'),
        'Task Lamp One',
        'task-lamp-one',
        'Warm directional light with a heavy base and a single mechanical hinge.',
        18900,
        'USD',
        'https://images.unsplash.com/photo-1507473885765-e6ed057f782c?auto=format&fit=crop&w=1200&q=80',
        9,
        1,
        1
    ),
    (
        (SELECT id FROM shop_categories WHERE slug = 'carry'),
        'Field Pack 24L',
        'field-pack-24l',
        'A weather resistant everyday pack with structured compartments for laptop, camera and cables.',
        24000,
        'USD',
        'https://images.unsplash.com/photo-1622560480605-d83c853bc5c3?auto=format&fit=crop&w=1200&q=80',
        12,
        1,
        1
    ),
    (
        (SELECT id FROM shop_categories WHERE slug = 'sound'),
        'Studio Monitor Headphones',
        'studio-monitor-headphones',
        'Closed-back headphones tuned for clear calls, editing and focused work.',
        16000,
        'USD',
        'https://images.unsplash.com/photo-1505740420928-5e560c06d30e?auto=format&fit=crop&w=1200&q=80',
        24,
        0,
        1
    )
ON CONFLICT (slug) DO NOTHING;
"#
    .to_string()
}

pub(super) fn shop_models_mod_rs() -> String {
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

pub(super) fn shop_category_model_rs() -> String {
    r#"use sea_orm::entity::prelude::*;
use serde::{Deserialize, Serialize};

#[derive(Clone, Debug, PartialEq, DeriveEntityModel, Serialize, Deserialize)]
#[sea_orm(table_name = "shop_categories")]
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

pub(super) fn shop_product_model_rs() -> String {
    r#"use sea_orm::entity::prelude::*;
use serde::{Deserialize, Serialize};

#[derive(Clone, Debug, PartialEq, DeriveEntityModel, Serialize, Deserialize)]
#[sea_orm(table_name = "shop_products")]
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

pub(super) fn shop_order_model_rs() -> String {
    r#"use sea_orm::entity::prelude::*;
use serde::{Deserialize, Serialize};

#[derive(Clone, Debug, PartialEq, DeriveEntityModel, Serialize, Deserialize)]
#[sea_orm(table_name = "shop_orders")]
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

pub(super) fn shop_order_item_model_rs() -> String {
    r#"use sea_orm::entity::prelude::*;
use serde::{Deserialize, Serialize};

#[derive(Clone, Debug, PartialEq, DeriveEntityModel, Serialize, Deserialize)]
#[sea_orm(table_name = "shop_order_items")]
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

pub(super) fn shop_repositories_mod_rs() -> String {
    r#"pub mod shop_repo;
"#
    .to_string()
}

pub(super) fn shop_repository_rs() -> String {
    r#"use crate::models::category::{self, Column as CategoryColumn, Entity as CategoryEntity};
use crate::models::order::{self, ActiveModel as OrderActiveModel, Column as OrderColumn, Entity as OrderEntity};
use crate::models::order_item::{self, ActiveModel as OrderItemActiveModel, Column as OrderItemColumn, Entity as OrderItemEntity};
use crate::models::product::{self, ActiveModel as ProductActiveModel, Column as ProductColumn, Entity as ProductEntity};
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

#[derive(Debug)]
pub struct SaveProduct {
    pub category_id: Option<i32>,
    pub name: String,
    pub slug: String,
    pub description: String,
    pub price_cents: i32,
    pub currency: String,
    pub image_url: String,
    pub inventory: i32,
    pub featured: bool,
    pub active: bool,
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

    // --- admin: products -------------------------------------------------

    pub async fn find_all_products_for_admin(&self) -> anyhow::Result<Vec<product::Model>> {
        Ok(ProductEntity::find()
            .order_by_desc(ProductColumn::UpdatedAt)
            .all(&self.db)
            .await?)
    }

    pub async fn find_product_by_id(&self, id: i32) -> anyhow::Result<Option<product::Model>> {
        Ok(ProductEntity::find_by_id(id).one(&self.db).await?)
    }

    pub async fn slug_exists(&self, slug: &str, except_id: Option<i32>) -> anyhow::Result<bool> {
        let mut query = ProductEntity::find().filter(ProductColumn::Slug.eq(slug));
        if let Some(id) = except_id {
            query = query.filter(ProductColumn::Id.ne(id));
        }
        Ok(query.one(&self.db).await?.is_some())
    }

    pub async fn create_product(&self, data: SaveProduct) -> anyhow::Result<product::Model> {
        let now = chrono::Utc::now().fixed_offset();
        let model = ProductActiveModel {
            category_id: Set(data.category_id),
            name: Set(data.name),
            slug: Set(data.slug),
            description: Set(data.description),
            price_cents: Set(data.price_cents),
            currency: Set(data.currency),
            image_url: Set(data.image_url),
            inventory: Set(data.inventory),
            featured: Set(data.featured),
            active: Set(data.active),
            created_at: Set(now),
            updated_at: Set(now),
            ..Default::default()
        };
        Ok(model.insert(&self.db).await?)
    }

    pub async fn update_product(&self, id: i32, data: SaveProduct) -> anyhow::Result<product::Model> {
        let product = ProductEntity::find_by_id(id)
            .one(&self.db)
            .await?
            .ok_or_else(|| anyhow::anyhow!("Product not found"))?;

        let mut active: ProductActiveModel = product.into();
        active.category_id = Set(data.category_id);
        active.name = Set(data.name);
        active.slug = Set(data.slug);
        active.description = Set(data.description);
        active.price_cents = Set(data.price_cents);
        active.currency = Set(data.currency);
        active.image_url = Set(data.image_url);
        active.inventory = Set(data.inventory);
        active.featured = Set(data.featured);
        active.active = Set(data.active);
        active.updated_at = Set(chrono::Utc::now().fixed_offset());
        Ok(active.update(&self.db).await?)
    }

    pub async fn delete_product(&self, id: i32) -> anyhow::Result<()> {
        ProductEntity::delete_by_id(id).exec(&self.db).await?;
        Ok(())
    }

    // --- admin: orders ---------------------------------------------------

    pub async fn find_all_orders_for_admin(&self) -> anyhow::Result<Vec<order::Model>> {
        Ok(OrderEntity::find()
            .order_by_desc(OrderColumn::CreatedAt)
            .all(&self.db)
            .await?)
    }

    pub async fn find_order_by_id(
        &self,
        id: i32,
    ) -> anyhow::Result<Option<(order::Model, Vec<order_item::Model>)>> {
        let Some(order) = OrderEntity::find_by_id(id).one(&self.db).await? else {
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

pub(super) fn shop_use_cases_mod_rs() -> String {
    r#"pub mod checkout;
pub mod save_product;
"#
    .to_string()
}

pub(super) fn shop_checkout_use_case_rs() -> String {
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

pub(super) fn shop_save_product_use_case_rs() -> String {
    r#"use crate::models::product;
use crate::repositories::shop_repo::{SaveProduct, ShopRepository};
use rwfw_core::error::AppError;
use serde::Deserialize;
use std::collections::HashMap;

#[derive(Debug, Deserialize)]
pub struct SaveProductInput {
    pub name: String,
    #[serde(default)]
    pub slug: Option<String>,
    #[serde(default)]
    pub description: String,
    #[serde(default)]
    pub price: String,
    #[serde(default = "default_currency")]
    pub currency: String,
    #[serde(default)]
    pub image_url: String,
    #[serde(default)]
    pub inventory: i32,
    #[serde(default)]
    pub category_id: Option<i32>,
    #[serde(default)]
    pub featured: Option<String>,
    #[serde(default)]
    pub active: Option<String>,
}

fn default_currency() -> String {
    "USD".to_string()
}

pub struct SaveProductUseCase;

impl SaveProductUseCase {
    pub async fn create(
        &self,
        repo: &ShopRepository,
        input: SaveProductInput,
    ) -> Result<product::Model, AppError> {
        let data = self.prepare(repo, None, input).await?;
        repo.create_product(data).await.map_err(AppError::Internal)
    }

    pub async fn update(
        &self,
        repo: &ShopRepository,
        id: i32,
        input: SaveProductInput,
    ) -> Result<product::Model, AppError> {
        repo.find_product_by_id(id)
            .await
            .map_err(AppError::Internal)?
            .ok_or_else(|| AppError::NotFound(format!("Product not found: {id}")))?;
        let data = self.prepare(repo, Some(id), input).await?;
        repo.update_product(id, data)
            .await
            .map_err(AppError::Internal)
    }

    async fn prepare(
        &self,
        repo: &ShopRepository,
        id: Option<i32>,
        input: SaveProductInput,
    ) -> Result<SaveProduct, AppError> {
        let name = input.name.trim().to_string();
        let description = input.description.trim().to_string();
        let image_url = input.image_url.trim().to_string();
        let currency = {
            let value = input.currency.trim().to_uppercase();
            if value.is_empty() {
                "USD".to_string()
            } else {
                value
            }
        };
        let inventory = input.inventory;
        let price_cents = parse_price_cents(input.price.trim());
        let featured = checkbox(&input.featured);
        let active = checkbox(&input.active);

        let mut errors = HashMap::new();
        if name.is_empty() {
            errors.insert("name".to_string(), vec!["Name is required".to_string()]);
        }
        match price_cents {
            Some(cents) if cents > 0 => {}
            _ => {
                errors.insert(
                    "price".to_string(),
                    vec!["Price must be greater than 0".to_string()],
                );
            }
        }
        if image_url.is_empty() {
            errors.insert(
                "image_url".to_string(),
                vec!["Image URL is required".to_string()],
            );
        }
        if inventory < 0 {
            errors.insert(
                "inventory".to_string(),
                vec!["Inventory must be zero or greater".to_string()],
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
            .unwrap_or_else(|| slugify(&name));
        let slug = unique_slug(repo, &base_slug, id).await?;

        Ok(SaveProduct {
            category_id: input.category_id,
            name,
            slug,
            description,
            price_cents: price_cents.unwrap_or(0),
            currency,
            image_url,
            inventory,
            featured,
            active,
        })
    }
}

fn checkbox(value: &Option<String>) -> bool {
    matches!(
        value.as_deref().map(str::trim),
        Some("on") | Some("true") | Some("1") | Some("yes")
    )
}

fn parse_price_cents(value: &str) -> Option<i32> {
    if value.is_empty() {
        return None;
    }
    let normalized = value.replace(',', ".");
    let amount: f64 = normalized.parse().ok()?;
    if !amount.is_finite() || amount < 0.0 {
        return None;
    }
    Some((amount * 100.0).round() as i32)
}

async fn unique_slug(
    repo: &ShopRepository,
    base: &str,
    except_id: Option<i32>,
) -> Result<String, AppError> {
    let base = if base.is_empty() { "product" } else { base };
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
"#
    .to_string()
}

pub(super) fn shop_index_route_rs() -> String {
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

pub(super) fn shop_product_show_route_rs() -> String {
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

pub(super) fn shop_cart_route_rs() -> String {
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

pub(super) fn shop_cart_add_route_rs() -> String {
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

pub(super) fn shop_cart_update_route_rs() -> String {
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

pub(super) fn shop_checkout_route_rs() -> String {
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

pub(super) fn shop_order_show_route_rs() -> String {
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

// ---------------------------------------------------------------------------
// Ecommerce admin (backoffice) — products CRUD + read-only orders. Mirrors the
// blog admin routes; guarded by `shop.products.manage` / `shop.orders.view`.
// ---------------------------------------------------------------------------

pub(super) fn shop_admin_products_index_route_rs() -> String {
    r#"use crate::repositories::shop_repo::ShopRepository;
use crate::use_cases::save_product::{SaveProductInput, SaveProductUseCase};
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
    if !user.can("shop.products.manage") {
        return AppError::Forbidden("Missing permission: shop.products.manage".into())
            .into_response();
    }

    let repo = ShopRepository::new(state.db.clone());
    let products = match repo.find_all_products_for_admin().await {
        Ok(products) => products,
        Err(error) => return AppError::Internal(error).into_response(),
    };

    v.render(
        "shop/admin/products/index",
        serde_json::json!({
            "products": products,
        }),
    )
}

async fn post(
    State(state): State<AppState>,
    user: CurrentUser,
    v: View,
    Form(input): Form<SaveProductInput>,
) -> Response {
    if !user.can("shop.products.manage") {
        return AppError::Forbidden("Missing permission: shop.products.manage".into())
            .into_response();
    }

    let old = serde_json::json!({
        "name": input.name,
        "slug": input.slug,
        "description": input.description,
        "price": input.price,
        "currency": input.currency,
        "image_url": input.image_url,
        "inventory": input.inventory,
        "category_id": input.category_id,
        "featured": input.featured,
        "active": input.active,
    });

    let repo = ShopRepository::new(state.db.clone());
    let categories = repo.categories().await.unwrap_or_default();
    let use_case = SaveProductUseCase;
    match use_case.create(&repo, input).await {
        Ok(product) => {
            Redirect::to(&format!("/shop/admin/products/{}/edit", product.id)).into_response()
        }
        Err(AppError::Validation(errors)) => v.render_status(
            StatusCode::UNPROCESSABLE_ENTITY,
            "shop/admin/products/new",
            serde_json::json!({
                "product": serde_json::Value::Null,
                "categories": categories,
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

pub(super) fn shop_admin_products_new_route_rs() -> String {
    r#"use crate::repositories::shop_repo::ShopRepository;
use axum::extract::State;
use axum::response::{IntoResponse, Response};
use axum::routing;
use rwfw_core::app::AppState;
use rwfw_core::auth::CurrentUser;
use rwfw_core::error::AppError;
use rwfw_core::view::View;

pub fn route() -> axum::routing::MethodRouter<AppState> {
    routing::get(get)
}

async fn get(State(state): State<AppState>, user: CurrentUser, v: View) -> Response {
    if !user.can("shop.products.manage") {
        return AppError::Forbidden("Missing permission: shop.products.manage".into())
            .into_response();
    }

    let repo = ShopRepository::new(state.db.clone());
    let categories = repo.categories().await.unwrap_or_default();

    v.render(
        "shop/admin/products/new",
        serde_json::json!({
            "product": serde_json::Value::Null,
            "categories": categories,
        }),
    )
}
"#
    .to_string()
}

pub(super) fn shop_admin_products_item_route_rs() -> String {
    r#"use crate::repositories::shop_repo::ShopRepository;
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
    if !user.can("shop.products.manage") {
        return AppError::Forbidden("Missing permission: shop.products.manage".into())
            .into_response();
    }

    let repo = ShopRepository::new(state.db.clone());
    match repo.delete_product(id).await {
        Ok(()) => Redirect::to("/shop/admin/products").into_response(),
        Err(error) => AppError::Internal(error).into_response(),
    }
}
"#
    .to_string()
}

pub(super) fn shop_admin_products_edit_route_rs() -> String {
    r#"use crate::repositories::shop_repo::ShopRepository;
use crate::use_cases::save_product::{SaveProductInput, SaveProductUseCase};
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
    if !user.can("shop.products.manage") {
        return AppError::Forbidden("Missing permission: shop.products.manage".into())
            .into_response();
    }

    let repo = ShopRepository::new(state.db.clone());
    let product = match repo.find_product_by_id(id).await {
        Ok(Some(product)) => product,
        Ok(None) => return AppError::NotFound(format!("Product not found: {id}")).into_response(),
        Err(error) => return AppError::Internal(error).into_response(),
    };
    let categories = repo.categories().await.unwrap_or_default();

    v.render(
        "shop/admin/products/edit",
        serde_json::json!({
            "product": product,
            "categories": categories,
        }),
    )
}

// HTML forms can't issue PUT, so the edit form POSTs here.
async fn put(
    State(state): State<AppState>,
    user: CurrentUser,
    v: View,
    Path(id): Path<i32>,
    Form(input): Form<SaveProductInput>,
) -> Response {
    if !user.can("shop.products.manage") {
        return AppError::Forbidden("Missing permission: shop.products.manage".into())
            .into_response();
    }

    let old = serde_json::json!({
        "id": id,
        "name": input.name,
        "slug": input.slug,
        "description": input.description,
        "price": input.price,
        "currency": input.currency,
        "image_url": input.image_url,
        "inventory": input.inventory,
        "category_id": input.category_id,
        "featured": input.featured,
        "active": input.active,
    });

    let repo = ShopRepository::new(state.db.clone());
    let categories = repo.categories().await.unwrap_or_default();
    let use_case = SaveProductUseCase;
    match use_case.update(&repo, id, input).await {
        Ok(product) => {
            Redirect::to(&format!("/shop/admin/products/{}/edit", product.id)).into_response()
        }
        Err(AppError::Validation(errors)) => v.render_status(
            StatusCode::UNPROCESSABLE_ENTITY,
            "shop/admin/products/edit",
            serde_json::json!({
                "product": old,
                "categories": categories,
                "errors": rwfw_core::validation::first_messages(errors),
            }),
        ),
        Err(error) => error.into_response(),
    }
}
"#
    .to_string()
}

pub(super) fn shop_admin_orders_index_route_rs() -> String {
    r#"use crate::repositories::shop_repo::ShopRepository;
use axum::extract::State;
use axum::response::{IntoResponse, Response};
use axum::routing;
use rwfw_core::app::AppState;
use rwfw_core::auth::CurrentUser;
use rwfw_core::error::AppError;
use rwfw_core::view::View;

pub fn route() -> axum::routing::MethodRouter<AppState> {
    routing::get(get)
}

async fn get(State(state): State<AppState>, user: CurrentUser, v: View) -> Response {
    if !user.can("shop.orders.view") {
        return AppError::Forbidden("Missing permission: shop.orders.view".into()).into_response();
    }

    let repo = ShopRepository::new(state.db.clone());
    let orders = match repo.find_all_orders_for_admin().await {
        Ok(orders) => orders,
        Err(error) => return AppError::Internal(error).into_response(),
    };

    let total_orders = orders.len();
    let revenue_cents: i64 = orders.iter().map(|order| order.total_cents as i64).sum();
    let currency = orders
        .first()
        .map(|order| order.currency.clone())
        .unwrap_or_else(|| "USD".to_string());

    v.render(
        "shop/admin/orders/index",
        serde_json::json!({
            "orders": orders,
            "total_orders": total_orders,
            "revenue_cents": revenue_cents,
            "currency": currency,
        }),
    )
}
"#
    .to_string()
}

pub(super) fn shop_admin_orders_show_route_rs() -> String {
    r#"use crate::repositories::shop_repo::ShopRepository;
use axum::extract::{Path, State};
use axum::response::{IntoResponse, Response};
use axum::routing;
use rwfw_core::app::AppState;
use rwfw_core::auth::CurrentUser;
use rwfw_core::error::AppError;
use rwfw_core::view::View;

pub fn route() -> axum::routing::MethodRouter<AppState> {
    routing::get(get)
}

async fn get(
    State(state): State<AppState>,
    user: CurrentUser,
    v: View,
    Path(id): Path<i32>,
) -> Response {
    if !user.can("shop.orders.view") {
        return AppError::Forbidden("Missing permission: shop.orders.view".into()).into_response();
    }

    let repo = ShopRepository::new(state.db.clone());
    let (order, items) = match repo.find_order_by_id(id).await {
        Ok(Some(order)) => order,
        Ok(None) => return AppError::NotFound(format!("Order not found: {id}")).into_response(),
        Err(error) => return AppError::Internal(error).into_response(),
    };

    v.render(
        "shop/admin/orders/show",
        serde_json::json!({
            "order": order,
            "items": items,
        }),
    )
}
"#
    .to_string()
}

pub(super) fn shop_index_page_template() -> String {
    r##"{% extends "layouts/app.html.j2" %}
{% block title %}Shop{% endblock %}
{% block content %}
<div class="font-sans text-slate-900">
  <section class="relative overflow-hidden rounded-2xl bg-gradient-to-r from-indigo-600 to-violet-600 text-white p-8 sm:p-10 mb-8 shadow-lg">
    <div class="relative z-10 max-w-2xl">
      <span class="inline-flex items-center rounded-full bg-white/15 px-3 py-1 text-xs font-semibold uppercase tracking-wider ring-1 ring-inset ring-white/25">New season</span>
      <h1 class="mt-4 text-3xl sm:text-4xl font-extrabold tracking-tight">Gear up. Stand out.</h1>
      <p class="mt-3 text-base sm:text-lg text-indigo-100">Curated essentials shipped fast. Discover featured drops and shop the full catalog below.</p>
      <div class="mt-6 flex flex-wrap gap-3">
        <a href="#all-products" class="inline-flex items-center rounded-lg bg-white px-5 py-2.5 text-sm font-semibold text-indigo-700 shadow-sm transition hover:-translate-y-0.5 hover:shadow-md">Shop the catalog</a>
        <a href="/shop/cart" class="inline-flex items-center rounded-lg bg-white/10 px-5 py-2.5 text-sm font-semibold text-white ring-1 ring-inset ring-white/25 transition hover:bg-white/20">View cart</a>
      </div>
    </div>
    <div class="pointer-events-none absolute -right-16 -top-16 h-64 w-64 rounded-full bg-white/10 blur-2xl"></div>
    <div class="pointer-events-none absolute -bottom-20 right-24 h-48 w-48 rounded-full bg-violet-400/20 blur-2xl"></div>
  </section>

  <div class="sticky top-0 z-10 -mx-2 mb-8 rounded-xl bg-slate-50/90 px-2 py-3 backdrop-blur supports-[backdrop-filter]:bg-slate-50/75">
    <form method="get" action="/shop" class="flex flex-wrap gap-3 items-end">
      <div class="flex-1 min-w-[200px]">
        <label for="q" class="block text-sm font-semibold text-slate-700">Search</label>
        <input id="q" name="q" type="search" value="{{ filters.q }}" placeholder="Search products"
               class="mt-1 block w-full rounded-lg border border-slate-300 bg-white px-3 py-2 text-slate-900 shadow-sm focus:border-indigo-500 focus:outline-none focus:ring-2 focus:ring-indigo-500/40">
      </div>
      <div>
        <label for="category" class="block text-sm font-semibold text-slate-700">Category</label>
        <select id="category" name="category"
                class="mt-1 block rounded-lg border border-slate-300 bg-white px-3 py-2 text-slate-900 shadow-sm focus:border-indigo-500 focus:outline-none focus:ring-2 focus:ring-indigo-500/40">
          <option value="">All categories</option>
          {% for category in categories %}
          <option value="{{ category.slug }}" {{ 'selected' if filters.category == category.slug }}>{{ category.name }}</option>
          {% endfor %}
        </select>
      </div>
      <button type="submit" class="rounded-lg bg-indigo-600 px-5 py-2 text-sm font-semibold text-white shadow-sm transition hover:bg-indigo-700">Filter</button>
      {% if filters.q or filters.category %}
      <a href="/shop" class="px-4 py-2 text-sm font-medium text-slate-500 hover:text-slate-700 hover:underline">Reset</a>
      {% endif %}
    </form>
  </div>

  {% if not filters.q and not filters.category and featured | length > 0 %}
  <section class="mb-12">
    <h2 class="mb-5 text-2xl font-bold tracking-tight text-slate-900">Featured</h2>
    <div class="grid grid-cols-1 sm:grid-cols-2 lg:grid-cols-3 xl:grid-cols-4 gap-6">
      {% for product in featured %}
      {% include "shop/_product_card.html.j2" %}
      {% endfor %}
    </div>
  </section>
  {% endif %}

  <section id="all-products" class="scroll-mt-24">
    <h2 class="mb-5 text-2xl font-bold tracking-tight text-slate-900">All products</h2>
    {% if products | length == 0 %}
      <p class="rounded-xl border border-dashed border-slate-300 bg-white px-6 py-10 text-center text-slate-500">No products match your filters.</p>
    {% else %}
    <div class="grid grid-cols-1 sm:grid-cols-2 lg:grid-cols-3 xl:grid-cols-4 gap-6">
      {% for product in products %}
      {% include "shop/_product_card.html.j2" %}
      {% endfor %}
    </div>
    {% endif %}
  </section>
</div>
{% endblock %}
"##
    .to_string()
}

pub(super) fn shop_product_card_template() -> String {
    r#"<article class="group rounded-xl border border-slate-200 bg-white overflow-hidden hover:shadow-lg hover:-translate-y-0.5 transition font-sans">
  <a href="/shop/products/{{ product.slug }}" class="relative block aspect-square overflow-hidden bg-slate-100">
    <img src="{{ product.image_url }}" alt="{{ product.name }}" class="h-full w-full object-cover group-hover:scale-105 transition-transform duration-300">
    <div class="absolute top-2 left-2 flex flex-col gap-1.5">
      {% if product.featured %}
      <span class="inline-flex items-center rounded-full bg-indigo-600 px-2.5 py-0.5 text-xs font-semibold text-white shadow-sm">Featured</span>
      {% endif %}
      {% if product.inventory and product.inventory <= 5 %}
      <span class="inline-flex items-center rounded-full bg-rose-500 px-2.5 py-0.5 text-xs font-semibold text-white shadow-sm">Low stock</span>
      {% endif %}
    </div>
  </a>
  <div class="flex flex-1 flex-col p-4">
    <a href="/shop/products/{{ product.slug }}">
      <h3 class="text-base font-semibold text-slate-900 group-hover:text-indigo-600 transition-colors">{{ product.name }}</h3>
    </a>
    <p class="mt-1 text-sm text-slate-500 line-clamp-2 flex-1">{{ product.description }}</p>
    <div class="mt-4 flex items-end justify-between gap-3">
      <span class="text-2xl font-bold text-slate-900">{{ "%.2f" | format(product.price_cents / 100) }} <span class="text-sm font-medium text-slate-400">{{ product.currency }}</span></span>
      <form method="post" action="/shop/cart/add">
        <input type="hidden" name="_csrf" value="{{ csrf_token }}">
        <input type="hidden" name="product_id" value="{{ product.id }}">
        <input type="hidden" name="quantity" value="1">
        <button type="submit" data-turbo-submits-with="Adding..."
                class="rounded-lg bg-indigo-600 px-3.5 py-2 text-sm font-semibold text-white shadow-sm transition hover:bg-indigo-700 disabled:opacity-50">Add to cart</button>
      </form>
    </div>
  </div>
</article>
"#
    .to_string()
}

pub(super) fn shop_product_show_page_template() -> String {
    r#"{% extends "layouts/app.html.j2" %}
{% block title %}{{ product.name }}{% endblock %}
{% block content %}
<div class="font-sans text-slate-900">
  <div class="mb-6">
    <a href="/shop" class="inline-flex items-center text-sm font-medium text-indigo-600 hover:text-indigo-700 hover:underline">&larr; Back to shop</a>
  </div>

  <div class="grid grid-cols-1 md:grid-cols-2 gap-8 rounded-2xl border border-slate-200 bg-white p-6 sm:p-8 shadow-sm">
    <div class="aspect-square overflow-hidden rounded-xl bg-slate-100">
      <img src="{{ product.image_url }}" alt="{{ product.name }}" class="h-full w-full object-cover">
    </div>
    <div class="flex flex-col">
      <h1 class="text-3xl font-extrabold tracking-tight text-slate-900 mb-3">{{ product.name }}</h1>
      <p class="text-2xl font-bold text-slate-900 mb-4">{{ "%.2f" | format(product.price_cents / 100) }} <span class="text-base font-medium text-slate-400">{{ product.currency }}</span></p>
      <div class="mb-5">
        {% if product.inventory > 0 %}
        <span class="inline-flex items-center gap-1.5 rounded-full bg-emerald-50 px-3 py-1 text-sm font-semibold text-emerald-700 ring-1 ring-inset ring-emerald-600/20">
          <span class="h-1.5 w-1.5 rounded-full bg-emerald-500"></span>{{ product.inventory }} in stock
        </span>
        {% else %}
        <span class="inline-flex items-center gap-1.5 rounded-full bg-rose-50 px-3 py-1 text-sm font-semibold text-rose-700 ring-1 ring-inset ring-rose-600/20">
          <span class="h-1.5 w-1.5 rounded-full bg-rose-500"></span>Out of stock
        </span>
        {% endif %}
      </div>
      <p class="text-slate-600 leading-relaxed mb-8">{{ product.description }}</p>
      {% if product.inventory > 0 %}
      <form method="post" action="/shop/cart/add" class="mt-auto flex items-end gap-4">
        <input type="hidden" name="_csrf" value="{{ csrf_token }}">
        <input type="hidden" name="product_id" value="{{ product.id }}">
        <div>
          <label for="quantity" class="block text-sm font-semibold text-slate-700">Quantity</label>
          <input id="quantity" name="quantity" type="number" min="1" max="20" value="1"
                 class="mt-1 block w-24 rounded-lg border border-slate-300 bg-white px-3 py-2 text-slate-900 shadow-sm focus:border-indigo-500 focus:outline-none focus:ring-2 focus:ring-indigo-500/40">
        </div>
        <button type="submit" data-turbo-submits-with="Adding..."
                class="flex-1 rounded-lg bg-indigo-600 px-6 py-2.5 text-sm font-semibold text-white shadow-sm transition hover:bg-indigo-700 disabled:opacity-50">Add to cart</button>
      </form>
      {% endif %}
    </div>
  </div>
</div>
{% endblock %}
"#
    .to_string()
}

pub(super) fn shop_cart_page_template() -> String {
    r#"{% extends "layouts/app.html.j2" %}
{% block title %}Cart{% endblock %}
{% block content %}
<div class="font-sans text-slate-900">
  <div class="flex justify-between items-center mb-8">
    <h1 class="text-3xl font-extrabold tracking-tight text-slate-900">Your cart</h1>
    <a href="/shop" class="text-sm font-medium text-indigo-600 hover:text-indigo-700 hover:underline">Continue shopping</a>
  </div>

  {% if lines | length == 0 %}
    <p class="rounded-xl border border-dashed border-slate-300 bg-white px-6 py-10 text-center text-slate-500">Your cart is empty. <a href="/shop" class="font-medium text-indigo-600 hover:underline">Browse products</a>.</p>
  {% else %}
  <div class="overflow-hidden rounded-2xl border border-slate-200 bg-white shadow-sm">
    <table class="min-w-full divide-y divide-slate-200">
      <thead class="bg-slate-50">
        <tr>
          <th class="px-5 py-3 text-left text-xs font-semibold uppercase tracking-wider text-slate-500">Product</th>
          <th class="px-5 py-3 text-left text-xs font-semibold uppercase tracking-wider text-slate-500">Price</th>
          <th class="px-5 py-3 text-left text-xs font-semibold uppercase tracking-wider text-slate-500">Quantity</th>
          <th class="px-5 py-3 text-right text-xs font-semibold uppercase tracking-wider text-slate-500">Subtotal</th>
        </tr>
      </thead>
      <tbody class="divide-y divide-slate-100">
        {% for line in lines %}
        <tr class="hover:bg-slate-50/60 transition-colors">
          <td class="px-5 py-4">
            <a href="/shop/products/{{ line.product.slug }}" class="font-semibold text-slate-900 hover:text-indigo-600">{{ line.product.name }}</a>
          </td>
          <td class="px-5 py-4 text-sm text-slate-500">{{ "%.2f" | format(line.product.price_cents / 100) }} {{ line.product.currency }}</td>
          <td class="px-5 py-4">
            <form method="post" action="/shop/cart/update" class="flex items-center gap-2">
              <input type="hidden" name="_csrf" value="{{ csrf_token }}">
              <input type="hidden" name="product_id" value="{{ line.product.id }}">
              <input name="quantity" type="number" min="0" max="20" value="{{ line.quantity }}"
                     class="w-20 rounded-lg border border-slate-300 bg-white px-2 py-1 text-slate-900 shadow-sm focus:border-indigo-500 focus:outline-none focus:ring-2 focus:ring-indigo-500/40">
              <button type="submit" class="text-sm font-medium text-indigo-600 hover:text-indigo-700 hover:underline">Update</button>
            </form>
          </td>
          <td class="px-5 py-4 text-right text-sm font-semibold text-slate-900">{{ "%.2f" | format(line.subtotal_cents / 100) }} {{ currency }}</td>
        </tr>
        {% endfor %}
      </tbody>
      <tfoot class="bg-slate-50">
        <tr>
          <td colspan="3" class="px-5 py-4 text-right font-semibold text-slate-700">Total</td>
          <td class="px-5 py-4 text-right text-lg font-bold text-slate-900">{{ "%.2f" | format(total_cents / 100) }} {{ currency }}</td>
        </tr>
      </tfoot>
    </table>
  </div>

  <div class="mt-6 flex justify-end">
    <a href="/shop/checkout" class="rounded-lg bg-indigo-600 px-6 py-2.5 text-sm font-semibold text-white shadow-sm transition hover:bg-indigo-700 hover:-translate-y-0.5">Proceed to checkout</a>
  </div>
  {% endif %}
</div>
{% endblock %}
"#
    .to_string()
}

pub(super) fn shop_checkout_page_template() -> String {
    r#"{% extends "layouts/app.html.j2" %}
{% block title %}Checkout{% endblock %}
{% block content %}
<div class="font-sans text-slate-900">
  <div class="mb-6">
    <a href="/shop/cart" class="inline-flex items-center text-sm font-medium text-indigo-600 hover:text-indigo-700 hover:underline">&larr; Back to cart</a>
  </div>
  <h1 class="text-3xl font-extrabold tracking-tight text-slate-900 mb-8">Checkout</h1>

  {% if lines | length == 0 %}
    <p class="rounded-xl border border-dashed border-slate-300 bg-white px-6 py-10 text-center text-slate-500">Your cart is empty. <a href="/shop" class="font-medium text-indigo-600 hover:underline">Browse products</a>.</p>
  {% else %}
  <div class="grid grid-cols-1 lg:grid-cols-3 gap-8 items-start">
    <form method="post" action="/shop/checkout" class="lg:col-span-2 rounded-2xl border border-slate-200 bg-white p-6 sm:p-8 shadow-sm space-y-6">
      <input type="hidden" name="_csrf" value="{{ csrf_token }}">
      <h2 class="text-lg font-bold text-slate-900">Shipping details</h2>
      <x-field name="customer_name" label="Full name" :value="old.customer_name | default('')" />
      <x-field name="customer_email" label="Email" type="email" :value="old.customer_email | default('')" />
      <x-field name="address_line" label="Address" :value="old.address_line | default('')" />
      <div class="grid grid-cols-2 gap-4">
        <x-field name="city" label="City" :value="old.city | default('')" />
        <x-field name="country" label="Country" :value="old.country | default('')" />
      </div>
      {% if errors.items %}<p class="rounded-lg bg-rose-50 px-3 py-2 text-sm font-medium text-rose-700 ring-1 ring-inset ring-rose-600/20">{{ errors.items }}</p>{% endif %}
      <button type="submit" data-turbo-submits-with="Placing order..."
              class="w-full rounded-lg bg-indigo-600 px-6 py-3 text-sm font-semibold text-white shadow-sm transition hover:bg-indigo-700 disabled:opacity-50">Place order (fake)</button>
    </form>

    <aside class="rounded-2xl border border-slate-200 bg-white p-6 shadow-sm lg:sticky lg:top-6">
      <h2 class="text-lg font-bold text-slate-900 mb-4">Order summary</h2>
      <ul class="space-y-3 text-sm">
        {% for line in lines %}
        <li class="flex justify-between gap-3">
          <span class="text-slate-600">{{ line.quantity }} &times; {{ line.product.name }}</span>
          <span class="font-medium text-slate-900">{{ "%.2f" | format(line.subtotal_cents / 100) }}</span>
        </li>
        {% endfor %}
      </ul>
      <div class="mt-5 border-t border-slate-200 pt-5 flex justify-between items-baseline">
        <span class="font-semibold text-slate-700">Total</span>
        <span class="text-xl font-bold text-slate-900">{{ "%.2f" | format(total_cents / 100) }} {{ currency }}</span>
      </div>
    </aside>
  </div>
  {% endif %}
</div>
{% endblock %}
"#
    .to_string()
}

pub(super) fn shop_order_show_page_template() -> String {
    r#"{% extends "layouts/app.html.j2" %}
{% block title %}Order {{ order.number }}{% endblock %}
{% block content %}
<div class="font-sans text-slate-900">
  <div class="mb-6">
    <a href="/shop" class="inline-flex items-center text-sm font-medium text-indigo-600 hover:text-indigo-700 hover:underline">&larr; Back to shop</a>
  </div>

  <div class="rounded-2xl border border-slate-200 bg-white p-6 sm:p-8 shadow-sm">
    <div class="mb-8">
      <span class="inline-flex items-center gap-1.5 rounded-full bg-emerald-50 px-3 py-1 text-xs font-semibold uppercase tracking-wider text-emerald-700 ring-1 ring-inset ring-emerald-600/20">
        <span class="h-1.5 w-1.5 rounded-full bg-emerald-500"></span>Order confirmed
      </span>
      <h1 class="mt-3 text-3xl font-extrabold tracking-tight text-slate-900">Order {{ order.number }}</h1>
      <p class="mt-2 text-slate-500">Thanks, {{ order.customer_name }}. A confirmation was sent to {{ order.customer_email }}.</p>
    </div>

    <div class="grid grid-cols-1 sm:grid-cols-2 gap-4 mb-8 text-sm">
      <div class="rounded-xl bg-slate-50 p-4 ring-1 ring-inset ring-slate-200/60">
        <p class="font-semibold text-slate-700">Shipping to</p>
        <p class="mt-1 text-slate-600">{{ order.address_line }}</p>
        <p class="text-slate-600">{{ order.city }}, {{ order.country }}</p>
      </div>
      <div class="rounded-xl bg-slate-50 p-4 ring-1 ring-inset ring-slate-200/60">
        <p class="font-semibold text-slate-700">Status</p>
        <p class="mt-1 text-slate-600">{{ order.status }}</p>
      </div>
    </div>

    <div class="overflow-hidden rounded-xl border border-slate-200">
      <table class="min-w-full divide-y divide-slate-200">
        <thead class="bg-slate-50">
          <tr>
            <th class="px-5 py-3 text-left text-xs font-semibold uppercase tracking-wider text-slate-500">Item</th>
            <th class="px-5 py-3 text-left text-xs font-semibold uppercase tracking-wider text-slate-500">Qty</th>
            <th class="px-5 py-3 text-right text-xs font-semibold uppercase tracking-wider text-slate-500">Subtotal</th>
          </tr>
        </thead>
        <tbody class="divide-y divide-slate-100">
          {% for item in items %}
          <tr>
            <td class="px-5 py-4 text-sm font-medium text-slate-900">{{ item.product_name }}</td>
            <td class="px-5 py-4 text-sm text-slate-500">{{ item.quantity }}</td>
            <td class="px-5 py-4 text-right text-sm font-semibold text-slate-900">{{ "%.2f" | format(item.subtotal_cents / 100) }} {{ order.currency }}</td>
          </tr>
          {% endfor %}
        </tbody>
        <tfoot class="bg-slate-50">
          <tr>
            <td colspan="2" class="px-5 py-4 text-right font-semibold text-slate-700">Total</td>
            <td class="px-5 py-4 text-right text-lg font-bold text-slate-900">{{ "%.2f" | format(order.total_cents / 100) }} {{ order.currency }}</td>
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

// ---------------------------------------------------------------------------
// Ecommerce admin (backoffice) templates — Flowbite-admin theme: data tables,
// uppercase tracking-wider headers, divide-y rows, status pills, stat cards.
// ---------------------------------------------------------------------------

pub(super) fn shop_admin_products_index_template() -> String {
    r##"{% extends "layouts/app.html.j2" %}
{% block title %}Products{% endblock %}
{% block content %}
<div class="font-sans">
  <div class="mb-8 flex items-center justify-between">
    <div>
      <h1 class="text-2xl font-bold tracking-tight text-gray-900">Products</h1>
      <p class="mt-1 text-sm text-gray-500">Manage your catalog: pricing, stock and visibility.</p>
    </div>
    <a href="/shop/admin/products/new"
       class="inline-flex items-center gap-2 rounded-lg bg-indigo-600 px-4 py-2 text-sm font-semibold text-white shadow-sm hover:bg-indigo-700 focus:outline-none focus:ring-2 focus:ring-indigo-500 focus:ring-offset-2">
      + New product
    </a>
  </div>

  {% if products | length == 0 %}
    <div class="rounded-xl border border-dashed border-gray-300 bg-white p-12 text-center">
      <p class="text-sm text-gray-500">No products yet. Create the first one.</p>
    </div>
  {% else %}
  <div class="overflow-hidden rounded-xl border border-gray-200 bg-white shadow-sm">
    <table class="min-w-full divide-y divide-gray-200">
      <thead class="bg-gray-50">
        <tr>
          <th class="px-6 py-3 text-left text-xs font-semibold uppercase tracking-wider text-gray-500">Product</th>
          <th class="px-6 py-3 text-left text-xs font-semibold uppercase tracking-wider text-gray-500">Price</th>
          <th class="px-6 py-3 text-left text-xs font-semibold uppercase tracking-wider text-gray-500">Stock</th>
          <th class="px-6 py-3 text-left text-xs font-semibold uppercase tracking-wider text-gray-500">Status</th>
          <th class="px-6 py-3 text-right text-xs font-semibold uppercase tracking-wider text-gray-500">Actions</th>
        </tr>
      </thead>
      <tbody class="divide-y divide-gray-100">
        {% for product in products %}
        <tr class="hover:bg-gray-50">
          <td class="px-6 py-4">
            <div class="flex items-center gap-3">
              {% if product.image_url %}
              <img src="{{ product.image_url }}" alt="" class="h-10 w-10 flex-none rounded-md object-cover ring-1 ring-gray-200">
              {% endif %}
              <div>
                <a href="/shop/admin/products/{{ product.id }}/edit" class="font-medium text-gray-900 hover:text-indigo-600">{{ product.name }}</a>
                <p class="text-xs text-gray-400">{{ product.slug }}</p>
              </div>
            </div>
          </td>
          <td class="px-6 py-4 text-sm font-medium text-gray-900">{{ "%.2f" | format(product.price_cents / 100) }} {{ product.currency }}</td>
          <td class="px-6 py-4 text-sm text-gray-500">{{ product.inventory }}</td>
          <td class="px-6 py-4">
            <div class="flex flex-wrap gap-1.5">
              {% if product.active %}
              <span class="inline-flex items-center gap-1 rounded-full bg-emerald-50 px-2 py-0.5 text-xs font-semibold text-emerald-700 ring-1 ring-inset ring-emerald-600/20">
                <span class="h-1.5 w-1.5 rounded-full bg-emerald-500"></span>Active
              </span>
              {% else %}
              <span class="inline-flex items-center gap-1 rounded-full bg-gray-100 px-2 py-0.5 text-xs font-semibold text-gray-600 ring-1 ring-inset ring-gray-500/20">
                <span class="h-1.5 w-1.5 rounded-full bg-gray-400"></span>Inactive
              </span>
              {% endif %}
              {% if product.featured %}
              <span class="inline-flex items-center rounded-full bg-amber-50 px-2 py-0.5 text-xs font-semibold text-amber-700 ring-1 ring-inset ring-amber-600/20">Featured</span>
              {% endif %}
            </div>
          </td>
          <td class="px-6 py-4 text-right text-sm">
            <a href="/shop/admin/products/{{ product.id }}/edit" class="font-medium text-indigo-600 hover:underline">Edit</a>
            <a href="/shop/admin/products/{{ product.id }}" data-turbo-method="delete"
               data-turbo-confirm="Delete this product?"
               class="ml-4 font-medium text-red-600 hover:underline">Delete</a>
          </td>
        </tr>
        {% endfor %}
      </tbody>
    </table>
  </div>
  {% endif %}
</div>
{% endblock %}
"##
        .to_string()
}

pub(super) fn shop_admin_products_form_template() -> String {
    r##"<form method="post" action="{{ action }}" class="rounded-xl border border-gray-200 bg-white p-8 shadow-sm space-y-6">
  <input type="hidden" name="_csrf" value="{{ csrf_token }}">
  <x-field name="name" label="Name" :value="old.name | default(product.name) | default('')" />
  <x-field name="slug" label="Slug (optional)" :value="old.slug | default(product.slug) | default('')" :required="false" />
  <x-field name="description" label="Description" type="textarea" :value="old.description | default(product.description) | default('')" :required="false" />
  <div class="grid grid-cols-1 gap-6 sm:grid-cols-3">
    <x-field name="price" label="Price" :value="old.price | default((product.price_cents / 100) if product.price_cents is defined else '') | default('')" />
    <x-field name="currency" label="Currency" :value="old.currency | default(product.currency) | default('USD')" />
    <x-field name="inventory" label="Inventory" type="number" :value="old.inventory | default(product.inventory) | default('0')" />
  </div>
  <x-field name="image_url" label="Image URL" :value="old.image_url | default(product.image_url) | default('')" />
  <div>
    <label for="category_id" class="block text-sm font-medium text-gray-700">Category</label>
    {% set current_category = old.category_id | default(product.category_id) | default('') %}
    <select id="category_id" name="category_id"
            class="mt-1 block w-full rounded-md border border-gray-300 px-3 py-2 shadow-sm focus:border-indigo-500 focus:outline-none focus:ring-1 focus:ring-indigo-500">
      <option value="" {{ 'selected' if current_category == '' }}>— No category —</option>
      {% for category in categories %}
      <option value="{{ category.id }}" {{ 'selected' if current_category | string == category.id | string }}>{{ category.name }}</option>
      {% endfor %}
    </select>
    {% if errors.category_id %}<p class="mt-1 text-sm text-red-600">{{ errors.category_id }}</p>{% endif %}
  </div>
  {% set current_featured = old.featured | default('on' if product.featured else '') | default('') %}
  {% set current_active = old.active | default('on' if (product.active is defined and product.active) or product is none else '') | default('') %}
  <div class="space-y-3">
    <label class="flex items-center gap-3">
      <input type="checkbox" name="featured" value="on" {{ 'checked' if current_featured }}
             class="h-4 w-4 rounded border-gray-300 text-indigo-600 focus:ring-indigo-500">
      <span class="text-sm font-medium text-gray-700">Featured</span>
    </label>
    <label class="flex items-center gap-3">
      <input type="checkbox" name="active" value="on" {{ 'checked' if current_active }}
             class="h-4 w-4 rounded border-gray-300 text-indigo-600 focus:ring-indigo-500">
      <span class="text-sm font-medium text-gray-700">Active</span>
    </label>
  </div>
  <div class="flex gap-4 pt-2">
    <button type="submit" data-turbo-submits-with="Saving..."
            class="rounded-lg bg-indigo-600 px-6 py-2 text-sm font-semibold text-white shadow-sm hover:bg-indigo-700 disabled:opacity-50">{{ submit_label }}</button>
    <a href="/shop/admin/products" class="rounded-lg bg-gray-100 px-6 py-2 text-sm font-semibold text-gray-700 hover:bg-gray-200">Cancel</a>
  </div>
</form>
"##
        .to_string()
}

pub(super) fn shop_admin_products_new_template() -> String {
    r##"{% extends "layouts/app.html.j2" %}
{% block title %}New Product{% endblock %}
{% block content %}
<div class="max-w-3xl font-sans">
  <div class="mb-6">
    <a href="/shop/admin/products" class="text-sm font-medium text-indigo-600 hover:underline">&larr; Back to products</a>
  </div>
  <h1 class="mb-8 text-2xl font-bold tracking-tight text-gray-900">New Product</h1>
  {% set action = "/shop/admin/products" %}
  {% set submit_label = "Create Product" %}
  {% include "shop/admin/products/_form.html.j2" %}
</div>
{% endblock %}
"##
        .to_string()
}

pub(super) fn shop_admin_products_edit_template() -> String {
    r##"{% extends "layouts/app.html.j2" %}
{% block title %}Edit Product{% endblock %}
{% block content %}
<div class="max-w-3xl font-sans">
  <div class="mb-6">
    <a href="/shop/admin/products" class="text-sm font-medium text-indigo-600 hover:underline">&larr; Back to products</a>
  </div>
  <h1 class="mb-8 text-2xl font-bold tracking-tight text-gray-900">Edit Product</h1>
  {% set action = "/shop/admin/products/" ~ product.id ~ "/edit" %}
  {% set submit_label = "Save Changes" %}
  {% include "shop/admin/products/_form.html.j2" %}
</div>
{% endblock %}
"##
        .to_string()
}

pub(super) fn shop_admin_orders_index_template() -> String {
    r##"{% extends "layouts/app.html.j2" %}
{% block title %}Orders{% endblock %}
{% block content %}
<div class="font-sans">
  <div class="mb-8">
    <h1 class="text-2xl font-bold tracking-tight text-gray-900">Orders</h1>
    <p class="mt-1 text-sm text-gray-500">Read-only view of customer orders.</p>
  </div>

  <div class="mb-8 grid grid-cols-1 gap-4 sm:grid-cols-2">
    <div class="rounded-xl border border-gray-200 bg-white p-6 shadow-sm">
      <p class="text-xs font-semibold uppercase tracking-wider text-gray-500">Total orders</p>
      <p class="mt-2 text-3xl font-bold text-gray-900">{{ total_orders }}</p>
    </div>
    <div class="rounded-xl border border-gray-200 bg-white p-6 shadow-sm">
      <p class="text-xs font-semibold uppercase tracking-wider text-gray-500">Revenue</p>
      <p class="mt-2 text-3xl font-bold text-gray-900">{{ "%.2f" | format(revenue_cents / 100) }} <span class="text-lg font-semibold text-gray-400">{{ currency }}</span></p>
    </div>
  </div>

  {% if orders | length == 0 %}
    <div class="rounded-xl border border-dashed border-gray-300 bg-white p-12 text-center">
      <p class="text-sm text-gray-500">No orders yet.</p>
    </div>
  {% else %}
  <div class="overflow-hidden rounded-xl border border-gray-200 bg-white shadow-sm">
    <table class="min-w-full divide-y divide-gray-200">
      <thead class="bg-gray-50">
        <tr>
          <th class="px-6 py-3 text-left text-xs font-semibold uppercase tracking-wider text-gray-500">Order</th>
          <th class="px-6 py-3 text-left text-xs font-semibold uppercase tracking-wider text-gray-500">Customer</th>
          <th class="px-6 py-3 text-left text-xs font-semibold uppercase tracking-wider text-gray-500">Status</th>
          <th class="px-6 py-3 text-right text-xs font-semibold uppercase tracking-wider text-gray-500">Total</th>
          <th class="px-6 py-3 text-right text-xs font-semibold uppercase tracking-wider text-gray-500"></th>
        </tr>
      </thead>
      <tbody class="divide-y divide-gray-100">
        {% for order in orders %}
        <tr class="hover:bg-gray-50">
          <td class="px-6 py-4">
            <a href="/shop/admin/orders/{{ order.id }}" class="font-medium text-gray-900 hover:text-indigo-600">{{ order.number }}</a>
          </td>
          <td class="px-6 py-4">
            <p class="text-sm font-medium text-gray-900">{{ order.customer_name }}</p>
            <p class="text-xs text-gray-400">{{ order.customer_email }}</p>
          </td>
          <td class="px-6 py-4">
            {% if order.status == "paid_fake" %}
            <span class="inline-flex items-center gap-1 rounded-full bg-emerald-50 px-2 py-0.5 text-xs font-semibold text-emerald-700 ring-1 ring-inset ring-emerald-600/20">
              <span class="h-1.5 w-1.5 rounded-full bg-emerald-500"></span>Paid
            </span>
            {% else %}
            <span class="inline-flex items-center gap-1 rounded-full bg-gray-100 px-2 py-0.5 text-xs font-semibold text-gray-600 ring-1 ring-inset ring-gray-500/20">{{ order.status }}</span>
            {% endif %}
          </td>
          <td class="px-6 py-4 text-right text-sm font-semibold text-gray-900">{{ "%.2f" | format(order.total_cents / 100) }} {{ order.currency }}</td>
          <td class="px-6 py-4 text-right text-sm">
            <a href="/shop/admin/orders/{{ order.id }}" class="font-medium text-indigo-600 hover:underline">View</a>
          </td>
        </tr>
        {% endfor %}
      </tbody>
    </table>
  </div>
  {% endif %}
</div>
{% endblock %}
"##
        .to_string()
}

pub(super) fn shop_admin_orders_show_template() -> String {
    r##"{% extends "layouts/app.html.j2" %}
{% block title %}Order {{ order.number }}{% endblock %}
{% block content %}
<div class="font-sans">
  <div class="mb-6">
    <a href="/shop/admin/orders" class="text-sm font-medium text-indigo-600 hover:underline">&larr; Back to orders</a>
  </div>

  <div class="rounded-xl border border-gray-200 bg-white p-6 shadow-sm sm:p-8">
    <div class="mb-8 flex items-start justify-between gap-4">
      <div>
        <h1 class="text-2xl font-bold tracking-tight text-gray-900">Order {{ order.number }}</h1>
        <p class="mt-1 text-sm text-gray-500">{{ order.customer_name }} · {{ order.customer_email }}</p>
      </div>
      {% if order.status == "paid_fake" %}
      <span class="inline-flex items-center gap-1.5 rounded-full bg-emerald-50 px-3 py-1 text-xs font-semibold uppercase tracking-wider text-emerald-700 ring-1 ring-inset ring-emerald-600/20">
        <span class="h-1.5 w-1.5 rounded-full bg-emerald-500"></span>Paid
      </span>
      {% else %}
      <span class="inline-flex items-center rounded-full bg-gray-100 px-3 py-1 text-xs font-semibold uppercase tracking-wider text-gray-600 ring-1 ring-inset ring-gray-500/20">{{ order.status }}</span>
      {% endif %}
    </div>

    <div class="mb-8 grid grid-cols-1 gap-4 text-sm sm:grid-cols-2">
      <div class="rounded-lg bg-gray-50 p-4 ring-1 ring-inset ring-gray-200/60">
        <p class="text-xs font-semibold uppercase tracking-wider text-gray-500">Shipping to</p>
        <p class="mt-1 text-gray-700">{{ order.address_line }}</p>
        <p class="text-gray-700">{{ order.city }}, {{ order.country }}</p>
      </div>
      <div class="rounded-lg bg-gray-50 p-4 ring-1 ring-inset ring-gray-200/60">
        <p class="text-xs font-semibold uppercase tracking-wider text-gray-500">Total</p>
        <p class="mt-1 text-2xl font-bold text-gray-900">{{ "%.2f" | format(order.total_cents / 100) }} {{ order.currency }}</p>
      </div>
    </div>

    <div class="overflow-hidden rounded-lg border border-gray-200">
      <table class="min-w-full divide-y divide-gray-200">
        <thead class="bg-gray-50">
          <tr>
            <th class="px-6 py-3 text-left text-xs font-semibold uppercase tracking-wider text-gray-500">Item</th>
            <th class="px-6 py-3 text-left text-xs font-semibold uppercase tracking-wider text-gray-500">Unit price</th>
            <th class="px-6 py-3 text-left text-xs font-semibold uppercase tracking-wider text-gray-500">Qty</th>
            <th class="px-6 py-3 text-right text-xs font-semibold uppercase tracking-wider text-gray-500">Subtotal</th>
          </tr>
        </thead>
        <tbody class="divide-y divide-gray-100">
          {% for item in items %}
          <tr>
            <td class="px-6 py-4 text-sm font-medium text-gray-900">{{ item.product_name }}</td>
            <td class="px-6 py-4 text-sm text-gray-500">{{ "%.2f" | format(item.unit_price_cents / 100) }} {{ order.currency }}</td>
            <td class="px-6 py-4 text-sm text-gray-500">{{ item.quantity }}</td>
            <td class="px-6 py-4 text-right text-sm font-semibold text-gray-900">{{ "%.2f" | format(item.subtotal_cents / 100) }} {{ order.currency }}</td>
          </tr>
          {% endfor %}
        </tbody>
        <tfoot class="bg-gray-50">
          <tr>
            <td colspan="3" class="px-6 py-4 text-right text-sm font-semibold text-gray-700">Total</td>
            <td class="px-6 py-4 text-right text-lg font-bold text-gray-900">{{ "%.2f" | format(order.total_cents / 100) }} {{ order.currency }}</td>
          </tr>
        </tfoot>
      </table>
    </div>
  </div>
</div>
{% endblock %}
"##
        .to_string()
}
