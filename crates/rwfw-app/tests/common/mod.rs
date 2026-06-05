//! Shared integration-test harness: spawns the real RWFW app on an ephemeral
//! port and exposes a cookie-aware `reqwest` client.
//!
//! Integration tests that need a database are gated behind the
//! `RWFW_TEST_DATABASE_URL` environment variable. When it is unset,
//! `TestApp::spawn()` returns `None` and the test should skip — this keeps a
//! plain `cargo test` (no database) green.

#![allow(dead_code)]

use rwfw_core::config::AppConfig;

pub struct TestApp {
    pub base_url: String,
    pub client: reqwest::Client,
}

impl TestApp {
    /// Spawn the full application against the test database.
    /// Returns `None` (skip) when `RWFW_TEST_DATABASE_URL` is not set.
    pub async fn spawn() -> Option<TestApp> {
        let db_url = std::env::var("RWFW_TEST_DATABASE_URL").ok()?;

        let config = AppConfig::for_test(&db_url).expect("build test config");
        let router = rwfw_app::build_router(config)
            .await
            .expect("build router");

        let listener = tokio::net::TcpListener::bind("127.0.0.1:0")
            .await
            .expect("bind ephemeral port");
        let addr = listener.local_addr().expect("local addr");

        tokio::spawn(async move {
            axum::serve(listener, router).await.expect("serve test app");
        });

        let client = reqwest::Client::builder()
            .cookie_store(true)
            // Assert redirects (303 etc.) explicitly instead of following them.
            .redirect(reqwest::redirect::Policy::none())
            .build()
            .expect("build reqwest client");

        Some(TestApp {
            base_url: format!("http://{addr}"),
            client,
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
}
