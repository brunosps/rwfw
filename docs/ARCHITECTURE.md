# RWFW Architecture

RWFW is a Rust web framework distributed through a CLI-generated starter app.

## Repository Boundaries

- `crates/rwfw-core`: runtime framework primitives: app state, module registry, auth, CSRF, Inertia responses, shared props, SSR, migrations, query helpers, validation, and repository/use-case traits.
- `crates/rwfw-cli`: developer entrypoint. It creates new apps, modules, scaffolds, migrations, dev server orchestration, builds, seeds, and auth helpers.
- `crates/rwfw-app`: internal dogfood application used while developing the framework.
- `crates/modules/*`: internal dogfood modules. `home`, `auth`, `blog`, and generated example modules define the conventions that generated apps receive. `auth` owns local login, RBAC, sessions, and OIDC SSO.
- `.rwfw/templates` in generated apps: project-local scaffold templates. These are copied by `rwfw new app` and are intentionally editable by application teams.
- `scripts/smoke-template.sh`: official local gate for template distribution. It generates an app from this checkout and validates the scaffold path end-to-end.

## Generated App Contract

A generated app is a real starter application, not a temporary demo. It includes:

- Docker and Docker Compose files for dev and production.
- PostgreSQL and pgAdmin dev services.
- An app crate plus generated `home` and auth UI modules.
- A selectable example module: `blog` for editorial CMS flows or `shop` for ecommerce storefront flows.
- Project-local `.rwfw/templates` for scaffold customization.
- Inertia + React frontend, SSR bundle, Vite build, and server-side asset serving.
- A protected-write example flow. The blog template has public posts and authenticated admin post management; the ecommerce template has public storefront and checkout flow.
- Optional OIDC SSO providers configured by `rwfw auth sso add`.

## Current Alpha Shape

For `0.1.0-alpha.1`, RWFW should be treated as a local-framework alpha:

- Generated apps depend on the framework via `--rwfw-path` while the framework is changing quickly.
- Git/tag app generation via `--rwfw-git` and `--rwfw-tag` is the recommended alpha distribution path before crates.io.
- The CLI is the product boundary.
- The template contract is intentionally visible and editable inside generated apps.
- Migrations are module-owned files registered centrally by the module migration registry.
- Auth identities remain local. OIDC providers link to local `auth.users` through provider+subject identity records and RWFW still issues its own session cookie.
