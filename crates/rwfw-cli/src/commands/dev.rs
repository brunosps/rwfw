pub async fn run() -> anyhow::Result<()> {
    use std::process::Stdio;
    use tokio::process::Command;

    println!("Starting development server...");
    let project = crate::commands::app::project_config();
    let app_package = project.app_package;

    // Rebuild CSS on change via the committed Tailwind binary so editing a
    // template's classes reflects in the browser (picked up by live reload).
    // Optional: skipped gracefully when the binary isn't present.
    let tailwind = spawn_tailwind_watch();

    let mut app = Command::new("cargo")
        .args(["run", "-p", &app_package])
        .env("RWFW_ENV", "development")
        .stdin(Stdio::null())
        .stdout(Stdio::inherit())
        .stderr(Stdio::inherit())
        .spawn()?;

    let result = tokio::select! {
        status = app.wait() => match status {
            Ok(status) if status.success() => Ok(()),
            Ok(status) => Err(anyhow::anyhow!("Rust app exited with {status}")),
            Err(error) => Err(error.into()),
        },
        signal = tokio::signal::ctrl_c() => {
            signal?;
            let _ = app.kill().await;
            Ok(())
        }
    };

    // Don't leave the Tailwind watcher running after the dev server stops.
    if let Some(mut child) = tailwind {
        let _ = child.kill().await;
    }
    result
}

/// Spawn `tailwindcss --watch` from the project's committed `.rwfw/` toolchain so
/// class changes rebuild `app.css`. Returns `None` (and prints a note) when the
/// binary isn't present — live reload still works for direct template/CSS edits.
fn spawn_tailwind_watch() -> Option<tokio::process::Child> {
    use std::path::Path;
    use std::process::Stdio;
    use tokio::process::Command;

    let bin = Path::new(".rwfw/bin/tailwindcss");
    if !bin.exists() {
        println!(
            "  (no .rwfw/bin/tailwindcss — skipping CSS watch; live reload still works for template/CSS edits)"
        );
        return None;
    }

    // Generated apps build into crates/app; the framework repo into crates/rwfw-app.
    let output = if Path::new("crates/app/web/assets").exists() {
        "crates/app/web/assets/app.css"
    } else {
        "crates/rwfw-app/web/assets/app.css"
    };

    match Command::new(bin)
        .args([
            "-i",
            ".rwfw/tailwind.input.css",
            "-c",
            ".rwfw/tailwind.config.js",
            "-o",
            output,
            "--watch",
        ])
        .stdin(Stdio::null())
        .stdout(Stdio::inherit())
        .stderr(Stdio::inherit())
        .spawn()
    {
        Ok(child) => {
            println!("  Watching CSS: tailwindcss --watch -> {output}");
            Some(child)
        }
        Err(error) => {
            println!("  (failed to start tailwindcss --watch: {error}; continuing without CSS watch)");
            None
        }
    }
}
