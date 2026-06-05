//! Shared integration-test harness: spawns the real RWFW app on an ephemeral
//! port and exposes a cookie-aware `reqwest` client.
//!
//! DB-gated via `RWFW_TEST_DATABASE_URL`; when unset, `TestApp::spawn()` returns
//! `None` and tests should skip — keeping a plain `cargo test` (no DB) green.

#![allow(dead_code)]

use std::sync::Arc;

use rwfw_core::config::AppConfig;
use sea_orm::DatabaseConnection;

pub struct TestApp {
    pub base_url: String,
    pub client: reqwest::Client,
    pub jar: Arc<reqwest::cookie::Jar>,
    pub db: DatabaseConnection,
}

impl TestApp {
    /// Spawn the full application against the test database (running migrations
    /// first). Returns `None` (skip) when `RWFW_TEST_DATABASE_URL` is unset.
    pub async fn spawn() -> Option<TestApp> {
        let db_url = std::env::var("RWFW_TEST_DATABASE_URL").ok()?;

        let config = AppConfig::for_test(&db_url).expect("build test config");

        // Run migrations exactly once per test process (tests run in parallel;
        // concurrent migrations on a fresh DB would race).
        static MIGRATED: tokio::sync::OnceCell<()> = tokio::sync::OnceCell::const_new();
        MIGRATED
            .get_or_init(|| {
                let config = config.clone();
                async move {
                    rwfw_app::run_migrations(&config)
                        .await
                        .expect("run migrations");
                }
            })
            .await;

        let db = rwfw_core::db::connect(&config).await.expect("connect db");
        let router = rwfw_app::build_router(config).await.expect("build router");

        let listener = tokio::net::TcpListener::bind("127.0.0.1:0")
            .await
            .expect("bind ephemeral port");
        let addr = listener.local_addr().expect("local addr");

        tokio::spawn(async move {
            axum::serve(listener, router).await.expect("serve test app");
        });

        let jar = Arc::new(reqwest::cookie::Jar::default());
        let client = reqwest::Client::builder()
            .cookie_provider(jar.clone())
            .redirect(reqwest::redirect::Policy::none())
            .build()
            .expect("build reqwest client");

        Some(TestApp {
            base_url: format!("http://{addr}"),
            client,
            jar,
            db,
        })
    }

    pub fn url(&self, path: &str) -> String {
        format!("{}{}", self.base_url, path)
    }

    pub fn get(&self, path: &str) -> reqwest::RequestBuilder {
        self.client.get(self.url(path))
    }

    pub fn post(&self, path: &str) -> reqwest::RequestBuilder {
        self.client.post(self.url(path))
    }

    pub fn delete(&self, path: &str) -> reqwest::RequestBuilder {
        self.client.delete(self.url(path))
    }

    /// Seed an admin user (role "admin" => all permissions) and inject its
    /// session cookie into the jar. Returns the admin user id.
    pub async fn login_admin(&self) -> uuid::Uuid {
        use rwfw_core::auth::{EnsureAdminUserInput, create_session, ensure_admin_user};
        use std::sync::atomic::{AtomicU64, Ordering};

        // Unique email per call so parallel tests don't race on the user insert.
        static SEQ: AtomicU64 = AtomicU64::new(0);
        let email = format!("admin-{}@test.local", SEQ.fetch_add(1, Ordering::Relaxed));

        let report = ensure_admin_user(
            &self.db,
            EnsureAdminUserInput {
                name: "Admin",
                email: &email,
                password: "rwfw-admin-123",
                update_password: true,
            },
        )
        .await
        .expect("ensure admin user");

        let token = create_session(&self.db, report.user_id, 3600)
            .await
            .expect("create session");

        let url: reqwest::Url = self.base_url.parse().unwrap();
        self.jar
            .add_cookie_str(&format!("rwfw_session={token}; Path=/"), &url);

        report.user_id
    }

    /// GET `path` and extract the CSRF token from its `<meta name="csrf-token">`,
    /// for replaying as the `X-CSRF-Token` header on a subsequent form post
    /// (what Turbo does automatically in the browser).
    pub async fn csrf_token(&self, path: &str) -> String {
        let res = self.get(path).send().await.expect("GET for csrf token");
        let status = res.status();
        let body = res.text().await.unwrap();
        extract_csrf(&body).unwrap_or_else(|| {
            panic!(
                "no csrf-token meta in {path} (status {status}); body starts: {:?}",
                body.chars().take(160).collect::<String>()
            )
        })
    }
}

fn extract_csrf(html: &str) -> Option<String> {
    let marker = "name=\"csrf-token\" content=\"";
    let start = html.find(marker)? + marker.len();
    let rest = &html[start..];
    let end = rest.find('"')?;
    Some(rest[..end].to_string())
}
