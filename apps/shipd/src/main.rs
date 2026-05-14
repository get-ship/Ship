use tracing_subscriber::{EnvFilter, Layer, layer::SubscriberExt, util::SubscriberInitExt};

fn main() -> anyhow::Result<()> {
    let args: Vec<String> = std::env::args().collect();

    if args.iter().any(|a| a == "--help" || a == "-h") {
        print_help();
        return Ok(());
    }

    let global_dir = ship_substrate::project::get_global_dir()?;

    let logs_dir = ship_substrate::ShipHome::at(global_dir.clone())
        .machine_dir()
        .join("logs");
    let _ = std::fs::create_dir_all(&logs_dir);

    // Rotate log file if >10 MB.
    let log_path = logs_dir.join("shipd.log");
    if let Ok(meta) = std::fs::metadata(&log_path)
        && meta.len() > 10 * 1024 * 1024 {
            let _ = std::fs::rename(&log_path, logs_dir.join("shipd.log.old"));
        }

    // File appender — JSON-structured logs for crash forensics.
    let file_appender = tracing_appender::rolling::never(&logs_dir, "shipd.log");
    let file_layer = tracing_subscriber::fmt::layer()
        .json()
        .with_writer(file_appender)
        .with_filter(EnvFilter::new("info"));

    // Stderr layer — human-readable for interactive use.
    let stderr_layer = tracing_subscriber::fmt::layer()
        .with_filter(EnvFilter::from_default_env());

    tracing_subscriber::registry()
        .with(file_layer)
        .with(stderr_layer)
        .init();

    // --port is optional — None means "auto-derive from path_hash, honoring
    // local.jsonc:worktrees.ports pins". See `ship_substrate::home::worktree_port_for`.
    let port = extract_u16_arg(&args, "--port");
    let host = extract_str_arg(&args, "--host").unwrap_or_else(|| "127.0.0.1".to_string());
    let dev = args.iter().any(|a| a == "--dev");

    // --mode development|production (default: development).
    // SHIP_DAEMON_MODE env var is a fallback when --mode is not passed.
    let mode_str = extract_str_arg(&args, "--mode")
        .or_else(|| ship_substrate::env::read(ship_substrate::env::Var::DaemonMode))
        .unwrap_or_else(|| "development".to_string());
    let daemon_mode = ship_sui::DaemonMode::from_str(&mode_str);

    // Project resolution:
    //   1. `--dev-worktree <path>` (dev-only; routes the daemon into a
    //      per-worktree IPC dir + hash-derived port so multiple Ship-dev
    //      worktrees can run side-by-side without colliding).
    //   2. `--workspace <path>` — deprecated alias kept for the dev scripts
    //      (`scripts/dev/start.sh`) that still pass it. Same effect: scopes
    //      the daemon to that project root so its per-worktree daemon dir
    //      matches what `ship daemon dir` resolves from cwd. Without this,
    //      `discover_workspace` redirects a linked worktree to the main
    //      checkout's `.ship/`, the daemon lands in the main checkout's
    //      daemon dir, and the dev scripts wait on a port file that never
    //      appears.
    //   3. Otherwise walk up from CWD looking for `.ship/`, like git
    //      discovers the repo root.
    let workspace = extract_str_arg(&args, "--dev-worktree")
        .or_else(|| extract_str_arg(&args, "--workspace"))
        .map(std::path::PathBuf::from)
        .or_else(discover_workspace);

    let rt = tokio::runtime::Builder::new_multi_thread()
        .enable_all()
        .build()?;
    rt.block_on(shipd::run_network(host, port, dev, workspace, daemon_mode))
}

fn extract_u16_arg(args: &[String], flag: &str) -> Option<u16> {
    args.windows(2)
        .find(|w| w[0] == flag)
        .and_then(|w| w[1].parse().ok())
}

fn extract_str_arg(args: &[String], flag: &str) -> Option<String> {
    args.windows(2)
        .find(|w| w[0] == flag)
        .map(|w| w[1].clone())
}

/// Walk up from CWD looking for a `.ship/` directory. Returns the project
/// root (the dir containing `.ship/`), not `.ship/` itself.
fn discover_workspace() -> Option<std::path::PathBuf> {
    let cwd = std::env::current_dir().ok()?;
    let ship_dir = ship_substrate::project::resolve_project_ship_dir(&cwd)?;
    ship_dir.parent().map(std::path::Path::to_path_buf)
}

fn print_help() {
    println!(
        "shipd — Ship daemon

USAGE:
    shipd [OPTIONS]

OPTIONS:
    --host <HOST>                  Bind address (default: 127.0.0.1)
    --port <PORT>                  Port to listen on (default: auto-derived)
    --dev                          Enable dev mode: relaxed auth, auto-pair
    --mode <development|production>
                                   Operating mode (default: development).
                                   production: immutable cache headers, HMR
                                   routes disabled, SSR cache enabled, error
                                   details redacted. Env var: SHIP_DAEMON_MODE
    --workspace <PATH>             Project root to scope this daemon to
    --dev-worktree <PATH>          (Dev) per-worktree IPC dir override
    -h, --help                     Print this help
"
    );
}
