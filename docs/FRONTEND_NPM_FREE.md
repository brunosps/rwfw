# RWFW sem npm: substituindo o motor React por Hotwire + templates Rust

> ✅ **Implementado** (PR #1, branch `feat/npm-free-hotwire`). Este documento é o **registro de decisão/design**,
> escrito antes da implementação. Para o **guia de uso prático** da stack atual (com exemplos), veja
> **[docs/FRONTEND.md](FRONTEND.md)**.

Documento de pesquisa/decisão. Objetivo: que os apps gerados pelo `rwfw new` **nasçam sem npm/node**,
mitigando o risco de ataques à cadeia de suprimentos npm, mantendo SSR, navegação SPA-like e tempo-real,
com a camada de view **acessível a quem sabe HTML** (não impondo Rust na view).

## 1. Por que (a ameaça npm, 2024–2026)

- **Worms auto-replicantes (Shai-Hulud, set/2025):** 500+ pacotes via tokens roubados; Shai-Hulud 2.0
  (nov/2025) atingiu Zapier, PostHog, Postman — 800+ pacotes, 400k+ segredos vazados.
- **Takeover de mantenedor (set/2025):** `chalk`, `debug`, `ansi-regex`, `color-name` comprometidos,
  malware entregue bilhões de vezes/dia.
- **Typosquatting / dependency confusion (out/2025–mai/2026):** campanhas mirando `react-router-dom`,
  `zustand`, libs de cloud/CI, roubando credenciais AWS/Vault.
- **`postinstall`:** executa código arbitrário no `npm install`.

Superfície concreta: um app React médio carrega **800–1.500 pacotes transitivos** — cada um um ponto de
confiança. É essa árvore que queremos remover dos apps gerados.

## 2. Diagnóstico do RWFW: onde mora o npm e a "costura"

| Camada | Hoje | Pacotes npm |
|---|---|---|
| View | React 19 + `@inertiajs/react` (`.tsx`) | `react`, `react-dom`, `@inertiajs/react` |
| SSR | Vite → `dist/server/ssr.js` rodado em V8 (`ssr_rs`) | bundle gerado por npm |
| Build | Vite + TS + Tailwind/PostCSS | `vite`, `@vitejs/plugin-react`, `typescript`, `tailwindcss`, `postcss`, `autoprefixer` |

A troca é viável porque todo handler renderiza por **um único ponto**:
`i.render_with_ssr("blog/Index", props_json)` (`crates/rwfw-core/src/inertia/extractor.rs:76`).
O nome do componente é uma string e props é JSON — **qualquer motor `(nome, props) → HTML` encaixa** sem
tocar em rotas, módulos, RBAC, migrations ou use-cases.

## 3. Decisão

**Hypermedia em Rust + Hotwire (Turbo + Stimulus) + templates MiniJinja/Askama, vendorizado via import maps,
Tailwind via binário standalone. Zero npm nos apps gerados.**

Comparação (todos vendorizáveis, sem npm):

| Critério | **Hotwire** (Turbo+Stimulus) | Datastar | HTMX+Alpine |
|---|---|---|---|
| Navegação SPA-like (substituir Inertia) | ⭐ Turbo Drive | bom | bom (hx-boost) |
| Estado local p/ dev JS | Stimulus | signals | Alpine |
| Tempo-real | Turbo Streams (WS/SSE) | ⭐ SSE nativo | extensão SSE |
| Maturidade | ⭐ Alta (Rails) | média | alta |
| Precedente zero-npm | ⭐ importmap-rails | sim | sim |

**Por quê Hotwire:** Turbo Drive é o substituto mais fiel do Inertia (navegação sem reload, sem JS escrito);
Turbo Streams cobre tempo-real; Stimulus é a válvula de escape em JS acessível ao dev JS; e o Rails prova
há anos que roda sem Node via import maps. **Alternativa secundária:** Datastar (1 arquivo, signals + SSE).

### Opção descartada como principal — Leptos/WASM (com números)

| Fator | Leptos (WASM) | Hypermedia |
|---|---|---|
| Payload inicial | ~150–600 KB WASM (brotli) + baixar/compilar | ~14–30 KB JS, já interativo |
| SSR | SSR + hydration (render duplo) | SSR puro, sem hydration |
| Linguagem da view | **Rust** (viola "acessível ao dev JS") | HTML (Jinja) |
| Curva p/ dev JS | alta | baixa |

Resolve zero-npm e SSR, mas viola a restrição "view acessível ao dev JS" e reintroduz hidratação. Fica como
alternativa futura.

## 4. A cola única no `rwfw-core` (escrita uma vez; depois cada página é só template)

### 4.1 Novo módulo `view` (irmão do `Inertia`; coexistem na migração)

```rust
// crates/rwfw-core/src/view/mod.rs  (NOVO)
pub struct View {
    shared_props: Value,
    url: String,
    headers: HeaderMap,
    is_turbo_stream: bool, // detecta Accept: text/vnd.turbo-stream.html
}

impl View {
    /// Mesma ergonomia do i.render: nome do template + props como contexto
    pub fn render(&self, template: &str, props: Value) -> Response { /* MiniJinja + html shell */ }
    pub fn render_status(&self, status: StatusCode, template: &str, props: Value) -> Response { /* ... */ }
    /// Fragmento sem layout (para Turbo Streams / parciais)
    pub fn render_fragment(&self, template: &str, props: Value) -> String { /* ... */ }
    /// Turbo Stream: <turbo-stream action="..." target="...">...</turbo-stream>
    pub fn turbo_stream(&self, action: StreamAction, target: &str, html: &str) -> Response { /* ... */ }
}
```

### 4.2 `render_html_shell` — trocar Vite por import map (`crates/rwfw-core/src/inertia/response.rs:3`)

```rust
// ANTES:  {vite_assets}  +  <div id="app" data-page="{json}">{ssr}</div>
// DEPOIS:
r#"<!DOCTYPE html>
<html lang="pt-br">
<head>
  <meta charset="UTF-8">
  <meta name="viewport" content="width=device-width, initial-scale=1.0">
  <meta name="csrf-token" content="{csrf}">
  <title>RWFW</title>
  <link rel="stylesheet" href="/assets/app.css">
  <script type="importmap">
  { "imports": {
      "@hotwired/turbo": "/vendor/turbo.min.js",
      "@hotwired/stimulus": "/vendor/stimulus.min.js"
  } }
  </script>
  <script type="module">
    import "@hotwired/turbo"                          // Turbo Drive liga sozinho
    import { Application } from "@hotwired/stimulus"
    import Dropdown from "/controllers/dropdown_controller.js"
    const Stimulus = Application.start()
    Stimulus.register("dropdown", Dropdown)
  </script>
</head>
<body>{body}</body>
</html>"#
```

### 4.3 Demais peças
- **Servir estáticos** `/vendor/*` e `/assets/app.css` via `tower-http::ServeDir` (ou `rust-embed` para
  manter o binário único da produção).
- **Helper SSE** (`axum::response::Sse`) + `turbo_stream(...)` para tempo-real.
- **CSRF** já existe (`crates/rwfw-core/src/csrf.rs`) — token via `<meta>`/hidden field.
- **Some do binário:** o motor SSR V8 inteiro (`crates/rwfw-core/src/ssr/*` + dependência `ssr_rs`) e o
  `crates/rwfw-core/src/vite.rs`. É menos código e dependência.

## 5. Antes e depois (no código real do RWFW)

### 5.1 Lista do blog — rota (`crates/modules/blog/src/routes/index.rs`)

```rust
// ANTES
async fn get(user: CurrentUser, i: Inertia) -> Response {
    if !user.can("blog.posts.view") { return AppError::Forbidden(...).into_response(); }
    i.render_with_ssr(
        "blog/Index",
        serde_json::json!({ "title": "Blog", "posts": [], "pagination": { "page": 1, "per_page": 20, "total": 0, "total_pages": 1 } }),
    ).await
}

// DEPOIS
async fn get(user: CurrentUser, v: View) -> Response {
    if !user.can("blog.posts.view") { return AppError::Forbidden(...).into_response(); }
    v.render(
        "blog/index",
        serde_json::json!({ "title": "Blog", "posts": [], "pagination": { "page": 1, "per_page": 20, "total": 0, "total_pages": 1 } }),
    ) // síncrono, sem V8
}
```

### 5.2 Lista do blog — página (`Index.tsx` → `crates/modules/blog/web/templates/index.html.j2`)

```jinja
{% extends "layouts/app.html.j2" %}
{% block content %}
<div class="max-w-4xl">
  <div class="flex justify-between items-center mb-8">
    <h1 class="text-3xl font-bold">{{ title or "Blog" }}</h1>
    <a href="/blog/posts/create" class="px-4 py-2 bg-blue-600 text-white rounded-md hover:bg-blue-700">New Post</a>
  </div>

  {% if posts | length == 0 %}
    <p class="text-gray-500">No posts yet.</p>
  {% else %}
    <div class="space-y-4">
      {% for post in posts %}
      <div class="p-6 bg-white rounded-lg shadow-sm border">
        <a href="/blog/posts/{{ post.id }}">
          <h2 class="text-xl font-semibold hover:text-blue-600">{{ post.title }}</h2>
        </a>
        <p class="mt-2 text-gray-600 line-clamp-2">{{ post.body }}</p>
        <p class="mt-2 text-sm text-gray-400">{{ post.created_at }}</p>
      </div>
      {% endfor %}
    </div>
  {% endif %}

  {% if pagination.total_pages > 1 %}
  <div class="mt-8 flex justify-center gap-2">
    {% for page in range(1, pagination.total_pages + 1) %}
      <a href="/blog?page={{ page }}"
         class="px-3 py-1 rounded {{ 'bg-blue-600 text-white' if page == pagination.page else 'bg-white text-gray-700 border hover:bg-gray-50' }}">{{ page }}</a>
    {% endfor %}
  </div>
  {% endif %}
</div>
{% endblock %}
```

Os `<a>` comuns são interceptados pelo Turbo Drive (navegação sem reload) — substituem o `<Link>` do Inertia.

### 5.3 Login — página (`Login.tsx` → `crates/modules/auth/web/templates/login.html.j2`)

```jinja
{% extends "layouts/auth.html.j2" %}
{% block content %}
<form method="post" action="/auth/login" class="mt-8 space-y-6 bg-white p-8 rounded-lg shadow-sm">
  <input type="hidden" name="_csrf" value="{{ csrf_token }}">
  <h2 class="text-2xl font-bold text-center">Sign In</h2>

  {% if ssoProviders | length > 0 %}
  <div class="space-y-3">
    {% for p in ssoProviders %}
      <a href="{{ p.loginUrl }}" class="block w-full rounded-md border border-gray-300 px-4 py-2 text-center text-sm font-semibold text-gray-800 hover:bg-gray-50">
        Continue with {{ p.displayName }}
      </a>
    {% endfor %}
    <div class="relative">
      <div class="absolute inset-0 flex items-center"><div class="w-full border-t border-gray-200"></div></div>
      <div class="relative flex justify-center text-xs uppercase"><span class="bg-white px-2 text-gray-500">or sign in locally</span></div>
    </div>
  </div>
  {% endif %}

  <div>
    <label for="email" class="block text-sm font-medium text-gray-700">Email</label>
    <input id="email" name="email" type="email" value="{{ old.email | default('') }}"
           class="mt-1 block w-full rounded-md border border-gray-300 px-3 py-2 shadow-sm focus:border-blue-500 focus:outline-none focus:ring-1 focus:ring-blue-500" required>
    {% if errors.email %}<p class="mt-1 text-sm text-red-600">{{ errors.email }}</p>{% endif %}
  </div>

  <div>
    <label for="password" class="block text-sm font-medium text-gray-700">Password</label>
    <input id="password" name="password" type="password"
           class="mt-1 block w-full rounded-md border border-gray-300 px-3 py-2 shadow-sm focus:border-blue-500 focus:outline-none focus:ring-1 focus:ring-blue-500" required>
    {% if errors.password %}<p class="mt-1 text-sm text-red-600">{{ errors.password }}</p>{% endif %}
  </div>

  <button type="submit" data-turbo-submits-with="Signing in..."
          class="w-full py-2 px-4 bg-blue-600 text-white rounded-md hover:bg-blue-700 disabled:opacity-50">
    Sign In
  </button>

  <p class="text-center text-sm text-gray-600">
    Don't have an account? <a href="/auth/register" class="text-blue-600 hover:underline">Register</a>
  </p>
</form>
{% endblock %}
```

`data-turbo-submits-with` reproduz o estado `processing`/disabled do React sem JS. `errors.*` vêm do servidor.

### 5.4 Login — rota (`crates/modules/auth/src/routes/login.rs`)

```rust
// ANTES: Json(input): Json<LoginUserInput>  +  i.redirect_back_with_errors(errors)
// DEPOIS: Form(input)  +  re-render com errors/old em 422
async fn post(State(state): State<AppState>, v: View, Form(input): Form<LoginUserInput>) -> Response {
    let strategy = LocalAuthStrategy::new();
    let use_case = LoginUserUseCase;
    match use_case.execute(&strategy, &state.db, input.clone()).await {
        Ok(output) => {
            // cria sessão + cookie (igual hoje); Turbo segue o redirect 303
            let mut response = Inertia::redirect("/home");
            // ... append_set_cookie(...) ...
            response
        }
        Err(AppError::Validation(errors)) => v.render_status(
            StatusCode::UNPROCESSABLE_ENTITY,
            "auth/login",
            serde_json::json!({ "errors": errors, "old": { "email": input.email }, "ssoProviders": login_providers(&state) }),
        ),
        Err(error) => error.into_response(),
    }
}
```

A lógica de auth/sessão/cookie não muda; só `Json→Form` e o caminho de erro.

### 5.5 Layout (`AppLayout.tsx` → `crates/app/web/layouts/app.html.j2`)

```jinja
<div class="min-h-screen flex">
  <aside class="w-64 bg-gray-900 text-white p-4 flex flex-col">
    <div class="mb-8">
      <h1 class="text-xl font-bold">RWFW</h1>
      <p class="text-gray-400 text-sm">Modular Framework</p>
    </div>
    <nav class="flex-1 space-y-1">
      {% for mod in modules %}
        {% for item in mod.nav_items %}
          <a href="{{ item.href }}" class="block px-3 py-2 rounded-md text-sm hover:bg-gray-800 transition-colors">{{ item.label }}</a>
        {% endfor %}
      {% endfor %}
    </nav>
    {% if auth.user %}
    <div class="border-t border-gray-700 pt-4 mt-4">
      <p class="text-sm text-gray-300">{{ auth.user.name }}</p>
      <p class="text-xs text-gray-500">{{ auth.user.email }}</p>
    </div>
    {% endif %}
  </aside>

  <main class="flex-1 p-8">
    {% if flash.success %}<div class="mb-4 p-4 bg-green-100 text-green-700 rounded-md">{{ flash.success }}</div>{% endif %}
    {% if flash.error %}<div class="mb-4 p-4 bg-red-100 text-red-700 rounded-md">{{ flash.error }}</div>{% endif %}
    {% if flash.info %}<div class="mb-4 p-4 bg-blue-100 text-blue-700 rounded-md">{{ flash.info }}</div>{% endif %}
    {% block content %}{% endblock %}
  </main>
</div>
```

Os shared props (`auth`/`flash`/`modules`) que já vêm do middleware viram o contexto do template.

### 5.6 Tempo-real (Turbo Streams + SSE)

```rust
// Quando um post é criado, faz broadcast de um fragmento para quem está na lista:
let card = v.render_fragment("blog/_post_card", serde_json::json!({ "post": post }));
broadcaster.send(turbo_stream(StreamAction::Prepend, "posts", &card));
// => <turbo-stream action="prepend" target="posts"><template>…card…</template></turbo-stream>
```

```jinja
{# no template da lista #}
<turbo-stream-source src="/blog/stream"></turbo-stream-source>
<div id="posts" class="space-y-4"> … </div>
```

```rust
// endpoint SSE em Axum
async fn stream(State(state): State<AppState>) -> Sse<impl Stream<Item = Result<Event, Infallible>>> {
    let rx = state.broadcaster.subscribe();
    Sse::new(BroadcastStream::new(rx).map(|html| Ok(Event::default().data(html))))
}
```

Tudo Rust + HTML, zero npm.

## 6. Antes e depois — nos arquivos gerados pelo CLI

`crates/rwfw-cli/src/commands/new_app.rs` hoje gera todo o aparato npm. Muda o que é emitido:

| Hoje gera (linhas em `new_app.rs`) | Vira |
|---|---|
| `package.json` (`:949`), `tsconfig.json` (`:982`), `vite.config.ts` (`:1014`), `tailwind.config.ts` (`:1248`), `postcss.config.js` | removidos (sem `package.json`) |
| `app.tsx` (`:187`), `ssr.tsx` (`:188`), layouts/components `.tsx` | `layouts/app.html.j2`, `layouts/auth.html.j2`, parciais |
| páginas `.tsx` dos módulos (`:211`, `:236`, `:1443`…) | `web/templates/*.html.j2` |
| Dockerfile com `node:22` + `npm ci` + `npm run build` (`:546`,`:548`) | Dockerfile só-Rust + passo do Tailwind standalone |
| next-steps imprime `npm install` (`:252`,`:729`) | imprime só `rwfw migrate` / `rwfw dev` |
| `.rwfw/templates/page_index.tsx.tera` (`:113`) etc. | `.rwfw/templates/page_index.html.tera` etc. |

### 6.1 Estrutura gerada — antes/depois

```text
ANTES (gerado por rwfw new)            DEPOIS
├── package.json                       ├── (sem package.json)
├── tsconfig.json                      ├── (sem tsconfig)
├── vite.config.ts                     ├── (sem vite)
├── tailwind.config.ts                 ├── tailwind.config.js  (lido pelo binário standalone)
├── crates/app/web/                    ├── crates/app/web/
│   ├── app.tsx                        │   ├── templates/layouts/app.html.j2
│   ├── ssr.tsx                        │   ├── templates/layouts/auth.html.j2
│   ├── layouts/AppLayout.tsx          │   ├── controllers/dropdown_controller.js
│   └── components/*.tsx               │   ├── vendor/turbo.min.js      (pinado, SHA)
└── crates/modules/blog/web/pages/*.tsx│   ├── vendor/stimulus.min.js   (pinado, SHA)
                                       │   └── assets/app.css           (saída do Tailwind)
                                       └── crates/modules/blog/web/templates/*.html.j2
```

### 6.2 Scaffolding CRUD (`rwfw generate` + `.rwfw/templates`)

`generate.rs` passa a emitir `*.html.j2` + rota em vez de `.tsx`. Os templates de scaffold
`.rwfw/templates/page_index.tsx.tera` → `page_index.html.tera` produzem páginas Hotwire.
Resultado: `rwfw new` cria app que roda com `cargo run` puro — sem `node_modules`, sem `npm install`.

## 7. Stimulus — onde fica o JS (pouco, e seu)

- Stimulus (lib) = 1 arquivo vendorizado (`web/vendor/stimulus.min.js`).
- Controllers = seus arquivos em `crates/app/web/controllers/*.js` (auditáveis, não-npm; ~10–30 linhas).

```js
// crates/app/web/controllers/dropdown_controller.js
import { Controller } from "@hotwired/stimulus"
export default class extends Controller {
  static targets = ["menu"]
  toggle() { this.menuTarget.classList.toggle("hidden") }
  hide(e)  { if (!this.element.contains(e.target)) this.menuTarget.classList.add("hidden") }
}
```

```jinja
{# uso no template #}
<div data-controller="dropdown" data-action="click@window->dropdown#hide">
  <button data-action="dropdown#toggle">Menu</button>
  <div data-dropdown-target="menu" class="hidden"> … </div>
</div>
```

Registro no `<script type="module">` do shell (ver 4.2). CLI: `rwfw generate controller <nome>`.

## 8. Componentes prontos ("preciso de um datepicker")

Catálogo menor que o npm, mas em 4 camadas:

1. **Plataforma nativa (zero deps):** `<input type="date">`, `<dialog>`, `<details>`, `<datalist>`.
2. **Web Components vendorizáveis (1 arquivo):**

| Lib | Tags | Observação |
|---|---|---|
| Shoelace / Web Awesome | `<sl-*>` (date, dialog, select, tabs, tooltip…) | suíte grande, Lit, ESM/CDN |
| Material Web (Google) | `<md-*>` Material 3 | importa Lit (vendorizar junto) |
| Spectrum (Adobe) / Carbon (IBM) / Fluent–FAST (Microsoft) | suítes corporativas | idem |
| Cally / Duet Date Picker | `<calendar-date>` / `<duet-date-picker>` | datepickers sem deps |
| Lit | — | para escrever os seus |

Descoberta: webcomponents.org; listas `awesome-web-components` e `awesome-standalones` (sem deps).

3. **UI Tailwind copia-e-cola:** daisyUI (roda com o binário standalone), Flowbite, Preline, Tailwind UI.
4. **Comportamento custom:** controllers Stimulus.

### 8.1 Datepicker — antes e depois

```text
HOJE (React):  npm i react-datepicker  →  N deps transitivas  →  import + <DatePicker/>
```

```jinja
{# DEPOIS — opção zero-dep (resolve a maioria) #}
<input type="date" name="published_at" value="{{ post.published_at }}"
       class="mt-1 block w-full rounded-md border border-gray-300 px-3 py-2">
```

```jinja
{# DEPOIS — opção rica (Web Component vendorizado) #}
{# 1) curl .../cally.js > web/vendor/cally.js   2) import map: "cally": "/vendor/cally.js" #}
<calendar-date class="cally" onchange="this.nextElementSibling.value = this.value">
  <calendar-month></calendar-month>
</calendar-date>
<input type="hidden" name="published_at">
```

## 9. Oportunidade: comando `rwfw add component` (vendoring com integridade)

```bash
rwfw add component cally        # ou: rwfw vendor add <nome|url>
```
- Baixa o ESM **pinado** de um manifesto curado (nome → URL + versão + SHA-384 esperado).
- **Verifica o hash (SRI)**; aborta se não bater. Grava em `web/vendor/`, atualiza import map + `vendor.lock`.
- (Opcional) scaffolda um controller Stimulus de ponte; pode emitir `<script integrity="sha384-…">`.
- `rwfw vendor verify` re-checa hashes no CI.

**Mais seguro que npm:** plano (sem deps transitivas automáticas), tudo pinado + verificado por hash,
commitado e auditável, **sem `postinstall`**. É um gerenciador de frontend com as defesas que faltam ao npm.

## 10. Migração incremental (alto nível)

- `Inertia` (React) e `View` (Hotwire) coexistem; migra-se **um módulo por vez**: home → blog → auth.
- Smoke test (`scripts/smoke-template.sh`) verde a cada passo.
- npm só é deletado do CLI quando o último `.tsx` sair.
- Se um módulo travar, o caminho já entrega mitigação parcial (import map remove npm do build).

## 11. Plano B (se a eliminação total não for viável)

Endurecer onde npm sobrar: lockfile commitado, `npm ci --ignore-scripts`, provenance/Sigstore,
escanear com Socket.dev/Snyk, minimizar deps, vendorizar libs críticas. Reduz o raio de explosão.

## 12. Trade-offs (honesto)

- **Ganha:** zero npm nos apps gerados; binário Rust único; SSR sem hidratação; first paint interativo;
  superfície de supply-chain drasticamente menor; menos código (some o V8/`ssr_rs`).
- **Perde:** ecossistema de componentes menor que o npm; estado client-side rico exige Stimulus/Web
  Components (JS escrito por você, porém pouco); reescrita das ~14 telas `.tsx` (incremental).

## 13. Fontes (seleção)

- Shai-Hulud / meltdown 2025: dev.to/usman_awan/the-night-npm-caught-fire-…; infoworld.com/article/4117145
- Microsoft Security (mai/2026): typosquatted npm packages stealing cloud/CI secrets; dependency confusion
- Transitive deps: pkgpulse.com/guides/dependencies-deep-dive-most-nested-dependency-trees
- Hotwire: turbo.hotwired.dev, stimulus.hotwired.dev; importmap-rails (github.com/rails/importmap-rails)
- HTMX vendoring: htmx.org/essays/vendoring/ ; Datastar: data-star.dev
- Templates Rust: Askama (github.com/rinja-rs/askama), MiniJinja (github.com/mitsuhiko/minijinja)
- Tailwind standalone sem Node: medium.com/@edgedoval/leptos-can-use-tailwind-css-and-daisyui-without-node-js…
- Web Components: webcomponents.org; github.com/web-padawan/awesome-web-components; github.com/davatron5000/awesome-standalones; shoelace.style; wicky.nillia.ms/cally; github.com/duetds/date-picker
- Leptos/WASM: reintech.io/blog/leptos-vs-yew-vs-dioxus-…; book.leptos.dev/deployment/binary_size.html
- npm hardening: nodejs-security.com (ignore-scripts); blog.sigstore.dev/npm-provenance-ga; socket.dev

## 14. Componentes de view reutilizáveis

> Nota: esta seção refina o modelo de componentes/JS esboçado na seção 7. Onde a seção 7 mostrava
> `controllers/*.js` em pasta plana e um `rwfw generate controller`, vale o que está aqui: componentes
> **autocontidos em pasta** e **sem** um `generate controller` solto (o JS vive dentro do componente).

### 14.1 O problema (no código atual)
Hoje há duplicação/repetição: `Sidebar` e `FlashMessages` reescritos inline no `AppLayout.tsx`; e padrões
repetidos em todas as páginas — campo de formulário (label+input+erro), botão, paginação, tabela, cabeçalho
(título + botão de ação). O scaffold `.rwfw/templates/form.tsx.tera` e `page_index.tsx.tera` já tentam
reusar, mas via geração de código, não componentes em runtime.

### 14.2 Panorama multi-linguagem (o que roubamos)
Há forte convergência entre ecossistemas em torno de um modelo de **tag HTML** com 5 recursos:
(1) sintaxe `<x-...>`; (2) componentes **anônimos/arquivo-só**; (3) **props com defaults** inline;
(4) **slot default + nomeados**; (5) **attribute bag** com *merge* de classes.

| Ecossistema | Modelo | Ideia roubada |
|---|---|---|
| Laravel Blade | `<x-alert>` + anônimos | `@props` + attribute bag (classes somam) |
| Symfony Twig Components | `<twig:Alert>` | **Twig = família Jinja** → precedente p/ MiniJinja; slots via `{% block %}` |
| JinjaX (Python/Jinja) | `<Alert>` | precedente direto: 1 arquivo, `{#def#}`, `{{ content }}`, `attrs.render()` |
| Django Cotton | `<c-button>` | componente = só um `.html`; slots nomeados |
| Phoenix HEEx (Elixir) | `<.button>` + `attr`/`slot` | validação em compile-time; **slots com `:let`** (tabelas/listas) |
| Go templ | função tipada | a assinatura É o contrato tipado |
| ASP.NET | View Components + Tag Helpers | tag custom → código; atributos "rest" |
| Plataforma nativa | `<slot>` + `@scope` + Declarative Shadow DOM | CSS com escopo **sem build**; slots nativos |

### 14.3 Decisão
**Modelo híbrido + motor Dynja:**
- **App:** componentes como tag HTML `<x-...>` — arquivo `.html.j2` com `{#def#}` props, `{{ content }}`,
  slots nomeados, attribute bag. Acessível ao dev front. Hospedados no lado **MiniJinja** do Dynja
  (runtime, inclusive em produção — perf suficiente), o que viabiliza a resolução das tags e hot-reload.
- **Framework:** **primitivos tipados em Rust** (`<x-button>`, `<x-input>`…) via Askama (compile-time),
  expostos pela mesma sintaxe de tag.
- **Comportamento:** **Stimulus** como padrão (idiomático Hotwire) + **custom elements** para widgets
  autocontidos/terceiros. Ambos sem npm.
- **Slots:** default + nomeados + **`:let`** (dados de volta ao chamador) — wired ao `{% call(arg) %}` /
  `{{ caller(arg) }}` que o MiniJinja já tem.
- **CSS:** `@scope` nativo (escopo sem build, Tailwind continua funcionando); Declarative Shadow DOM só
  para widgets que exijam isolamento total (aceitando que o Tailwind não atravessa o shadow).

### 14.4 Componente autocontido (estrutura de pasta)
```
crates/app/web/components/alert/
├── index.html.j2     # markup (nome da pasta = tag → <x-alert>)
├── style.css         # CSS escopado via @scope (opcional)
└── controller.js     # comportamento Stimulus (opcional)
crates/app/web/elements/        # custom elements de terceiros (vendorizados, com SRI)
└── cally.js
```
Auto-descoberta: cada pasta em `components/*` registra o template, inclui o `style.css` (escopado) e
registra o `controller.js` no Stimulus usando o nome da pasta como identificador (`alert` →
`data-controller="alert"`).

### 14.5 Criar e usar (exemplo `Alert`)
`components/alert/index.html.j2`:
```jinja
{#def type="info", dismissible=false #}
<div class="alert alert-{{ type }}" {{ attrs.render() }}>
  <div class="alert-body">{{ content }}</div>
  {% if slots.actions %}<div class="alert-actions">{{ slots.actions }}</div>{% endif %}
  {% if dismissible %}<button data-action="alert#close" aria-label="Fechar">&times;</button>{% endif %}
</div>
```
Uso:
```jinja
<x-alert type="error" dismissible class="mt-4" id="save-error">
  Falha ao salvar o post.
  <x-slot name="actions"><a href="/blog/posts/create">Tentar de novo</a></x-slot>
</x-alert>
```
`class`/`id` caem no `attrs` (classes somam, não substituem); `{{ content }}` = slot default;
`<x-slot name="actions">` = slot nomeado.

### 14.6 Sintaxe `:` (expressão vs string) e fluxo de dados
- `rows="posts"` (sem `:`) = string literal `"posts"`.
- `:rows="posts"` (com `:`) = **expressão** → a variável `posts` do contexto da página.
- A variável vem do **handler Rust**: `v.render("blog/posts/index", json!({ "posts": posts, … }))`, onde
  `posts` é o `Vec<Post>` do repositório. A tag recebe os dados (serde), não uma string.

### 14.7 Slots com dados (`:let`) — tabela reutilizável
`components/table/index.html.j2`:
```jinja
{#def rows, columns #}
<table class="min-w-full">
  <thead><tr>{% for col in columns %}<th class="text-left p-2">{{ col }}</th>{% endfor %}</tr></thead>
  <tbody>
    {% for row in rows %}<tr class="border-t">{{ slots.row(row) }}</tr>{% endfor %}
  </tbody>
</table>
```
Uso (a página descreve só **uma linha** e recebe cada item via `:let`):
```jinja
<x-table :rows="posts" :columns="['Título','Criado em','Ações']">
  <x-slot name="row" :let="post">
    <td class="p-2">{{ post.title }}</td>
    <td class="p-2">{{ post.created_at }}</td>
    <td class="p-2"><a href="/blog/posts/{{ post.id }}/edit">Editar</a></td>
  </x-slot>
</x-table>
```
Compila para `{% call(post) … %}` + `{{ caller(post) }}` do MiniJinja. Onde paga: o `page_index` do CRUD e
os formulários (`<x-form-field>`) — os pontos mais repetidos do scaffold.

### 14.8 Web Components nativos (camada de comportamento/isolamento)
Resolvem um eixo diferente do reúso de HTML: **comportamento/isolamento no cliente**. Complementam (não
substituem) o componente de servidor.
- **Custom element + light DOM:** servidor emite HTML normal dentro de `<rwfw-foo>`; um JS vendorizado liga
  o comportamento. Tailwind funciona. Alternativa ao Stimulus para widgets autocontidos.
- **Declarative Shadow DOM:** `<template shadowrootmode="open">` com `<style>` + `<slot>` nativos; isolamento
  real, slots nativos, zero JS para render. ⚠️ Tailwind global não atravessa o shadow → só para widgets que
  precisam de isolamento forte.
- **CSS `@scope`** (baseline 2026): escopo por componente **sem** shadow DOM e **sem** quebrar o Tailwind —
  o meio-termo recomendado para CSS de componente.

### 14.9 Motor (Dynja) — nuance
A camada de tags `<x-...>` precisa de runtime → vive no lado **MiniJinja** (dev: hot-reload; prod: runtime,
perf ok). O lado **Askama** cobre os **primitivos tipados**. "Dynja híbrido" aqui = MiniJinja para
componentes-de-arquivo do app + Askama para primitivos Rust.

### 14.10 CLI (sem ambiguidade com o backend)
O backend usa rotas/handlers/use_cases (não "controllers"); o Stimulus controller só existe como arquivo
dentro da pasta do componente. Não há `generate controller` solto.
```bash
rwfw generate component alert          # components/alert/index.html.j2 (+ style.css)
rwfw generate component alert --js     # idem + components/alert/controller.js (Stimulus) já registrado
rwfw add component cally               # vendoriza Web Component de terceiros em web/elements/ (com SRI)
```

### 14.11 Catálogo `/components` (Storybook sem npm)
Rota auto-gerada (só em dev, ou protegida) que renderiza cada componente com props de exemplo — análogo a
Lookbook/Phoenix Storybook, sem npm. Cada componente declara variações de exemplo; a rota lista e renderiza
cada uma isolada. Bom para desenvolver e documentar componentes.

### 14.12 Mapeamento do código atual
- `Sidebar.tsx` → `<x-sidebar>`; `FlashMessages.tsx` → `<x-flash>` (deduplicado do layout).
- Padrões repetidos → `<x-form-field>`, `<x-button>`, `<x-pagination>`, `<x-table>`, `<x-page-header>`.
- Scaffolds `.rwfw/templates/form.tsx.tera`, `page_index.tsx.tera` → emitem páginas usando esses componentes.

### 14.13 Esforço
O grosso do trabalho novo é a **camada de tags** sobre o MiniJinja (parse `<x-...>` → macros/includes,
attribute bag, slots nomeados, `:let`). O `:let` liga-se ao `caller(args)` que o MiniJinja já tem (fiação,
não pesquisa). Primitivos Rust reusam Askama. CLI estende os geradores Tera existentes.

### 14.14 Fontes (componentes)
- ViewComponent (viewcomponent.org), Phlex (phlex.fun), Trailblazer Cells
- Laravel Blade Components (laravel.com/docs/blade); Symfony UX Twig Components (symfony.com/bundles/ux-twig-component)
- JinjaX (jinjax.scaletti.dev); Django Cotton (django-cotton.com); Slippers (mitchel.me/slippers)
- Phoenix function components/HEEx (hexdocs.pm/phoenix_live_view); Phoenix Storybook (hexdocs.pm/phoenix_storybook)
- Go templ (templ.guide); gomponents; ASP.NET View Components / Tag Helpers (learn.microsoft.com)
- MiniJinja (github.com/mitsuhiko/minijinja) — `{% call %}`/`caller`; Askama (askama.rs); Dynja (github.com/rdbo/dynja)
- Declarative Shadow DOM (web.dev/articles/declarative-shadow-dom); CSS @scope baseline 2026 (web-standards.dev)
- Lookbook (lookbook.build)
