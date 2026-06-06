use serde::{Deserialize, Serialize};

#[derive(Debug, Clone, Copy, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum SearchOperator {
    Eq,
    NotEq,
    Gt,
    Gteq,
    Lt,
    Lteq,
    Cont,
    ICont,
    NotCont,
    NotICont,
    Like,
    ILike,
    Matches,
    In,
    NotIn,
    Start,
    End,
    NotStart,
    NotEnd,
    Null,
    NotNull,
    Empty,
    NotEmpty,
    Present,
    Blank,
    True,
    False,
}

impl SearchOperator {
    /// Convert operator + column + value into a SQL WHERE clause fragment
    pub fn to_sql(
        &self,
        backend: sea_orm::DatabaseBackend,
        column: &str,
        param_index: usize,
    ) -> (String, bool) {
        use sea_orm::DatabaseBackend::Postgres;
        let pi = crate::sql::placeholder(backend, param_index);
        let nocase = if matches!(backend, Postgres) {
            ""
        } else {
            " COLLATE NOCASE"
        };
        match self {
            SearchOperator::Eq => (format!("{column} = {pi}"), true),
            SearchOperator::NotEq => (format!("{column} != {pi}"), true),
            SearchOperator::Gt => (format!("{column} > {pi}"), true),
            SearchOperator::Gteq => (format!("{column} >= {pi}"), true),
            SearchOperator::Lt => (format!("{column} < {pi}"), true),
            SearchOperator::Lteq => (format!("{column} <= {pi}"), true),
            SearchOperator::Cont => (format!("{column} LIKE '%' || {pi} || '%'"), true),
            SearchOperator::ICont => {
                if matches!(backend, Postgres) {
                    (format!("{column} ILIKE '%' || {pi} || '%'"), true)
                } else {
                    (format!("{column} LIKE '%' || {pi} || '%'{nocase}"), true)
                }
            }
            SearchOperator::NotCont => (format!("{column} NOT LIKE '%' || {pi} || '%'"), true),
            SearchOperator::NotICont => {
                if matches!(backend, Postgres) {
                    (format!("{column} NOT ILIKE '%' || {pi} || '%'"), true)
                } else {
                    (format!("{column} NOT LIKE '%' || {pi} || '%'{nocase}"), true)
                }
            }
            SearchOperator::Like => (format!("{column} LIKE {pi}"), true),
            SearchOperator::ILike => {
                if matches!(backend, Postgres) {
                    (format!("{column} ILIKE {pi}"), true)
                } else {
                    (format!("{column} LIKE {pi}{nocase}"), true)
                }
            }
            SearchOperator::Matches => {
                if matches!(backend, Postgres) {
                    (format!("{column} ~ {pi}"), true)
                } else {
                    // SQLite has no built-in regex; fail closed rather than mismatch.
                    tracing::warn!(
                        column,
                        "SearchOperator::Matches (regex ~) is unsupported on SQLite; predicate forced to FALSE"
                    );
                    ("(1 = 0)".to_string(), false)
                }
            }
            SearchOperator::In => {
                if matches!(backend, Postgres) {
                    (format!("{column} = ANY({pi})"), true)
                } else {
                    (format!("{column} IN ({pi})"), true)
                }
            }
            SearchOperator::NotIn => {
                if matches!(backend, Postgres) {
                    (format!("{column} != ALL({pi})"), true)
                } else {
                    (format!("{column} NOT IN ({pi})"), true)
                }
            }
            SearchOperator::Start => (format!("{column} LIKE {pi} || '%'"), true),
            SearchOperator::End => (format!("{column} LIKE '%' || {pi}"), true),
            SearchOperator::NotStart => (format!("{column} NOT LIKE {pi} || '%'"), true),
            SearchOperator::NotEnd => (format!("{column} NOT LIKE '%' || {pi}"), true),
            SearchOperator::Null => (format!("{column} IS NULL"), false),
            SearchOperator::NotNull => (format!("{column} IS NOT NULL"), false),
            SearchOperator::Empty => (format!("({column} IS NULL OR {column} = '')"), false),
            SearchOperator::NotEmpty => {
                (format!("({column} IS NOT NULL AND {column} != '')"), false)
            }
            SearchOperator::Present => {
                (format!("({column} IS NOT NULL AND {column} != '')"), false)
            }
            SearchOperator::Blank => (format!("({column} IS NULL OR {column} = '')"), false),
            SearchOperator::True => (
                format!("{column} = {}", crate::sql::bool_literal(backend, true)),
                false,
            ),
            SearchOperator::False => (
                format!("{column} = {}", crate::sql::bool_literal(backend, false)),
                false,
            ),
        }
    }
}
