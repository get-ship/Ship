import { mountDaemonFragment, openDaemonSSE } from "./mount";

const DAEMON_BASE = (import.meta.env.VITE_DAEMON_URL as string) ?? "http://127.0.0.1:51742";
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

type Route = { path: string; render: () => Promise<void> };

let currentSSE: EventSource | null = null;
function closeSSE() {
  if (currentSSE) {
    currentSSE.close();
    currentSSE = null;
  }
}

const routes: Route[] = [
  {
    path: "/",
    render: async () => {
      closeSSE();
      await mountDaemonFragment(view, "/api/runtime/view/poc-home/raw", {
        token: TOKEN,
        base: DAEMON_BASE,
      });
    },
  },
  {
    path: "/pipelines",
    render: async () => {
      closeSSE();
      await mountDaemonFragment(view, "/api/runtime/view/poc-pipelines/raw", {
        token: TOKEN,
        base: DAEMON_BASE,
      });
    },
  },
  {
    path: "/chat",
    render: async () => {
      closeSSE();
      await mountDaemonFragment(view, "/api/runtime/view/poc-chat/raw", {
        token: TOKEN,
        base: DAEMON_BASE,
      });
      // Exercise #3 — SSE update into the mounted fragment. The fragment
      // contains an element with id="sse-target" that we push messages into.
      currentSSE = openDaemonSSE("/api/runtime/view/poc-chat/events", {
        token: TOKEN,
        base: DAEMON_BASE,
      });
      currentSSE.onmessage = (ev) => {
        const el = document.getElementById("sse-target");
        if (el) el.insertAdjacentHTML("afterbegin", `<li>${ev.data}</li>`);
      };
      currentSSE.onerror = () => {
        const el = document.getElementById("sse-target");
        if (el)
          el.insertAdjacentHTML(
            "afterbegin",
            `<li style="color:#a00">[SSE error — see findings §3]</li>`,
          );
      };
    },
  },
  {
    path: "/form",
    render: async () => {
      closeSSE();
      await mountDaemonFragment(view, "/api/runtime/view/poc-form/raw", {
        token: TOKEN,
        base: DAEMON_BASE,
      });
      // Intercept form submits inside the mounted fragment so the daemon
      // response becomes a fragment replacement. This mimics what
      // `<ship-form>` / datastar would do; we wire it manually here to
      // keep the POC dependency-light.
      view.addEventListener("submit", onFragmentSubmit, { once: true });
    },
  },
];

async function onFragmentSubmit(ev: Event) {
  const form = ev.target as HTMLFormElement;
  if (!(form instanceof HTMLFormElement)) return;
  ev.preventDefault();
  const body = new URLSearchParams(new FormData(form) as unknown as Record<string, string>);
  const res = await fetch(form.action || form.getAttribute("data-action") || "", {
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
  await match(path).render();
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
  void match(location.pathname).render();
});

// Drawer — exercise #6: two daemon-served fragments mounted simultaneously.
drawerBtn.addEventListener("click", async () => {
  document.body.classList.toggle("drawer-open");
  if (document.body.classList.contains("drawer-open")) {
    await mountDaemonFragment(drawerContent, "/api/runtime/view/poc-drawer/raw", {
      token: TOKEN,
      base: DAEMON_BASE,
    });
  }
});

// Boot.
updateNav();
void match(location.pathname).render();
