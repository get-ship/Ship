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
