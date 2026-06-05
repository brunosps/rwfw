//! Helpers for deserializing HTML form (`application/x-www-form-urlencoded`)
//! payloads, where the wire format differs from JSON.
//!
//! The most common gap is the checkbox: a checked box submits the field with a
//! value (`"on"` by default), while an *unchecked* box omits the field entirely.
//! Pair [`checkbox`] with `#[serde(default)]` so an absent field deserializes to
//! `false`:
//!
//! ```ignore
//! #[derive(serde::Deserialize)]
//! struct Input {
//!     #[serde(default, deserialize_with = "rwfw_core::forms::checkbox")]
//!     active: bool,
//! }
//! ```

use serde::{Deserialize, Deserializer};

/// Deserialize an HTML checkbox into a `bool`.
///
/// Truthy values: `"on"`, `"true"`, `"1"`, `"yes"`, `"checked"` (any case).
/// Everything else (including `"off"`, `"false"`, `""`) is `false`. Combine with
/// `#[serde(default)]` so an omitted (unchecked) field becomes `false`.
pub fn checkbox<'de, D>(deserializer: D) -> Result<bool, D::Error>
where
    D: Deserializer<'de>,
{
    let raw = Option::<String>::deserialize(deserializer)?;
    Ok(match raw {
        Some(value) => matches!(
            value.trim().to_ascii_lowercase().as_str(),
            "on" | "true" | "1" | "yes" | "checked"
        ),
        None => false,
    })
}

#[cfg(test)]
mod tests {
    use super::*;

    #[derive(serde::Deserialize)]
    struct Form {
        #[serde(default, deserialize_with = "checkbox")]
        active: bool,
    }

    fn parse(query: &str) -> bool {
        serde_urlencoded::from_str::<Form>(query).unwrap().active
    }

    #[test]
    fn checked_box_is_true() {
        assert!(parse("active=on"));
        assert!(parse("active=true"));
        assert!(parse("active=1"));
    }

    #[test]
    fn unchecked_box_is_false() {
        // Absent field (unchecked checkbox) defaults to false.
        assert!(!parse(""));
        assert!(!parse("active=off"));
        assert!(!parse("active="));
    }
}
