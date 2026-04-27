use std::time::Duration;
use tokio::net::TcpListener;
use tokio_util::sync::CancellationToken;

async fn start_test_server_with_token(
    token: Option<&str>,
) -> (u16, CancellationToken, Option<tempfile::TempDir>) {
    let listener = TcpListener::bind("127.0.0.1:0").await.unwrap();
    let port = listener.local_addr().unwrap().port();

    // auto_test_global_dir() provides process-scoped DB isolation for test binaries.
    // No need to set SHIP_HOME — it races in parallel tests.
    let tmp_dir: Option<tempfile::TempDir> = None;

    let ct = CancellationToken::new();
    let app = mcp::http::build_mcp_app(token.map(|s| s.to_string()), ct.child_token());

    let ct_clone = ct.clone();
    tokio::spawn(async move {
        axum::serve(listener, app)
            .with_graceful_shutdown(async move { ct_clone.cancelled_owned().await })
            .await
            .unwrap();
    });

    tokio::time::sleep(Duration::from_millis(50)).await;

    (port, ct, tmp_dir)
}

const INIT_BODY: &str = r#"{"jsonrpc":"2.0","id":1,"method":"initialize","params":{"protocolVersion":"2025-03-26","capabilities":{},"clientInfo":{"name":"test","version":"1.0"}}}"#;

#[tokio::test]
async fn http_server_responds_to_initialize() {
    let (port, ct, _tmp) = start_test_server_with_token(None).await;

    let client = reqwest::Client::new();
    let resp = client
        .post(format!("http://127.0.0.1:{port}/mcp"))
        .header("Content-Type", "application/json")
        .header("Accept", "application/json, text/event-stream")
        .body(INIT_BODY)
        .send()
        .await
        .expect("request failed");

    assert_eq!(resp.status(), 200);
    let text = resp.text().await.unwrap();
    assert!(
        text.contains("\"result\""),
        "expected result in response, got: {text}"
    );

    ct.cancel();
}

#[tokio::test]
async fn http_bearer_token_rejected_without_header() {
    let (port, ct, _tmp) = start_test_server_with_token(Some("mysecret")).await;

    let client = reqwest::Client::new();
    let resp = client
        .post(format!("http://127.0.0.1:{port}/mcp"))
        .header("Content-Type", "application/json")
        .header("Accept", "application/json, text/event-stream")
        .body(INIT_BODY)
        .send()
        .await
        .expect("request failed");

    assert_eq!(resp.status(), 401);

    ct.cancel();
}

#[tokio::test]
async fn http_bearer_token_accepted_with_correct_header() {
    let (port, ct, _tmp) = start_test_server_with_token(Some("mysecret")).await;

    let client = reqwest::Client::new();
    let resp = client
        .post(format!("http://127.0.0.1:{port}/mcp"))
        .header("Content-Type", "application/json")
        .header("Accept", "application/json, text/event-stream")
        .header("Authorization", "Bearer mysecret")
        .body(INIT_BODY)
        .send()
        .await
        .expect("request failed");

    assert_eq!(resp.status(), 200);

    ct.cancel();
}
