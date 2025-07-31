//! Build command implementations for the Rustisan CLI

use anyhow::Result;
use colored::*;
use super::CommandUtils;

/// Handle build command
pub async fn handle(env: String, optimize: bool, output: Option<String>) -> Result<()> {
    CommandUtils::ensure_rustisan_project()?;

    CommandUtils::info(&format!("Building application for {} environment", env));

    if optimize {
        CommandUtils::info("Optimizations enabled");
    }

    if let Some(ref output_dir) = output {
        CommandUtils::info(&format!("Output directory: {}", output_dir));
    }

    build_application(&env, optimize, output).await
}

async fn build_application(env: &str, optimize: bool, output: Option<String>) -> Result<()> {
    // Set environment variables
    unsafe {
        std::env::set_var("RUSTISAN_ENV", env);
        std::env::set_var("APP_ENV", env);
    }

    // Determine build profile
    let profile = if optimize || env == "production" {
        "release"
    } else {
        "debug"
    };

    CommandUtils::info(&format!("Using build profile: {}", profile));

    // Clean previous build if in production
    if env == "production" {
        CommandUtils::info("Cleaning previous build...");
        clean_build().await?;
    }

    // Cache configuration
    CommandUtils::info("Caching configuration...");
    cache_configuration().await?;

    // Build the application
    CommandUtils::info("Compiling application...");
    compile_application(profile).await?;

    // Copy assets and resources
    CommandUtils::info("Processing assets...");
    process_assets().await?;

    // Generate optimized autoloads
    CommandUtils::info("Generating autoloads...");
    generate_autoloads().await?;

    // Copy built files to output directory if specified
    if let Some(output_dir) = output {
        CommandUtils::info(&format!("Copying build to: {}", output_dir));
        copy_to_output(&output_dir, profile).await?;
    }

    print_build_summary(env, profile);

    CommandUtils::success("Build completed successfully");

    Ok(())
}

async fn clean_build() -> Result<()> {
    let output = std::process::Command::new("cargo")
        .args(&["clean"])
        .output()?;

    if !output.status.success() {
        let stderr = String::from_utf8_lossy(&output.stderr);
        anyhow::bail!("Failed to clean build: {}", stderr);
    }

    Ok(())
}

async fn cache_configuration() -> Result<()> {
    // Load and cache all configuration files
    let config_files = [
        "config/app.toml",
        "config/database.toml",
        "config/cache.toml",
        "config/mail.toml",
        "config/queue.toml",
        "config/logging.toml",
    ];

    let mut cached_config = std::collections::HashMap::new();

    for config_file in &config_files {
        if CommandUtils::file_exists(config_file) {
            let content = CommandUtils::read_file(config_file)?;
            match toml::from_str::<toml::Value>(&content) {
                Ok(value) => {
                    let key = std::path::Path::new(config_file)
                        .file_stem()
                        .and_then(|s| s.to_str())
                        .unwrap_or("unknown");
                    cached_config.insert(key.to_string(), value);
                }
                Err(e) => {
                    CommandUtils::warning(&format!("Failed to parse {}: {}", config_file, e));
                }
            }
        }
    }

    // Ensure bootstrap/cache directory exists
    CommandUtils::ensure_directory(&std::path::Path::new("bootstrap/cache"))?;

    // Write cached configuration
    let cache_data = serde_json::to_string_pretty(&cached_config)?;
    CommandUtils::write_file("bootstrap/cache/config.json", &cache_data)?;

    Ok(())
}

async fn compile_application(profile: &str) -> Result<()> {
    let mut args = vec!["build"];

    if profile == "release" {
        args.push("--release");
    }

    let output = std::process::Command::new("cargo")
        .args(&args)
        .output()?;

    if !output.status.success() {
        let stderr = String::from_utf8_lossy(&output.stderr);
        anyhow::bail!("Compilation failed: {}", stderr);
    }

    let stdout = String::from_utf8_lossy(&output.stdout);
    if !stdout.is_empty() {
        println!("{}", stdout);
    }

    Ok(())
}

async fn process_assets() -> Result<()> {
    let assets_dir = std::path::Path::new("assets");
    let public_dir = std::path::Path::new("public");

    // Create public directory if it doesn't exist
    CommandUtils::ensure_directory(public_dir)?;

    // Copy static assets
    if assets_dir.exists() {
        copy_directory(assets_dir, public_dir)?;
    }

    // Process CSS and JavaScript files
    process_css_files().await?;
    process_js_files().await?;

    Ok(())
}

async fn process_css_files() -> Result<()> {
    // TODO: Implement CSS processing (minification, bundling, etc.)
    Ok(())
}

async fn process_js_files() -> Result<()> {
    // TODO: Implement JavaScript processing (minification, bundling, etc.)
    Ok(())
}

async fn generate_autoloads() -> Result<()> {
    // TODO: Implement autoload generation for optimized class loading
    Ok(())
}

async fn copy_to_output(output_dir: &str, profile: &str) -> Result<()> {
    let output_path = std::path::Path::new(output_dir);
    CommandUtils::ensure_directory(output_path)?;

    // Copy binary
    let binary_src = format!("target/{}/rustisan", profile);
    let binary_dst = output_path.join("rustisan");

    if CommandUtils::file_exists(&binary_src) {
        use crate::utils::FileUtils;
        FileUtils::copy_file(&binary_src, &binary_dst)?;
    }

    // Copy configuration cache
    let config_cache = "bootstrap/cache/config.json";
    if CommandUtils::file_exists(config_cache) {
        let cache_dst = output_path.join("config.json");
        use crate::utils::FileUtils;
        FileUtils::copy_file(config_cache, cache_dst)?;
    }

    // Copy public assets
    let public_dir = std::path::Path::new("public");
    if public_dir.exists() {
        let public_dst = output_path.join("public");
        copy_directory(public_dir, &public_dst)?;
    }

    Ok(())
}

fn copy_directory(src: &std::path::Path, dst: &std::path::Path) -> Result<()> {
    if !src.exists() {
        return Ok(());
    }

    CommandUtils::ensure_directory(dst)?;

    for entry in walkdir::WalkDir::new(src) {
        let entry = entry?;
        let path = entry.path();

        if path.is_file() {
            let relative_path = path.strip_prefix(src)?;
            let dst_path = dst.join(relative_path);

            if let Some(parent) = dst_path.parent() {
                CommandUtils::ensure_directory(parent)?;
            }

            use crate::utils::FileUtils;
            FileUtils::copy_file(path, dst_path)?;
        }
    }

    Ok(())
}

fn print_build_summary(env: &str, profile: &str) {
    println!("\n{}", "Build Summary:".bold());
    println!("┌─────────────────────────────────────────────────────────────────────────────┐");
    println!("│ {} │ {} │", "Environment".bold(), env);
    println!("│ {} │ {} │", "Profile".bold(), profile);
    println!("│ {} │ {} │", "Target".bold(), get_target_info());
    println!("│ {} │ {} │", "Binary Size".bold(), get_binary_size(profile));
    println!("└─────────────────────────────────────────────────────────────────────────────┘");
}

fn get_target_info() -> String {
    std::env::var("TARGET").unwrap_or_else(|_| "unknown".to_string())
}

fn get_binary_size(profile: &str) -> String {
    let binary_path = format!("target/{}/rustisan", profile);

    if let Ok(metadata) = std::fs::metadata(&binary_path) {
        let size = metadata.len();
        format_size(size)
    } else {
        "unknown".to_string()
    }
}

fn format_size(size: u64) -> String {
    const UNITS: &[&str] = &["B", "KB", "MB", "GB"];
    let mut size = size as f64;
    let mut unit_index = 0;

    while size >= 1024.0 && unit_index < UNITS.len() - 1 {
        size /= 1024.0;
        unit_index += 1;
    }

    format!("{:.2} {}", size, UNITS[unit_index])
}

/// Build for production with optimizations
pub async fn build_production() -> Result<()> {
    handle("production".to_string(), true, None).await
}

/// Build for development
pub async fn build_development() -> Result<()> {
    handle("development".to_string(), false, None).await
}

/// Build with specific target
pub async fn build_target(target: &str) -> Result<()> {
    CommandUtils::info(&format!("Building for target: {}", target));

    let output = std::process::Command::new("cargo")
        .args(&["build", "--target", target, "--release"])
        .output()?;

    if !output.status.success() {
        let stderr = String::from_utf8_lossy(&output.stderr);
        anyhow::bail!("Cross-compilation failed: {}", stderr);
    }

    CommandUtils::success(&format!("Successfully built for target: {}", target));

    Ok(())
}
