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
    pub fn to_sql(&self, column: &str, param_index: usize) -> (String, bool) {
        let pi = format!("${}", param_index);
        match self {
            SearchOperator::Eq => (format!("{column} = {pi}"), true),
            SearchOperator::NotEq => (format!("{column} != {pi}"), true),
            SearchOperator::Gt => (format!("{column} > {pi}"), true),
            SearchOperator::Gteq => (format!("{column} >= {pi}"), true),
            SearchOperator::Lt => (format!("{column} < {pi}"), true),
            SearchOperator::Lteq => (format!("{column} <= {pi}"), true),
            SearchOperator::Cont => (format!("{column} LIKE '%' || {pi} || '%'"), true),
            SearchOperator::ICont => (format!("{column} ILIKE '%' || {pi} || '%'"), true),
            SearchOperator::NotCont => (format!("{column} NOT LIKE '%' || {pi} || '%'"), true),
            SearchOperator::NotICont => (format!("{column} NOT ILIKE '%' || {pi} || '%'"), true),
            SearchOperator::Like => (format!("{column} LIKE {pi}"), true),
            SearchOperator::ILike => (format!("{column} ILIKE {pi}"), true),
            SearchOperator::Matches => (format!("{column} ~ {pi}"), true),
            SearchOperator::In => (format!("{column} = ANY({pi})"), true),
            SearchOperator::NotIn => (format!("{column} != ALL({pi})"), true),
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
            SearchOperator::True => (format!("{column} = TRUE"), false),
            SearchOperator::False => (format!("{column} = FALSE"), false),
        }
    }
}
