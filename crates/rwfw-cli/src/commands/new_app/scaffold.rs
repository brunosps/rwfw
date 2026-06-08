//! Scaffold (.tera) templates emitted into .rwfw/templates for `rwfw generate`.

pub(super) fn scaffold_model_template() -> String {
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

pub(super) fn scaffold_migration_template() -> String {
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

pub(super) fn scaffold_repository_template() -> String {
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

pub(super) fn scaffold_use_case_create_template() -> String {
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

pub(super) fn scaffold_use_case_update_template() -> String {
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

pub(super) fn scaffold_use_case_delete_template() -> String {
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

pub(super) fn scaffold_route_index_template() -> String {
    crate::commands::generate::BUILTIN_ROUTE_INDEX_TEMPLATE.to_string()
}

pub(super) fn scaffold_route_create_template() -> String {
    crate::commands::generate::BUILTIN_ROUTE_CREATE_TEMPLATE.to_string()
}

pub(super) fn scaffold_route_edit_template() -> String {
    crate::commands::generate::BUILTIN_ROUTE_EDIT_TEMPLATE.to_string()
}

pub(super) fn scaffold_route_item_template() -> String {
    crate::commands::generate::BUILTIN_ROUTE_ITEM_TEMPLATE.to_string()
}

pub(super) fn scaffold_page_index_template() -> String {
    crate::commands::generate::BUILTIN_PAGE_INDEX_TEMPLATE.to_string()
}

pub(super) fn scaffold_form_template() -> String {
    crate::commands::generate::BUILTIN_FORM_TEMPLATE.to_string()
}

pub(super) fn scaffold_page_create_template() -> String {
    crate::commands::generate::BUILTIN_PAGE_CREATE_TEMPLATE.to_string()
}

pub(super) fn scaffold_page_edit_template() -> String {
    crate::commands::generate::BUILTIN_PAGE_EDIT_TEMPLATE.to_string()
}

pub(super) fn scaffold_page_show_template() -> String {
    crate::commands::generate::BUILTIN_PAGE_SHOW_TEMPLATE.to_string()
}

// ---------------------------------------------------------------------------
// Ecommerce example — Hotwire `View` routes (replace the Inertia handlers) and
// `.html.j2` MiniJinja pages (replace the `.tsx` pages). All server-rendered.
// ---------------------------------------------------------------------------

