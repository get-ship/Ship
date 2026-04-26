/**
 * Client-side error reporter.
 *
 * Captures browser-side errors (window.error, unhandledrejection, Datastar
 * fetch errors) and forwards them to the daemon's ring buffer so agents can
 * inspect them via the `ship_view_errors` MCP tool without the developer
 * copy-pasting console output.
 *
 * Guardrails:
 *  - Rate-limits identical errors (same message+stack hash) to 1 per 5s.
 *  - Never reports its own failures — a failed POST to the endpoint is
 *    swallowed silently.
 *  - Ring buffer lives in the daemon; this file is pure transport.
 */

type ReportPayload = {
  view?: string;
  url: string;
  type: string;
  message: string;
  stack?: string;
  filename?: string;
  line?: number;
  column?: number;
  detail?: unknown;
  user_agent: string;
  timestamp_client: string;
};

const ENDPOINT = "/api/v1/view-errors";
const DEDUPE_WINDOW_MS = 5000;
const dedupe = new Map<string, number>();

let installed = false;
let reporting = false; // re-entrancy guard while the fetch is in flight

function currentView(): string | undefined {
  // Route-style: first path segment after the leading slash.
  const parts = location.pathname.split("/").filter(Boolean);
  return parts[0] ?? undefined;
}

function hashKey(msg: string, stack?: string): string {
  // Cheap, deterministic: message + first line of stack is enough to
  // collapse a tight error loop without cryptographic hashing.
  const head = stack?.split("\n", 1)[0] ?? "";
  return `${msg}::${head}`;
}

function shouldReport(key: string): boolean {
  const now = Date.now();
  const last = dedupe.get(key) ?? 0;
  if (now - last < DEDUPE_WINDOW_MS) return false;
  dedupe.set(key, now);
  // Opportunistic cleanup to keep the map small.
  if (dedupe.size > 200) {
    for (const [k, t] of dedupe) {
      if (now - t > DEDUPE_WINDOW_MS * 4) dedupe.delete(k);
    }
  }
  return true;
}

function send(payload: ReportPayload): void {
  if (reporting) return;
  reporting = true;
  // Fire-and-forget. We use `keepalive` so late errors during page unload
  // still make it out. Ignore all failures — this path must never throw.
  try {
    fetch(ENDPOINT, {
      method: "POST",
      headers: { "Content-Type": "application/json" },
      credentials: "include",
      body: JSON.stringify(payload),
      keepalive: true,
    })
      .catch(() => {
        /* swallow — reporter must never cascade */
      })
      .finally(() => {
        reporting = false;
      });
  } catch {
    reporting = false;
  }
}

function report(
  type: string,
  message: string,
  extras: Partial<ReportPayload> = {},
): void {
  if (!message) return;
  const key = hashKey(message, extras.stack);
  if (!shouldReport(key)) return;
  send({
    view: currentView(),
    url: location.pathname + location.search,
    type,
    message,
    user_agent: navigator.userAgent,
    timestamp_client: new Date().toISOString(),
    ...extras,
  });
}

export function installErrorReporter(): void {
  if (installed) return;
  installed = true;

  window.addEventListener("error", (evt) => {
    const err = evt.error;
    const msg =
      (err && (err.message as string)) ||
      (typeof evt.message === "string" ? evt.message : "") ||
      "window error";
    report("js-error", msg, {
      stack: err?.stack,
      filename: evt.filename,
      line: evt.lineno,
      column: evt.colno,
    });
  });

  window.addEventListener("unhandledrejection", (evt) => {
    const reason = (evt as PromiseRejectionEvent).reason;
    const msg =
      (reason && (reason.message as string)) ||
      (typeof reason === "string" ? reason : "") ||
      "unhandled rejection";
    report("unhandled-rejection", String(msg), {
      stack: reason?.stack,
    });
  });

  // Datastar fires `datastar-fetch` on document when a fetch action fails
  // (HTTP >= 400 or non-SSE body). detail: { type, el, argsRaw: { status } }.
  // See _ship.html.j2 on_fetch_error macro for the canonical handler shape.
  document.addEventListener("datastar-fetch", (evt: Event) => {
    const detail = (evt as CustomEvent).detail;
    if (!detail || detail.type !== "error") return;
    const status = detail.argsRaw?.status;
    const url = detail.argsRaw?.url ?? detail.argsRaw?.action;
    const tag = (detail.el as HTMLElement | undefined)?.tagName?.toLowerCase();
    report(
      "datastar-fetch",
      `datastar fetch error${status ? ` (HTTP ${status})` : ""}${tag ? ` on <${tag}>` : ""}`,
      {
        detail: {
          status,
          action_url: url,
          trigger_tag: tag,
          raw_type: detail.type,
        },
      },
    );
  });
}
