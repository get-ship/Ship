//! End-to-end behavioral tests for the live-data multiplex through the
//! generated `WindowSubscriptionServiceClient` (ADR 0001, step 3). Serves the
//! real `ops_rpc::router` on an ephemeral port with an injected `AuthContext`
//! (mirroring `ops_rpc::tests::meta_round_trip`), then drives the wire:
//! Open → Subscribe → Stream. The load-bearing property is that ONE stream
//! demuxes N subscriptions by `subscription_id`.

use std::collections::HashSet;

use connectrpc::client::{ClientConfig, HttpClient};
use ship_ops_rpc::{
    OpenRequest, StreamRequest, SubscribeRequest, WindowSubscriptionServiceClient,
};
use ship_substrate::events::CallerKind;

use crate::auth::{AuthContext, Scope};

fn test_auth() -> AuthContext {
    AuthContext {
        caller_kind: CallerKind::Cli,
        agent_id: None,
        workspace_id: None,
        space_id: None,
        session_id: None,
        scopes: vec![Scope::Full],
        grants: vec![],
        operator_cred: false,
    }
}

/// Stand up the multiplex over a real router + injected auth, returning a
/// client bound to the ephemeral address.
async fn serve() -> WindowSubscriptionServiceClient<HttpClient> {
    let harness = crate::test_harness::TestDaemon::new().await;
    let listener = tokio::net::TcpListener::bind("127.0.0.1:0").await.unwrap();
    let addr = listener.local_addr().unwrap();
    let app = crate::ops_rpc::router(harness.state.clone()).layer(axum::middleware::from_fn(
        |mut req: axum::extract::Request, next: axum::middleware::Next| async move {
            req.extensions_mut().insert(test_auth());
            next.run(req).await
        },
    ));
    tokio::spawn(async move { axum::serve(listener, app).await.unwrap() });
    // Keep the harness alive for the duration of the process (its temp dir backs
    // the daemon). Leaking it is fine in a test.
    std::mem::forget(harness);
    let uri: http::Uri = format!("http://{addr}").parse().unwrap();
    WindowSubscriptionServiceClient::new(HttpClient::plaintext(), ClientConfig::new(uri))
}

#[tokio::test(flavor = "multi_thread", worker_threads = 2)]
async fn open_mints_a_window_and_advertises_heartbeat() {
    let client = serve().await;
    let opened = client.open(OpenRequest::default()).await.unwrap().into_owned();
    assert!(opened.window_id.starts_with("win."), "Open mints win.<ulid>; got {}", opened.window_id);
    assert!(!opened.session_id.is_empty(), "Open returns a session token");
    assert!(opened.heartbeat_ms > 0, "Open advertises a heartbeat interval");
}

#[tokio::test(flavor = "multi_thread", worker_threads = 2)]
async fn subscribe_to_unopened_window_is_rejected() {
    let client = serve().await;
    let err = client
        .subscribe(SubscribeRequest {
            window_id: "win.nope".into(),
            domain: "pipelines".into(),
            op: "summaries".into(),
            ..Default::default()
        })
        .await
        .expect_err("subscribe to an unopened window must fail");
    // Surfaced as not_found (the registry returns None for an unknown window).
    assert!(
        format!("{err}").to_lowercase().contains("window"),
        "error should name the missing window; got: {err}"
    );
}

#[tokio::test(flavor = "multi_thread", worker_threads = 2)]
async fn unknown_op_has_no_resolver() {
    let client = serve().await;
    let opened = client.open(OpenRequest::default()).await.unwrap().into_owned();
    let err = client
        .subscribe(SubscribeRequest {
            window_id: opened.window_id,
            domain: "nonsense".into(),
            op: "void".into(),
            ..Default::default()
        })
        .await
        .expect_err("an op with no resolver must be refused, not silently empty");
    assert!(
        format!("{err}").contains("nonsense.void"),
        "error should name the unrouted op; got: {err}"
    );
}

/// The load-bearing multiplex property: a single `Stream` carries the initial
/// value for EVERY subscription in the window, each tagged by its own
/// `subscription_id`. Two subscriptions ⇒ two distinct tagged data events on
/// one stream — the per-`<data live>` socket fan-out collapsed to one.
#[tokio::test(flavor = "multi_thread", worker_threads = 2)]
async fn one_stream_demuxes_two_subscriptions_by_id() {
    let client = serve().await;
    let window_id = client.open(OpenRequest::default()).await.unwrap().into_owned().window_id;

    let sub_a = client
        .subscribe(SubscribeRequest {
            window_id: window_id.clone(),
            domain: "pipelines".into(),
            op: "summaries".into(),
            ..Default::default()
        })
        .await
        .unwrap()
        .into_owned();
    assert!(sub_a.subscription_id.starts_with("sub."), "Subscribe mints sub.<ulid>");
    assert!(!sub_a.initial_value.is_empty(), "ack carries the pre-resolved value");

    let sub_b = client
        .subscribe(SubscribeRequest {
            window_id: window_id.clone(),
            domain: "git".into(),
            op: "overview".into(),
            ..Default::default()
        })
        .await
        .unwrap()
        .into_owned();

    let want: HashSet<String> =
        [sub_a.subscription_id.clone(), sub_b.subscription_id.clone()].into_iter().collect();

    let mut stream = client
        .stream(StreamRequest { window_id, ..Default::default() })
        .await
        .unwrap();

    // The first two events are the initial data frames — one per subscription,
    // emitted before the heartbeat tick. Collect their subscription_ids.
    let mut got: HashSet<String> = HashSet::new();
    for _ in 0..2 {
        let ev = stream
            .message()
            .await
            .unwrap()
            .expect("multiplex stream must emit the initial data events");
        assert!(
            !ev.subscription_id.is_empty(),
            "an initial data frame is tagged by subscription_id (empty = heartbeat)"
        );
        got.insert(ev.subscription_id.to_string());
    }

    assert_eq!(got, want, "one stream demuxed both subscriptions by id");
}
