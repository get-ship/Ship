use std::sync::Arc;

use tokio::io::{AsyncBufReadExt, AsyncReadExt, AsyncWriteExt, BufReader};
use tokio::net::{TcpListener, TcpStream};
use tokio::sync::Mutex;

use crate::trait_def::IsolatorError;

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
    let Some(target) = parse_connect_target(&request_line) else {
        let _ = wr
            .write_all(b"HTTP/1.1 400 Bad Request\r\n\r\n")
            .await;
        return;
    };
    let mut headers = String::new();
    loop {
        let mut line = String::new();
        if reader.read_line(&mut line).await.is_err() {
            return;
        }
        if line == "\r\n" || line.is_empty() {
            break;
        }
        headers.push_str(&line);
        if headers.len() > 8192 {
            break;
        }
    }

    let host = target.split(':').next().unwrap_or("").to_string();
    if !matcher.matches(&host) {
        log.lock()
            .await
            .push(host.clone(), ProxyDecision::Deny("not in allowlist".into()));
        let _ = wr.write_all(b"HTTP/1.1 403 Forbidden\r\n\r\n").await;
        return;
    }

    let upstream = match TcpStream::connect(&target).await {
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

    if wr
        .write_all(b"HTTP/1.1 200 Connection Established\r\n\r\n")
        .await
        .is_err()
    {
        return;
    }

    let (mut up_rd, mut up_wr) = upstream.into_split();
    let (mut downbuf_rd, _) = (reader.into_inner(), ());

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

fn parse_connect_target(line: &str) -> Option<String> {
    let trimmed = line.trim_end_matches(['\r', '\n']);
    let mut parts = trimmed.splitn(3, ' ');
    let method = parts.next()?;
    let target = parts.next()?;
    let _version = parts.next()?;
    if method.eq_ignore_ascii_case("CONNECT") {
        Some(target.to_string())
    } else {
        None
    }
}

#[allow(dead_code)]
async fn drain_until_eof<R: AsyncReadExt + Unpin>(mut r: R) {
    let mut buf = [0u8; 1024];
    while r.read(&mut buf).await.unwrap_or(0) > 0 {}
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
    fn parse_connect_line() {
        assert_eq!(
            parse_connect_target("CONNECT example.com:443 HTTP/1.1\r\n"),
            Some("example.com:443".into())
        );
        assert_eq!(
            parse_connect_target("GET / HTTP/1.1\r\n"),
            None
        );
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
