use std::fs;

use anyhow::{bail, Result};
use owo_colors::OwoColorize;
use zehd_rune::chunk::Chunk;
use zehd_rune::op::decode_ops;

use crate::cli::FileArgs;

pub fn run(args: FileArgs) -> Result<()> {
    if !args.file.exists() {
        bail!("File not found: {}", args.file.display());
    }

    let source = fs::read_to_string(&args.file)?;

    // Parse
    let parse_result = zehd_codex::parse(&source);
    if !parse_result.is_ok() {
        for err in &parse_result.errors {
            eprintln!("  {} {}", "error".red().bold(), err);
        }
        std::process::exit(1);
    }

    // Type check
    let module_types = zehd_sigil::std_module_types();
    let check_result = zehd_sigil::check(&parse_result.program, &source, &module_types);
    if check_result.has_errors() {
        for err in check_result.errors.iter().filter(|e| e.is_error()) {
            eprintln!("  {} {}", "error".red().bold(), err);
        }
        std::process::exit(1);
    }

    // Compile
    let (_, native_registry, _) = zehd_server::std_lib::build_std();
    let compile_result = zehd_rune::compile(
        &parse_result.program,
        check_result,
        &native_registry,
        &Default::default(),
    );
    if compile_result.has_errors() {
        for err in compile_result.errors.iter().filter(|e| e.is_error()) {
            eprintln!("  {} {}", "error".red().bold(), err);
        }
        std::process::exit(1);
    }

    let module = &compile_result.module;

    // Print server_init
    if let Some(ref chunk) = module.server_init {
        print_chunk("server_init", chunk);
    }

    // Print handlers
    for handler in &module.handlers {
        let method = match handler.method {
            zehd_codex::ast::HttpMethod::Get => "GET",
            zehd_codex::ast::HttpMethod::Post => "POST",
            zehd_codex::ast::HttpMethod::Put => "PUT",
            zehd_codex::ast::HttpMethod::Patch => "PATCH",
            zehd_codex::ast::HttpMethod::Delete => "DELETE",
        };
        print_chunk(&format!("handler:{method}"), &handler.chunk);
    }

    // Print init block
    if let Some(ref chunk) = module.init_block {
        print_chunk("init_block", chunk);
    }

    // Print error handler
    if let Some(ref chunk) = module.error_handler {
        print_chunk("error_handler", chunk);
    }

    // Print functions
    for func in &module.functions {
        print_chunk(&format!("fn:{}", func.name), &func.chunk);
    }

    Ok(())
}

fn print_chunk(label: &str, chunk: &Chunk) {
    println!();
    println!(
        "  {} {} (arity={}, locals={})",
        "==".dimmed(),
        label.cyan().bold(),
        chunk.arity,
        chunk.local_count,
    );

    // Disassemble bytecode
    let instructions = decode_ops(&chunk.code);
    let mut offset = 0usize;
    for instr in &instructions {
        println!("    {:04}  {}", offset.to_string().dimmed(), instr);
        // Advance offset: 1 for opcode + operand_size
        let op = match instr {
            zehd_rune::op::Instruction::Simple(op)
            | zehd_rune::op::Instruction::U16(op, _)
            | zehd_rune::op::Instruction::U8(op, _)
            | zehd_rune::op::Instruction::U16U16(op, _, _)
            | zehd_rune::op::Instruction::U16U8(op, _, _) => *op,
        };
        offset += 1 + op.operand_size();
    }

    // Print constant pool
    if !chunk.constants.is_empty() {
        println!("    {}", "constants:".dimmed());
        for (i, val) in chunk.constants.iter().enumerate() {
            println!("      [{i}] {val}");
        }
    }
}
