//! M2: static serving of vendored JS / compiled CSS, and CSRF double-submit
//! enforcement on form-encoded posts. DB-gated via `RWFW_TEST_DATABASE_URL`.

mod common;

use common::TestApp;

#[tokio::test]
async fn serves_vendored_turbo() {
    let Some(app) = TestApp::spawn().await else {
        eprintln!("skipping serves_vendored_turbo: RWFW_TEST_DATABASE_URL not set");
        return;
    };

    let res = app
        .get("/vendor/turbo.min.js")
        .send()
        .await
        .expect("GET /vendor/turbo.min.js");
    assert_eq!(res.status().as_u16(), 200);

    let content_type = res
        .headers()
        .get("content-type")
        .and_then(|v| v.to_str().ok())
        .unwrap_or("")
        .to_string();
    assert!(
        content_type.contains("javascript"),
        "unexpected content-type: {content_type}"
    );

    let body = res.text().await.unwrap();
    assert!(body.contains("Turbo 8"));
}

#[tokio::test]
async fn serves_compiled_css() {
    let Some(app) = TestApp::spawn().await else {
        eprintln!("skipping serves_compiled_css: RWFW_TEST_DATABASE_URL not set");
        return;
    };

    let res = app
        .get("/assets/app.css")
        .send()
        .await
        .expect("GET /assets/app.css");
    assert_eq!(res.status().as_u16(), 200);

    let content_type = res
        .headers()
        .get("content-type")
        .and_then(|v| v.to_str().ok())
        .unwrap_or("")
        .to_string();
    assert!(content_type.contains("css"), "unexpected content-type: {content_type}");
}

#[tokio::test]
async fn rejects_form_post_without_csrf_token() {
    let Some(app) = TestApp::spawn().await else {
        eprintln!("skipping rejects_form_post_without_csrf_token: RWFW_TEST_DATABASE_URL not set");
        return;
    };

    // Form-encoded POST with no X-CSRF-Token header is rejected before any
    // handler runs.
    let res = app
        .post("/blog/posts")
        .header("content-type", "application/x-www-form-urlencoded")
        .body("title=x&body=y")
        .send()
        .await
        .expect("POST /blog/posts");
    assert_eq!(res.status().as_u16(), 403);
}
