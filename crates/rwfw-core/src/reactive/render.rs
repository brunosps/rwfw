use minijinja::value::{Kwargs, Value, merge_maps};

use super::token::Payload;
use super::{ReactiveComponent, ReactiveError, RegisteredOutput};
use crate::view::ViewRenderer;

pub(crate) fn render_component<C>(
    component: &C,
    renderer: &ViewRenderer,
    key: &[u8],
) -> Result<String, ReactiveError>
where
    C: ReactiveComponent,
{
    let output = RegisteredOutput {
        reply: super::Reply::default(),
        component: C::NAME,
        template: component.template(),
        dom_id: component.dom_id(),
        state: serde_json::to_value(component).map_err(|_| ReactiveError::Internal)?,
        context: component.context(),
    };
    render_registered_output(&output, renderer, key)
}

pub(crate) fn render_registered_output(
    output: &RegisteredOutput,
    renderer: &ViewRenderer,
    key: &[u8],
) -> Result<String, ReactiveError> {
    let token = super::token::sign(&Payload::new(output.component, output.state.clone()), key)
        .map_err(|_| ReactiveError::Internal)?;
    let body = renderer
        .render_to_string(output.template, reactive_context(output.context.clone()))
        .map_err(|error| {
            tracing::error!(
                template = %output.template,
                error = %error,
                "reactive template render failed"
            );
            ReactiveError::Internal
        })?;

    Ok(format!(
        r#"<div id="{}" data-controller="reactive" data-reactive-token="{}" data-reactive-component="{}">{}</div>"#,
        escape_attr(&output.dom_id),
        escape_attr(&token),
        escape_attr(output.component),
        body
    ))
}

fn reactive_context(context: Value) -> Value {
    merge_maps([
        context,
        minijinja::context! {
            on => Value::from_function(on_helper),
        },
    ])
}

fn on_helper(action: String, params: Option<Value>, kwargs: Kwargs) -> Result<Value, minijinja::Error> {
    let event: Option<String> = kwargs.get("event")?;
    kwargs.assert_all_used()?;

    let mut attrs = format!(
        r#"data-action="{}-&gt;reactive#trigger" data-reactive-action="{}""#,
        escape_attr(event.as_deref().unwrap_or("click")),
        escape_attr(&action)
    );
    if let Some(params) = params.filter(|value| !value.is_undefined() && !value.is_none()) {
        let json = serde_json::to_string(&params).map_err(|error| {
            minijinja::Error::new(minijinja::ErrorKind::InvalidOperation, error.to_string())
        })?;
        attrs.push_str(&format!(
            r#" data-reactive-params="{}""#,
            escape_attr(&json)
        ));
    }
    Ok(Value::from_safe_string(attrs))
}

fn escape_attr(input: &str) -> String {
    input
        .replace('&', "&amp;")
        .replace('"', "&quot;")
        .replace('<', "&lt;")
        .replace('>', "&gt;")
}
