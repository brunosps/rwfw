# Scaffold Template Contract

Generated apps include editable scaffold templates in `.rwfw/templates`.

The generator reads project templates first and falls back to built-in CLI templates only when a local template is missing. Existing generated files are not overwritten.

## Template Files

- `model.rs.tera`: SeaORM entity model.
- `migration.sql.tera`: module-owned SQL migration file.
- `repository.rs.tera`: SeaORM repository.
- `use_case_create.rs.tera`: create input, validation, parsing, and use-case.
- `use_case_update.rs.tera`: update input, validation, parsing, and use-case.
- `use_case_delete.rs.tera`: delete use-case.
- `route_index.rs.tera`: index and create POST routes.
- `route_create.rs.tera`: create page route.
- `route_edit.rs.tera`: edit page and update route.
- `route_item.rs.tera`: show and delete route.
- `page_index.html.j2.tera`: MiniJinja index page template.
- `form.html.j2.tera`: reusable MiniJinja form partial template.
- `page_create.html.j2.tera`: MiniJinja create page template.
- `page_edit.html.j2.tera`: MiniJinja edit page template.
- `page_show.html.j2.tera`: MiniJinja show page template.

## Context

- `module`: module name, for example `blog`.
- `table`: plural table/resource name, for example `products`.
- `name_snake`: singular snake_case name, for example `product`.
- `name_pascal`: singular PascalCase name, for example `Product`.
- `migration_version`: generated migration version.
- `fields`: field list.
- `fields_len`: number of fields, minimum `1`.
- `fields_colspan`: number of fields plus action column, minimum `2`.
- `has_number`, `has_decimal`, `has_uuid`, `has_date`, `has_datetime`: true when any field uses that type family.
- `has_optional`, `has_optional_number`, `has_optional_decimal`, `has_optional_uuid`, `has_optional_date`, `has_optional_datetime`: true when optional fields need helpers.
- `has_searchable`: true when any field is `string` or `text`.

Each field exposes:

- `name`, `kind`, `title`, `column_variant`.
- `rust_type`, `owned_rust_type`, `input_rust_type`, `sql_type`.
- `input_type`, `input_step`, `has_input_step`.
- `optional`, `is_searchable`.
- `is_text`, `is_bool`, `is_number`, `is_decimal`, `is_uuid`, `is_date`, `is_datetime`.

## Default Behavior

- String and text fields participate in the generated search query.
- Index pages receive `items`, `filters`, and `pagination`.
- Optional fields generate nullable SQL columns and `Option<T>` model/repository types.
- Required non-boolean fields are validated as non-empty before parsing.
- Create/update/delete routes redirect with flash confirmations.
