//! `PipelinesService` policy + gate conformance tests.
//!
//! The `impl PipelinesService for OpsImpl` is deleted — `PipelinesWire`
//! (ship-pipeline-runtime) is now the sole handler, mounted via
//! `PipelinesServiceExt::register` in `ops_rpc.rs`.

//! `pipelines_focus_round_trip` proves the `Focus` rpc reaches the focus
//! builder through the read_fn closure end-to-end: a seeded pipeline file
//! resolves to a non-null `FocusCtx` json, a missing one lifts to
//! `ConnectError::not_found` (the live stream surfaces that as an error
//! envelope to the controller — never a null `ok` value).
#[cfg(test)]
mod tests {
    use connectrpc::client::{ClientConfig, HttpClient};
    use ship_ops_rpc::{PipelineFocusRequest, PipelinesServiceClient};
    use ship_substrate::events::CallerKind;
    use crate::auth::{AuthContext, Scope};
    use crate::ops_rpc::{policy_for, router};

    #[test]
    fn pipelines_list_is_c0_read() {
        let p = policy_for("ship.ops.v1.PipelinesService/List")
            .expect("PipelinesService/List must be in the descriptor");
        assert_eq!(p.capability, "ship.pipelines.read");
        assert_eq!(p.blast_radius, ship_ops_rpc::BlastRadius::C0Read);
    }
    #[test]
    fn update_status_is_c1_runtime_write() {
        let p = policy_for("ship.ops.v1.PipelinesService/UpdateStatus")
            .expect("PipelinesService/UpdateStatus must be in the descriptor");
        assert_eq!(p.capability, "ship.runtime.write");
        assert_eq!(p.blast_radius, ship_ops_rpc::BlastRadius::C1Space);
    }
    #[test]
    fn pipelines_put_is_c1_config_write() {
        let p = policy_for("ship.ops.v1.PipelinesService/Put")
            .expect("PipelinesService/Put must be in the descriptor");
        assert_eq!(p.capability, "ship.pipelines.write");
        assert_eq!(p.blast_radius, ship_ops_rpc::BlastRadius::C1Space);
    }
    #[test]
    fn pipelines_focus_is_c0_read() {
        let p = policy_for("ship.ops.v1.PipelinesService/Focus")
            .expect("PipelinesService/Focus must be in the descriptor");
        assert_eq!(p.capability, "ship.pipelines.read");
        assert_eq!(p.blast_radius, ship_ops_rpc::BlastRadius::C0Read);
        let live = policy_for("ship.ops.v1.PipelinesService/FocusLive")
            .expect("PipelinesService/FocusLive must be in the descriptor");
        assert_eq!(live.capability, "ship.pipelines.read");
        assert_eq!(live.blast_radius, ship_ops_rpc::BlastRadius::C0Read);
    }
    fn operator_auth() -> AuthContext {
        AuthContext {
            caller_kind: CallerKind::Cli,
            agent_id: None,
            workspace_id: None,
            space_id: None,
            session_id: None,
            scopes: vec![Scope::Full],
            grants: vec![],
            operator_cred: true,
        }
    }
    /// Stand up the full router with a seeded `pipelines/<name>.jsonc`
    /// file in the harness temp ship_dir. Returns the client + the harness
    /// (held alive so the temp dir backs the daemon through the call).
    async fn serve_with_pipeline(
        name: &str,
        body: &str,
    ) -> (PipelinesServiceClient<HttpClient>, crate::test_harness::TestDaemon) {
        let harness = crate::test_harness::TestDaemon::new().await;
        let pipelines_dir = harness.state.ship_dir().join("pipelines");
        std::fs::create_dir_all(&pipelines_dir).unwrap();
        std::fs::write(pipelines_dir.join(format!("{name}.jsonc")), body).unwrap();
        let listener = tokio::net::TcpListener::bind("127.0.0.1:0").await.unwrap();
        let addr = listener.local_addr().unwrap();
        let app = router(harness.state.clone()).layer(axum::middleware::from_fn(
            |mut req: axum::extract::Request, next: axum::middleware::Next| async move {
                req.extensions_mut().insert(operator_auth());
                next.run(req).await
            },
        ));
        tokio::spawn(async move { axum::serve(listener, app).await.unwrap() });
        let uri: http::Uri = format!("http://{addr}").parse().unwrap();
        let client = PipelinesServiceClient::new(HttpClient::plaintext(), ClientConfig::new(uri));
        (client, harness)
    }
    #[tokio::test(flavor = "multi_thread", worker_threads = 2)]
    async fn pipelines_focus_round_trip_resolves_seeded_pipeline() {
        let body = r#"{
            "name": "demo",
            "description": "tiny demo for the focus rpc",
            "auto_merge": true,
            "steps": [
                { "id": "build", "run": "echo build" },
                { "id": "review", "agent": "gatekeeper" }
            ]
        }"#;
        let (client, harness) = serve_with_pipeline("demo", body).await;
        // Hold the harness so its temp dir is not dropped mid-call.
        std::mem::forget(harness);
        let resp = client
            .focus(PipelineFocusRequest {
                name: "demo".to_string(),
                ..Default::default()
            })
            .await
            .expect("pipelines.focus on a seeded pipeline should succeed")
            .view()
            .clone();
        // The wire returns the focus builder's `FocusCtx` as JSON. Parse
        // the inner json string and assert the load-bearing fields are
        // present and well-formed — the full view shape is asserted by
        // the studio controller's type contract.
        let v: serde_json::Value = serde_json::from_str(&resp.json)
            .expect("focus json is valid JSON");
        assert_eq!(v["format"], "jsonc", "format name from the file extension");
        assert_eq!(v["def"]["name"], "demo");
        assert_eq!(v["def"]["step_count"], 2);
        assert_eq!(v["doc"]["auto_merge"], true);
        let steps = v["doc"]["steps"].as_array().expect("steps is an array");
        assert_eq!(steps.len(), 2);
        assert_eq!(steps[0]["id"], "build");
        assert_eq!(steps[0]["run"], "echo build");
        assert_eq!(steps[1]["agent"], "gatekeeper");
        assert!(
            v["providers"].is_array(),
            "providers is always serialized (may be empty)"
        );
    }
    #[tokio::test(flavor = "multi_thread", worker_threads = 2)]
    async fn pipelines_focus_unknown_name_raises_not_found() {
        let (client, harness) = serve_with_pipeline("demo", "{}").await;
        std::mem::forget(harness);
        let err = client
            .focus(PipelineFocusRequest {
                name: "no-such-pipeline".to_string(),
                ..Default::default()
            })
            .await
            .expect_err("pipelines.focus on an unknown name must error, not return null");
        // The wire lifts the builder's `None` into `ConnectError::not_found`
        // with the missing name in the message. Anything else means the
        // error path is wrong.
        let msg = format!("{err:?}");
        assert!(
            msg.contains("NotFound"),
            "unknown pipeline should surface a NotFound error; got: {msg}"
        );
        assert!(
            msg.contains("no-such-pipeline"),
            "error message should name the missing pipeline; got: {msg}"
        );
    }
}