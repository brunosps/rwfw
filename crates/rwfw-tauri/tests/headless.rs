//! Headless boot test (no GUI, no external services): boot
//! `serve_in_background` against a temp SQLite DB, hit `/health`, then shut down.
//! Runs under a plain `cargo test -p rwfw-tauri` (default features, no Tauri).

use rwfw_core::config::AppConfig;

#[tokio::test]
async fn serves_health_then_shuts_down() {
    let path = tempfile::NamedTempFile::new()
        .expect("temp db file")
        .into_temp_path();
    let db_url = format!("sqlite://{}?mode=rwc", path.to_string_lossy());
    let config = AppConfig::for_test(&db_url).expect("test config");

    rwfw_app::run_migrations(&config).await.expect("migrate");
    let handle = rwfw_tauri::serve_in_background(config)
        .await
        .expect("boot server");

    assert_ne!(handle.addr().port(), 0, "an ephemeral port was bound");

    let base = handle.base_url();
    let res = reqwest::get(format!("{base}/health"))
        .await
        .expect("GET /health");
    assert_eq!(res.status().as_u16(), 200);
    let body: serde_json::Value = res.json().await.expect("json body");
    assert_eq!(body["status"], "ok");

    handle.shutdown().await;
}
