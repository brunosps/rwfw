pub async fn run() -> anyhow::Result<()> {
    use std::process::Stdio;
    use tokio::process::Command;

    println!("Starting development server...");
    let project = crate::commands::app::project_config();
    let app_package = project.app_package;

    let mut app = Command::new("cargo")
        .args(["run", "-p", &app_package])
        .env("RWFW_ENV", "development")
        .stdin(Stdio::null())
        .stdout(Stdio::inherit())
        .stderr(Stdio::inherit())
        .spawn()?;

    tokio::select! {
        status = app.wait() => {
            let status = status?;
            if status.success() {
                Ok(())
            } else {
                anyhow::bail!("Rust app exited with {status}");
            }
        }
        signal = tokio::signal::ctrl_c() => {
            signal?;
            let _ = app.kill().await;
            Ok(())
        }
    }
}
