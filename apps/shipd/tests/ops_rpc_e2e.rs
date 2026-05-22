//! End-to-end round-trip for the ConnectRPC op surface: serve the real
//! `ops_rpc::router()` on an ephemeral port and call it through the generated
//! `OpsServiceClient`. Proves the full vertical — proto → server trait → mount
//! → wire → typed client — not just the pieces.

use connectrpc::client::{ClientConfig, HttpClient};
use ship_ops_rpc::{MetaRequest, OpsServiceClient};

#[tokio::test]
async fn meta_round_trip_through_generated_client() {
    // Serve the op surface on a loopback ephemeral port.
    let listener = tokio::net::TcpListener::bind("127.0.0.1:0").await.unwrap();
    let addr = listener.local_addr().unwrap();
    tokio::spawn(async move {
        axum::serve(listener, shipd::ops_rpc::router()).await.unwrap();
    });

    let uri: http::Uri = format!("http://{addr}").parse().unwrap();
    let client = OpsServiceClient::new(HttpClient::plaintext(), ClientConfig::new(uri));

    let resp = client
        .meta(MetaRequest::default())
        .await
        .expect("Meta RPC should succeed over the wire");

    // The daemon answers with its own crate version (see ops_rpc::OpsImpl).
    // `.view()` derefs to the zero-copy response view; `version` is a `&str`.
    assert_eq!(resp.view().version, env!("CARGO_PKG_VERSION"));
}
