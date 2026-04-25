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
        // Stored relative path begins with `blobs/`.
        assert!(data["path"].as_str().unwrap().starts_with("blobs/"));
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

        // Walk the project-scoped artifacts dir and nuke any blob present.
        let artifacts_root = runtime::ShipHome::project_ship_dir(daemon.temp_dir.path()).join("artifacts");
        remove_all_files(&artifacts_root);

        let resp = daemon.get(&format!("/api/v1/artifacts/{id}")).await;
        assert_eq!(resp.status, 404);
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
    async fn delete_removes_blob_and_metadata() {
        let daemon = TestDaemon::new().await;
        let up = daemon
            .post_multipart("/api/v1/artifacts", &upload_parts("gone.txt", "text/plain", b"bye"))
            .await;
        let id = up.body["data"]["id"].as_str().unwrap().to_string();

        let del = daemon.delete(&format!("/api/v1/artifacts/{id}")).await;
        assert_eq!(del.status, 200);

        let get = daemon.get(&format!("/api/v1/artifacts/{id}")).await;
        assert_eq!(get.status, 404);
    }

    #[tokio::test(flavor = "multi_thread")]
    async fn delete_unknown_id_returns_404() {
        let daemon = TestDaemon::new().await;
        let resp = daemon.delete("/api/v1/artifacts/nope").await;
        assert_eq!(resp.status, 404);
    }

    #[tokio::test(flavor = "multi_thread")]
    async fn list_returns_all_and_filters_by_mime() {
        let daemon = TestDaemon::new().await;
        daemon.post_multipart("/api/v1/artifacts", &upload_parts("a.txt", "text/plain", b"a")).await;
        daemon.post_multipart("/api/v1/artifacts", &upload_parts("b.png", "image/png", b"b")).await;
        daemon.post_multipart("/api/v1/artifacts", &upload_parts("c.png", "image/png", b"c")).await;

        let all = daemon.get("/api/v1/artifacts").await;
        assert_eq!(all.status, 200);
        assert_eq!(all.body["count"], 3);

        let pngs = daemon.get("/api/v1/artifacts?mime=image/png").await;
        assert_eq!(pngs.status, 200);
        assert_eq!(pngs.body["count"], 2);
    }

    #[tokio::test(flavor = "multi_thread")]
    async fn upload_links_to_collection_item() {
        let daemon = TestDaemon::new().await;
        let up = daemon
            .post_multipart(
                "/api/v1/artifacts",
                &[
                    ("file", "r.txt", "text/plain", b"payload" as &[u8]),
                    ("collection_id", "", "", b"notes" as &[u8]),
                    ("item_id", "", "", b"note-1" as &[u8]),
                ],
            )
            .await;
        assert_eq!(up.status, 201, "body: {}", up.body);

        let filtered = daemon.get("/api/v1/artifacts?collection_id=notes").await;
        assert_eq!(filtered.status, 200);
        assert_eq!(filtered.body["count"], 1);
        assert_eq!(filtered.body["data"][0]["item_id"], "note-1");
    }

    #[tokio::test(flavor = "multi_thread")]
    async fn blob_stored_under_project_slug() {
        let daemon = TestDaemon::new().await;
        let up = daemon
            .post_multipart("/api/v1/artifacts", &upload_parts("p.txt", "text/plain", b"p"))
            .await;
        assert_eq!(up.status, 201);
        let artifacts_root = runtime::ShipHome::project_ship_dir(daemon.temp_dir.path()).join("artifacts");
        // Exactly one slug dir should exist.
        let slugs: Vec<_> = std::fs::read_dir(&artifacts_root)
            .unwrap()
            .flatten()
            .filter(|e| e.file_type().map(|t| t.is_dir()).unwrap_or(false))
            .collect();
        assert_eq!(slugs.len(), 1, "expected one project slug dir");
        assert!(slugs[0].path().join("blobs").is_dir());
    }

    #[tokio::test(flavor = "multi_thread")]
    async fn full_lifecycle_upload_list_download_delete() {
        let daemon = TestDaemon::new().await;

        let up = daemon
            .post_multipart("/api/v1/artifacts", &upload_parts("life.txt", "text/plain", b"content"))
            .await;
        assert_eq!(up.status, 201);
        let id = up.body["data"]["id"].as_str().unwrap().to_string();

        let list = daemon.get("/api/v1/artifacts").await;
        assert_eq!(list.body["count"], 1);

        let req = Request::builder()
            .method("GET")
            .uri(format!("/api/v1/artifacts/{id}"))
            .body(Body::empty())
            .unwrap();
        let resp = daemon.router.clone().oneshot(req).await.unwrap();
        assert_eq!(resp.status().as_u16(), 200);
        let bytes = resp.into_body().collect().await.unwrap().to_bytes();
        assert_eq!(bytes.as_ref(), b"content");

        let del = daemon.delete(&format!("/api/v1/artifacts/{id}")).await;
        assert_eq!(del.status, 200);

        let gone = daemon.get(&format!("/api/v1/artifacts/{id}")).await;
        assert_eq!(gone.status, 404);

        let empty = daemon.get("/api/v1/artifacts").await;
        assert_eq!(empty.body["count"], 0);
    }
}
