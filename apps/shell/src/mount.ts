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
  const headers: Record<string, string> = {
    Accept: "text/html",
    "Datastar-Request": "true",
  };
  if (!sameOrigin && opts.token) headers.Authorization = `Bearer ${opts.token}`;
  let res: Response;
  try {
    res = await fetch(url, {
      method: "GET",
      credentials: sameOrigin ? "include" : "omit",
      headers,
    });
  } catch (err) {
    target.innerHTML = renderErrorState({
      title: "Can't reach the Ship daemon",
      body: "The view server didn't respond. Make sure the daemon is running.",
      details: `GET ${url}\n${String(err)}`,
      retry: path,
    });
    return;
  }
  if (!res.ok) {
    target.innerHTML = renderErrorState({
      title: `Failed to load this view (${res.status})`,
      body: res.status >= 500
        ? "The daemon couldn't render this view. The details below may help."
        : "The view couldn't be loaded. Try reloading, or check that the route exists.",
      details: `GET ${url}\n${res.status} ${res.statusText}`,
      retry: path,
    });
    return;
  }
  const html = await res.text();
  target.innerHTML = html;
  // innerHTML inserts <script> nodes but does not execute them. Re-create
  // each <script> so views with inline interactivity (e.g. the canvas in
  // /overview) actually run. Inline scripts first, then src= scripts so
  // any bootstrapping state is ready before external modules load.
  for (const old of Array.from(target.querySelectorAll('script'))) {
    const fresh = document.createElement('script');
    for (const { name, value } of Array.from(old.attributes)) fresh.setAttribute(name, value);
    fresh.text = old.textContent ?? '';
    old.replaceWith(fresh);
  }
}

/**
 * Render a ship-error-state HTML fragment for mount failures.
 *
 * Kept as a string template rather than a Lit component because this
 * runs before ship-elements.js finishes loading in the worst case
 * (daemon unreachable from the first nav). <ship-error-state> is
 * already registered, so the element hydrates when it connects; the
 * fallback copy below stays readable even if hydration never happens.
 */
function renderErrorState(opts: {
  title: string;
  body: string;
  details?: string;
  retry?: string;
}): string {
  const esc = (s: string) =>
    s.replace(/[&<>"']/g, c => ({ "&": "&amp;", "<": "&lt;", ">": "&gt;", '"': "&quot;", "'": "&#39;" })[c]!);
  const details = opts.details
    ? `<pre slot="details">${esc(opts.details)}</pre>`
    : "";
  const retry = opts.retry
    ? `<ship-button slot="action" tone="accent" variant="solid"
         onclick="window.__shipRetryMount && window.__shipRetryMount(${JSON.stringify(opts.retry)})">
         Retry
       </ship-button>`
    : "";
  return `<ship-error-state size="lg" title="${esc(opts.title)}">
    ${esc(opts.body)}
    ${details}
    ${retry}
  </ship-error-state>`;
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
