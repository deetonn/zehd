use std::fs;
use std::sync::Arc;

use anyhow::{bail, Result};
use owo_colors::OwoColorize;
use zehd_rune::value::Value;
use zehd_ward::vm::StackVm;
use zehd_ward::VmBackend;

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
    let (module_types, native_registry, native_fns) = zehd_server::std_lib::build_std();
    let check_result = zehd_sigil::check(&parse_result.program, &source, &module_types);
    if check_result.has_errors() {
        for err in check_result.errors.iter().filter(|e| e.is_error()) {
            eprintln!("  {} {}", "error".red().bold(), err);
        }
        std::process::exit(1);
    }

    // Compile
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

    let module = compile_result.module;

    // Execute server_init (top-level code)
    let context = zehd_ward::Context {
        module,
        native_fns: Arc::new(native_fns),
        module_fns: Arc::new(vec![]),
    };

    let mut vm = StackVm::new();
    if let Some(ref init_chunk) = context.module.server_init {
        match vm.execute(init_chunk, &context) {
            Ok(value) => {
                if value != Value::Unit {
                    println!("{value}");
                }
            }
            Err(e) => {
                eprintln!("  {} {}", "runtime error".red().bold(), e.message);
                std::process::exit(1);
            }
        }
    }

    Ok(())
}
