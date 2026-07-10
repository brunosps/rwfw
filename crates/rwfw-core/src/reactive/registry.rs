use serde::Serialize;
use serde::de::DeserializeOwned;

use super::{Ctx, ReactiveComponent, ReactiveError, Reply};

/// Result of dispatching a registered component action.
#[derive(Debug)]
pub struct RegisteredOutput {
    pub reply: Reply,
    pub component: &'static str,
    pub template: &'static str,
    pub dom_id: String,
    pub state: serde_json::Value,
    pub context: minijinja::Value,
}

/// Inventory entry for a reactive component.
pub struct ReactiveRegistration {
    pub name: &'static str,
    pub dispatch: fn(
        serde_json::Value,
        &str,
        serde_json::Value,
        &Ctx<'_>,
    ) -> Result<RegisteredOutput, ReactiveError>,
}

impl ReactiveRegistration {
    pub const fn new<T>() -> Self
    where
        T: ReactiveComponent,
    {
        Self {
            name: T::NAME,
            dispatch: dispatch_registered::<T>,
        }
    }
}

inventory::collect!(ReactiveRegistration);

/// Find a registered reactive component by token component name.
pub fn find(name: &str) -> Option<&'static ReactiveRegistration> {
    inventory::iter::<ReactiveRegistration>
        .into_iter()
        .find(|registration| registration.name == name)
}

/// True when any component registered with inventory.
pub fn has_registered_components() -> bool {
    inventory::iter::<ReactiveRegistration>.into_iter().next().is_some()
}

fn dispatch_registered<T>(
    state: serde_json::Value,
    action: &str,
    params: serde_json::Value,
    ctx: &Ctx<'_>,
) -> Result<RegisteredOutput, ReactiveError>
where
    T: ReactiveComponent + DeserializeOwned + Serialize,
{
    let mut component: T =
        serde_json::from_value(state).map_err(|error| ReactiveError::Invalid(error.to_string()))?;
    let reply = component.dispatch(action, params, ctx)?;
    let state =
        serde_json::to_value(&component).map_err(|_| ReactiveError::Internal)?;
    Ok(RegisteredOutput {
        reply,
        component: T::NAME,
        template: component.template(),
        dom_id: component.dom_id(),
        context: component.context(),
        state,
    })
}
