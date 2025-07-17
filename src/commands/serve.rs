//! Serve command for running the Rustisan development server
//!
//! This command starts the development server for the Rustisan application.

use anyhow::Result;
use colored::*;
use std::process::{Command, Stdio};
use std::sync::{Arc, Mutex};
use std::thread;
use std::time::Duration;

use super::CommandUtils;

/// Handle the serve command
pub async fn handle(host: String, port: u16, env: String, reload: bool) -> Result<()> {
    CommandUtils::ensure_rustisan_project()?;

    CommandUtils::info(&format!("Starting Rustisan development server on {}:{}...", host, port));

    // Set environment variables
    unsafe {
        std::env::set_var("APP_ENV", &env);
        std::env::set_var("SERVER_HOST", &host);
        std::env::set_var("SERVER_PORT", &port.to_string());
    }

    if reload {
        start_with_hot_reload(host, port, env).await
    } else {
        start_normal_server().await
    }
}

/// Start the server normally
async fn start_normal_server() -> Result<()> {
    CommandUtils::info("Building application...");

    // Build the application first
    let build_output = Command::new("cargo")
        .args(&["build"])
        .output()?;

    if !build_output.status.success() {
        let stderr = String::from_utf8_lossy(&build_output.stderr);
        CommandUtils::error(&format!("Build failed: {}", stderr));
        return Err(anyhow::anyhow!("Build failed"));
    }

    CommandUtils::success("Application built successfully");
    CommandUtils::info("Starting server...");

    // Run the application
    let child = Command::new("cargo")
        .args(&["run"])
        .stdout(Stdio::inherit())
        .stderr(Stdio::inherit())
        .spawn()?;

    let child_arc = Arc::new(Mutex::new(child));
    let child_clone = Arc::clone(&child_arc);

    // Wait for the process to complete or handle Ctrl+C
    let ctrl_c = tokio::signal::ctrl_c();
    tokio::pin!(ctrl_c);

    tokio::select! {
        _ = &mut ctrl_c => {
            CommandUtils::info("Shutting down server...");
            if let Ok(mut child) = child_arc.lock() {
                let _ = child.kill();
                let _ = child.wait();
            }
        }
        result = tokio::task::spawn_blocking(move || {
            if let Ok(mut child) = child_clone.lock() {
                child.wait()
            } else {
                Err(std::io::Error::new(std::io::ErrorKind::Other, "Failed to lock child"))
            }
        }) => {
            match result? {
                Ok(status) => {
                    if !status.success() {
                        CommandUtils::error("Server exited with error");
                        return Err(anyhow::anyhow!("Server failed"));
                    }
                }
                Err(e) => {
                    CommandUtils::error(&format!("Server error: {}", e));
                    return Err(anyhow::anyhow!("Server failed"));
                }
            }
        }
    }

    CommandUtils::success("Server stopped");
    Ok(())
}

/// Start the server with hot reload functionality
async fn start_with_hot_reload(host: String, port: u16, env: String) -> Result<()> {
    CommandUtils::info("Starting development server with hot reload...");

    // Check if cargo-watch is installed
    if !is_cargo_watch_installed() {
        CommandUtils::warning("cargo-watch is not installed. Installing...");
        install_cargo_watch()?;
    }

    // Use cargo-watch to monitor file changes
    let child = Command::new("cargo")
        .args(&[
            "watch",
            "-x", "run",
            "-w", "src",
            "-w", "Cargo.toml",
            "-w", "rustisan.toml",
        ])
        .env("APP_ENV", env)
        .env("SERVER_HOST", host)
        .env("SERVER_PORT", port.to_string())
        .stdout(Stdio::inherit())
        .stderr(Stdio::inherit())
        .spawn()?;

    let child_arc = Arc::new(Mutex::new(child));
    let child_clone = Arc::clone(&child_arc);

    // Handle Ctrl+C gracefully
    let ctrl_c = tokio::signal::ctrl_c();
    tokio::pin!(ctrl_c);

    tokio::select! {
        _ = &mut ctrl_c => {
            CommandUtils::info("Shutting down development server...");
            if let Ok(mut child) = child_arc.lock() {
                let _ = child.kill();
                let _ = child.wait();
            }
        }
        result = tokio::task::spawn_blocking(move || {
            if let Ok(mut child) = child_clone.lock() {
                child.wait()
            } else {
                Err(std::io::Error::new(std::io::ErrorKind::Other, "Failed to lock child"))
            }
        }) => {
            match result? {
                Ok(status) => {
                    if !status.success() {
                        CommandUtils::error("Development server exited with error");
                        return Err(anyhow::anyhow!("Development server failed"));
                    }
                }
                Err(e) => {
                    CommandUtils::error(&format!("Development server error: {}", e));
                    return Err(anyhow::anyhow!("Development server failed"));
                }
            }
        }
    }

    CommandUtils::success("Development server stopped");
    Ok(())
}

/// Check if cargo-watch is installed
fn is_cargo_watch_installed() -> bool {
    Command::new("cargo")
        .args(&["watch", "--version"])
        .output()
        .map(|output| output.status.success())
        .unwrap_or(false)
}

/// Install cargo-watch
fn install_cargo_watch() -> Result<()> {
    CommandUtils::info("Installing cargo-watch...");

    let output = Command::new("cargo")
        .args(&["install", "cargo-watch"])
        .output()?;

    if !output.status.success() {
        let stderr = String::from_utf8_lossy(&output.stderr);
        CommandUtils::error(&format!("Failed to install cargo-watch: {}", stderr));
        return Err(anyhow::anyhow!("Failed to install cargo-watch"));
    }

    CommandUtils::success("cargo-watch installed successfully");
    Ok(())
}

/// Check if the server is responding
pub async fn check_server_health(host: &str, port: u16) -> bool {
    let url = format!("http://{}:{}/health", host, port);

    match reqwest::get(&url).await {
        Ok(response) => response.status().is_success(),
        Err(_) => false,
    }
}

/// Wait for server to be ready
pub async fn wait_for_server(host: &str, port: u16, timeout_seconds: u64) -> Result<()> {
    let start = std::time::Instant::now();
    let timeout = Duration::from_secs(timeout_seconds);

    while start.elapsed() < timeout {
        if check_server_health(host, port).await {
            return Ok(());
        }
        tokio::time::sleep(Duration::from_millis(500)).await;
    }

    Err(anyhow::anyhow!("Server did not start within {} seconds", timeout_seconds))
}

/// Display server information
pub fn display_server_info(host: &str, port: u16) {
    println!("\n{}", "Server Information:".bold().green());
    println!("  Local:    http://{}:{}", host, port);

    if host == "0.0.0.0" || host == "127.0.0.1" {
        if let Ok(local_ip) = get_local_ip() {
            println!("  Network:  http://{}:{}", local_ip, port);
        }
    }

    println!("\n{}", "Available endpoints:".bold());
    println!("  Health check: http://{}:{}/health", host, port);
    println!("  API docs:     http://{}:{}/docs", host, port);

    println!("\n{}", "Press Ctrl+C to stop the server".dimmed());
}

/// Get local IP address
fn get_local_ip() -> Result<String> {
    use std::net::UdpSocket;

    let socket = UdpSocket::bind("0.0.0.0:0")?;
    socket.connect("8.8.8.8:80")?;
    let local_addr = socket.local_addr()?;
    Ok(local_addr.ip().to_string())
}
