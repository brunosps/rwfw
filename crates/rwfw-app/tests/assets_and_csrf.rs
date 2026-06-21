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
    assert!(
        content_type.contains("css"),
        "unexpected content-type: {content_type}"
    );
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

#[tokio::test]
async fn emits_security_headers() {
    let Some(app) = TestApp::spawn().await else {
        eprintln!("skipping emits_security_headers: RWFW_TEST_DATABASE_URL not set");
        return;
    };

    // Default SecurityConfig (test harness has no `security:` section) enables
    // response headers; they apply to every route, including /health.
    let res = app.get("/health").send().await.expect("GET /health");
    let header = |name: &str| {
        res.headers()
            .get(name)
            .and_then(|v| v.to_str().ok())
            .unwrap_or("")
            .to_string()
    };
    assert_eq!(header("x-content-type-options"), "nosniff");
    assert_eq!(header("x-frame-options"), "SAMEORIGIN");
    assert!(
        header("content-security-policy").contains("default-src 'self'"),
        "missing/unexpected CSP: {}",
        header("content-security-policy")
    );
    // HSTS is off by default (no TLS assumption in dev/tests).
    assert!(
        res.headers().get("strict-transport-security").is_none(),
        "HSTS should be off by default"
    );
}
