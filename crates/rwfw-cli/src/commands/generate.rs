use anyhow::Context;
use serde::Serialize;
use std::fs;
use std::path::{Path, PathBuf};
use std::time::{SystemTime, UNIX_EPOCH};
use tera::{Context as TeraContext, Tera};

const PROJECT_TEMPLATES_DIR: &str = ".rwfw/templates";

#[derive(Debug, Clone, Serialize)]
struct Field {
    name: String,
    kind: String,
    rust_type: String,
    sql_type: String,
    input_type: String,
    input_rust_type: String,
    owned_rust_type: String,
    title: String,
    column_variant: String,
    optional: bool,
    is_text: bool,
    is_bool: bool,
    is_datetime: bool,
    is_date: bool,
    is_decimal: bool,
    is_number: bool,
    is_uuid: bool,
    is_searchable: bool,
    input_step: String,
    has_input_step: bool,
}

pub async fn run_model(name: &str, module: &str, fields: Option<&str>) -> anyhow::Result<()> {
    let parsed_fields = parse_fields(fields.unwrap_or_default())?;
    generate_model_files(name, module, &parsed_fields)?;
    println!("Generated model `{name}` in module `{module}`.");
    Ok(())
}

pub async fn run_scaffold(name: &str, module: &str, fields: &[String]) -> anyhow::Result<()> {
    let parsed_fields = parse_fields(&fields.join(" "))?;
    let context = generate_model_files(name, module, &parsed_fields)?;
    generate_scaffold_files(&context, &parsed_fields)?;
    println!("Generated scaffold `{name}` in module `{module}`.");
    Ok(())
}

struct ResourceContext {
    module: String,
    module_dir: PathBuf,
    snake: String,
    pascal: String,
    table: String,
}

fn generate_model_files(
    name: &str,
    module: &str,
    fields: &[Field],
) -> anyhow::Result<ResourceContext> {
    let module = to_kebab(module);
    let module_dir = Path::new("crates/modules").join(&module);
    if !module_dir.exists() {
        anyhow::bail!("Module does not exist: {}", module_dir.display());
    }

    let snake = to_snake(name);
    let pascal = to_pascal(&snake);
    let table = pluralize(&snake);
    let migration_version = migration_version();
    let context = ResourceContext {
        module,
        module_dir,
        snake,
        pascal,
        table,
    };

    fs::create_dir_all(context.module_dir.join("src/models"))?;
    fs::create_dir_all(context.module_dir.join("src/migrations"))?;

    add_module_decl(&context.module_dir.join("src/lib.rs"), "models")?;
    upsert_models_mod(
        &context.module_dir.join("src/models/mod.rs"),
        &context.snake,
        &context.pascal,
    )?;
    ensure_module_dependency(
        &context.module_dir.join("Cargo.toml"),
        "sea-orm = { workspace = true }\n",
    )?;
    ensure_module_dependency(
        &context.module_dir.join("Cargo.toml"),
        "serde = { workspace = true }\n",
    )?;
    ensure_module_dependency(
        &context.module_dir.join("Cargo.toml"),
        "chrono = { workspace = true }\n",
    )?;
    if fields.iter().any(|field| field.kind == "uuid") {
        ensure_module_dependency(
            &context.module_dir.join("Cargo.toml"),
            "uuid = { workspace = true }\n",
        )?;
    }

    let model_content = render_resource_template(
        "model.rs.tera",
        &context,
        Some(&migration_version),
        fields,
        model_rs(&context.module, &context.table, &context.pascal, fields),
    )?;
    write_new(
        &context
            .module_dir
            .join(format!("src/models/{}.rs", context.snake)),
        &model_content,
    )?;

    let migration_file = format!("{}_create_{}.sql", migration_version, context.table);
    let migration_content = render_resource_template(
        "migration.sql.tera",
        &context,
        Some(&migration_version),
        fields,
        migration_sql(&context.module, &context.table, fields),
    )?;
    write_new(
        &context
            .module_dir
            .join(format!("src/migrations/{migration_file}")),
        &migration_content,
    )?;
    upsert_migrations_mod(
        &context.module_dir.join("src/migrations/mod.rs"),
        &migration_version,
        &format!("create_{}_table", context.table),
        &migration_file,
    )?;

    Ok(context)
}

fn generate_scaffold_files(context: &ResourceContext, fields: &[Field]) -> anyhow::Result<()> {
    fs::create_dir_all(
        context
            .module_dir
            .join(format!("src/routes/{}", context.table)),
    )?;
    fs::create_dir_all(
        context
            .module_dir
            .join(format!("src/routes/{}/[id]", context.table)),
    )?;
    fs::create_dir_all(
        context
            .module_dir
            .join(format!("web/pages/{}", context.table)),
    )?;
    fs::create_dir_all(context.module_dir.join("src/repositories"))?;
    fs::create_dir_all(context.module_dir.join("src/use_cases"))?;
    add_module_decl(&context.module_dir.join("src/lib.rs"), "repositories")?;
    add_module_decl(&context.module_dir.join("src/lib.rs"), "use_cases")?;
    upsert_mod_decl(
        &context.module_dir.join("src/repositories/mod.rs"),
        &format!("{}_repo", context.snake),
    )?;
    upsert_mod_decl(
        &context.module_dir.join("src/use_cases/mod.rs"),
        &format!("create_{}", context.snake),
    )?;
    upsert_mod_decl(
        &context.module_dir.join("src/use_cases/mod.rs"),
        &format!("update_{}", context.snake),
    )?;
    upsert_mod_decl(
        &context.module_dir.join("src/use_cases/mod.rs"),
        &format!("delete_{}", context.snake),
    )?;
    ensure_module_dependency(
        &context.module_dir.join("Cargo.toml"),
        "anyhow = { workspace = true }\n",
    )?;
    ensure_module_dependency(
        &context.module_dir.join("Cargo.toml"),
        "tracing = { workspace = true }\n",
    )?;
    upsert_resource_permissions(&context.module_dir.join("src/lib.rs"), context)?;

    let repository_content = render_resource_template(
        "repository.rs.tera",
        context,
        None,
        fields,
        render_builtin_resource_template(
            "repository.rs.tera",
            context,
            None,
            fields,
            BUILTIN_REPOSITORY_TEMPLATE,
        )?,
    )?;
    write_new(
        &context
            .module_dir
            .join(format!("src/repositories/{}_repo.rs", context.snake)),
        &repository_content,
    )?;
    let create_use_case_content = render_resource_template(
        "use_case_create.rs.tera",
        context,
        None,
        fields,
        render_builtin_resource_template(
            "use_case_create.rs.tera",
            context,
            None,
            fields,
            BUILTIN_USE_CASE_CREATE_TEMPLATE,
        )?,
    )?;
    write_new(
        &context
            .module_dir
            .join(format!("src/use_cases/create_{}.rs", context.snake)),
        &create_use_case_content,
    )?;
    let update_use_case_content = render_resource_template(
        "use_case_update.rs.tera",
        context,
        None,
        fields,
        render_builtin_resource_template(
            "use_case_update.rs.tera",
            context,
            None,
            fields,
            BUILTIN_USE_CASE_UPDATE_TEMPLATE,
        )?,
    )?;
    write_new(
        &context
            .module_dir
            .join(format!("src/use_cases/update_{}.rs", context.snake)),
        &update_use_case_content,
    )?;
    let delete_use_case_content = render_resource_template(
        "use_case_delete.rs.tera",
        context,
        None,
        fields,
        render_builtin_resource_template(
            "use_case_delete.rs.tera",
            context,
            None,
            fields,
            BUILTIN_USE_CASE_DELETE_TEMPLATE,
        )?,
    )?;
    write_new(
        &context
            .module_dir
            .join(format!("src/use_cases/delete_{}.rs", context.snake)),
        &delete_use_case_content,
    )?;

    let route_content = render_resource_template(
        "route_index.rs.tera",
        context,
        None,
        fields,
        render_builtin_resource_template(
            "route_index.rs.tera",
            context,
            None,
            fields,
            BUILTIN_ROUTE_INDEX_TEMPLATE,
        )?,
    )?;
    write_new(
        &context
            .module_dir
            .join(format!("src/routes/{}/index.rs", context.table)),
        &route_content,
    )?;
    let create_route_content = render_resource_template(
        "route_create.rs.tera",
        context,
        None,
        fields,
        render_builtin_resource_template(
            "route_create.rs.tera",
            context,
            None,
            fields,
            BUILTIN_ROUTE_CREATE_TEMPLATE,
        )?,
    )?;
    write_new(
        &context
            .module_dir
            .join(format!("src/routes/{}/create.rs", context.table)),
        &create_route_content,
    )?;
    let edit_route_content = render_resource_template(
        "route_edit.rs.tera",
        context,
        None,
        fields,
        render_builtin_resource_template(
            "route_edit.rs.tera",
            context,
            None,
            fields,
            BUILTIN_ROUTE_EDIT_TEMPLATE,
        )?,
    )?;
    write_new(
        &context
            .module_dir
            .join(format!("src/routes/{}/[id]/edit.rs", context.table)),
        &edit_route_content,
    )?;
    let item_route_content = render_resource_template(
        "route_item.rs.tera",
        context,
        None,
        fields,
        render_builtin_resource_template(
            "route_item.rs.tera",
            context,
            None,
            fields,
            BUILTIN_ROUTE_ITEM_TEMPLATE,
        )?,
    )?;
    write_new(
        &context
            .module_dir
            .join(format!("src/routes/{}/[id].rs", context.table)),
        &item_route_content,
    )?;
    let page_content = render_resource_template(
        "page_index.tsx.tera",
        context,
        None,
        fields,
        resource_index_page_tsx(&context.module, &context.table, &context.pascal, fields),
    )?;
    write_new(
        &context
            .module_dir
            .join(format!("web/pages/{}/Index.tsx", context.table)),
        &page_content,
    )?;
    let form_content = render_resource_template(
        "form.tsx.tera",
        context,
        None,
        fields,
        render_builtin_resource_template(
            "form.tsx.tera",
            context,
            None,
            fields,
            BUILTIN_FORM_TEMPLATE,
        )?,
    )?;
    write_new(
        &context
            .module_dir
            .join(format!("web/pages/{}/Form.tsx", context.table)),
        &form_content,
    )?;
    let create_page_content = render_resource_template(
        "page_create.tsx.tera",
        context,
        None,
        fields,
        render_builtin_resource_template(
            "page_create.tsx.tera",
            context,
            None,
            fields,
            BUILTIN_PAGE_CREATE_TEMPLATE,
        )?,
    )?;
    write_new(
        &context
            .module_dir
            .join(format!("web/pages/{}/Create.tsx", context.table)),
        &create_page_content,
    )?;
    let edit_page_content = render_resource_template(
        "page_edit.tsx.tera",
        context,
        None,
        fields,
        render_builtin_resource_template(
            "page_edit.tsx.tera",
            context,
            None,
            fields,
            BUILTIN_PAGE_EDIT_TEMPLATE,
        )?,
    )?;
    write_new(
        &context
            .module_dir
            .join(format!("web/pages/{}/Edit.tsx", context.table)),
        &edit_page_content,
    )?;
    let show_page_content = render_resource_template(
        "page_show.tsx.tera",
        context,
        None,
        fields,
        render_builtin_resource_template(
            "page_show.tsx.tera",
            context,
            None,
            fields,
            BUILTIN_PAGE_SHOW_TEMPLATE,
        )?,
    )?;
    write_new(
        &context
            .module_dir
            .join(format!("web/pages/{}/Show.tsx", context.table)),
        &show_page_content,
    )?;

    Ok(())
}

fn render_resource_template(
    template_name: &str,
    resource: &ResourceContext,
    migration_version: Option<&str>,
    fields: &[Field],
    fallback: String,
) -> anyhow::Result<String> {
    let template_path = Path::new(PROJECT_TEMPLATES_DIR).join(template_name);
    if !template_path.exists() {
        return Ok(fallback);
    }

    let template = fs::read_to_string(&template_path)
        .with_context(|| format!("reading template {}", template_path.display()))?;
    render_template_source(
        template_name,
        &template,
        &template_path.display().to_string(),
        resource,
        migration_version,
        fields,
    )
}

fn render_builtin_resource_template(
    template_name: &str,
    resource: &ResourceContext,
    migration_version: Option<&str>,
    fields: &[Field],
    template: &str,
) -> anyhow::Result<String> {
    render_template_source(
        template_name,
        template,
        &format!("built-in template {template_name}"),
        resource,
        migration_version,
        fields,
    )
}

fn render_template_source(
    template_name: &str,
    template: &str,
    source: &str,
    resource: &ResourceContext,
    migration_version: Option<&str>,
    fields: &[Field],
) -> anyhow::Result<String> {
    let mut tera = Tera::default();
    tera.add_raw_template(template_name, &template)
        .with_context(|| format!("parsing template {source}"))?;

    let mut context = TeraContext::new();
    context.insert("module", &resource.module);
    context.insert("table", &resource.table);
    context.insert("name_snake", &resource.snake);
    context.insert("name_pascal", &resource.pascal);
    context.insert("migration_version", &migration_version.unwrap_or_default());
    context.insert("fields", fields);
    context.insert("fields_len", &fields.len().max(1));
    context.insert("fields_colspan", &(fields.len().max(1) + 1));
    context.insert("has_number", &fields.iter().any(|field| field.is_number));
    context.insert("has_uuid", &fields.iter().any(|field| field.is_uuid));
    context.insert(
        "has_datetime",
        &fields.iter().any(|field| field.is_datetime),
    );
    context.insert("has_date", &fields.iter().any(|field| field.is_date));
    context.insert("has_decimal", &fields.iter().any(|field| field.is_decimal));
    context.insert("has_optional", &fields.iter().any(|field| field.optional));
    context.insert(
        "has_optional_number",
        &fields.iter().any(|field| field.optional && field.is_number),
    );
    context.insert(
        "has_optional_decimal",
        &fields
            .iter()
            .any(|field| field.optional && field.is_decimal),
    );
    context.insert(
        "has_optional_uuid",
        &fields.iter().any(|field| field.optional && field.is_uuid),
    );
    context.insert(
        "has_optional_date",
        &fields.iter().any(|field| field.optional && field.is_date),
    );
    context.insert(
        "has_optional_datetime",
        &fields
            .iter()
            .any(|field| field.optional && field.is_datetime),
    );
    context.insert(
        "has_searchable",
        &fields.iter().any(|field| field.is_searchable),
    );

    tera.render(template_name, &context)
        .with_context(|| format!("rendering template {source}"))
}

const BUILTIN_REPOSITORY_TEMPLATE: &str = r#"use crate::models::{{ name_snake }}::{self, ActiveModel, Column, Entity as {{ name_pascal }}Entity};
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
"#;

const BUILTIN_USE_CASE_CREATE_TEMPLATE: &str = r#"use crate::models::{{ name_snake }};
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
{% endif %}"#;

const BUILTIN_USE_CASE_UPDATE_TEMPLATE: &str = r#"use crate::models::{{ name_snake }};
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
{% endif %}"#;

const BUILTIN_USE_CASE_DELETE_TEMPLATE: &str = r#"use crate::repositories::{{ name_snake }}_repo::{{ name_pascal }}Repository;
use rwfw_core::error::AppError;

pub struct Delete{{ name_pascal }}UseCase;

impl Delete{{ name_pascal }}UseCase {
    pub async fn execute(&self, repo: &{{ name_pascal }}Repository, id: i32) -> Result<(), AppError> {
        repo.delete(id).await.map_err(AppError::Internal)?;
        tracing::info!({{ name_snake }}_id = %id, "{{ name_pascal }} deleted");
        Ok(())
    }
}
"#;

const BUILTIN_ROUTE_INDEX_TEMPLATE: &str = r#"use crate::repositories::{{ name_snake }}_repo::{{ name_pascal }}Repository;
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
"#;

const BUILTIN_ROUTE_CREATE_TEMPLATE: &str = r#"use axum::response::IntoResponse;
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
"#;

const BUILTIN_ROUTE_EDIT_TEMPLATE: &str = r#"use crate::repositories::{{ name_snake }}_repo::{{ name_pascal }}Repository;
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
"#;

const BUILTIN_ROUTE_ITEM_TEMPLATE: &str = r#"use crate::repositories::{{ name_snake }}_repo::{{ name_pascal }}Repository;
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
"#;

const BUILTIN_FORM_TEMPLATE: &str = r#"import type { FormEvent } from 'react'
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
"#;

const BUILTIN_PAGE_CREATE_TEMPLATE: &str = r#"import type { FormEvent } from 'react'
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
"#;

const BUILTIN_PAGE_EDIT_TEMPLATE: &str = r#"import type { FormEvent } from 'react'
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
"#;

const BUILTIN_PAGE_SHOW_TEMPLATE: &str = r#"import { Link, router } from '@inertiajs/react'
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
"#;

fn upsert_resource_permissions(lib_path: &Path, context: &ResourceContext) -> anyhow::Result<()> {
    let mut content = fs::read_to_string(lib_path)?;
    let permissions = [
        ("view", format!("View {}", context.table)),
        ("create", format!("Create {}", context.table)),
        ("update", format!("Update {}", context.table)),
        ("delete", format!("Delete {}", context.table)),
    ];

    let mut entries = String::new();
    for (action, description) in permissions {
        let name = format!("{}.{}.{}", context.module, context.table, action);
        if content.contains(&name) {
            continue;
        }
        entries.push_str(&format!(
            "            rwfw_core::auth::Permission::new(\"{name}\", \"{description}\"),\n"
        ));
    }

    if entries.is_empty() {
        return Ok(());
    }

    if !content.contains("fn permissions(&self)") {
        let marker = "    fn nav_items(&self)";
        let index = content
            .find(marker)
            .with_context(|| format!("nav_items marker not found in {}", lib_path.display()))?;
        let block = "    fn permissions(&self) -> Vec<rwfw_core::auth::Permission> {\n        vec![\n        ]\n    }\n\n";
        content.insert_str(index, block);
    }

    let fn_index = content
        .find("fn permissions(&self)")
        .with_context(|| format!("permissions function not found in {}", lib_path.display()))?;
    upsert_entries_into_permissions_vec(&mut content, fn_index, &entries)
        .with_context(|| format!("updating permissions vec in {}", lib_path.display()))?;
    fs::write(lib_path, content)?;
    Ok(())
}

fn upsert_entries_into_permissions_vec(
    content: &mut String,
    fn_index: usize,
    entries: &str,
) -> anyhow::Result<()> {
    let vec_relative_index = content[fn_index..]
        .find("vec![")
        .ok_or_else(|| anyhow::anyhow!("permissions vec![] not found"))?;
    let vec_start = fn_index + vec_relative_index;
    let inner_start = vec_start + "vec![".len();
    let vec_end = find_matching_bracket(content, inner_start - 1)
        .ok_or_else(|| anyhow::anyhow!("permissions vec![] closing bracket not found"))?;
    let existing = content[inner_start..vec_end].trim();

    let mut replacement = String::from("vec![\n");
    if !existing.is_empty() {
        if existing.contains('\n') {
            replacement.push_str(existing.trim_end());
            replacement.push('\n');
        } else {
            replacement.push_str("            ");
            replacement.push_str(existing.trim_end_matches(','));
            replacement.push_str(",\n");
        }
    }
    replacement.push_str(entries);
    replacement.push_str("        ]");

    content.replace_range(vec_start..=vec_end, &replacement);
    Ok(())
}

fn find_matching_bracket(content: &str, open_index: usize) -> Option<usize> {
    let bytes = content.as_bytes();
    let mut depth = 0usize;

    for (index, byte) in bytes.iter().enumerate().skip(open_index) {
        match byte {
            b'[' => depth += 1,
            b']' => {
                depth = depth.checked_sub(1)?;
                if depth == 0 {
                    return Some(index);
                }
            }
            _ => {}
        }
    }

    None
}

fn model_rs(module: &str, table: &str, pascal: &str, fields: &[Field]) -> String {
    let mut field_lines = String::new();
    for field in fields {
        if field.kind == "text" {
            field_lines.push_str("    #[sea_orm(column_type = \"Text\")]\n");
        }
        field_lines.push_str(&format!("    pub {}: {},\n", field.name, field.rust_type));
    }

    format!(
        r#"use sea_orm::entity::prelude::*;
use serde::{{Deserialize, Serialize}};

#[derive(Clone, Debug, PartialEq, DeriveEntityModel, Serialize, Deserialize)]
#[sea_orm(table_name = "{table}", schema_name = "{module}")]
pub struct Model {{
    #[sea_orm(primary_key)]
    pub id: i32,
{field_lines}    pub created_at: DateTimeWithTimeZone,
    pub updated_at: DateTimeWithTimeZone,
}}

#[derive(Copy, Clone, Debug, EnumIter, DeriveRelation)]
pub enum Relation {{}}

impl ActiveModelBehavior for ActiveModel {{}}

pub type {pascal} = Model;
"#
    )
}

fn migration_sql(module: &str, table: &str, fields: &[Field]) -> String {
    let mut sql = format!(
        "CREATE SCHEMA IF NOT EXISTS {module};\n\nCREATE TABLE IF NOT EXISTS {module}.{table} (\n    id SERIAL PRIMARY KEY,\n"
    );
    for field in fields {
        sql.push_str(&format!("    {} {},\n", field.name, field.sql_type));
    }
    sql.push_str(
        "    created_at TIMESTAMPTZ NOT NULL DEFAULT NOW(),\n    updated_at TIMESTAMPTZ NOT NULL DEFAULT NOW()\n);\n",
    );
    sql
}

fn resource_index_page_tsx(module: &str, table: &str, pascal: &str, fields: &[Field]) -> String {
    let mut columns = String::new();
    let mut rows = String::new();
    for field in fields {
        columns.push_str(&format!(
            "              <th className=\"px-4 py-2 text-left\">{}</th>\n",
            to_title(&field.name)
        ));
        rows.push_str(&format!(
            "                  <td className=\"px-4 py-2\">{{String(item['{}'] ?? '')}}</td>\n",
            field.name
        ));
    }

    format!(
        r#"import type {{ FormEvent }} from 'react'
import {{ Link, router }} from '@inertiajs/react'
import AppLayout from '@app/layouts/AppLayout'

interface Pagination {{
  page: number
  per_page: number
  total: number
  total_pages: number
}}

interface Props {{
  items: Array<Record<string, unknown>>
  pagination: Pagination
  filters?: {{
    q?: string
  }}
}}

function pageHref(page: number, q?: string) {{
  const params = new URLSearchParams()
  if (q) params.set('q', q)
  if (page > 1) params.set('page', String(page))
  const query = params.toString()
  return query ? `/{module}/{table}?${{query}}` : '/{module}/{table}'
}}

export default function {pascal}Index({{ items, pagination, filters = {{}} }}: Props) {{
  const q = filters.q ?? ''
  const canGoBack = pagination.page > 1
  const canGoForward = pagination.page < pagination.total_pages

  function submitSearch(event: FormEvent<HTMLFormElement>) {{
    event.preventDefault()
    const form = new FormData(event.currentTarget)
    const q = String(form.get('q') ?? '').trim()
    const data = q ? {{ q }} : {{}}
    router.get('/{module}/{table}', data, {{ preserveState: true, replace: true }})
  }}

  return (
    <AppLayout>
      <div className="max-w-5xl">
        <div className="mb-8 flex flex-col gap-4 sm:flex-row sm:items-center sm:justify-between">
          <div>
            <p className="text-sm font-semibold uppercase tracking-[0.2em] text-slate-500">{table}</p>
            <h1 className="mt-2 text-3xl font-bold text-gray-900">{pascal}</h1>
          </div>
          <Link
            href="/{module}/{table}/create"
            className="rounded-xl bg-slate-950 px-4 py-2 text-sm font-semibold text-white hover:bg-slate-800"
          >
            New {pascal}
          </Link>
        </div>

        <form onSubmit={{submitSearch}} className="mb-4 flex flex-col gap-3 rounded-2xl border border-slate-200 bg-white p-4 shadow-sm sm:flex-row">
          <input
            name="q"
            type="search"
            defaultValue={{q}}
            placeholder="Search {table}..."
            className="min-w-0 flex-1 rounded-xl border border-slate-300 px-3 py-2 text-sm outline-none focus:border-slate-950"
          />
          <div className="flex gap-2">
            <button
              type="submit"
              className="rounded-xl bg-slate-950 px-4 py-2 text-sm font-semibold text-white hover:bg-slate-800"
            >
              Search
            </button>
            {{q ? (
              <Link
                href="/{module}/{table}"
                className="rounded-xl border border-slate-300 px-4 py-2 text-sm font-semibold text-slate-700 hover:bg-slate-50"
              >
                Clear
              </Link>
            ) : null}}
          </div>
        </form>

        <div className="overflow-hidden rounded-lg border bg-white">
          <table className="w-full text-sm">
            <thead className="bg-gray-50">
              <tr>
{columns}                <th className="px-4 py-2 text-right">Actions</th>
              </tr>
            </thead>
            <tbody>
              {{items.length === 0 && (
                <tr>
                  <td className="px-4 py-6 text-gray-500" colSpan={{{fields_colspan}}}>
                    No records yet.
                  </td>
                </tr>
              )}}
              {{items.map((item, index) => (
                <tr key={{index}} className="border-t">
{rows}                  <td className="px-4 py-2 text-right">
                    {{item['id'] ? (
                      <div className="flex justify-end gap-3">
                      <Link
                        href={{`/{module}/{table}/${{String(item['id'])}}`}}
                        className="text-sm font-semibold text-slate-700 hover:text-slate-950"
                      >
                        View
                      </Link>
                      <Link
                        href={{`/{module}/{table}/${{String(item['id'])}}/edit`}}
                        className="text-sm font-semibold text-slate-700 hover:text-slate-950"
                      >
                        Edit
                      </Link>
                      <button
                        type="button"
                        onClick={{() => {{
                          if (window.confirm('Delete this {pascal}?')) {{
                            router.delete('/{module}/{table}/' + String(item['id']))
                          }}
                        }}}}
                        className="text-sm font-semibold text-red-600 hover:text-red-700"
                      >
                        Delete
                      </button>
                      </div>
                    ) : null}}
                  </td>
                </tr>
              ))}}
            </tbody>
          </table>
        </div>

        <div className="mt-4 flex flex-col gap-3 text-sm text-slate-600 sm:flex-row sm:items-center sm:justify-between">
          <p>
            Showing {{items.length}} of {{pagination.total}} records
          </p>
          <div className="flex items-center gap-2">
            {{canGoBack ? (
              <Link
                href={{pageHref(pagination.page - 1, q)}}
                className="rounded-xl border border-slate-300 px-3 py-2 font-semibold text-slate-700 hover:bg-slate-50"
              >
                Previous
              </Link>
            ) : (
              <span className="rounded-xl border border-slate-200 px-3 py-2 font-semibold text-slate-300">
                Previous
              </span>
            )}}
            <span>
              Page {{pagination.page}} of {{pagination.total_pages}}
            </span>
            {{canGoForward ? (
              <Link
                href={{pageHref(pagination.page + 1, q)}}
                className="rounded-xl border border-slate-300 px-3 py-2 font-semibold text-slate-700 hover:bg-slate-50"
              >
                Next
              </Link>
            ) : (
              <span className="rounded-xl border border-slate-200 px-3 py-2 font-semibold text-slate-300">
                Next
              </span>
            )}}
          </div>
        </div>
      </div>
    </AppLayout>
  )
}}
"#,
        fields_colspan = fields.len().max(1) + 1,
        module = module,
        table = table,
        rows = rows,
    )
}

fn upsert_models_mod(path: &Path, snake: &str, pascal: &str) -> anyhow::Result<()> {
    let mut content = if path.exists() {
        fs::read_to_string(path)?
    } else {
        String::new()
    };
    let module_line = format!("pub mod {snake};\n");
    let export_line = format!("pub use {snake}::{pascal};\n");
    if !content.contains(&module_line) {
        content.push_str(&module_line);
    }
    if !content.contains(&export_line) {
        content.push_str(&export_line);
    }
    fs::write(path, content)?;
    Ok(())
}

fn upsert_migrations_mod(path: &Path, version: &str, name: &str, file: &str) -> anyhow::Result<()> {
    let mut content = if path.exists() {
        fs::read_to_string(path)?
    } else {
        "use rwfw_core::migration::Migration;\n\npub fn migrations() -> Vec<Migration> {\n    vec![\n    ]\n}\n".to_string()
    };

    if content.contains(version) {
        return Ok(());
    }

    let entry =
        format!("        Migration::new(\"{version}\", \"{name}\", include_str!(\"{file}\")),\n");
    if let Some(index) = content.find("    ]") {
        content.insert_str(index, &entry);
    } else if let Some(index) = content.rfind("]\n}") {
        content.insert_str(index, &format!(",\n{entry}    "));
    } else {
        anyhow::bail!("migration marker not found in {}", path.display());
    }
    fs::write(path, content)?;
    Ok(())
}

fn add_module_decl(lib_path: &Path, module: &str) -> anyhow::Result<()> {
    let mut content = fs::read_to_string(lib_path)?;
    let line = format!("pub mod {module};\n");
    if !content.contains(&line) {
        content.insert_str(0, &line);
    }
    fs::write(lib_path, content)?;
    Ok(())
}

fn upsert_mod_decl(path: &Path, module: &str) -> anyhow::Result<()> {
    let mut content = if path.exists() {
        fs::read_to_string(path)?
    } else {
        String::new()
    };
    let line = format!("pub mod {module};\n");
    if !content.contains(&line) {
        content.push_str(&line);
    }
    fs::write(path, content)?;
    Ok(())
}

fn ensure_module_dependency(cargo_toml: &Path, line: &str) -> anyhow::Result<()> {
    let mut content = fs::read_to_string(cargo_toml)?;
    if content.contains(line) {
        return Ok(());
    }
    if !content.ends_with('\n') {
        content.push('\n');
    }
    content.push_str(line);
    fs::write(cargo_toml, content)?;
    Ok(())
}

fn write_new(path: &Path, content: &str) -> anyhow::Result<()> {
    if path.exists() {
        anyhow::bail!("File already exists: {}", path.display());
    }
    fs::write(path, content)?;
    Ok(())
}

fn parse_fields(value: &str) -> anyhow::Result<Vec<Field>> {
    value
        .split(|c: char| c == ',' || c.is_whitespace())
        .filter(|part| !part.is_empty())
        .map(|part| {
            let (raw_name, raw_kind) = part
                .split_once(':')
                .with_context(|| format!("field must use name:type syntax: {part}"))?;
            let optional = raw_name.ends_with('?') || raw_kind.ends_with('?');
            let raw_name = raw_name.trim_end_matches('?');
            let raw_kind = raw_kind.trim_end_matches('?');
            let name = to_snake(raw_name);
            let kind = normalize_kind(raw_kind)?;
            let is_bool = matches!(kind.as_str(), "bool" | "boolean");
            let is_datetime = matches!(kind.as_str(), "datetime" | "timestamp");
            let is_date = kind == "date";
            let is_decimal = matches!(kind.as_str(), "decimal" | "float");
            let is_number = matches!(kind.as_str(), "int" | "integer");
            let is_uuid = kind == "uuid";
            let is_searchable = matches!(kind.as_str(), "string" | "text");
            let input_step = input_step(&kind).to_string();
            let has_input_step = !input_step.is_empty();
            Ok(Field {
                rust_type: rust_type(&kind, optional),
                sql_type: sql_type(&kind, optional),
                input_type: input_type(&kind).to_string(),
                input_rust_type: input_rust_type(&kind, optional),
                owned_rust_type: owned_rust_type(&kind, optional),
                title: to_title(&name),
                column_variant: to_pascal(&name),
                optional,
                is_text: kind == "text",
                is_bool,
                is_datetime,
                is_date,
                is_decimal,
                is_number,
                is_uuid,
                is_searchable,
                input_step,
                has_input_step,
                name,
                kind,
            })
        })
        .collect()
}

fn normalize_kind(kind: &str) -> anyhow::Result<String> {
    let kind = kind.trim();
    match kind {
        "string" | "text" | "int" | "integer" | "decimal" | "float" | "bool" | "boolean"
        | "uuid" | "date" | "datetime" | "timestamp" => Ok(kind.to_string()),
        _ => anyhow::bail!("Unsupported field type: {kind}"),
    }
}

fn rust_type(kind: &str, optional: bool) -> String {
    let base = match kind {
        "string" | "text" => "String",
        "int" | "integer" => "i32",
        "decimal" | "float" => "f64",
        "bool" | "boolean" => "bool",
        "uuid" => "Uuid",
        "date" => "Date",
        "datetime" | "timestamp" => "DateTimeWithTimeZone",
        _ => "String",
    };
    if optional {
        format!("Option<{base}>")
    } else {
        base.to_string()
    }
}

fn sql_type(kind: &str, optional: bool) -> String {
    let base = match kind {
        "string" => "VARCHAR(255)",
        "text" => "TEXT",
        "int" | "integer" => "INTEGER",
        "decimal" | "float" => "DOUBLE PRECISION",
        "bool" | "boolean" => "BOOLEAN",
        "uuid" => "UUID",
        "date" => "DATE",
        "datetime" | "timestamp" => "TIMESTAMPTZ",
        _ => "VARCHAR(255)",
    };
    if optional {
        base.to_string()
    } else if matches!(kind, "bool" | "boolean") {
        format!("{base} NOT NULL DEFAULT FALSE")
    } else {
        format!("{base} NOT NULL")
    }
}

fn input_type(kind: &str) -> &'static str {
    match kind {
        "int" | "integer" | "decimal" | "float" => "number",
        "date" => "date",
        "datetime" | "timestamp" => "datetime-local",
        "uuid" | "string" | "text" => "text",
        "bool" | "boolean" => "checkbox",
        _ => "text",
    }
}

fn input_rust_type(kind: &str, _optional: bool) -> String {
    match kind {
        "bool" | "boolean" => "bool",
        _ => "String",
    }
    .to_string()
}

fn input_step(kind: &str) -> &'static str {
    match kind {
        "decimal" | "float" => "any",
        _ => "",
    }
}

fn owned_rust_type(kind: &str, optional: bool) -> String {
    let base = match kind {
        "string" | "text" => "String",
        "int" | "integer" => "i32",
        "decimal" | "float" => "f64",
        "bool" | "boolean" => "bool",
        "uuid" => "uuid::Uuid",
        "date" => "chrono::NaiveDate",
        "datetime" | "timestamp" => "chrono::DateTime<chrono::FixedOffset>",
        _ => "String",
    };
    if optional {
        format!("Option<{base}>")
    } else {
        base.to_string()
    }
}

fn migration_version() -> String {
    let seconds = SystemTime::now()
        .duration_since(UNIX_EPOCH)
        .map(|duration| duration.as_secs())
        .unwrap_or(0);
    format!("{seconds}")
}

fn pluralize(value: &str) -> String {
    if value.ends_with('s') {
        value.to_string()
    } else {
        format!("{value}s")
    }
}

fn to_snake(value: &str) -> String {
    value
        .trim()
        .chars()
        .enumerate()
        .fold(String::new(), |mut acc, (index, c)| {
            if c.is_ascii_uppercase() {
                if index > 0 {
                    acc.push('_');
                }
                acc.push(c.to_ascii_lowercase());
            } else if c.is_ascii_alphanumeric() {
                acc.push(c.to_ascii_lowercase());
            } else if !acc.ends_with('_') {
                acc.push('_');
            }
            acc
        })
        .trim_matches('_')
        .to_string()
}

fn to_kebab(value: &str) -> String {
    to_snake(value).replace('_', "-")
}

fn to_pascal(value: &str) -> String {
    value
        .split('_')
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
        .split('_')
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
