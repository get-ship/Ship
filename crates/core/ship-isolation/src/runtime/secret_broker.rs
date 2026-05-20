//! Host-side credential broker for the v1 `Events.Stream`.
//!
//! A sandboxed actor that needs a secret emits `SecretRequest { name }`
//! over the Events stream; the host resolves it from a daemon-side source
//! and replies `SecretReply { ok, value, error }`, echoing the envelope's
//! `request_id`. The raw value never enters the sandbox image or the
//! actor's env — it travels only in the reply, in memory. See
//! `docs/threat-model.md` (T-CRED).
//!
//! This module owns the *mechanism* (request → reply mapping). The secret
//! *source* is injected via [`SecretResolver`] so this crate stays
//! decoupled from `ship-platform`'s credential store; shipd supplies the
//! concrete resolver.

use crate::sidecar::proto::*;

/// Resolves a logical secret name to its value. Implemented on the daemon
/// side (backed by the platform credential/account store) and injected
/// into the broker loop.
#[async_trait::async_trait]
pub trait SecretResolver: Send + Sync {
    /// Resolve `name` (e.g. `"anthropic.api_key"`). `Ok(value)` becomes a
    /// successful reply; `Err(reason)` becomes `SecretReply.error` with no
    /// value — a failed lookup never carries a partial/placeholder value.
    async fn resolve(&self, name: &str) -> Result<String, String>;
}

/// Build the `SecretReply` envelope for an incoming `SecretRequest`,
/// echoing `request_id` so the sidecar can correlate it. On failure the
/// reply carries `ok = false` and an error, never a value.
pub async fn secret_reply(
    req: &SecretRequest,
    request_id: &str,
    resolver: &dyn SecretResolver,
) -> Envelope {
    let mut reply = SecretReply::default();
    match resolver.resolve(&req.name).await {
        Ok(value) => {
            reply.ok = true;
            reply.value = value;
        }
        Err(error) => {
            reply.ok = false;
            reply.error = error;
        }
    }
    let mut env = Envelope::default();
    env.request_id = request_id.to_string();
    env.payload = Some(envelope::Payload::SecretReply(Box::new(reply)));
    env
}

/// Run the host-side broker loop: read inbound envelopes from the sandbox,
/// answer the request variants we handle, and push replies to `outbound`.
///
/// Returns when `inbound` closes (sandbox gone) or `outbound` is dropped.
/// Only `SecretRequest` is handled here today; `ProcessOutput`/`Exited` are
/// serviced on the AttachPty path, and permission/setting/hook routing
/// lands as those brokers come online (they share this loop).
///
/// The real Events.Stream is adapted into `inbound`/`outbound` channels by
/// the integration layer; keeping the loop channel-shaped makes the policy
/// testable without standing up ConnectRPC.
pub async fn run_broker(
    mut inbound: tokio::sync::mpsc::Receiver<Envelope>,
    outbound: tokio::sync::mpsc::Sender<Envelope>,
    resolver: std::sync::Arc<dyn SecretResolver>,
) {
    while let Some(env) = inbound.recv().await {
        let request_id = env.request_id.clone();
        match env.payload {
            Some(envelope::Payload::SecretRequest(req)) => {
                let reply = secret_reply(&req, &request_id, resolver.as_ref()).await;
                if outbound.send(reply).await.is_err() {
                    break; // outbound closed — sandbox teardown
                }
            }
            // Other variants are handled elsewhere or not yet wired.
            _ => {}
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    struct MockResolver;

    #[async_trait::async_trait]
    impl SecretResolver for MockResolver {
        async fn resolve(&self, name: &str) -> Result<String, String> {
            match name {
                "anthropic.api_key" => Ok("sk-test-123".into()),
                other => Err(format!("unknown secret: {other}")),
            }
        }
    }

    #[tokio::test]
    async fn resolves_known_secret_into_reply() {
        let mut req = SecretRequest::default();
        req.name = "anthropic.api_key".into();
        let env = secret_reply(&req, "rid-1", &MockResolver).await;
        assert_eq!(env.request_id, "rid-1");
        match env.payload {
            Some(envelope::Payload::SecretReply(r)) => {
                assert!(r.ok);
                assert_eq!(r.value, "sk-test-123");
                assert!(r.error.is_empty());
            }
            other => panic!("expected SecretReply payload, got {other:?}"),
        }
    }

    #[tokio::test]
    async fn unknown_secret_yields_error_reply_without_value() {
        let mut req = SecretRequest::default();
        req.name = "nope.token".into();
        let env = secret_reply(&req, "rid-2", &MockResolver).await;
        match env.payload {
            Some(envelope::Payload::SecretReply(r)) => {
                assert!(!r.ok);
                // A failed lookup must never carry a value.
                assert!(r.value.is_empty());
                assert!(r.error.contains("unknown secret"));
            }
            other => panic!("expected SecretReply payload, got {other:?}"),
        }
    }

    #[tokio::test]
    async fn broker_loop_answers_secret_request_then_exits_on_close() {
        let (in_tx, in_rx) = tokio::sync::mpsc::channel(4);
        let (out_tx, mut out_rx) = tokio::sync::mpsc::channel(4);

        let mut req_env = Envelope::default();
        req_env.request_id = "rid-9".into();
        let mut sr = SecretRequest::default();
        sr.name = "anthropic.api_key".into();
        req_env.payload = Some(envelope::Payload::SecretRequest(Box::new(sr)));
        in_tx.send(req_env).await.unwrap();
        // A non-request envelope must be ignored, not answered.
        let mut noise = Envelope::default();
        noise.payload = Some(envelope::Payload::LogProgress(Box::new(LogProgress::default())));
        in_tx.send(noise).await.unwrap();
        drop(in_tx); // close inbound → loop should drain and return

        run_broker(in_rx, out_tx, std::sync::Arc::new(MockResolver)).await;

        let reply = out_rx.recv().await.expect("one reply");
        assert_eq!(reply.request_id, "rid-9");
        match reply.payload {
            Some(envelope::Payload::SecretReply(r)) => {
                assert!(r.ok);
                assert_eq!(r.value, "sk-test-123");
            }
            other => panic!("expected SecretReply, got {other:?}"),
        }
        // Only the request got a reply; the LogProgress produced nothing.
        assert!(out_rx.recv().await.is_none());
    }
}
