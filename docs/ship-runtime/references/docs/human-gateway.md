---
group: Runtime
order: 12
title: Human Gateway
description: Route job.blocked events to a human via pluggable notification adapters. Telegram today; webhook replies re-emit as human.response.
audience: internal
---

# Human Gateway

`crates/core/ship-substrate/src/services/human_gateway.rs` defines `HumanGatewayService`; `ship-platform` re-exports it as `ship_platform::services::human_gateway`, which is the path the daemon imports. A service actor that watches for `job.blocked` events where the payload flags `needs_human: true`, formats a message, and hands it to a pluggable `NotificationAdapter`. Replies come back via webhook (handled in shipd) and are emitted as `human.response` events into the kernel.

Subscribes to `job.` namespace. Earlier framing of this service as a generic "human:" pipeline-step pauser is inaccurate — the current implementation is specifically a `job.blocked` notifier.

## Types

```rust
pub struct OutboundMessage {
    pub text: String,
    pub job_id: Option<String>,
    pub metadata: serde_json::Value,
}

#[async_trait]
pub trait NotificationAdapter: Send + Sync {
    async fn send(&self, msg: &OutboundMessage) -> Result<String>;  // returns provider message id
    fn name(&self) -> &'static str;
}

pub struct HumanGatewayService {
    adapter: Arc<dyn NotificationAdapter>,
    // ...
}
```

## TelegramAdapter

The only built-in adapter today. Env-configured:

- `SHIP_TELEGRAM_TOKEN` — bot token from BotFather.
- `SHIP_TELEGRAM_CHAT_ID` — target chat (user or group).

`TelegramAdapter::from_env()` returns `None` if either variable is unset, which is how the daemon decides whether to spawn the gateway at all.

`send(msg)` POSTs to the Telegram Bot API using `reqwest`. Returns the Telegram message id on success so the daemon can correlate inbound replies.

## Behavior

`HumanGatewayService` implements `ServiceHandler`. The handle path:

1. Filter events — only `job.blocked`.
2. Parse `JobBlockedPayload`. Skip if `needs_human != Some(true)`.
3. Format an outbound text message:

   ```
   🚧 Blocked: {blocker}
   Job: {slug} ({job_id})
   ```

4. Spawn a detached task that calls `adapter.send(&msg)`. The handler returns immediately so the actor's mailbox loop isn't blocked waiting on network I/O.
5. Log on success (message id) or failure.

## Inbound replies

Replies from the operator (e.g. Telegram message to the bot) land on a shipd webhook endpoint. That handler emits a `human.response` event with the reply text and a reference to the original job_id. The gateway itself doesn't process replies — any actor interested in the answer subscribes to `human.*`.

This separation means:

- Outbound and inbound paths are independent.
- Adding a new adapter is a `NotificationAdapter` impl + a webhook route; the event protocol stays the same.
- Replies are in the event log; any downstream that needs "what did the human say" reads events.

## Extending

To add Slack / WhatsApp / email:

1. Implement `NotificationAdapter` in a new module.
2. Add a constructor (ideally `from_env`) that surfaces the config vars clearly.
3. Wire it into the daemon's gateway-spawn logic alongside the Telegram path.
4. Add an inbound webhook route that emits `human.response` with the reply text and correlation metadata.

No changes needed to the gateway itself — adapters are pluggable.

## When it fires

`job.blocked` events come from the orchestrator when a pipeline step can't advance. Today the primary caller is the `human:` gate type in pipelines — a step that waits for explicit approval. The payload sets `needs_human: true` and includes a blocker reason. The gateway forwards; the operator responds; the orchestrator sees `human.response` and unblocks.

This flow is fragile — the orchestrator unblock depends on the `human.response` carrying the right correlation metadata. When adding new `needs_human: true` call sites, make sure the response can be traced back.

## Related

- [Orchestrator](/runtime/orchestrator/) — emits `job.blocked`, consumes `human.response` (where implemented).
- [Events](/runtime/events/) — `job.` namespace, `human.` namespace (not in the reserved list today — open for expansion).
- [Daemon](/runtime/daemon/) — where webhook routes live.
