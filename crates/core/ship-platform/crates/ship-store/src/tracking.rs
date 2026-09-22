//! Fire-and-forget install tracking.
//!
//! After a package is freshly fetched from the registry (not a cache hit),
//! we POST to the registry's install endpoint so download counts are tracked.
//! This is non-blocking: a detached thread runs the POST in the background.
//! Failures are silently ignored — tracking must never slow down or break installs.
//!
//! Implementation note: this uses [`reqwest::blocking`] running in-process on a
//! worker thread rather than shelling out to `curl(1)`. The behavior is
//! *almost* identical for the happy path (POST with 5 s timeout, silently
//! discard the response), but the failure modes differ: the blocking client
//! runs in-process, so if the daemon is killed mid-POST the in-flight request
//! is abandoned; `curl` runs in its own process and would survive a daemon
//! SIGKILL. We accept that edge case — install tracking is best-effort
//! telemetry, never a correctness boundary.

use std::time::Duration;

use super::url_encode::percent_encode;

const TRACK_TIMEOUT: Duration = Duration::from_secs(5);

/// Fire-and-forget POST to track a package install.
///
/// Spawns a detached thread that performs a blocking POST with a 5 s timeout.
/// Returns immediately. Never errors — all failures are silently swallowed.
///
/// Skipped when:
/// - `offline` is true (--offline flag)
/// - `SHIP_REGISTRY_URL` is set to an empty string (opt-out)
pub fn track_install(package_path: &str, offline: bool) {
    if offline {
        return;
    }

    // Check opt-out: SHIP_REGISTRY_URL set to empty string.
    if let Ok(val) = ship_substrate::env::read_raw(ship_substrate::env::Var::RegistryUrl)
        && val.is_empty()
    {
        return;
    }

    let base_url = ship_substrate::env::read(ship_substrate::env::Var::RegistryUrl)
        .unwrap_or_else(|| "https://getship.dev".to_string());
    let encoded = percent_encode(package_path);
    let endpoint = format!("{}/api/registry/{}/install", base_url, encoded);

    std::thread::spawn(move || {
        let Ok(client) = reqwest::blocking::Client::builder()
            .timeout(TRACK_TIMEOUT)
            .build()
        else {
            return;
        };
        let _ = client
            .post(&endpoint)
            .header("User-Agent", "ship-pkg/0.1")
            .send();
    });
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn track_install_skipped_when_offline() {
        // Should return immediately without spawning anything.
        // No assertion needed — this just verifies it doesn't panic.
        track_install("github.com/test/pkg", true);
    }

    #[test]
    fn track_install_skipped_when_opted_out() {
        // Set SHIP_REGISTRY_URL to empty to opt out.
        // SAFETY: test is single-threaded; env var is restored immediately after.
        let prev = ship_substrate::env::read_raw(ship_substrate::env::Var::RegistryUrl).ok();
        unsafe { ship_substrate::env::set(ship_substrate::env::Var::RegistryUrl, "") };
        track_install("github.com/test/pkg", false);
        match prev {
            Some(val) => unsafe {
                ship_substrate::env::set(ship_substrate::env::Var::RegistryUrl, &val)
            },
            None => unsafe { ship_substrate::env::remove(ship_substrate::env::Var::RegistryUrl) },
        }
    }

    #[test]
    fn track_install_does_not_block_on_bad_url() {
        // Point to an unreachable port on localhost. The spawned worker thread
        // should fail fast (connection refused) without making `track_install`
        // block. We assert that the call returns well under our internal
        // 5 s timeout — a hard upper bound of 500 ms leaves 10x headroom for
        // slow CI while still catching a regression that re-introduces a
        // blocking wait on the caller side.
        // SAFETY: test is single-threaded; env var is restored immediately after.
        let prev = ship_substrate::env::read_raw(ship_substrate::env::Var::RegistryUrl).ok();
        unsafe {
            ship_substrate::env::set(ship_substrate::env::Var::RegistryUrl, "http://127.0.0.1:1")
        };
        let start = std::time::Instant::now();
        track_install("github.com/test/pkg", false);
        let elapsed = start.elapsed();
        match prev {
            Some(val) => unsafe {
                ship_substrate::env::set(ship_substrate::env::Var::RegistryUrl, &val)
            },
            None => unsafe { ship_substrate::env::remove(ship_substrate::env::Var::RegistryUrl) },
        }
        assert!(
            elapsed < std::time::Duration::from_millis(500),
            "track_install blocked for {elapsed:?} on a bad URL; expected < 500ms"
        );
    }
}
