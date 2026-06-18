//! Daemon token primitives — generation, constant-time comparison, and the
//! on-disk token file under `<daemon_dir>/token`.

use std::path::Path;

/// Constant-time token comparison to prevent timing attacks.
/// XORs all bytes and checks the accumulator — execution time is
/// independent of how many bytes match.
pub(super) fn tokens_equal(a: &str, b: &str) -> bool {
    let a = a.as_bytes();
    let b = b.as_bytes();
    if a.len() != b.len() {
        return false;
    }
    let mut acc: u8 = 0;
    for (x, y) in a.iter().zip(b.iter()) {
        acc |= x ^ y;
    }
    acc == 0
}

/// Generate a cryptographically random 256-bit token as a hex string.
///
/// Returns an error if the OS entropy source is unavailable.
pub fn generate_token() -> Result<String, getrandom::Error> {
    let mut bytes = [0u8; 32];
    getrandom::getrandom(&mut bytes)?;
    Ok(hex_encode(&bytes))
}

fn hex_encode(bytes: &[u8]) -> String {
    let mut s = String::with_capacity(bytes.len() * 2);
    for b in bytes {
        s.push_str(&format!("{b:02x}"));
    }
    s
}

/// Write the daemon token to `<daemon_dir>/token` with restricted
/// permissions. Parent directories are created if missing. On Unix the
/// daemon dir itself is forced to 0700 so other local users can't enumerate
/// or read sibling secrets (sessions/, pending-pairings, etc.).
pub fn write_token_file(daemon_dir: &Path, token: &str) -> std::io::Result<()> {
    std::fs::create_dir_all(daemon_dir)?;
    #[cfg(unix)]
    {
        use std::os::unix::fs::PermissionsExt;
        std::fs::set_permissions(daemon_dir, std::fs::Permissions::from_mode(0o700))?;
    }
    let path = daemon_dir.join("token");
    std::fs::write(&path, token)?;
    #[cfg(unix)]
    {
        use std::os::unix::fs::PermissionsExt;
        std::fs::set_permissions(&path, std::fs::Permissions::from_mode(0o600))?;
    }
    Ok(())
}

pub fn read_token(daemon_dir: &Path) -> std::io::Result<String> {
    Ok(std::fs::read_to_string(daemon_dir.join("token"))?.trim().to_string())
}

/// Read the on-disk daemon token, or generate + persist a fresh one
/// when the file is missing or empty. Returns an `Arc<String>` so the
/// bearer-handler can hold the same handle the runtime threads through
/// the router without re-allocating on every request.
pub fn read_or_generate_token(daemon_dir: &Path) -> std::io::Result<std::sync::Arc<String>> {
    match read_token(daemon_dir) {
        Ok(existing) if !existing.is_empty() => Ok(std::sync::Arc::new(existing)),
        _ => {
            let new_token = generate_token().map_err(|e| {
                std::io::Error::new(std::io::ErrorKind::Other, format!("generate token: {e}"))
            })?;
            write_token_file(daemon_dir, &new_token)?;
            Ok(std::sync::Arc::new(new_token))
        }
    }
}

/// Determine if a token is the local daemon token (64-char hex).
pub fn is_daemon_token(token: &str) -> bool {
    token.len() == 64 && token.chars().all(|c| c.is_ascii_hexdigit())
}
