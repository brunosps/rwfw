use super::SqlValue;
use std::collections::HashMap;

/// Replace `{{param_name}}` placeholders in SQL templates with actual values.
pub fn render_template(sql: &str, params: &HashMap<String, SqlValue>) -> String {
    let mut result = sql.to_string();
    for (key, value) in params {
        let placeholder = format!("{{{{{key}}}}}");
        let replacement = sql_value_to_string(value);
        result = result.replace(&placeholder, &replacement);
    }
    result
}

fn sql_value_to_string(value: &SqlValue) -> String {
    match value {
        SqlValue::String(s) => format!("'{}'", s.replace('\'', "''")),
        SqlValue::Int(i) => i.to_string(),
        SqlValue::Float(f) => f.to_string(),
        SqlValue::Bool(b) => if *b { "TRUE" } else { "FALSE" }.to_string(),
        SqlValue::Null => "NULL".to_string(),
    }
}
