//! deck-demo — a ship-free deck server.
//!
//! The host contract lives in `lib.rs` ([`deck_demo::deck_app`]); this binary
//! is just the listener. It serves the app under `app/` with NO daemon, NO
//! ship domains, NO `@ship/primitives` — the deck framework standing on its
//! own (effectively the `deck serve` the CLI doesn't have yet).
//!
//! Run: `cargo run -p deck-demo`, then open:
//!   - http://127.0.0.1:7878/_deck/view/hello    (SSR, no controller, no build step)
//!   - http://127.0.0.1:7878/_deck/view/counter  (controller → bundled → hydrated)

#[tokio::main]
async fn main() {
    let app_dir = deck_demo::default_app_dir();
    // The dev server starts the FS watcher, so editing a view hot-reloads.
    let app = deck_demo::deck_dev_server(app_dir.clone()).await;

    let addr = std::env::var("DECK_DEMO_ADDR").unwrap_or_else(|_| "127.0.0.1:7878".to_string());
    let listener = tokio::net::TcpListener::bind(&addr)
        .await
        .unwrap_or_else(|e| panic!("deck-demo: bind {addr}: {e}"));
    println!("deck-demo: serving {} on http://{addr}", app_dir.display());
    println!("deck-demo: try /_deck/view/hello (SSR) or /_deck/view/counter (hydrated)");
    axum::serve(listener, app).await.expect("deck-demo: serve");
}
