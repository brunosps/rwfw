# Spec — Reactive Components, Fase 1: core loop

> Branch: `feat/reactive-components` (worktree). Esta é a Fase 1 de 4 do plano
> "componentes reativos no rwfw" (conceitos do phlex-reactive/Livewire, modelo
> **stateless**: identidade assinada no DOM, sem estado por conexão).
> Entregável da fase: **o counter reativo funcionando end-to-end** com testes.

## Modelo mental (o que estamos construindo)

Um componente reativo é um struct Rust (estado serde) + um template MiniJinja + ações
declaradas. O DOM carrega um **token HMAC** com `{versão, componente, estado}` — nunca
estado cru. Um clique vira `POST /__rwfw/reactive/actions {token, act, params}`; o
servidor **verifica o token → reconstrói o struct → roda a ação → re-renderiza o
template → responde um `<turbo-stream action=replace target=<id>>`** com token novo no
root (rolling token). O Turbo (já vendorado) aplica o patch. Zero JS por feature: um
único controller Stimulus genérico interpreta atributos `data-*`.

Segurança (contratos do phlex-reactive a preservar):
- **Nada do cliente é confiável**: token adulterado → 400; estado só viaja assinado.
- **Default-deny**: só ações declaradas são despacháveis (ação desconhecida → 403).
- **Params tipados via serde**: chave não declarada é dropada, tipo errado → 422/400.
- **Fail-closed em versão**: token com `v` desconhecida → 400 (rollback seguro).
- **Autorização é responsabilidade da ação** (o token prova identidade, não permissão);
  a ação recebe `Ctx` com `Option<CurrentUser>` para checar.

## O que já existe e DEVE ser reusado (não recriar)

- `crates/rwfw-core/src/view/turbo.rs` — `TurboStream`/`TurboAction` (formato de resposta).
- `crates/rwfw-core/src/view/extractor.rs` — extractor `View` (ganha método novo).
- `crates/rwfw-core/src/view/renderer.rs` — `ViewRenderer` (render de template por path).
- `crates/rwfw-core/src/app.rs` — `RwfwApp::build()` monta rotas built-in (`/health`,
  `/components`, `/__rwfw/livereload`…) — o endpoint novo entra aqui, atrás dos
  middlewares existentes (shared props/CurrentUser, CSRF).
- `crates/rwfw-core/src/module.rs` + `inventory` — padrão de registro a copiar.
- `crates/rwfw-core/src/config.rs` — config por env (`RWFW_ENV`); adicionar `RWFW_SECRET_KEY`.
- `crates/rwfw-app/web/templates/layouts/base.html.j2` — import map + boot do Stimulus;
  CSRF meta tag já existe (middleware injeta `csrf_token` nas shared props).
- `crates/rwfw-app/web/assets/dev-livereload.js` — exemplo de asset framework-owned
  embutido via rust-embed (`crates/rwfw-app/src/embed.rs` + `build.rs`).
- e2e com **chromiumoxide** — já há padrão no repo; seguir o existente.

## Entregas

### 1. Token assinado — `crates/rwfw-core/src/reactive/token.rs`

- HMAC-SHA256 (crates `hmac` + `sha2`, adicionar no workspace) sobre JSON compacto
  `{"v":1,"c":"<nome>","s":{...}}`, encodado base64-url sem padding.
- API: `sign(payload) -> String`, `verify(token) -> Result<Payload, TokenError>`.
- `verify` falha fechado: assinatura inválida, JSON inválido, `v` != suportada → erro
  tipado (vira 400 no endpoint).
- Chave: `RWFW_SECRET_KEY` no config. Em `development`/`test`: se ausente, derivar uma
  chave estável do path do projeto + warning no log. Em produção: **ausência de
  `RWFW_SECRET_KEY` só falha o boot se houver componente reativo registrado** (checável
  via inventory — app sem reactive não paga nada).
- Comparação constant-time (o crate `hmac` já dá via `verify_slice`).

### 2. Trait + registry — `crates/rwfw-core/src/reactive/{mod.rs,registry.rs}`

```rust
pub struct Ctx<'a> {
    pub state: &'a AppState,
    pub user: Option<&'a CurrentUser>,
}

pub trait ReactiveComponent: Serialize + DeserializeOwned + Send {
    const NAME: &'static str;            // "counter" — vai no token ("c")
    fn template(&self) -> &'static str;  // path minijinja, ex. "reactive/counter"
    fn dom_id(&self) -> String;          // id estável do root (default: NAME)
    fn context(&self) -> minijinja::Value; // estado → contexto do template
    // dispatch é GERADO pela macro — não implementar à mão:
    fn dispatch(&mut self, action: &str, params: serde_json::Value, ctx: &Ctx)
        -> Result<Reply, ReactiveError>;
}
```

- `Reply` (enum simples nesta fase): `Replace { morph: bool }` (default) e
  `Nothing`. Verbos extras (remove/redirect/append) ficam pra fase 2+ — deixar o
  enum extensível.
- `ReactiveError`: `Unauthorized` (→403), `NotFound` (→404), `Invalid(String)` (→422),
  `Internal` (→500). Endpoints mapeiam para status como o phlex-reactive.
- Registry: `inventory::collect!(ReactiveRegistration)` com
  `name -> fn(payload_json, action, params, ctx) -> Result<(String /*html*/, ...)>`
  (uma função monomorfizada por componente que desserializa o estado, despacha e
  re-renderiza). Copiar o padrão de `ModuleRegistration`.

### 3. Macros — `crates/rwfw-macros/src/reactive.rs` (novo; `routes.rs` intocado)

- `#[derive(ReactiveComponent)]` + atributos:

```rust
#[derive(Serialize, Deserialize, ReactiveComponent)]
#[reactive(name = "counter", template = "reactive/counter")]
pub struct Counter { pub count: i64 }

#[reactive_actions]           // impl block com as ações
impl Counter {
    #[action]
    fn increment(&mut self, _ctx: &Ctx) { self.count += 1; }
    #[action]
    fn decrement(&mut self, _ctx: &Ctx) { self.count -= 1; }
    // ação com params tipados: qualquer arg extra vira campo desserializado
    // de `params` via serde (struct de params gerada pela macro):
    #[action]
    fn set(&mut self, _ctx: &Ctx, value: i64) { self.count = value; }
}
```

- A macro gera: o `match` de dispatch (default-deny — nome fora do match → erro
  `UnknownAction`, endpoint responde 403 listando ações em dev), a desserialização
  tipada dos params (campo desconhecido dropado; use `#[serde(deny_unknown_fields)]`
  NÃO — dropar em silêncio como o original), e o `inventory::submit!` do registro.
- Ações podem retornar `()` (vira `Reply::Replace{morph:false}`), `Reply`, ou
  `Result<Reply, ReactiveError>` — a macro normaliza.
- Erro de compilação claro se `#[action]` for usado fora de `#[reactive_actions]`.

### 4. Endpoint — `crates/rwfw-core/src/reactive/endpoint.rs` + mount em `app.rs`

- `POST /__rwfw/reactive/actions`, body JSON `{token, act, params}`.
- Fluxo: verify token (400) → resolve `c` no registry (400 se desconhecido) →
  dispatch (403 unknown action / 403 Unauthorized / 404 NotFound / 422 Invalid) →
  re-render template com contexto novo → embrulhar no root com **token novo** →
  responder `TurboStream::new(TurboAction::Replace, dom_id, html)` (morph quando
  `Reply::Replace{morph:true}` — usar `method="morph"` no elemento turbo-stream;
  estender `turbo.rs` se preciso, de forma aditiva).
- Em dev (`is_development`): corpo de erro em texto com diagnóstico (espelho do
  `verbose_errors`); em produção, corpo vazio + warn no log em todo caso.
- CSRF: a rota passa pelo middleware existente; o controller JS manda o header.

### 5. Render server-side + helpers MiniJinja

- `View::reactive(&self, component: &impl ReactiveComponent) -> String` (em
  `view/extractor.rs`): renderiza `component.template()` com
  `component.context()` **+ funções bound no contexto**, e envolve num root:

```html
<div id="{dom_id}" data-controller="reactive"
     data-reactive-token="{token}" data-reactive-component="{name}">
  ...template renderizado...
</div>
```

- Helper de template `on(action, params?)` (função MiniJinja injetada no contexto
  do render reativo): emite os atributos do trigger —
  `data-action="click->reactive#trigger" data-reactive-action="increment"`
  (+ `data-reactive-params` JSON quando houver). Suportar `event=` kwarg
  (`on('save', event='submit')` → `submit->reactive#trigger`). Só isso na fase 1
  (modifiers debounce/confirm etc. são fase 3).
- No handler, uso: `v.render("page", context!{ counter => v.reactive(&Counter{count:0}) })`
  e no template `{{ counter|safe }}`. Documentar esse fluxo no rustdoc.

### 6. Controller JS genérico — `crates/rwfw-app/web/assets/reactive.js`

- Controller Stimulus registrado como `reactive` no `base.html.j2` (junto do boot
  existente do Stimulus; ~2 linhas no import map/script module).
- Responsabilidades (fase 1, manter enxuto — alvo 300–500 linhas comentadas):
  - `trigger(event)`: lê `data-reactive-action`/`data-reactive-params` do elemento,
    `preventDefault()` (exceto em window-bound — não existe ainda), coleta **campos
    nomeados** do root (`input[name], select[name], textarea[name]`), **parando em
    roots reativos aninhados** (elementos sob outro `[data-controller~="reactive"]`
    interno não são coletados), merge: params explícitos vencem campo homônimo.
  - POST JSON pra `/__rwfw/reactive/actions` com header CSRF lido da meta tag
    (mesmo nome que o middleware CSRF do rwfw espera — conferir em `csrf.rs`).
  - Resposta ok → `Turbo.renderStreamMessage(text)`. Erro → `console.error` com o
    corpo (dev traz diagnóstico).
  - Requisições serializadas por root (fila simples) pra não corromper o rolling
    token com cliques rápidos.
- Embutir via rust-embed como os demais assets; servir por `/assets/reactive.js`.

### 7. Demo + testes

- **Demo**: componente `Counter` no mod-blog OU num módulo demo mínimo novo
  (`crates/modules/demo`) — página `/demo/counter` com o counter (dois botões + valor),
  seguindo o exemplo canônico do phlex-reactive. Registrar no starter apenas se módulo
  demo (decidir pelo caminho de menor atrito; mod-blog já existe e linka).
- **Unit tests** (rwfw-core): sign/verify roundtrip; tamper → erro; versão desconhecida
  → erro; dispatch default-deny; params: chave desconhecida dropada, tipo errado →
  Invalid; rolling token (re-render emite token != anterior mas verificável).
- **Teste da macro** (trybuild ou teste de expansão simples se já houver padrão;
  senão, testes de comportamento no crate demo bastam).
- **e2e chromiumoxide** (seguir padrão existente no repo): abrir `/demo/counter`,
  clicar `+` duas vezes e `-` uma, asserir valor `1` sem reload de página; adulterar
  o token via JS e asserir que o clique não altera o DOM (e loga erro).

## Restrições (gate rejeita se violar)

1. **Tudo aditivo**: nenhuma API pública existente muda de assinatura; arquivos
   existentes tocados só os listados (app.rs, config.rs, view/extractor.rs,
   turbo.rs aditivo, base.html.j2, embed/build do rwfw-app, Cargo.tomls).
2. **Zero npm/Node**: o `reactive.js` é escrito à mão, vanilla + API do Stimulus
   vendorado. Nenhum passo de build JS.
3. App que não usa reactive: comportamento **byte-idêntico** (reactive.js é inerte
   sem `data-controller="reactive"`; secret não exigido sem componente registrado).
4. `cargo fmt` + `cargo clippy --workspace` limpo (warnings novos zero) +
   `cargo test --workspace` verde.
5. Commits atômicos por entrega (token / trait+registry / macro / endpoint /
   render+helpers / js / demo+testes), mensagens em inglês estilo conventional
   commits como o histórico do repo.
6. Docs: rustdoc nos itens públicos + seção curta em `docs/REACTIVE.md` (visão geral
   + exemplo counter completo). Não escrever docs especulativas de fases futuras.

## Fora de escopo (NÃO implementar nesta fase)

Broadcasts/tópicos/SSE novo (fase 2); modifiers debounce/throttle/confirm/disable_with
(fase 3); scaffold `--reactive-components` e `generate reactive` (fase 4); verbos de
Reply além de Replace/Nothing; record-backed sugar; optimistic hints; reactive_show.
