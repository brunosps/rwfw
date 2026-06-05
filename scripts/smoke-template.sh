#!/usr/bin/env bash
set -euo pipefail

ROOT_DIR="$(cd "$(dirname "${BASH_SOURCE[0]}")/.." && pwd)"
TMP_DIR="$(mktemp -d "${TMPDIR:-/tmp}/rwfw-template-smoke.XXXXXX")"
APP_NAME="${RWFW_SMOKE_APP_NAME:-sample-shop}"
ADMIN_EMAIL="${RWFW_SMOKE_ADMIN_EMAIL:-admin@example.com}"
ADMIN_PASSWORD="${RWFW_SMOKE_ADMIN_PASSWORD:-rwfw-admin-123}"
export CARGO_TARGET_DIR="${RWFW_SMOKE_TARGET_DIR:-$ROOT_DIR/target/smoke-template}"

cleanup() {
  if [[ -n "${DEV_PID:-}" ]]; then
    kill "$DEV_PID" >/dev/null 2>&1 || true
  fi

  if [[ -d "$TMP_DIR/$APP_NAME" ]] && command -v docker >/dev/null 2>&1; then
    (
      cd "$TMP_DIR/$APP_NAME"
      docker compose -f compose.dev.yaml down -v >/dev/null 2>&1 || true
    )
  fi

  if [[ "${RWFW_SMOKE_KEEP_TMP:-0}" != "1" ]]; then
    for _ in 1 2 3; do
      rm -rf "$TMP_DIR" 2>/dev/null && break
      sleep 1
    done
    if [[ -d "$TMP_DIR" ]]; then
      echo "Could not remove smoke workspace: $TMP_DIR" >&2
    fi
  else
    echo "Kept smoke workspace: $TMP_DIR"
  fi
}
trap cleanup EXIT

pick_port() {
  local candidate
  while true; do
    candidate="$((30000 + RANDOM % 20000))"
    if ! ss -ltn 2>/dev/null | awk '{print $4}' | grep -q ":${candidate}$"; then
      echo "$candidate"
      return
    fi
  done
}

wait_for_url() {
  local url="$1"
  local attempts="${2:-60}"

  for _ in $(seq 1 "$attempts"); do
    if curl -fsS "$url" >/dev/null 2>&1; then
      return 0
    fi
    sleep 1
  done

  echo "Timed out waiting for $url" >&2
  return 1
}

run_rwfw() {
  cargo run --manifest-path "$ROOT_DIR/Cargo.toml" -p rwfw-cli -- "$@"
}

echo "Smoke workspace: $TMP_DIR"

cd "$TMP_DIR"
run_rwfw new app "$APP_NAME" --rwfw-path "$ROOT_DIR"

cd "$TMP_DIR/$APP_NAME"
test -f Dockerfile
test -f Dockerfile.prod
test -f compose.yaml
test -f compose.dev.yaml
test -f docker/entrypoint.sh
test -f .rwfw/templates/page_show.html.j2.tera
grep -q "rwfw seed admin" README.md

cargo check

run_rwfw new module billing
cargo check

run_rwfw generate scaffold Product --module blog \
  name:string \
  summary:text? \
  description:text \
  active:bool \
  price:decimal \
  available_on:date \
  published_at:datetime

test -f crates/modules/blog/web/templates/products/show.html.j2
test -f 'crates/modules/blog/src/routes/products/[id].rs'
grep -q "DOUBLE PRECISION" crates/modules/blog/src/migrations/*_create_products.sql
grep -q "summary TEXT" crates/modules/blog/src/migrations/*_create_products.sql
grep -q "DATE NOT NULL" crates/modules/blog/src/migrations/*_create_products.sql
cargo check

# Generated apps are npm-free (Hotwire + MiniJinja). RWFW_SMOKE_SKIP_NPM is kept
# for CI compatibility; with it set we stop after the cargo checks above.
if [[ "${RWFW_SMOKE_SKIP_NPM:-0}" == "1" ]]; then
  echo "Skipping runtime smoke checks because RWFW_SMOKE_SKIP_NPM=1"
  exit 0
fi

if [[ "${RWFW_SMOKE_SKIP_DOCKER:-0}" == "1" ]]; then
  echo "Skipping docker smoke checks because RWFW_SMOKE_SKIP_DOCKER=1"
  exit 0
fi

if ! command -v docker >/dev/null 2>&1 || ! docker compose version >/dev/null 2>&1; then
  echo "Skipping docker smoke checks because docker compose is not available"
  exit 0
fi

export RWFW_DEV_DB_PORT="${RWFW_DEV_DB_PORT:-$(pick_port)}"
export RWFW_PGADMIN_PORT="${RWFW_PGADMIN_PORT:-$(pick_port)}"
export RWFW_APP_PORT="${RWFW_APP_PORT:-$(pick_port)}"
export RWFW__DATABASE__URL="postgres://rwfw:rwfw@localhost:${RWFW_DEV_DB_PORT}/${APP_NAME//-/_}_dev"
export RWFW__SERVER__PORT="$RWFW_APP_PORT"

docker compose -f compose.dev.yaml up -d
run_rwfw migrate
run_rwfw seed admin --email "$ADMIN_EMAIL" --password "$ADMIN_PASSWORD" --update-password

RWFW__DATABASE__URL="$RWFW__DATABASE__URL" \
RWFW__SERVER__PORT="$RWFW_APP_PORT" \
run_rwfw dev >"$TMP_DIR/rwfw-dev.log" 2>&1 &
DEV_PID="$!"

wait_for_url "http://localhost:${RWFW_APP_PORT}/health"
wait_for_url "http://localhost:${RWFW_APP_PORT}/"
# Server-rendered HTML pages (no client bundle, no SSR node process).
curl -fsS "http://localhost:${RWFW_APP_PORT}/blog/posts" | grep -q "Posts"
curl -fsS "http://localhost:${RWFW_APP_PORT}/vendor/turbo.min.js" >/dev/null
curl -fsS "http://localhost:${RWFW_APP_PORT}/assets/app.css" >/dev/null

echo "Template smoke test passed."
