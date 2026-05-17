//! In-process stub HTTP server for `tests_opencode_session`. Parses
//! HTTP/1.1 enough to route the session's four endpoint families and
//! records captured requests for assertion.

use std::sync::Arc;
use std::time::Duration;

use tokio::io::{AsyncBufReadExt, AsyncReadExt, AsyncWriteExt, BufReader};
use tokio::net::TcpListener;
use tokio::sync::Mutex;

#[derive(Debug, Clone)]
pub(super) struct CapturedRequest {
    pub method: String,
    pub path: String,
    pub body: String,
}

/// Scripted server response. `Sse` delays frame emission so tests can
/// register subscribers before events arrive.
#[derive(Clone)]
pub(super) enum Response {
    Json(serde_json::Value),
    Sse(Vec<serde_json::Value>),
    Empty,
}

pub(super) struct StubServer {
    pub base_url: String,
    pub captures: Arc<Mutex<Vec<CapturedRequest>>>,
}

impl StubServer {
    /// Spawn on an ephemeral port. Unmatched requests yield 404.
    pub async fn spawn<F>(handler: F) -> Self
    where
        F: Fn(&str, &str) -> Option<Response> + Send + Sync + 'static,
    {
        let listener = TcpListener::bind("127.0.0.1:0").await.expect("bind");
        let port = listener.local_addr().expect("addr").port();
        let base_url = format!("http://127.0.0.1:{port}");
        let captures: Arc<Mutex<Vec<CapturedRequest>>> = Arc::new(Mutex::new(Vec::new()));
        let handler = Arc::new(handler);

        let caps = captures.clone();
        tokio::spawn(async move {
            loop {
                let Ok((socket, _)) = listener.accept().await else {
                    break;
                };
                let caps = caps.clone();
                let handler = handler.clone();
                tokio::spawn(async move {
                    let _ = handle_connection(socket, caps, handler).await;
                });
            }
        });

        // Yield so the listener is ready on localhost.
        tokio::time::sleep(Duration::from_millis(10)).await;
        Self { base_url, captures }
    }

    pub async fn captures(&self) -> Vec<CapturedRequest> {
        self.captures.lock().await.clone()
    }
}

async fn handle_connection(
    mut socket: tokio::net::TcpStream,
    captures: Arc<Mutex<Vec<CapturedRequest>>>,
    handler: Arc<dyn Fn(&str, &str) -> Option<Response> + Send + Sync>,
) -> std::io::Result<()> {
    let (read_half, mut write_half) = socket.split();
    let mut reader = BufReader::new(read_half);

    let mut request_line = String::new();
    if reader.read_line(&mut request_line).await? == 0 {
        return Ok(());
    }
    let mut parts = request_line.split_whitespace();
    let method = parts.next().unwrap_or("").to_string();
    let path = parts.next().unwrap_or("").to_string();

    // Read headers until blank line, capturing Content-Length.
    let mut content_length: usize = 0;
    loop {
        let mut line = String::new();
        let n = reader.read_line(&mut line).await?;
        if n == 0 || line == "\r\n" || line == "\n" {
            break;
        }
        let lower = line.to_ascii_lowercase();
        if let Some(rest) = lower.strip_prefix("content-length:") {
            content_length = rest.trim().parse().unwrap_or(0);
        }
    }

    let mut body = vec![0u8; content_length];
    if content_length > 0 {
        reader.read_exact(&mut body).await?;
    }
    let body_str = String::from_utf8_lossy(&body).to_string();

    captures.lock().await.push(CapturedRequest {
        method: method.clone(),
        path: path.clone(),
        body: body_str,
    });

    match handler(&method, &path) {
        Some(Response::Json(value)) => {
            let payload = serde_json::to_string(&value).unwrap();
            let head = format!(
                "HTTP/1.1 200 OK\r\nContent-Type: application/json\r\nContent-Length: {}\r\nConnection: close\r\n\r\n",
                payload.len()
            );
            write_half.write_all(head.as_bytes()).await?;
            write_half.write_all(payload.as_bytes()).await?;
        }
        Some(Response::Sse(frames)) => {
            let head = "HTTP/1.1 200 OK\r\nContent-Type: text/event-stream\r\nCache-Control: no-cache\r\nConnection: keep-alive\r\n\r\n";
            write_half.write_all(head.as_bytes()).await?;
            write_half.flush().await?;
            // Delay frames so tests can `subscribe` before events arrive.
            tokio::time::sleep(Duration::from_millis(150)).await;
            for frame in frames {
                let line = format!("data: {}\n\n", serde_json::to_string(&frame).unwrap());
                write_half.write_all(line.as_bytes()).await?;
                write_half.flush().await?;
            }
            // Hold connection open so subscribers can drain events.
            tokio::time::sleep(Duration::from_millis(300)).await;
        }
        Some(Response::Empty) => {
            let head = "HTTP/1.1 200 OK\r\nContent-Length: 0\r\nConnection: close\r\n\r\n";
            write_half.write_all(head.as_bytes()).await?;
        }
        None => {
            let head = "HTTP/1.1 404 Not Found\r\nContent-Length: 0\r\nConnection: close\r\n\r\n";
            write_half.write_all(head.as_bytes()).await?;
        }
    }
    write_half.flush().await?;
    Ok(())
}

/// Handler for the standard 4-endpoint dance. The `/event` stream emits
/// `sse_events` once (on the first connection) and then closes.
pub(super) fn handler_with_events(
    session_id: &'static str,
    message_id: &'static str,
    sse_events: Vec<serde_json::Value>,
) -> impl Fn(&str, &str) -> Option<Response> + Send + Sync {
    let sse_events = Arc::new(std::sync::Mutex::new(Some(sse_events)));
    move |method, path| {
        if method == "POST" && path == "/session" {
            return Some(Response::Json(serde_json::json!({ "id": session_id })));
        }
        if method == "POST" && path == format!("/session/{session_id}/message") {
            return Some(Response::Json(
                serde_json::json!({ "messageID": message_id }),
            ));
        }
        if method == "GET" && path == "/event" {
            let frames = sse_events
                .lock()
                .ok()
                .and_then(|mut g| g.take())
                .unwrap_or_default();
            return Some(Response::Sse(frames));
        }
        if method == "DELETE" && path.starts_with(&format!("/session/{session_id}/message/")) {
            return Some(Response::Empty);
        }
        None
    }
}
