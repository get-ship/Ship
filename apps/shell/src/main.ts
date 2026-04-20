import Navigo from "navigo";
import { mountDaemonFragment, isSameOrigin } from "./mount";
import { initSignalInspector } from "./signal-inspector";
import { initThemeLoader, setTheme, getThemeName } from "./theme-loader";
import { toggle as toggleCmdk, isCommandPaletteOpen } from "./command-palette";

// ── Daemon URL rewriting ─────────────────────────────────────────────
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
let view = document.getElementById("ship-view-root") as HTMLElement;
const drawerContent = document.getElementById("drawer-content") as HTMLElement;
const breadcrumbs = document.getElementById("breadcrumbs") as HTMLElement;
const statusBranch = document.getElementById("status-branch") as HTMLElement;
const statusAgents = document.getElementById("status-agents") as HTMLElement;
const statusTheme = document.getElementById("status-theme") as HTMLElement;
const statusDot = document.getElementById("status-dot") as HTMLElement;

// ── Theme toggle ─────────────────────────────────────────────────────
document.getElementById("toggle-theme")!.addEventListener("click", () => {
  const isDark = document.documentElement.classList.toggle("dark");
  localStorage.setItem("ship-shell-theme", isDark ? "dark" : "light");
});

// ── Rail toggle ──────────────────────────────────────────────────────
document.getElementById("toggle-rail")!.addEventListener("click", () => {
  document.body.classList.toggle("rail-open");
  localStorage.setItem(
    "ship-rail-open",
    document.body.classList.contains("rail-open") ? "1" : "",
  );
});
// Restore rail state
if (localStorage.getItem("ship-rail-open")) {
  document.body.classList.add("rail-open");
}

// ── Drawer toggle ────────────────────────────────────────────────────
async function toggleDrawer() {
  const opening = !document.body.classList.contains("drawer-open");
  document.body.classList.toggle("drawer-open");
  if (opening) {
    try {
      await mountDaemonFragment(drawerContent, "/api/runtime/view/chat", {
        token: TOKEN,
        base: DAEMON_BASE,
      });
    } catch {
      drawerContent.innerHTML = `<div class="empty-hint">Failed to load chat</div>`;
    }
  } else {
    // Clear drawer content to stop any Datastar SSE watchers
    drawerContent.textContent = "";
  }
}

document.getElementById("toggle-drawer")!.addEventListener("click", () => {
  void toggleDrawer();
});

// ── Terminal panel toggle ────────────────────────────────────────────
document.getElementById("close-terminal")!.addEventListener("click", () => {
  document.body.classList.remove("terminal-open");
});

// ── Mount a daemon view ──────────────────────────────────────────────
async function mount(endpoint: string) {

  // Remove old view from DOM entirely. This is critical — Datastar's
  // MutationObserver fires on removed nodes and cleans up internal SSE
  // connections from {{ live() }} macros. textContent="" alone is not
  // enough; we need to remove and re-create the mount point.
  //
  // Re-query fresh: a prior Datastar @get may have morphed the root
  // element out from under us (replacing <main id="ship-view-root"> with
  // a <div id="ship-view-root"> from the fragment response). The cached
  // `view` reference would then be detached with a null parentElement.
  const live = document.getElementById("ship-view-root");
  const parent = live?.parentElement ?? document.body;
  const newView = document.createElement("main");
  newView.id = "ship-view-root";
  newView.className = "ship-content";
  if (live && live.parentElement) {
    parent.replaceChild(newView, live);
  } else {
    parent.appendChild(newView);
  }
  view = newView;

  try {
    await mountDaemonFragment(view, endpoint, {
      token: TOKEN,
      base: DAEMON_BASE,
    });
  } catch {
    view.innerHTML = `<div class="mount-error">Failed to load view.\nCheck that the daemon is running.</div>`;
    return;
  }
}

// ── Form submit handler (delegated on document, not view — view gets replaced) ──
document.addEventListener("submit", (ev) => {
  const form = ev.target;
  if (!(form instanceof HTMLFormElement)) return;
  if (!form.closest("#ship-view-root")) return;
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

// ── Breadcrumbs ──────────────────────────────────────────────────────
function updateBreadcrumbs(path: string) {
  const parts = path.split("/").filter(Boolean);
  if (parts.length === 0) parts.push("dashboard");
  const crumbs = parts.map((part, i) => {
    const isLast = i === parts.length - 1;
    const href = "/" + parts.slice(0, i + 1).join("/");
    if (isLast) {
      return `<span class="current">${part}</span>`;
    }
    return `<a onclick="event.preventDefault();window.router.navigate('${href}')">${part}</a><span class="chevron">›</span>`;
  });
  breadcrumbs.innerHTML = crumbs.join("");
}

// ── Update nav active state (rail + any data-route links) ────────────
function updateNav(path: string) {
  document
    .querySelectorAll<HTMLAnchorElement>("[data-route]")
    .forEach((a) => {
      const route = a.dataset.route!;
      const active = path === route || path.startsWith(route + "/");
      a.classList.toggle("active", active);
    });
  updateBreadcrumbs(path);
}

// ── Router (Navigo) ──────────────────────────────────────────────────
const router = new Navigo("/");

function viewRoute(path: string, endpoint: string) {
  router.on(path, () => {
    updateNav(path);
    void mount(endpoint);
  });
}

viewRoute("/", "/api/runtime/view/dashboard");
viewRoute("/pipelines", "/api/runtime/view/pipelines");
viewRoute("/specs", "/api/runtime/view/specs");
viewRoute("/git", "/api/runtime/view/git");
viewRoute("/settings", "/api/runtime/view/settings");
viewRoute("/agents", "/api/runtime/view/agents");
viewRoute("/skills", "/api/runtime/view/skills");
viewRoute("/chat", "/api/runtime/view/chat");
viewRoute("/events", "/api/runtime/view/events");
viewRoute("/data", "/api/runtime/view/data");
viewRoute("/dashboard", "/api/runtime/view/dashboard");
viewRoute("/canvas", "/api/runtime/view/canvas");
viewRoute("/overview", "/api/runtime/view/overview");

// Dynamic routes
router.on("/agents/:id", ({ data }) => {
  updateNav("/agents");
  updateBreadcrumbs(`/agents/${data!.id}`);
  void mount(`/api/runtime/view/agents/detail/${data!.id}`);
});

router.on("/skills/:id", ({ data }) => {
  updateNav("/skills");
  updateBreadcrumbs(`/skills/${data!.id}`);
  void mount(`/api/runtime/view/skills/detail/${data!.id}`);
});

router.on("/skills/:id/:file", ({ data }) => {
  updateNav("/skills");
  updateBreadcrumbs(`/skills/${data!.id}/${data!.file}`);
  void mount(`/api/runtime/view/skills/detail/${data!.id}?file=${encodeURIComponent(data!.file)}`);
});

router.on("/canvas/:doc", ({ data }) => {
  updateNav("/canvas");
  updateBreadcrumbs(`/canvas/${data!.doc}`);
  void mount(`/api/runtime/view/canvas?doc=${encodeURIComponent(data!.doc)}`);
});

router.on("/pipelines/pipeline/:name", ({ data }) => {
  updateNav("/pipelines");
  updateBreadcrumbs(`/pipelines/${data!.name}`);
  void mount(`/api/runtime/view/pipelines/pipeline/${encodeURIComponent(data!.name)}`);
});

router.on("/pipelines/history", () => {
  updateNav("/pipelines");
  updateBreadcrumbs(`/pipelines/history`);
  // Forward any query string so pagination/search survive shell nav.
  const qs = location.search ?? "";
  void mount(`/api/runtime/view/pipelines/history${qs}`);
});

router.on("/pipelines/run/:slug", ({ data }) => {
  updateNav("/pipelines");
  updateBreadcrumbs(`/pipelines/run/${data!.slug}`);
  void mount(`/api/runtime/view/pipelines/run/${encodeURIComponent(data!.slug)}`);
});

// 404
router.notFound(() => {
  updateNav("");
  updateBreadcrumbs(location.pathname);
  view.innerHTML = `
    <div style="display:flex;flex-direction:column;align-items:center;justify-content:center;height:100%;gap:var(--space-3)">
      <h2 style="font-family:var(--font-display);font-size:var(--text-xl);margin:0">Page not found</h2>
      <p style="color:var(--muted-foreground);margin:0">The route "${location.pathname}" doesn't exist.</p>
      <ship-button tone="accent" variant="solid" onclick="window.router.navigate('/')">Go home</ship-button>
    </div>`;
});

// Wire all data-route links (rail + any future nav)
document
  .querySelectorAll<HTMLAnchorElement>("[data-route]")
  .forEach((a) => {
    a.addEventListener("click", (e) => {
      e.preventDefault();
      router.navigate(a.dataset.route!);
    });
    // Middle-click = new tab
    a.addEventListener("auxclick", (e) => {
      if (e.button === 1) {
        e.preventDefault();
        window.open(a.dataset.route!, "_blank");
      }
    });
  });

// ── Keyboard shortcuts ───────────────────────────────────────────────
document.addEventListener("keydown", (e) => {
  const meta = e.metaKey || e.ctrlKey;

  // ⌘K — Command palette
  if (meta && e.key === "k") {
    e.preventDefault();
    toggleCmdk();
    return;
  }

  // Escape — close palette if open
  if (e.key === "Escape" && isCommandPaletteOpen()) {
    e.preventDefault();
    toggleCmdk();
    return;
  }

  // ⌘B — Toggle rail
  if (meta && e.key === "b") {
    e.preventDefault();
    document.getElementById("toggle-rail")?.click();
  }

  // ⌘/ — Toggle drawer (chat)
  if (meta && e.key === "/") {
    e.preventDefault();
    toggleDrawer();
  }

  // ⌘J — Toggle terminal panel
  if (meta && e.key === "j") {
    e.preventDefault();
    document.body.classList.toggle("terminal-open");
  }

  // F11 — Fullscreen
  if (e.key === "F11") {
    e.preventDefault();
    if (document.fullscreenElement) {
      document.exitFullscreen();
    } else {
      document.documentElement.requestFullscreen();
    }
  }
});

// ── Status bar — live daemon data ────────────────────────────────────
let daemonConnected = false;

async function updateStatus() {
  statusTheme.textContent = getThemeName();
  try {
    const res = await fetch("/api/runtime/workspaces", {
      credentials: SAME_ORIGIN ? "include" : "omit",
      headers: !SAME_ORIGIN && TOKEN ? { Authorization: `Bearer ${TOKEN}` } : {},
    });
    if (res.ok) {
      const envelope = await res.json();
      const ws = envelope?.data?.workspaces?.[0];
      statusBranch.textContent = ws?.name || ws?.id || "no workspace";
      statusAgents.textContent = ws?.active_agent ? `agent: ${ws.active_agent}` : "";
      statusDot.classList.remove("disconnected");
      daemonConnected = true;
    } else {
      markDisconnected();
    }
  } catch {
    markDisconnected();
  }
}

function markDisconnected() {
  if (daemonConnected) {
    statusDot.classList.add("disconnected");
    statusBranch.textContent = "offline";
    statusAgents.textContent = "";
    daemonConnected = false;
  }
}

// Poll status every 15s
setInterval(updateStatus, 15000);

// ── Expose globals ───────────────────────────────────────────────────
(window as any).router = router;
(window as any).setShipTheme = setTheme;

// ── Boot ─────────────────────────────────────────────────────────────
router.resolve();
initThemeLoader();
initSignalInspector();
updateStatus();

// ⌘K trigger button
document.getElementById("cmdk-open")?.addEventListener("click", () => {
  toggleCmdk();
});

// Fullscreen button
document.getElementById("toggle-fullscreen")?.addEventListener("click", () => {
  if (document.fullscreenElement) {
    document.exitFullscreen();
  } else {
    document.documentElement.requestFullscreen();
  }
});
