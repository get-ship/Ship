use tracing_subscriber::{EnvFilter, Layer, layer::SubscriberExt, util::SubscriberInitExt};

fn main() -> anyhow::Result<()> {
    let global_dir = runtime::project::get_global_dir()?;

    // Rotate log file if >10 MB.
    let log_path = global_dir.join("shipd.log");
    if let Ok(meta) = std::fs::metadata(&log_path) {
        if meta.len() > 10 * 1024 * 1024 {
            let _ = std::fs::rename(&log_path, global_dir.join("shipd.log.old"));
        }
    }

    // File appender — JSON-structured logs for crash forensics.
    let file_appender = tracing_appender::rolling::never(&global_dir, "shipd.log");
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
    let port = extract_u16_arg(&args, "--port").unwrap_or(9315);
    let host = extract_str_arg(&args, "--host").unwrap_or_else(|| "127.0.0.1".to_string());
    let dev = args.iter().any(|a| a == "--dev");
    let with_ui = args.iter().any(|a| a == "--with-ui");
    // Workspace resolution: explicit flag wins; otherwise walk up from CWD
    // looking for `.ship/` like git discovers the repo root. Only fall back
    // to global-only when the daemon genuinely isn't inside a project.
    let workspace = extract_str_arg(&args, "--workspace")
        .map(std::path::PathBuf::from)
        .or_else(discover_workspace);

    let rt = tokio::runtime::Builder::new_multi_thread()
        .enable_all()
        .build()?;
    rt.block_on(shipd::run_network(host, port, dev, with_ui, workspace))
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
/// root (the dir containing `.ship/`), not `.ship/` itself. Stops at the
/// filesystem root or the home directory — the latter prevents `~/.ship/`
/// from being auto-selected as a "project" for global contexts.
fn discover_workspace() -> Option<std::path::PathBuf> {
    let home = dirs::home_dir();
    let cwd = std::env::current_dir().ok()?;
    let mut cur = cwd.as_path();
    loop {
        if Some(cur) == home.as_deref() {
            return None;
        }
        if runtime::ShipHome::project_ship_dir(&cur).is_dir() {
            return Some(cur.to_path_buf());
        }
        match cur.parent() {
            Some(p) => cur = p,
            None => return None,
        }
    }
}
