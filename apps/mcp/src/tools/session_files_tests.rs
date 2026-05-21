use super::*;

#[test]
fn validate_path_rejects_traversal() {
    assert!(validate_path("../etc/passwd").is_err());
    assert!(validate_path("foo/../../bar").is_err());
    assert!(validate_path("..").is_err());
}

#[test]
fn validate_path_rejects_absolute() {
    assert!(validate_path("/etc/passwd").is_err());
    assert!(validate_path("\\Windows\\System32").is_err());
}

#[test]
fn validate_path_rejects_empty() {
    assert!(validate_path("").is_err());
}

#[test]
fn validate_path_accepts_valid() {
    assert!(validate_path("canvas.html").is_ok());
    assert!(validate_path("screenshots/issue-001.png").is_ok());
    assert!(validate_path("deep/nested/file.json").is_ok());
}

#[test]
fn file_type_classification() {
    assert_eq!(file_type_from_extension("test.html"), "html");
    assert_eq!(file_type_from_extension("test.htm"), "html");
    assert_eq!(file_type_from_extension("test.md"), "md");
    assert_eq!(file_type_from_extension("test.json"), "json");
    assert_eq!(file_type_from_extension("test.png"), "image");
    assert_eq!(file_type_from_extension("test.jpg"), "image");
    assert_eq!(file_type_from_extension("test.jpeg"), "image");
    assert_eq!(file_type_from_extension("test.gif"), "image");
    assert_eq!(file_type_from_extension("test.webp"), "image");
    assert_eq!(file_type_from_extension("test.rs"), "other");
    assert_eq!(file_type_from_extension("noext"), "other");
}

#[test]
fn list_empty_session_dir() {
    let tmp = tempfile::tempdir().unwrap();
    let result = list_session_files(tmp.path());
    assert_eq!(result, "[]");
}

#[test]
fn list_with_files() {
    let tmp = tempfile::tempdir().unwrap();
    let session = tmp.path().join(SESSION_DIR);
    std::fs::create_dir_all(session.join("screenshots")).unwrap();
    std::fs::write(session.join("canvas.html"), "<html></html>").unwrap();
    std::fs::write(session.join("screenshots/s.png"), &[0u8; 10]).unwrap();

    let result = list_session_files(tmp.path());
    let entries: Vec<SessionFileEntry> = serde_json::from_str(&result).unwrap();
    assert_eq!(entries.len(), 2);

    let html = entries.iter().find(|e| e.path == "canvas.html").unwrap();
    assert_eq!(html.file_type, "html");

    let img = entries
        .iter()
        .find(|e| e.path == "screenshots/s.png")
        .unwrap();
    assert_eq!(img.file_type, "image");
    assert_eq!(img.size, 10);
}

#[test]
fn read_text_file() {
    let tmp = tempfile::tempdir().unwrap();
    let session = tmp.path().join(SESSION_DIR);
    std::fs::create_dir_all(&session).unwrap();
    std::fs::write(session.join("notes.md"), "# Notes").unwrap();

    let req = ReadSessionFileRequest {
        path: "notes.md".into(),
    };
    let result = read_session_file(tmp.path(), req);
    assert_eq!(result, "# Notes");
}

#[test]
fn read_binary_file_returns_data_uri() {
    let tmp = tempfile::tempdir().unwrap();
    let session = tmp.path().join(SESSION_DIR);
    std::fs::create_dir_all(&session).unwrap();
    let bytes = vec![0x89, 0x50, 0x4E, 0x47]; // PNG magic bytes
    std::fs::write(session.join("img.png"), &bytes).unwrap();

    let req = ReadSessionFileRequest {
        path: "img.png".into(),
    };
    let result = read_session_file(tmp.path(), req);
    assert!(result.starts_with("data:image/png;base64,"));
}

#[test]
fn read_nonexistent_file_returns_error() {
    let tmp = tempfile::tempdir().unwrap();
    let req = ReadSessionFileRequest {
        path: "missing.txt".into(),
    };
    let result = read_session_file(tmp.path(), req);
    assert!(result.starts_with("Error:"));
}

#[test]
fn read_rejects_path_traversal() {
    let tmp = tempfile::tempdir().unwrap();
    let req = ReadSessionFileRequest {
        path: "../secret.txt".into(),
    };
    let result = read_session_file(tmp.path(), req);
    assert!(result.starts_with("Error:"));
    assert!(result.contains("traversal"));
}
