pub mod filter;
pub mod operators;
pub mod paginator;
pub mod template;

use serde::{Deserialize, Serialize};
use std::collections::HashMap;

use operators::SearchOperator;

#[derive(Debug, Clone, Serialize)]
pub struct SearchColumn {
    pub column: String,
    pub column_type: SearchColumnType,
    pub default_operator: SearchOperator,
}

impl SearchColumn {
    pub fn new(
        column: &str,
        column_type: SearchColumnType,
        default_operator: SearchOperator,
    ) -> Self {
        Self {
            column: column.to_string(),
            column_type,
            default_operator,
        }
    }
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub enum SearchColumnType {
    String,
    Integer,
    Float,
    Boolean,
    Date,
    DateTime,
}

#[derive(Debug, Clone, Copy, Serialize, Deserialize)]
pub enum OrderDirection {
    Asc,
    Desc,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub enum SqlValue {
    String(String),
    Int(i64),
    Float(f64),
    Bool(bool),
    Null,
}

pub trait SqlQuery: Send + Sync {
    fn raw_sql(&self) -> &str;
    fn search_columns(&self) -> Vec<SearchColumn>;
    fn default_order(&self) -> Vec<(String, OrderDirection)> {
        vec![]
    }
    fn sql_params(&self) -> HashMap<String, SqlValue> {
        HashMap::new()
    }
    fn default_per_page(&self) -> u32 {
        25
    }
    fn primary_key(&self) -> &str {
        "id"
    }
}

#[derive(Debug, Deserialize)]
pub struct QueryParams {
    pub page: Option<u32>,
    pub per_page: Option<u32>,
    pub search: Option<String>,
    pub filter: Option<HashMap<String, filter::FilterCondition>>,
    pub order: Option<HashMap<String, OrderDirection>>,
}

#[derive(Debug, Serialize)]
pub struct ListOfRecords<T: Serialize> {
    pub records: Vec<T>,
    pub meta: PaginationMeta,
}

#[derive(Debug, Clone, Serialize)]
pub struct PaginationMeta {
    pub page: u32,
    pub per_page: u32,
    pub total_pages: u32,
    pub total_rows: u64,
}
