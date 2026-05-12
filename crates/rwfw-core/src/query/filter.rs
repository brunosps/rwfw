use super::SqlValue;
use super::operators::SearchOperator;
use serde::Deserialize;

#[derive(Debug, Clone, Deserialize)]
pub struct FilterCondition {
    pub op: SearchOperator,
    pub value: Option<SqlValue>,
}
