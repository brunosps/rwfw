use rwfw_core::query::{SearchColumn, SqlQuery, SqlValue};
use std::collections::HashMap;

pub struct GetPostQuery {
    pub post_id: i32,
}

impl SqlQuery for GetPostQuery {
    fn raw_sql(&self) -> &str {
        r#"
        SELECT p.id, p.title, p.body, p.published, p.created_at, p.updated_at,
               p.author_id
        FROM blog.posts p
        WHERE p.id = {{post_id}}
        "#
    }

    fn search_columns(&self) -> Vec<SearchColumn> {
        vec![]
    }

    fn sql_params(&self) -> HashMap<String, SqlValue> {
        HashMap::from([("post_id".into(), SqlValue::Int(self.post_id as i64))])
    }
}
