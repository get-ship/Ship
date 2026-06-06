//! The `!Send` bridge — code-mode inside the rig loop.
//!
//! deno_core's `JsRuntime` is `!Send`, but rig's `Tool::call` must return a
//! `Send` future. So the JsRuntime never crosses a thread boundary: it lives on
//! a DEDICATED current-thread executor thread. The loop tool sends a job over a
//! channel and awaits a `Send` result. The tool's future holds only channel
//! ends (Send-clean); all V8 work stays on the one VM thread.
//!
//! Concurrency: jobs `spawn_local` onto the thread's `LocalSet`, so many scripts
//! interleave cooperatively on the single thread (each gets its own JsRuntime via
//! `run_module`). That's the same single-thread-multiplex model as the isolate
//! pool — this is one such thread.

use std::sync::Arc;

use serde_json::Value;
use tokio::sync::{mpsc, oneshot};

use crate::auth::AuthContext;
use crate::gateway::Gateway;

struct ScriptJob {
    src: String,
    grants: Vec<String>,
    auth: AuthContext,
    reply: oneshot::Sender<anyhow::Result<Vec<Value>>>,
}

/// Handle to the VM thread. Cloneable-cheap (just a channel sender). Dropping the
/// last handle ends the recv loop and the thread exits cleanly.
pub struct ScriptExecutor {
    tx: mpsc::UnboundedSender<ScriptJob>,
}

impl ScriptExecutor {
    /// Start the dedicated VM thread. `gateway` is moved onto it (Send+Sync) and
    /// every script's ops route through it — so the gate still runs per op.
    pub fn spawn(gateway: Arc<dyn Gateway>) -> Self {
        let (tx, mut rx) = mpsc::unbounded_channel::<ScriptJob>();
        std::thread::Builder::new()
            .name("ship-script-vm".into())
            .spawn(move || {
                let rt = tokio::runtime::Builder::new_current_thread()
                    .enable_all()
                    .build()
                    .expect("current-thread runtime for the script VM");
                let local = tokio::task::LocalSet::new();
                local.block_on(&rt, async move {
                    while let Some(job) = rx.recv().await {
                        let gw = gateway.clone();
                        // The !Send JsRuntime created inside run_module stays on
                        // THIS thread — never sent anywhere.
                        tokio::task::spawn_local(async move {
                            let res =
                                crate::runtime::run_module(gw, job.auth, job.grants, job.src).await;
                            let _ = job.reply.send(res);
                        });
                    }
                });
            })
            .expect("spawn script VM thread");
        Self { tx }
    }

    /// Run a script on the VM thread. The returned future is `Send` (it only
    /// awaits a oneshot), so it composes inside rig's loop.
    pub async fn run(
        &self,
        src: String,
        grants: Vec<String>,
        auth: AuthContext,
    ) -> anyhow::Result<Vec<Value>> {
        let (reply, rx) = oneshot::channel();
        self.tx
            .send(ScriptJob { src, grants, auth, reply })
            .map_err(|_| anyhow::anyhow!("script executor stopped"))?;
        rx.await
            .map_err(|_| anyhow::anyhow!("script executor dropped the reply"))?
    }
}
