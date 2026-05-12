use std::path::PathBuf;

#[derive(Debug, Clone)]
pub struct ProjectConfig {
    pub app_package: String,
    pub app_crate_dir: PathBuf,
    pub modules_dir: PathBuf,
}

impl Default for ProjectConfig {
    fn default() -> Self {
        Self {
            app_package: "rwfw-app".to_string(),
            app_crate_dir: PathBuf::from("crates/rwfw-app"),
            modules_dir: PathBuf::from("crates/modules"),
        }
    }
}

pub fn project_config() -> ProjectConfig {
    let mut config = project_config_from_file().unwrap_or_default();

    if let Ok(value) = std::env::var("RWFW_APP_PACKAGE") {
        config.app_package = value;
    }
    if let Ok(value) = std::env::var("RWFW_APP_CRATE_DIR") {
        config.app_crate_dir = PathBuf::from(value);
    }
    if let Ok(value) = std::env::var("RWFW_MODULES_DIR") {
        config.modules_dir = PathBuf::from(value);
    }

    config
}

pub fn package_name() -> String {
    project_config().app_package
}

pub fn crate_dir() -> PathBuf {
    project_config().app_crate_dir
}

pub fn modules_dir() -> PathBuf {
    project_config().modules_dir
}

fn project_config_from_file() -> anyhow::Result<ProjectConfig> {
    let mut config = ProjectConfig::default();
    let content = match std::fs::read_to_string("rwfw.toml") {
        Ok(content) => content,
        Err(error) if error.kind() == std::io::ErrorKind::NotFound => return Ok(config),
        Err(error) => return Err(error.into()),
    };

    for line in content.lines() {
        let line = line.trim();
        if line.is_empty() || line.starts_with('#') {
            continue;
        }

        let Some((key, value)) = line.split_once('=') else {
            continue;
        };
        let value = value.trim().trim_matches('"');

        match key.trim() {
            "app_package" => config.app_package = value.to_string(),
            "app_crate_dir" => config.app_crate_dir = PathBuf::from(value),
            "modules_dir" => config.modules_dir = PathBuf::from(value),
            _ => {}
        }
    }

    Ok(config)
}
