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
  const res = await fetch(url, {
    method: "GET",
    credentials: sameOrigin ? "include" : "omit",
    headers,
  });
  if (!res.ok) {
    target.innerHTML = `<div class="mount-error">mount failed: ${res.status} ${res.statusText}\nGET ${url}</div>`;
    return;
  }
  const html = await res.text();
  target.innerHTML = html;

  // `innerHTML =` does not execute <script> elements per HTML spec —
  // any inline helpers a view ships never register. Rehydrate them by
  // cloning each script into a fresh element and swapping it in; the
  // clone counts as "parser-inserted" and runs.
  for (const stale of Array.from(target.querySelectorAll("script"))) {
    const fresh = document.createElement("script");
    for (const attr of Array.from(stale.attributes)) {
      fresh.setAttribute(attr.name, attr.value);
    }
    fresh.text = stale.textContent ?? "";
    stale.replaceWith(fresh);
  }
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
