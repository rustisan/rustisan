//! Configuration management commands for the Rustisan CLI
//!
//! This module provides commands for managing the rustisan.toml configuration file.

use anyhow::Result;
use colored::*;
use std::fs;
use std::path::Path;
use toml::Value;
use rand::Rng;
use base64::{Engine as _, engine::general_purpose};

use super::CommandUtils;
use crate::ConfigCommands;

/// Handle configuration commands
pub async fn handle(operation: ConfigCommands) -> Result<()> {
    match operation {
        ConfigCommands::Show => show_config().await,
        ConfigCommands::Get { key } => get_config_value(key).await,
        ConfigCommands::Set { key, value } => set_config_value(key, value).await,
        ConfigCommands::GenerateKey => generate_app_key().await,
        ConfigCommands::Validate => validate_config().await,
        ConfigCommands::Reset => reset_config().await,
    }
}

/// Show all configuration values
async fn show_config() -> Result<()> {
    CommandUtils::ensure_rustisan_project()?;

    let config_path = "rustisan.toml";
    if !Path::new(config_path).exists() {
        return Err(anyhow::anyhow!("rustisan.toml not found. This doesn't appear to be a Rustisan project."));
    }

    let content = fs::read_to_string(config_path)?;
    let config: Value = toml::from_str(&content)?;

    CommandUtils::info("Current configuration (rustisan.toml):");
    println!();

    display_config_section(&config, "", 0);

    println!();
    CommandUtils::info("Use 'rustisan config:set KEY VALUE' to modify configuration values");
    CommandUtils::info("Use 'rustisan config:generate-key' to generate a new application key");

    Ok(())
}

/// Display configuration section recursively
fn display_config_section(value: &Value, prefix: &str, indent: usize) {
    let indent_str = "  ".repeat(indent);

    match value {
        Value::Table(table) => {
            for (key, val) in table {
                let full_key = if prefix.is_empty() {
                    key.clone()
                } else {
                    format!("{}.{}", prefix, key)
                };

                match val {
                    Value::Table(_) => {
                        println!("{}[{}]", indent_str, full_key.cyan().bold());
                        display_config_section(val, &full_key, indent + 1);
                    }
                    _ => {
                        let display_value = if is_sensitive_key(&full_key) {
                            if val.as_str().unwrap_or("").is_empty() {
                                "".dimmed().to_string()
                            } else {
                                "••••••••".dimmed().to_string()
                            }
                        } else {
                            format_value(val)
                        };
                        println!("{}{} = {}", indent_str, key.cyan().bold(), display_value);
                    }
                }
            }
        }
        _ => {
            let display_value = if is_sensitive_key(prefix) {
                if value.as_str().unwrap_or("").is_empty() {
                    "".dimmed().to_string()
                } else {
                    "••••••••".dimmed().to_string()
                }
            } else {
                format_value(value)
            };
            println!("{}{} = {}", indent_str, prefix.cyan().bold(), display_value);
        }
    }
}

/// Get a specific configuration value
async fn get_config_value(key: String) -> Result<()> {
    CommandUtils::ensure_rustisan_project()?;

    let config_path = "rustisan.toml";
    if !Path::new(config_path).exists() {
        return Err(anyhow::anyhow!("rustisan.toml not found."));
    }

    let content = fs::read_to_string(config_path)?;
    let config: Value = toml::from_str(&content)?;

    let value = get_nested_value(&config, &key);

    match value {
        Some(val) => {
            let display_value = if is_sensitive_key(&key) {
                if val.as_str().unwrap_or("").is_empty() {
                    "".dimmed().to_string()
                } else {
                    "••••••••".dimmed().to_string()
                }
            } else {
                format_value(val)
            };
            println!("{} = {}", key.cyan().bold(), display_value);
        }
        None => {
            CommandUtils::warning(&format!("Configuration key '{}' not found", key));
        }
    }

    Ok(())
}

/// Set a configuration value
async fn set_config_value(key: String, value: String) -> Result<()> {
    CommandUtils::ensure_rustisan_project()?;

    let config_path = "rustisan.toml";
    if !Path::new(config_path).exists() {
        return Err(anyhow::anyhow!("rustisan.toml not found."));
    }

    let content = fs::read_to_string(config_path)?;
    let mut config: Value = toml::from_str(&content)?;

    // Parse the value to the appropriate type
    let parsed_value = parse_config_value(&value);

    // Set the nested value
    set_nested_value(&mut config, &key, parsed_value)?;

    // Write back to file
    let new_content = toml::to_string_pretty(&config)?;
    fs::write(config_path, new_content)?;

    CommandUtils::success(&format!("Configuration key '{}' updated successfully", key.cyan().bold()));

    Ok(())
}

/// Generate a new application key
async fn generate_app_key() -> Result<()> {
    CommandUtils::ensure_rustisan_project()?;

    CommandUtils::info("Generating new application key...");

    // Generate 32 random bytes
    let mut rng = rand::thread_rng();
    let mut key_bytes = [0u8; 32];
    rng.fill(&mut key_bytes);

    // Encode as base64
    let key = format!("base64:{}", general_purpose::STANDARD.encode(&key_bytes));

    // Set the APP_KEY in rustisan.toml
    set_config_value("app.key".to_string(), key.clone()).await?;

    CommandUtils::success("Application key generated successfully!");
    CommandUtils::info(&format!("New key: {}", key.dimmed()));

    println!();
    CommandUtils::warning("Make sure to update your production configuration with the new key!");

    Ok(())
}

/// Validate configuration
async fn validate_config() -> Result<()> {
    CommandUtils::ensure_rustisan_project()?;

    CommandUtils::info("Validating rustisan.toml configuration...");

    let config_path = "rustisan.toml";
    if !Path::new(config_path).exists() {
        return Err(anyhow::anyhow!("rustisan.toml not found."));
    }

    let content = fs::read_to_string(config_path)?;
    let config: Value = toml::from_str(&content)
        .map_err(|e| anyhow::anyhow!("Invalid TOML syntax: {}", e))?;

    let mut errors = Vec::new();
    let mut warnings = Vec::new();

    // Required configuration keys
    let required_keys = vec![
        "app.name",
        "app.env",
        "app.key",
        "database.default",
        "database.connections.default.driver",
        "database.connections.default.host",
        "database.connections.default.database",
    ];

    // Check required keys
    for key in required_keys {
        if let Some(value) = get_nested_value(&config, key) {
            if value.as_str().unwrap_or("").is_empty() {
                warnings.push(format!("'{}' is empty", key));
            }
        } else {
            errors.push(format!("Required key '{}' is missing", key));
        }
    }

    // Validate app.key format
    if let Some(app_key) = get_nested_value(&config, "app.key") {
        if let Some(key_str) = app_key.as_str() {
            if !key_str.starts_with("base64:") && !key_str.is_empty() {
                warnings.push("app.key should start with 'base64:' for proper encoding".to_string());
            }
            if key_str.len() < 32 {
                warnings.push("app.key appears to be too short for security".to_string());
            }
        }
    }

    // Validate database driver
    if let Some(driver) = get_nested_value(&config, "database.connections.default.driver") {
        if let Some(driver_str) = driver.as_str() {
            if !["mysql", "postgres", "sqlite"].contains(&driver_str) {
                warnings.push(format!("Unsupported database driver: {}", driver_str));
            }
        }
    }

    // Validate environment
    if let Some(env_val) = get_nested_value(&config, "app.env") {
        if let Some(env_str) = env_val.as_str() {
            if !["development", "testing", "production"].contains(&env_str) {
                warnings.push(format!("Unknown environment: {}", env_str));
            }

            // Production-specific checks
            if env_str == "production" {
                if let Some(debug_val) = get_nested_value(&config, "app.debug") {
                    if debug_val.as_bool().unwrap_or(false) {
                        errors.push("app.debug should be false in production".to_string());
                    }
                }

                if let Some(log_level) = get_nested_value(&config, "logging.level") {
                    if let Some(level_str) = log_level.as_str() {
                        if level_str == "debug" || level_str == "trace" {
                            warnings.push("Consider using 'info' or 'warn' log level in production".to_string());
                        }
                    }
                }
            }
        }
    }

    // Validate port numbers
    if let Some(port) = get_nested_value(&config, "server.port") {
        if let Some(port_num) = port.as_integer() {
            if port_num < 1 || port_num > 65535 {
                errors.push("server.port must be between 1 and 65535".to_string());
            }
        }
    }

    // Display results
    if errors.is_empty() && warnings.is_empty() {
        CommandUtils::success("Configuration is valid!");
    } else {
        if !errors.is_empty() {
            CommandUtils::error("Configuration errors found:");
            for error in &errors {
                println!("  {} {}", "✗".red(), error);
            }
        }

        if !warnings.is_empty() {
            CommandUtils::warning("Configuration warnings:");
            for warning in &warnings {
                println!("  {} {}", "⚠".yellow(), warning);
            }
        }

        if !errors.is_empty() {
            return Err(anyhow::anyhow!("Configuration validation failed"));
        }
    }

    Ok(())
}

/// Reset configuration to defaults
async fn reset_config() -> Result<()> {
    CommandUtils::ensure_rustisan_project()?;

    CommandUtils::warning("This will reset rustisan.toml to default values!");
    CommandUtils::info("Press Enter to continue or Ctrl+C to cancel...");

    let mut input = String::new();
    std::io::stdin().read_line(&mut input)?;

    let default_config = create_default_config();
    fs::write("rustisan.toml", default_config)?;

    CommandUtils::success("Configuration reset to defaults!");
    CommandUtils::info("Don't forget to:");
    println!("  1. Configure your database connection");
    println!("  2. Generate a new application key with: rustisan config:generate-key");
    println!("  3. Update other environment-specific settings");

    Ok(())
}

/// Get nested value from TOML structure
fn get_nested_value<'a>(config: &'a Value, key: &str) -> Option<&'a Value> {
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

    Some(current)
}

/// Set nested value in TOML structure
fn set_nested_value(config: &mut Value, key: &str, value: Value) -> Result<()> {
    let parts: Vec<&str> = key.split('.').collect();
    let mut current = config;

    // Navigate to the parent table
    for part in &parts[..parts.len() - 1] {
        if let Value::Table(table) = current {
            current = table.entry(part.to_string()).or_insert(Value::Table(toml::map::Map::new()));
        } else {
            return Err(anyhow::anyhow!("Cannot navigate: intermediate value is not a table"));
        }
    }

    // Set the final value
    if let Some(last_part) = parts.last() {
        if let Value::Table(table) = current {
            table.insert(last_part.to_string(), value);
        } else {
            return Err(anyhow::anyhow!("Cannot set value: parent is not a table"));
        }
    }

    Ok(())
}

/// Parse string value to appropriate TOML type
fn parse_config_value(value: &str) -> Value {
    // Try to parse as boolean
    if value.to_lowercase() == "true" {
        return Value::Boolean(true);
    }
    if value.to_lowercase() == "false" {
        return Value::Boolean(false);
    }

    // Try to parse as integer
    if let Ok(int_val) = value.parse::<i64>() {
        return Value::Integer(int_val);
    }

    // Try to parse as float
    if let Ok(float_val) = value.parse::<f64>() {
        return Value::Float(float_val);
    }

    // Default to string
    Value::String(value.to_string())
}

/// Format value for display
fn format_value(value: &Value) -> String {
    match value {
        Value::String(s) => s.clone(),
        Value::Integer(i) => i.to_string(),
        Value::Float(f) => f.to_string(),
        Value::Boolean(b) => b.to_string(),
        Value::Array(arr) => format!("[{}]", arr.iter().map(format_value).collect::<Vec<_>>().join(", ")),
        Value::Table(_) => "{...}".to_string(),
        Value::Datetime(dt) => dt.to_string(),
    }
}

/// Check if a configuration key contains sensitive information
fn is_sensitive_key(key: &str) -> bool {
    let sensitive_keys = [
        "app.key",
        "database.connections.default.password",
        "cache.redis.password",
        "jwt_secret",
        "mail_password",
        "aws_secret_access_key",
        "sentry_dsn",
        "api_key",
        "secret",
        "token",
        "private_key",
    ];

    let key_lower = key.to_lowercase();

    sensitive_keys.iter().any(|&sensitive| {
        key_lower.contains(&sensitive.replace('_', ".")) || key_lower.contains(sensitive)
    })
}

/// Create default configuration content
fn create_default_config() -> String {
    r#"[app]
name = "Rustisan App"
env = "development"
debug = true
url = "http://localhost:3001"
timezone = "UTC"
locale = "en"
key = ""
cors_enabled = true

[server]
host = "127.0.0.1"
port = 3001
timeout = 60
max_connections = 1000
https_enabled = false

[database]
default = "default"

[database.connections.default]
driver = "mysql"
host = "localhost"
port = 3306
database = "rustisan_app"
username = "root"
password = ""
charset = "utf8mb4"
pool_min = 1
pool_max = 10
timeout = 30

[cache]
default = "memory"
ttl = 3600

[session]
driver = "cookie"
lifetime = 120
expire_on_close = false
encrypt = true
cookie_name = "rustisan_session"
cookie_path = "/"
cookie_secure = false
cookie_http_only = true

[logging]
level = "info"
default = "console"

# Additional configuration sections can be added here
# For example:
# [mail]
# driver = "smtp"
# host = "localhost"
# port = 587
# encryption = "tls"
# username = ""
# password = ""

# [redis]
# host = "localhost"
# port = 6379
# password = ""
# database = 0

# [api]
# rate_limit_enabled = true
# rate_limit_max_requests = 60
# rate_limit_window = 60
# default_version = "v1"
# prefix = "api"
"#.to_string()
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_is_sensitive_key() {
        assert!(is_sensitive_key("app.key"));
        assert!(is_sensitive_key("database.connections.default.password"));
        assert!(is_sensitive_key("cache.redis.password"));
        assert!(!is_sensitive_key("app.name"));
        assert!(!is_sensitive_key("database.connections.default.host"));
        assert!(!is_sensitive_key("logging.level"));
    }

    #[test]
    fn test_parse_config_value() {
        assert_eq!(parse_config_value("true"), Value::Boolean(true));
        assert_eq!(parse_config_value("false"), Value::Boolean(false));
        assert_eq!(parse_config_value("42"), Value::Integer(42));
        assert_eq!(parse_config_value("3.14"), Value::Float(3.14));
        assert_eq!(parse_config_value("hello"), Value::String("hello".to_string()));
    }

    #[test]
    fn test_format_value() {
        assert_eq!(format_value(&Value::String("test".to_string())), "test");
        assert_eq!(format_value(&Value::Integer(42)), "42");
        assert_eq!(format_value(&Value::Boolean(true)), "true");
    }
}
