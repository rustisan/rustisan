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
use crate::utils::{FileUtils, ProcessUtils, TextUtils};

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
        ProcessUtils::execute_or_fail(cmd, args)
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
        FileUtils::ensure_dir(path)
    }

    /// Convert string to snake_case
    pub fn to_snake_case(input: &str) -> String {
        TextUtils::to_snake_case(input)
    }

    /// Convert string to PascalCase
    pub fn to_pascal_case(input: &str) -> String {
        TextUtils::to_pascal_case(input)
    }

    /// Convert string to camelCase
    pub fn to_camel_case(input: &str) -> String {
        TextUtils::to_camel_case(input)
    }

    /// Check if a command exists in PATH
    pub fn command_exists(command: &str) -> bool {
        ProcessUtils::command_exists(command)
    }

    /// Check if a file exists
    pub fn file_exists<P: AsRef<std::path::Path>>(path: P) -> bool {
        FileUtils::exists(path)
    }

    /// Write content to a file
    pub fn write_file<P: AsRef<std::path::Path>>(path: P, content: &str) -> Result<()> {
        FileUtils::write_file(path, content)
    }

    /// Read file content as string
    pub fn read_file<P: AsRef<std::path::Path>>(path: P) -> Result<String> {
        FileUtils::read_file(path)
    }
}
