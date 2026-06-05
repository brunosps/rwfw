//! M0 smoke: the app boots and `/health` answers. DB-gated via
//! `RWFW_TEST_DATABASE_URL` (skips cleanly when unset).

mod common;

use common::TestApp;

#[tokio::test]
async fn health_returns_ok() {
    let Some(app) = TestApp::spawn().await else {
        eprintln!("skipping health_returns_ok: RWFW_TEST_DATABASE_URL not set");
        return;
    };

    let res = app.get("/health").send().await.expect("request /health");
    assert_eq!(res.status().as_u16(), 200);

    let body: serde_json::Value = res.json().await.expect("parse json");
    assert_eq!(body["status"], "ok");
}
