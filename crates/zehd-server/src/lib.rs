mod compile;
mod discover;
mod handler;
mod json;
mod router;
mod std_lib;
mod watcher;

pub mod config;
pub mod error;

use std::collections::HashMap;
use std::sync::Arc;
use std::time::Instant;

use arc_swap::ArcSwap;
use axum::Router;
use owo_colors::OwoColorize;
use tokio::net::TcpListener;
use zehd_rune::value::Value;
use zehd_ward::vm::StackVm;
use zehd_ward::VmBackend;

use config::ServerOptions;
use error::StartupError;
use router::RouteTable;

/// Start the zehd HTTP server.
///
/// Discovers routes, compiles them, builds the route table, and serves HTTP.
/// This is the single public entry point — the CLI calls this function.
pub async fn start(options: ServerOptions) -> Result<(), StartupError> {
    let start_time = Instant::now();

    // 0. Build standard library
    let (module_types, native_registry, native_fns) = std_lib::build_std();
    let native_fns = Arc::new(native_fns);

    // 0b. Compile and execute main.z (if present) to extract DI registry
    let global_di = process_main_z(
        &options.project_dir,
        &module_types,
        &native_registry,
        &native_fns,
    )?;

    // 1. Discover route files
    let routes = discover::discover_routes(&options.routes_dir)?;

    if routes.is_empty() {
        eprintln!(
            "  {}  no route files found in {}",
            "warning".yellow().bold(),
            options.routes_dir.display()
        );
    }

    // 2. Compile all routes
    let (compiled, errors) =
        compile::compile_routes(routes, &module_types, &native_registry);

    if !errors.is_empty() {
        // Print each error before failing
        for err in &errors {
            eprintln!();
            eprintln!(
                "  {} {}",
                "error".red().bold(),
                err.url_path.bold()
            );
            eprintln!("  {}", err.file_path.display().dimmed());
            for msg in &err.messages {
                eprintln!("    {msg}");
            }
        }
        eprintln!();

        return Err(StartupError::CompilationFailed {
            count: errors.len(),
            errors,
        });
    }

    // 3. Build route table (runs server_init for each route)
    let route_table = RouteTable::build(compiled, Arc::clone(&native_fns), &global_di)?;

    // 4. Collect route info for the banner
    let mut route_lines: Vec<(String, String)> = Vec::new();
    let mut sorted_paths: Vec<&String> = route_table.routes.keys().collect();
    sorted_paths.sort();

    for path in &sorted_paths {
        let entry = &route_table.routes[*path];
        for method in &entry.allowed_methods {
            route_lines.push((method.as_str().to_string(), (*path).clone()));
        }
    }

    // 5. Wrap in ArcSwap for hot-reload
    let route_table = Arc::new(ArcSwap::from_pointee(route_table));

    // 6. Create concurrency semaphore (OOM safety net)
    let semaphore = Arc::new(tokio::sync::Semaphore::new(options.max_requests));

    // 7. Build axum app with fallback handler
    let table = Arc::clone(&route_table);
    let sem = Arc::clone(&semaphore);
    let request_logging = options.request_logging;
    let app = Router::new().fallback(move |request| {
        let current = table.load_full();
        let sem = Arc::clone(&sem);
        handler::handle_request(request, current, sem, request_logging)
    });

    // 8. Bind listener
    let addr = format!("{}:{}", options.host, options.port);
    let listener = TcpListener::bind(&addr).await.map_err(|source| {
        StartupError::BindError {
            host: options.host.clone(),
            port: options.port,
            source,
        }
    })?;

    // 9. Spawn filesystem watcher for hot-reload
    let _watcher = watcher::spawn(
        options.routes_dir.clone(),
        Arc::clone(&route_table),
        module_types,
        native_registry,
        Arc::clone(&native_fns),
        global_di,
    )?;

    let elapsed = start_time.elapsed();

    // 10. Print startup banner
    print_banner(&options, &route_lines, elapsed);

    // 11. Serve
    axum::serve(listener, app)
        .await
        .map_err(|source| StartupError::BindError {
            host: options.host,
            port: options.port,
            source,
        })?;

    Ok(())
}

/// Compile and execute main.z if it exists. Returns the DI registry.
fn process_main_z(
    project_dir: &std::path::Path,
    module_types: &zehd_sigil::ModuleTypes,
    native_registry: &zehd_rune::registry::NativeRegistry,
    native_fns: &Arc<Vec<zehd_ward::NativeFn>>,
) -> Result<HashMap<String, Value>, StartupError> {
    let main_path = project_dir.join("main.z");
    if !main_path.exists() {
        return Ok(HashMap::new());
    }

    let source = std::fs::read_to_string(&main_path).map_err(|source| {
        StartupError::IoError {
            path: main_path.clone(),
            source,
        }
    })?;

    let route = discover::DiscoveredRoute {
        url_path: "main.z".to_string(),
        file_path: main_path,
        source,
    };

    let module = compile::compile_one(&route, module_types, native_registry).map_err(|msgs| {
        StartupError::InitFailed {
            url_path: "main.z".to_string(),
            message: msgs.join("; "),
        }
    })?;

    let context = zehd_ward::Context {
        module,
        native_fns: Arc::clone(native_fns),
    };

    let mut vm = StackVm::new();
    if let Some(ref init_chunk) = context.module.server_init {
        vm.execute(init_chunk, &context).map_err(|e| {
            StartupError::InitFailed {
                url_path: "main.z".to_string(),
                message: e.message,
            }
        })?;
    }

    Ok(vm.di_registry().clone())
}

fn print_banner(
    options: &ServerOptions,
    route_lines: &[(String, String)],
    elapsed: std::time::Duration,
) {
    println!();
    println!(
        "  {} {}",
        "zehd".cyan().bold(),
        "v0.1.0".dimmed()
    );
    println!(
        "  {}  http://{}:{}",
        "→".green(),
        options.host,
        options.port
    );
    println!(
        "  {}  {} max concurrent requests",
        "→".green(),
        options.max_requests
    );
    if !options.request_logging {
        println!(
            "  {}  request logging {}",
            "→".green(),
            "disabled".yellow()
        );
    }

    if !route_lines.is_empty() {
        println!();
        println!("  {}", "routes".dimmed());

        // Find the longest method name for alignment
        let max_method_len = route_lines
            .iter()
            .map(|(m, _)| m.len())
            .max()
            .unwrap_or(0);

        for (method, path) in route_lines {
            println!(
                "    {:<width$}  {}",
                method.green(),
                path,
                width = max_method_len
            );
        }
    }

    println!(
        "  ready in {}",
        format!("{}ms", elapsed.as_millis()).green()
    );
}
