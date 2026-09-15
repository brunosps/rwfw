use crate::sql::escape_sql;
use argon2::{
    Argon2,
    password_hash::{PasswordHash, PasswordHasher, PasswordVerifier, SaltString, rand_core::OsRng},
};
use axum::extract::{Request, State};
use axum::http::request::Parts;
use axum::middleware::Next;
use axum::response::{IntoResponse, Redirect, Response};
use sea_orm::{ConnectionTrait, DatabaseConnection, Statement, Value};
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
    oidc_provider: Option<&str>,
    id_token: Option<&str>,
) -> anyhow::Result<String> {
    let token = generate_session_token();
    let ttl_seconds = ttl_seconds.max(60);
    let expires_at = crate::sql::now_plus_seconds_iso(ttl_seconds);
    let backend = db.get_database_backend();
    // user_id/token/oidc_provider/id_token are bound; expires_at is a
    // framework-generated ISO literal (binding a string into a Postgres
    // `timestamptz` column would be rejected). The OIDC columns are NULL for
    // password logins and carry the provider name + raw id_token for SSO logins.
    let sql = format!(
        "INSERT INTO auth_sessions (user_id, token, expires_at, oidc_provider, id_token) \
         VALUES ({}, {}, '{}', {}, {})",
        crate::sql::placeholder(backend, 1),
        crate::sql::placeholder(backend, 2),
        escape_sql(&expires_at),
        crate::sql::placeholder(backend, 3),
        crate::sql::placeholder(backend, 4),
    );

    db.execute(Statement::from_sql_and_values(
        backend,
        sql,
        [
            user_id.into(),
            token.clone().into(),
            Value::String(oidc_provider.map(|s| Box::new(s.to_string()))),
            Value::String(id_token.map(|s| Box::new(s.to_string()))),
        ],
    ))
    .await?;
    Ok(token)
}

/// Read the OIDC provider name + raw id_token stored for a session. Both are
/// present only for SSO logins; returns `None` for password sessions. Used to
/// drive RP-initiated logout against the IdP.
pub async fn session_oidc(
    db: &DatabaseConnection,
    token: &str,
) -> anyhow::Result<Option<(String, String)>> {
    let backend = db.get_database_backend();
    let sql = format!(
        "SELECT oidc_provider, id_token FROM auth_sessions WHERE token = {} LIMIT 1",
        crate::sql::placeholder(backend, 1),
    );
    let Some(row) = db
        .query_one(Statement::from_sql_and_values(
            backend,
            sql,
            [token.to_owned().into()],
        ))
        .await?
    else {
        return Ok(None);
    };
    let provider: Option<String> = row.try_get("", "oidc_provider")?;
    let id_token: Option<String> = row.try_get("", "id_token")?;
    Ok(match (provider, id_token) {
        (Some(provider), Some(id_token)) => Some((provider, id_token)),
        _ => None,
    })
}

pub async fn delete_session(db: &DatabaseConnection, token: &str) -> anyhow::Result<()> {
    let backend = db.get_database_backend();
    let sql = format!(
        "DELETE FROM auth_sessions WHERE token = {}",
        crate::sql::placeholder(backend, 1)
    );
    db.execute(Statement::from_sql_and_values(
        backend,
        sql,
        [token.to_owned().into()],
    ))
    .await?;
    Ok(())
}

pub async fn find_current_user(
    db: &DatabaseConnection,
    token: &str,
) -> anyhow::Result<Option<CurrentUser>> {
    let now = crate::sql::now_iso();
    let backend = db.get_database_backend();
    // The session token comes from the request cookie (attacker-controlled), so
    // it is bound as a parameter; `now` is a framework ISO literal.
    let sql = format!(
        "SELECT u.id, u.name, u.email \
         FROM auth_sessions s \
         JOIN auth_users u ON u.id = s.user_id \
         WHERE s.token = {} AND s.expires_at > '{}' \
         LIMIT 1",
        crate::sql::placeholder(backend, 1),
        escape_sql(&now)
    );

    let Some(row) = db
        .query_one(Statement::from_sql_and_values(
            backend,
            sql,
            [token.to_owned().into()],
        ))
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

/// Carrega o `CurrentUser` a partir do id, sem passar por sessao.
///
/// Existe para quem ja autenticou por outro meio e tem o id em maos: token de
/// dispositivo, chave de API, job de fundo. Sem isso, cada consumidor reescreve
/// SELECT contra `auth_users`, `auth_roles` e `auth_user_roles` — tabelas que
/// pertencem ao mod-auth. Foi o que aconteceu no neodrive, cujo `mod-sync`
/// reimplementou `load_roles` linha a linha.
///
/// Duplicar esse SQL nao e so repeticao: no dia em que o formato de papeis mudar
/// (papel por escopo, permissao negativa, soft-delete de usuario), o consumidor
/// que copiou continua respondendo com o modelo velho e ninguem percebe, porque
/// ele responde — apenas errado.
///
/// Devolve `None` se o usuario nao existe. Autenticar e trabalho de quem chama:
/// esta funcao nao valida credencial nenhuma, so materializa o usuario.
pub async fn find_user_by_id(
    db: &DatabaseConnection,
    user_id: uuid::Uuid,
) -> anyhow::Result<Option<CurrentUser>> {
    let backend = db.get_database_backend();
    let sql = format!(
        "SELECT u.id, u.name, u.email \
         FROM auth_users u \
         WHERE u.id = {} \
         LIMIT 1",
        crate::sql::placeholder(backend, 1),
    );

    let Some(row) = db
        .query_one(Statement::from_sql_and_values(
            backend,
            sql,
            vec![user_id.into()],
        ))
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
    let backend = db.get_database_backend();
    let sql = format!(
        "SELECT 1 \
         FROM auth_user_roles ur \
         JOIN auth_role_permissions rp ON rp.role_id = ur.role_id \
         JOIN auth_permissions p ON p.id = rp.permission_id \
         WHERE ur.user_id = {} AND p.name = {} \
         LIMIT 1",
        crate::sql::placeholder(backend, 1),
        crate::sql::placeholder(backend, 2),
    );

    exists(db, sql, vec![user_id.into(), permission.into()]).await
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
        let backend = db.get_database_backend();
        let sql = format!(
            "INSERT INTO auth_permissions (name, description) \
             VALUES ({}, {}) \
             ON CONFLICT (name) DO UPDATE SET \
                 description = EXCLUDED.description, \
                 updated_at = '{}'",
            crate::sql::placeholder(backend, 1),
            crate::sql::placeholder(backend, 2),
            escape_sql(&now)
        );
        exec_params(db, sql, vec![permission.name.into(), description.into()]).await?;
    }

    Ok(())
}

pub async fn ensure_role(
    db: &DatabaseConnection,
    role: &str,
    description: &str,
) -> anyhow::Result<()> {
    let now = crate::sql::now_iso();
    let backend = db.get_database_backend();
    let sql = format!(
        "INSERT INTO auth_roles (name, description) \
         VALUES ({}, {}) \
         ON CONFLICT (name) DO UPDATE SET \
             description = EXCLUDED.description, \
             updated_at = '{}'",
        crate::sql::placeholder(backend, 1),
        crate::sql::placeholder(backend, 2),
        escape_sql(&now)
    );
    exec_params(db, sql, vec![role.into(), description.into()]).await?;
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
        let (sql, values): (String, Vec<Value>) = if input.update_password {
            (
                format!(
                    "UPDATE auth_users \
                     SET name = {}, password_hash = {}, updated_at = '{}' \
                     WHERE id = {}",
                    crate::sql::placeholder(backend, 1),
                    crate::sql::placeholder(backend, 2),
                    escape_sql(&now),
                    crate::sql::placeholder(backend, 3),
                ),
                vec![name.into(), password_hash.clone().into(), user_id.into()],
            )
        } else {
            (
                format!(
                    "UPDATE auth_users SET name = {}, updated_at = '{}' WHERE id = {}",
                    crate::sql::placeholder(backend, 1),
                    escape_sql(&now),
                    crate::sql::placeholder(backend, 2),
                ),
                vec![name.into(), user_id.into()],
            )
        };
        exec_params(db, sql, values).await?;
        (user_id, false, input.update_password)
    } else {
        let user_id = uuid::Uuid::new_v4();
        let sql = format!(
            "INSERT INTO auth_users (id, name, email, password_hash, created_at, updated_at) \
             VALUES ({}, {}, {}, {}, '{}', '{}')",
            crate::sql::placeholder(backend, 1),
            crate::sql::placeholder(backend, 2),
            crate::sql::placeholder(backend, 3),
            crate::sql::placeholder(backend, 4),
            escape_sql(&now),
            escape_sql(&now),
        );
        exec_params(
            db,
            sql,
            vec![
                user_id.into(),
                name.into(),
                email.into(),
                password_hash.clone().into(),
            ],
        )
        .await?;
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

    let backend = db.get_database_backend();
    let sql = format!(
        "INSERT INTO auth_role_permissions (role_id, permission_id) \
         SELECT r.id, p.id \
         FROM auth_roles r \
         JOIN auth_permissions p ON p.name = {} \
         WHERE r.name = {} \
         ON CONFLICT DO NOTHING",
        crate::sql::placeholder(backend, 1),
        crate::sql::placeholder(backend, 2),
    );
    exec_params(db, sql, vec![permission.into(), role.into()]).await?;
    Ok(())
}

pub async fn assign_role(
    db: &DatabaseConnection,
    user_id: uuid::Uuid,
    role: &str,
) -> anyhow::Result<()> {
    let backend = db.get_database_backend();
    let sql = format!(
        "INSERT INTO auth_user_roles (user_id, role_id) \
         SELECT {}, id FROM auth_roles WHERE name = {} \
         ON CONFLICT DO NOTHING",
        crate::sql::placeholder(backend, 1),
        crate::sql::placeholder(backend, 2),
    );
    exec_params(db, sql, vec![user_id.into(), role.into()]).await?;
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

    let backend = db.get_database_backend();
    let sql = format!(
        "INSERT INTO auth_user_roles (user_id, role_id) \
         SELECT u.id, r.id \
         FROM auth_users u \
         JOIN auth_roles r ON r.name = {} \
         WHERE u.email = {} \
         ON CONFLICT DO NOTHING",
        crate::sql::placeholder(backend, 1),
        crate::sql::placeholder(backend, 2),
    );
    exec_params(db, sql, vec![role.into(), email.into()]).await?;
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
    let backend = db.get_database_backend();
    let sql = format!(
        "SELECT r.name \
         FROM auth_roles r \
         JOIN auth_user_roles ur ON ur.role_id = r.id \
         WHERE ur.user_id = {} \
         ORDER BY r.name",
        crate::sql::placeholder(backend, 1),
    );

    let rows = db
        .query_all(Statement::from_sql_and_values(
            backend,
            sql,
            vec![user_id.into()],
        ))
        .await?;

    rows.into_iter()
        .map(|row| row.try_get("", "name").map_err(Into::into))
        .collect()
}

async fn role_exists(db: &DatabaseConnection, role: &str) -> anyhow::Result<bool> {
    let backend = db.get_database_backend();
    let sql = format!(
        "SELECT 1 FROM auth_roles WHERE name = {} LIMIT 1",
        crate::sql::placeholder(backend, 1),
    );
    exists(db, sql, vec![role.into()]).await
}

async fn user_has_role(
    db: &DatabaseConnection,
    user_id: uuid::Uuid,
    role: &str,
) -> anyhow::Result<bool> {
    let backend = db.get_database_backend();
    let sql = format!(
        "SELECT 1 \
         FROM auth_user_roles ur \
         JOIN auth_roles r ON r.id = ur.role_id \
         WHERE ur.user_id = {} AND r.name = {} \
         LIMIT 1",
        crate::sql::placeholder(backend, 1),
        crate::sql::placeholder(backend, 2),
    );
    exists(db, sql, vec![user_id.into(), role.into()]).await
}

async fn table_exists(db: &DatabaseConnection, schema: &str, table: &str) -> anyhow::Result<bool> {
    // Tables are uniformly named `<schema>_<table>` on both backends.
    let physical = format!("{schema}_{table}");
    let backend = db.get_database_backend();
    let sql = match backend {
        sea_orm::DatabaseBackend::Sqlite => format!(
            "SELECT 1 FROM sqlite_master \
             WHERE type = 'table' AND name = {} \
             LIMIT 1",
            crate::sql::placeholder(backend, 1),
        ),
        _ => format!(
            "SELECT 1 FROM information_schema.tables \
             WHERE table_name = {} \
             LIMIT 1",
            crate::sql::placeholder(backend, 1),
        ),
    };
    exists(db, sql, vec![physical.into()]).await
}

async fn permission_exists(db: &DatabaseConnection, permission: &str) -> anyhow::Result<bool> {
    let backend = db.get_database_backend();
    let sql = format!(
        "SELECT 1 FROM auth_permissions WHERE name = {} LIMIT 1",
        crate::sql::placeholder(backend, 1),
    );
    exists(db, sql, vec![permission.into()]).await
}

async fn find_user_id_by_email(
    db: &DatabaseConnection,
    email: &str,
) -> anyhow::Result<Option<uuid::Uuid>> {
    let backend = db.get_database_backend();
    let sql = format!(
        "SELECT id FROM auth_users WHERE email = {} LIMIT 1",
        crate::sql::placeholder(backend, 1),
    );
    let row = db
        .query_one(Statement::from_sql_and_values(
            backend,
            sql,
            vec![email.into()],
        ))
        .await?;

    Ok(row.map(|row| row.try_get("", "id")).transpose()?)
}

async fn user_email_exists(db: &DatabaseConnection, email: &str) -> anyhow::Result<bool> {
    let backend = db.get_database_backend();
    let sql = format!(
        "SELECT 1 FROM auth_users WHERE email = {} LIMIT 1",
        crate::sql::placeholder(backend, 1),
    );
    exists(db, sql, vec![email.into()]).await
}

/// Run a parameterized non-query statement. The single audited binding path for
/// the raw-SQL call sites that interpolate dynamic values: callers build the SQL
/// with `crate::sql::placeholder` and pass the values here (never string-format
/// user input into the query).
async fn exec_params(
    db: &DatabaseConnection,
    sql: String,
    values: Vec<Value>,
) -> anyhow::Result<()> {
    db.execute(Statement::from_sql_and_values(
        db.get_database_backend(),
        sql,
        values,
    ))
    .await?;
    Ok(())
}

/// Parameterized existence check (`SELECT 1 ... LIMIT 1`).
async fn exists(db: &DatabaseConnection, sql: String, values: Vec<Value>) -> anyhow::Result<bool> {
    Ok(db
        .query_one(Statement::from_sql_and_values(
            db.get_database_backend(),
            sql,
            values,
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
    let backend = db.get_database_backend();
    let sql = format!(
        "SELECT DISTINCT p.name \
         FROM auth_permissions p \
         JOIN auth_role_permissions rp ON rp.permission_id = p.id \
         JOIN auth_user_roles ur ON ur.role_id = rp.role_id \
         WHERE ur.user_id = {} \
         ORDER BY p.name",
        crate::sql::placeholder(backend, 1),
    );

    let rows = db
        .query_all(Statement::from_sql_and_values(
            backend,
            sql,
            vec![user_id.into()],
        ))
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

#[cfg(test)]
mod find_user_by_id_tests {
    use super::*;
    use sea_orm::{ConnectionTrait, Database};

    /// Schema copiado das migrations do mod-auth (variantes `.sqlite.sql`).
    /// Se elas mudarem e este teste continuar verde, e sinal de que a copia
    /// envelheceu — exatamente o problema que `find_user_by_id` existe para
    /// evitar do lado dos consumidores.
    const SCHEMA: &[&str] = &[
        "CREATE TABLE auth_users (id TEXT PRIMARY KEY NOT NULL, name TEXT NOT NULL, \
         email TEXT NOT NULL UNIQUE, password_hash TEXT)",
        "CREATE TABLE auth_roles (id TEXT PRIMARY KEY NOT NULL, name TEXT NOT NULL UNIQUE)",
        "CREATE TABLE auth_permissions (id TEXT PRIMARY KEY NOT NULL, name TEXT NOT NULL UNIQUE)",
        "CREATE TABLE auth_user_roles (user_id TEXT NOT NULL, role_id TEXT NOT NULL, \
         PRIMARY KEY (user_id, role_id))",
        "CREATE TABLE auth_role_permissions (role_id TEXT NOT NULL, permission_id TEXT NOT NULL, \
         PRIMARY KEY (role_id, permission_id))",
    ];

    /// Escreve o id como a produção escreve.
    ///
    /// No SQLite o `sqlx` codifica `Uuid` como BLOB, então um id inserido como
    /// texto NÃO casa com o mesmo id ligado como parâmetro — a consulta devolve
    /// vazio, sem erro. `crate::sql::uuid_literal` é o helper que resolve isso, e
    /// semear sem ele faz o teste reprovar código correto.
    fn id_sql(db: &DatabaseConnection, id: uuid::Uuid) -> String {
        crate::sql::uuid_literal(db.get_database_backend(), id)
    }

    async fn banco_com_usuario() -> (DatabaseConnection, uuid::Uuid) {
        let db = Database::connect("sqlite::memory:").await.expect("sqlite");
        for ddl in SCHEMA {
            db.execute_unprepared(ddl).await.expect("ddl");
        }

        let user_id = uuid::Uuid::new_v4();
        let role_id = uuid::Uuid::new_v4();
        let outro_role = uuid::Uuid::new_v4();
        let perm_a = uuid::Uuid::new_v4();
        let perm_b = uuid::Uuid::new_v4();

        for sql in [
            format!(
                "INSERT INTO auth_users (id, name, email) VALUES ({}, 'Bruno', 'bruno@example.com')",
                id_sql(&db, user_id)
            ),
            format!("INSERT INTO auth_roles (id, name) VALUES ({}, 'editor')", id_sql(&db, role_id)),
            format!("INSERT INTO auth_roles (id, name) VALUES ({}, 'admin')", id_sql(&db, outro_role)),
            format!("INSERT INTO auth_permissions (id, name) VALUES ({}, 'sync:read')", id_sql(&db, perm_a)),
            format!("INSERT INTO auth_permissions (id, name) VALUES ({}, 'sync:write')", id_sql(&db, perm_b)),
            format!("INSERT INTO auth_user_roles (user_id, role_id) VALUES ({}, {})", id_sql(&db, user_id), id_sql(&db, role_id)),
            format!("INSERT INTO auth_user_roles (user_id, role_id) VALUES ({}, {})", id_sql(&db, user_id), id_sql(&db, outro_role)),
            format!(
                "INSERT INTO auth_role_permissions (role_id, permission_id) VALUES ({}, {})",
                id_sql(&db, role_id), id_sql(&db, perm_a)
            ),
            format!(
                "INSERT INTO auth_role_permissions (role_id, permission_id) VALUES ({}, {})",
                id_sql(&db, outro_role), id_sql(&db, perm_b)
            ),
            // Mesma permissao por DOIS papeis: o DISTINCT do load_permissions
            // precisa colapsar, senao o consumidor recebe duplicata.
            format!(
                "INSERT INTO auth_role_permissions (role_id, permission_id) VALUES ({}, {})",
                id_sql(&db, outro_role), id_sql(&db, perm_a)
            ),
        ] {
            db.execute_unprepared(&sql).await.expect("seed");
        }

        (db, user_id)
    }

    #[tokio::test]
    async fn materializa_usuario_com_papeis_e_permissoes() {
        let (db, user_id) = banco_com_usuario().await;

        let user = find_user_by_id(&db, user_id)
            .await
            .expect("consulta")
            .expect("usuario existe");

        assert_eq!(user.id, user_id);
        assert_eq!(user.email, "bruno@example.com");
        assert_eq!(
            user.roles,
            vec!["admin".to_string(), "editor".to_string()],
            "papeis vem ordenados por nome, como em find_current_user"
        );
        assert_eq!(
            user.permissions,
            vec!["sync:read".to_string(), "sync:write".to_string()],
            "permissao concedida por dois papeis nao pode aparecer duplicada"
        );
    }

    #[tokio::test]
    async fn usuario_inexistente_devolve_none_em_vez_de_erro() {
        let (db, _) = banco_com_usuario().await;

        let achado = find_user_by_id(&db, uuid::Uuid::new_v4())
            .await
            .expect("id desconhecido e resposta valida, nao falha");

        assert!(achado.is_none());
    }

    #[tokio::test]
    async fn usuario_sem_papel_vem_com_listas_vazias_e_sem_poder() {
        let (db, _) = banco_com_usuario().await;
        let orfao = uuid::Uuid::new_v4();
        db.execute_unprepared(&format!(
            "INSERT INTO auth_users (id, name, email) VALUES ({}, 'Sem Papel', 'orfao@example.com')",
            id_sql(&db, orfao)
        ))
        .await
        .expect("seed");

        let user = find_user_by_id(&db, orfao)
            .await
            .expect("consulta")
            .expect("usuario existe");

        assert!(user.roles.is_empty());
        assert!(user.permissions.is_empty());
        assert!(
            !user.can("sync:read"),
            "sem papel nao pode nada — fail-closed"
        );
    }

    /// O contrato que torna `find_user_by_id` substituivel pelo caminho de sessao:
    /// os dois precisam materializar o MESMO usuario. Se divergirem, um consumidor
    /// autenticado por token de dispositivo teria autorizacao diferente de um
    /// autenticado por cookie.
    #[tokio::test]
    async fn concorda_com_find_current_user_para_o_mesmo_usuario() {
        let (db, user_id) = banco_com_usuario().await;
        db.execute_unprepared(
            "CREATE TABLE auth_sessions (token TEXT PRIMARY KEY NOT NULL, user_id TEXT NOT NULL, \
             expires_at TEXT NOT NULL)",
        )
        .await
        .expect("ddl");
        db.execute_unprepared(&format!(
            "INSERT INTO auth_sessions (token, user_id, expires_at) \
             VALUES ('tok', {}, '2999-01-01T00:00:00.000Z')",
            id_sql(&db, user_id)
        ))
        .await
        .expect("seed");

        let por_id = find_user_by_id(&db, user_id).await.expect("id").expect("achou");
        let por_sessao = find_current_user(&db, "tok").await.expect("sessao").expect("achou");

        assert_eq!(por_id.id, por_sessao.id);
        assert_eq!(por_id.name, por_sessao.name);
        assert_eq!(por_id.email, por_sessao.email);
        assert_eq!(por_id.roles, por_sessao.roles);
        assert_eq!(por_id.permissions, por_sessao.permissions);
    }
}
