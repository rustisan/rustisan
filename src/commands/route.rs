//! Route command implementations for the Rustisan CLI

use anyhow::Result;
use colored::*;
use crate::RouteCommands;
use super::CommandUtils;

/// Handle route command
pub async fn handle(operation: RouteCommands) -> Result<()> {
    CommandUtils::ensure_rustisan_project()?;

    match operation {
        RouteCommands::List { method, name, middleware } => {
            list_routes(method, name, middleware).await
        }
        RouteCommands::Clear => clear_route_cache().await,
        RouteCommands::Cache => cache_routes().await,
    }
}

async fn list_routes(method: Option<String>, name: Option<String>, show_middleware: bool) -> Result<()> {
    CommandUtils::info("Listing application routes...");

    println!("\n{}", "Route List:".bold());
    println!("┌─────────────┬─────────────────────────────────────────────────────────────────────┐");
    println!("│ {} │ {} │", "Method".bold(), "URI".bold());

    if show_middleware {
        println!("│ {} │ {} │", "Middleware".bold(), "Action".bold());
    } else {
        println!("│ {} │", "Action".bold());
    }

    println!("├─────────────┼─────────────────────────────────────────────────────────────────────┤");

    // TODO: Implement actual route discovery
    let routes = discover_routes()?;
    let filtered_routes = filter_routes(routes, method, name);

    if filtered_routes.is_empty() {
        println!("│ {} │", "No routes found".dimmed());
    } else {
        for route in filtered_routes {
            print_route(&route, show_middleware);
        }
    }

    println!("└─────────────┴─────────────────────────────────────────────────────────────────────┘");

    Ok(())
}

async fn clear_route_cache() -> Result<()> {
    CommandUtils::info("Clearing route cache...");

    let cache_path = "bootstrap/cache/routes.json";

    if std::path::Path::new(cache_path).exists() {
        std::fs::remove_file(cache_path)?;
        CommandUtils::success("Route cache cleared successfully");
    } else {
        CommandUtils::warning("Route cache file not found");
    }

    Ok(())
}

async fn cache_routes() -> Result<()> {
    CommandUtils::info("Caching routes...");

    // TODO: Implement route caching logic
    let routes = discover_routes()?;
    let cache_data = serde_json::to_string_pretty(&routes)?;

    CommandUtils::ensure_directory(&std::path::Path::new("bootstrap/cache"))?;
    std::fs::write("bootstrap/cache/routes.json", cache_data)?;

    CommandUtils::success("Routes cached successfully");

    Ok(())
}

#[derive(Debug, serde::Serialize, serde::Deserialize)]
struct Route {
    method: String,
    uri: String,
    name: Option<String>,
    action: String,
    middleware: Vec<String>,
}

fn discover_routes() -> Result<Vec<Route>> {
    // TODO: Implement actual route discovery by parsing route files
    // For now, return some example routes
    Ok(vec![
        Route {
            method: "GET".to_string(),
            uri: "/".to_string(),
            name: Some("home".to_string()),
            action: "HomeController@index".to_string(),
            middleware: vec!["web".to_string()],
        },
        Route {
            method: "GET".to_string(),
            uri: "/api/users".to_string(),
            name: Some("users.index".to_string()),
            action: "UserController@index".to_string(),
            middleware: vec!["api".to_string(), "auth".to_string()],
        },
        Route {
            method: "POST".to_string(),
            uri: "/api/users".to_string(),
            name: Some("users.store".to_string()),
            action: "UserController@store".to_string(),
            middleware: vec!["api".to_string(), "auth".to_string()],
        },
        Route {
            method: "GET".to_string(),
            uri: "/api/users/{id}".to_string(),
            name: Some("users.show".to_string()),
            action: "UserController@show".to_string(),
            middleware: vec!["api".to_string(), "auth".to_string()],
        },
        Route {
            method: "PUT".to_string(),
            uri: "/api/users/{id}".to_string(),
            name: Some("users.update".to_string()),
            action: "UserController@update".to_string(),
            middleware: vec!["api".to_string(), "auth".to_string()],
        },
        Route {
            method: "DELETE".to_string(),
            uri: "/api/users/{id}".to_string(),
            name: Some("users.destroy".to_string()),
            action: "UserController@destroy".to_string(),
            middleware: vec!["api".to_string(), "auth".to_string()],
        },
    ])
}

fn filter_routes(routes: Vec<Route>, method: Option<String>, name: Option<String>) -> Vec<Route> {
    routes
        .into_iter()
        .filter(|route| {
            if let Some(ref method_filter) = method {
                if route.method.to_lowercase() != method_filter.to_lowercase() {
                    return false;
                }
            }

            if let Some(ref name_filter) = name {
                if let Some(ref route_name) = route.name {
                    if !route_name.contains(name_filter) {
                        return false;
                    }
                } else {
                    return false;
                }
            }

            true
        })
        .collect()
}

fn print_route(route: &Route, show_middleware: bool) {
    let method_color = match route.method.as_str() {
        "GET" => route.method.green(),
        "POST" => route.method.blue(),
        "PUT" => route.method.yellow(),
        "PATCH" => route.method.cyan(),
        "DELETE" => route.method.red(),
        _ => route.method.normal(),
    };

    if show_middleware {
        let middleware_str = if route.middleware.is_empty() {
            "none".dimmed().to_string()
        } else {
            route.middleware.join(", ")
        };

        println!(
            "│ {} │ {} │",
            format!("{:11}", method_color),
            format!("{:67}", route.uri)
        );
        println!(
            "│ {} │ {} │",
            format!("{:11}", middleware_str),
            format!("{:67}", route.action)
        );
    } else {
        println!(
            "│ {} │ {} │ {} │",
            format!("{:11}", method_color),
            format!("{:35}", route.uri),
            format!("{:31}", route.action)
        );
    }

    if let Some(ref name) = route.name {
        println!(
            "│ {} │ {} │",
            format!("{:11}", "Name:".dimmed()),
            format!("{:67}", name.dimmed())
        );
    }

    println!("├─────────────┼─────────────────────────────────────────────────────────────────────┤");
}
