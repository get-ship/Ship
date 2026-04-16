import { mountDaemonFragment, openDaemonSSE, isSameOrigin } from "./mount";

// Daemon templates emit absolute URLs (http://127.0.0.1:9315/api/...) so they
// work under iframe embedding. In fragment-mount, we want those requests to
// ride the Vite proxy (dev) or same-origin cookie (when daemon serves the
// shell). Rewrite daemon-origin URLs to relative at the fetch boundary so
// datastar, <form action>, and <a href> all Just Work without each template
// needing to know it's inside the shell.
const DAEMON_ORIGINS = new Set([
  "http://127.0.0.1:9315",
  "http://localhost:9315",
]);
function rewriteDaemonUrl(input) {
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
window.fetch = (input, init) => nativeFetch(rewriteDaemonUrl(input), init);

const NativeEventSource = window.EventSource;
window.EventSource = function (url, opts) {
  return new NativeEventSource(rewriteDaemonUrl(url), {
    withCredentials: true,
    ...(opts ?? {}),
  });
};
window.EventSource.CONNECTING = NativeEventSource.CONNECTING;
window.EventSource.OPEN = NativeEventSource.OPEN;
window.EventSource.CLOSED = NativeEventSource.CLOSED;

// Same-origin detection: when served by the daemon under /shell/, the daemon
// base URL either matches window.location.origin or is deliberately empty so
// fetches stay relative. In that mode we drop Bearer entirely and ride the
// ship_session cookie.
// Default: empty base → fetches are relative → Vite dev proxy forwards /api/*
// to the real daemon and injects the Bearer token server-side. Production /
// daemon-served path also uses empty base (same-origin under /shell/).
// Set VITE_DAEMON_URL to a full URL only if you need cross-origin direct.
const RAW_BASE = (import.meta.env.VITE_DAEMON_URL as string) ?? "";
const SAME_ORIGIN = isSameOrigin(RAW_BASE);
const DAEMON_BASE = SAME_ORIGIN ? "" : RAW_BASE;
const TOKEN = SAME_ORIGIN ? "" : ((import.meta.env.VITE_DAEMON_TOKEN as string) ?? "");

const view = document.getElementById("ship-view-root") as HTMLElement;
const drawerContent = document.getElementById("drawer-content") as HTMLElement;
const drawerBtn = document.getElementById("toggle-drawer") as HTMLButtonElement;
const themeBtn = document.getElementById("toggle-theme") as HTMLButtonElement;

// Theme — boot-time sync already applied .dark if needed (see index.html).
// This wires the toggle button + persists the choice.
themeBtn.addEventListener("click", () => {
  const isDark = document.documentElement.classList.toggle("dark");
  localStorage.setItem("ship-shell-theme", isDark ? "dark" : "light");
});

type Route = { path: string; mount: string; sse?: string };

const routes: Route[] = [
  { path: "/", mount: "/api/runtime/view/pipelines", sse: "/api/runtime/view/pipelines/sse" },
  { path: "/pipelines", mount: "/api/runtime/view/pipelines", sse: "/api/runtime/view/pipelines/sse" },
  { path: "/specs", mount: "/api/runtime/view/specs", sse: "/api/runtime/view/specs/sse" },
  { path: "/git", mount: "/api/runtime/view/git", sse: "/api/runtime/view/git/sse" },
  { path: "/settings", mount: "/api/runtime/view/settings", sse: "/api/runtime/view/settings/sse" },
  { path: "/agents", mount: "/api/runtime/view/agents" },
  { path: "/skills", mount: "/api/runtime/view/skills" },
  { path: "/chat", mount: "/api/runtime/view/chat", sse: "/api/runtime/view/chat/sse" },
  { path: "/events", mount: "/api/runtime/view/events", sse: "/api/runtime/view/events/sse" },
  { path: "/data", mount: "/api/runtime/view/data", sse: "/api/runtime/view/data/sse" },
];

let currentSSE: EventSource | null = null;
function closeSSE() {
  if (currentSSE) {
    currentSSE.close();
    currentSSE = null;
  }
}

async function renderRoute(r: Route) {
  closeSSE();
  await mountDaemonFragment(view, r.mount, { token: TOKEN, base: DAEMON_BASE });
  if (r.sse) {
    currentSSE = openDaemonSSE(r.sse, { token: TOKEN, base: DAEMON_BASE });
    // Datastar normally consumes SSE frames directly against the mounted
    // fragment. Here we just log events so the POC is observable.
    currentSSE.onmessage = (ev) => console.log("[sse]", ev.data);
    currentSSE.onerror = () => console.warn("[sse] error — see findings §3");
  }
}

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
  const body = new URLSearchParams(new FormData(form) as unknown as Record<string, string>);
  const res = await fetch(url, {
    method: form.method || "POST",
    credentials: SAME_ORIGIN ? "include" : "omit",
    headers,
    body: body.toString(),
  });
  const html = await res.text();
  const target = document.getElementById(form.dataset.target ?? "ship-view-root") ?? view;
  target.innerHTML = html;
}

function match(path: string): Route {
  return routes.find((r) => r.path === path) ?? routes[0];
}

async function navigate(path: string) {
  history.pushState({}, "", path);
  updateNav();
  await renderRoute(match(path));
}

function updateNav() {
  document.querySelectorAll<HTMLAnchorElement>("header a[data-route]").forEach((a) => {
    a.classList.toggle("active", a.dataset.route === location.pathname);
  });
}

document.querySelectorAll<HTMLAnchorElement>("header a[data-route]").forEach((a) => {
  a.addEventListener("click", (e) => {
    e.preventDefault();
    void navigate(a.dataset.route!);
  });
});

window.addEventListener("popstate", () => {
  updateNav();
  void renderRoute(match(location.pathname));
});

// Drawer — multi-mount: a second daemon fragment alongside the main region.
drawerBtn.addEventListener("click", async () => {
  document.body.classList.toggle("drawer-open");
  if (document.body.classList.contains("drawer-open")) {
    await mountDaemonFragment(drawerContent, "/api/runtime/view/events", {
      token: TOKEN,
      base: DAEMON_BASE,
    });
  }
});

updateNav();
void renderRoute(match(location.pathname));
