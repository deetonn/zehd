use std::fs;
use std::path::Path;

use anyhow::{bail, Result};
use owo_colors::OwoColorize;

use crate::cli::NewArgs;
use crate::templates;

pub fn run(args: NewArgs) -> Result<()> {
    cliclack::intro(format!("{}", "  zehd  ".on_cyan().black().bold()))?;

    let name = match args.name {
        Some(name) => {
            validate_name(&name)?;
            cliclack::log::info(format!("Project name: {}", name.bold()))?;
            name
        }
        None => {
            let name: String = cliclack::input("What is your project name?")
                .placeholder("my-app")
                .validate(|input: &String| {
                    if input.is_empty() {
                        return Err("Project name cannot be empty".to_string());
                    }
                    if let Err(e) = validate_name(input) {
                        return Err(e.to_string());
                    }
                    Ok(())
                })
                .interact()?;
            name
        }
    };

    let project_dir = Path::new(&name);
    if project_dir.exists() {
        bail!(
            "Directory {} already exists",
            name.bold()
        );
    }

    let spinner = cliclack::spinner();
    spinner.start("Scaffolding project...");

    // Create directories
    fs::create_dir_all(project_dir.join("routes"))?;
    fs::create_dir_all(project_dir.join("lib"))?;
    fs::create_dir_all(project_dir.join("public"))?;

    // Write template files
    fs::write(
        project_dir.join("zehd.toml"),
        templates::render("zehd.toml.tmpl", &name)?,
    )?;
    fs::write(
        project_dir.join("main.z"),
        templates::render("main.z.tmpl", &name)?,
    )?;
    fs::write(
        project_dir.join("routes/index.z"),
        templates::render("routes/index.z.tmpl", &name)?,
    )?;

    spinner.stop("Project scaffolded");

    let next_steps = format!(
        "cd {} && zehd dev",
        name
    );

    cliclack::outro(format!(
        "Done! Next steps:\n\n  {}",
        next_steps.cyan()
    ))?;

    Ok(())
}

fn validate_name(name: &str) -> Result<()> {
    if name.is_empty() {
        bail!("Project name cannot be empty");
    }

    if !name.chars().next().unwrap().is_alphanumeric() {
        bail!("Project name must start with a letter or number");
    }

    for ch in name.chars() {
        if !ch.is_alphanumeric() && ch != '-' && ch != '_' {
            bail!(
                "Project name can only contain letters, numbers, hyphens, and underscores (got '{ch}')"
            );
        }
    }

    Ok(())
}
