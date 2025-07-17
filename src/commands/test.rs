//! Test command implementations for the Rustisan CLI

use anyhow::Result;
use colored::*;
use super::CommandUtils;

/// Handle test command
pub async fn handle(
    pattern: Option<String>,
    unit: bool,
    integration: bool,
    verbose: bool,
) -> Result<()> {
    CommandUtils::ensure_rustisan_project()?;

    let test_type = if unit {
        "unit"
    } else if integration {
        "integration"
    } else {
        "all"
    };

    CommandUtils::info(&format!("Running {} tests...", test_type));

    if let Some(ref pattern) = pattern {
        CommandUtils::info(&format!("Test pattern: {}", pattern));
    }

    run_tests(pattern, unit, integration, verbose).await
}

async fn run_tests(
    pattern: Option<String>,
    unit: bool,
    integration: bool,
    verbose: bool,
) -> Result<()> {
    let mut cargo_args = vec!["test"];

    // Add test type filters
    if unit {
        cargo_args.push("--lib");
    } else if integration {
        cargo_args.push("--test");
        cargo_args.push("*");
    }

    // Add pattern filter
    if let Some(ref pattern) = pattern {
        cargo_args.push(pattern);
    }

    // Add verbose flag
    if verbose {
        cargo_args.push("--");
        cargo_args.push("--nocapture");
    }

    CommandUtils::info(&format!("Running: cargo {}", cargo_args.join(" ")));

    let output = std::process::Command::new("cargo")
        .args(&cargo_args)
        .output()?;

    if output.status.success() {
        let stdout = String::from_utf8_lossy(&output.stdout);
        println!("{}", stdout);

        // Parse test results
        let results = parse_test_results(&stdout);
        print_test_summary(&results);

        CommandUtils::success("Tests completed successfully");
    } else {
        let stderr = String::from_utf8_lossy(&output.stderr);
        eprintln!("{}", stderr);

        CommandUtils::error("Tests failed");
        std::process::exit(1);
    }

    Ok(())
}

#[derive(Debug, Default)]
struct TestResults {
    passed: u32,
    failed: u32,
    ignored: u32,
    total: u32,
}

fn parse_test_results(output: &str) -> TestResults {
    let mut results = TestResults::default();

    // Look for test result summary line
    for line in output.lines() {
        if line.contains("test result:") {
            // Example: "test result: ok. 15 passed; 0 failed; 0 ignored; 0 measured; 0 filtered out"
            let parts: Vec<&str> = line.split_whitespace().collect();

            for (i, part) in parts.iter().enumerate() {
                if part == &"passed;" && i > 0 {
                    if let Ok(count) = parts[i - 1].parse::<u32>() {
                        results.passed = count;
                    }
                } else if part == &"failed;" && i > 0 {
                    if let Ok(count) = parts[i - 1].parse::<u32>() {
                        results.failed = count;
                    }
                } else if part == &"ignored;" && i > 0 {
                    if let Ok(count) = parts[i - 1].parse::<u32>() {
                        results.ignored = count;
                    }
                }
            }

            results.total = results.passed + results.failed + results.ignored;
            break;
        }
    }

    results
}

fn print_test_summary(results: &TestResults) {
    println!("\n{}", "Test Summary:".bold());
    println!("┌─────────────────────────────────────────────────────────────────────────────┐");
    println!("│ {} │ {} │ {} │ {} │",
        "Total".bold(),
        "Passed".green().bold(),
        "Failed".red().bold(),
        "Ignored".yellow().bold()
    );
    println!("├─────────────────────────────────────────────────────────────────────────────┤");
    println!("│ {} │ {} │ {} │ {} │",
        format!("{:5}", results.total),
        format!("{:6}", results.passed).green(),
        format!("{:6}", results.failed).red(),
        format!("{:7}", results.ignored).yellow()
    );
    println!("└─────────────────────────────────────────────────────────────────────────────┘");

    // Print status message
    if results.failed > 0 {
        println!("\n{} {} test(s) failed", "✗".red().bold(), results.failed);
    } else if results.passed > 0 {
        println!("\n{} All {} test(s) passed", "✓".green().bold(), results.passed);
    } else {
        println!("\n{} No tests were run", "ℹ".blue().bold());
    }

    // Print coverage information if available
    print_coverage_info();
}

fn print_coverage_info() {
    // TODO: Implement test coverage reporting
    // This would require integration with cargo-tarpaulin or similar tools

    println!("\n{}", "Coverage Information:".bold());
    println!("Run `cargo tarpaulin` to generate coverage reports");
}

/// Run specific test suites
pub async fn run_unit_tests() -> Result<()> {
    CommandUtils::info("Running unit tests...");

    let output = std::process::Command::new("cargo")
        .args(&["test", "--lib"])
        .output()?;

    if output.status.success() {
        CommandUtils::success("Unit tests completed successfully");
    } else {
        CommandUtils::error("Unit tests failed");
    }

    Ok(())
}

pub async fn run_integration_tests() -> Result<()> {
    CommandUtils::info("Running integration tests...");

    let output = std::process::Command::new("cargo")
        .args(&["test", "--test", "*"])
        .output()?;

    if output.status.success() {
        CommandUtils::success("Integration tests completed successfully");
    } else {
        CommandUtils::error("Integration tests failed");
    }

    Ok(())
}

pub async fn run_benchmark_tests() -> Result<()> {
    CommandUtils::info("Running benchmark tests...");

    let output = std::process::Command::new("cargo")
        .args(&["bench"])
        .output()?;

    if output.status.success() {
        CommandUtils::success("Benchmark tests completed successfully");
    } else {
        CommandUtils::error("Benchmark tests failed");
    }

    Ok(())
}

/// Watch tests for changes
pub async fn watch_tests() -> Result<()> {
    CommandUtils::info("Watching tests for changes...");

    // Check if cargo-watch is installed
    let watch_check = std::process::Command::new("cargo")
        .args(&["watch", "--version"])
        .output();

    if watch_check.is_err() {
        CommandUtils::error("cargo-watch is not installed");
        CommandUtils::info("Install it with: cargo install cargo-watch");
        return Ok(());
    }

    // Start watching
    let output = std::process::Command::new("cargo")
        .args(&["watch", "-x", "test"])
        .status()?;

    if !output.success() {
        CommandUtils::error("Test watcher failed");
    }

    Ok(())
}
