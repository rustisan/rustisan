//! Rustisan CLI - Command-line interface for the Rustisan web framework
//!
//! This CLI provides Laravel-like commands for Rustisan applications.

use clap::{Parser, Subcommand};
use colored::*;
// Mock rustisan-core module for testing
mod rustisan_core {
    pub fn init_logging() {
        // Mock implementation
    }
    pub const VERSION: &str = "0.1.1";
}

use rustisan_core::{init_logging, VERSION};
use std::process;

mod commands;
mod generators;
mod utils;

use commands::*;

/// Rustisan CLI - A Laravel-inspired web framework for Rust
#[derive(Parser)]
#[command(name = "rustisan")]
#[command(version = VERSION)]
#[command(about = "A Laravel-inspired web framework for Rust")]
#[command(long_about = "Rustisan CLI provides commands for creating, managing, and deploying Rustisan applications.")]
struct Cli {
    #[command(subcommand)]
    command: Commands,

    /// Enable verbose output
    #[arg(short, long, global = true)]
    verbose: bool,

    /// Suppress output
    #[arg(short, long, global = true)]
    quiet: bool,
}

#[derive(Subcommand)]
pub enum Commands {
    /// Create a new Rustisan application
    New {
        /// Name of the application
        name: String,
        /// Directory to create the application in
        #[arg(short, long)]
        path: Option<String>,
        /// Use a specific template
        #[arg(short, long)]
        template: Option<String>,
        /// Initialize git repository
        #[arg(long, default_value = "true")]
        git: bool,
    },

    /// Generate application components
    Make {
        #[command(subcommand)]
        component: MakeCommands,
    },

    /// Serve the application
    Serve {
        /// Host to bind to
        #[arg(short, long, default_value = "127.0.0.1")]
        host: String,
        /// Port to bind to
        #[arg(short, long, default_value = "3000")]
        port: u16,
        /// Environment to run in
        #[arg(short, long, default_value = "development")]
        env: String,
        /// Enable hot reload
        #[arg(long)]
        reload: bool,
    },

    /// Database operations
    Db {
        #[command(subcommand)]
        operation: DbCommands,
    },

    /// Migration operations
    Migrate {
        #[command(subcommand)]
        operation: Option<MigrateCommands>,
    },

    /// Seeder operations
    Seed {
        /// Specific seeder to run
        #[arg(short, long)]
        class: Option<String>,
        /// Force seeding in production
        #[arg(long)]
        force: bool,
    },

    /// Route operations
    Route {
        #[command(subcommand)]
        operation: RouteCommands,
    },

    /// Cache operations
    Cache {
        #[command(subcommand)]
        operation: CacheCommands,
    },

    /// Queue operations
    Queue {
        #[command(subcommand)]
        operation: QueueCommands,
    },

    /// Configuration operations
    Config {
        #[command(subcommand)]
        operation: ConfigCommands,
    },

    /// Run tests
    Test {
        /// Specific test file or pattern
        pattern: Option<String>,
        /// Run only unit tests
        #[arg(long)]
        unit: bool,
        /// Run only integration tests
        #[arg(long)]
        integration: bool,
        /// Show test output
        #[arg(long)]
        verbose: bool,
    },

    /// Build the application for production
    Build {
        /// Target environment
        #[arg(short, long, default_value = "production")]
        env: String,
        /// Enable optimizations
        #[arg(long)]
        optimize: bool,
        /// Output directory
        #[arg(short, long)]
        output: Option<String>,
    },

    /// Deploy the application
    Deploy {
        /// Deployment target
        target: Option<String>,
        /// Skip build step
        #[arg(long)]
        skip_build: bool,
        /// Dry run (show what would be deployed)
        #[arg(long)]
        dry_run: bool,
    },

    /// Show application information
    Info {
        /// Show detailed information
        #[arg(long)]
        detailed: bool,
    },



    /// Package management
    Package {
        #[command(subcommand)]
        operation: PackageCommands,
    },

    /// Development tools
    Dev {
        #[command(subcommand)]
        tool: DevCommands,
    },
}

#[derive(Subcommand)]
pub enum MakeCommands {
    /// Generate a controller
    Controller {
        /// Controller name
        name: String,
        /// Generate a resource controller
        #[arg(short, long)]
        resource: bool,
        /// Generate an API controller
        #[arg(long)]
        api: bool,
        /// Generate with model
        #[arg(short, long)]
        model: Option<String>,
    },

    /// Generate a model
    Model {
        /// Model name
        name: String,
        /// Generate migration
        #[arg(short, long)]
        migration: bool,
        /// Generate factory
        #[arg(short, long)]
        factory: bool,
        /// Generate seeder
        #[arg(short, long)]
        seeder: bool,
    },

    /// Generate a migration
    Migration {
        /// Migration name
        name: String,
        /// Create table migration
        #[arg(long)]
        create: Option<String>,
        /// Modify table migration
        #[arg(long)]
        table: Option<String>,
    },

    /// Generate middleware
    Middleware {
        /// Middleware name
        name: String,
    },

    /// Generate a request validator
    Request {
        /// Request name
        name: String,
    },

    /// Generate a resource transformer
    Resource {
        /// Resource name
        name: String,
        /// Generate collection resource
        #[arg(short, long)]
        collection: bool,
    },

    /// Generate a seeder
    Seeder {
        /// Seeder name
        name: String,
        /// Associated model
        #[arg(short, long)]
        model: Option<String>,
    },

    /// Generate a factory
    Factory {
        /// Factory name
        name: String,
        /// Associated model
        #[arg(short, long)]
        model: Option<String>,
    },

    /// Generate a command
    Command {
        /// Command name
        name: String,
    },

    /// Generate a job
    Job {
        /// Job name
        name: String,
        /// Synchronous job
        #[arg(long)]
        sync: bool,
    },

    /// Generate an event
    Event {
        /// Event name
        name: String,
    },

    /// Generate a listener
    Listener {
        /// Listener name
        name: String,
        /// Associated event
        #[arg(short, long)]
        event: Option<String>,
    },

    /// Generate a policy
    Policy {
        /// Policy name
        name: String,
        /// Associated model
        #[arg(short, long)]
        model: Option<String>,
    },

    /// Generate a trait
    Trait {
        /// Trait name
        name: String,
    },

    /// Generate a test
    Test {
        /// Test name
        name: String,
        /// Unit test
        #[arg(short, long)]
        unit: bool,
        /// Integration test
        #[arg(long)]
        integration: bool,
    },
}

#[derive(Subcommand)]
pub enum DbCommands {
    /// Show database status
    Status,
    /// Create database
    Create,
    /// Drop database
    Drop {
        #[arg(long)]
        force: bool,
    },
    /// Reset database
    Reset {
        #[arg(long)]
        force: bool,
    },
    /// Seed database
    Seed,
}

#[derive(Subcommand)]
pub enum MigrateCommands {
    /// Run pending migrations
    Up,
    /// Rollback migrations
    Down {
        /// Number of migrations to rollback
        #[arg(short, long, default_value = "1")]
        steps: u32,
    },
    /// Reset all migrations
    Reset,
    /// Rollback and re-run migrations
    Refresh,
    /// Show migration status
    Status,
    /// Create a new migration
    Make {
        name: String,
    },
}

#[derive(Subcommand)]
pub enum RouteCommands {
    /// List all routes
    List {
        /// Filter by method
        #[arg(short, long)]
        method: Option<String>,
        /// Filter by name
        #[arg(short, long)]
        name: Option<String>,
        /// Show middleware
        #[arg(long)]
        middleware: bool,
    },
    /// Clear route cache
    Clear,
    /// Cache routes
    Cache,
}

#[derive(Subcommand)]
pub enum CacheCommands {
    /// Clear all cache
    Clear,
    /// Clear specific cache store
    Forget {
        key: String,
    },
    /// Cache configuration
    Config,
}

#[derive(Subcommand)]
pub enum QueueCommands {
    /// Start queue worker
    Work {
        /// Queue name
        #[arg(short, long)]
        queue: Option<String>,
        /// Number of jobs to process
        #[arg(long)]
        max_jobs: Option<u32>,
        /// Memory limit in MB
        #[arg(long)]
        memory: Option<u32>,
        /// Sleep time when no jobs
        #[arg(long, default_value = "3")]
        sleep: u64,
    },
    /// Restart queue workers
    Restart,
    /// Show failed jobs
    Failed,
    /// Retry failed jobs
    Retry {
        /// Job ID to retry
        id: Option<String>,
    },
    /// Flush failed jobs
    Flush,
}

#[derive(Subcommand)]
pub enum ConfigCommands {
    /// Show all configuration values
    Show,
    /// Get a specific configuration value
    Get {
        /// Configuration key (e.g., app.name, database.default)
        key: String,
    },
    /// Set configuration value
    Set {
        /// Configuration key (e.g., app.name, database.default)
        key: String,
        /// Configuration value
        value: String,
    },
    /// Generate application key
    GenerateKey,
    /// Validate configuration
    Validate,
    /// Reset configuration to defaults
    Reset,
}



#[derive(Subcommand)]
pub enum PackageCommands {
    /// Install a package
    Install {
        name: String,
        /// Package version
        #[arg(short, long)]
        version: Option<String>,
    },
    /// Remove a package
    Remove {
        name: String,
    },
    /// List installed packages
    List,
    /// Update packages
    Update,
}

#[derive(Subcommand)]
pub enum DevCommands {
    /// Start development server with hot reload
    Server {
        #[arg(short, long, default_value = "127.0.0.1")]
        host: String,
        #[arg(short, long, default_value = "3000")]
        port: u16,
    },
    /// Watch files for changes
    Watch,
    /// Format code
    Format,
    /// Check code with clippy
    Check,
    /// Generate documentation
    Docs {
        #[arg(long)]
        open: bool,
    },
}

#[tokio::main]
async fn main() {
    let cli = Cli::parse();

    // Initialize logging based on verbosity
    if !cli.quiet {
        init_logging();
    }

    // Print banner unless quiet
    if !cli.quiet {
        print_banner();
    }

    let result = match cli.command {
        Commands::New { name, path, template, git } => {
            commands::new::handle(name, path, template, git).await
        }
        Commands::Make { component } => {
            commands::make::handle(component).await
        }
        Commands::Serve { host, port, env, reload } => {
            commands::serve::handle(host, port, env, reload).await
        }
        Commands::Db { operation } => {
            commands::db::handle(operation).await
        }
        Commands::Migrate { operation } => {
            commands::migrate::handle(operation).await
        }
        Commands::Seed { class, force } => {
            commands::seed::handle(class, force).await
        }
        Commands::Route { operation } => {
            commands::route::handle(operation).await
        }
        Commands::Cache { operation } => {
            commands::cache::handle(operation).await
        }
        Commands::Queue { operation } => {
            commands::queue::handle(operation).await
        }
        Commands::Config { operation } => {
            commands::config::handle(operation).await
        }
        Commands::Test { pattern, unit, integration, verbose } => {
            commands::test::handle(pattern, unit, integration, verbose).await
        }
        Commands::Build { env, optimize, output } => {
            commands::build::handle(env, optimize, output).await
        }
        Commands::Deploy { target, skip_build, dry_run } => {
            commands::deploy::handle(target, skip_build, dry_run).await
        }
        Commands::Info { detailed } => {
            commands::info::handle(detailed).await
        }

        Commands::Package { operation } => {
            commands::package::handle(operation).await
        }
        Commands::Dev { tool } => {
            commands::dev::handle(tool).await
        }
    };

    if let Err(e) = result {
        eprintln!("{} {}", "Error:".red().bold(), e);
        process::exit(1);
    }
}

fn print_banner() {
    println!("{}", "
██████╗ ██╗   ██╗███████╗████████╗██╗███████╗ █████╗ ███╗   ██╗
██╔══██╗██║   ██║██╔════╝╚══██╔══╝██║██╔════╝██╔══██╗████╗  ██║
██████╔╝██║   ██║███████╗   ██║   ██║███████╗███████║██╔██╗ ██║
██╔══██╗██║   ██║╚════██║   ██║   ██║╚════██║██╔══██║██║╚██╗██║
██║  ██║╚██████╔╝███████║   ██║   ██║███████║██║  ██║██║ ╚████║
╚═╝  ╚═╝ ╚═════╝ ╚══════╝   ╚═╝   ╚═╝╚══════╝╚═╝  ╚═╝╚═╝  ╚═══╝
".cyan().bold());

    println!("{} {}", "Rustisan CLI".green().bold(), format!("v{}", VERSION).dimmed());
    println!("{}\n", "A Laravel-inspired web framework for Rust".dimmed());
}
