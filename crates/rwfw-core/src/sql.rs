//! Backend-aware SQL helpers shared by the migration ledger, the query
//! operators, and raw-SQL call sites. One code path serves both Postgres and
//! SQLite, chosen at runtime from `db.get_database_backend()`.
//!
//! Table names are uniform across backends (prefixed, e.g. `auth_users`,
//! `rwfw_migrations`) so SeaORM entities — whose `table_name` is fixed at
//! compile time — work unchanged on both. The helpers here cover the few places
//! that still genuinely differ: placeholders, boolean literals, case-insensitive
//! matching, and `NOW()`.

use sea_orm::DatabaseBackend;

/// Escape a string for safe inclusion in a single-quoted SQL literal by doubling
/// embedded quotes. The single sanctioned escaper for the few framework-owned
/// raw-SQL sites that cannot use bind parameters (e.g. the migration ledger).
/// Prefer parameterized queries (`Statement::from_sql_and_values`) for anything
/// touching user input.
pub(crate) fn escape_sql(value: &str) -> String {
    value.replace('\'', "''")
}

/// Bind-parameter placeholder: `$N` on Postgres, `?` on SQLite/MySQL.
pub fn placeholder(backend: DatabaseBackend, n: usize) -> String {
    match backend {
        DatabaseBackend::Postgres => format!("${n}"),
        _ => "?".to_string(),
    }
}

/// Boolean literal: `TRUE`/`FALSE` on Postgres, `1`/`0` on SQLite.
pub fn bool_literal(backend: DatabaseBackend, value: bool) -> &'static str {
    match (backend, value) {
        (DatabaseBackend::Sqlite, true) => "1",
        (DatabaseBackend::Sqlite, false) => "0",
        (_, true) => "TRUE",
        (_, false) => "FALSE",
    }
}

/// ISO-8601 (RFC3339, millisecond precision, `+00:00` offset) timestamp literal
/// for "now". Replaces `NOW()` by computing in Rust so one SQL string works on
/// both backends. Postgres parses it into TIMESTAMPTZ; SQLite stores the TEXT
/// and compares it lexically (the fixed-width UTC form is lexically monotonic).
///
/// The `+00:00` offset (not `Z`) matches what SeaORM/sqlx-sqlite writes and
/// parses for `DateTimeWithTimeZone` columns, so values written here via raw SQL
/// round-trip through the ORM (e.g. reading `auth_users` seeded by raw SQL).
pub fn now_iso() -> String {
    chrono::Utc::now().to_rfc3339_opts(chrono::SecondsFormat::Millis, false)
}

/// ISO-8601 timestamp `seconds` in the future. Replaces `NOW() + INTERVAL`.
pub fn now_plus_seconds_iso(seconds: i64) -> String {
    (chrono::Utc::now() + chrono::Duration::seconds(seconds))
        .to_rfc3339_opts(chrono::SecondsFormat::Millis, false)
}

/// Render a UUID as a SQL literal for the active backend.
///
/// Postgres uses the hyphenated text form (`'xxxx-...'`, cast to `uuid`).
/// SQLite uses a 16-byte blob literal (`X'..'`) to match how SeaORM stores
/// `Uuid` columns there (sqlx-sqlite encodes `Uuid` as a BLOB). Without this,
/// raw-SQL text UUIDs and ORM-written blob UUIDs would not compare equal, so
/// foreign keys and joins between the two would silently fail.
pub fn uuid_literal(backend: DatabaseBackend, id: uuid::Uuid) -> String {
    match backend {
        DatabaseBackend::Sqlite => format!("X'{}'", id.simple()),
        _ => format!("'{id}'"),
    }
}
