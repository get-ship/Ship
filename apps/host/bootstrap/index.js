// Retry wiring for the bootstrap error state. Lives in its own file because
// the shell CSP is `script-src 'self'` — inline handlers never execute.
//
// This page cannot reach the daemon directly (the loopback port is unknown
// here), so Retry drives the same native commands the host's setup thread
// uses — `daemon_status` (read-only probe), then `daemon_start` — via the
// `withGlobalTauri` bridge, and reports what they said. Navigation stays
// native: the host's watcher thread probes indefinitely and navigates the
// moment a daemon handshake appears, so this page never composes a daemon
// URL of its own.
(() => {
  const button = document.getElementById('retry');
  if (!button) return;
  const status = document.getElementById('retry-status');
  const show = (text) => {
    if (status) status.textContent = text;
  };
  // A successful probe normally ends with the native watcher navigating away
  // within one poll interval. If we're still here well past that, the daemon
  // exited again — re-arm Retry instead of leaving it disabled for good.
  const rearmIfStranded = () => {
    setTimeout(() => {
      button.disabled = false;
      show('Still not connected — the daemon may have exited again.');
    }, 15000);
  };

  button.addEventListener('click', async () => {
    const invoke = window.__TAURI__?.core?.invoke;
    if (!invoke) {
      show('Native bridge unavailable — restart the app.');
      return;
    }
    button.disabled = true;
    show('Checking for a running daemon…');
    try {
      const current = await invoke('daemon_status');
      if (current && current.running && current.port) {
        show('Daemon is running — connecting…');
        rearmIfStranded();
        return;
      }
      show('Starting the daemon…');
      const started = await invoke('daemon_start');
      if (started && started.port) {
        show('Daemon started — connecting…');
        rearmIfStranded();
        return;
      }
      show('Daemon started but reported no port — check the daemon log.');
    } catch (err) {
      show(String(err));
    }
    button.disabled = false;
  });
})();
