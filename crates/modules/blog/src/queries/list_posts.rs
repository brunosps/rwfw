use rwfw_core::query::{
    OrderDirection, SearchColumn, SearchColumnType, SqlQuery, SqlValue, operators::SearchOperator,
};
use std::collections::HashMap;

pub struct ListPostsQuery {
    pub published_only: bool,
}

impl SqlQuery for ListPostsQuery {
    fn raw_sql(&self) -> &str {
        r#"
        SELECT p.id, p.title, p.body, p.published, p.created_at,
               p.author_id
        FROM blog.posts p
        WHERE p.published = {{published}}
        "#
    }

    fn search_columns(&self) -> Vec<SearchColumn> {
        vec![
            SearchColumn::new("p.title", SearchColumnType::String, SearchOperator::ICont),
            SearchColumn::new("p.body", SearchColumnType::String, SearchOperator::ICont),
        ]
    }

    fn sql_params(&self) -> HashMap<String, SqlValue> {
        HashMap::from([("published".into(), SqlValue::Bool(self.published_only))])
    }

    fn default_order(&self) -> Vec<(String, OrderDirection)> {
        vec![("p.created_at".into(), OrderDirection::Desc)]
    }

    fn default_per_page(&self) -> u32 {
        20
    }
}
