# Distribution Plan

Current target: `0.1.0-alpha.3`.

## Recommended Alpha Path

During framework development, use path-based generation:

```bash
cargo run --manifest-path /path/to/rwfw/Cargo.toml -p rwfw-cli -- \
  new app demo --example blog --rwfw-path /path/to/rwfw
```

This keeps generated apps pinned to the local checkout while the framework API and templates are still moving.

## Git Tag Path

For the first external alpha, cut a Git tag:

```bash
git tag v0.1.0-alpha.3
git push origin v0.1.0-alpha.3
```

Users can then install the CLI from Git:

```bash
cargo install --git https://github.com/<org>/rwfw --tag v0.1.0-alpha.3 rwfw-cli
rwfw new app demo --example ecommerce \
  --rwfw-git https://github.com/<org>/rwfw \
  --rwfw-tag v0.1.0-alpha.3
```

Generated apps should depend on the same Git tag until crates.io publishing is stable. The CLI install source does not automatically become the generated app dependency source; pass `--rwfw-git` and `--rwfw-tag` explicitly so the generated `Cargo.toml` is reproducible.

Environment variables are also supported for release scripts:

```bash
RWFW_FRAMEWORK_GIT=https://github.com/<org>/rwfw \
RWFW_FRAMEWORK_TAG=v0.1.0-alpha.7 \
rwfw new app demo --example blog
```

The generated dependencies use this shape:

```toml
rwfw-core = { git = "https://github.com/<org>/rwfw", tag = "v0.1.0-alpha.7", default-features = false, features = ["postgres"] }
rwfw-shared = { git = "https://github.com/<org>/rwfw", tag = "v0.1.0-alpha.7" }
rwfw-macros = { git = "https://github.com/<org>/rwfw", tag = "v0.1.0-alpha.7" }
mod-auth = { git = "https://github.com/<org>/rwfw", tag = "v0.1.0-alpha.7", default-features = false, features = ["postgres", "oidc"] }
```

## Cargo features

`rwfw-core`, `mod-auth` and `rwfw-tauri` compile each database driver behind a feature, so an app can
leave out the one it does not use (for SQLite that also drops the bundled `libsqlite3-sys`):

| Crate | Feature | What it adds | Default |
|---|---|---|---|
| `rwfw-core` | `postgres` | sea-orm `sqlx-postgres` driver | on |
| `rwfw-core` | `sqlite` | sea-orm `sqlx-sqlite` driver | on |
| `mod-auth` | `postgres` / `sqlite` | forwards to `rwfw-core` | on |
| `mod-auth` | `oidc` | SSO/OIDC login via `openidconnect` | on |
| `rwfw-tauri` | `postgres` / `sqlite` | forwards to `rwfw-core` | on |

The defaults keep every driver on, so existing `Cargo.toml` files build as before. `rwfw new app`
generates `default-features = false` with only the chosen database, and the generated workspace enables
only that sea-orm driver. Because Cargo unifies features, **every** dependency on these crates in the app
must turn default features off; one plain `rwfw-core = { ... }` line brings both drivers back.

Without `oidc`, the `/auth/sso/*` routes answer with an error redirect to the login page and logout ends
only the local session; the OIDC tables are still migrated. A database URL whose driver was left out
fails at connect with a message naming the missing feature.

## crates.io Path

Use crates.io after the alpha API is less volatile:

- Publish `rwfw-core`, `rwfw-shared`, `rwfw-macros`, and `rwfw-cli`.
- Keep all framework crates on the same version.
- Make `rwfw new app` default to versioned crate dependencies.
- Keep `--rwfw-path` for framework contributors and local testing.
- Keep `--rwfw-git` and `--rwfw-tag` for teams that want Git-pinned internal releases.

## Release Gate

Before tagging:

```bash
cargo check --workspace
cargo test --workspace
scripts/smoke-template.sh
```

The smoke script is the distribution gate because it validates what users actually receive: a fresh generated app, Docker files, scaffold templates, migrations, and — when not skipped — a running server whose pages are server-rendered HTML. There is no npm/Node build step.

For Git distribution, also generate one app with Git dependencies and inspect the generated manifest:

```bash
rwfw new app git-smoke --example blog \
  --rwfw-git https://github.com/<org>/rwfw \
  --rwfw-tag v0.1.0-alpha.3
grep -n "git = " git-smoke/Cargo.toml git-smoke/crates/app/Cargo.toml git-smoke/crates/modules/home/Cargo.toml
```
