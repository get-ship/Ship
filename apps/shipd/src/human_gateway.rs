//! Telegram-backed human gateway service bootstrap.

use std::sync::Arc;

use crate::supervised;

/// Spawn `HumanGatewayService` if Telegram env vars are present.
/// Silently skips if `SHIP_TELEGRAM_TOKEN` or `SHIP_TELEGRAM_CHAT_ID` is absent.
pub async fn spawn_human_gateway(
    kernel: &Arc<tokio::sync::Mutex<runtime::events::KernelRouter>>,
    health_registry: &supervised::HealthRegistry,
) {
    use runtime::events::ActorConfig;
    use runtime::services::human_gateway::{HumanGatewayService, TelegramAdapter};
    use runtime::services::spawn_service;
    use std::sync::Arc as StdArc;

    let adapter = match TelegramAdapter::from_env() {
        Some(a) => a,
        None => {
            tracing::info!("shipd: SHIP_TELEGRAM_TOKEN/CHAT_ID not set — human gateway disabled");
            return;
        }
    };

    let (outbox_tx, mut outbox_rx) =
        tokio::sync::mpsc::unbounded_channel::<runtime::events::EventEnvelope>();

    let config = ActorConfig {
        namespace: "service.human-gateway".to_string(),
        write_namespaces: vec!["human.".to_string()],
        read_namespaces: vec!["job.".to_string()],
        subscribe_namespaces: vec!["job.".to_string()],
    };

    let handler: Box<dyn runtime::services::ServiceHandler> = Box::new(
        HumanGatewayService::new(StdArc::new(adapter), outbox_tx),
    );

    match spawn_service(&mut *kernel.lock().await, "service.human-gateway", config, handler) {
        Ok(_) => tracing::info!("shipd: human gateway service started (telegram)"),
        Err(e) => {
            tracing::warn!("shipd: failed to spawn human gateway: {e}");
            return;
        }
    }

    // Drain outbox → kernel (currently unused, reserved for future adapter replies)
    let kr = kernel.clone();
    supervised::spawn_monitored(health_registry, "human-gateway-outbox", async move {
        let ctx = runtime::events::EmitContext {
            caller_kind: runtime::events::CallerKind::Mcp,
            skill_id: None,
            project_id: None,
            session_id: None,
        };
        while let Some(event) = outbox_rx.recv().await {
            if let Err(e) = kr.lock().await.route(event, &ctx).await {
                tracing::warn!("human-gateway outbox routing error: {e}");
            }
        }
    });
}
