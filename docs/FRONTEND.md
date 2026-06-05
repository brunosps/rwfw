# Frontend Guide — Hotwire + MiniJinja (zero npm)

RWFW renders HTML **on the server** with [MiniJinja](https://github.com/mitsuhiko/minijinja) templates and
drives the browser with **Hotwire** (Turbo + Stimulus). There is no React, no Inertia, no Vite, no V8 SSR,
and **no npm/Node** anywhere — `cargo` is the only toolchain. The view layer stays plain HTML, so anyone who
knows HTML/CSS can work on it.

> Why this stack? See the decision record in [FRONTEND_NPM_FREE.md](FRONTEND_NPM_FREE.md).

Paths below use `crates/<app>` — that is `crates/rwfw-app` in this repo's dogfood and `crates/app` in a
generated app. Module paths are `crates/modules/<module>`.

---

## Rendering a page

A route handler takes the `View` extractor and calls `v.render("<module>/<page>", props)`. Props are a
`serde_json::Value`; the template name maps to a `.html.j2` file under the module's `web/templates`.

```rust
// crates/modules/home/src/routes/index.rs
use axum::response::Response;
use axum::routing;
use rwfw_core::app::AppState;
use rwfw_core::view::View;

pub fn route() -> axum::routing::MethodRouter<AppState> {
    routing::get(get)
}

async fn get(v: View) -> Response {
    v.render(
        "home/index",
        serde_json::json!({ "title": "Welcome to RWFW", "description": "Zero npm." }),
    )
}
```

```jinja
{# crates/modules/home/web/templates/index.html.j2 #}
{% extends "layouts/app.html.j2" %}
{% block title %}{{ title }}{% endblock %}
{% block content %}
  <h1 class="text-4xl font-bold mb-4">{{ title }}</h1>
  <p class="text-lg text-gray-600">{{ description }}</p>
{% endblock %}
```

**Template resolution.** `"home/index"` → the `home` module's `web/templates/index.html.j2`. App-level
templates (layouts, components) live in `crates/<app>/web/templates/`. Each crate exposes its `web/` dir
via `web_root()` (using `CARGO_MANIFEST_DIR`), so resolution is independent of the working directory and
works the same in dev, tests, and the production container.

**Shared props.** Middleware injects these into every template automatically — no need to pass them:

| Prop | Contents |
|---|---|
| `auth` | `{ user: { id, name, email }, roles, permissions }` or null |
| `flash` | one-shot `{ success, error, info }` from the previous request |
| `errors` | validation errors (`{ field: "message" }`) |
| `csrf_token` | the CSRF token for forms / the `<meta>` tag |
| `modules` | navigation items from every registered module |

Layouts use them, e.g. the app layout renders the sidebar from `modules` and the signed-in user from `auth`.

---

## Forms & validation

HTML forms post normally; the handler takes the `Form` extractor (not JSON). On validation failure the same
template is re-rendered at **422** with `errors` and `old` (the submitted values).

```rust
// POST /blog/posts  (crates/modules/blog/src/routes/posts/index.rs)
async fn post(
    State(state): State<AppState>,
    user: CurrentUser,
    v: View,
    Form(input): Form<CreatePostInput>,
) -> Response {
    let (title, body) = (input.title.clone(), input.body.clone());
    match use_case.execute(&repo, user.id, input).await {
        Ok(output) => Redirect::to(&format!("/blog/posts/{}", output.post.id)).into_response(),
        Err(AppError::Validation(errors)) => v.render_status(
            StatusCode::UNPROCESSABLE_ENTITY,
            "blog/create",
            serde_json::json!({
                "errors": rwfw_core::validation::first_messages(errors),
                "old": { "title": title, "body": body }
            }),
        ),
        Err(error) => error.into_response(),
    }
}
```

```jinja
{# crates/modules/blog/web/templates/create.html.j2 #}
<form method="post" action="/blog/posts" class="space-y-6">
  <input type="hidden" name="_csrf" value="{{ csrf_token }}">
  <x-field name="title" label="Title" :value="old.title | default('')" />
  <x-field name="body" label="Content" type="textarea" :value="old.body | default('')" />
  <button type="submit" data-turbo-submits-with="Creating...">Create Post</button>
</form>
```

- **CSRF** — every form includes `<input type="hidden" name="_csrf" value="{{ csrf_token }}">`. Turbo also
  echoes the `<meta name="csrf-token">` token as the `X-CSRF-Token` header, which the middleware verifies
  (double-submit) on `POST`/`PUT`/`PATCH`/`DELETE`. JSON requests and SSO callbacks are exempt.
- **Edit** — HTML can't issue `PUT`, so edit forms `POST` to the edit route (the handler maps it to update).
- **Delete** — use a Turbo link: `<a href="/blog/posts/{{ post.id }}" data-turbo-method="delete"
  data-turbo-confirm="Are you sure?">Delete</a>` (Turbo sends a real `DELETE` with the CSRF header).
- `data-turbo-submits-with="..."` gives the submit button a busy label with no JS.

---

## Components (`<x-...>`)

A component is a folder `web/templates/components/<name>/index.html.j2` declaring its props with a leading
`{#def ... #}` comment. Use it as an HTML tag; the compiler (`rwfw_core::view::tags`) rewrites it to a
MiniJinja include.

```jinja
{# crates/<app>/web/templates/components/field/index.html.j2 #}
{#def name, label, type="text", value="", required=true #}
<div>
  <label for="{{ name }}" class="block text-sm font-medium text-gray-700">{{ label }}</label>
  {% if type == "textarea" %}
  <textarea id="{{ name }}" name="{{ name }}" {% if required %}required{% endif %}>{{ value }}</textarea>
  {% else %}
  <input id="{{ name }}" name="{{ name }}" type="{{ type }}" value="{{ value }}" {% if required %}required{% endif %}>
  {% endif %}
  {% if errors[name] %}<p class="mt-1 text-sm text-red-600">{{ errors[name] }}</p>{% endif %}
</div>
```

```jinja
{# usage #}
<x-field name="email" label="Email" type="email" :value="old.email | default('')" />
```

- **Props** — `prop="value"` is a string literal; `:prop="expr"` is a MiniJinja expression. Defaults come
  from `{#def ... #}`.
- **Default slot** — the tag body is available as `{{ content }}`.
- **Named slots** — `<x-slot name="actions">…</x-slot>` inside the tag is rendered with `{{ slots.actions }}`.
- **Attribute bag** — undeclared attributes are collected and rendered (with class-merge) via the `attrs`
  filter, e.g. a button component:

  ```jinja
  {#def kind="primary" #}
  <button {{ __attrs | attrs(class="btn btn-" ~ kind) }}>{{ content }}</button>
  ```
  `<x-button kind="danger" id="go">Delete</x-button>` → `<button class="btn btn-danger" id="go">Delete</button>`.

**Catalog.** In development, `GET /components` lists every discovered component and its declared props
(a lightweight, npm-free "Storybook").

---

## Interactivity (Hotwire)

- **Turbo Drive** is loaded automatically and intercepts link clicks and form submits to give SPA-like
  navigation without a full reload — no JS to write.
- **Stimulus** is started in the shell (`window.Stimulus = Application.start()`). For custom behaviour, add
  a controller (a small JS file under the app's `web/`), register it in the shell's `<script type="module">`,
  and wire it in HTML with `data-controller="..."` / `data-action="..."`.
- For third-party Web Components (e.g. a date picker), vendor the single ESM file and add it to the import
  map — still no npm.

---

## Real-time (Turbo Streams over SSE)

`AppState` carries a broadcast channel. An SSE endpoint streams `<turbo-stream>` fragments; the page
subscribes with `<turbo-stream-source>`.

```rust
// GET /blog/posts/stream  (crates/modules/blog/src/routes/posts/stream.rs)
async fn get(State(state): State<AppState>) -> Sse<impl Stream<Item = Result<Event, Infallible>>> {
    let receiver = state.broadcaster.subscribe();
    let stream = BroadcastStream::new(receiver)
        .filter_map(|m| m.ok().map(|html| Ok(Event::default().data(html))));
    Sse::new(stream).keep_alive(KeepAlive::default())
}
```

```rust
// when a post is created, broadcast a prepend
let card = v.render_fragment("blog/posts/_card", serde_json::json!({ "post": output.post }));
let stream = rwfw_core::view::turbo::TurboStream::new(
    rwfw_core::view::turbo::TurboAction::Prepend, "posts", card,
).render();
let _ = state.broadcaster.send(stream);
```

```jinja
{# the list page #}
<div id="posts" class="space-y-4">
  {% for post in posts %}{% include "blog/posts/_card.html.j2" %}{% endfor %}
</div>
<turbo-stream-source src="/blog/posts/stream"></turbo-stream-source>
```

Anyone viewing the list sees new posts appear live, with no client framework.

---

## Assets & the shell

The base layout wires everything via a native **import map** — no bundler:

```jinja
{# crates/<app>/web/templates/layouts/base.html.j2 #}
<head>
  <meta name="csrf-token" content="{{ csrf_token }}">
  <link rel="stylesheet" href="/assets/app.css">
  <script type="importmap">
  { "imports": {
      "@hotwired/turbo": "/vendor/turbo.min.js",
      "@hotwired/stimulus": "/vendor/stimulus.min.js"
  } }
  </script>
  <script type="module">
    import "@hotwired/turbo"
    import { Application } from "@hotwired/stimulus"
    window.Stimulus = Application.start()
  </script>
</head>
```

- **Vendored JS** — `web/vendor/turbo.min.js` and `web/vendor/stimulus.min.js`, pinned with SHA-384
  integrity in `web/vendor.lock`. Served at `/vendor/*`.
- **CSS** — `web/assets/app.css`, compiled by the **Tailwind standalone binary** (no PostCSS/npm). Served
  at `/assets/*`.

---

## Dev / build / run

All `cargo`, no npm:

```bash
rwfw dev      # = cargo run -p <app>           (no Vite watcher)
rwfw build    # = cargo build --release -p <app>
rwfw migrate
rwfw seed admin --email admin@example.com --password rwfw-admin-123
```

The production image (`Dockerfile.prod`) is Rust-only and serves `web/` from disk.

---

## Testing

- **Integration tests** (`crates/rwfw-app/tests/`) spawn the real app and drive it with `reqwest`
  (cookie-aware), asserting rendered HTML, redirects, `422` re-renders, CSRF, sessions, and SSE. They are
  gated behind `RWFW_TEST_DATABASE_URL` (point it at Postgres; e.g. `docker compose -f compose.dev.yaml up -d db`).
- **Browser e2e** (`crates/rwfw-app/tests/e2e.rs`) drives headless Chromium with
  [`chromiumoxide`](https://crates.io/crates/chromiumoxide) — **no Playwright/Node**. It verifies Turbo and
  Stimulus load via the import map and that Turbo Drive navigates without a full reload. Gated behind
  `RWFW_E2E=1` (skips cleanly without a browser).

```bash
RWFW_TEST_DATABASE_URL=postgres://rwfw:rwfw@localhost:54329/rwfw_test cargo test --workspace
RWFW_E2E=1 RWFW_TEST_DATABASE_URL=... cargo test -p rwfw-app --test e2e
```
