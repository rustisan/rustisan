//! New command for creating Rustisan projects
//!
//! This command creates a new Rustisan application with the proper structure.

use anyhow::Result;
use colored::*;
use std::fs;
use std::path::{Path, PathBuf};
use std::process::Command;

use super::CommandUtils;

/// Handle the new command
pub async fn handle(name: String, path: Option<String>, template: Option<String>, git: bool) -> Result<()> {
    // Determine the project path
    let project_path = if let Some(p) = path {
        PathBuf::from(p).join(&name)
    } else {
        PathBuf::from(&name)
    };

    // Check if directory already exists
    if project_path.exists() {
        anyhow::bail!("Directory '{}' already exists", project_path.display());
    }

    CommandUtils::info(&format!("Creating new Rustisan application '{}'...", name));

    // Create project directory
    fs::create_dir_all(&project_path)?;

    // Create project structure
    create_project_structure(&project_path, &name, template.as_deref()).await?;

    // Initialize git repository if requested
    if git {
        initialize_git(&project_path)?;
    }

    CommandUtils::success(&format!("Successfully created Rustisan application '{}'", name));

    println!("\n{}", "Next steps:".bold());
    println!("  cd {}", name);
    println!("  rustisan config:generate-key  # Generate application key");
    println!("  # Configure your database in rustisan.toml");
    println!("  rustisan migrate           # Run database migrations");
    println!("  rustisan serve             # Start development server");

    Ok(())
}

/// Create the project structure
async fn create_project_structure(path: &Path, name: &str, template: Option<&str>) -> Result<()> {
    // Create main Cargo.toml
    create_main_cargo_toml(path, name)?;

    // Create rustisan.toml configuration
    create_rustisan_config(path)?;

    // Create .gitignore
    create_gitignore(path)?;

    // Create src directory structure
    create_src_structure(path)?;

    // Create other directories
    create_directory_structure(path)?;

    // Create main.rs
    create_main_rs(path, name)?;

    // Create README.md
    create_readme(path, name)?;

    // Apply template if specified
    if let Some(template_name) = template {
        apply_template(path, template_name).await?;
    }

    Ok(())
}

/// Create main Cargo.toml
fn create_main_cargo_toml(path: &Path, name: &str) -> Result<()> {
    let cargo_toml = format!(r#"[package]
name = "{}"
version = "0.1.0"
edition = "2024"
authors = ["Your Name <your.email@example.com>"]
description = "A Rustisan web application"

[dependencies]
rustisan-core = "0.0.1"
# rustisan-orm = "0.0.1"
# rustisan-macros = "0.1.0"

# Async runtime
tokio = {{ version = "1.0", features = ["full"] }}

# Web framework
axum = {{ version = "0.7", features = ["macros", "multipart"] }}

# Serialization
serde = {{ version = "1.0", features = ["derive"] }}
serde_json = "1.0"

# Database
sqlx = {{ version = "0.7", features = ["runtime-tokio-rustls", "mysql", "postgres", "chrono", "uuid"] }}

# Error handling
anyhow = "1.0"
thiserror = "1.0"

# Logging
tracing = "0.1"
tracing-subscriber = {{ version = "0.3", features = ["env-filter"] }}

# Configuration

# Time handling
chrono = {{ version = "0.4", features = ["serde"] }}

# UUID
uuid = {{ version = "1.0", features = ["v4", "serde"] }}

[dev-dependencies]
tokio-test = "0.4"

[[bin]]
name = "{}"
path = "src/main.rs"
"#, name, name);

    fs::write(path.join("Cargo.toml"), cargo_toml)?;
    Ok(())
}

/// Create rustisan.toml configuration
fn create_rustisan_config(path: &Path) -> Result<()> {
    let config = r#"[app]
name = "Rustisan App"
env = "development"
debug = true
url = "http://localhost:3000"
timezone = "UTC"
locale = "en"
key = ""
cors_enabled = true

[server]
host = "127.0.0.1"
port = 3000
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
"#;

    fs::write(path.join("rustisan.toml"), config)?;
    Ok(())
}



/// Create .gitignore
fn create_gitignore(path: &Path) -> Result<()> {
    let gitignore = r#"# Rust
/target/
Cargo.lock

# IDE
.vscode/
.idea/
*.swp
*.swo
*~

# OS
.DS_Store
Thumbs.db

# Environment
# No .env files needed - configuration is in rustisan.toml

# Logs
*.log
logs/

# Database
*.db
*.sqlite
*.sqlite3

# Cache
storage/cache/
storage/sessions/
storage/logs/

# Build artifacts
dist/
build/
"#;

    fs::write(path.join(".gitignore"), gitignore)?;
    Ok(())
}

/// Create src directory structure
fn create_src_structure(path: &Path) -> Result<()> {
    let src_path = path.join("src");
    fs::create_dir_all(&src_path)?;

    // Create subdirectories
    fs::create_dir_all(src_path.join("controllers"))?;
    fs::create_dir_all(src_path.join("models"))?;
    fs::create_dir_all(src_path.join("middleware"))?;
    fs::create_dir_all(src_path.join("requests"))?;
    fs::create_dir_all(src_path.join("resources"))?;
    fs::create_dir_all(src_path.join("services"))?;
    fs::create_dir_all(src_path.join("jobs"))?;
    fs::create_dir_all(src_path.join("events"))?;
    fs::create_dir_all(src_path.join("listeners"))?;

    // Create module files
    fs::write(src_path.join("controllers").join("mod.rs"), "//! Application controllers\n")?;
    fs::write(src_path.join("models").join("mod.rs"), "//! Application models\n")?;
    fs::write(src_path.join("middleware").join("mod.rs"), "//! Application middleware\n")?;
    fs::write(src_path.join("requests").join("mod.rs"), "//! Form request validators\n")?;
    fs::write(src_path.join("resources").join("mod.rs"), "//! API resources\n")?;
    fs::write(src_path.join("services").join("mod.rs"), "//! Application services\n")?;
    fs::write(src_path.join("jobs").join("mod.rs"), "//! Background jobs\n")?;
    fs::write(src_path.join("events").join("mod.rs"), "//! Application events\n")?;
    fs::write(src_path.join("listeners").join("mod.rs"), "//! Event listeners\n")?;

    Ok(())
}

/// Create other directory structure
fn create_directory_structure(path: &Path) -> Result<()> {
    // Database directories
    fs::create_dir_all(path.join("database").join("migrations"))?;
    fs::create_dir_all(path.join("database").join("seeders"))?;
    fs::create_dir_all(path.join("database").join("factories"))?;

    // Storage directories
    fs::create_dir_all(path.join("storage").join("logs"))?;
    fs::create_dir_all(path.join("storage").join("cache"))?;
    fs::create_dir_all(path.join("storage").join("sessions"))?;
    fs::create_dir_all(path.join("storage").join("uploads"))?;

    // Resources directories
    fs::create_dir_all(path.join("resources").join("views"))?;
    fs::create_dir_all(path.join("resources").join("assets"))?;

    // Config directory
    fs::create_dir_all(path.join("config"))?;

    // Tests directory
    fs::create_dir_all(path.join("tests").join("unit"))?;
    fs::create_dir_all(path.join("tests").join("integration"))?;

    // Routes directory
    fs::create_dir_all(path.join("routes"))?;

    Ok(())
}

/// Create main.rs
fn create_main_rs(path: &Path, name: &str) -> Result<()> {
    let main_rs = format!(r#"//! {} - A Rustisan web application
//!
//! This is the main entry point for the application.

use rustisan_core::{{Application, RustisanApp}};
use anyhow::Result;

mod controllers;
mod models;
mod middleware;
mod requests;
mod resources;
mod services;
mod jobs;
mod events;
mod listeners;

/// Main application struct
pub struct App {{
    inner: Application,
}}

impl App {{
    /// Creates a new application instance
    pub async fn new() -> Result<Self> {{
        let mut app = Application::new()?;
        app.configure().await?;

        Ok(Self {{ inner: app }})
    }}
}}

#[rustisan_core::async_trait]
impl RustisanApp for App {{
    fn configure(&mut self) -> Result<()> {{
        // Configure your application here
        // Register services, middleware, routes, etc.
        Ok(())
    }}

    async fn run(&self) -> Result<()> {{
        self.inner.run().await
    }}
}}

#[tokio::main]
async fn main() -> Result<()> {{
    // Initialize logging
    rustisan_core::init_logging();

    // Load configuration from rustisan.toml
    // Configuration is loaded automatically by the framework

    // Create and run the application
    let app = App::new().await?;
    app.run().await?;

    Ok(())
}}
"#, name);

    fs::write(path.join("src").join("main.rs"), main_rs)?;
    Ok(())
}

/// Create README.md
fn create_readme(path: &Path, name: &str) -> Result<()> {
    let readme = format!(r#"# {}

A web application built with Rustisan, a Laravel-inspired web framework for Rust.

## Getting Started

### Prerequisites

- Rust 1.70+ with 2024 edition support
- MySQL or PostgreSQL database

### Installation

1. Clone this repository
2. Configure your environment in `rustisan.toml`
3. Install dependencies:
   ```bash
   cargo build
   ```

### Running the Application

```bash
# Development server
rustisan serve

# Or with cargo
cargo run
```

The application will be available at `http://localhost:3000`.

### Database Setup

```bash
# Create database
rustisan db:create

# Run migrations
rustisan migrate

# Seed database (optional)
rustisan seed
```

### Available Commands

```bash
# Generate a controller
rustisan make:controller UserController

# Generate a model
rustisan make:model User --migration

# Generate a migration
rustisan make:migration create_users_table

# View all routes
rustisan route:list

# Run tests
rustisan test
```

## Project Structure

```
src/
├── controllers/     # HTTP controllers
├── models/         # Data models
├── middleware/     # HTTP middleware
├── requests/       # Form request validators
├── resources/      # API resources
├── services/       # Business logic services
├── jobs/          # Background jobs
├── events/        # Application events
└── listeners/     # Event listeners

database/
├── migrations/    # Database migrations
├── seeders/      # Database seeders
└── factories/    # Model factories

routes/           # Route definitions
resources/        # Views and assets
storage/         # Application storage
tests/           # Test files
```

## Documentation

For more information about Rustisan, visit: [https://github.com/lugotardo/rustisan](https://github.com/lugotardo/rustisan)

## License

This project is licensed under the MIT License.
"#, name);

    fs::write(path.join("README.md"), readme)?;
    Ok(())
}

/// Initialize git repository
fn initialize_git(path: &Path) -> Result<()> {
    CommandUtils::info("Initializing git repository...");

    Command::new("git")
        .args(&["init"])
        .current_dir(path)
        .output()?;

    Command::new("git")
        .args(&["add", "."])
        .current_dir(path)
        .output()?;

    Command::new("git")
        .args(&["commit", "-m", "Initial commit"])
        .current_dir(path)
        .output()?;

    Ok(())
}

/// Apply a project template
async fn apply_template(path: &Path, template: &str) -> Result<()> {
    CommandUtils::info(&format!("Applying template '{}'...", template));

    match template {
        "api" => apply_api_template(path).await?,
        "web" => apply_web_template(path).await?,
        "minimal" => apply_minimal_template(path).await?,
        _ => {
            CommandUtils::warning(&format!("Unknown template '{}', using default", template));
        }
    }

    Ok(())
}

/// Apply API template
async fn apply_api_template(_path: &Path) -> Result<()> {
    // Add API-specific configuration and files
    CommandUtils::info("API template applied");
    Ok(())
}

/// Apply web template
async fn apply_web_template(_path: &Path) -> Result<()> {
    // Add web-specific configuration and files
    CommandUtils::info("Web template applied");
    Ok(())
}

/// Apply minimal template
async fn apply_minimal_template(_path: &Path) -> Result<()> {
    // Apply minimal configuration
    CommandUtils::info("Minimal template applied");
    Ok(())
}
