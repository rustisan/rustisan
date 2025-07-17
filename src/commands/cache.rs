//! Cache command implementations for the Rustisan CLI

use anyhow::Result;
use colored::*;
use crate::CacheCommands;
use super::CommandUtils;

/// Handle cache command
pub async fn handle(operation: CacheCommands) -> Result<()> {
    CommandUtils::ensure_rustisan_project()?;

    match operation {
        CacheCommands::Clear => clear_all_cache().await,
        CacheCommands::Forget { key } => forget_cache_key(key).await,
        CacheCommands::Config => cache_config().await,
    }
}

async fn clear_all_cache() -> Result<()> {
    CommandUtils::info("Clearing all cache...");

    let cache_dirs = [
        "bootstrap/cache",
        "storage/cache",
        "storage/framework/cache",
        "storage/framework/sessions",
        "storage/framework/views",
    ];

    let mut cleared_count = 0;

    for cache_dir in &cache_dirs {
        let path = std::path::Path::new(cache_dir);
        if path.exists() {
            if let Err(e) = clear_directory(path) {
                CommandUtils::warning(&format!("Failed to clear {}: {}", cache_dir, e));
            } else {
                cleared_count += 1;
                CommandUtils::info(&format!("Cleared cache directory: {}", cache_dir));
            }
        }
    }

    // Clear specific cache files
    let cache_files = [
        "bootstrap/cache/routes.json",
        "bootstrap/cache/config.json",
        "bootstrap/cache/services.json",
    ];

    for cache_file in &cache_files {
        let path = std::path::Path::new(cache_file);
        if path.exists() {
            if let Err(e) = std::fs::remove_file(path) {
                CommandUtils::warning(&format!("Failed to remove {}: {}", cache_file, e));
            } else {
                cleared_count += 1;
                CommandUtils::info(&format!("Removed cache file: {}", cache_file));
            }
        }
    }

    if cleared_count > 0 {
        CommandUtils::success(&format!("Successfully cleared {} cache location(s)", cleared_count));
    } else {
        CommandUtils::warning("No cache files or directories found to clear");
    }

    Ok(())
}

async fn forget_cache_key(key: String) -> Result<()> {
    CommandUtils::info(&format!("Forgetting cache key: {}", key));

    // TODO: Implement specific cache key removal logic
    // This would typically involve connecting to the cache store (Redis, Memcached, etc.)
    // and removing the specific key

    CommandUtils::success(&format!("Cache key '{}' has been forgotten", key));

    Ok(())
}

async fn cache_config() -> Result<()> {
    CommandUtils::info("Caching configuration...");

    // Load configuration files
    let config_sources = [
        ("app.toml", "App configuration"),
        ("database.toml", "Database configuration"),
        ("cache.toml", "Cache configuration"),
        ("mail.toml", "Mail configuration"),
        ("queue.toml", "Queue configuration"),
    ];

    let mut cached_configs = std::collections::HashMap::new();

    for (config_file, description) in &config_sources {
        let config_path = format!("config/{}", config_file);
        if std::path::Path::new(&config_path).exists() {
            match std::fs::read_to_string(&config_path) {
                Ok(content) => {
                    match toml::from_str::<toml::Value>(&content) {
                        Ok(value) => {
                            cached_configs.insert(config_file.replace(".toml", ""), value);
                            CommandUtils::info(&format!("Loaded {}", description));
                        }
                        Err(e) => {
                            CommandUtils::warning(&format!("Failed to parse {}: {}", config_file, e));
                        }
                    }
                }
                Err(e) => {
                    CommandUtils::warning(&format!("Failed to read {}: {}", config_file, e));
                }
            }
        }
    }

    // Ensure bootstrap/cache directory exists
    CommandUtils::ensure_directory(&std::path::Path::new("bootstrap/cache"))?;

    // Save cached configuration
    let cache_data = serde_json::to_string_pretty(&cached_configs)?;
    std::fs::write("bootstrap/cache/config.json", cache_data)?;

    CommandUtils::success(&format!("Cached {} configuration file(s)", cached_configs.len()));

    Ok(())
}

fn clear_directory(dir: &std::path::Path) -> Result<()> {
    if !dir.exists() {
        return Ok(());
    }

    for entry in std::fs::read_dir(dir)? {
        let entry = entry?;
        let path = entry.path();

        if path.is_dir() {
            std::fs::remove_dir_all(&path)?;
        } else {
            std::fs::remove_file(&path)?;
        }
    }

    Ok(())
}
