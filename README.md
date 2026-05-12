# RWFW

RWFW is being shaped as a Rust web framework distributed primarily through a CLI. Current target: `0.1.0-alpha.1`.

The CLI generates a runnable starter application, while the framework crates provide the reusable runtime pieces: modules, routing, migrations, auth, Inertia, SSR, flash messages, and code generation conventions.

## Documentation

- [Architecture](docs/ARCHITECTURE.md)
- [CLI contract](docs/CLI_CONTRACT.md)
- [SSO / OIDC](docs/SSO.md)
- [Scaffold template contract](docs/TEMPLATE_CONTRACT.md)
- [Distribution plan](docs/DISTRIBUTION.md)

## Distribution Model

- `rwfw-cli` is the product entrypoint. Developers use `rwfw new app`, `rwfw dev`, `rwfw migrate`, `rwfw seed admin`, `rwfw auth sso add`, and `rwfw generate scaffold`.
- Generated apps are starter applications, not throwaway demos. They include Docker, PostgreSQL, pgAdmin, production build files, local scaffold templates, auth pages, and a selectable example: `--example blog` or `--example ecommerce`.
- Framework crates are consumed by generated apps through `--rwfw-path` during local framework development, `--rwfw-git` + `--rwfw-tag` for Git/tag alpha distribution, and later through versioned crate dependencies when published.
- Project-local templates live in `.rwfw/templates` inside every generated app. Teams can customize scaffold output without forking the CLI.

## Local Framework Workflow

```bash
cargo check --workspace
npm install
npm run build
npm run build:ssr
```

Generate a test application from this checkout:

```bash
mkdir -p /tmp/rwfw-test
cd /tmp/rwfw-test
cargo run --manifest-path /home/bruno/code/rwfw/Cargo.toml -p rwfw-cli -- \
  new app demo-app --example blog --rwfw-path /home/bruno/code/rwfw
```

Inside the generated app:

```bash
docker compose -f compose.dev.yaml up -d
cargo run --manifest-path /home/bruno/code/rwfw/Cargo.toml -p rwfw-cli -- migrate
cargo run --manifest-path /home/bruno/code/rwfw/Cargo.toml -p rwfw-cli -- \
  seed admin --email admin@example.com --password rwfw-admin-123
cargo run --manifest-path /home/bruno/code/rwfw/Cargo.toml -p rwfw-cli -- dev
```

## Git Distribution Workflow

Before crates.io publishing is stable, distribute RWFW from a Git tag:

```bash
git tag v0.1.0-alpha.1
git push origin v0.1.0-alpha.1
cargo install --git https://github.com/<org>/rwfw --tag v0.1.0-alpha.1 rwfw-cli
rwfw new app demo-app --example ecommerce \
  --rwfw-git https://github.com/<org>/rwfw \
  --rwfw-tag v0.1.0-alpha.1
```

The generated app pins all RWFW framework dependencies to that Git tag. This makes the app reproducible even while the framework API is still moving.

## SSO / OIDC

RWFW auth supports OpenID Connect SSO for providers such as Keycloak and Azure AD B2C:

```bash
rwfw auth sso add keycloak \
  --provider keycloak \
  --issuer-url http://localhost:8081/realms/rwfw \
  --client-id-env KEYCLOAK_CLIENT_ID \
  --client-secret-env KEYCLOAK_CLIENT_SECRET
```

The command updates app config and `.env.example`. Runtime secrets stay in env vars.

## Smoke Test

The template smoke test generates a fresh app, checks scaffold output, builds assets, optionally starts Docker services, runs migrations and seeds, and optionally runs Playwright:

```bash
scripts/smoke-template.sh
```

This script is the release gate for generated app distribution. CI runs the same script with Docker disabled; local release validation should run the full script.

Useful switches:

```bash
RWFW_SMOKE_SKIP_NPM=1 scripts/smoke-template.sh
RWFW_SMOKE_SKIP_DOCKER=1 scripts/smoke-template.sh
RWFW_SMOKE_SKIP_PLAYWRIGHT=1 scripts/smoke-template.sh
RWFW_SMOKE_KEEP_TMP=1 scripts/smoke-template.sh
```

## Production

Generated apps include `Dockerfile.prod` and `compose.yaml`. The production image builds Vite client assets, the SSR bundle, and the Rust binary, then serves hashed assets from `dist/client` using the Vite manifest.

```bash
docker compose up --build
```

Set `RWFW_RUN_MIGRATIONS=0` if migrations should be handled by external deployment automation.
