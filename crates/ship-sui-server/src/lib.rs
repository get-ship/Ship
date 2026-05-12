pub mod assets;
pub mod bundle;
pub mod render_helpers;
pub mod resolver;
pub mod view;
pub mod view_host;
pub mod view_payload;

#[cfg(test)]
mod view_payload_tests;
#[cfg(test)]
mod mode_tests;

#[cfg(test)]
pub(crate) mod test_lock {
    use std::sync::Mutex;
    static TEST_ENV_LOCK: Mutex<()> = Mutex::new(());
    pub fn lock() -> std::sync::MutexGuard<'static, ()> {
        TEST_ENV_LOCK.lock().unwrap_or_else(|e| e.into_inner())
    }
}

pub use assets::{
    serve_sui_components, serve_sui_dom, serve_sui_island, serve_sui_lucide_icons,
    serve_sui_runtime, serve_sui_sdk_client,
};
pub use view::{
    serve_sui_view_css, serve_sui_view_setup, serve_sui_view_template, ViewSetupQuery,
};
pub use view_host::serve_view_host;
pub use view_payload::{serve_view_payload, serve_view_payload_subpath};

/// Daemon operating mode — controls cache headers, HMR availability, and SSR cache.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum DaemonMode {
    Development,
    Production,
}

impl DaemonMode {
    pub fn from_str(s: &str) -> Self {
        match s {
            "production" => Self::Production,
            _ => Self::Development,
        }
    }
}

/// Extract mode from an optional Axum Extension, defaulting to Development.
pub(crate) fn mode_from_ext(ext: Option<axum::Extension<DaemonMode>>) -> DaemonMode {
    ext.map(|e| e.0).unwrap_or(DaemonMode::Development)
}

/// Cache-control value for framework bundles (runtime/dom/components/islands).
pub(crate) fn asset_cache(mode: DaemonMode) -> &'static str {
    match mode {
        DaemonMode::Development => "no-cache",
        DaemonMode::Production => "public, immutable, max-age=31536000",
    }
}

/// Cache-control value for view artifacts (template/setup/css/payload/host).
pub(crate) fn view_cache(mode: DaemonMode) -> &'static str {
    match mode {
        DaemonMode::Development => "no-cache",
        DaemonMode::Production => "must-revalidate, max-age=0",
    }
}

/// Strong ETag derived from content bytes (fast non-cryptographic hash).
pub(crate) fn compute_etag(content: &[u8]) -> String {
    use std::collections::hash_map::DefaultHasher;
    use std::hash::{Hash, Hasher};
    let mut h = DefaultHasher::new();
    content.hash(&mut h);
    format!("\"{}\"", h.finish())
}
