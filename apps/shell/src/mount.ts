/**
 * mountDaemonFragment — fetch an HTML fragment from the daemon and drop it
 * into `target`. ship-elements and datastar are already loaded at shell
 * boot, so hydration happens automatically as elements connect.
 *
 * Two auth paths:
 *   - **Same-origin** (daemon serves the shell under /shell/*): send
 *     `credentials: "include"` and rely on the `ship_session` cookie minted
 *     by the initial /shell HTML response. No Authorization header.
 *   - **Cross-origin dev** (vite on :3100 → daemon on :9316): fall back to
 *     Bearer + `credentials: "omit"`. Preserves the original POC path.
 */
export async function mountDaemonFragment(
  target: HTMLElement,
  path: string,
  opts: { token: string; base: string },
): Promise<void> {
  const sameOrigin = isSameOrigin(opts.base);
  const url = sameOrigin ? path : opts.base.replace(/\/$/, "") + path;
  const headers: Record<string, string> = { Accept: "text/html" };
  if (!sameOrigin && opts.token) headers.Authorization = `Bearer ${opts.token}`;
  const res = await fetch(url, {
    method: "GET",
    credentials: sameOrigin ? "include" : "omit",
    headers,
  });
  if (!res.ok) {
    target.innerHTML = `<pre>mount failed: ${res.status} ${res.statusText}\nGET ${url}</pre>`;
    return;
  }
  const html = await res.text();
  target.innerHTML = html;
}

/**
 * openDaemonSSE — opens an EventSource against the daemon.
 *
 * Same-origin: cookie rides automatically, no query-param token needed.
 * Cross-origin: fall back to `?token=` since EventSource cannot set headers.
 */
export function openDaemonSSE(
  path: string,
  opts: { token: string; base: string },
): EventSource {
  if (isSameOrigin(opts.base)) {
    return new EventSource(path, { withCredentials: true });
  }
  const base = opts.base.replace(/\/$/, "");
  const sep = path.includes("?") ? "&" : "?";
  const url = `${base}${path}${sep}token=${encodeURIComponent(opts.token)}`;
  return new EventSource(url, { withCredentials: true });
}

/**
 * True when the shell is served from the same origin as the daemon.
 * Treat an empty/relative `base` as same-origin.
 */
export function isSameOrigin(base: string): boolean {
  if (!base) return true;
  try {
    return new URL(base).origin === window.location.origin;
  } catch {
    return true;
  }
}
