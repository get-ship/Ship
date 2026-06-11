//! End-to-end tests for the artifacts REST API.
//!
//! Uses the shared TestDaemon — real router, real SQLite, real kernel.

#[cfg(test)]
mod tests {
    use crate::test_harness::TestDaemon;
    use axum::body::Body;
    use axum::http::Request;
    use http_body_util::BodyExt;
    use tower::ServiceExt;

    fn upload_parts<'a>(name: &'a str, mime: &'a str, bytes: &'a [u8]) -> Vec<(&'a str, &'a str, &'a str, &'a [u8])> {
        vec![("file", name, mime, bytes)]
    }

    fn remove_all_files(dir: &std::path::Path) {
        if !dir.is_dir() {
            return;
        }
        for entry in std::fs::read_dir(dir).unwrap().flatten() {
            let p = entry.path();
            if p.is_dir() {
                remove_all_files(&p);
            } else {
                let _ = std::fs::remove_file(&p);
            }
        }
    }

    #[tokio::test(flavor = "multi_thread")]
    async fn upload_returns_201_with_metadata() {
        let daemon = TestDaemon::new().await;
        let resp = daemon
            .post_multipart("/api/v1/artifacts", &upload_parts("hello.txt", "text/plain", b"hi there"))
            .await;

        assert_eq!(resp.status, 201, "body: {}", resp.body);
        assert_eq!(resp.body["ok"], true);
        let data = &resp.body["data"];
        assert_eq!(data["name"], "hello.txt");
        assert_eq!(data["mime"], "text/plain");
        assert_eq!(data["size"], 8);
        assert!(data["id"].as_str().unwrap().len() >= 8);
        // Stored relative path includes `blobs/<prefix>/<id>`. The blob
        // namespace is the active space id (the event's space partition,
        // matching what the read ctx queries); see
        // `blob_stored_under_project_slug` for the on-disk shape.
        assert!(
            data["path"].as_str().unwrap().contains("blobs/"),
            "expected blobs/ in path, got {}",
            data["path"]
        );
    }

    #[tokio::test(flavor = "multi_thread")]
    async fn upload_missing_file_field_returns_400() {
        let daemon = TestDaemon::new().await;
        let resp = daemon
            .post_multipart(
                "/api/v1/artifacts",
                &[("collection_id", "", "", b"notes")],
            )
            .await;
        assert_eq!(resp.status, 400);
    }

    #[tokio::test(flavor = "multi_thread")]
    async fn download_returns_bytes_and_content_type() {
        let daemon = TestDaemon::new().await;
        let up = daemon
            .post_multipart("/api/v1/artifacts", &upload_parts("b.bin", "application/octet-stream", &[1, 2, 3, 4]))
            .await;
        let id = up.body["data"]["id"].as_str().unwrap().to_string();

        let req = Request::builder()
            .method("GET")
            .uri(format!("/api/v1/artifacts/{id}"))
            .body(Body::empty())
            .unwrap();
        let resp = daemon.router.clone().oneshot(req).await.unwrap();
        assert_eq!(resp.status().as_u16(), 200);
        assert_eq!(
            resp.headers().get("content-type").unwrap().to_str().unwrap(),
            "application/octet-stream"
        );
        let disp = resp.headers().get("content-disposition").unwrap().to_str().unwrap();
        assert!(disp.contains("attachment"));
        assert!(disp.contains("b.bin"));
        let bytes = resp.into_body().collect().await.unwrap().to_bytes();
        assert_eq!(bytes.as_ref(), &[1u8, 2, 3, 4]);
    }

    #[tokio::test(flavor = "multi_thread")]
    async fn download_unknown_id_returns_404() {
        let daemon = TestDaemon::new().await;
        let resp = daemon.get("/api/v1/artifacts/missing-id").await;
        assert_eq!(resp.status, 404);
    }

    #[tokio::test(flavor = "multi_thread")]
    async fn download_missing_blob_returns_404() {
        let daemon = TestDaemon::new().await;
        let up = daemon
            .post_multipart("/api/v1/artifacts", &upload_parts("x.txt", "text/plain", b"x"))
            .await;
        let id = up.body["data"]["id"].as_str().unwrap().to_string();

        // Walk the active-space blob partition and nuke any blob present. Blobs
        // land under `<ship_dir>/spaces/<space_id>/blobs/...` (the active-space
        // partition the read ctx queries), not the old `artifacts/` layout.
        let blob_root = ship_substrate::ShipHome::project_ship_dir(daemon.temp_dir.path()).join("spaces");
        remove_all_files(&blob_root);

        let resp = daemon.get(&format!("/api/v1/artifacts/{id}")).await;
        assert_eq!(resp.status, 404);
    }






    #[tokio::test(flavor = "multi_thread")]
    async fn blob_stored_under_active_space_partition() {
        let daemon = TestDaemon::new().await;
        let up = daemon
            .post_multipart("/api/v1/artifacts", &upload_parts("p.txt", "text/plain", b"p"))
            .await;
        assert_eq!(up.status, 201);
        // Blobs land under `<ship_dir>/spaces/<space_id>/blobs/...` — one
        // partition dir per active space (the test fixture binds `test-space`).
        let spaces_root = ship_substrate::ShipHome::project_ship_dir(daemon.temp_dir.path()).join("spaces");
        let spaces: Vec<_> = std::fs::read_dir(&spaces_root)
            .unwrap()
            .flatten()
            .filter(|e| e.file_type().map(|t| t.is_dir()).unwrap_or(false))
            .collect();
        assert_eq!(spaces.len(), 1, "expected one active-space partition dir");
        assert!(spaces[0].path().join("blobs").is_dir());
    }

}
