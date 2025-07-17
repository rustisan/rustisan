//! Database management commands for the Rustisan CLI
//!
//! This module provides commands for managing database operations,
//! similar to Laravel's database commands.

use anyhow::Result;
use colored::*;
use std::process::Command;
use std::fs;
use toml::Value;

use super::CommandUtils;
use crate::DbCommands;

/// Handle database commands
pub async fn handle(operation: DbCommands) -> Result<()> {
    match operation {
        DbCommands::Status => show_status().await,
        DbCommands::Create => create_database().await,
        DbCommands::Drop { force } => drop_database(force).await,
        DbCommands::Reset { force } => reset_database(force).await,
        DbCommands::Seed => seed_database().await,
    }
}

/// Show database status
async fn show_status() -> Result<()> {
    CommandUtils::ensure_rustisan_project()?;

    CommandUtils::info("Database Status:");
    println!();

    // Load configuration from rustisan.toml
    if let Ok(config) = load_config() {
        if let Some(db_driver) = get_config_value(&config, "database.connections.default.driver") {
            let db_host = get_config_value(&config, "database.connections.default.host")
                .unwrap_or_else(|| "localhost".to_string());
            let db_port = get_config_value(&config, "database.connections.default.port")
                .unwrap_or_else(|| "3306".to_string());
            let db_name = get_config_value(&config, "database.connections.default.database")
                .unwrap_or_else(|| "unknown".to_string());

            println!("  {} {}", "Driver:".cyan().bold(), db_driver);
            println!("  {} {}:{}", "Host:".cyan().bold(), db_host, db_port);
            println!("  {} {}", "Database:".cyan().bold(), db_name);

            // Test connection
            match test_connection(&db_driver, &db_host, &db_port, &db_name).await {
                Ok(_) => {
                    CommandUtils::success("Database connection: OK");
                }
                Err(e) => {
                    CommandUtils::error(&format!("Database connection failed: {}", e));
                }
            }
        } else {
            CommandUtils::warning("No database configuration found in rustisan.toml");
        }
    } else {
        CommandUtils::warning("No database configuration found in rustisan.toml");
    }

    println!();
    CommandUtils::info("Migration status:");
    // TODO: Check migration status
    println!("  Use 'rustisan migrate:status' for detailed migration information");

    Ok(())
}

/// Create database
async fn create_database() -> Result<()> {
    CommandUtils::ensure_rustisan_project()?;

    let config = load_config()?;
    let db_driver = get_config_value(&config, "database.connections.default.driver")
        .ok_or_else(|| anyhow::anyhow!("Database driver not configured in rustisan.toml"))?;
    let db_name = get_config_value(&config, "database.connections.default.database")
        .ok_or_else(|| anyhow::anyhow!("Database name not configured in rustisan.toml"))?;

    CommandUtils::info(&format!("Creating database '{}'...", db_name.cyan().bold()));

    match db_driver.as_str() {
        "mysql" => create_mysql_database(&db_name).await?,
        "postgres" => create_postgres_database(&db_name).await?,
        _ => {
            return Err(anyhow::anyhow!("Unsupported database driver: {}", db_driver));
        }
    }

    CommandUtils::success(&format!("Database '{}' created successfully!", db_name.cyan().bold()));

    Ok(())
}

/// Drop database
async fn drop_database(force: bool) -> Result<()> {
    CommandUtils::ensure_rustisan_project()?;

    let config = load_config()?;
    let db_driver = get_config_value(&config, "database.connections.default.driver")
        .ok_or_else(|| anyhow::anyhow!("Database driver not configured in rustisan.toml"))?;
    let db_name = get_config_value(&config, "database.connections.default.database")
        .ok_or_else(|| anyhow::anyhow!("Database name not configured in rustisan.toml"))?;

    if !force {
        CommandUtils::warning(&format!("This will permanently delete database '{}'", db_name));
        print!("Are you sure? (yes/no): ");
        use std::io::{self, Write};
        io::stdout().flush()?;

        let mut input = String::new();
        io::stdin().read_line(&mut input)?;

        if input.trim().to_lowercase() != "yes" {
            CommandUtils::info("Operation cancelled");
            return Ok(());
        }
    }

    CommandUtils::info(&format!("Dropping database '{}'...", db_name.cyan().bold()));

    match db_driver.as_str() {
        "mysql" => drop_mysql_database(&db_name).await?,
        "postgres" => drop_postgres_database(&db_name).await?,
        _ => {
            return Err(anyhow::anyhow!("Unsupported database driver: {}", db_driver));
        }
    }

    CommandUtils::success(&format!("Database '{}' dropped successfully!", db_name.cyan().bold()));

    Ok(())
}

/// Reset database (drop and recreate)
async fn reset_database(force: bool) -> Result<()> {
    CommandUtils::ensure_rustisan_project()?;

    CommandUtils::info("Resetting database...");

    // Drop database
    drop_database(force).await?;

    // Create database
    create_database().await?;

    CommandUtils::success("Database reset completed!");

    Ok(())
}

/// Seed database
async fn seed_database() -> Result<()> {
    CommandUtils::ensure_rustisan_project()?;

    CommandUtils::info("Seeding database...");

    // Run seed command
    let output = Command::new("cargo")
        .args(&["run", "--bin", "seed"])
        .output()?;

    if output.status.success() {
        CommandUtils::success("Database seeded successfully!");
    } else {
        let stderr = String::from_utf8_lossy(&output.stderr);
        CommandUtils::error(&format!("Seeding failed: {}", stderr));
        return Err(anyhow::anyhow!("Database seeding failed"));
    }

    Ok(())
}

/// Test database connection
async fn test_connection(driver: &str, host: &str, port: &str, database: &str) -> Result<()> {
    match driver {
        "mysql" => test_mysql_connection(host, port, database).await,
        "postgres" => test_postgres_connection(host, port, database).await,
        _ => Err(anyhow::anyhow!("Unsupported database driver: {}", driver)),
    }
}

/// Create MySQL database
async fn create_mysql_database(db_name: &str) -> Result<()> {
    let config = load_config()?;
    let host = get_config_value(&config, "database.connections.default.host")
        .unwrap_or_else(|| "localhost".to_string());
    let port = get_config_value(&config, "database.connections.default.port")
        .unwrap_or_else(|| "3306".to_string());
    let username = get_config_value(&config, "database.connections.default.username")
        .unwrap_or_else(|| "root".to_string());
    let password = get_config_value(&config, "database.connections.default.password")
        .unwrap_or_default();

    let mut args = vec![
        format!("-h{}", host),
        format!("-P{}", port),
        format!("-u{}", username),
    ];

    if !password.is_empty() {
        args.push(format!("-p{}", password));
    }

    args.push("-e".to_string());
    args.push(format!("CREATE DATABASE IF NOT EXISTS `{}`", db_name));

    let output = Command::new("mysql")
        .args(&args)
        .output()?;

    if !output.status.success() {
        let stderr = String::from_utf8_lossy(&output.stderr);
        return Err(anyhow::anyhow!("MySQL error: {}", stderr));
    }

    Ok(())
}

/// Drop MySQL database
async fn drop_mysql_database(db_name: &str) -> Result<()> {
    let config = load_config()?;
    let host = get_config_value(&config, "database.connections.default.host")
        .unwrap_or_else(|| "localhost".to_string());
    let port = get_config_value(&config, "database.connections.default.port")
        .unwrap_or_else(|| "3306".to_string());
    let username = get_config_value(&config, "database.connections.default.username")
        .unwrap_or_else(|| "root".to_string());
    let password = get_config_value(&config, "database.connections.default.password")
        .unwrap_or_default();

    let mut args = vec![
        format!("-h{}", host),
        format!("-P{}", port),
        format!("-u{}", username),
    ];

    if !password.is_empty() {
        args.push(format!("-p{}", password));
    }

    args.push("-e".to_string());
    args.push(format!("DROP DATABASE IF EXISTS `{}`", db_name));

    let output = Command::new("mysql")
        .args(&args)
        .output()?;

    if !output.status.success() {
        let stderr = String::from_utf8_lossy(&output.stderr);
        return Err(anyhow::anyhow!("MySQL error: {}", stderr));
    }

    Ok(())
}

/// Test MySQL connection
async fn test_mysql_connection(host: &str, port: &str, database: &str) -> Result<()> {
    let config = load_config()?;
    let username = get_config_value(&config, "database.connections.default.username")
        .unwrap_or_else(|| "root".to_string());
    let password = get_config_value(&config, "database.connections.default.password")
        .unwrap_or_default();

    let mut args = vec![
        format!("-h{}", host),
        format!("-P{}", port),
        format!("-u{}", username),
    ];

    if !password.is_empty() {
        args.push(format!("-p{}", password));
    }

    args.push(database.to_string());
    args.push("-e".to_string());
    args.push("SELECT 1".to_string());

    let output = Command::new("mysql")
        .args(&args)
        .output()?;

    if !output.status.success() {
        let stderr = String::from_utf8_lossy(&output.stderr);
        return Err(anyhow::anyhow!("Connection failed: {}", stderr));
    }

    Ok(())
}

/// Create PostgreSQL database
async fn create_postgres_database(db_name: &str) -> Result<()> {
    // TODO: Implement PostgreSQL database creation
    CommandUtils::warning("PostgreSQL support not yet implemented");
    Ok(())
}

/// Drop PostgreSQL database
async fn drop_postgres_database(db_name: &str) -> Result<()> {
    // TODO: Implement PostgreSQL database dropping
    CommandUtils::warning("PostgreSQL support not yet implemented");
    Ok(())
}

/// Test PostgreSQL connection
async fn test_postgres_connection(host: &str, port: &str, database: &str) -> Result<()> {
    // TODO: Implement PostgreSQL connection testing
    CommandUtils::warning("PostgreSQL support not yet implemented");
    Ok(())
}

/// Load configuration from rustisan.toml
fn load_config() -> Result<Value> {
    let config_content = fs::read_to_string("rustisan.toml")
        .map_err(|_| anyhow::anyhow!("rustisan.toml not found"))?;
    let config: Value = toml::from_str(&config_content)
        .map_err(|e| anyhow::anyhow!("Failed to parse rustisan.toml: {}", e))?;
    Ok(config)
}

/// Get nested value from TOML configuration
fn get_config_value(config: &Value, key: &str) -> Option<String> {
    let parts: Vec<&str> = key.split('.').collect();
    let mut current = config;

    for part in parts {
        match current {
            Value::Table(table) => {
                current = table.get(part)?;
            }
            _ => return None,
        }
    }

    match current {
        Value::String(s) => Some(s.clone()),
        Value::Integer(i) => Some(i.to_string()),
        Value::Float(f) => Some(f.to_string()),
        Value::Boolean(b) => Some(b.to_string()),
        _ => None,
    }
}
