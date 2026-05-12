use std::process::Stdio;
use tokio::process::Command;

pub async fn run() -> anyhow::Result<()> {
    println!("Building for production...");

    let app_package = crate::commands::app::package_name();
    run_command("npm", &["run", "build"]).await?;
    run_command("npm", &["run", "build:ssr"]).await?;
    run_command("cargo", &["build", "--release", "-p", &app_package]).await?;

    println!("Production build complete.");
    Ok(())
}

async fn run_command(program: &str, args: &[&str]) -> anyhow::Result<()> {
    let status = Command::new(program)
        .args(args)
        .stdin(Stdio::null())
        .stdout(Stdio::inherit())
        .stderr(Stdio::inherit())
        .status()
        .await?;

    if !status.success() {
        anyhow::bail!("{program} {} failed with {status}", args.join(" "));
    }

    Ok(())
}
