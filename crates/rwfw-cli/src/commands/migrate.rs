pub async fn run() -> anyhow::Result<()> {
    use std::process::Stdio;
    use tokio::process::Command;

    let app_package = crate::commands::app::package_name();
    let status = Command::new("cargo")
        .args(["run", "-p", &app_package, "--", "__rwfw", "migrate"])
        .stdin(Stdio::null())
        .stdout(Stdio::inherit())
        .stderr(Stdio::inherit())
        .status()
        .await?;

    if !status.success() {
        anyhow::bail!("App migration command failed with {status}");
    }

    Ok(())
}
