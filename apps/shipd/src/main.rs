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
    let host = extract_str_arg(&args, "--host").unwrap_or_else(|| "0.0.0.0".to_string());
    let dev = args.iter().any(|a| a == "--dev");

    let rt = tokio::runtime::Builder::new_multi_thread()
        .enable_all()
        .build()?;
    rt.block_on(shipd::run_network(host, port, dev))
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
