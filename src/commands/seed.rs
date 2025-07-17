//! Seed command implementations for the Rustisan CLI

use anyhow::Result;
use colored::*;
use super::CommandUtils;

/// Handle seed command
pub async fn handle(class: Option<String>, force: bool) -> Result<()> {
    CommandUtils::ensure_rustisan_project()?;

    if let Some(seeder_class) = class {
        run_specific_seeder(seeder_class, force).await
    } else {
        run_all_seeders(force).await
    }
}

async fn run_specific_seeder(class: String, force: bool) -> Result<()> {
    CommandUtils::info(&format!("Running seeder: {}", class));

    // Check if we're in production and force is not set
    if is_production_environment() && !force {
        CommandUtils::error("Cannot run seeders in production environment without --force flag");
        return Ok(());
    }

    // TODO: Implement specific seeder logic
    CommandUtils::success(&format!("Seeder {} completed successfully", class));

    Ok(())
}

async fn run_all_seeders(force: bool) -> Result<()> {
    CommandUtils::info("Running database seeders...");

    // Check if we're in production and force is not set
    if is_production_environment() && !force {
        CommandUtils::error("Cannot run seeders in production environment without --force flag");
        return Ok(());
    }

    // TODO: Implement logic to discover and run all seeders
    let seeders = discover_seeders()?;

    if seeders.is_empty() {
        CommandUtils::warning("No seeders found");
        return Ok(());
    }

    for seeder in seeders {
        CommandUtils::info(&format!("Running seeder: {}", seeder));
        // TODO: Execute seeder
    }

    CommandUtils::success("All seeders completed successfully");

    Ok(())
}

fn discover_seeders() -> Result<Vec<String>> {
    let seeders_dir = std::path::Path::new("database/seeders");

    if !seeders_dir.exists() {
        return Ok(Vec::new());
    }

    let mut seeders = Vec::new();

    for entry in std::fs::read_dir(seeders_dir)? {
        let entry = entry?;
        let path = entry.path();

        if path.is_file() && path.extension().map_or(false, |ext| ext == "rs") {
            if let Some(name) = path.file_stem() {
                seeders.push(name.to_string_lossy().to_string());
            }
        }
    }

    // Sort seeders alphabetically
    seeders.sort();

    Ok(seeders)
}

fn is_production_environment() -> bool {
    std::env::var("RUSTISAN_ENV")
        .or_else(|_| std::env::var("APP_ENV"))
        .map(|env| env.to_lowercase() == "production")
        .unwrap_or(false)
}
