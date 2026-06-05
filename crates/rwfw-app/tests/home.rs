//! M3: the home page renders server-side HTML via the View engine (Hotwire),
//! not an Inertia/React payload. DB-gated via `RWFW_TEST_DATABASE_URL`.

mod common;

use common::TestApp;

#[tokio::test]
async fn home_renders_hotwire_html() {
    let Some(app) = TestApp::spawn().await else {
        eprintln!("skipping home_renders_hotwire_html: RWFW_TEST_DATABASE_URL not set");
        return;
    };

    let res = app.get("/home").send().await.expect("GET /home");
    assert_eq!(res.status().as_u16(), 200);

    let body = res.text().await.unwrap();

    // Server-rendered page content.
    assert!(body.contains("Welcome to RWFW"), "missing page title");
    assert!(body.contains("Hotwire + MiniJinja"), "missing card content");

    // Hotwire shell markers.
    assert!(
        body.contains("/vendor/turbo.min.js"),
        "missing Turbo import map"
    );
    assert!(
        body.contains(r#"name="csrf-token""#),
        "missing CSRF meta tag"
    );

    // Sidebar (shared props / modules nav) rendered.
    assert!(body.contains("Modular Framework"), "missing app layout sidebar");

    // No leftover Inertia/React payload.
    assert!(!body.contains("data-page="), "unexpected Inertia payload");
}
