// Re-export validator for convenience
pub use validator::Validate;
pub use validator::ValidationErrors;

use crate::error::AppError;
use std::collections::HashMap;

/// Convert validator::ValidationErrors into our AppError::Validation format
pub fn into_app_error(errors: ValidationErrors) -> AppError {
    let mut error_map: HashMap<String, Vec<String>> = HashMap::new();

    for (field, field_errors) in errors.field_errors() {
        let messages: Vec<String> = field_errors
            .iter()
            .map(|e| {
                e.message
                    .as_ref()
                    .map(|m| m.to_string())
                    .unwrap_or_else(|| format!("Invalid value for {field}"))
            })
            .collect();
        error_map.insert(field.to_string(), messages);
    }

    AppError::Validation(error_map)
}

/// Reduce a field -> messages map to field -> first message, for rendering
/// inline form errors in templates (`{{ errors.title }}`).
pub fn first_messages(errors: HashMap<String, Vec<String>>) -> HashMap<String, String> {
    errors
        .into_iter()
        .map(|(field, messages)| {
            (
                field,
                messages
                    .into_iter()
                    .next()
                    .unwrap_or_else(|| "Invalid value".to_string()),
            )
        })
        .collect()
}
