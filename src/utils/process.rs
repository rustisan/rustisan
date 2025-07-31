//! Process utilities for the Rustisan CLI
//!
//! This module provides common process and command execution utilities.

use anyhow::Result;
use std::process::Command;

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

    /// Execute a command and return Result based on success
    pub fn execute_or_fail(command: &str, args: &[&str]) -> Result<()> {
        let output = Command::new(command)
            .args(args)
            .output()?;

        if !output.status.success() {
            let stderr = String::from_utf8_lossy(&output.stderr);
            anyhow::bail!("Command '{}' failed: {}", command, stderr);
        }

        Ok(())
    }

    /// Execute a command in a specific working directory
    pub fn execute_in_dir<P: AsRef<std::path::Path>>(
        command: &str,
        args: &[&str],
        working_dir: P
    ) -> Result<bool> {
        let output = Command::new(command)
            .args(args)
            .current_dir(working_dir)
            .output()?;

        Ok(output.status.success())
    }

    /// Execute a command and get the exit code
    pub fn execute_with_code(command: &str, args: &[&str]) -> Result<i32> {
        let output = Command::new(command)
            .args(args)
            .output()?;

        Ok(output.status.code().unwrap_or(-1))
    }

    /// Check if we're running on Windows
    pub fn is_windows() -> bool {
        cfg!(target_os = "windows")
    }

    /// Check if we're running on macOS
    pub fn is_macos() -> bool {
        cfg!(target_os = "macos")
    }

    /// Check if we're running on Linux
    pub fn is_linux() -> bool {
        cfg!(target_os = "linux")
    }

    /// Get the appropriate shell command for the current OS
    pub fn get_shell_command() -> (&'static str, &'static str) {
        if Self::is_windows() {
            ("cmd", "/C")
        } else {
            ("sh", "-c")
        }
    }

    /// Execute a shell command
    pub fn execute_shell(command: &str) -> Result<bool> {
        let (shell, flag) = Self::get_shell_command();
        Self::execute(shell, &[flag, command])
    }
}
