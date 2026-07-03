use std::sync::Arc;

use tokio::io::{AsyncBufReadExt, AsyncReadExt, AsyncWriteExt, BufReader};
use tokio::net::{TcpListener, TcpStream};
use tokio::sync::Mutex;

use isolation_port::IsolatorError;

#[derive(Debug, Clone)]
pub struct HostMatcher {
    rules: Vec<String>,
}

impl HostMatcher {
    pub fn new(rules: Vec<String>) -> Self {
        Self { rules }
    }

    pub fn matches(&self, host: &str) -> bool {
        let host_lower = host.to_ascii_lowercase();
        for rule in &self.rules {
            let r = rule.to_ascii_lowercase();
            if r == host_lower {
                return true;
            }
            if let Some(suffix) = r.strip_prefix("*.") {
                if host_lower == suffix
                    || host_lower.ends_with(&format!(".{suffix}"))
                {
                    return true;
                }
            }
        }
        false
    }
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub enum ProxyDecision {
    Allow,
    Deny(String),
}

#[derive(Debug, Default)]
pub struct ProxyLog {
    entries: Vec<(String, ProxyDecision)>,
}

impl ProxyLog {
    pub fn push(&mut self, host: String, decision: ProxyDecision) {
        self.entries.push((host, decision));
    }
    pub fn entries(&self) -> &[(String, ProxyDecision)] {
        &self.entries
    }
}

pub struct ProxyHandle {
    pub port: u16,
    pub log: Arc<Mutex<ProxyLog>>,
    shutdown_tx: tokio::sync::oneshot::Sender<()>,
}

impl ProxyHandle {
    pub async fn shutdown(self) {
        let _ = self.shutdown_tx.send(());
    }
}

pub async fn spawn_allowlist_proxy(
    matcher: HostMatcher,
) -> Result<ProxyHandle, IsolatorError> {
    let listener = TcpListener::bind("127.0.0.1:0").await?;
    let port = listener.local_addr()?.port();
    let log = Arc::new(Mutex::new(ProxyLog::default()));
    let (shutdown_tx, mut shutdown_rx) = tokio::sync::oneshot::channel();
    let matcher = Arc::new(matcher);
    let log_for_task = log.clone();

    tokio::spawn(async move {
        loop {
            tokio::select! {
                _ = &mut shutdown_rx => break,
                accept = listener.accept() => {
                    let Ok((sock, _peer)) = accept else { continue };
                    let matcher = matcher.clone();
                    let log = log_for_task.clone();
                    tokio::spawn(handle_conn(sock, matcher, log));
                }
            }
        }
    });

    Ok(ProxyHandle { port, log, shutdown_tx })
}

enum ParsedRequest {
    Connect { host: String, port: u16 },
    AbsoluteUri { host: String, port: u16, raw: String },
}

fn parse_request_line(line: &str) -> Option<ParsedRequest> {
    let trimmed = line.trim_end_matches(['\r', '\n']);
    let mut parts = trimmed.splitn(3, ' ');
    let method = parts.next()?;
    let target = parts.next()?;
    let _version = parts.next()?;
    if method.eq_ignore_ascii_case("CONNECT") {
        let (host, port) = split_host_port(target, 443);
        return Some(ParsedRequest::Connect { host, port });
    }
    if let Some(after) = target.strip_prefix("http://") {
        let (authority, _path) = after.split_once('/').unwrap_or((after, ""));
        let (host, port) = split_host_port(authority, 80);
        return Some(ParsedRequest::AbsoluteUri {
            host,
            port,
            raw: line.to_string(),
        });
    }
    None
}

fn split_host_port(authority: &str, default_port: u16) -> (String, u16) {
    if let Some((h, p)) = authority.rsplit_once(':') {
        if let Ok(port) = p.parse::<u16>() {
            return (h.to_string(), port);
        }
    }
    (authority.to_string(), default_port)
}

async fn handle_conn(
    sock: TcpStream,
    matcher: Arc<HostMatcher>,
    log: Arc<Mutex<ProxyLog>>,
) {
    let (rd, mut wr) = sock.into_split();
    let mut reader = BufReader::new(rd);
    let mut request_line = String::new();
    if reader.read_line(&mut request_line).await.is_err() {
        return;
    }
    let parsed = match parse_request_line(&request_line) {
        Some(p) => p,
        None => {
            let _ = wr
                .write_all(b"HTTP/1.1 400 Bad Request\r\n\r\n")
                .await;
            return;
        }
    };

    let (host, port, mode) = match &parsed {
        ParsedRequest::Connect { host, port } => (host.clone(), *port, "CONNECT"),
        ParsedRequest::AbsoluteUri { host, port, .. } => (host.clone(), *port, "HTTP"),
    };

    if !matcher.matches(&host) {
        log.lock()
            .await
            .push(host.clone(), ProxyDecision::Deny("not in allowlist".into()));
        let _ = wr.write_all(b"HTTP/1.1 403 Forbidden\r\n\r\n").await;
        return;
    }

    // Drain remaining headers in both modes; for HTTP we need them to
    // forward verbatim.
    let mut header_lines = Vec::new();
    loop {
        let mut line = String::new();
        if reader.read_line(&mut line).await.is_err() {
            return;
        }
        let is_terminator = line == "\r\n" || line == "\n" || line.is_empty();
        if matches!(parsed, ParsedRequest::AbsoluteUri { .. }) {
            header_lines.push(line.clone());
        }
        if is_terminator {
            break;
        }
        if header_lines.iter().map(|s| s.len()).sum::<usize>() > 8192 {
            break;
        }
    }

    let upstream = match TcpStream::connect((host.as_str(), port)).await {
        Ok(s) => s,
        Err(e) => {
            log.lock()
                .await
                .push(host, ProxyDecision::Deny(format!("dial: {e}")));
            let _ = wr
                .write_all(b"HTTP/1.1 502 Bad Gateway\r\n\r\n")
                .await;
            return;
        }
    };
    log.lock().await.push(host, ProxyDecision::Allow);

    match parsed {
        ParsedRequest::Connect { .. } => {
            if wr
                .write_all(b"HTTP/1.1 200 Connection Established\r\n\r\n")
                .await
                .is_err()
            {
                return;
            }
            let (mut up_rd, mut up_wr) = upstream.into_split();
            let mut downbuf_rd = reader.into_inner();
            let to_up = tokio::spawn(async move {
                let _ = tokio::io::copy(&mut downbuf_rd, &mut up_wr).await;
                let _ = up_wr.shutdown().await;
            });
            let to_dn = tokio::spawn(async move {
                let _ = tokio::io::copy(&mut up_rd, &mut wr).await;
                let _ = wr.shutdown().await;
            });
            let _ = tokio::join!(to_up, to_dn);
        }
        ParsedRequest::AbsoluteUri { raw, host, .. } => {
            let _ = mode; // tag for grep
            // Rewrite the request line so the path is relative (most
            // origin servers won't accept an absolute URI in non-proxy
            // mode). Replace "http://<host>[:port]" with "" in the
            // first line; pass headers through unchanged.
            let rewritten_first = rewrite_first_line(&raw, &host);
            let (mut up_rd, mut up_wr) = upstream.into_split();
            if up_wr.write_all(rewritten_first.as_bytes()).await.is_err() {
                return;
            }
            for h in &header_lines {
                if up_wr.write_all(h.as_bytes()).await.is_err() {
                    return;
                }
            }
            // Forward any body bytes from the client to upstream.
            let mut downbuf_rd = reader.into_inner();
            let to_up = tokio::spawn(async move {
                let _ = tokio::io::copy(&mut downbuf_rd, &mut up_wr).await;
                let _ = up_wr.shutdown().await;
            });
            let to_dn = tokio::spawn(async move {
                let _ = tokio::io::copy(&mut up_rd, &mut wr).await;
                let _ = wr.shutdown().await;
            });
            let _ = tokio::join!(to_up, to_dn);
        }
    }
}

fn rewrite_first_line(raw: &str, host: &str) -> String {
    if let Some(scheme_end) = raw.find("http://") {
        let head = &raw[..scheme_end];
        let after = &raw[scheme_end + "http://".len()..];
        let path_start = after.find('/').unwrap_or(after.len());
        let (_, path_and_version) = after.split_at(path_start);
        let path = if path_and_version.is_empty() { "/" } else { path_and_version };
        return format!("{head}{path}");
    }
    let _ = host;
    raw.to_string()
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn host_matcher_exact() {
        let m = HostMatcher::new(vec!["api.anthropic.com".into()]);
        assert!(m.matches("api.anthropic.com"));
        assert!(m.matches("API.Anthropic.COM"));
        assert!(!m.matches("evil.com"));
    }

    #[test]
    fn host_matcher_glob() {
        let m = HostMatcher::new(vec!["*.googleapis.com".into()]);
        assert!(m.matches("storage.googleapis.com"));
        assert!(m.matches("googleapis.com")); // suffix-only match
        assert!(!m.matches("evil.com"));
        assert!(!m.matches("notgoogleapis.com"));
    }

    #[test]
    fn parse_connect_request_line() {
        match parse_request_line("CONNECT example.com:443 HTTP/1.1\r\n").unwrap() {
            ParsedRequest::Connect { host, port } => {
                assert_eq!(host, "example.com");
                assert_eq!(port, 443);
            }
            _ => panic!("expected Connect"),
        }
    }

    #[test]
    fn parse_absolute_uri_http_line() {
        match parse_request_line("GET http://example.com/api HTTP/1.1\r\n").unwrap() {
            ParsedRequest::AbsoluteUri { host, port, raw } => {
                assert_eq!(host, "example.com");
                assert_eq!(port, 80);
                assert!(raw.starts_with("GET http://example.com/api"));
            }
            _ => panic!("expected AbsoluteUri"),
        }
    }

    #[test]
    fn parse_absolute_uri_with_port() {
        match parse_request_line("POST http://example.com:8080/x HTTP/1.1\r\n").unwrap() {
            ParsedRequest::AbsoluteUri { host, port, .. } => {
                assert_eq!(host, "example.com");
                assert_eq!(port, 8080);
            }
            _ => panic!("expected AbsoluteUri"),
        }
    }

    #[test]
    fn parse_relative_path_returns_none() {
        assert!(parse_request_line("GET / HTTP/1.1\r\n").is_none());
    }

    #[test]
    fn rewrite_strips_scheme_and_authority() {
        let r = rewrite_first_line("GET http://example.com/api?x=1 HTTP/1.1\r\n", "example.com");
        assert_eq!(r, "GET /api?x=1 HTTP/1.1\r\n");
    }

    #[tokio::test]
    async fn proxy_denies_unknown_host() {
        let matcher = HostMatcher::new(vec!["allowed.test".into()]);
        let proxy = spawn_allowlist_proxy(matcher).await.unwrap();
        let port = proxy.port;

        let mut sock = TcpStream::connect(("127.0.0.1", port)).await.unwrap();
        sock.write_all(
            b"CONNECT evil.example.com:443 HTTP/1.1\r\nHost: evil.example.com:443\r\n\r\n",
        )
        .await
        .unwrap();
        let mut buf = vec![0u8; 64];
        let n = sock.read(&mut buf).await.unwrap();
        let resp = std::str::from_utf8(&buf[..n]).unwrap_or("");
        assert!(resp.starts_with("HTTP/1.1 403"), "got: {resp:?}");

        let log = proxy.log.clone();
        proxy.shutdown().await;
        let entries = log.lock().await;
        assert!(entries.entries().iter().any(|(h, d)| h == "evil.example.com"
            && matches!(d, ProxyDecision::Deny(_))));
    }

    #[tokio::test]
    async fn proxy_allows_listed_host_to_loopback_listener() {
        // Stand up a local TCP listener that the proxy is allowed to reach.
        let upstream = TcpListener::bind("127.0.0.1:0").await.unwrap();
        let up_port = upstream.local_addr().unwrap().port();
        tokio::spawn(async move {
            if let Ok((mut s, _)) = upstream.accept().await {
                let _ = s.write_all(b"ack").await;
            }
        });

        // The proxy resolves whatever Host: it gets via DNS; for the test
        // we point it at a hostname that resolves to 127.0.0.1.
        let matcher = HostMatcher::new(vec!["localhost".into()]);
        let proxy = spawn_allowlist_proxy(matcher).await.unwrap();
        let port = proxy.port;

        let mut sock = TcpStream::connect(("127.0.0.1", port)).await.unwrap();
        sock.write_all(
            format!(
                "CONNECT localhost:{up_port} HTTP/1.1\r\nHost: localhost:{up_port}\r\n\r\n"
            )
            .as_bytes(),
        )
        .await
        .unwrap();
        let mut buf = vec![0u8; 64];
        let n = sock.read(&mut buf).await.unwrap();
        let resp = std::str::from_utf8(&buf[..n]).unwrap_or("");
        assert!(resp.starts_with("HTTP/1.1 200"), "got: {resp:?}");

        // After the 200, the proxy splices. Read the upstream's "ack".
        let mut ack = vec![0u8; 8];
        let n = sock.read(&mut ack).await.unwrap();
        assert_eq!(&ack[..n], b"ack");

        proxy.shutdown().await;
    }
}
