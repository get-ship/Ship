import Navigo from "navigo";
import { mountDaemonFragment, isSameOrigin } from "./mount";
import { initSignalInspector } from "./signal-inspector";
import { toggle as toggleCmdk, isCommandPaletteOpen } from "./command-palette";
import { installErrorReporter } from "./error-reporter";

// Install as early as possible so any boot-time throw gets captured.
installErrorReporter();

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

// ── Theme picker ─────────────────────────────────────────────────────
import {
  THEMES,
  applyTheme,
  persistTheme,
  readStoredTheme,
  isValidTheme,
} from "@ship/set/themes";

function mountThemePicker() {
  const root = document.getElementById("theme-picker");
  if (!root) return;
  const grid = root.querySelector<HTMLElement>(".theme-grid");
  if (!grid) return;

  // Flat grid — every Ship theme is a full aesthetic bundle, so there
  // are no tiers. Pick one.
  const card = (t: (typeof THEMES)[number]) => `
    <button type="button" class="theme-card" role="option"
      data-theme-card="${t.id}" data-dark="${t.dark}"
      aria-label="${t.label}"
      style="
        --swatch-bg: ${t.swatches[0]};
        --swatch-card: ${t.swatches[1]};
        --swatch-primary: ${t.swatches[2]};
        --swatch-accent: ${t.swatches[3]};
      ">
      <div class="theme-card-swatches">
        <span style="background:${t.swatches[0]}"></span>
        <span style="background:${t.swatches[1]}"></span>
        <span style="background:${t.swatches[2]}"></span>
        <span style="background:${t.swatches[3]}"></span>
      </div>
      <div class="theme-card-name">${t.label}</div>
    </button>`;
  grid.innerHTML = `
    <section class="theme-group" aria-label="Themes">
      <div class="theme-group-cards">${THEMES.map(card).join("")}</div>
    </section>`;

  const setActive = (id: string) => {
    grid.querySelectorAll<HTMLElement>("[data-theme-card]").forEach((c) => {
      c.setAttribute(
        "aria-selected",
        c.dataset.themeCard === id ? "true" : "false",
      );
    });
  };

  const initial = readStoredTheme();
  applyTheme(initial);
  setActive(initial);

  grid.addEventListener("click", (e) => {
    const card = (e.target as HTMLElement).closest<HTMLElement>(
      "[data-theme-card]",
    );
    if (!card) return;
    const id = card.dataset.themeCard;
    if (!isValidTheme(id)) return;
    const next = applyTheme(id);
    persistTheme(next);
    setActive(next);
    document.dispatchEvent(
      new CustomEvent("ship-theme-change", { detail: { id: next } }),
    );
    (root as any).hide?.();
  });

  document.addEventListener("ship-theme-change", (e) => {
    const id = (e as CustomEvent<{ id: string }>).detail?.id;
    if (isValidTheme(id)) {
      setActive(id);
      const status = document.getElementById("status-theme");
      if (status) status.textContent = id;
    }
  });
}

mountThemePicker();

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
      await mountDaemonFragment(drawerContent, "/api/v/chat", {
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
//
// Cross-fade strategy: the OLD root stays visible during the fetch (no blank
// interval). Once the new content has settled:
//  1. Insert the new root (opacity 0 via ship-content-entering).
//  2. Mark the old root as leaving (ship-content-leaving → opacity 0).
//  3. After ~140ms remove the old root and reveal the new one.
// Respects prefers-reduced-motion (CSS collapses transitions to none).
async function mount(endpoint: string) {
  // Keep a reference to the OLD root — it stays in the DOM during fetch.
  const old = document.getElementById("ship-view-root");
  const parent = old?.parentElement ?? document.body;

  // Build the new root but don't insert it yet.
  const newView = document.createElement("main");
  newView.id = "ship-view-root-next";
  newView.className = "ship-content ship-content-entering";
  // Position it out of flow until content is ready.
  newView.style.position = "absolute";
  newView.style.inset = "0";
  newView.style.zIndex = "-1";
  newView.style.pointerEvents = "none";

  // Attach alongside old so Datastar can see both in the DOM, then fetch.
  if (old) {
    parent.insertBefore(newView, old.nextSibling);
  } else {
    parent.appendChild(newView);
  }

  try {
    await mountDaemonFragment(newView, endpoint, {
      token: TOKEN,
      base: DAEMON_BASE,
    });
  } catch {
    newView.innerHTML = `<div class="mount-error">Failed to load view.\nCheck that the daemon is running.</div>`;
  }

  // Content landed. Now do the visual swap atomically.
  // Clear old root's id first to avoid duplicate ids during the brief overlap.
  if (old) old.id = "ship-view-root-leaving";
  newView.id = "ship-view-root";
  newView.style.position = "";
  newView.style.inset = "";
  newView.style.zIndex = "";
  newView.style.pointerEvents = "";
  view = newView;

  // Fade in new view.
  requestAnimationFrame(() => {
    newView.classList.remove("ship-content-entering");
  });

  // Fade out old view and remove after transition completes.
  if (old && old !== newView) {
    old.classList.add("ship-content-leaving");
    // Remove old root after the leave transition. Use a slightly longer
    // timeout than the CSS transition (140ms) to ensure cleanup.
    setTimeout(() => {
      if (old.parentElement) old.parentElement.removeChild(old);
    }, 180);
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
const ROUTE_DISPLAY_NAMES: Record<string, string> = {
  collections: "Collections",
  dashboard: "Dashboard",
  pipelines: "Pipelines",
  agents: "Agents",
  skills: "Skills",
  settings: "Settings",
  git: "Git",
  events: "Events",
  canvas: "Canvas",
  overview: "Overview",
};

function displayName(segment: string): string {
  return ROUTE_DISPLAY_NAMES[segment] ?? segment;
}

function updateBreadcrumbs(path: string) {
  const parts = path.split("/").filter(Boolean);
  if (parts.length === 0) parts.push("dashboard");
  const crumbs = parts.map((part, i) => {
    const isLast = i === parts.length - 1;
    const href = "/" + parts.slice(0, i + 1).join("/");
    const label = displayName(part);
    if (isLast) {
      return `<span class="current" id="breadcrumbs-title">${label}</span>`;
    }
    return `<a onclick="event.preventDefault();window.router.navigate('${href}')">${label}</a><span class="chevron">›</span>`;
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

// Retry hook exposed to the error-state fallback rendered by mount.ts
// when a view fetch fails. mount.ts is a lower-level module and can't
// reach the router directly; this indirection keeps the boundary tidy.
(window as unknown as { __shipRetryMount?: (endpoint: string) => void }).__shipRetryMount = (endpoint) => {
  void mount(endpoint);
};

function viewRoute(path: string, endpoint: string) {
  router.on(path, () => {
    updateNav(path);
    void mount(endpoint);
  });
}

viewRoute("/", "/api/v/dashboard");
viewRoute("/pipelines", "/api/v/pipelines");
viewRoute("/specs", "/api/v/specs");
viewRoute("/git", "/api/v/git");
viewRoute("/settings", "/api/v/settings");
viewRoute("/agents", "/api/v/agents");
viewRoute("/skills", "/api/v/skills");
viewRoute("/chat", "/api/v/chat");
viewRoute("/events", "/api/v/events");
viewRoute("/collections", "/api/v/collections");
viewRoute("/dashboard", "/api/v/dashboard");
viewRoute("/canvas", "/api/v/canvas");
viewRoute("/overview", "/api/v/overview");

// Dynamic routes
router.on("/collections/:collection", ({ data }) => {
  updateNav("/collections");
  updateBreadcrumbs(`/collections/${data!.collection}`);
  void mount(
    `/api/v/collections?collection=${encodeURIComponent(data!.collection)}`,
  );
});

// Static sub-routes under /agents must be registered BEFORE the
// generic `/agents/:id` so Navigo doesn't match "create" as an id.
router.on("/agents/create", () => {
  updateNav("/agents");
  updateBreadcrumbs("/agents/create");
  void mount("/api/v/agents/create");
});

// Detail sub-mode: wire/canvas view (also served as the default detail).
router.on("/agents/:id/wire", ({ data }) => {
  updateNav("/agents");
  updateBreadcrumbs(`/agents/${data!.id}/wire`);
  void mount(`/api/v/agents/${data!.id}/wire`);
});

// Detail sub-mode: classic panels layout (skills / MCP / permissions / rules).
router.on("/agents/:id/settings", ({ data }) => {
  updateNav("/agents");
  updateBreadcrumbs(`/agents/${data!.id}/settings`);
  void mount(`/api/v/agents/${data!.id}/settings`);
});

router.on("/agents/:id", ({ data }) => {
  updateNav("/agents");
  updateBreadcrumbs(`/agents/${data!.id}`);
  void mount(`/api/v/agents/${data!.id}`);
});

router.on("/skills/:id", ({ data }) => {
  updateNav("/skills");
  updateBreadcrumbs(`/skills/${data!.id}`);
  void mount(`/api/view/skills/detail/${data!.id}`);
});

router.on("/skills/:id/:file", ({ data }) => {
  updateNav("/skills");
  updateBreadcrumbs(`/skills/${data!.id}/${data!.file}`);
  void mount(`/api/view/skills/detail/${data!.id}?file=${encodeURIComponent(data!.file)}`);
});

router.on("/canvas/:doc", ({ data }) => {
  updateNav("/canvas");
  updateBreadcrumbs(`/canvas/${data!.doc}`);
  void mount(`/api/view/canvas?doc=${encodeURIComponent(data!.doc)}`);
});

router.on("/pipelines/focus/:name", ({ data }) => {
  updateNav("/pipelines");
  updateBreadcrumbs(`/pipelines/${data!.name}`);
  void mount(`/api/v/pipelines/focus/${encodeURIComponent(data!.name)}`);
});

router.on("/pipelines/history", () => {
  updateNav("/pipelines");
  updateBreadcrumbs(`/pipelines/history`);
  // Forward any query string so pagination/search survive shell nav.
  const qs = location.search ?? "";
  void mount(`/api/v/pipelines/history${qs}`);
});

router.on("/pipelines/run/:slug", ({ data }) => {
  updateNav("/pipelines");
  updateBreadcrumbs(`/pipelines/run/${data!.slug}`);
  void mount(`/api/v/pipelines/run/${encodeURIComponent(data!.slug)}`);
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
  statusTheme.textContent = document.documentElement.dataset.theme ?? "ship-light";
  try {
    const res = await fetch("/api/workspaces", {
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
(window as any).setShipTheme = (id: string) => {
  const applied = applyTheme(id);
  persistTheme(applied);
  document.dispatchEvent(new CustomEvent("ship-theme-change", { detail: { id: applied } }));
};

// ── Boot ─────────────────────────────────────────────────────────────
router.resolve();
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
