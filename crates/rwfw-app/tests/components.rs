//! M8: the dev-only `/components` catalog lists discovered `<x-...>` components.

mod common;

use common::TestApp;

#[tokio::test]
async fn components_catalog_lists_components() {
    let Some(app) = TestApp::spawn().await else {
        eprintln!("skipping components_catalog_lists_components: RWFW_TEST_DATABASE_URL not set");
        return;
    };

    let res = app
        .get("/components")
        .send()
        .await
        .expect("GET /components");
    assert_eq!(res.status().as_u16(), 200);

    let body = res.text().await.unwrap();
    assert!(body.contains("Components"));
    assert!(body.contains("x-flash"), "flash component not listed");
    assert!(body.contains("x-field"), "field component not listed");
}
