mod common;

use common::TestApp;

#[tokio::test]
async fn reactive_counter_roundtrip_and_error_contracts() {
    let Some(app) = TestApp::spawn().await else {
        eprintln!("skipping reactive test: database unavailable");
        return;
    };

    let page = app.get("/demo/counter").send().await.unwrap();
    assert_eq!(page.status(), reqwest::StatusCode::OK);
    let page = page.text().await.unwrap();
    assert!(page.contains("data-controller=\"reactive\""));
    assert!(page.contains("data-reactive-component=\"counter\""));

    let token = extract_attr(&page, "data-reactive-token").expect("initial token");
    let increment = post_action(&app, &token, "increment", serde_json::json!({})).await;
    assert_eq!(increment.status, reqwest::StatusCode::OK);
    assert_eq!(extract_count(&increment.body), Some(1));
    let token = extract_attr(&increment.body, "data-reactive-token").expect("rolling token");
    assert_ne!(token, extract_attr(&page, "data-reactive-token").unwrap());

    let set = post_action(
        &app,
        &token,
        "set",
        serde_json::json!({ "value": 7, "ignored": true }),
    )
    .await;
    assert_eq!(set.status, reqwest::StatusCode::OK);
    assert_eq!(extract_count(&set.body), Some(7));
    let token = extract_attr(&set.body, "data-reactive-token").expect("set token");

    let wrong_type = post_action(&app, &token, "set", serde_json::json!({ "value": "bad" })).await;
    assert_eq!(wrong_type.status, reqwest::StatusCode::UNPROCESSABLE_ENTITY);

    let unknown = post_action(&app, &token, "missing", serde_json::json!({})).await;
    assert_eq!(unknown.status, reqwest::StatusCode::FORBIDDEN);

    let tampered = post_action(&app, &format!("{token}x"), "increment", serde_json::json!({})).await;
    assert_eq!(tampered.status, reqwest::StatusCode::BAD_REQUEST);
}

struct ActionResponse {
    status: reqwest::StatusCode,
    body: String,
}

async fn post_action(
    app: &TestApp,
    token: &str,
    act: &str,
    params: serde_json::Value,
) -> ActionResponse {
    let response = app
        .post("/__rwfw/reactive/actions")
        .json(&serde_json::json!({ "token": token, "act": act, "params": params }))
        .send()
        .await
        .unwrap();
    let status = response.status();
    let body = response.text().await.unwrap();
    ActionResponse { status, body }
}

fn extract_attr(html: &str, attr: &str) -> Option<String> {
    let marker = format!("{attr}=\"");
    let start = html.find(&marker)? + marker.len();
    let rest = &html[start..];
    let end = rest.find('"')?;
    Some(rest[..end].to_string())
}

fn extract_count(html: &str) -> Option<i64> {
    let marker = "data-testid=\"count\"";
    let start = html.find(marker)?;
    let rest = &html[start..];
    let start = rest.find('>')? + 1;
    let rest = &rest[start..];
    let end = rest.find('<')?;
    rest[..end].trim().parse().ok()
}
