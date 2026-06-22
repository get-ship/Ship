use tracing_subscriber::{EnvFilter, Layer, layer::SubscriberExt, util::SubscriberInitExt};

// jemalloc global allocator. Replaces the glibc malloc_trim(0) FFI the
// bundler was calling per build (the band-aid lived in
// crates/deck/deck-bundler/src/arena.rs, deleted as part of cut 4).
// jemalloc returns freed arenas to the OS by default — no per-build
// trim required. The earlier glibc leak would have manifested in the
// daemon's RSS after sustained build/watch cycles; jemalloc keeps the
// heap trimmed. (musl, macOS, and Windows manage their heaps differently
// and don't strand freed arenas the same way; jemalloc is glibc-specific
// in the sense that it's only needed on glibc, but it runs fine on
// every target — the import is no-op on non-glibc.)
#[global_allocator]
static GLOBAL: tikv_jemallocator::Jemalloc = tikv_jemallocator::Jemalloc;

fn main() -> anyhow::Result<()> {
    // Install the crash hook FIRST — before any work, before tracing,
    // before tokio. If a panic unwinds at any point during startup
    // (the bundle-watcher startup, the HMR firehose registration, an
    // actor spawn), the hook fires and writes a JSON crash file we
    // can post-mortem read. Tracing's async file layer can lose the
    // final entries on a panic; the sync write below is reliable.
    install_panic_hook();
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
    let daemon_mode = deck::DaemonMode::parse(&mode_str);

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
    // `--apps-config <path>` — override the apps.jsonc path. Default is
    // `<worktree>/apps/shipd/apps.jsonc` (set inside `run_network`); the
    // flag is for non-default layouts (a marketing site, docs, a customer
    // shipd install that doesn't ship the Ship apps).
    let apps_config_override = extract_str_arg(&args, "--apps-config")
        .map(std::path::PathBuf::from);

    let rt = tokio::runtime::Builder::new_multi_thread()
        .enable_all()
        .build()?;
    rt.block_on(shipd::run_network(host, port, dev, workspace, daemon_mode, apps_config_override))
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

/// Install a panic hook that writes a JSON crash file before the
/// process exits. The hook fires on every panic, on every thread
/// (the default `std::panic::take_hook` only fires on the
/// panicking thread; we call it from our hook to preserve the
/// abort behavior). The crash file is at
/// `<ship_dir>/logs/shipd.crash.json` — `ship_dir` is the
/// `.ship/machine/daemons/<hash>/` dir for the active daemon.
/// We resolve it lazily on the first panic (the env / cwd may
/// not be set up before the first call).
///
/// The hook also calls `std::process::abort()` after writing —
/// this skips the normal unwind cleanup, which can hang on
/// poisoned mutexes from a tokio-rt-worker panic. The crash
/// file is the artifact we want; let the process die fast.
fn install_panic_hook() {
    use std::io::Write as _;
    let default_hook = std::panic::take_hook();
    std::panic::set_hook(Box::new(move |info| {
        // Resolve ship_dir at panic time. Lazy: env / cwd may not
        // be ready at install_hook time. Best-effort; if any of
        // these calls panic themselves we just print to stderr.
        let payload = serde_json::json!({
            "schema": "shipd.crash/1",
            "ts": chrono::Utc::now().to_rfc3339(),
            "pid": std::process::id(),
            "panicked_at": info.location().map(|l| format!("{}:{}:{}", l.file(), l.line(), l.column())),
            "panic_message": info.payload().downcast_ref::<&'static str>().map(|s| s.to_string())
                .or_else(|| info.payload().downcast_ref::<String>().cloned())
                .unwrap_or_else(|| "<non-string panic payload>".to_string()),
            "backtrace": std::backtrace::Backtrace::force_capture().to_string(),
        });
        // Try to write to shipd.crash.json. If anything goes wrong
        // (no ship_dir, no perm, etc.) fall back to stderr. The
        // crash is the artifact — get it onto disk even if we have
        // to do it badly.
        if let Ok(global_dir) = ship_substrate::project::get_global_dir() {
            let logs_dir = ship_substrate::ShipHome::at(global_dir)
                .machine_dir()
                .join("logs");
            let _ = std::fs::create_dir_all(&logs_dir);
            let crash_path = logs_dir.join("shipd.crash.json");
            // Open with truncate so we always see the latest crash.
            if let Ok(mut f) = std::fs::File::create(&crash_path) {
                let _ = f.write_all(payload.to_string().as_bytes());
                let _ = f.sync_all();
            }
            eprintln!("[shipd-crash] wrote {}", crash_path.display());
        } else {
            eprintln!("[shipd-crash] {}", payload);
        }
        // Run the default hook (prints the panic site + aborts).
        // `unwind_safe` boundary: the hook runs after the unwinder
        // has marked the thread as panicking; we don't try to grab
        // any locks or do anything that could re-panic.
        default_hook(info);
        // Belt and suspenders: if the default hook somehow returns
        // (it usually calls `abort()` itself), abort here so the
        // process dies instead of returning to the panic site.
        std::process::abort();
    }));
}
