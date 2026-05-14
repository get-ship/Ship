use tracing_subscriber::{EnvFilter, Layer, layer::SubscriberExt, util::SubscriberInitExt};

fn main() -> anyhow::Result<()> {
    let global_dir = ship_application::project::get_global_dir()?;

    // Logs live in machine/logs/ per the home-layout ADR. Eagerly create the
    // directory so the file appender can attach before the migrator runs in
    // ship_runtime startup.
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

    let args: Vec<String> = std::env::args().collect();
    // --port is optional — None means "auto-derive from path_hash, honoring
    // local.jsonc:worktrees.ports pins". See `ship_substrate::home::worktree_port_for`.
    let port = extract_u16_arg(&args, "--port");
    let host = extract_str_arg(&args, "--host").unwrap_or_else(|| "127.0.0.1".to_string());
    let dev = args.iter().any(|a| a == "--dev");
    // Project resolution:
    //   1. `--dev-worktree <path>` (dev-only; routes the daemon into a
    //      per-worktree IPC dir + hash-derived port so multiple Ship-dev
    //      worktrees can run side-by-side without colliding). Hidden
    //      behind the `dev` semantics — not a user-facing flag.
    //   2. Otherwise walk up from CWD looking for `.ship/`, like git
    //      discovers the repo root.
    let workspace = extract_str_arg(&args, "--dev-worktree")
        .map(std::path::PathBuf::from)
        .or_else(discover_workspace);

    let rt = tokio::runtime::Builder::new_multi_thread()
        .enable_all()
        .build()?;
    rt.block_on(shipd::run_network(host, port, dev, workspace))
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
/// root (the dir containing `.ship/`), not `.ship/` itself. Delegates to
/// the runtime's canonical resolver so worktree handling, env overrides,
/// and `~/.ship/` exclusion all behave consistently.
fn discover_workspace() -> Option<std::path::PathBuf> {
    let cwd = std::env::current_dir().ok()?;
    let ship_dir = ship_application::project::resolve_project_ship_dir(&cwd)?;
    // resolver returns the .ship dir; we want its parent (the project root).
    ship_dir.parent().map(std::path::Path::to_path_buf)
}
