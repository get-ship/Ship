/**
 * mountDaemonFragment — fetch an HTML fragment from the daemon and drop it
 * into `target`. ship-elements and datastar are already loaded at shell
 * boot, so hydration happens automatically as elements connect.
 *
 * Auth: we send the Bearer token on the fetch. The daemon's /raw endpoint
 * also mints a ship_session cookie scoped to /api/runtime, which lets
 * subsequent same-origin requests (including EventSource SSE) authenticate
 * without explicit headers.
 */
export async function mountDaemonFragment(
  target: HTMLElement,
  path: string,
  opts: { token: string; base: string },
): Promise<void> {
  const url = opts.base.replace(/\/$/, "") + path;
  // credentials: "omit" — the daemon only emits
  // `access-control-allow-credentials: true` on preflight in this env. We
  // don't need cookies here; the Bearer header authenticates the GET.
  const res = await fetch(url, {
    method: "GET",
    credentials: "omit",
    headers: { Authorization: `Bearer ${opts.token}`, Accept: "text/html" },
  });
  if (!res.ok) {
    target.innerHTML = `<pre>mount failed: ${res.status} ${res.statusText}\nGET ${url}</pre>`;
    return;
  }
  const html = await res.text();
  target.innerHTML = html;
  // Elements hydrate on connectedCallback; datastar scans attributes on
  // DOMContentLoaded and via a MutationObserver. Nothing else to wire here.
}

/**
 * openDaemonSSE — opens an EventSource against the daemon. EventSource
 * cannot set an Authorization header, so we rely on either:
 *   - the ship_session cookie minted by a prior /raw call (same-origin
 *     via a dev proxy), or
 *   - `?token=<bearer>` query param if the daemon accepts it.
 *
 * Returns the EventSource for teardown.
 */
export function openDaemonSSE(
  path: string,
  opts: { token: string; base: string; sameOrigin?: boolean },
): EventSource {
  const base = (opts.sameOrigin ? "" : opts.base).replace(/\/$/, "");
  const sep = path.includes("?") ? "&" : "?";
  const url = `${base}${path}${sep}token=${encodeURIComponent(opts.token)}`;
  return new EventSource(url, { withCredentials: true });
}
