//! Run the bundle in ship-js and collect the first-paint HTML.
//!
//! A `JsRuntime` is `!Send`, so the run happens on a dedicated current-thread
//! VM thread; the caller blocks on it. The production view-host would instead
//! pool `ship_js::deno::ScriptExecutor` threads (one isolate live per thread),
//! awaiting their `Send` reply futures — a fresh isolate per request, pooled
//! across VM threads. This single-shot path is the unit the pool repeats.

use std::sync::Arc;

use anyhow::{anyhow, Result};
use serde_json::Value;
use ship_js::deno::run_module;
use ship_js::Dispatch;

use crate::host::SsrDispatch;
use crate::{RenderOut, SHIP_SDK};

/// Render `bundle` once through `run_module` with `dispatch`, returning the
/// first-paint HTML + the host-dispatch call path.
pub fn render(bundle: &str, dispatch: Arc<SsrDispatch>) -> Result<RenderOut> {
    let bundle_bytes = bundle.len();
    let emits = on_vm_thread(bundle.to_string(), dispatch.clone(), move |b, d| async move {
        let host: Arc<dyn Dispatch> = d;
        run_module(host, SHIP_SDK.to_string(), b)
            .await
            .map_err(|e| anyhow!("run_module: {e:?}"))
    })?;
    let first = emits.first().ok_or_else(|| anyhow!("ssr entry emitted nothing"))?;
    Ok(RenderOut {
        html: first["html"].as_str().unwrap_or_default().to_string(),
        route_params: first["routeParams"].clone(),
        call_path: dispatch.call_path(),
        bundle_bytes,
    })
}

/// Run an async job on a dedicated current-thread VM thread, blocking until done.
fn on_vm_thread<F, Fut>(bundle: String, dispatch: Arc<SsrDispatch>, job: F) -> Result<Vec<Value>>
where
    F: FnOnce(String, Arc<SsrDispatch>) -> Fut + Send + 'static,
    Fut: std::future::Future<Output = Result<Vec<Value>>>,
{
    std::thread::Builder::new()
        .name("deck-ssr-vm".into())
        .spawn(move || {
            let rt = tokio::runtime::Builder::new_current_thread()
                .enable_all()
                .build()
                .expect("current-thread runtime for the SSR VM");
            rt.block_on(job(bundle, dispatch))
        })
        .expect("spawn SSR VM thread")
        .join()
        .map_err(|_| anyhow!("SSR VM thread panicked"))?
}
