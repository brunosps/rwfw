pub mod app;
pub mod auth;
pub mod config;
pub mod csrf;
pub mod db;
pub mod dev_reload;
pub mod error;
pub mod forms;
pub mod health;
pub mod inertia;
pub mod logging;
pub mod migration;
pub mod module;
pub mod pagination;
pub mod query;
pub mod rate_limit;
pub mod reactive;
pub mod repository;
pub mod security;
pub mod sql;
pub mod use_case;
pub mod validation;
pub mod view;

pub mod prelude {
    pub use crate::app::{AppContext, AppState};
    pub use crate::auth::CurrentUser;
    pub use crate::error::AppError;
    pub use crate::inertia::Inertia;
    pub use crate::migration::Migration;
    pub use crate::module::{Module, ModuleRegistration, NavItem};
    pub use crate::use_case::UseCase;
    pub use crate::view::View;
    pub use axum::extract::{Form, Path, Query, State};
    pub use axum::response::{IntoResponse, Redirect};
    pub use serde_json::json;
}
