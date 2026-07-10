//! Browser end-to-end test (npm-free) — drives a real headless Chromium via
//! `chromiumoxide` (no Node/Playwright) to validate the JS behaviour the
//! HTTP-level tests can't: that the vendored Turbo/Stimulus load via the import
//! map, and that Turbo Drive navigates without a full page reload.
//!
//! Gated behind `RWFW_E2E=1` (and a usable Chromium); skips cleanly otherwise so
//! a plain `cargo test` stays green without a browser.

mod common;

use std::time::Duration;

use chromiumoxide::{Browser, BrowserConfig};
use common::TestApp;
use futures::StreamExt;

#[tokio::test]
async fn turbo_drive_navigates_without_full_reload() {
    if std::env::var("RWFW_E2E").ok().as_deref() != Some("1") {
        eprintln!("skipping e2e: set RWFW_E2E=1 (and have Chromium) to run browser tests");
        return;
    }
    let Some(app) = TestApp::spawn().await else {
        eprintln!("skipping e2e: RWFW_TEST_DATABASE_URL not set");
        return;
    };

    let config = BrowserConfig::builder()
        .no_sandbox()
        .arg("--disable-dev-shm-usage")
        .build()
        .expect("browser config");

    let (mut browser, mut handler) = match Browser::launch(config).await {
        Ok(pair) => pair,
        Err(error) => {
            eprintln!("skipping e2e: could not launch Chromium: {error}");
            return;
        }
    };
    let handler_task = tokio::spawn(async move { while handler.next().await.is_some() {} });

    let page = browser.new_page("about:blank").await.expect("new page");
    page.goto(app.url("/home")).await.expect("goto /home");
    page.wait_for_navigation().await.expect("home loaded");

    // The vendored ESM loaded via the import map (the core "Hotwire without npm" claim).
    let turbo_loaded: bool = page
        .evaluate("typeof window.Turbo !== 'undefined'")
        .await
        .expect("eval turbo")
        .into_value()
        .expect("turbo bool");
    let stimulus_loaded: bool = page
        .evaluate("typeof window.Stimulus !== 'undefined'")
        .await
        .expect("eval stimulus")
        .into_value()
        .expect("stimulus bool");
    assert!(turbo_loaded, "Turbo did not load from the import map");
    assert!(stimulus_loaded, "Stimulus did not start");

    // Tag the JS context; a Turbo Drive visit swaps <body> but preserves window.
    page.evaluate("window.__rwfw_marker = 'kept'")
        .await
        .expect("set marker");

    // Click the sidebar "Posts" link — Turbo Drive intercepts it.
    page.find_element("a[href='/blog/posts']")
        .await
        .expect("posts link")
        .click()
        .await
        .expect("click posts");

    // Wait for the Turbo Drive navigation to land.
    let mut navigated = false;
    for _ in 0..50 {
        let path: String = page
            .evaluate("window.location.pathname")
            .await
            .ok()
            .and_then(|r| r.into_value().ok())
            .unwrap_or_default();
        if path == "/blog/posts" {
            let body = page.content().await.unwrap_or_default();
            if body.contains("New Post")
                || body.contains("No posts yet")
                || body.contains("Sign in")
            {
                navigated = true;
                break;
            }
        }
        tokio::time::sleep(Duration::from_millis(100)).await;
    }
    assert!(navigated, "Turbo Drive did not navigate to /blog/posts");

    // The marker survived => Turbo Drive (not a full page reload).
    let marker: Option<String> = page
        .evaluate("window.__rwfw_marker || null")
        .await
        .expect("eval marker")
        .into_value()
        .expect("marker value");
    assert_eq!(
        marker.as_deref(),
        Some("kept"),
        "window was reset => a full reload happened instead of a Turbo Drive visit"
    );

    let _ = browser.close().await;
    handler_task.abort();
}

#[tokio::test]
async fn reactive_counter_updates_without_reload_and_rejects_tampering() {
    if std::env::var("RWFW_E2E").ok().as_deref() != Some("1") {
        eprintln!("skipping e2e: set RWFW_E2E=1 (and have Chromium) to run browser tests");
        return;
    }
    let Some(app) = TestApp::spawn().await else {
        eprintln!("skipping e2e: RWFW_TEST_DATABASE_URL not set");
        return;
    };

    let config = BrowserConfig::builder()
        .no_sandbox()
        .arg("--disable-dev-shm-usage")
        .build()
        .expect("browser config");

    let (mut browser, mut handler) = match Browser::launch(config).await {
        Ok(pair) => pair,
        Err(error) => {
            eprintln!("skipping e2e: could not launch Chromium: {error}");
            return;
        }
    };
    let handler_task = tokio::spawn(async move { while handler.next().await.is_some() {} });

    let page = browser.new_page("about:blank").await.expect("new page");
    page.goto(app.url("/demo/counter")).await.expect("goto counter");
    page.wait_for_navigation().await.expect("counter loaded");

    page.evaluate("window.__rwfw_marker = 'kept'")
        .await
        .expect("set marker");

    page.find_element("[data-reactive-action='increment']")
        .await
        .expect("increment")
        .click()
        .await
        .expect("click increment");
    wait_for_count(&page, 1).await;

    page.find_element("[data-reactive-action='increment']")
        .await
        .expect("increment")
        .click()
        .await
        .expect("click increment");
    wait_for_count(&page, 2).await;

    page.find_element("[data-reactive-action='decrement']")
        .await
        .expect("decrement")
        .click()
        .await
        .expect("click decrement");
    wait_for_count(&page, 1).await;

    let marker: Option<String> = page
        .evaluate("window.__rwfw_marker || null")
        .await
        .expect("eval marker")
        .into_value()
        .expect("marker value");
    assert_eq!(marker.as_deref(), Some("kept"));

    page.evaluate(
        "document.querySelector('[data-controller~=reactive]').dataset.reactiveToken += 'x'",
    )
    .await
    .expect("tamper token");
    page.find_element("[data-reactive-action='increment']")
        .await
        .expect("increment")
        .click()
        .await
        .expect("click tampered increment");
    tokio::time::sleep(Duration::from_millis(500)).await;
    assert_eq!(count_value(&page).await, 1);

    let _ = browser.close().await;
    handler_task.abort();
}

async fn wait_for_count(page: &chromiumoxide::Page, expected: i64) {
    for _ in 0..50 {
        if count_value(page).await == expected {
            return;
        }
        tokio::time::sleep(Duration::from_millis(100)).await;
    }
    panic!("counter did not reach {expected}");
}

async fn count_value(page: &chromiumoxide::Page) -> i64 {
    page.evaluate("Number(document.querySelector('[data-testid=count]').textContent.trim())")
        .await
        .expect("eval count")
        .into_value()
        .expect("count")
}
