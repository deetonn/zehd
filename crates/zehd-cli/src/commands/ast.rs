use std::fs;

use anyhow::{bail, Result};
use owo_colors::OwoColorize;
use zehd_codex::ast::*;
use zehd_sigil::checker::TypeTable;

use crate::cli::AstArgs;

pub fn run(args: AstArgs) -> Result<()> {
    if !args.file.exists() {
        bail!("File not found: {}", args.file.display());
    }

    let source = fs::read_to_string(&args.file)?;
    let parse_result = zehd_codex::parse(&source);

    if !parse_result.is_ok() {
        for err in &parse_result.errors {
            eprintln!("  {} {}", "error".red().bold(), err);
        }
        eprintln!();
        std::process::exit(1);
    }

    let types = if args.typed {
        let module_types = zehd_sigil::std_module_types();
        let check_result = zehd_sigil::check(&parse_result.program, &source, &module_types);
        for err in &check_result.errors {
            let severity = if err.is_error() {
                "error".red().bold().to_string()
            } else {
                "warning".yellow().bold().to_string()
            };
            eprintln!("  {} {}", severity, err);
        }
        if check_result.has_errors() {
            eprintln!();
            std::process::exit(1);
        }
        Some(check_result.types)
    } else {
        None
    };

    let printer = AstPrinter { types: types.as_ref() };
    printer.print_program(&parse_result.program);

    Ok(())
}

struct AstPrinter<'a> {
    types: Option<&'a TypeTable>,
}

impl<'a> AstPrinter<'a> {
    fn print_program(&self, program: &Program) {
        println!("{}", "Program".cyan().bold());
        for item in &program.items {
            self.print_item(item, 1);
        }
    }

    fn print_item(&self, item: &Item, depth: usize) {
        let indent = "  ".repeat(depth);
        match &item.kind {
            ItemKind::Import(imp) => {
                let names: Vec<&str> = imp.names.iter().map(|n| n.name.name.as_str()).collect();
                let path: Vec<&str> = imp.path.segments.iter().map(|s| s.name.as_str()).collect();
                println!(
                    "{}{} {{ {} }} from {}",
                    indent,
                    "Import".cyan(),
                    names.join(", "),
                    path.join("::"),
                );
            }
            ItemKind::TypeDef(td) => {
                println!("{}{} {}", indent, "TypeDef".cyan(), td.name.name.green());
                for field in &td.fields {
                    println!(
                        "{}  {}: {:?}",
                        indent,
                        field.name.name,
                        field.ty.kind
                    );
                }
            }
            ItemKind::EnumDef(ed) => {
                println!("{}{} {}", indent, "EnumDef".cyan(), ed.name.name.green());
                for variant in &ed.variants {
                    if let Some(payload) = &variant.payload {
                        println!("{}  {}({:?})", indent, variant.name.name, payload.kind);
                    } else {
                        println!("{}  {}", indent, variant.name.name);
                    }
                }
            }
            ItemKind::Function(f) => {
                let params: Vec<String> = f
                    .params
                    .iter()
                    .map(|p| {
                        if let Some(ty) = &p.ty {
                            format!("{}: {:?}", p.name.name, ty.kind)
                        } else {
                            p.name.name.clone()
                        }
                    })
                    .collect();
                let ret = f
                    .return_type
                    .as_ref()
                    .map(|t| format!(": {:?}", t.kind))
                    .unwrap_or_default();
                println!(
                    "{}{} {}({}){} ",
                    indent,
                    "Function".cyan(),
                    f.name.name.green(),
                    params.join(", "),
                    ret,
                );
                self.print_block(&f.body, depth + 1);
            }
            ItemKind::VarDecl(vd) => {
                let kw = if vd.mutable { "let" } else { "const" };
                print!("{}{} {} {}", indent, "VarDecl".cyan(), kw.yellow(), vd.name.name);
                if let Some(ty) = &vd.ty {
                    print!(": {:?}", ty.kind);
                }
                println!();
                if let Some(init) = &vd.initializer {
                    self.print_expr(init, depth + 1);
                }
            }
            ItemKind::HttpBlock(hb) => {
                let method = match hb.method {
                    HttpMethod::Get => "GET",
                    HttpMethod::Post => "POST",
                    HttpMethod::Put => "PUT",
                    HttpMethod::Patch => "PATCH",
                    HttpMethod::Delete => "DELETE",
                };
                println!("{}{} {}", indent, "HttpBlock".cyan(), method.green());
                self.print_block(&hb.body, depth + 1);
            }
            ItemKind::InitBlock(ib) => {
                println!("{}{}", indent, "InitBlock".cyan());
                self.print_block(&ib.body, depth + 1);
            }
            ItemKind::ErrorHandler(eh) => {
                println!("{}{} ({})", indent, "ErrorHandler".cyan(), eh.param.name);
                self.print_block(&eh.body, depth + 1);
            }
            ItemKind::ExprStmt(es) => {
                println!("{}{}", indent, "ExprStmt".cyan());
                self.print_expr(&es.expr, depth + 1);
            }
        }
    }

    fn print_block(&self, block: &Block, depth: usize) {
        let indent = "  ".repeat(depth);
        println!("{}{}", indent, "Block".dimmed());
        for stmt in &block.stmts {
            self.print_stmt(stmt, depth + 1);
        }
        if let Some(tail) = &block.tail_expr {
            let tail_indent = "  ".repeat(depth + 1);
            println!("{}{}", tail_indent, "TailExpr".dimmed());
            self.print_expr(tail, depth + 2);
        }
    }

    fn print_stmt(&self, stmt: &Stmt, depth: usize) {
        let indent = "  ".repeat(depth);
        match &stmt.kind {
            StmtKind::VarDecl(vd) => {
                let kw = if vd.mutable { "let" } else { "const" };
                print!("{}{} {} {}", indent, "VarDecl".cyan(), kw.yellow(), vd.name.name);
                if let Some(ty) = &vd.ty {
                    print!(": {:?}", ty.kind);
                }
                println!();
                if let Some(init) = &vd.initializer {
                    self.print_expr(init, depth + 1);
                }
            }
            StmtKind::ExprStmt(es) => {
                println!("{}{}", indent, "ExprStmt".cyan());
                self.print_expr(&es.expr, depth + 1);
            }
            StmtKind::Return(ret) => {
                println!("{}{}", indent, "Return".cyan());
                if let Some(val) = &ret.value {
                    self.print_expr(val, depth + 1);
                }
            }
            StmtKind::Break => println!("{}{}", indent, "Break".cyan()),
            StmtKind::Continue => println!("{}{}", indent, "Continue".cyan()),
            StmtKind::For(f) => {
                println!("{}{} {} in", indent, "For".cyan(), f.binding.name.green());
                self.print_expr(&f.iterable, depth + 1);
                self.print_block(&f.body, depth + 1);
            }
            StmtKind::While(w) => {
                println!("{}{}", indent, "While".cyan());
                self.print_expr(&w.condition, depth + 1);
                self.print_block(&w.body, depth + 1);
            }
            StmtKind::Assignment(a) => {
                println!("{}{}", indent, "Assignment".cyan());
                self.print_expr(&a.target, depth + 1);
                self.print_expr(&a.value, depth + 1);
            }
        }
    }

    fn print_expr(&self, expr: &Expr, depth: usize) {
        let indent = "  ".repeat(depth);

        // Type annotation suffix
        let type_suffix = self
            .types
            .and_then(|t| t.get(&expr.id))
            .map(|ty| format!(" {}", format!(": {ty:?}").dimmed()))
            .unwrap_or_default();

        match &expr.kind {
            ExprKind::IntLiteral(n) => println!("{}{}{}", indent, format!("Int({n})").yellow(), type_suffix),
            ExprKind::FloatLiteral(n) => println!("{}{}{}", indent, format!("Float({n})").yellow(), type_suffix),
            ExprKind::StringLiteral(s) => println!("{}{}{}",indent, format!("String({s:?})").yellow(), type_suffix),
            ExprKind::TimeLiteral(ms) => println!("{}{}{}", indent, format!("Time({ms}ms)").yellow(), type_suffix),
            ExprKind::BoolLiteral(b) => println!("{}{}{}", indent, format!("Bool({b})").yellow(), type_suffix),
            ExprKind::NoneLiteral => println!("{}{}{}", indent, "None".yellow(), type_suffix),
            ExprKind::EnumConstructor { name, arg } => {
                println!("{}{}{}", indent, format!("EnumConstructor({})", name.name).cyan(), type_suffix);
                self.print_expr(arg, depth + 1);
            }
            ExprKind::Ident(ident) => {
                println!("{}{}{}", indent, format!("Ident({})", ident.name).green(), type_suffix);
            }
            ExprKind::SelfExpr => println!("{}{}{}", indent, "Self".green(), type_suffix),
            ExprKind::Binary { op, left, right } => {
                println!("{}{}{}", indent, format!("Binary({op:?})").cyan(), type_suffix);
                self.print_expr(left, depth + 1);
                self.print_expr(right, depth + 1);
            }
            ExprKind::Unary { op, operand } => {
                println!("{}{}{}", indent, format!("Unary({op:?})").cyan(), type_suffix);
                self.print_expr(operand, depth + 1);
            }
            ExprKind::Try(inner) => {
                println!("{}{}{}", indent, "Try(?)".cyan(), type_suffix);
                self.print_expr(inner, depth + 1);
            }
            ExprKind::FieldAccess { object, field } => {
                println!("{}{}{}", indent, format!("FieldAccess(.{})", field.name).cyan(), type_suffix);
                self.print_expr(object, depth + 1);
            }
            ExprKind::Index { object, index } => {
                println!("{}{}{}", indent, "Index".cyan(), type_suffix);
                self.print_expr(object, depth + 1);
                self.print_expr(index, depth + 1);
            }
            ExprKind::Call { callee, type_args, args } => {
                let ta = if type_args.is_empty() {
                    String::new()
                } else {
                    format!("<{} type args>", type_args.len())
                };
                println!("{}{}{}", indent, format!("Call{ta}({} args)", args.len()).cyan(), type_suffix);
                self.print_expr(callee, depth + 1);
                for arg in args {
                    self.print_expr(arg, depth + 1);
                }
            }
            ExprKind::If { condition, then_block, else_block } => {
                println!("{}{}{}", indent, "If".cyan(), type_suffix);
                self.print_expr(condition, depth + 1);
                self.print_block(then_block, depth + 1);
                if let Some(eb) = else_block {
                    match eb {
                        ElseBranch::ElseBlock(block) => {
                            let ei = "  ".repeat(depth + 1);
                            println!("{}{}", ei, "Else".cyan());
                            self.print_block(block, depth + 2);
                        }
                        ElseBranch::ElseIf(expr) => {
                            let ei = "  ".repeat(depth + 1);
                            println!("{}{}", ei, "ElseIf".cyan());
                            self.print_expr(expr, depth + 2);
                        }
                    }
                }
            }
            ExprKind::Match { scrutinee, arms } => {
                println!("{}{}{}", indent, "Match".cyan(), type_suffix);
                self.print_expr(scrutinee, depth + 1);
                for arm in arms {
                    let ai = "  ".repeat(depth + 1);
                    println!("{}{} {:?}", ai, "Arm".dimmed(), arm.pattern.kind);
                    self.print_expr(&arm.body, depth + 2);
                }
            }
            ExprKind::ArrowFunction { params, body, .. } => {
                let pnames: Vec<&str> = params.iter().map(|p| p.name.name.as_str()).collect();
                println!("{}{}{}", indent, format!("Arrow({})", pnames.join(", ")).cyan(), type_suffix);
                match body {
                    ArrowBody::Expr(e) => self.print_expr(e, depth + 1),
                    ArrowBody::Block(b) => self.print_block(b, depth + 1),
                }
            }
            ExprKind::ObjectLiteral { fields } => {
                let keys: Vec<&str> = fields.iter().map(|f| f.key.name.as_str()).collect();
                println!("{}{}{}", indent, format!("Object({{ {} }})", keys.join(", ")).cyan(), type_suffix);
                for field in fields {
                    if let Some(val) = &field.value {
                        self.print_expr(val, depth + 1);
                    }
                }
            }
            ExprKind::ListLiteral { elements } => {
                println!("{}{}{}", indent, format!("List({} items)", elements.len()).cyan(), type_suffix);
                for elem in elements {
                    self.print_expr(elem, depth + 1);
                }
            }
            ExprKind::InterpolatedString { parts } => {
                println!("{}{}{}", indent, "InterpolatedString".cyan(), type_suffix);
                for part in parts {
                    match part {
                        InterpolatedPart::Literal(s, _) => {
                            let pi = "  ".repeat(depth + 1);
                            println!("{}{}", pi, format!("Literal({s:?})").yellow());
                        }
                        InterpolatedPart::Expr(e) => self.print_expr(e, depth + 1),
                    }
                }
            }
            ExprKind::Block(block) => {
                println!("{}{}{}", indent, "BlockExpr".cyan(), type_suffix);
                self.print_block(block, depth + 1);
            }
            ExprKind::Grouped(inner) => {
                println!("{}{}{}", indent, "Grouped".cyan(), type_suffix);
                self.print_expr(inner, depth + 1);
            }
        }
    }
}
