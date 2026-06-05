mod commands;

use clap::{Parser, Subcommand};
use std::path::PathBuf;

#[derive(Parser)]
#[command(name = "rwfw", version, about = "RWFW - Rust Web Framework CLI")]
struct Cli {
    #[command(subcommand)]
    command: Commands,
}

#[derive(Subcommand)]
enum Commands {
    /// Start development server (cargo watch + vite dev)
    Dev,
    /// Build for production
    Build,
    /// Run database migrations
    Migrate,
    /// Run database seeds
    Seed {
        #[command(subcommand)]
        what: Option<SeedCommands>,
    },
    /// Create a new app, example app, or module
    New {
        #[command(subcommand)]
        what: NewCommands,
    },
    /// Generate framework code
    Generate {
        #[command(subcommand)]
        what: GenerateCommands,
    },
    /// Manage roles and permissions
    Rbac {
        #[command(subcommand)]
        what: RbacCommands,
    },
    /// Manage authentication users
    Auth {
        #[command(subcommand)]
        what: AuthCommands,
    },
}

#[derive(Subcommand)]
enum NewCommands {
    /// Create a new RWFW application
    App {
        name: String,
        /// Example application to generate
        #[arg(long, value_enum, default_value = "blog")]
        example: commands::new_app::ExampleKind,
        /// Database backend for the generated app
        #[arg(long, value_enum, default_value = "postgres")]
        database: commands::new_app::DatabaseKind,
        /// Generate a Tauri desktop shell (src-tauri/) around the app
        #[arg(long)]
        tauri: bool,
        /// Use local RWFW framework crates from this repository path
        #[arg(long)]
        rwfw_path: Option<PathBuf>,
        /// Use published RWFW crates at this version
        #[arg(long)]
        rwfw_version: Option<String>,
        /// Use RWFW framework crates from this Git repository URL
        #[arg(long)]
        rwfw_git: Option<String>,
        /// Git tag used with --rwfw-git
        #[arg(long)]
        rwfw_tag: Option<String>,
    },
    /// Create a new RWFW example application
    Example {
        name: String,
        /// Example application to generate
        #[arg(long, value_enum, default_value = "blog")]
        example: commands::new_app::ExampleKind,
        /// Database backend for the generated app
        #[arg(long, value_enum, default_value = "postgres")]
        database: commands::new_app::DatabaseKind,
        /// Generate a Tauri desktop shell (src-tauri/) around the app
        #[arg(long)]
        tauri: bool,
        /// Use local RWFW framework crates from this repository path
        #[arg(long)]
        rwfw_path: Option<PathBuf>,
        /// Use published RWFW crates at this version
        #[arg(long)]
        rwfw_version: Option<String>,
        /// Use RWFW framework crates from this Git repository URL
        #[arg(long)]
        rwfw_git: Option<String>,
        /// Git tag used with --rwfw-git
        #[arg(long)]
        rwfw_tag: Option<String>,
    },
    /// Create a new module
    Module { name: String },
}

#[derive(Subcommand)]
enum GenerateCommands {
    /// Generate a SeaORM model and modular migration
    Model {
        name: String,
        #[arg(long)]
        module: String,
        #[arg(long)]
        fields: Option<String>,
    },
    /// Generate a CRUD scaffold slice for a module
    Scaffold {
        name: String,
        #[arg(long)]
        module: String,
        fields: Vec<String>,
    },
}

#[derive(Subcommand)]
enum RbacCommands {
    /// List synced permissions
    ListPermissions,
    /// List roles
    ListRoles,
    /// Create or update a role
    CreateRole {
        name: String,
        #[arg(long, default_value = "")]
        description: String,
    },
    /// Grant a permission to a role
    Grant { role: String, permission: String },
    /// Assign a role to a user by email
    Assign { email: String, role: String },
}

#[derive(Subcommand)]
enum SeedCommands {
    /// Create or ensure the initial admin user
    Admin {
        #[arg(long, default_value = "Admin")]
        name: String,
        #[arg(long)]
        email: String,
        /// Prefer RWFW_ADMIN_PASSWORD to avoid storing secrets in shell history
        #[arg(long)]
        password: Option<String>,
        /// Replace password when the user already exists
        #[arg(long)]
        update_password: bool,
    },
}

#[derive(Subcommand)]
enum AuthCommands {
    /// Create or ensure the initial admin user
    CreateAdmin {
        #[arg(long, default_value = "Admin")]
        name: String,
        #[arg(long)]
        email: String,
        /// Prefer RWFW_ADMIN_PASSWORD to avoid storing secrets in shell history
        #[arg(long)]
        password: Option<String>,
        /// Replace password when the user already exists
        #[arg(long)]
        update_password: bool,
    },
    /// Manage SSO/OIDC providers
    Sso {
        #[command(subcommand)]
        what: SsoCommands,
    },
}

#[derive(Subcommand)]
enum SsoCommands {
    /// Add or update an OIDC SSO provider in this app
    Add {
        name: String,
        #[arg(long, value_enum, default_value = "oidc")]
        provider: commands::auth::SsoProviderKind,
        #[arg(long)]
        issuer_url: String,
        #[arg(long)]
        client_id_env: String,
        #[arg(long)]
        client_secret_env: String,
        #[arg(long)]
        display_name: Option<String>,
        #[arg(long)]
        redirect_base_url: Option<String>,
        #[arg(long, default_value = "openid profile email")]
        scopes: String,
        #[arg(long)]
        force: bool,
    },
}

#[tokio::main]
async fn main() -> anyhow::Result<()> {
    let cli = Cli::parse();

    match cli.command {
        Commands::Dev => commands::dev::run().await?,
        Commands::Build => commands::build::run().await?,
        Commands::Migrate => commands::migrate::run().await?,
        Commands::Seed { what } => match what {
            Some(SeedCommands::Admin {
                name,
                email,
                password,
                update_password,
            }) => {
                commands::auth::create_admin(&name, &email, password.as_deref(), update_password)
                    .await?
            }
            None => commands::seed::run().await?,
        },
        Commands::New { what } => match what {
            NewCommands::App {
                name,
                example,
                database,
                tauri,
                rwfw_path,
                rwfw_version,
                rwfw_git,
                rwfw_tag,
            } => commands::new_app::run(
                &name,
                example,
                database,
                tauri,
                rwfw_path.as_deref(),
                rwfw_version.as_deref(),
                rwfw_git.as_deref(),
                rwfw_tag.as_deref(),
            )?,
            NewCommands::Example {
                name,
                example,
                database,
                tauri,
                rwfw_path,
                rwfw_version,
                rwfw_git,
                rwfw_tag,
            } => commands::new_app::run(
                &name,
                example,
                database,
                tauri,
                rwfw_path.as_deref(),
                rwfw_version.as_deref(),
                rwfw_git.as_deref(),
                rwfw_tag.as_deref(),
            )?,
            NewCommands::Module { name } => commands::new_module::run(&name).await?,
        },
        Commands::Generate { what } => match what {
            GenerateCommands::Model {
                name,
                module,
                fields,
            } => commands::generate::run_model(&name, &module, fields.as_deref()).await?,
            GenerateCommands::Scaffold {
                name,
                module,
                fields,
            } => commands::generate::run_scaffold(&name, &module, &fields).await?,
        },
        Commands::Rbac { what } => match what {
            RbacCommands::ListPermissions => commands::rbac::list_permissions().await?,
            RbacCommands::ListRoles => commands::rbac::list_roles().await?,
            RbacCommands::CreateRole { name, description } => {
                commands::rbac::create_role(&name, &description).await?
            }
            RbacCommands::Grant { role, permission } => {
                commands::rbac::grant(&role, &permission).await?
            }
            RbacCommands::Assign { email, role } => commands::rbac::assign(&email, &role).await?,
        },
        Commands::Auth { what } => match what {
            AuthCommands::CreateAdmin {
                name,
                email,
                password,
                update_password,
            } => {
                commands::auth::create_admin(&name, &email, password.as_deref(), update_password)
                    .await?
            }
            AuthCommands::Sso { what } => match what {
                SsoCommands::Add {
                    name,
                    provider,
                    issuer_url,
                    client_id_env,
                    client_secret_env,
                    display_name,
                    redirect_base_url,
                    scopes,
                    force,
                } => commands::auth::sso_add(commands::auth::SsoAddInput {
                    name,
                    provider,
                    issuer_url,
                    client_id_env,
                    client_secret_env,
                    display_name,
                    redirect_base_url,
                    scopes,
                    force,
                })?,
            },
        },
    }

    Ok(())
}
