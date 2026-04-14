import { mountDaemonFragment, openDaemonSSE } from "./mount";

const DAEMON_BASE = (import.meta.env.VITE_DAEMON_URL as string) ?? "http://127.0.0.1:9316";
const TOKEN = (import.meta.env.VITE_DAEMON_TOKEN as string) ?? "";

if (!TOKEN) {
  console.warn(
    "[shell] VITE_DAEMON_TOKEN not set. " +
      "Export it from ~/.ship/daemon.token before running `pnpm dev`.",
  );
}

const view = document.getElementById("view") as HTMLElement;
const drawerContent = document.getElementById("drawer-content") as HTMLElement;
const drawerBtn = document.getElementById("toggle-drawer") as HTMLButtonElement;

type Route = { path: string; mount: string; sse?: string };

const routes: Route[] = [
  { path: "/", mount: "/api/runtime/view/agents" },
  { path: "/pipelines", mount: "/api/runtime/view/pipelines", sse: "/api/runtime/view/pipelines/sse" },
  { path: "/chat", mount: "/api/runtime/view/chat", sse: "/api/runtime/view/chat/sse" },
  { path: "/events", mount: "/api/runtime/view/events", sse: "/api/runtime/view/events/sse" },
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
  const body = new URLSearchParams(new FormData(form) as unknown as Record<string, string>);
  const res = await fetch(url, {
    method: form.method || "POST",
    credentials: "omit",
    headers: {
      Authorization: `Bearer ${TOKEN}`,
      "Content-Type": "application/x-www-form-urlencoded",
      Accept: "text/html",
    },
    body: body.toString(),
  });
  const html = await res.text();
  const target = document.getElementById(form.dataset.target ?? "view") ?? view;
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
