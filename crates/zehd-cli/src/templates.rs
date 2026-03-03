use anyhow::{Context, Result};
use rust_embed::Embed;

#[derive(Embed)]
#[folder = "templates/"]
pub struct Templates;

pub fn render(path: &str, project_name: &str) -> Result<String> {
    let file = Templates::get(path)
        .with_context(|| format!("template not found: {path}"))?;
    let content = std::str::from_utf8(file.data.as_ref())
        .with_context(|| format!("template is not valid UTF-8: {path}"))?;
    Ok(content.replace("{{project_name}}", project_name))
}
