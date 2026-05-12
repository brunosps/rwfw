# CLI Contract

Version target: `0.1.0-alpha.1`.

## App Lifecycle

```bash
rwfw new app demo --example blog --rwfw-path /path/to/rwfw
rwfw dev
rwfw build
rwfw migrate
rwfw seed admin --email admin@example.com --password rwfw-admin-123
rwfw auth sso add keycloak --provider keycloak --issuer-url http://localhost:8081/realms/rwfw --client-id-env KEYCLOAK_CLIENT_ID --client-secret-env KEYCLOAK_CLIENT_SECRET
```

`rwfw new app` and `rwfw new example` support these app templates:

- `--example blog`: editorial/blog CMS with public posts and protected admin post management.
- `--example ecommerce`: storefront/cart/checkout example.

Dependency source flags are mutually exclusive:

- `--rwfw-path /path/to/rwfw`: local framework checkout, preferred for framework development.
- `--rwfw-git https://github.com/<org>/rwfw --rwfw-tag v0.1.0-alpha.1`: Git/tag distribution, preferred for external alpha use.
- `--rwfw-version 0.1.0-alpha.1`: published crate version, for the future crates.io path.

## Module and Scaffold

```bash
rwfw new module billing
rwfw generate model Product --module billing --fields "name:string price:decimal"
rwfw generate scaffold Product --module billing name:string summary:text? price:decimal active:bool
```

## Auth and SSO

```bash
rwfw auth create-admin --email admin@example.com --password rwfw-admin-123
rwfw auth sso add keycloak \
  --provider keycloak \
  --issuer-url http://localhost:8081/realms/rwfw \
  --client-id-env KEYCLOAK_CLIENT_ID \
  --client-secret-env KEYCLOAK_CLIENT_SECRET
```

`auth sso add` writes OIDC provider config into the generated app, keeps secrets in env vars, and supports `oidc`, `keycloak`, and `azure-b2c` provider presets.

## Field Syntax

- Required field: `name:string`.
- Optional field: `summary:text?` or `summary?:text`.
- Supported types: `string`, `text`, `int`, `integer`, `decimal`, `float`, `bool`, `boolean`, `uuid`, `date`, `datetime`, `timestamp`.

## Scaffold Output

`rwfw generate scaffold` creates:

- SeaORM model.
- One module-owned SQL migration file.
- Repository with `find_by_id`, searchable/paginated `find_all`, `create`, `update`, and `delete`.
- Create/update/delete use-cases with validation and type parsing.
- Protected Axum routes.
- React/Inertia index, form, create, edit, and show pages.
- Flash confirmations after create, update, and delete.

## Non-Goals For This Alpha

- Publishing to crates.io as the default installation path.
- Supporting every database backend.
- Overwriting existing generated files.
- Fully abstracting frontend choices.
