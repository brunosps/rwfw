pub async fn run() -> anyhow::Result<()> {
    use std::process::Stdio;
    use tokio::process::Command;

    println!("Starting development server...");
    let project = crate::commands::app::project_config();
    let app_package = project.app_package;
    let vite_entry = format!("{}/web/app.tsx", project.app_crate_dir.to_string_lossy());
    let vite_port = std::env::var("RWFW_VITE_PORT").unwrap_or_else(|_| "5173".to_string());
    let vite_dev_server = std::env::var("RWFW_VITE_DEV_SERVER")
        .unwrap_or_else(|_| format!("http://localhost:{vite_port}"));

    let mut vite = Command::new("npm")
        .args(["run", "dev"])
        .env("RWFW_VITE_PORT", &vite_port)
        .stdin(Stdio::null())
        .stdout(Stdio::inherit())
        .stderr(Stdio::inherit())
        .spawn()?;

    let mut app = Command::new("cargo")
        .args(["run", "-p", &app_package])
        .env("RWFW_ENV", "development")
        .env("RWFW_VITE_ENTRY", vite_entry)
        .env("RWFW_VITE_DEV_SERVER", vite_dev_server)
        .stdin(Stdio::null())
        .stdout(Stdio::inherit())
        .stderr(Stdio::inherit())
        .spawn()?;

    tokio::select! {
        status = vite.wait() => {
            let status = status?;
            let _ = app.kill().await;
            if status.success() {
                Ok(())
            } else {
                anyhow::bail!("Vite dev server exited with {status}");
            }
        }
        status = app.wait() => {
            let status = status?;
            let _ = vite.kill().await;
            if status.success() {
                Ok(())
            } else {
                anyhow::bail!("Rust app exited with {status}");
            }
        }
        signal = tokio::signal::ctrl_c() => {
            signal?;
            let _ = vite.kill().await;
            let _ = app.kill().await;
            Ok(())
        }
    }
}
