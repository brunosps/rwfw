# RWFW Architecture

RWFW is a Rust web framework distributed through a CLI-generated starter app.

## Repository Boundaries

- `crates/rwfw-core`: runtime framework primitives: app state, module registry, auth, CSRF (double-submit), the **View** rendering layer (MiniJinja templates, the `<x-...>` component compiler, and Turbo Streams over SSE), shared props, migrations, query helpers, validation, and repository/use-case traits.
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
- Hotwire frontend: server-rendered MiniJinja templates with Turbo Drive (SPA-like navigation) and Stimulus, vendored Turbo/Stimulus served via an import map, and Tailwind CSS compiled by the standalone binary — **zero npm/Node**.
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

## Rendering / View Layer

RWFW renders HTML on the server with [MiniJinja](https://github.com/mitsuhiko/minijinja) — there is no
client bundler, no React, and no V8 SSR. See the [Frontend guide](FRONTEND.md) for usage and examples.

- **`View` extractor** (`rwfw_core::view::View`): handlers call `v.render("module/page", json!{ ... })`.
  Templates live in `crates/modules/<m>/web/templates/*.html.j2` and extend the app layouts in
  `crates/<app>/web/templates/layouts/`. Shared props (`auth`, `flash`, `errors`, `csrf_token`, `modules`)
  are injected automatically by middleware. Template roots are discovered per crate via `web_root()`.
- **Components** (`<x-...>`): a compiler in `rwfw_core::view::tags` rewrites `<x-name prop=…>…</x-name>` tags
  into MiniJinja includes. A component is a folder `web/templates/components/<name>/index.html.j2` with a
  `{#def … #}` prop declaration, a default slot (`{{ content }}`) + named slots, and an attribute bag (the
  `attrs` filter merges classes). A dev-only `/components` catalog lists them.
- **Interactivity**: Hotwire — vendored Turbo + Stimulus loaded via an import map (no npm). Turbo Drive gives
  SPA-like navigation; HTML forms post normally and re-render at `422` on validation errors; CSRF uses a
  double-submit token (`X-CSRF-Token`, echoed by Turbo from the `<meta>` tag). Real-time updates use Turbo
  Streams broadcast over an SSE endpoint.
- **Styling**: Tailwind CSS compiled by the standalone binary into `web/assets/app.css` (no PostCSS/npm).
- **Static assets**: served from the app's `web/` dir — `/vendor/*` (Turbo/Stimulus) and `/assets/*` (CSS).
- **Testing**: HTTP integration tests (`reqwest` against a spawned server) plus a browser e2e using
  `chromiumoxide` (no Playwright/Node), gated behind `RWFW_E2E=1`.
