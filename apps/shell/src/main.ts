import Navigo from "navigo";
import { mountDaemonFragment, openDaemonSSE, isSameOrigin } from "./mount";
import { initSignalInspector } from "./signal-inspector";
import { initThemeLoader, setTheme } from "./theme-loader";

// ── Daemon URL rewriting ─────────────────────────────────────────────
// Daemon templates emit absolute URLs (http://127.0.0.1:9315/api/...).
// Rewrite to relative at the fetch boundary so datastar, <form action>,
// and <a href> all Just Work through the shell proxy.
const DAEMON_ORIGINS = new Set([
  "http://127.0.0.1:9315",
  "http://localhost:9315",
]);
function rewriteDaemonUrl(input: unknown) {
  if (typeof input === "string") {
    for (const origin of DAEMON_ORIGINS)
      if (input.startsWith(origin)) return input.slice(origin.length) || "/";
    return input;
  }
  if (input instanceof URL) {
    const s = input.toString();
    for (const origin of DAEMON_ORIGINS)
      if (s.startsWith(origin))
        return new URL(s.slice(origin.length) || "/", location.origin);
    return input;
  }
  if (input instanceof Request) {
    for (const origin of DAEMON_ORIGINS) {
      if (input.url.startsWith(origin)) {
        const path = input.url.slice(origin.length) || "/";
        return new Request(path, input);
      }
    }
    return input;
  }
  return input;
}
const nativeFetch = window.fetch.bind(window);
(window as any).fetch = (input: any, init: any) =>
  nativeFetch(rewriteDaemonUrl(input) as any, init);

const NativeEventSource = window.EventSource;
(window as any).EventSource = function (url: string, opts?: EventSourceInit) {
  return new NativeEventSource(rewriteDaemonUrl(url) as string, {
    withCredentials: true,
    ...(opts ?? {}),
  });
} as any;
(window as any).EventSource.CONNECTING = NativeEventSource.CONNECTING;
(window as any).EventSource.OPEN = NativeEventSource.OPEN;
(window as any).EventSource.CLOSED = NativeEventSource.CLOSED;

// ── Auth / origin ────────────────────────────────────────────────────
const RAW_BASE = (import.meta.env.VITE_DAEMON_URL as string) ?? "";
const SAME_ORIGIN = isSameOrigin(RAW_BASE);
const DAEMON_BASE = SAME_ORIGIN ? "" : RAW_BASE;
const TOKEN = SAME_ORIGIN
  ? ""
  : ((import.meta.env.VITE_DAEMON_TOKEN as string) ?? "");

// ── DOM refs ─────────────────────────────────────────────────────────
const view = document.getElementById("ship-view-root") as HTMLElement;
const drawerContent = document.getElementById(
  "drawer-content",
) as HTMLElement;
const drawerBtn = document.getElementById(
  "toggle-drawer",
) as HTMLButtonElement;
const themeBtn = document.getElementById(
  "toggle-theme",
) as HTMLButtonElement;

// ── Theme toggle ─────────────────────────────────────────────────────
themeBtn.addEventListener("click", () => {
  const isDark = document.documentElement.classList.toggle("dark");
  localStorage.setItem("ship-shell-theme", isDark ? "dark" : "light");
});

// ── SSE management ───────────────────────────────────────────────────
let currentSSE: EventSource | null = null;
function closeSSE() {
  if (currentSSE) {
    currentSSE.close();
    currentSSE = null;
  }
}

// ── Mount a daemon view into the main pane ───────────────────────────
async function mount(endpoint: string, sse?: string) {
  closeSSE();
  view.innerHTML = `<div style="padding:40px;text-align:center;color:var(--muted-foreground)">Loading…</div>`;
  try {
    await mountDaemonFragment(view, endpoint, {
      token: TOKEN,
      base: DAEMON_BASE,
    });
  } catch {
    view.innerHTML = `<div style="padding:40px;text-align:center;color:var(--destructive)">Failed to load view.</div>`;
    return;
  }
  if (sse) {
    currentSSE = openDaemonSSE(sse, { token: TOKEN, base: DAEMON_BASE });
  }
}

// ── Form submit handler (delegated) ──────────────────────────────────
view.addEventListener("submit", (ev) => {
  const form = ev.target;
  if (!(form instanceof HTMLFormElement)) return;
  ev.preventDefault();
  void submitFragmentForm(form);
});

async function submitFragmentForm(form: HTMLFormElement) {
  const action = form.getAttribute("action") || form.dataset.action || "";
  const url = /^https?:/.test(action) ? action : `${DAEMON_BASE}${action}`;
  const headers: Record<string, string> = {
    "Content-Type": "application/x-www-form-urlencoded",
    Accept: "text/html",
  };
  if (!SAME_ORIGIN && TOKEN) headers.Authorization = `Bearer ${TOKEN}`;
  const body = new URLSearchParams(
    new FormData(form) as unknown as Record<string, string>,
  );
  const res = await fetch(url, {
    method: form.method || "POST",
    credentials: SAME_ORIGIN ? "include" : "omit",
    headers,
    body: body.toString(),
  });
  const html = await res.text();
  const target =
    document.getElementById(form.dataset.target ?? "ship-view-root") ?? view;
  target.innerHTML = html;
}

// ── Update nav active state ──────────────────────────────────────────
function updateNav(path: string) {
  document
    .querySelectorAll<HTMLAnchorElement>("header a[data-route]")
    .forEach((a) => {
      const route = a.dataset.route!;
      // Match exact or prefix (e.g. /skills matches /skills/:id)
      const active = path === route || path.startsWith(route + "/");
      a.classList.toggle("active", active);
    });
}

// ── Router (Navigo) ──────────────────────────────────────────────────
const router = new Navigo("/");

// View routes — static top-level
function viewRoute(
  path: string,
  endpoint: string,
  sse?: string,
) {
  router.on(path, () => {
    updateNav(path);
    void mount(endpoint, sse);
  });
}

viewRoute("/", "/api/runtime/view/pipelines", "/api/runtime/view/pipelines/sse");
viewRoute("/pipelines", "/api/runtime/view/pipelines", "/api/runtime/view/pipelines/sse");
viewRoute("/specs", "/api/runtime/view/specs", "/api/runtime/view/specs/sse");
viewRoute("/git", "/api/runtime/view/git", "/api/runtime/view/git/sse");
viewRoute("/settings", "/api/runtime/view/settings", "/api/runtime/view/settings/sse");
viewRoute("/agents", "/api/runtime/view/agents", "/api/runtime/view/agents/sse");
viewRoute("/skills", "/api/runtime/view/skills", "/api/runtime/view/skills/sse");
viewRoute("/chat", "/api/runtime/view/chat", "/api/runtime/view/chat/sse");
viewRoute("/events", "/api/runtime/view/events", "/api/runtime/view/events/sse");
viewRoute("/data", "/api/runtime/view/data", "/api/runtime/view/data/sse");
viewRoute("/dashboard", "/api/runtime/view/dashboard", "/api/runtime/view/dashboard/sse");
viewRoute("/canvas", "/api/runtime/view/canvas", "/api/runtime/view/canvas/sse");

// Dynamic routes — detail pages with params
router.on("/agents/:id", ({ data }) => {
  updateNav("/agents");
  void mount(`/api/runtime/view/agents/detail/${data!.id}`);
});

router.on("/skills/:id", ({ data }) => {
  updateNav("/skills");
  void mount(`/api/runtime/view/skills/detail/${data!.id}`);
});

router.on("/skills/:id/:file", ({ data }) => {
  updateNav("/skills");
  void mount(`/api/runtime/view/skills/detail/${data!.id}?file=${encodeURIComponent(data!.file)}`);
});

router.on("/canvas/:doc", ({ data }) => {
  updateNav("/canvas");
  void mount(`/api/runtime/view/canvas?doc=${encodeURIComponent(data!.doc)}`);
});

// 404 fallback
router.notFound(() => {
  updateNav("");
  view.innerHTML = `
    <div style="padding:60px;text-align:center;">
      <h2 style="font-family:var(--font-display);font-size:var(--text-xl);margin:0 0 8px">Page not found</h2>
      <p style="color:var(--muted-foreground)">The route "${location.pathname}" doesn't exist.</p>
      <ship-button tone="accent" variant="solid" style="margin-top:16px"
        onclick="window.router.navigate('/')">Go home</ship-button>
    </div>`;
});

// Wire nav links to router
document
  .querySelectorAll<HTMLAnchorElement>("header a[data-route]")
  .forEach((a) => {
    a.addEventListener("click", (e) => {
      e.preventDefault();
      router.navigate(a.dataset.route!);
    });
  });

// Drawer toggle
drawerBtn.addEventListener("click", async () => {
  document.body.classList.toggle("drawer-open");
  if (document.body.classList.contains("drawer-open")) {
    await mountDaemonFragment(drawerContent, "/api/runtime/view/events", {
      token: TOKEN,
      base: DAEMON_BASE,
    });
  }
});

// Expose router globally so views can navigate programmatically
// e.g. onclick="window.router.navigate('/agents/commander')"
(window as any).router = router;

// Start routing
router.resolve();

// Theming
initThemeLoader();
(window as any).setShipTheme = setTheme;

// Dev tooling
initSignalInspector();
