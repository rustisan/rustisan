//! File system utilities for the Rustisan CLI
//!
//! This module provides common file system operations used throughout the CLI.

use anyhow::Result;
use std::fs;
use std::path::{Path, PathBuf};

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

    /// Copy a directory recursively
    pub fn copy_dir<P: AsRef<Path>, Q: AsRef<Path>>(from: P, to: Q) -> Result<()> {
        let from = from.as_ref();
        let to = to.as_ref();

        if !from.exists() {
            return Ok(());
        }

        Self::ensure_dir(to)?;

        for entry in fs::read_dir(from)? {
            let entry = entry?;
            let path = entry.path();
            let file_name = entry.file_name();
            let dest_path = to.join(file_name);

            if path.is_dir() {
                Self::copy_dir(&path, &dest_path)?;
            } else {
                fs::copy(&path, &dest_path)?;
            }
        }

        Ok(())
    }

    /// Remove a directory and all its contents
    pub fn remove_dir_all<P: AsRef<Path>>(path: P) -> Result<()> {
        let path = path.as_ref();
        if path.exists() {
            fs::remove_dir_all(path)?;
        }
        Ok(())
    }

    /// Check if a path is a file
    pub fn is_file<P: AsRef<Path>>(path: P) -> bool {
        path.as_ref().is_file()
    }

    /// Check if a path is a directory
    pub fn is_dir<P: AsRef<Path>>(path: P) -> bool {
        path.as_ref().is_dir()
    }
}
