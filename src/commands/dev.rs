//! Dev command implementations for the Rustisan CLI

use anyhow::Result;
use colored::*;
use crate::DevCommands;
use super::CommandUtils;

/// Handle dev command
pub async fn handle(tool: DevCommands) -> Result<()> {
    CommandUtils::ensure_rustisan_project()?;

    match tool {
        DevCommands::Server { host, port } => {
            start_dev_server(host, port).await
        }
        DevCommands::Watch => {
            watch_files().await
        }
        DevCommands::Format => {
            format_code().await
        }
        DevCommands::Check => {
            check_code().await
        }
        DevCommands::Docs { open } => {
            generate_docs(open).await
        }
    }
}

async fn start_dev_server(host: String, port: u16) -> Result<()> {
    CommandUtils::info(&format!("Starting development server at {}:{}", host, port));

    // Check if cargo-watch is available
    let watch_available = check_cargo_watch_available();

    if watch_available {
        CommandUtils::info("Hot reload enabled with cargo-watch");

        // Start server with hot reload
        let server_command = format!("cargo run -- serve --host {} --port {} --reload", host, port);

        let output = std::process::Command::new("cargo")
            .args(&["watch", "-x", &format!("run -- serve --host {} --port {} --reload", host, port)])
            .status()?;

        if !output.success() {
            CommandUtils::error("Development server failed to start");
        }
    } else {
        CommandUtils::warning("cargo-watch not found, starting without hot reload");
        CommandUtils::info("Install cargo-watch for hot reload: cargo install cargo-watch");

        // Start server without hot reload
        let output = std::process::Command::new("cargo")
            .args(&["run", "--", "serve", "--host", &host, "--port", &port.to_string()])
            .status()?;

        if !output.success() {
            CommandUtils::error("Development server failed to start");
        }
    }

    Ok(())
}

async fn watch_files() -> Result<()> {
    CommandUtils::info("Watching files for changes...");

    if !check_cargo_watch_available() {
        CommandUtils::error("cargo-watch is not installed");
        CommandUtils::info("Install it with: cargo install cargo-watch");
        return Ok(());
    }

    // Watch for file changes and run tests
    let output = std::process::Command::new("cargo")
        .args(&["watch", "-x", "check", "-x", "test"])
        .status()?;

    if !output.success() {
        CommandUtils::error("File watcher failed");
    }

    Ok(())
}

async fn format_code() -> Result<()> {
    CommandUtils::info("Formatting code...");

    // Run cargo fmt
    let output = std::process::Command::new("cargo")
        .args(&["fmt"])
        .output()?;

    if output.status.success() {
        CommandUtils::success("Code formatted successfully");

        // Show any changes made
        let stdout = String::from_utf8_lossy(&output.stdout);
        if !stdout.is_empty() {
            println!("{}", stdout);
        }
    } else {
        let stderr = String::from_utf8_lossy(&output.stderr);
        CommandUtils::error(&format!("Formatting failed: {}", stderr));
    }

    Ok(())
}

async fn check_code() -> Result<()> {
    CommandUtils::info("Checking code with clippy...");

    // Run clippy
    let output = std::process::Command::new("cargo")
        .args(&["clippy", "--all-targets", "--all-features", "--", "-D", "warnings"])
        .output()?;

    if output.status.success() {
        CommandUtils::success("Code check passed");

        let stdout = String::from_utf8_lossy(&output.stdout);
        if !stdout.is_empty() {
            println!("{}", stdout);
        }
    } else {
        let stderr = String::from_utf8_lossy(&output.stderr);
        CommandUtils::error(&format!("Code check failed: {}", stderr));

        // Still show the output as it contains useful information
        println!("{}", stderr);
    }

    Ok(())
}

async fn generate_docs(open: bool) -> Result<()> {
    CommandUtils::info("Generating documentation...");

    let mut args = vec!["doc", "--no-deps"];
    if open {
        args.push("--open");
    }

    let output = std::process::Command::new("cargo")
        .args(&args)
        .output()?;

    if output.status.success() {
        CommandUtils::success("Documentation generated successfully");

        if !open {
            CommandUtils::info("Documentation available at: target/doc/");
        }
    } else {
        let stderr = String::from_utf8_lossy(&output.stderr);
        CommandUtils::error(&format!("Documentation generation failed: {}", stderr));
    }

    Ok(())
}

fn check_cargo_watch_available() -> bool {
    std::process::Command::new("cargo")
        .args(&["watch", "--version"])
        .output()
        .map(|output| output.status.success())
        .unwrap_or(false)
}

/// Run all development checks
pub async fn run_all_checks() -> Result<()> {
    CommandUtils::info("Running all development checks...");

    // Format code
    CommandUtils::info("Step 1/3: Formatting code...");
    format_code().await?;

    // Check code
    CommandUtils::info("Step 2/3: Checking code...");
    check_code().await?;

    // Run tests
    CommandUtils::info("Step 3/3: Running tests...");
    run_tests().await?;

    CommandUtils::success("All development checks passed");

    Ok(())
}

async fn run_tests() -> Result<()> {
    let output = std::process::Command::new("cargo")
        .args(&["test"])
        .output()?;

    if output.status.success() {
        CommandUtils::success("Tests passed");
    } else {
        let stderr = String::from_utf8_lossy(&output.stderr);
        CommandUtils::error(&format!("Tests failed: {}", stderr));
    }

    Ok(())
}

/// Setup development environment
pub async fn setup_dev_environment() -> Result<()> {
    CommandUtils::info("Setting up development environment...");

    // Install useful development tools
    let tools = [
        ("cargo-watch", "File watcher for automatic rebuilds"),
        ("cargo-edit", "Commands for editing Cargo.toml"),
        ("cargo-outdated", "Check for outdated dependencies"),
        ("cargo-audit", "Security audit for dependencies"),
    ];

    for (tool, description) in &tools {
        CommandUtils::info(&format!("Installing {}: {}", tool, description));

        let output = std::process::Command::new("cargo")
            .args(&["install", tool])
            .output()?;

        if output.status.success() {
            CommandUtils::success(&format!("{} installed successfully", tool));
        } else {
            CommandUtils::warning(&format!("Failed to install {}", tool));
        }
    }

    // Create development configuration files
    create_dev_config_files().await?;

    CommandUtils::success("Development environment setup complete");

    Ok(())
}

async fn create_dev_config_files() -> Result<()> {
    // Create .rustfmt.toml if it doesn't exist
    if !std::path::Path::new(".rustfmt.toml").exists() {
        let rustfmt_config = r#"# Rustfmt configuration for Rustisan project
max_width = 100
tab_spaces = 4
newline_style = "Unix"
use_small_heuristics = "Default"
reorder_imports = true
reorder_modules = true
remove_nested_parens = true
edition = "2021"
"#;

        std::fs::write(".rustfmt.toml", rustfmt_config)?;
        CommandUtils::success("Created .rustfmt.toml");
    }

    // Create clippy.toml if it doesn't exist
    if !std::path::Path::new("clippy.toml").exists() {
        let clippy_config = r#"# Clippy configuration for Rustisan project
avoid-breaking-exported-api = false
msrv = "1.70.0"
"#;

        std::fs::write("clippy.toml", clippy_config)?;
        CommandUtils::success("Created clippy.toml");
    }

    // Create .cargo/config.toml if it doesn't exist
    CommandUtils::ensure_directory(&std::path::Path::new(".cargo"))?;

    if !std::path::Path::new(".cargo/config.toml").exists() {
        let cargo_config = r#"# Cargo configuration for Rustisan project

[build]
# Use all available CPU cores for compilation
jobs = 0

[target.x86_64-unknown-linux-gnu]
# Link with system libraries
rustflags = ["-C", "link-arg=-fuse-ld=lld"]

[alias]
# Useful aliases for development
r = "run"
b = "build"
t = "test"
c = "check"
f = "fmt"
l = "clippy"
"#;

        std::fs::write(".cargo/config.toml", cargo_config)?;
        CommandUtils::success("Created .cargo/config.toml");
    }

    Ok(())
}

/// Profile the application
pub async fn profile_app() -> Result<()> {
    CommandUtils::info("Profiling application...");

    // Check if profiling tools are available
    let tools = ["perf", "valgrind", "cargo-profdata"];

    for tool in &tools {
        let available = std::process::Command::new("which")
            .arg(tool)
            .output()
            .map(|output| output.status.success())
            .unwrap_or(false);

        if available {
            CommandUtils::info(&format!("{} is available", tool));
        } else {
            CommandUtils::warning(&format!("{} is not available", tool));
        }
    }

    // Build with profiling symbols
    CommandUtils::info("Building with profiling symbols...");
    let output = std::process::Command::new("cargo")
        .args(&["build", "--release", "--profile", "profiling"])
        .output()?;

    if output.status.success() {
        CommandUtils::success("Profiling build complete");
        CommandUtils::info("Run your application with profiling tools:");
        CommandUtils::info("  perf record ./target/release/rustisan");
        CommandUtils::info("  valgrind --tool=callgrind ./target/release/rustisan");
    } else {
        let stderr = String::from_utf8_lossy(&output.stderr);
        CommandUtils::error(&format!("Profiling build failed: {}", stderr));
    }

    Ok(())
}

/// Benchmark the application
pub async fn benchmark() -> Result<()> {
    CommandUtils::info("Running benchmarks...");

    let output = std::process::Command::new("cargo")
        .args(&["bench"])
        .output()?;

    if output.status.success() {
        let stdout = String::from_utf8_lossy(&output.stdout);
        println!("{}", stdout);
        CommandUtils::success("Benchmarks completed");
    } else {
        let stderr = String::from_utf8_lossy(&output.stderr);
        CommandUtils::error(&format!("Benchmarks failed: {}", stderr));
    }

    Ok(())
}
