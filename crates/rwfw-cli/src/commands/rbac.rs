pub async fn list_permissions() -> anyhow::Result<()> {
    let db = connect().await?;
    let permissions = rwfw_core::auth::list_permissions(&db).await?;

    if permissions.is_empty() {
        println!("No permissions found. Run `rwfw migrate` first.");
        return Ok(());
    }

    for permission in permissions {
        match permission.description {
            Some(description) if !description.is_empty() => {
                println!("{} - {}", permission.name, description);
            }
            _ => println!("{}", permission.name),
        }
    }

    Ok(())
}

pub async fn list_roles() -> anyhow::Result<()> {
    let db = connect().await?;
    let roles = rwfw_core::auth::list_roles(&db).await?;

    if roles.is_empty() {
        println!("No roles found. Run `rwfw migrate` first.");
        return Ok(());
    }

    for role in roles {
        match role.description {
            Some(description) if !description.is_empty() => {
                println!("{} - {}", role.name, description);
            }
            _ => println!("{}", role.name),
        }
    }

    Ok(())
}

pub async fn create_role(name: &str, description: &str) -> anyhow::Result<()> {
    let db = connect().await?;
    rwfw_core::auth::ensure_role(&db, name, description).await?;
    println!("Role `{name}` is available.");
    Ok(())
}

pub async fn grant(role: &str, permission: &str) -> anyhow::Result<()> {
    let db = connect().await?;
    rwfw_core::auth::grant_permission_to_role(&db, role, permission).await?;
    println!("Granted `{permission}` to role `{role}`.");
    Ok(())
}

pub async fn assign(email: &str, role: &str) -> anyhow::Result<()> {
    let db = connect().await?;
    rwfw_core::auth::assign_role_by_email(&db, email, role).await?;
    println!("Assigned role `{role}` to `{email}`.");
    Ok(())
}

async fn connect() -> anyhow::Result<sea_orm::DatabaseConnection> {
    let config = rwfw_core::config::AppConfig::load()?;
    rwfw_core::db::connect(&config).await
}
