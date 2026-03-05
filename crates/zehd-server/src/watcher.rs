use std::collections::HashMap;
use std::path::PathBuf;
use std::sync::Arc;
use std::time::Instant;

use arc_swap::ArcSwap;
use notify::{RecommendedWatcher, RecursiveMode, Watcher};
use owo_colors::OwoColorize;
use tokio::sync::mpsc;
use zehd_rune::registry::{ModuleFnRegistry, NativeRegistry};
use zehd_rune::value::Value;
use zehd_sigil::ModuleTypes;
use zehd_ward::{ModuleFunction, NativeFn};

use crate::compile;
use crate::discover;
use crate::error::StartupError;
use crate::router::RouteTable;

/// Spawn a filesystem watcher on `routes_dir`.
///
/// On any `.z` file change, re-discovers and recompiles all routes, then
/// atomically swaps the route table via `ArcSwap`. On error, the old table
/// stays live and the error is printed.
///
/// Returns the `RecommendedWatcher` — caller must hold this binding alive.
pub fn spawn(
    routes_dir: PathBuf,
    route_table: Arc<ArcSwap<RouteTable>>,
    module_types: ModuleTypes,
    native_registry: NativeRegistry,
    module_fn_registry: ModuleFnRegistry,
    native_fns: Arc<Vec<NativeFn>>,
    module_fns: Arc<Vec<ModuleFunction>>,
    global_di: HashMap<String, Value>,
) -> Result<RecommendedWatcher, StartupError> {
    let (tx, rx) = mpsc::channel::<notify::Event>(64);

    // Create the OS watcher — sends events into the tokio channel.
    let mut watcher = {
        let tx = tx.clone();
        RecommendedWatcher::new(
            move |res: Result<notify::Event, notify::Error>| {
                if let Ok(event) = res {
                    let _ = tx.blocking_send(event);
                }
            },
            notify::Config::default(),
        )
        .map_err(|source| StartupError::WatcherError { source })?
    };

    watcher
        .watch(&routes_dir, RecursiveMode::Recursive)
        .map_err(|source| StartupError::WatcherError { source })?;

    // Spawn the async reload loop.
    tokio::spawn(watch_loop(
        rx,
        routes_dir,
        route_table,
        module_types,
        native_registry,
        module_fn_registry,
        native_fns,
        module_fns,
        global_di,
    ));

    Ok(watcher)
}

/// Debounced reload loop — collects events, filters to .z files, recompiles.
async fn watch_loop(
    mut rx: mpsc::Receiver<notify::Event>,
    routes_dir: PathBuf,
    route_table: Arc<ArcSwap<RouteTable>>,
    module_types: ModuleTypes,
    native_registry: NativeRegistry,
    module_fn_registry: ModuleFnRegistry,
    native_fns: Arc<Vec<NativeFn>>,
    module_fns: Arc<Vec<ModuleFunction>>,
    global_di: HashMap<String, Value>,
) {
    loop {
        // Wait for the first event.
        let Some(first) = rx.recv().await else {
            return; // channel closed
        };

        // Debounce: collect more events for 100ms.
        tokio::time::sleep(std::time::Duration::from_millis(100)).await;
        let mut events = vec![first];
        while let Ok(ev) = rx.try_recv() {
            events.push(ev);
        }

        // Filter to .z file changes only.
        let changed_z_files: Vec<&PathBuf> = events
            .iter()
            .flat_map(|ev| &ev.paths)
            .filter(|p| p.extension().and_then(|e| e.to_str()) == Some("z"))
            .collect();

        if changed_z_files.is_empty() {
            continue;
        }

        // Extract a short display name from the first changed file.
        let display_name = changed_z_files[0]
            .file_name()
            .and_then(|n| n.to_str())
            .unwrap_or("routes");

        let start = Instant::now();

        // Full recompile.
        let routes = match discover::discover_routes(&routes_dir) {
            Ok(r) => r,
            Err(e) => {
                eprintln!("  {} {}", "error".red().bold(), e);
                continue;
            }
        };

        let (compiled, errors) =
            compile::compile_routes(routes, &module_types, &native_registry, &module_fn_registry);

        if !errors.is_empty() {
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
            continue;
        }

        match RouteTable::build(compiled, Arc::clone(&native_fns), Arc::clone(&module_fns), &global_di) {
            Ok(new_table) => {
                route_table.store(Arc::new(new_table));
                let ms = start.elapsed().as_millis();
                println!(
                    "  {} {} in {}",
                    "reloaded".green(),
                    display_name,
                    format!("{ms}ms").dimmed()
                );
            }
            Err(e) => {
                eprintln!("  {} {}", "error".red().bold(), e);
            }
        }
    }
}
