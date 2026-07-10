# Reactive Components

Reactive components are stateless server-rendered Rust structs. The DOM stores a
signed token with the component name and serialized state; actions post that
token to `/__rwfw/reactive/actions`, the server verifies it, dispatches a
declared action, re-renders the MiniJinja template, and returns a Turbo Stream
replacement.

```rust
#[derive(Serialize, Deserialize, ReactiveComponent)]
#[reactive(name = "counter", template = "demo/reactive/counter")]
pub struct Counter {
    pub count: i64,
}

#[reactive_actions]
impl Counter {
    #[action]
    fn increment(&mut self, _ctx: &Ctx<'_>) {
        self.count += 1;
    }

    #[action]
    fn set(&mut self, _ctx: &Ctx<'_>, value: i64) {
        self.count = value;
    }
}
```

Render from a handler with `View::reactive`:

```rust
async fn counter(v: View) -> Response {
    let counter = v.reactive(&Counter { count: 0 });
    v.render("demo/counter", serde_json::json!({ "counter": counter }))
}
```

Use `{{ counter|safe }}` in the page template. Inside the component template,
`on()` emits the generic Stimulus trigger attributes:

```jinja
<button type="button" {{ on("increment") }}>+</button>
<button type="submit" {{ on("set", event="submit") }}>Set</button>
```

Set `RWFW_SECRET_KEY` in production when any reactive component is registered.
Development and test derive a stable local key if it is absent.
