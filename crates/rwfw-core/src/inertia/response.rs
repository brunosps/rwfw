/// Render the HTML shell for an initial (non-XHR) Inertia page load.
/// The shell includes the Vite client script in dev mode and the data-page attribute.
pub fn render_html_shell(page_json: &str, ssr_html: Option<&str>) -> String {
    let body_content = ssr_html.unwrap_or("");

    // Escape the JSON for safe embedding in an HTML attribute
    let escaped_json = page_json
        .replace('&', "&amp;")
        .replace('"', "&quot;")
        .replace('<', "&lt;")
        .replace('>', "&gt;");

    let vite_assets = if is_dev_mode() {
        let entry = std::env::var("RWFW_VITE_ENTRY")
            .unwrap_or_else(|_| "crates/rwfw-app/web/app.tsx".to_string());
        let dev_server = std::env::var("RWFW_VITE_DEV_SERVER")
            .unwrap_or_else(|_| "http://localhost:5173".to_string());
        let dev_server = dev_server.trim_end_matches('/');
        format!(
            r#"<script type="module">
        import RefreshRuntime from "{dev_server}/@react-refresh"
        RefreshRuntime.injectIntoGlobalHook(window)
        window.$RefreshReg$ = () => {{}}
        window.$RefreshSig$ = () => (type) => type
        window.__vite_plugin_react_preamble_installed__ = true
    </script>
    <script type="module" src="{dev_server}/@vite/client"></script>
    <script type="module" src="{dev_server}/{entry}"></script>"#
        )
    } else {
        match vite_production_assets() {
            Ok(assets) => assets,
            Err(error) => {
                tracing::error!(error = %error, "Failed to load Vite production assets");
                String::new()
            }
        }
    };

    format!(
        r#"<!DOCTYPE html>
<html lang="en">
<head>
    <meta charset="UTF-8">
    <meta name="viewport" content="width=device-width, initial-scale=1.0">
    <title>RWFW</title>
    {vite_assets}
</head>
<body>
    <div id="app" data-page="{escaped_json}">{body_content}</div>
</body>
</html>"#
    )
}

fn is_dev_mode() -> bool {
    std::env::var("RWFW_ENV").unwrap_or_else(|_| "development".into()) == "development"
}

fn vite_production_assets() -> anyhow::Result<String> {
    let manifest_path = std::env::var("RWFW_VITE_MANIFEST")
        .unwrap_or_else(|_| "dist/client/.vite/manifest.json".to_string());
    let manifest = crate::vite::ViteManifest::load(&manifest_path)?;
    let configured_entry = std::env::var("RWFW_VITE_ENTRY").ok();
    let entry = configured_entry
        .as_deref()
        .or_else(|| manifest.first_entry_key())
        .unwrap_or("crates/rwfw-app/web/app.tsx");

    let mut tags = String::new();
    for css in manifest.entry_css(entry) {
        tags.push_str(&format!(
            r#"<link rel="stylesheet" href="{}">"#,
            public_asset_path(css)
        ));
        tags.push('\n');
    }

    if let Some(script) = manifest.entry_script(entry) {
        tags.push_str(&format!(
            r#"<script type="module" src="{}"></script>"#,
            public_asset_path(script)
        ));
    }

    Ok(tags)
}

fn public_asset_path(file: &str) -> String {
    format!("/{}", file.trim_start_matches('/'))
}
