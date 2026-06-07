use argon2::{
    Argon2,
    password_hash::{PasswordHash, PasswordHasher, PasswordVerifier, SaltString, rand_core::OsRng},
};
use axum::extract::{Request, State};
use axum::http::request::Parts;
use axum::middleware::Next;
use axum::response::{IntoResponse, Redirect, Response};
use crate::sql::escape_sql;
use sea_orm::{ConnectionTrait, DatabaseConnection, Statement};
use serde::Serialize;

pub const SESSION_COOKIE: &str = "rwfw_session";

#[derive(Debug, Clone, Serialize)]
pub struct Permission {
    pub name: &'static str,
    pub description: &'static str,
}

impl Permission {
    pub const fn new(name: &'static str, description: &'static str) -> Self {
        Self { name, description }
    }
}

#[derive(Debug, Clone, Serialize)]
pub struct StoredPermission {
    pub name: String,
    pub description: Option<String>,
}

#[derive(Debug, Clone, Serialize)]
pub struct StoredRole {
    pub name: String,
    pub description: Option<String>,
}

#[derive(Debug, Clone, Serialize)]
pub struct EnsureAdminUserReport {
    pub user_id: uuid::Uuid,
    pub email: String,
    pub created: bool,
    pub password_updated: bool,
    pub role_assigned: bool,
}

#[derive(Debug, Clone)]
pub struct EnsureAdminUserInput<'a> {
    pub name: &'a str,
    pub email: &'a str,
    pub password: &'a str,
    pub update_password: bool,
}

#[derive(Debug, Clone, Serialize)]
pub struct CurrentUser {
    pub id: uuid::Uuid,
    pub name: String,
    pub email: String,
    pub roles: Vec<String>,
    pub permissions: Vec<String>,
}

impl CurrentUser {
    pub fn can(&self, permission: &str) -> bool {
        self.roles.iter().any(|role| role == "admin")
            || self
                .permissions
                .iter()
                .any(|existing| existing == permission)
    }

    pub fn has_role(&self, role: &str) -> bool {
        self.roles.iter().any(|existing| existing == role)
    }
}

pub fn hash_password(password: &str) -> anyhow::Result<String> {
    let salt = SaltString::generate(&mut OsRng);
    let argon2 = Argon2::default();
    let hash = argon2
        .hash_password(password.as_bytes(), &salt)
        .map_err(|e| anyhow::anyhow!("Failed to hash password: {}", e))?;
    Ok(hash.to_string())
}

pub fn verify_password(password: &str, hash: &str) -> anyhow::Result<bool> {
    let parsed =
        PasswordHash::new(hash).map_err(|e| anyhow::anyhow!("Invalid password hash: {}", e))?;
    Ok(Argon2::default()
        .verify_password(password.as_bytes(), &parsed)
        .is_ok())
}

impl<S> axum::extract::FromRequestParts<S> for CurrentUser
where
    S: Send + Sync,
{
    type Rejection = Response;

    async fn from_request_parts(parts: &mut Parts, _state: &S) -> Result<Self, Self::Rejection> {
        parts
            .extensions
            .get::<CurrentUser>()
            .cloned()
            .ok_or_else(|| Redirect::to("/auth/login").into_response())
    }
}

pub async fn create_session(
    db: &DatabaseConnection,
    user_id: uuid::Uuid,
    ttl_seconds: i64,
) -> anyhow::Result<String> {
    let token = generate_session_token();
    let ttl_seconds = ttl_seconds.max(60);
    let expires_at = crate::sql::now_plus_seconds_iso(ttl_seconds);
    let uid = crate::sql::uuid_literal(db.get_database_backend(), user_id);
    let sql = format!(
        "INSERT INTO auth_sessions (user_id, token, expires_at) \
         VALUES ({uid}, '{}', '{}')",
        escape_sql(&token),
        escape_sql(&expires_at)
    );

    db.execute_unprepared(&sql).await?;
    Ok(token)
}

pub async fn delete_session(db: &DatabaseConnection, token: &str) -> anyhow::Result<()> {
    let sql = format!(
        "DELETE FROM auth_sessions WHERE token = '{}'",
        escape_sql(token)
    );
    db.execute_unprepared(&sql).await?;
    Ok(())
}

pub async fn find_current_user(
    db: &DatabaseConnection,
    token: &str,
) -> anyhow::Result<Option<CurrentUser>> {
    let now = crate::sql::now_iso();
    let sql = format!(
        "SELECT u.id, u.name, u.email \
         FROM auth_sessions s \
         JOIN auth_users u ON u.id = s.user_id \
         WHERE s.token = '{}' AND s.expires_at > '{}' \
         LIMIT 1",
        escape_sql(token),
        escape_sql(&now)
    );

    let Some(row) = db
        .query_one(Statement::from_string(db.get_database_backend(), sql))
        .await?
    else {
        return Ok(None);
    };

    let id: uuid::Uuid = row.try_get("", "id")?;
    let roles = load_roles(db, id).await?;
    let permissions = load_permissions(db, id).await?;

    Ok(Some(CurrentUser {
        id,
        name: row.try_get("", "name")?,
        email: row.try_get("", "email")?,
        roles,
        permissions,
    }))
}

pub async fn user_has_permission(
    db: &DatabaseConnection,
    user_id: uuid::Uuid,
    permission: &str,
) -> anyhow::Result<bool> {
    let uid = crate::sql::uuid_literal(db.get_database_backend(), user_id);
    let sql = format!(
        "SELECT 1 \
         FROM auth_user_roles ur \
         JOIN auth_role_permissions rp ON rp.role_id = ur.role_id \
         JOIN auth_permissions p ON p.id = rp.permission_id \
         WHERE ur.user_id = {uid} AND p.name = '{}' \
         LIMIT 1",
        escape_sql(permission)
    );

    Ok(db
        .query_one(Statement::from_string(db.get_database_backend(), sql))
        .await?
        .is_some())
}

pub async fn ensure_permissions(
    db: &DatabaseConnection,
    module: &str,
    permissions: &[Permission],
) -> anyhow::Result<()> {
    for permission in permissions {
        let description = if permission.description.is_empty() {
            format!("{} permission from {module}", permission.name)
        } else {
            permission.description.to_string()
        };
        let now = crate::sql::now_iso();
        let sql = format!(
            "INSERT INTO auth_permissions (name, description) \
             VALUES ('{}', '{}') \
             ON CONFLICT (name) DO UPDATE SET \
                 description = EXCLUDED.description, \
                 updated_at = '{}'",
            escape_sql(permission.name),
            escape_sql(&description),
            escape_sql(&now)
        );
        db.execute_unprepared(&sql).await?;
    }

    Ok(())
}

pub async fn ensure_role(
    db: &DatabaseConnection,
    role: &str,
    description: &str,
) -> anyhow::Result<()> {
    let now = crate::sql::now_iso();
    let sql = format!(
        "INSERT INTO auth_roles (name, description) \
         VALUES ('{}', '{}') \
         ON CONFLICT (name) DO UPDATE SET \
             description = EXCLUDED.description, \
             updated_at = '{}'",
        escape_sql(role),
        escape_sql(description),
        escape_sql(&now)
    );
    db.execute_unprepared(&sql).await?;
    Ok(())
}

pub async fn rbac_tables_available(db: &DatabaseConnection) -> anyhow::Result<bool> {
    Ok(table_exists(db, "auth", "roles").await?
        && table_exists(db, "auth", "permissions").await?
        && table_exists(db, "auth", "user_roles").await?
        && table_exists(db, "auth", "role_permissions").await?)
}

pub async fn ensure_admin_user(
    db: &DatabaseConnection,
    input: EnsureAdminUserInput<'_>,
) -> anyhow::Result<EnsureAdminUserReport> {
    let name = input.name.trim();
    let email = input.email.trim();

    if name.is_empty() {
        anyhow::bail!("Admin name is required");
    }
    if email.is_empty() {
        anyhow::bail!("Admin email is required");
    }
    if input.password.len() < 8 {
        anyhow::bail!("Admin password must be at least 8 characters");
    }

    ensure_role(db, "admin", "Full framework administrator").await?;

    let password_hash = hash_password(input.password)?;
    let existing = find_user_id_by_email(db, email).await?;

    let now = crate::sql::now_iso();
    let backend = db.get_database_backend();
    let (user_id, created, password_updated) = if let Some(user_id) = existing {
        let uid = crate::sql::uuid_literal(backend, user_id);
        let sql = if input.update_password {
            format!(
                "UPDATE auth_users \
                 SET name = '{}', password_hash = '{}', updated_at = '{}' \
                 WHERE id = {uid}",
                escape_sql(name),
                escape_sql(&password_hash),
                escape_sql(&now)
            )
        } else {
            format!(
                "UPDATE auth_users SET name = '{}', updated_at = '{}' WHERE id = {uid}",
                escape_sql(name),
                escape_sql(&now)
            )
        };
        db.execute_unprepared(&sql).await?;
        (user_id, false, input.update_password)
    } else {
        let user_id = uuid::Uuid::new_v4();
        let uid = crate::sql::uuid_literal(backend, user_id);
        let sql = format!(
            "INSERT INTO auth_users (id, name, email, password_hash, created_at, updated_at) \
             VALUES ({uid}, '{}', '{}', '{}', '{}', '{}')",
            escape_sql(name),
            escape_sql(email),
            escape_sql(&password_hash),
            escape_sql(&now),
            escape_sql(&now)
        );
        db.execute_unprepared(&sql).await?;
        (user_id, true, true)
    };

    let had_role = user_has_role(db, user_id, "admin").await?;
    assign_role(db, user_id, "admin").await?;

    Ok(EnsureAdminUserReport {
        user_id,
        email: email.to_string(),
        created,
        password_updated,
        role_assigned: !had_role,
    })
}

pub async fn list_permissions(db: &DatabaseConnection) -> anyhow::Result<Vec<StoredPermission>> {
    let rows = db
        .query_all(Statement::from_string(
            db.get_database_backend(),
            "SELECT name, description FROM auth_permissions ORDER BY name".to_string(),
        ))
        .await?;

    rows.into_iter()
        .map(|row| {
            Ok(StoredPermission {
                name: row.try_get("", "name")?,
                description: row.try_get("", "description")?,
            })
        })
        .collect()
}

pub async fn list_roles(db: &DatabaseConnection) -> anyhow::Result<Vec<StoredRole>> {
    let rows = db
        .query_all(Statement::from_string(
            db.get_database_backend(),
            "SELECT name, description FROM auth_roles ORDER BY name".to_string(),
        ))
        .await?;

    rows.into_iter()
        .map(|row| {
            Ok(StoredRole {
                name: row.try_get("", "name")?,
                description: row.try_get("", "description")?,
            })
        })
        .collect()
}

pub async fn grant_permission_to_role(
    db: &DatabaseConnection,
    role: &str,
    permission: &str,
) -> anyhow::Result<()> {
    if !role_exists(db, role).await? {
        anyhow::bail!("Role not found: {role}");
    }
    if !permission_exists(db, permission).await? {
        anyhow::bail!("Permission not found: {permission}");
    }

    let sql = format!(
        "INSERT INTO auth_role_permissions (role_id, permission_id) \
         SELECT r.id, p.id \
         FROM auth_roles r \
         JOIN auth_permissions p ON p.name = '{}' \
         WHERE r.name = '{}' \
         ON CONFLICT DO NOTHING",
        escape_sql(permission),
        escape_sql(role)
    );
    db.execute_unprepared(&sql).await?;
    Ok(())
}

pub async fn assign_role(
    db: &DatabaseConnection,
    user_id: uuid::Uuid,
    role: &str,
) -> anyhow::Result<()> {
    let uid = crate::sql::uuid_literal(db.get_database_backend(), user_id);
    let sql = format!(
        "INSERT INTO auth_user_roles (user_id, role_id) \
         SELECT {uid}, id FROM auth_roles WHERE name = '{}' \
         ON CONFLICT DO NOTHING",
        escape_sql(role)
    );
    db.execute_unprepared(&sql).await?;
    Ok(())
}

pub async fn assign_role_by_email(
    db: &DatabaseConnection,
    email: &str,
    role: &str,
) -> anyhow::Result<()> {
    if !role_exists(db, role).await? {
        anyhow::bail!("Role not found: {role}");
    }
    if !user_email_exists(db, email).await? {
        anyhow::bail!("User not found: {email}");
    }

    let sql = format!(
        "INSERT INTO auth_user_roles (user_id, role_id) \
         SELECT u.id, r.id \
         FROM auth_users u \
         JOIN auth_roles r ON r.name = '{}' \
         WHERE u.email = '{}' \
         ON CONFLICT DO NOTHING",
        escape_sql(role),
        escape_sql(email)
    );
    db.execute_unprepared(&sql).await?;
    Ok(())
}

pub async fn user_count(db: &DatabaseConnection) -> anyhow::Result<u64> {
    let row = db
        .query_one(Statement::from_string(
            db.get_database_backend(),
            "SELECT CAST(COUNT(*) AS BIGINT) AS count FROM auth_users".to_string(),
        ))
        .await?;

    let count: i64 = row
        .ok_or_else(|| anyhow::anyhow!("Failed to count auth users"))?
        .try_get("", "count")?;
    Ok(count as u64)
}

pub async fn require_permission(
    State(state): State<crate::app::AppState>,
    mut request: Request,
    next: Next,
    permission: &'static str,
) -> Response {
    let current_user = resolve_current_user(&state.db, &mut request).await;

    let Some(user) = current_user else {
        return Redirect::to("/auth/login").into_response();
    };

    if user.roles.iter().any(|role| role == "admin")
        || user
            .permissions
            .iter()
            .any(|existing| existing == permission)
    {
        request.extensions_mut().insert(user);
        return next.run(request).await;
    }

    (
        axum::http::StatusCode::FORBIDDEN,
        axum::Json(serde_json::json!({ "error": "Forbidden" })),
    )
        .into_response()
}

pub async fn require_auth(
    State(state): State<crate::app::AppState>,
    mut request: Request,
    next: Next,
) -> Response {
    let current_user = resolve_current_user(&state.db, &mut request).await;

    let Some(user) = current_user else {
        return Redirect::to("/auth/login").into_response();
    };

    request.extensions_mut().insert(user);
    next.run(request).await
}

pub fn extract_session_token(headers: &axum::http::HeaderMap) -> Option<String> {
    let cookie_header = headers.get(axum::http::header::COOKIE)?.to_str().ok()?;
    parse_cookie(cookie_header, SESSION_COOKIE).map(str::to_string)
}

pub fn session_cookie(token: &str, ttl_seconds: i64, secure: bool) -> String {
    let secure = if secure { "; Secure" } else { "" };
    format!(
        "{SESSION_COOKIE}={}; Path=/; HttpOnly; SameSite=Lax; Max-Age={}{}",
        token,
        ttl_seconds.max(60),
        secure
    )
}

pub fn expired_session_cookie() -> String {
    format!("{SESSION_COOKIE}=; Path=/; HttpOnly; SameSite=Lax; Max-Age=0")
}

pub fn append_set_cookie(response: &mut axum::response::Response, cookie: String) {
    match axum::http::HeaderValue::from_str(&cookie) {
        Ok(value) => {
            response
                .headers_mut()
                .append(axum::http::header::SET_COOKIE, value);
        }
        Err(error) => {
            tracing::warn!(error = %error, "Failed to create Set-Cookie header");
        }
    }
}

async fn load_roles(db: &DatabaseConnection, user_id: uuid::Uuid) -> anyhow::Result<Vec<String>> {
    let uid = crate::sql::uuid_literal(db.get_database_backend(), user_id);
    let sql = format!(
        "SELECT r.name \
         FROM auth_roles r \
         JOIN auth_user_roles ur ON ur.role_id = r.id \
         WHERE ur.user_id = {uid} \
         ORDER BY r.name"
    );

    let rows = db
        .query_all(Statement::from_string(db.get_database_backend(), sql))
        .await?;

    rows.into_iter()
        .map(|row| row.try_get("", "name").map_err(Into::into))
        .collect()
}

async fn role_exists(db: &DatabaseConnection, role: &str) -> anyhow::Result<bool> {
    exists(
        db,
        &format!(
            "SELECT 1 FROM auth_roles WHERE name = '{}' LIMIT 1",
            escape_sql(role)
        ),
    )
    .await
}

async fn user_has_role(
    db: &DatabaseConnection,
    user_id: uuid::Uuid,
    role: &str,
) -> anyhow::Result<bool> {
    let uid = crate::sql::uuid_literal(db.get_database_backend(), user_id);
    exists(
        db,
        &format!(
            "SELECT 1 \
             FROM auth_user_roles ur \
             JOIN auth_roles r ON r.id = ur.role_id \
             WHERE ur.user_id = {uid} AND r.name = '{}' \
             LIMIT 1",
            escape_sql(role)
        ),
    )
    .await
}

async fn table_exists(db: &DatabaseConnection, schema: &str, table: &str) -> anyhow::Result<bool> {
    // Tables are uniformly named `<schema>_<table>` on both backends.
    let physical = format!("{schema}_{table}");
    let sql = match db.get_database_backend() {
        sea_orm::DatabaseBackend::Sqlite => format!(
            "SELECT 1 FROM sqlite_master \
             WHERE type = 'table' AND name = '{}' \
             LIMIT 1",
            escape_sql(&physical)
        ),
        _ => format!(
            "SELECT 1 FROM information_schema.tables \
             WHERE table_name = '{}' \
             LIMIT 1",
            escape_sql(&physical)
        ),
    };
    exists(db, &sql).await
}

async fn permission_exists(db: &DatabaseConnection, permission: &str) -> anyhow::Result<bool> {
    exists(
        db,
        &format!(
            "SELECT 1 FROM auth_permissions WHERE name = '{}' LIMIT 1",
            escape_sql(permission)
        ),
    )
    .await
}

async fn find_user_id_by_email(
    db: &DatabaseConnection,
    email: &str,
) -> anyhow::Result<Option<uuid::Uuid>> {
    let sql = format!(
        "SELECT id FROM auth_users WHERE email = '{}' LIMIT 1",
        escape_sql(email)
    );
    let row = db
        .query_one(Statement::from_string(db.get_database_backend(), sql))
        .await?;

    Ok(row.map(|row| row.try_get("", "id")).transpose()?)
}

async fn user_email_exists(db: &DatabaseConnection, email: &str) -> anyhow::Result<bool> {
    exists(
        db,
        &format!(
            "SELECT 1 FROM auth_users WHERE email = '{}' LIMIT 1",
            escape_sql(email)
        ),
    )
    .await
}

async fn exists(db: &DatabaseConnection, sql: &str) -> anyhow::Result<bool> {
    Ok(db
        .query_one(Statement::from_string(
            db.get_database_backend(),
            sql.to_string(),
        ))
        .await?
        .is_some())
}

async fn resolve_current_user(
    db: &DatabaseConnection,
    request: &mut Request,
) -> Option<CurrentUser> {
    if let Some(user) = request.extensions().get::<CurrentUser>().cloned() {
        return Some(user);
    }

    let Some(token) = extract_session_token(request.headers()) else {
        return None;
    };

    match find_current_user(db, &token).await {
        Ok(user) => user,
        Err(error) => {
            tracing::warn!(error = %error, "Failed to resolve current user");
            None
        }
    }
}

async fn load_permissions(
    db: &DatabaseConnection,
    user_id: uuid::Uuid,
) -> anyhow::Result<Vec<String>> {
    let uid = crate::sql::uuid_literal(db.get_database_backend(), user_id);
    let sql = format!(
        "SELECT DISTINCT p.name \
         FROM auth_permissions p \
         JOIN auth_role_permissions rp ON rp.permission_id = p.id \
         JOIN auth_user_roles ur ON ur.role_id = rp.role_id \
         WHERE ur.user_id = {uid} \
         ORDER BY p.name"
    );

    let rows = db
        .query_all(Statement::from_string(db.get_database_backend(), sql))
        .await?;

    rows.into_iter()
        .map(|row| row.try_get("", "name").map_err(Into::into))
        .collect()
}

fn generate_session_token() -> String {
    let first = uuid::Uuid::new_v4().simple();
    let second = uuid::Uuid::new_v4().simple();
    format!("{first}{second}")
}

fn parse_cookie<'a>(cookie_header: &'a str, name: &str) -> Option<&'a str> {
    cookie_header.split(';').find_map(|cookie| {
        let (key, value) = cookie.trim().split_once('=')?;
        (key == name).then_some(value)
    })
}
