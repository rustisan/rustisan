//! Code generators for the Rustisan CLI
//!
//! This module contains code generators for creating various application components.

// pub mod controller;
// pub mod model;
// pub mod migration;
// pub mod middleware;
// pub mod request;
// pub mod resource;
// pub mod seeder;
// pub mod factory;
// pub mod command;
// pub mod job;
// pub mod event;
// pub mod listener;
// pub mod policy;
// pub mod test;

use anyhow::Result;
use handlebars::Handlebars;
// use serde_json::json;
use std::collections::HashMap;
use std::fs;
use std::path::{Path, PathBuf};

use crate::commands::CommandUtils;

/// Base trait for all generators
pub trait Generator {
    /// Generate the component
    fn generate(&self, name: &str, options: &GeneratorOptions) -> Result<()>;

    /// Get the template name
    fn template_name(&self) -> &'static str;

    /// Get the output path
    fn output_path(&self, name: &str) -> PathBuf;

    /// Get template variables
    fn template_vars(&self, name: &str, options: &GeneratorOptions) -> HashMap<String, serde_json::Value>;

    /// Post-generation actions
    fn post_generate(&self, name: &str, _options: &GeneratorOptions) -> Result<()> {
        CommandUtils::success(&format!("Generated {}", name));
        Ok(())
    }
}

/// Options for generators
#[derive(Debug, Clone, Default)]
pub struct GeneratorOptions {
    pub resource: bool,
    pub api: bool,
    pub migration: bool,
    pub factory: bool,
    pub seeder: bool,
    pub model: Option<String>,
    pub event: Option<String>,
    pub collection: bool,
    pub sync: bool,
    pub unit: bool,
    pub integration: bool,
    pub force: bool,
    pub create_table: Option<String>,
    pub modify_table: Option<String>,
}

/// Template manager for handling Handlebars templates
pub struct TemplateManager {
    handlebars: Handlebars<'static>,
}

impl TemplateManager {
    /// Create a new template manager
    pub fn new() -> Result<Self> {
        let mut handlebars = Handlebars::new();

        // Register built-in templates
        Self::register_templates(&mut handlebars)?;

        Ok(Self { handlebars })
    }

    /// Register all built-in templates
    fn register_templates(_handlebars: &mut Handlebars) -> Result<()> {
        // TODO: Templates will be registered here once template files are created
        // For now, just return Ok to avoid compilation errors
        Ok(())
    }

    /// Render a template with the given context
    pub fn render(&self, template: &str, context: &serde_json::Value) -> Result<String> {
        Ok(self.handlebars.render(template, context)?)
    }

    /// Check if a template exists
    pub fn has_template(&self, name: &str) -> bool {
        self.handlebars.get_template(name).is_some()
    }
}

impl Default for TemplateManager {
    fn default() -> Self {
        Self::new().expect("Failed to create template manager")
    }
}

/// Common utility functions for generators
pub struct GeneratorUtils;

impl GeneratorUtils {
    /// Convert name to various cases
    pub fn name_variations(name: &str) -> HashMap<String, String> {
        let mut vars = HashMap::new();

        vars.insert("name".to_string(), name.to_string());
        vars.insert("snake_case".to_string(), CommandUtils::to_snake_case(name));
        vars.insert("pascal_case".to_string(), CommandUtils::to_pascal_case(name));
        vars.insert("camel_case".to_string(), to_camel_case(name));
        vars.insert("kebab_case".to_string(), to_kebab_case(name));
        vars.insert("title_case".to_string(), to_title_case(name));
        vars.insert("plural".to_string(), pluralize(name));
        vars.insert("singular".to_string(), singularize(name));

        vars
    }

    /// Check if file exists and handle force option
    pub fn check_file_exists(path: &Path, force: bool) -> Result<()> {
        if path.exists() && !force {
            return Err(anyhow::anyhow!(
                "File '{}' already exists. Use --force to overwrite.",
                path.display()
            ));
        }
        Ok(())
    }

    /// Ensure directory exists
    pub fn ensure_directory(path: &Path) -> Result<()> {
        if let Some(parent) = path.parent() {
            fs::create_dir_all(parent)?;
        }
        Ok(())
    }

    /// Write file with content
    pub fn write_file(path: &Path, content: &str) -> Result<()> {
        Self::ensure_directory(path)?;
        fs::write(path, content)?;
        Ok(())
    }

    /// Update module file to include new component
    pub fn update_module_file(module_dir: &Path, component_name: &str) -> Result<()> {
        let mod_file = module_dir.join("mod.rs");

        if mod_file.exists() {
            let content = fs::read_to_string(&mod_file)?;
            let module_line = format!("pub mod {};", CommandUtils::to_snake_case(component_name));

            if !content.contains(&module_line) {
                let new_content = format!("{}\n{}", content.trim(), module_line);
                fs::write(&mod_file, new_content)?;
            }
        }

        Ok(())
    }
}

/// Convert to camelCase
fn to_camel_case(input: &str) -> String {
    let pascal = CommandUtils::to_pascal_case(input);
    if let Some(first_char) = pascal.chars().next() {
        first_char.to_lowercase().collect::<String>() + &pascal[1..]
    } else {
        pascal
    }
}

/// Convert to kebab-case
fn to_kebab_case(input: &str) -> String {
    CommandUtils::to_snake_case(input).replace('_', "-")
}

/// Convert to Title Case
fn to_title_case(input: &str) -> String {
    input
        .split('_')
        .map(|word| {
            let mut chars = word.chars();
            match chars.next() {
                None => String::new(),
                Some(first) => first.to_uppercase().collect::<String>() + &chars.as_str(),
            }
        })
        .collect::<Vec<String>>()
        .join(" ")
}

/// Simple pluralize function (basic English rules)
fn pluralize(input: &str) -> String {
    let lower = input.to_lowercase();

    if lower.ends_with('y') && !lower.ends_with("ay") && !lower.ends_with("ey") && !lower.ends_with("iy") && !lower.ends_with("oy") && !lower.ends_with("uy") {
        format!("{}ies", &input[..input.len()-1])
    } else if lower.ends_with('s') || lower.ends_with("sh") || lower.ends_with("ch") || lower.ends_with('x') || lower.ends_with('z') {
        format!("{}es", input)
    } else if lower.ends_with('f') {
        format!("{}ves", &input[..input.len()-1])
    } else if lower.ends_with("fe") {
        format!("{}ves", &input[..input.len()-2])
    } else {
        format!("{}s", input)
    }
}

/// Simple singularize function (basic English rules)
fn singularize(input: &str) -> String {
    let lower = input.to_lowercase();

    if lower.ends_with("ies") && input.len() > 3 {
        format!("{}y", &input[..input.len()-3])
    } else if lower.ends_with("ves") && input.len() > 3 {
        if lower.ends_with("aves") || lower.ends_with("eves") || lower.ends_with("ives") || lower.ends_with("oves") {
            format!("{}f", &input[..input.len()-3])
        } else {
            format!("{}fe", &input[..input.len()-3])
        }
    } else if lower.ends_with("ses") || lower.ends_with("shes") || lower.ends_with("ches") || lower.ends_with("xes") || lower.ends_with("zes") {
        input[..input.len()-2].to_string()
    } else if lower.ends_with('s') && input.len() > 1 {
        input[..input.len()-1].to_string()
    } else {
        input.to_string()
    }
}
