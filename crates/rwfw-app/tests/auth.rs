//! M5: auth module served via the View engine (Hotwire) — login/register forms,
//! validation re-render, session cookies, logout. DB-gated.

mod common;

use common::TestApp;

macro_rules! app_or_skip {
    ($name:literal) => {
        match TestApp::spawn().await {
            Some(app) => app,
            None => {
                eprintln!(concat!("skipping ", $name, ": RWFW_TEST_DATABASE_URL not set"));
                return;
            }
        }
    };
}

#[tokio::test]
async fn login_page_renders() {
    let app = app_or_skip!("login_page_renders");
    let res = app.get("/auth/login").send().await.expect("GET /auth/login");
    assert_eq!(res.status().as_u16(), 200);
    let body = res.text().await.unwrap();
    assert!(body.contains("Sign In"));
    assert!(body.contains(r#"name="csrf-token""#));
    assert!(body.contains(r#"action="/auth/login""#));
}

#[tokio::test]
async fn login_succeeds_and_sets_session() {
    let app = app_or_skip!("login_succeeds_and_sets_session");
    app.seed_user("login-ok@test.local", "supersecret123").await;

    let token = app.csrf_token("/auth/login").await;
    let res = app
        .post("/auth/login")
        .header("x-csrf-token", &token)
        .form(&[("email", "login-ok@test.local"), ("password", "supersecret123")])
        .send()
        .await
        .expect("POST /auth/login");
    assert_eq!(res.status().as_u16(), 303);
    assert_eq!(res.headers().get("location").unwrap().to_str().unwrap(), "/home");

    // Session cookie now lets us see the authenticated layout.
    let home = app.get("/home").send().await.expect("GET /home");
    let body = home.text().await.unwrap();
    assert!(body.contains("Sign out"), "expected authenticated layout");
}

#[tokio::test]
async fn login_invalid_credentials_re_renders_422() {
    let app = app_or_skip!("login_invalid_credentials_re_renders_422");
    app.seed_user("login-bad@test.local", "supersecret123").await;

    let token = app.csrf_token("/auth/login").await;
    let res = app
        .post("/auth/login")
        .header("x-csrf-token", &token)
        .form(&[("email", "login-bad@test.local"), ("password", "wrongpassword")])
        .send()
        .await
        .expect("POST /auth/login");
    assert_eq!(res.status().as_u16(), 422);
    let body = res.text().await.unwrap();
    assert!(body.contains("Invalid email or password"));
    // Old email is repopulated.
    assert!(body.contains("login-bad@test.local"));
}

#[tokio::test]
async fn register_creates_user_and_logs_in() {
    let app = app_or_skip!("register_creates_user_and_logs_in");

    let token = app.csrf_token("/auth/register").await;
    let res = app
        .post("/auth/register")
        .header("x-csrf-token", &token)
        .form(&[
            ("name", "New Person"),
            ("email", "register-new@test.local"),
            ("password", "supersecret123"),
            ("password_confirmation", "supersecret123"),
        ])
        .send()
        .await
        .expect("POST /auth/register");
    assert_eq!(res.status().as_u16(), 303);
    assert_eq!(res.headers().get("location").unwrap().to_str().unwrap(), "/home");
}

#[tokio::test]
async fn register_password_mismatch_re_renders_422() {
    let app = app_or_skip!("register_password_mismatch_re_renders_422");

    let token = app.csrf_token("/auth/register").await;
    let res = app
        .post("/auth/register")
        .header("x-csrf-token", &token)
        .form(&[
            ("name", "Mismatch"),
            ("email", "register-mismatch@test.local"),
            ("password", "supersecret123"),
            ("password_confirmation", "different123"),
        ])
        .send()
        .await
        .expect("POST /auth/register");
    assert_eq!(res.status().as_u16(), 422);
    let body = res.text().await.unwrap();
    assert!(body.contains("Passwords do not match"));
}

#[tokio::test]
async fn logout_clears_session() {
    let app = app_or_skip!("logout_clears_session");
    app.login_admin().await;

    // Confirm we start authenticated.
    let before = app.get("/home").send().await.unwrap().text().await.unwrap();
    assert!(before.contains("Sign out"));

    let token = app.csrf_token("/home").await;
    let res = app
        .post("/auth/logout")
        .header("x-csrf-token", &token)
        .form(&[("_csrf", token.as_str())])
        .send()
        .await
        .expect("POST /auth/logout");
    assert_eq!(res.status().as_u16(), 303);
    assert_eq!(res.headers().get("location").unwrap().to_str().unwrap(), "/auth/login");
}
