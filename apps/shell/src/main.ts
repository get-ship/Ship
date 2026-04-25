import Navigo from "navigo";
import { mountDaemonFragment, isSameOrigin } from "./mount";
// Legacy command palette — retained for the scripted `/cmdk` fallback
// but no longer bound to ⌘K (claimed by <ship-command-bar>).
import { toggle as toggleCmdk } from "./command-palette";
import { installErrorReporter } from "./error-reporter";
// Side-effect import: registers `<ship-shell>` + mounts `window.ship`
// on first `connectedCallback`. The custom-element registration fires
// before the element in index.html upgrades, so the shell is ready by
// the time view code starts calling `window.ship.*`.
import "./chrome";
// Side-effect import: registers every `<ship-*>` web component in the
// same module graph as the shell chrome. Sharing the graph means a
// single Lit copy at runtime (no "Multiple versions" warning) and
// removes the need for a daemon-served `/api/assets/ship-elements.js`
// from the SPA bootstrap.
import "@ship/set/web-components";

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
  syncThemeFromSettings,
} from "@ship/set/themes";

// Reconcile local theme cache with daemon settings on boot. Fire and
// forget — the pre-paint boot script already applied the cached value;
// this just updates it if another device/browser changed the tier-2
// authoritative copy.
void syncThemeFromSettings().then((remote) => {
  if (remote && remote !== document.documentElement.dataset.theme) {
    applyTheme(remote);
    document.dispatchEvent(
      new CustomEvent("ship-theme-change", { detail: { id: remote } }),
    );
  }
});

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
    }
  });
}

// applyTheme() in @ship/set/themes always dispatches `ship:theme:change`
// — listen here to repaint the status bar regardless of caller (UI
// picker, programmatic `setShipTheme`, or external code calling
// applyTheme directly). Cleaner than a MutationObserver on dataset.
document.addEventListener("ship:theme:change", (e) => {
  const id = (e as CustomEvent<{ id: string }>).detail?.id;
  if (typeof id === "string" && statusTheme) statusTheme.textContent = id;
});

// Wait for <ship-nav-top> to render its light-DOM theme-picker host before
// wiring up the grid. firstUpdated on nav-top dispatches ship:nav-top:ready.
const _navTopReady = () => {
  mountThemePicker();
};
document
  .querySelector("ship-shell")
  ?.addEventListener("ship:nav-top:ready", _navTopReady, { once: true });
// Fallback: if nav-top already rendered (e.g. HMR), wire immediately.
if (document.getElementById("theme-picker")) mountThemePicker();

// ── Rail toggle ──────────────────────────────────────────────────────
// <ship-nav-top>'s own @click on #toggle-rail dispatches
// ship:rail:toggle; <ship-shell> owns the body class toggle. Nothing
// to wire here.

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

// Chat drawer: nav-top button and `window.ship.chat.*` dispatch events
// on the shell; this app owns the actual toggle because it uses
// mountDaemonFragment which holds auth context.
const shellEl = document.querySelector("ship-shell");
shellEl?.addEventListener("ship:chat:toggle", () => {
  void toggleDrawer();
});
shellEl?.addEventListener("ship:chat:open", () => {
  if (!document.body.classList.contains("drawer-open")) void toggleDrawer();
});
shellEl?.addEventListener("ship:chat:close", () => {
  if (document.body.classList.contains("drawer-open")) void toggleDrawer();
});

// ── Panel dock toggle ───────────────────────────────────────────────
const dock = document.getElementById("panel-dock")!;
dock.addEventListener("ship:dock:close", () => {
  document.body.classList.remove("dock-open");
});
// Clear → forward to the active pane via a `clear` method if present.
dock.addEventListener("ship:dock:clear", (ev) => {
  const tab = (ev as CustomEvent<{ tab: string }>).detail.tab;
  const pane = dock.querySelector(`[slot="${tab}"]`) as
    | (Element & { clear?: () => void; refresh?: () => void })
    | null;
  pane?.clear?.() ?? pane?.refresh?.();
});
// Pop-out → maximize the dock to fill the viewport (toggle a body class).
dock.addEventListener("ship:dock:popout", () => {
  document.body.classList.toggle("dock-popout");
});
document.querySelector("ship-shell")?.addEventListener("ship:dock:toggle", () => {
  document.body.classList.toggle("dock-open");
  maybeInitDockActive();
});

// Devtools packs load lazily on first activation of the tab that needs
// them. Tracked per pack so repeat activations don't re-inject.
const loadedPacks = new Set<string>();
function loadPack(src: string): void {
  if (loadedPacks.has(src)) return;
  loadedPacks.add(src);
  const s = document.createElement("script");
  s.type = "module";
  s.src = `/api/assets/${src}`;
  document.head.appendChild(s);
}

// ── Agent task completion notifications ─────────────────────────────
// The `agent` kernel namespace emits `agent.task.complete` when a
// creation or iteration finishes. Toast + click-through routes to
// the created artifact (e.g. the new view).
window.ship?.subscribe?.("agent", (ev) => {
  if (ev.event_type !== "agent.task.complete") return;
  const payload = ev.payload as {
    surface?: string;
    name?: string;
    route?: string;
  };
  const name = payload?.name ?? "task";
  window.ship?.notify?.(`${name} is ready`, "success");
  // Also fire a public shell event that the chat drawer listens on
  // to clear its working-state pill.
  document.querySelector("ship-shell")?.dispatchEvent(
    new CustomEvent("ship:create:done", { detail: payload, bubbles: true }),
  );
  if (payload?.route) {
    // Click-through: toast-host doesn't expose a click API yet, so
    // auto-navigate after a short delay to let the user see the toast.
    setTimeout(() => {
      if (payload.route) window.ship?.navigate(payload.route);
    }, 800);
  }
});

dock.addEventListener("ship:dock:activate", (ev) => {
  const tab = (ev as CustomEvent<{ tab: string }>).detail.tab;
  if (tab === "source") {
    loadPack("ship-code-editor.js");
    const el = dock.querySelector("ship-devtools-source") as
      | (HTMLElement & { refresh?: () => void })
      | null;
    el?.refresh?.();
  }
  if (tab === "terminal") {
    loadPack("ship-terminal.js");
    void initDockTerminal();
  }
});

// ── Dock terminal ↔ PTY shell session ─────────────────────────────
// First activation spawns a kind=shell session, subscribes to its
// output SSE, and wires xterm input back via /input. One shared
// session per browser; tab switches reuse it.
let dockTerminalSessionId: string | null = null;
let dockTerminalES: EventSource | null = null;

function maybeInitDockActive(): void {
  if (!document.body.classList.contains("dock-open")) return;
  const active = dock.getAttribute("active-tab") ?? "terminal";
  if (active === "terminal") {
    loadPack("ship-terminal.js");
    void initDockTerminal();
  }
}

async function initDockTerminal(): Promise<void> {
  const xterm = document.getElementById("shell-terminal") as
    | (HTMLElement & {
        write?: (s: string) => void;
        terminal?: { onData?: (cb: (s: string) => void) => void };
      })
    | null;
  if (!xterm) return;
  if (!dockTerminalSessionId) {
    try {
      const resp = await fetch("/api/sessions/spawn", {
        method: "POST",
        credentials: "same-origin",
        headers: { "Content-Type": "application/json" },
        body: JSON.stringify({ workspace_id: "main", kind: "shell" }),
      });
      if (!resp.ok) return;
      const body = (await resp.json()) as { session_id: string };
      dockTerminalSessionId = body.session_id;
    } catch {
      return;
    }
  }
  // Wait for xterm to be ready, then wire output + input.
  const wireWhenReady = (): void => {
    if (!xterm.write || !xterm.terminal?.onData) {
      xterm.addEventListener("ship-ready", wireWhenReady, { once: true });
      return;
    }
    if (dockTerminalES) return; // already wired
    dockTerminalES = new EventSource(
      `/api/sessions/${dockTerminalSessionId}/stream`,
      { withCredentials: true },
    );
    dockTerminalES.addEventListener("bytes", (ev) => {
      const text = (ev as MessageEvent).data;
      xterm.write?.(text);
    });
    // Queue keystrokes + flush serially so multi-char input arrives
    // in-order. xterm fires `ship-data` per key; parallel fetches
    // let bursts land out-of-order and corrupt input to the shell.
    let inputBuffer = "";
    let inputInflight = false;
    const flushInput = async (): Promise<void> => {
      if (inputInflight || !inputBuffer || !dockTerminalSessionId) return;
      inputInflight = true;
      const batch = inputBuffer;
      inputBuffer = "";
      try {
        await fetch(`/api/sessions/${dockTerminalSessionId}/input`, {
          method: "POST",
          credentials: "same-origin",
          headers: { "Content-Type": "application/json" },
          body: JSON.stringify({ data: batch }),
        });
      } catch {
        // On failure, requeue so the user's keystrokes aren't dropped.
        inputBuffer = batch + inputBuffer;
      } finally {
        inputInflight = false;
        if (inputBuffer) void flushInput();
      }
    };
    xterm.addEventListener("ship-data", (ev) => {
      const data = (ev as CustomEvent<{ data: string }>).detail?.data;
      if (!data || !dockTerminalSessionId) return;
      inputBuffer += data;
      void flushInput();
    });
  };
  wireWhenReady();
}

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

// Breadcrumbs moved to <ship-nav-top>; display-name mapping lives in
// packages/ship-shell/src/view-catalog.ts.

// ── Navigation broadcast ─────────────────────────────────────────────
// <ship-rail> highlights the active item; <ship-nav-top> re-renders
// breadcrumbs. Both listen for ship:navigate on <ship-shell>.
let lastNavPath = "/";
function updateNav(path: string) {
  document
    .querySelectorAll<HTMLAnchorElement>("[data-route]")
    .forEach((a) => {
      const route = a.dataset.route!;
      const active = path === route || path.startsWith(route + "/");
      a.classList.toggle("active", active);
    });
  document.querySelector("ship-shell")?.dispatchEvent(
    new CustomEvent("ship:navigate", {
      detail: { from: lastNavPath, to: path },
      bubbles: true,
    }),
  );
  lastNavPath = path;
}

// ── Router (Navigo) ──────────────────────────────────────────────────
const router = new Navigo("/");

// Register with <ship-shell> so `window.ship.navigate/back/current`
// delegates here. Has to happen after the element upgrades — we defer
// to the next microtask since customElements.define() ran via the
// `./chrome` side-effect import at the top of this file.
queueMicrotask(() => {
  const shell = document.querySelector("ship-shell") as
    | (HTMLElement & {
        registerRouter?: (a: {
          navigate: (p: string, o?: { replace?: boolean }) => void;
          back: () => void;
          current: () => { path: string; params: Record<string, string>; query: Record<string, string> };
        }) => void;
      })
    | null;
  shell?.registerRouter?.({
    navigate: (p) => {
      router.navigate(p);
    },
    back: () => history.back(),
    current: () => ({
      path: location.pathname,
      params: {},
      query: Object.fromEntries(new URLSearchParams(location.search)),
    }),
  });
});

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
viewRoute("/chats", "/api/v/chats");
viewRoute("/actors", "/api/v/actors");
viewRoute("/events", "/api/v/events");
viewRoute("/collections", "/api/v/collections");
viewRoute("/dashboard", "/api/v/dashboard");
viewRoute("/canvas", "/api/v/canvas");
viewRoute("/overview", "/api/v/overview");
viewRoute("/theme-studio", "/api/v/theme-studio");

// Dynamic routes
router.on("/collections/:collection", ({ data }) => {
  updateNav("/collections");
  void mount(
    `/api/v/collections?collection=${encodeURIComponent(data!.collection)}`,
  );
});

// Static sub-routes under /agents must be registered BEFORE the
// generic `/agents/:id` so Navigo doesn't match "create" as an id.
router.on("/agents/create", () => {
  updateNav("/agents");
  void mount("/api/v/agents/create");
});

// Detail sub-mode: wire/canvas view (also served as the default detail).
router.on("/agents/:id/wire", ({ data }) => {
  updateNav("/agents");
  void mount(`/api/v/agents/${data!.id}/wire`);
});

// Detail sub-mode: classic panels layout (skills / MCP / permissions / rules).
router.on("/agents/:id/settings", ({ data }) => {
  updateNav("/agents");
  void mount(`/api/v/agents/${data!.id}/settings`);
});

router.on("/agents/:id", ({ data }) => {
  updateNav("/agents");
  void mount(`/api/v/agents/${data!.id}`);
});

router.on("/skills/:id", ({ data }) => {
  updateNav("/skills");
  void mount(`/api/view/skills/detail/${data!.id}`);
});

router.on("/skills/:id/:file", ({ data }) => {
  updateNav("/skills");
  void mount(`/api/view/skills/detail/${data!.id}?file=${encodeURIComponent(data!.file)}`);
});

router.on("/canvas/:doc", ({ data }) => {
  updateNav("/canvas");
  void mount(`/api/view/canvas?doc=${encodeURIComponent(data!.doc)}`);
});

router.on("/pipelines/focus/:name", ({ data }) => {
  updateNav("/pipelines");
  void mount(`/api/v/pipelines/focus/${encodeURIComponent(data!.name)}`);
});

router.on("/pipelines/history", () => {
  updateNav("/pipelines");
  // Forward any query string so pagination/search survive shell nav.
  const qs = location.search ?? "";
  void mount(`/api/v/pipelines/history${qs}`);
});

router.on("/pipelines/run/:slug", ({ data }) => {
  updateNav("/pipelines");
  void mount(`/api/v/pipelines/run/${encodeURIComponent(data!.slug)}`);
});

// 404
router.notFound(() => {
  updateNav("");
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

  // ⌘K is claimed by <ship-command-bar>; its own keydown handler
  // focuses the bar from anywhere. main.ts no longer toggles the
  // legacy palette.

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

  // ⌘J — Toggle panel dock
  if (meta && e.key === "j") {
    e.preventDefault();
    document.body.classList.toggle("dock-open");
    maybeInitDockActive();
  }

  // ⌘⇧D — Focus devtools (opens dock + jumps to Source)
  if (meta && e.shiftKey && (e.key === "D" || e.key === "d")) {
    e.preventDefault();
    document.body.classList.add("dock-open");
    const dockEl = document.getElementById("panel-dock") as
      | (HTMLElement & { activate?: (t: string) => void })
      | null;
    dockEl?.activate?.("source");
  }

  // ⌘⇧E — Toggle edit mode
  if (meta && e.shiftKey && (e.key === "E" || e.key === "e")) {
    e.preventDefault();
    const on = document.body.classList.toggle("edit-mode");
    window.ship?.notify?.(
      on ? "Edit mode on — hover to highlight, click to annotate." : "Edit mode off.",
      "info",
    );
  }

  // ⌥1–⌥6 — Jump to dock tab N (only when dock is open).
  if (e.altKey && !meta && !e.shiftKey && /^[1-6]$/.test(e.key)) {
    if (document.body.classList.contains("dock-open")) {
      e.preventDefault();
      const idx = Number.parseInt(e.key, 10) - 1;
      const dockEl = document.getElementById("panel-dock") as
        | (HTMLElement & {
            tabs?: Array<{ id: string }>;
            activate?: (t: string) => void;
          })
        | null;
      const tab = dockEl?.tabs?.[idx]?.id;
      if (tab) dockEl?.activate?.(tab);
    }
  }

  // Escape — close dock if it has focus and active tab body is empty.
  if (e.key === "Escape" && document.body.classList.contains("dock-open")) {
    const dockEl = document.getElementById("panel-dock");
    if (dockEl && dockEl.contains(document.activeElement)) {
      e.preventDefault();
      document.body.classList.remove("dock-open");
    }
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
updateStatus();

// Legacy ship:command:open listener — kept as a non-shortcut way to
// raise the old palette while we migrate callers to the bar. Not
// wired to ⌘K anymore.
document
  .querySelector("ship-shell")
  ?.addEventListener("ship:command:open", () => {
    toggleCmdk();
  });

// Fullscreen button — handled inline by <ship-nav-top>.
