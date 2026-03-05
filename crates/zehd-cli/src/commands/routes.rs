use anyhow::Result;
use owo_colors::OwoColorize;
use zehd_codex::ast::{HttpMethod, ItemKind};

use crate::config::load_project_config;

pub fn run() -> Result<()> {
    let pc = load_project_config()?;
    let routes = zehd_server::discover::discover_routes(&pc.routes_dir)?;

    if routes.is_empty() {
        println!("  {} no routes found", "warning".yellow().bold());
        return Ok(());
    }

    for route in &routes {
        // Parse to extract HTTP method blocks
        let parse_result = zehd_codex::parse(&route.source);
        let methods: Vec<&str> = parse_result
            .program
            .items
            .iter()
            .filter_map(|item| match &item.kind {
                ItemKind::HttpBlock(hb) => Some(match hb.method {
                    HttpMethod::Get => "GET",
                    HttpMethod::Post => "POST",
                    HttpMethod::Put => "PUT",
                    HttpMethod::Patch => "PATCH",
                    HttpMethod::Delete => "DELETE",
                }),
                _ => None,
            })
            .collect();

        if methods.is_empty() {
            println!(
                "  {}  {}  {}",
                "(none)".dimmed(),
                route.url_path,
                route.file_path.display().to_string().dimmed(),
            );
        } else {
            for method in &methods {
                let colored = match *method {
                    "GET" => method.green().to_string(),
                    "POST" => method.blue().to_string(),
                    "PUT" => method.yellow().to_string(),
                    "PATCH" => method.yellow().to_string(),
                    "DELETE" => method.red().to_string(),
                    _ => method.to_string(),
                };
                println!(
                    "  {:<8} {}  {}",
                    colored,
                    route.url_path,
                    route.file_path.display().to_string().dimmed(),
                );
            }
        }
    }

    Ok(())
}
