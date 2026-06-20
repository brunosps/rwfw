use std::process::Stdio;
use tokio::process::{Child, Command};

const CARGO_WATCH_FALLBACK_MESSAGE: &str =
    "cargo-watch não encontrado; rodando cargo run sem auto-restart de Rust";

pub async fn run() -> anyhow::Result<()> {
    println!("Starting development server...");
    let project = crate::commands::app::project_config();
    let app_package = project.app_package;

    // Rebuild CSS on change via the committed Tailwind binary so editing a
    // template's classes reflects in the browser (picked up by live reload).
    // Optional: skipped gracefully when the binary isn't present.
    let tailwind = spawn_tailwind_watch();

    let mut app = spawn_rust_app(&app_package)?;

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

fn spawn_rust_app(app_package: &str) -> anyhow::Result<Child> {
    let dev_command = rust_dev_command(app_package, cargo_watch_available());
    if let Some(cargo_watch_command) = dev_command.watch_command.as_deref() {
        println!("  Watching Rust: cargo watch -x \"{cargo_watch_command}\"");
    } else {
        println!("{CARGO_WATCH_FALLBACK_MESSAGE}");
    }

    Ok(Command::new(dev_command.program)
        .args(&dev_command.args)
        .env("RWFW_ENV", "development")
        .stdin(Stdio::null())
        .stdout(Stdio::inherit())
        .stderr(Stdio::inherit())
        .spawn()?)
}

#[derive(Debug, PartialEq, Eq)]
struct RustDevCommand {
    program: &'static str,
    args: Vec<String>,
    watch_command: Option<String>,
}

fn rust_dev_command(app_package: &str, cargo_watch_available: bool) -> RustDevCommand {
    if cargo_watch_available {
        let watch_command = format!("run -p {app_package}");
        return RustDevCommand {
            program: "cargo",
            args: vec!["watch".to_string(), "-x".to_string(), watch_command.clone()],
            watch_command: Some(watch_command),
        };
    }

    RustDevCommand {
        program: "cargo",
        args: vec!["run".to_string(), "-p".to_string(), app_package.to_string()],
        watch_command: None,
    }
}

fn cargo_watch_available() -> bool {
    std::process::Command::new("cargo-watch")
        .arg("--version")
        .stdin(Stdio::null())
        .stdout(Stdio::null())
        .stderr(Stdio::null())
        .status()
        .is_ok_and(|status| status.success())
}

/// Spawn `tailwindcss --watch` from the project's committed `.rwfw/` toolchain so
/// class changes rebuild `app.css`. Returns `None` (and prints a note) when the
/// binary isn't present — live reload still works for direct template/CSS edits.
fn spawn_tailwind_watch() -> Option<Child> {
    use std::path::Path;

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
            println!(
                "  (failed to start tailwindcss --watch: {error}; continuing without CSS watch)"
            );
            None
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn rust_dev_command_uses_cargo_watch_when_available() {
        let command = rust_dev_command("rwfw-app", true);

        assert_eq!(command.program, "cargo");
        assert_eq!(
            command.args,
            vec![
                "watch".to_string(),
                "-x".to_string(),
                "run -p rwfw-app".to_string()
            ]
        );
        assert_eq!(command.watch_command.as_deref(), Some("run -p rwfw-app"));
    }

    #[test]
    fn rust_dev_command_falls_back_to_cargo_run() {
        let command = rust_dev_command("rwfw-app", false);

        assert_eq!(command.program, "cargo");
        assert_eq!(
            command.args,
            vec!["run".to_string(), "-p".to_string(), "rwfw-app".to_string()]
        );
        assert_eq!(command.watch_command, None);
    }

    #[test]
    fn cargo_watch_fallback_message_matches_cli_contract() {
        assert_eq!(
            CARGO_WATCH_FALLBACK_MESSAGE,
            "cargo-watch não encontrado; rodando cargo run sem auto-restart de Rust"
        );
    }
}
