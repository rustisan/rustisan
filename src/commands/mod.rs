//! Command modules for the Rustisan CLI
//!
//! This module contains all the command implementations for the Rustisan CLI.

pub mod new;
pub mod make;
pub mod serve;
pub mod db;
pub mod migrate;
pub mod seed;
pub mod route;
pub mod cache;
pub mod queue;
pub mod config;
pub mod test;
pub mod build;
pub mod deploy;
pub mod info;

pub mod package;
pub mod dev;

// Re-export command types for easier access
pub use crate::{
    DbCommands, MakeCommands, MigrateCommands, RouteCommands,
    CacheCommands, QueueCommands, ConfigCommands,
    PackageCommands, DevCommands
};

use anyhow::Result;
use colored::*;
use std::process::Command;

/// Common utilities for all commands
pub struct CommandUtils;

impl CommandUtils {
    /// Check if we're in a Rustisan project
    pub fn is_rustisan_project() -> bool {
        std::path::Path::new("Cargo.toml").exists() &&
        std::path::Path::new("rustisan.toml").exists()
    }

    /// Ensure we're in a Rustisan project
    pub fn ensure_rustisan_project() -> Result<()> {
        if !Self::is_rustisan_project() {
            anyhow::bail!("This command must be run from within a Rustisan project directory");
        }
        Ok(())
    }

    /// Execute a shell command
    pub fn execute_command(cmd: &str, args: &[&str]) -> Result<()> {
        let output = Command::new(cmd)
            .args(args)
            .output()?;

        if !output.status.success() {
            let stderr = String::from_utf8_lossy(&output.stderr);
            anyhow::bail!("Command failed: {}", stderr);
        }

        Ok(())
    }

    /// Print success message
    pub fn success(message: &str) {
        println!("{} {}", "✓".green().bold(), message);
    }

    /// Print info message
    pub fn info(message: &str) {
        println!("{} {}", "ℹ".blue().bold(), message);
    }

    /// Print warning message
    pub fn warning(message: &str) {
        println!("{} {}", "⚠".yellow().bold(), message);
    }

    /// Print error message
    pub fn error(message: &str) {
        eprintln!("{} {}", "✗".red().bold(), message);
    }

    /// Create directory if it doesn't exist
    pub fn ensure_directory(path: &std::path::Path) -> Result<()> {
        if !path.exists() {
            std::fs::create_dir_all(path)?;
        }
        Ok(())
    }

    /// Convert string to snake_case
    pub fn to_snake_case(input: &str) -> String {
        let mut result = String::new();
        for (i, ch) in input.chars().enumerate() {
            if i > 0 && ch.is_uppercase() {
                result.push('_');
            }
            result.push(ch.to_lowercase().next().unwrap_or(ch));
        }
        result
    }

    /// Convert string to PascalCase
    pub fn to_pascal_case(input: &str) -> String {
        input
            .split('_')
            .map(|word| {
                let mut chars = word.chars();
                match chars.next() {
                    None => String::new(),
                    Some(first) => first.to_uppercase().collect::<String>() + &chars.as_str(),
                }
            })
            .collect()
    }
}
