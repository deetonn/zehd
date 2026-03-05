use std::fs;

use anyhow::Result;
use owo_colors::OwoColorize;

use crate::cli::CheckArgs;
use crate::config::load_project_config;

pub fn run(args: CheckArgs) -> Result<()> {
    if let Some(file) = args.file {
        check_single_file(&file)
    } else {
        check_project()
    }
}

fn check_single_file(file: &std::path::Path) -> Result<()> {
    if !file.exists() {
        anyhow::bail!("File not found: {}", file.display());
    }

    let source = fs::read_to_string(file)?;
    let module_types = zehd_sigil::std_module_types();
    let (errors, warnings) = check_source(&source, &module_types);

    print_diagnostics(&errors, &warnings);

    if !errors.is_empty() {
        eprintln!(
            "  {} {} error(s), {} warning(s)",
            "result".bold(),
            errors.len(),
            warnings.len(),
        );
        std::process::exit(1);
    }

    println!("  {} no errors", "ok".green().bold());
    if !warnings.is_empty() {
        println!("  {} {} warning(s)", "".yellow(), warnings.len());
    }

    Ok(())
}

fn check_project() -> Result<()> {
    let pc = load_project_config()?;
    let module_types = zehd_sigil::std_module_types();

    let mut total_files = 0;
    let mut total_errors = 0;
    let mut total_warnings = 0;

    // Check main.z if present
    let main_path = pc.project_dir.join("main.z");
    if main_path.exists() {
        let source = fs::read_to_string(&main_path)?;
        let (errors, warnings) = check_source(&source, &module_types);
        if !errors.is_empty() || !warnings.is_empty() {
            eprintln!("  {} {}", "file".dimmed(), main_path.display());
            print_diagnostics(&errors, &warnings);
        }
        total_files += 1;
        total_errors += errors.len();
        total_warnings += warnings.len();
    }

    // Discover and check modules
    let discovered_modules = zehd_server::discover::discover_modules(&pc.module_dirs)?;
    for module in &discovered_modules {
        let (errors, warnings) = check_source(&module.source, &module_types);
        if !errors.is_empty() || !warnings.is_empty() {
            eprintln!("  {} {}", "file".dimmed(), module.file_path.display());
            print_diagnostics(&errors, &warnings);
        }
        total_files += 1;
        total_errors += errors.len();
        total_warnings += warnings.len();
    }

    // Discover and check routes
    let routes = zehd_server::discover::discover_routes(&pc.routes_dir)?;
    for route in &routes {
        let (errors, warnings) = check_source(&route.source, &module_types);
        if !errors.is_empty() || !warnings.is_empty() {
            eprintln!("  {} {}", "file".dimmed(), route.file_path.display());
            print_diagnostics(&errors, &warnings);
        }
        total_files += 1;
        total_errors += errors.len();
        total_warnings += warnings.len();
    }

    println!();
    println!(
        "  Checked {} file(s): {} error(s), {} warning(s)",
        total_files, total_errors, total_warnings,
    );

    if total_errors > 0 {
        std::process::exit(1);
    }

    Ok(())
}

/// Parse and type-check a source string. Returns (errors, warnings) as display strings.
fn check_source(
    source: &str,
    module_types: &zehd_sigil::ModuleTypes,
) -> (Vec<String>, Vec<String>) {
    let parse_result = zehd_codex::parse(source);
    if !parse_result.is_ok() {
        let errors: Vec<String> = parse_result.errors.iter().map(|e| e.to_string()).collect();
        return (errors, vec![]);
    }

    let check_result = zehd_sigil::check(&parse_result.program, source, module_types);
    let errors: Vec<String> = check_result
        .errors
        .iter()
        .filter(|e| e.is_error())
        .map(|e| e.to_string())
        .collect();
    let warnings: Vec<String> = check_result
        .errors
        .iter()
        .filter(|e| e.is_warning())
        .map(|e| e.to_string())
        .collect();

    (errors, warnings)
}

fn print_diagnostics(errors: &[String], warnings: &[String]) {
    for err in errors {
        eprintln!("    {} {}", "error".red().bold(), err);
    }
    for warn in warnings {
        eprintln!("    {} {}", "warning".yellow().bold(), warn);
    }
}
