//! Info command implementations for the Rustisan CLI

use anyhow::Result;
use colored::*;
use super::CommandUtils;

/// Handle info command
pub async fn handle(detailed: bool) -> Result<()> {
    CommandUtils::ensure_rustisan_project()?;

    if detailed {
        show_detailed_info().await
    } else {
        show_basic_info().await
    }
}

async fn show_basic_info() -> Result<()> {
    CommandUtils::info("Gathering application information...");

    let app_info = gather_app_info()?;
    let system_info = gather_system_info()?;

    print_app_header(&app_info);
    print_basic_info(&app_info, &system_info);

    Ok(())
}

async fn show_detailed_info() -> Result<()> {
    CommandUtils::info("Gathering detailed application information...");

    let app_info = gather_app_info()?;
    let system_info = gather_system_info()?;
    let dependencies = gather_dependencies()?;
    let environment = gather_environment_info()?;

    print_app_header(&app_info);
    print_detailed_info(&app_info, &system_info, &dependencies, &environment);

    Ok(())
}

#[derive(Debug)]
struct AppInfo {
    name: String,
    version: String,
    description: String,
    authors: Vec<String>,
    edition: String,
    repository: Option<String>,
    license: Option<String>,
}

#[derive(Debug)]
struct SystemInfo {
    rustc_version: String,
    cargo_version: String,
    target_triple: String,
    os: String,
    architecture: String,
}

#[derive(Debug)]
struct DependencyInfo {
    name: String,
    version: String,
    features: Vec<String>,
}

fn gather_app_info() -> Result<AppInfo> {
    let cargo_toml_content = std::fs::read_to_string("Cargo.toml")?;
    let cargo_toml: toml::Value = toml::from_str(&cargo_toml_content)?;

    let package = cargo_toml.get("package")
        .ok_or_else(|| anyhow::anyhow!("Invalid Cargo.toml: missing [package] section"))?;

    let name = package.get("name")
        .and_then(|v| v.as_str())
        .unwrap_or("unknown")
        .to_string();

    let version = package.get("version")
        .and_then(|v| v.as_str())
        .unwrap_or("unknown")
        .to_string();

    let description = package.get("description")
        .and_then(|v| v.as_str())
        .unwrap_or("No description available")
        .to_string();

    let authors = package.get("authors")
        .and_then(|v| v.as_array())
        .map(|arr| {
            arr.iter()
                .filter_map(|v| v.as_str())
                .map(|s| s.to_string())
                .collect()
        })
        .unwrap_or_default();

    let edition = package.get("edition")
        .and_then(|v| v.as_str())
        .unwrap_or("2021")
        .to_string();

    let repository = package.get("repository")
        .and_then(|v| v.as_str())
        .map(|s| s.to_string());

    let license = package.get("license")
        .and_then(|v| v.as_str())
        .map(|s| s.to_string());

    Ok(AppInfo {
        name,
        version,
        description,
        authors,
        edition,
        repository,
        license,
    })
}

fn gather_system_info() -> Result<SystemInfo> {
    let rustc_version = get_rustc_version()?;
    let cargo_version = get_cargo_version()?;
    let target_triple = get_target_triple();
    let os = std::env::consts::OS.to_string();
    let architecture = std::env::consts::ARCH.to_string();

    Ok(SystemInfo {
        rustc_version,
        cargo_version,
        target_triple,
        os,
        architecture,
    })
}

fn gather_dependencies() -> Result<Vec<DependencyInfo>> {
    let cargo_toml_content = std::fs::read_to_string("Cargo.toml")?;
    let cargo_toml: toml::Value = toml::from_str(&cargo_toml_content)?;

    let mut dependencies = Vec::new();

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

            dependencies.push(DependencyInfo {
                name: name.clone(),
                version,
                features,
            });
        }
    }

    dependencies.sort_by(|a, b| a.name.cmp(&b.name));

    Ok(dependencies)
}

fn gather_environment_info() -> Result<std::collections::HashMap<String, String>> {
    let mut env_vars = std::collections::HashMap::new();

    // Common environment variables
    let common_vars = [
        "RUSTISAN_ENV",
        "APP_ENV",
        "DATABASE_URL",
        "REDIS_URL",
        "PORT",
        "HOST",
        "LOG_LEVEL",
        "RUST_LOG",
    ];

    for var in &common_vars {
        if let Ok(value) = std::env::var(var) {
            env_vars.insert(var.to_string(), value);
        }
    }

    Ok(env_vars)
}

fn get_rustc_version() -> Result<String> {
    let output = std::process::Command::new("rustc")
        .arg("--version")
        .output()?;

    if output.status.success() {
        let version = String::from_utf8_lossy(&output.stdout);
        Ok(version.trim().to_string())
    } else {
        Ok("unknown".to_string())
    }
}

fn get_cargo_version() -> Result<String> {
    let output = std::process::Command::new("cargo")
        .arg("--version")
        .output()?;

    if output.status.success() {
        let version = String::from_utf8_lossy(&output.stdout);
        Ok(version.trim().to_string())
    } else {
        Ok("unknown".to_string())
    }
}

fn get_target_triple() -> String {
    std::env::var("TARGET")
        .unwrap_or_else(|_| "unknown".to_string())
}

fn print_app_header(app_info: &AppInfo) {
    println!("\n{}", format!("🦀 {} v{}", app_info.name, app_info.version).bold().cyan());
    println!("{}", app_info.description.dimmed());

    if !app_info.authors.is_empty() {
        println!("{} {}", "Authors:".bold(), app_info.authors.join(", "));
    }

    if let Some(ref repo) = app_info.repository {
        println!("{} {}", "Repository:".bold(), repo);
    }

    if let Some(ref license) = app_info.license {
        println!("{} {}", "License:".bold(), license);
    }

    println!();
}

fn print_basic_info(app_info: &AppInfo, system_info: &SystemInfo) {
    println!("{}", "Application Information:".bold());
    println!("┌─────────────────────────────────────────────────────────────────────────────┐");
    println!("│ {} │ {} │", "Name".bold(), app_info.name);
    println!("│ {} │ {} │", "Version".bold(), app_info.version);
    println!("│ {} │ {} │", "Edition".bold(), app_info.edition);
    println!("└─────────────────────────────────────────────────────────────────────────────┘");

    println!("\n{}", "System Information:".bold());
    println!("┌─────────────────────────────────────────────────────────────────────────────┐");
    println!("│ {} │ {} │", "Rust Compiler".bold(), system_info.rustc_version);
    println!("│ {} │ {} │", "Cargo Version".bold(), system_info.cargo_version);
    println!("│ {} │ {} │", "Operating System".bold(), system_info.os);
    println!("│ {} │ {} │", "Architecture".bold(), system_info.architecture);
    println!("│ {} │ {} │", "Target Triple".bold(), system_info.target_triple);
    println!("└─────────────────────────────────────────────────────────────────────────────┘");
}

fn print_detailed_info(
    app_info: &AppInfo,
    system_info: &SystemInfo,
    dependencies: &[DependencyInfo],
    environment: &std::collections::HashMap<String, String>,
) {
    print_basic_info(app_info, system_info);

    // Dependencies
    if !dependencies.is_empty() {
        println!("\n{}", "Dependencies:".bold());
        println!("┌─────────────────────────────────────────────────────────────────────────────┐");
        println!("│ {} │ {} │ {} │", "Name".bold(), "Version".bold(), "Features".bold());
        println!("├─────────────────────────────────────────────────────────────────────────────┤");

        for dep in dependencies {
            let features_str = if dep.features.is_empty() {
                "default".dimmed().to_string()
            } else {
                dep.features.join(", ")
            };

            println!("│ {} │ {} │ {} │",
                format!("{:20}", dep.name),
                format!("{:10}", dep.version),
                format!("{:40}", features_str)
            );
        }

        println!("└─────────────────────────────────────────────────────────────────────────────┘");
    }

    // Environment Variables
    if !environment.is_empty() {
        println!("\n{}", "Environment Variables:".bold());
        println!("┌─────────────────────────────────────────────────────────────────────────────┐");
        println!("│ {} │ {} │", "Variable".bold(), "Value".bold());
        println!("├─────────────────────────────────────────────────────────────────────────────┤");

        let mut env_vars: Vec<_> = environment.iter().collect();
        env_vars.sort_by(|a, b| a.0.cmp(b.0));

        for (key, value) in env_vars {
            // Mask sensitive values
            let masked_value = if key.to_lowercase().contains("password") ||
                                key.to_lowercase().contains("secret") ||
                                key.to_lowercase().contains("token") ||
                                key.to_lowercase().contains("key") {
                "***".to_string()
            } else {
                value.clone()
            };

            println!("│ {} │ {} │",
                format!("{:20}", key),
                format!("{:50}", masked_value)
            );
        }

        println!("└─────────────────────────────────────────────────────────────────────────────┘");
    }

    // Project Statistics
    print_project_statistics();
}

fn print_project_statistics() {
    println!("\n{}", "Project Statistics:".bold());
    println!("┌─────────────────────────────────────────────────────────────────────────────┐");

    let stats = calculate_project_stats();

    println!("│ {} │ {} │", "Source Files".bold(), stats.source_files);
    println!("│ {} │ {} │", "Test Files".bold(), stats.test_files);
    println!("│ {} │ {} │", "Total Lines".bold(), stats.total_lines);
    println!("│ {} │ {} │", "Code Lines".bold(), stats.code_lines);
    println!("│ {} │ {} │", "Comment Lines".bold(), stats.comment_lines);
    println!("│ {} │ {} │", "Blank Lines".bold(), stats.blank_lines);

    println!("└─────────────────────────────────────────────────────────────────────────────┘");
}

#[derive(Debug, Default)]
struct ProjectStats {
    source_files: usize,
    test_files: usize,
    total_lines: usize,
    code_lines: usize,
    comment_lines: usize,
    blank_lines: usize,
}

fn calculate_project_stats() -> ProjectStats {
    let mut stats = ProjectStats::default();

    if let Ok(entries) = walkdir::WalkDir::new("src").into_iter().collect::<Result<Vec<_>, _>>() {
        for entry in entries {
            if let Some(path) = entry.path().to_str() {
                if path.ends_with(".rs") {
                    stats.source_files += 1;

                    if path.contains("test") {
                        stats.test_files += 1;
                    }

                    if let Ok(content) = std::fs::read_to_string(entry.path()) {
                        for line in content.lines() {
                            let trimmed = line.trim();
                            stats.total_lines += 1;

                            if trimmed.is_empty() {
                                stats.blank_lines += 1;
                            } else if trimmed.starts_with("//") || trimmed.starts_with("/*") {
                                stats.comment_lines += 1;
                            } else {
                                stats.code_lines += 1;
                            }
                        }
                    }
                }
            }
        }
    }

    stats
}
