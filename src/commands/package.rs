//! Package command implementations for the Rustisan CLI

use anyhow::Result;
use colored::*;
use crate::PackageCommands;
use super::CommandUtils;

/// Handle package command
pub async fn handle(operation: PackageCommands) -> Result<()> {
    CommandUtils::ensure_rustisan_project()?;

    match operation {
        PackageCommands::Install { name, version } => {
            install_package(name, version).await
        }
        PackageCommands::Remove { name } => {
            remove_package(name).await
        }
        PackageCommands::List => {
            list_packages().await
        }
        PackageCommands::Update => {
            update_packages().await
        }
    }
}

async fn install_package(name: String, version: Option<String>) -> Result<()> {
    let version_spec = version.unwrap_or_else(|| "latest".to_string());

    CommandUtils::info(&format!("Installing package: {} ({})", name, version_spec));

    // Check if package already exists
    if is_package_installed(&name)? {
        CommandUtils::warning(&format!("Package {} is already installed", name));
        return Ok(());
    }

    // Validate package name
    if !is_valid_package_name(&name) {
        CommandUtils::error(&format!("Invalid package name: {}", name));
        return Ok(());
    }

    // Add to Cargo.toml
    add_dependency_to_cargo_toml(&name, &version_spec)?;

    // Run cargo add command
    let mut cargo_args = vec!["add", &name];
    if version_spec != "latest" {
        cargo_args.push("--version");
        cargo_args.push(&version_spec);
    }

    let output = std::process::Command::new("cargo")
        .args(&cargo_args)
        .output()?;

    if output.status.success() {
        CommandUtils::success(&format!("Package {} installed successfully", name));

        // Update package registry
        update_package_registry(&name, &version_spec)?;

        // Show installation info
        show_package_info(&name).await?;
    } else {
        let stderr = String::from_utf8_lossy(&output.stderr);
        CommandUtils::error(&format!("Failed to install package: {}", stderr));
    }

    Ok(())
}

async fn remove_package(name: String) -> Result<()> {
    CommandUtils::info(&format!("Removing package: {}", name));

    // Check if package exists
    if !is_package_installed(&name)? {
        CommandUtils::warning(&format!("Package {} is not installed", name));
        return Ok(());
    }

    // Remove from Cargo.toml
    remove_dependency_from_cargo_toml(&name)?;

    // Run cargo remove command
    let output = std::process::Command::new("cargo")
        .args(&["remove", &name])
        .output()?;

    if output.status.success() {
        CommandUtils::success(&format!("Package {} removed successfully", name));

        // Update package registry
        remove_from_package_registry(&name)?;
    } else {
        let stderr = String::from_utf8_lossy(&output.stderr);
        CommandUtils::error(&format!("Failed to remove package: {}", stderr));
    }

    Ok(())
}

async fn list_packages() -> Result<()> {
    CommandUtils::info("Listing installed packages...");

    let packages = get_installed_packages()?;

    if packages.is_empty() {
        CommandUtils::warning("No packages found");
        return Ok(());
    }

    println!("\n{}", "Installed Packages:".bold());
    println!("┌─────────────────────────────────────────────────────────────────────────────┐");
    println!("│ {} │ {} │ {} │", "Name".bold(), "Version".bold(), "Features".bold());
    println!("├─────────────────────────────────────────────────────────────────────────────┤");

    for package in packages {
        let features_str = if package.features.is_empty() {
            "default".dimmed().to_string()
        } else {
            package.features.join(", ")
        };

        println!("│ {} │ {} │ {} │",
            format!("{:25}", package.name),
            format!("{:15}", package.version),
            format!("{:30}", features_str)
        );
    }

    println!("└─────────────────────────────────────────────────────────────────────────────┘");

    Ok(())
}

async fn update_packages() -> Result<()> {
    CommandUtils::info("Updating packages...");

    // Run cargo update
    let output = std::process::Command::new("cargo")
        .args(&["update"])
        .output()?;

    if output.status.success() {
        let stdout = String::from_utf8_lossy(&output.stdout);
        if !stdout.is_empty() {
            println!("{}", stdout);
        }

        CommandUtils::success("Packages updated successfully");

        // Show update summary
        show_update_summary().await?;
    } else {
        let stderr = String::from_utf8_lossy(&output.stderr);
        CommandUtils::error(&format!("Failed to update packages: {}", stderr));
    }

    Ok(())
}

#[derive(Debug, Clone)]
struct PackageInfo {
    name: String,
    version: String,
    features: Vec<String>,
    description: Option<String>,
}

fn is_package_installed(name: &str) -> Result<bool> {
    let cargo_toml_content = std::fs::read_to_string("Cargo.toml")?;
    let cargo_toml: toml::Value = toml::from_str(&cargo_toml_content)?;

    if let Some(deps) = cargo_toml.get("dependencies").and_then(|v| v.as_table()) {
        Ok(deps.contains_key(name))
    } else {
        Ok(false)
    }
}

fn is_valid_package_name(name: &str) -> bool {
    // Basic validation for package names
    !name.is_empty() &&
    name.chars().all(|c| c.is_alphanumeric() || c == '_' || c == '-') &&
    !name.starts_with('-') &&
    !name.ends_with('-')
}

fn add_dependency_to_cargo_toml(name: &str, version: &str) -> Result<()> {
    // This is handled by cargo add command
    // Additional custom logic can be added here
    Ok(())
}

fn remove_dependency_from_cargo_toml(name: &str) -> Result<()> {
    // This is handled by cargo remove command
    // Additional custom logic can be added here
    Ok(())
}

fn update_package_registry(name: &str, version: &str) -> Result<()> {
    // Update local package registry file
    let registry_path = ".rustisan/packages.json";

    CommandUtils::ensure_directory(&std::path::Path::new(".rustisan"))?;

    let mut registry: std::collections::HashMap<String, String> = if std::path::Path::new(registry_path).exists() {
        let content = std::fs::read_to_string(registry_path)?;
        serde_json::from_str(&content).unwrap_or_default()
    } else {
        std::collections::HashMap::new()
    };

    registry.insert(name.to_string(), version.to_string());

    let registry_content = serde_json::to_string_pretty(&registry)?;
    std::fs::write(registry_path, registry_content)?;

    Ok(())
}

fn remove_from_package_registry(name: &str) -> Result<()> {
    let registry_path = ".rustisan/packages.json";

    if std::path::Path::new(registry_path).exists() {
        let content = std::fs::read_to_string(registry_path)?;
        let mut registry: std::collections::HashMap<String, String> = serde_json::from_str(&content)?;

        registry.remove(name);

        let registry_content = serde_json::to_string_pretty(&registry)?;
        std::fs::write(registry_path, registry_content)?;
    }

    Ok(())
}

fn get_installed_packages() -> Result<Vec<PackageInfo>> {
    let cargo_toml_content = std::fs::read_to_string("Cargo.toml")?;
    let cargo_toml: toml::Value = toml::from_str(&cargo_toml_content)?;

    let mut packages = Vec::new();

    if let Some(deps) = cargo_toml.get("dependencies").and_then(|v| v.as_table()) {
        for (name, info) in deps {
            let (version, features) = match info {
                toml::Value::String(v) => (v.clone(), Vec::new()),
                toml::Value::Table(t) => {
                    let version = t.get("version")
                        .and_then(|v| v.as_str())
                        .unwrap_or("*")
                        .to_string();

                    let features = t.get("features")
                        .and_then(|v| v.as_array())
                        .map(|arr| {
                            arr.iter()
                                .filter_map(|v| v.as_str())
                                .map(|s| s.to_string())
                                .collect()
                        })
                        .unwrap_or_default();

                    (version, features)
                }
                _ => ("*".to_string(), Vec::new()),
            };

            packages.push(PackageInfo {
                name: name.clone(),
                version,
                features,
                description: None,
            });
        }
    }

    packages.sort_by(|a, b| a.name.cmp(&b.name));

    Ok(packages)
}

async fn show_package_info(name: &str) -> Result<()> {
    CommandUtils::info(&format!("Package {} information:", name));

    // Try to get package info from cargo
    let output = std::process::Command::new("cargo")
        .args(&["search", name, "--limit", "1"])
        .output()?;

    if output.status.success() {
        let stdout = String::from_utf8_lossy(&output.stdout);
        if !stdout.is_empty() {
            println!("{}", stdout);
        }
    }

    Ok(())
}

async fn show_update_summary() -> Result<()> {
    CommandUtils::info("Update summary:");

    // Read Cargo.lock to get version information
    if let Ok(lock_content) = std::fs::read_to_string("Cargo.lock") {
        let lock: toml::Value = toml::from_str(&lock_content)?;

        if let Some(packages) = lock.get("package").and_then(|v| v.as_array()) {
            println!("\n{} packages in dependency tree", packages.len());
        }
    }

    Ok(())
}

/// Search for packages in the registry
pub async fn search_packages(query: &str) -> Result<()> {
    CommandUtils::info(&format!("Searching for packages matching: {}", query));

    let output = std::process::Command::new("cargo")
        .args(&["search", query])
        .output()?;

    if output.status.success() {
        let stdout = String::from_utf8_lossy(&output.stdout);
        println!("{}", stdout);
    } else {
        let stderr = String::from_utf8_lossy(&output.stderr);
        CommandUtils::error(&format!("Search failed: {}", stderr));
    }

    Ok(())
}

/// Check for outdated packages
pub async fn check_outdated() -> Result<()> {
    CommandUtils::info("Checking for outdated packages...");

    // This would require parsing Cargo.lock and comparing with registry
    // For now, suggest using cargo-outdated
    CommandUtils::info("Install cargo-outdated for detailed outdated package information:");
    CommandUtils::info("cargo install cargo-outdated");
    CommandUtils::info("Then run: cargo outdated");

    Ok(())
}
