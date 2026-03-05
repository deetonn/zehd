use std::fs;

use anyhow::{bail, Result};
use owo_colors::OwoColorize;
use zehd_tome::TokenKind;

use crate::cli::FileArgs;

pub fn run(args: FileArgs) -> Result<()> {
    if !args.file.exists() {
        bail!("File not found: {}", args.file.display());
    }

    let source = fs::read_to_string(&args.file)?;
    let result = zehd_tome::lex(&source);

    if !result.is_ok() {
        for err in &result.errors {
            eprintln!("  {} {}", "error".red().bold(), err);
        }
        eprintln!();
    }

    // Compute line/col for each token from byte offsets
    let line_starts = compute_line_starts(&source);

    for token in &result.tokens {
        let (line, col) = offset_to_line_col(&line_starts, token.span.start as usize);
        let lexeme = token.span.lexeme(&source);
        let kind_str = format!("{:?}", token.kind);

        let colored_kind = match &token.kind {
            k if k.is_keyword() => kind_str.cyan().to_string(),
            TokenKind::Integer(_)
            | TokenKind::Float(_)
            | TokenKind::String
            | TokenKind::TimeLiteral(_) => kind_str.yellow().to_string(),
            TokenKind::Identifier => kind_str.green().to_string(),
            TokenKind::Eof => kind_str.dimmed().to_string(),
            _ => kind_str.white().to_string(),
        };

        println!(
            "  {:<8} {:<24} {}",
            format!("{}:{}", line, col).dimmed(),
            colored_kind,
            lexeme.dimmed(),
        );
    }

    if !result.is_ok() {
        std::process::exit(1);
    }

    Ok(())
}

fn compute_line_starts(source: &str) -> Vec<usize> {
    let mut starts = vec![0];
    for (i, ch) in source.char_indices() {
        if ch == '\n' {
            starts.push(i + 1);
        }
    }
    starts
}

fn offset_to_line_col(line_starts: &[usize], offset: usize) -> (usize, usize) {
    let line = line_starts.partition_point(|&s| s <= offset).saturating_sub(1);
    let col = offset - line_starts[line] + 1;
    (line + 1, col)
}
