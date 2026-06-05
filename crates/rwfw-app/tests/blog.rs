//! M4: blog module served via the View engine (Hotwire) — public list,
//! authenticated create with CSRF, validation re-render (422), and delete via
//! Turbo `data-turbo-method`. DB-gated via `RWFW_TEST_DATABASE_URL`.

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
async fn blog_redirects_to_posts() {
    let app = app_or_skip!("blog_redirects_to_posts");
    let res = app.get("/blog").send().await.expect("GET /blog");
    assert_eq!(res.status().as_u16(), 303);
    assert_eq!(
        res.headers().get("location").unwrap().to_str().unwrap(),
        "/blog/posts"
    );
}

#[tokio::test]
async fn post_list_is_public() {
    let app = app_or_skip!("post_list_is_public");
    let res = app.get("/blog/posts").send().await.expect("GET /blog/posts");
    assert_eq!(res.status().as_u16(), 200);
    let body = res.text().await.unwrap();
    assert!(body.contains("Posts"));
    // Anonymous visitors see the sign-in prompt instead of the New Post button.
    assert!(body.contains("Sign in"));
    assert!(!body.contains(r#"href="/blog/posts/create""#));
}

#[tokio::test]
async fn create_post_then_show() {
    let app = app_or_skip!("create_post_then_show");
    app.login_admin().await;

    let token = app.csrf_token("/blog/posts/create").await;
    let res = app
        .post("/blog/posts")
        .header("x-csrf-token", &token)
        .form(&[
            ("title", "Hotwire Rocks"),
            ("body", "A nice long post body that exceeds ten characters."),
        ])
        .send()
        .await
        .expect("POST /blog/posts");
    assert_eq!(res.status().as_u16(), 303);

    let location = res
        .headers()
        .get("location")
        .unwrap()
        .to_str()
        .unwrap()
        .to_string();
    assert!(location.starts_with("/blog/posts/"));

    let show = app.get(&location).send().await.expect("GET show");
    assert_eq!(show.status().as_u16(), 200);
    let body = show.text().await.unwrap();
    assert!(body.contains("Hotwire Rocks"));
    // Authenticated => edit/delete controls present.
    assert!(body.contains(r#"data-turbo-method="delete""#));
}

#[tokio::test]
async fn create_post_validation_re_renders_422() {
    let app = app_or_skip!("create_post_validation_re_renders_422");
    app.login_admin().await;

    let token = app.csrf_token("/blog/posts/create").await;
    let res = app
        .post("/blog/posts")
        .header("x-csrf-token", &token)
        .form(&[("title", "Kept Title"), ("body", "short")])
        .send()
        .await
        .expect("POST /blog/posts");

    assert_eq!(res.status().as_u16(), 422);
    let body = res.text().await.unwrap();
    assert!(body.contains("at least 10 characters"), "missing body error");
    assert!(body.contains("Kept Title"), "old title not repopulated");
}

#[tokio::test]
async fn sse_stream_broadcasts_new_post() {
    let app = app_or_skip!("sse_stream_broadcasts_new_post");
    app.login_admin().await;

    // Open the SSE stream (subscribes to the broadcaster).
    let stream_res = app
        .get("/blog/posts/stream")
        .send()
        .await
        .expect("GET /blog/posts/stream");
    assert_eq!(stream_res.status().as_u16(), 200);
    let content_type = stream_res
        .headers()
        .get("content-type")
        .and_then(|v| v.to_str().ok())
        .unwrap_or("")
        .to_string();
    assert!(content_type.contains("text/event-stream"), "ct: {content_type}");

    // Create a post; it should be broadcast to the open stream.
    let token = app.csrf_token("/blog/posts/create").await;
    let created = app
        .post("/blog/posts")
        .header("x-csrf-token", &token)
        .form(&[("title", "Live Post"), ("body", "Streamed in real time over SSE.")])
        .send()
        .await
        .expect("create");
    assert_eq!(created.status().as_u16(), 303);

    // Read the broadcast Turbo Stream event from the open SSE connection.
    let mut stream_res = stream_res;
    let mut received = String::new();
    let outcome = tokio::time::timeout(std::time::Duration::from_secs(5), async {
        while let Some(chunk) = stream_res.chunk().await.expect("read chunk") {
            received.push_str(&String::from_utf8_lossy(&chunk));
            if received.contains("turbo-stream") {
                return;
            }
        }
    })
    .await;

    assert!(outcome.is_ok(), "timed out waiting for SSE; received: {received:?}");
    assert!(received.contains("turbo-stream"), "received: {received:?}");
    assert!(received.contains("Live Post"), "received: {received:?}");
}

#[tokio::test]
async fn delete_post() {
    let app = app_or_skip!("delete_post");
    app.login_admin().await;

    let token = app.csrf_token("/blog/posts/create").await;
    let created = app
        .post("/blog/posts")
        .header("x-csrf-token", &token)
        .form(&[
            ("title", "To Delete"),
            ("body", "This post will be deleted soon."),
        ])
        .send()
        .await
        .expect("create");
    let location = created
        .headers()
        .get("location")
        .unwrap()
        .to_str()
        .unwrap()
        .to_string();

    let del = app
        .delete(&location)
        .header("x-csrf-token", &token)
        .send()
        .await
        .expect("DELETE post");
    assert_eq!(del.status().as_u16(), 303);
    assert_eq!(
        del.headers().get("location").unwrap().to_str().unwrap(),
        "/blog/posts"
    );

    let show = app.get(&location).send().await.expect("GET deleted");
    assert_eq!(show.status().as_u16(), 404);
}
