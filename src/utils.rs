//! Utility functions for the Rustisan CLI
//!
//! This module provides common utility functions used throughout the CLI.

use anyhow::Result;
use colored::*;
use std::fs;
use std::path::{Path, PathBuf};
use std::process::Command;

/// File system utilities
pub struct FileUtils;

impl FileUtils {
    /// Check if a file exists
    pub fn exists<P: AsRef<Path>>(path: P) -> bool {
        path.as_ref().exists()
    }

    /// Create a directory if it doesn't exist
    pub fn ensure_dir<P: AsRef<Path>>(path: P) -> Result<()> {
        let path = path.as_ref();
        if !path.exists() {
            fs::create_dir_all(path)?;
        }
        Ok(())
    }

    /// Copy a file from source to destination
    pub fn copy_file<P: AsRef<Path>, Q: AsRef<Path>>(from: P, to: Q) -> Result<()> {
        let to_path = to.as_ref();
        if let Some(parent) = to_path.parent() {
            Self::ensure_dir(parent)?;
        }
        fs::copy(from, to)?;
        Ok(())
    }

    /// Write content to a file
    pub fn write_file<P: AsRef<Path>>(path: P, content: &str) -> Result<()> {
        let path = path.as_ref();
        if let Some(parent) = path.parent() {
            Self::ensure_dir(parent)?;
        }
        fs::write(path, content)?;
        Ok(())
    }

    /// Read file content as string
    pub fn read_file<P: AsRef<Path>>(path: P) -> Result<String> {
        Ok(fs::read_to_string(path)?)
    }

    /// Find files with a specific extension in a directory
    pub fn find_files_with_extension<P: AsRef<Path>>(dir: P, extension: &str) -> Result<Vec<PathBuf>> {
        let mut files = Vec::new();
        let dir = dir.as_ref();

        if dir.is_dir() {
            for entry in fs::read_dir(dir)? {
                let entry = entry?;
                let path = entry.path();

                if path.is_file() {
                    if let Some(ext) = path.extension() {
                        if ext == extension {
                            files.push(path);
                        }
                    }
                }
            }
        }

        Ok(files)
    }

    /// Get file name without extension
    pub fn file_stem<P: AsRef<Path>>(path: P) -> Option<String> {
        path.as_ref()
            .file_stem()
            .and_then(|s| s.to_str())
            .map(|s| s.to_string())
    }
}

/// Process utilities
pub struct ProcessUtils;

impl ProcessUtils {
    /// Check if a command exists in PATH
    pub fn command_exists(command: &str) -> bool {
        Command::new("which")
            .arg(command)
            .output()
            .map(|output| output.status.success())
            .unwrap_or(false)
    }

    /// Execute a command and return success status
    pub fn execute(command: &str, args: &[&str]) -> Result<bool> {
        let output = Command::new(command)
            .args(args)
            .output()?;

        Ok(output.status.success())
    }

    /// Execute a command and capture output
    pub fn execute_with_output(command: &str, args: &[&str]) -> Result<(bool, String, String)> {
        let output = Command::new(command)
            .args(args)
            .output()?;

        let stdout = String::from_utf8_lossy(&output.stdout).to_string();
        let stderr = String::from_utf8_lossy(&output.stderr).to_string();

        Ok((output.status.success(), stdout, stderr))
    }
}

/// Text utilities
pub struct TextUtils;

impl TextUtils {
    /// Capitalize first letter
    pub fn capitalize(s: &str) -> String {
        let mut chars = s.chars();
        match chars.next() {
            None => String::new(),
            Some(first) => first.to_uppercase().collect::<String>() + &chars.as_str(),
        }
    }
}
