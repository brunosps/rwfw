use rwfw_core::query::{SearchColumn, SearchColumnType, SqlQuery, operators::SearchOperator};
use std::collections::HashMap;

pub struct GetUserQuery {
    pub user_id: sea_orm::prelude::Uuid,
}

impl SqlQuery for GetUserQuery {
    fn raw_sql(&self) -> &str {
        "SELECT id, name, email, created_at, updated_at FROM auth.users WHERE id = {{user_id}}"
    }

    fn search_columns(&self) -> Vec<SearchColumn> {
        vec![
            SearchColumn::new("name", SearchColumnType::String, SearchOperator::ICont),
            SearchColumn::new("email", SearchColumnType::String, SearchOperator::ICont),
        ]
    }

    fn sql_params(&self) -> HashMap<String, rwfw_core::query::SqlValue> {
        HashMap::from([(
            "user_id".into(),
            rwfw_core::query::SqlValue::String(self.user_id.to_string()),
        )])
    }
}
