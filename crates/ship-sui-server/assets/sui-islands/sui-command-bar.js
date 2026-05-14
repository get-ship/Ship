// packages/ui-components/src/islands/sui-command-bar/sui-command-bar.sui
import { adoptSuiSheets, attachRef, bindIf, bindText, component, effect as effect2, on, prop, signal as signal2 } from "@ship/sui-runtime";

// packages/ui-components/src/islands/sui-command-bar/sui-command-bar.controller.ts
import { controller, signal, derived, effect, ref, shared } from "@ship/sui-runtime";

// packages/ui-components/src/islands/sui-command-bar/types.ts
var CREATE_SURFACES = [
  { id: "view", label: "View", icon: "file", shapes: ["list", "form", "detail", "dashboard", "tabs", "blank"], shapeLabel: "Starting shape" },
  { id: "skill", label: "Skill", icon: "sparkles", shapes: ["task", "reference", "workflow"], shapeLabel: "Skill kind" },
  { id: "agent", label: "Agent", icon: "users", shapes: ["specialist", "generalist", "reviewer"], shapeLabel: "Agent kind" },
  { id: "pipeline", label: "Pipeline", icon: "play", shapes: ["sequential", "fanout", "review-gated"], shapeLabel: "Topology" },
  { id: "trigger", label: "Trigger", icon: "zap", shapes: ["webhook", "schedule", "file-change"], shapeLabel: "Kind" },
  { id: "theme", label: "Theme", icon: "palette", shapes: ["light", "dark", "brand-accent", "high-contrast"], shapeLabel: "Starting shape" }
];
var NAV_DESTINATIONS = [
  { label: "Dashboard", path: "/dashboard", icon: "sparkles", keywords: "home overview" },
  { label: "Pipelines", path: "/pipelines", icon: "play", keywords: "workflows runs jobs" },
  { label: "Agents", path: "/agents", icon: "users", keywords: "configure profiles" },
  { label: "Skills", path: "/skills", icon: "zap", keywords: "browse edit files" },
  { label: "Git", path: "/git", icon: "git", keywords: "status diff commit branch" },
  { label: "Collections", path: "/collections", icon: "folder", keywords: "data items query" },
  { label: "Canvas", path: "/canvas", icon: "image", keywords: "docs visual feedback" },
  { label: "Events", path: "/events", icon: "sparkles", keywords: "stream log kernel" },
  { label: "Settings", path: "/settings", icon: "settings", keywords: "config theme provider" },
  { label: "Chat", path: "/chat", icon: "message-square-plus", keywords: "agent conversation" },
  { label: "Specs", path: "/specs", icon: "file", keywords: "features design" },
  { label: "Theme Studio", path: "/theme-studio", icon: "palette", keywords: "design tokens" },
  { label: "Chats (Mission Control)", path: "/chats", icon: "messages-square", keywords: "sessions active" }
];
var SLASH_COMMANDS = [
  { key: "new", label: "/new", hint: "Scaffold a view, skill, agent, …" },
  { key: "chats", label: "/chats", hint: "Open Mission Control" },
  { key: "settings", label: "/settings", hint: "Jump to settings" },
  { key: "lint", label: "/lint", hint: "Run lint on the current view" },
  { key: "end", label: "/end", hint: "End the current session" },
  { key: "close", label: "/close", hint: "Close the command bar" }
];
var KEBAB_RE = /^[a-z][a-z0-9-]{0,48}$/;
function isKebab(s) {
  return KEBAB_RE.test(s);
}
function detectMode(value, fallback) {
  if (value.startsWith("/"))
    return "command";
  if (value.startsWith(">"))
    return "find";
  if (value.startsWith("+"))
    return "create";
  if (value.trim().length === 0)
    return fallback;
  if (/(^|\s)@[^\s@]*$/.test(value))
    return "mention";
  return "ai";
}
async function navigateTo(path) {
  const w = window;
  if (w.ship?.navigate) {
    w.ship.navigate(path);
    return;
  }
  history.pushState({}, "", path);
  document.dispatchEvent(new CustomEvent("sui:navigate", { detail: { to: path } }));
}

// packages/ui-components/src/islands/sui-command-bar/sui-command-bar.controller.ts
var sui_command_bar_controller_default = controller({
  scope: (init) => {
    const initAny = init;
    const open = shared(initAny.open);
    const mode = shared(initAny.mode);
    const value = shared(signal(""));
    const session = shared(signal(null));
    const agents = shared(signal([]));
    const providers = shared(signal([]));
    const allSessions = shared(signal([]));
    const views = shared(signal([]));
    const selectedIdx = shared(signal(0));
    const sessionPickerOpen = shared(signal(false));
    const spawnAgent = shared(signal(""));
    const spawnProvider = shared(signal(""));
    const spawning = shared(signal(false));
    const spawnError = shared(signal(null));
    const thread = shared(signal([]));
    const workspaceId = shared(signal("main"));
    const createSurface = shared(signal("view"));
    const createName = shared(signal(""));
    const createPurpose = shared(signal(""));
    const createShape = shared(signal("list"));
    const createSubmitting = shared(signal(false));
    const createError = shared(signal(null));
    const isChat = shared(derived(() => mode() === "ai" && session() !== null));
    const dialogRef = ref(null);
    return {
      open,
      mode,
      value,
      session,
      agents,
      providers,
      allSessions,
      views,
      selectedIdx,
      sessionPickerOpen,
      spawnAgent,
      spawnProvider,
      spawning,
      spawnError,
      thread,
      workspaceId,
      createSurface,
      createName,
      createPurpose,
      createShape,
      createSubmitting,
      createError,
      isChat,
      dialogRef
    };
  },
  derived: {
    triggerLabel: (s) => {
      const sess = s.session();
      return sess ? `Ask ${sess.agent_id ?? "Ship"} or jump to…` : "Ask Ship or jump to…";
    },
    dotBg: (s) => {
      const sess = s.session();
      if (sess?.status === "thinking")
        return "var(--warning,#d97706)";
      if (sess)
        return "var(--success,#16a34a)";
      return "var(--muted-foreground)";
    }
  },
  on: {
    openFind: (s) => {
      s.mode = "find";
      s.value = "";
      s.selectedIdx = 0;
      s.open = true;
    }
  },
  mount: (state, host) => {
    const st = state;
    const open = st.open;
    const mode = st.mode;
    const value = st.value;
    const session = st.session;
    const agents = st.agents;
    const providers = st.providers;
    const allSessions = st.allSessions;
    const views = st.views;
    const selectedIdx = st.selectedIdx;
    const spawnAgent = st.spawnAgent;
    const spawnProvider = st.spawnProvider;
    const spawning = st.spawning;
    const spawnError = st.spawnError;
    const thread = st.thread;
    const workspaceId = st.workspaceId;
    const createSurface = st.createSurface;
    const createName = st.createName;
    const createPurpose = st.createPurpose;
    const createShape = st.createShape;
    const createSubmitting = st.createSubmitting;
    const createError = st.createError;
    const dialogRef = st.dialogRef?.();
    let streamAbort = null;
    let lastSpaceAt = 0;
    const stopOpenSync = effect(() => {
      const isOpen = open();
      host.setAttribute("data-state", isOpen ? "open" : "closed");
      if (isOpen) {
        if (typeof dialogRef?.show === "function")
          dialogRef.show();
        queueMicrotask(() => {
          const pal = host.shadowRoot?.querySelector("sui-command-palette");
          const inp = pal?.shadowRoot?.querySelector(".cb-search");
          inp?.focus();
        });
      } else {
        if (typeof dialogRef?.close === "function")
          dialogRef.close();
        host.dispatchEvent(new CustomEvent("sui-command:cancel", {
          detail: { mode: mode() },
          bubbles: true,
          composed: true
        }));
      }
    });
    host.show = (m, q) => {
      if (m)
        mode.set(m);
      if (typeof q === "string")
        value.set(q);
      open.set(true);
    };
    host.hide = () => open.set(false);
    host.toggle = (m) => {
      if (open()) {
        open.set(false);
      } else {
        if (m)
          mode.set(m);
        open.set(true);
      }
    };
    host.setMode = (m) => mode.set(m);
    function attachStream(sessionId) {
      detachStream();
      const ac = new AbortController;
      streamAbort = ac;
      const es = new EventSource(`/api/v1/sessions/${sessionId}/events`, { withCredentials: true });
      let currentAgentId = null;
      es.addEventListener("provider", (ev) => {
        let parsed;
        try {
          parsed = JSON.parse(ev.data);
        } catch {
          return;
        }
        switch (parsed.kind) {
          case "message": {
            const chunk = parsed.text ?? "";
            if (!chunk)
              return;
            if (currentAgentId) {
              thread.set(thread().map((m) => m.id === currentAgentId ? { ...m, text: m.text + chunk } : m));
            } else {
              const id = String(Date.now() + Math.random());
              currentAgentId = id;
              thread.set([...thread(), { id, sender: "agent", text: chunk }]);
            }
            break;
          }
          case "tool_use":
            thread.set([...thread(), { id: String(Date.now() + Math.random()), sender: "system", text: `→ tool: ${parsed.tool ?? "?"}` }]);
            break;
          case "turn_complete":
            currentAgentId = null;
            break;
        }
      });
      es.onerror = () => {
        if (!ac.signal.aborted)
          es.close();
      };
      ac.signal.addEventListener("abort", () => es.close());
    }
    function detachStream() {
      streamAbort?.abort();
      streamAbort = null;
    }
    const stopStreamSync = effect(() => {
      const s = session();
      if (s?.id)
        attachStream(s.id);
      else
        detachStream();
    });
    async function loadCurrentSession() {
      try {
        const resp = await fetch("/api/v1/pty-sessions", { credentials: "same-origin" });
        if (resp.ok) {
          const body = await resp.json();
          allSessions.set(body.sessions);
          const active = body.sessions.find((s) => s.status !== "ended" && s.agent_id);
          if (active && active.id !== session()?.id) {
            session.set(active);
            thread.set([]);
          } else if (!active) {
            session.set(null);
          }
        }
      } catch {}
      if (agents().length === 0)
        loadAgents();
      if (providers().length === 0)
        loadProviders();
      if (views().length === 0)
        loadViews();
    }
    async function loadAgents() {
      try {
        const resp = await fetch("/api/v1/agents", { credentials: "same-origin" });
        if (!resp.ok)
          return;
        const body = await resp.json();
        const list = (body.items ?? []).map((a) => ({ id: a.id }));
        agents.set(list);
        if (!spawnAgent() && list.length > 0)
          spawnAgent.set(list[0].id);
      } catch {}
    }
    async function loadProviders() {
      try {
        const resp = await fetch("/api/v1/providers/installed", { credentials: "same-origin" });
        if (!resp.ok)
          return;
        const body = await resp.json();
        providers.set(body.providers);
        if (!spawnProvider() && body.providers.length > 0)
          spawnProvider.set(body.providers[0]);
      } catch {}
    }
    async function loadViews() {
      try {
        const resp = await fetch("/api/v1/views", { credentials: "same-origin" });
        if (!resp.ok)
          return;
        const body = await resp.json();
        const list = body.data?.views ?? body.views ?? [];
        views.set(list.map((v) => v.name).sort());
      } catch {}
    }
    function getPaletteItems() {
      const pal = host.shadowRoot?.querySelector("sui-command-palette");
      return pal?.shadowRoot?.querySelectorAll(".cb-item");
    }
    function moveSelection(dir) {
      const items = getPaletteItems();
      if (!items?.length)
        return;
      selectedIdx.set(Math.max(0, Math.min(items.length - 1, selectedIdx() + dir)));
    }
    function confirmSelection() {
      const m = mode();
      const v = value();
      const items = getPaletteItems();
      const active = items?.[selectedIdx()];
      if (active) {
        const path = active.dataset["path"];
        if (path) {
          navigateTo(path);
          value.set("");
          open.set(false);
        }
        const cmd = active.dataset["cmd"];
        if (cmd) {
          value.set("");
          runSlashCommand("/" + cmd);
        }
        const mKind = active.dataset["mentionKind"];
        const mId = active.dataset["mentionId"];
        if (mKind && mId) {
          const ref2 = mKind === "view" ? `@${mId}` : `@${mKind}:${mId}`;
          value.set(v.replace(/@[^@\s]*$/, `${ref2} `));
          mode.set(detectMode(value(), m));
        }
        host.dispatchEvent(new CustomEvent("sui-command:select", {
          detail: { mode: m, item: active.dataset },
          bubbles: true,
          composed: true
        }));
        return;
      }
      if (m === "command")
        runSlashCommand(v);
      else if (m === "find")
        navigateFirst(v);
    }
    function runSlashCommand(v) {
      const needle = v.replace(/^\/+\s*/, "").toLowerCase();
      const cmd = SLASH_COMMANDS.find((c) => c.key === needle || c.key.startsWith(needle));
      if (!cmd)
        return;
      switch (cmd.key) {
        case "new":
          mode.set("create");
          value.set("");
          break;
        case "chats":
          navigateTo("/chats");
          open.set(false);
          break;
        case "settings":
          navigateTo("/settings");
          open.set(false);
          break;
        case "end":
          if (session())
            endSession(session().id);
          open.set(false);
          break;
        case "close":
          open.set(false);
          break;
        case "lint":
          runLint();
          break;
      }
    }
    function navigateFirst(v) {
      const needle = v.replace(/^>+\s*/, "").toLowerCase().trim();
      const dest = NAV_DESTINATIONS.find((d) => d.label.toLowerCase().includes(needle) || d.keywords.toLowerCase().includes(needle) || d.path.toLowerCase().includes(needle));
      if (dest) {
        navigateTo(dest.path);
        value.set("");
        open.set(false);
      }
    }
    async function endSession(id) {
      try {
        await fetch(`/api/v1/sessions/${id}`, { method: "DELETE", credentials: "same-origin" });
      } catch {}
      if (session()?.id === id) {
        session.set(null);
      }
      await loadCurrentSession();
    }
    async function runLint() {
      const current = window.ship?.current?.();
      const name = current?.path?.replace(/^\/+/, "").split("/")[0] ?? "";
      if (!name)
        return;
      try {
        const resp = await fetch("/api/v1/lint/view", {
          method: "POST",
          credentials: "same-origin",
          headers: { "Content-Type": "application/json" },
          body: JSON.stringify({ name })
        });
        if (resp.ok) {
          const body = await resp.json();
          const count = body.data?.findings?.length ?? 0;
          console.log(count === 0 ? `${name}: no lint findings` : `${name}: ${count} lint findings`);
        }
      } catch {}
      open.set(false);
    }
    const onDocKeydown = (e) => {
      if (e.key === "Escape" && open()) {
        open.set(false);
        return;
      }
      if (e.key === " " && !open()) {
        const t = e.target;
        if (t && (t.tagName === "INPUT" || t.tagName === "TEXTAREA" || t.isContentEditable))
          return;
        const now = performance.now();
        if (now - lastSpaceAt < 350) {
          lastSpaceAt = 0;
          e.preventDefault();
          mode.set("ai");
          value.set("");
          open.set(true);
        } else {
          lastSpaceAt = now;
        }
      }
    };
    const onPaletteKeydown = (e) => {
      if (e.key === "ArrowDown") {
        e.preventDefault();
        moveSelection(1);
      } else if (e.key === "ArrowUp") {
        e.preventDefault();
        moveSelection(-1);
      } else if (e.key === "Enter") {
        e.preventDefault();
        confirmSelection();
      }
    };
    host.addEventListener("keydown", onPaletteKeydown);
    document.addEventListener("keydown", onDocKeydown);
    const onDialogClose = () => {
      if (open())
        open.set(false);
    };
    dialogRef?.addEventListener("sui-close", onDialogClose);
    const onSessionChanged = () => void loadCurrentSession();
    const onBarOpen = (e) => {
      const ce = e;
      if (ce.detail?.mode)
        mode.set(ce.detail.mode);
      if (typeof ce.detail?.prefill === "string")
        value.set(ce.detail.prefill);
      open.set(true);
    };
    const onCreateOpen = () => {
      mode.set("create");
      open.set(true);
    };
    window.addEventListener("ship:session:changed", onSessionChanged);
    const shell = document.querySelector("ship-shell") ?? document.querySelector("sui-shell");
    shell?.addEventListener("ship:bar:open", onBarOpen);
    shell?.addEventListener("ship:create:open", onCreateOpen);
    const ctxKey = "__sui_ctx_sui-command-bar";
    const ctx = host[ctxKey];
    if (ctx) {
      ctx["endSession"] = endSession;
      ctx["loadCurrentSession"] = loadCurrentSession;
    }
    loadCurrentSession();
    return () => {
      document.removeEventListener("keydown", onDocKeydown);
      host.removeEventListener("keydown", onPaletteKeydown);
      window.removeEventListener("ship:session:changed", onSessionChanged);
      shell?.removeEventListener("ship:bar:open", onBarOpen);
      shell?.removeEventListener("ship:create:open", onCreateOpen);
      if (dialogRef)
        dialogRef.removeEventListener("sui-close", onDialogClose);
      stopStreamSync();
      stopOpenSync();
    };
  }
});

// packages/ui-components/src/islands/sui-command-bar/sui-command-bar.sui
/*! _compilerVersion {"cargo":"0.1.0","git":"d4751a31","dirty":true,"builtAt":"2026-05-14T02:37:23Z"} */
var sui_command_bar_default = component("sui-command-bar", {
  _compilerVersion: { cargo: "0.1.0", git: "d4751a31", dirty: true, builtAt: "2026-05-14T02:37:23Z" },
  props: {
    open: prop.bool({ default: false, reflect: true }),
    mode: prop.string({ default: "ai", reflect: true })
  },
  shadow: true,
  styles: `
  :host { display: contents; }

  /* ── Trigger pill ─────────────────────────────────────────────── */
  .cb-trigger {
    display: inline-flex;
    align-items: center;
    gap: 8px;
    padding: 4px 12px;
    border: 1px solid var(--border);
    border-radius: var(--radius-md, 6px);
    background: var(--background);
    color: var(--muted-foreground);
    font: inherit;
    font-size: var(--text-xs, 12px);
    cursor: pointer;
    min-width: 200px;
    max-width: 320px;
    text-align: left;
    transition: border-color var(--duration-fast, 120ms) var(--ease-out, ease);
  }
  .cb-trigger:hover {
    border-color: color-mix(in oklch, var(--foreground) 30%, var(--border));
  }
  .cb-label { flex: 1; }
  .cb-trigger kbd {
    font-family: var(--font-mono);
    font-size: 10px;
    padding: 1px 5px;
    border: 1px solid var(--border);
    border-radius: 3px;
    background: var(--muted);
    color: var(--muted-foreground);
  }

  /* ── Status dot ───────────────────────────────────────────────── */
  .cb-dot {
    width: 6px;
    height: 6px;
    border-radius: 999px;
    background: var(--muted-foreground);
    flex-shrink: 0;
    display: inline-block;
  }

  /* ── Dialog host override ─────────────────────────────────────── */
  .cb-dialog::part(body) {
    padding: 0;
    overflow: hidden;
    display: flex;
    flex-direction: column;
    min-height: 0;
  }

  /* ── Wrap (inside dialog body slot) ──────────────────────────── */
  .cb-wrap {
    display: flex;
    flex-direction: column;
    flex: 1;
    min-height: 0;
    overflow: hidden;
  }
`,
  setup(ctx) {
    const { props, host } = ctx;
    if (!ctx.cleanups)
      ctx.cleanups = [];
    const __cleanups = ctx.cleanups;
    const $scope = Object.create(null);
    $scope.open = props.open;
    $scope.mode = props.mode;
    $scope["dialogRef"] = signal2(null);
    let __r = null;
    if (sui_command_bar_controller_default && typeof sui_command_bar_controller_default.setup === "function") {
      ctx.scope = $scope;
      __r = sui_command_bar_controller_default.setup(ctx);
      if (__r && typeof __r === "object")
        Object.assign($scope, __r);
    }
    const __wireBoundary_b4 = (anchor) => bindIf({
      anchor,
      test: () => {
        if ($scope.isChat())
          return 0;
        return 1;
      },
      branches: [
        () => {
          const __frag10 = document.createDocumentFragment();
          const __c11 = [];
          const __e12 = document.createElement("sui-command-chat");
          __frag10.appendChild(__e12);
          return { nodes: Array.from(__frag10.childNodes), cleanup: () => {
            for (const c of __c11)
              try {
                c();
              } catch (e) {}
          } };
        },
        () => {
          const __frag13 = document.createDocumentFragment();
          const __c14 = [];
          const __e15 = document.createElement("sui-command-palette");
          __frag13.appendChild(__e15);
          return { nodes: Array.from(__frag13.childNodes), cleanup: () => {
            for (const c of __c14)
              try {
                c();
              } catch (e) {}
          } };
        }
      ]
    });
    const __root = host.shadowRoot ?? host;
    adoptSuiSheets(__root);
    const __e0 = document.createElement("button");
    __e0.setAttribute("class", "cb-trigger");
    __e0.setAttribute("type", "button");
    __e0.setAttribute("aria-label", "Open command bar");
    __cleanups.push(on(__e0, "click", function() {
      const __h = $scope.openFind;
      return typeof __h === "function" ? __h.bind(undefined) : () => {};
    }(), { modifiers: [] }));
    const __e1 = document.createElement("sui-icon");
    __e1.setAttribute("name", "search");
    __e1.setAttribute("size", "sm");
    __e0.appendChild(__e1);
    const __e2 = document.createElement("span");
    __e2.setAttribute("class", "cb-label");
    const __t3 = document.createTextNode("");
    __e2.appendChild(__t3);
    __cleanups.push(bindText(__t3, () => $scope.triggerLabel()));
    __e0.appendChild(__e2);
    const __e4 = document.createElement("span");
    __e4.setAttribute("class", "cb-dot");
    __cleanups.push(effect2(() => {
      const __v = $scope.dotBg();
      if (__v == null)
        __e4.style.removeProperty("background");
      else
        __e4.style.setProperty("background", String(__v));
    }));
    __e0.appendChild(__e4);
    const __e5 = document.createElement("kbd");
    const __t6 = document.createTextNode("⌘K");
    __e5.appendChild(__t6);
    __e0.appendChild(__e5);
    __root.appendChild(__e0);
    const __e7 = document.createElement("sui-dialog");
    __e7.setAttribute("class", "cb-dialog");
    __e7.setAttribute("no-header", "");
    __e7.setAttribute("no-footer", "");
    __e7.setAttribute("no-close-button", "");
    __e7.setAttribute("backdrop", "blur");
    __e7.setAttribute("size", "full");
    __e7.setAttribute("style", "--dlg-max-w: min(840px, 94vw)");
    attachRef($scope, "dialogRef", __e7);
    const __e8 = document.createElement("div");
    __e8.setAttribute("class", "cb-wrap");
    const __ifAnchor9 = document.createComment("if");
    __e8.appendChild(__ifAnchor9);
    __cleanups.push(__wireBoundary_b4(__ifAnchor9));
    __e7.appendChild(__e8);
    __root.appendChild(__e7);
    if (__r != null) {
      const __lc = Object.getOwnPropertyDescriptor(__r, "__shipSetLifecycle")?.value;
      if (__lc) {
        const __sp = new Proxy($scope, {
          get(t, k) {
            return t[k];
          },
          set(t, k, v) {
            const s = t[k];
            if (s != null && typeof s.set === "function") {
              if (typeof v === "function") {
                s.set(() => v);
              } else {
                s.set(v);
              }
              return true;
            }
            return false;
          }
        });
        if (__lc.mount) {
          const __mc = __lc.mount(__sp, host);
          if (typeof __mc === "function")
            __cleanups.push(__mc);
        }
        if (__lc.unmount)
          __cleanups.push(() => __lc.unmount(__sp));
      }
    }
    return $scope;
  }
});

// packages/ui-components/src/islands/sui-command-bar/sui-command-chat.sui
import { bindAttr, bindClass, bindFor, bindIf as bindIf2, bindText as bindText2, bindValue, effect as effect3, on as on2, signal as signal3, view } from "@ship/sui-runtime";
import { collectElementMarkers, collectTextOpens, hydrateAttr, hydrateBindValue, hydrateOn, hydrateStyle, hydrateText } from "@ship/sui-dom";

// packages/ui-components/src/islands/sui-command-bar/sui-command-chat.controller.ts
import { controller as controller2, derived as derived2, useShared } from "@ship/sui-runtime";
var sui_command_chat_controller_default = controller2({
  scope: (_, host) => {
    const bar = useShared(host, "sui-command-bar");
    const headerDotBg = derived2(() => {
      const s = bar.session();
      if (s?.status === "thinking")
        return "var(--warning,#d97706)";
      return "var(--success,#16a34a)";
    });
    const activeSessions = derived2(() => bar.allSessions().filter((s) => s.status !== "ended" && s.agent_id));
    function endSession(id) {
      fetch(`/api/v1/sessions/${id}`, { method: "DELETE", credentials: "same-origin" }).catch(() => {});
      if (bar.session()?.id === id)
        bar.session.set(null);
      bar.sessionPickerOpen.set(false);
    }
    function selectSession(id) {
      const s = bar.allSessions().find((x) => x.id === id);
      if (s) {
        bar.session.set(s);
        bar.thread.set([]);
      }
      bar.sessionPickerOpen.set(false);
    }
    function startNewSession() {
      bar.sessionPickerOpen.set(false);
      bar.mode.set("ai");
      bar.session.set(null);
    }
    function viewChats() {
      bar.sessionPickerOpen.set(false);
      navigateTo("/chats");
      bar.open.set(false);
    }
    return { bar, headerDotBg, activeSessions, endSession, selectSession, startNewSession, viewChats };
  },
  on: {
    toggleSessionPicker: (s) => {
      s.bar.sessionPickerOpen.set(!s.bar.sessionPickerOpen());
    },
    sendMessage: async (s) => {
      const text = s.bar.value().trim();
      if (!text || !s.bar.session())
        return;
      const sessionId = s.bar.session().id;
      s.bar.thread.set([
        ...s.bar.thread(),
        { id: String(Date.now() + Math.random()), sender: "user", text }
      ]);
      s.bar.value.set("");
      try {
        await fetch(`/api/v1/sessions/${sessionId}/message`, {
          method: "POST",
          credentials: "same-origin",
          headers: { "Content-Type": "application/json" },
          body: JSON.stringify({ text })
        });
      } catch {}
    },
    onComposerKeydown: async (s, e) => {
      if (e.key === "Enter" && !e.shiftKey) {
        e.preventDefault();
        const text = s.bar.value().trim();
        if (!text || !s.bar.session())
          return;
        const sessionId = s.bar.session().id;
        s.bar.thread.set([
          ...s.bar.thread(),
          { id: String(Date.now() + Math.random()), sender: "user", text }
        ]);
        s.bar.value.set("");
        try {
          await fetch(`/api/v1/sessions/${sessionId}/message`, {
            method: "POST",
            credentials: "same-origin",
            headers: { "Content-Type": "application/json" },
            body: JSON.stringify({ text })
          });
        } catch {}
      }
    }
  }
});

// packages/ui-components/src/islands/sui-command-bar/sui-command-chat.sui
/*! _compilerVersion {"cargo":"0.1.0","git":"d4751a31","dirty":true,"builtAt":"2026-05-14T02:37:23Z"} */
var sui_command_chat_default = Object.assign(view((ctx) => {
  const { ship, route, navigate } = ctx;
  const $scope = Object.create(null);
  $scope.ship = ship;
  $scope.route = route;
  $scope.navigate = navigate;
  if (sui_command_chat_controller_default && typeof sui_command_chat_controller_default.setup === "function") {
    ctx.scope = $scope;
    const __r = sui_command_chat_controller_default.setup(ctx);
    if (__r && typeof __r === "object")
      Object.assign($scope, __r);
  }
  const __wireBoundary_b6 = (anchor) => bindFor({
    anchor,
    items: () => $scope.activeSessions(),
    key: (s, __i) => {
      const __child = Object.create($scope);
      __child.s = s;
      __child.__i = __i;
      return (($scope2) => s.id)(__child);
    },
    render: (s, __i) => {
      const __childScope18 = Object.create($scope);
      __childScope18.s = s;
      __childScope18.__i = __i;
      __childScope18.__v = signal3(0);
      return (($scope2) => {
        const __frag19 = document.createDocumentFragment();
        const __c20 = [];
        const __e21 = document.createElement("div");
        __e21.setAttribute("class", "cb-session-row");
        __c20.push(bindClass(__e21, "current", () => (void $scope2.__v(), $scope2.bar.session()?.id === $scope2.s.id)));
        __e21.setAttribute("role", "option");
        __c20.push(on2(__e21, "click", function() {
          const __h = () => $scope2.selectSession($scope2.s.id);
          return typeof __h === "function" ? __h.bind(undefined) : () => {};
        }(), { modifiers: [] }));
        const __e22 = document.createElement("span");
        __e22.setAttribute("class", "cb-dot");
        __c20.push(effect3(() => {
          $scope2.__v();
          const __v = $scope2.s.status === "thinking" ? "var(--warning,#d97706)" : "var(--success,#16a34a)";
          if (__v == null)
            __e22.style.removeProperty("background");
          else
            __e22.style.setProperty("background", String(__v));
        }));
        __e21.appendChild(__e22);
        const __e23 = document.createElement("span");
        const __e24 = document.createElement("strong");
        const __t25 = document.createTextNode("");
        __e24.appendChild(__t25);
        __c20.push(bindText2(__t25, () => (void $scope2.__v(), $scope2.s.agent_id ?? "")));
        __e23.appendChild(__e24);
        const __e26 = document.createElement("span");
        __e26.setAttribute("class", "meta");
        const __t27 = document.createTextNode(" · ");
        __e26.appendChild(__t27);
        const __t28 = document.createTextNode("");
        __e26.appendChild(__t28);
        __c20.push(bindText2(__t28, () => (void $scope2.__v(), $scope2.s.provider ?? "")));
        const __t29 = document.createTextNode(" · ");
        __e26.appendChild(__t29);
        const __t30 = document.createTextNode("");
        __e26.appendChild(__t30);
        __t30.textContent = $scope2.s.workspace_id;
        __e23.appendChild(__e26);
        __e21.appendChild(__e23);
        const __e31 = document.createElement("button");
        __e31.setAttribute("class", "cb-session-end");
        __e31.setAttribute("type", "button");
        __e31.setAttribute("aria-label", "End session");
        __c20.push(on2(__e31, "click", function() {
          const __h = (e) => {
            e.stopPropagation();
            $scope2.endSession($scope2.s.id);
          };
          return typeof __h === "function" ? __h.bind(undefined) : () => {};
        }(), { modifiers: [] }));
        const __t32 = document.createTextNode("×");
        __e31.appendChild(__t32);
        __e21.appendChild(__e31);
        __frag19.appendChild(__e21);
        return { nodes: Array.from(__frag19.childNodes), cleanup: () => {
          for (const c of __c20)
            try {
              c();
            } catch (e) {}
        }, update: (__new33, __newI34) => {
          $scope2.s = __new33;
          $scope2.__i = __newI34;
          __t30.textContent = $scope2.s.workspace_id;
          $scope2.__v.set((c) => c + 1);
        } };
      })(__childScope18);
    }
  });
  const __wireBoundary_b14 = (anchor) => bindIf2({
    anchor,
    test: () => {
      if ($scope.activeSessions().length === 0)
        return 0;
      return -1;
    },
    branches: [
      () => {
        const __frag36 = document.createDocumentFragment();
        const __c37 = [];
        const __e38 = document.createElement("div");
        __e38.setAttribute("class", "cb-session-row");
        const __e39 = document.createElement("span");
        __e38.appendChild(__e39);
        const __e40 = document.createElement("span");
        __e40.setAttribute("class", "meta");
        const __t41 = document.createTextNode("No active sessions");
        __e40.appendChild(__t41);
        __e38.appendChild(__e40);
        const __e42 = document.createElement("span");
        __e38.appendChild(__e42);
        __frag36.appendChild(__e38);
        return { nodes: Array.from(__frag36.childNodes), cleanup: () => {
          for (const c of __c37)
            try {
              c();
            } catch (e) {}
        } };
      }
    ]
  });
  const __wireBoundary_b5 = (anchor) => bindIf2({
    anchor,
    test: () => {
      if ($scope.bar.sessionPickerOpen())
        return 0;
      return -1;
    },
    branches: [
      () => {
        const __frag14 = document.createDocumentFragment();
        const __c15 = [];
        const __e16 = document.createElement("div");
        __e16.setAttribute("class", "cb-session-popover");
        __e16.setAttribute("role", "listbox");
        __e16.setAttribute("aria-label", "Sessions");
        const __forAnchor17 = document.createComment("for");
        __e16.appendChild(__forAnchor17);
        __c15.push(__wireBoundary_b6(__forAnchor17));
        const __ifAnchor35 = document.createComment("if");
        __e16.appendChild(__ifAnchor35);
        __c15.push(__wireBoundary_b14(__ifAnchor35));
        const __e43 = document.createElement("div");
        __e43.setAttribute("class", "cb-session-sep");
        __e43.setAttribute("role", "separator");
        __e16.appendChild(__e43);
        const __e44 = document.createElement("div");
        __e44.setAttribute("class", "cb-session-action");
        __c15.push(on2(__e44, "click", function() {
          const __h = $scope.startNewSession;
          return typeof __h === "function" ? __h.bind(undefined) : () => {};
        }(), { modifiers: [] }));
        const __e45 = document.createElement("sui-icon");
        __e45.setAttribute("name", "plus");
        __e45.setAttribute("size", "sm");
        __e45.setAttribute("aria-hidden", "true");
        __e44.appendChild(__e45);
        const __e46 = document.createElement("span");
        const __t47 = document.createTextNode("Start new session");
        __e46.appendChild(__t47);
        __e44.appendChild(__e46);
        __e16.appendChild(__e44);
        const __e48 = document.createElement("div");
        __e48.setAttribute("class", "cb-session-action");
        __c15.push(on2(__e48, "click", function() {
          const __h = $scope.viewChats;
          return typeof __h === "function" ? __h.bind(undefined) : () => {};
        }(), { modifiers: [] }));
        const __e49 = document.createElement("sui-icon");
        __e49.setAttribute("name", "columns");
        __e49.setAttribute("size", "sm");
        __e49.setAttribute("aria-hidden", "true");
        __e48.appendChild(__e49);
        const __e50 = document.createElement("span");
        const __t51 = document.createTextNode("View all chats");
        __e50.appendChild(__t51);
        __e48.appendChild(__e50);
        __e16.appendChild(__e48);
        __frag14.appendChild(__e16);
        return { nodes: Array.from(__frag14.childNodes), cleanup: () => {
          for (const c of __c15)
            try {
              c();
            } catch (e) {}
        } };
      }
    ]
  });
  const __wireBoundary_b18 = (anchor) => bindFor({
    anchor,
    items: () => $scope.bar.thread(),
    key: (msg, __i) => {
      const __child = Object.create($scope);
      __child.msg = msg;
      __child.__i = __i;
      return (($scope2) => msg.id)(__child);
    },
    render: (msg, __i) => {
      const __childScope63 = Object.create($scope);
      __childScope63.msg = msg;
      __childScope63.__i = __i;
      __childScope63.__v = signal3(0);
      return (($scope2) => {
        const __frag64 = document.createDocumentFragment();
        const __c65 = [];
        const __e66 = document.createElement("div");
        __e66.setAttribute("class", "cb-chat-turn");
        {
          const __av67 = $scope2.msg.sender;
          if (__av67 == null || __av67 === false || __av67 === "")
            __e66.removeAttribute("data-sender");
          else
            __e66.setAttribute("data-sender", __av67 === true ? "" : String(__av67));
        }
        const __e68 = document.createElement("div");
        __e68.setAttribute("class", "cb-chat-bubble");
        const __t69 = document.createTextNode("");
        __e68.appendChild(__t69);
        __t69.textContent = $scope2.msg.text;
        __e66.appendChild(__e68);
        __frag64.appendChild(__e66);
        return { nodes: Array.from(__frag64.childNodes), cleanup: () => {
          for (const c of __c65)
            try {
              c();
            } catch (e) {}
        }, update: (__new70, __newI71) => {
          $scope2.msg = __new70;
          $scope2.__i = __newI71;
          {
            const __av67 = $scope2.msg.sender;
            if (__av67 == null || __av67 === false || __av67 === "")
              __e66.removeAttribute("data-sender");
            else
              __e66.setAttribute("data-sender", __av67 === true ? "" : String(__av67));
          }
          __t69.textContent = $scope2.msg.text;
          $scope2.__v.set((c) => c + 1);
        } };
      })(__childScope63);
    }
  });
  const __wireBoundary_b17 = (anchor) => bindIf2({
    anchor,
    test: () => {
      if ($scope.bar.thread().length === 0)
        return 0;
      return 1;
    },
    branches: [
      () => {
        const __frag56 = document.createDocumentFragment();
        const __c57 = [];
        const __e58 = document.createElement("div");
        __e58.setAttribute("class", "cb-chat-empty");
        const __t59 = document.createTextNode("Type below and press Enter. The agent's replies land here.");
        __e58.appendChild(__t59);
        __frag56.appendChild(__e58);
        return { nodes: Array.from(__frag56.childNodes), cleanup: () => {
          for (const c of __c57)
            try {
              c();
            } catch (e) {}
        } };
      },
      () => {
        const __frag60 = document.createDocumentFragment();
        const __c61 = [];
        const __forAnchor62 = document.createComment("for");
        __frag60.appendChild(__forAnchor62);
        __c61.push(__wireBoundary_b18(__forAnchor62));
        return { nodes: Array.from(__frag60.childNodes), cleanup: () => {
          for (const c of __c61)
            try {
              c();
            } catch (e) {}
        } };
      }
    ]
  });
  return Object.assign({}, $scope, {
    __mount(container) {
      const __cleanups = [];
      const __e0 = document.createElement("div");
      __e0.setAttribute("class", "cb-chat-header");
      const __e1 = document.createElement("span");
      __e1.setAttribute("class", "cb-dot");
      __cleanups.push(effect3(() => {
        const __v = $scope.headerDotBg();
        if (__v == null)
          __e1.style.removeProperty("background");
        else
          __e1.style.setProperty("background", String(__v));
      }));
      __e1.setAttribute("aria-hidden", "true");
      __e0.appendChild(__e1);
      const __e2 = document.createElement("div");
      __e2.setAttribute("style", "display:grid;line-height:1.25");
      const __e3 = document.createElement("span");
      __e3.setAttribute("class", "title");
      const __t4 = document.createTextNode("");
      __e3.appendChild(__t4);
      __cleanups.push(bindText2(__t4, () => $scope.bar.session()?.agent_id ?? ""));
      __e2.appendChild(__e3);
      const __e5 = document.createElement("span");
      __e5.setAttribute("class", "sub");
      const __t6 = document.createTextNode("");
      __e5.appendChild(__t6);
      __cleanups.push(bindText2(__t6, () => $scope.bar.session()?.provider ?? "—"));
      const __t7 = document.createTextNode(" · ");
      __e5.appendChild(__t7);
      const __t8 = document.createTextNode("");
      __e5.appendChild(__t8);
      __cleanups.push(bindText2(__t8, () => $scope.bar.session()?.workspace_id ?? ""));
      __e2.appendChild(__e5);
      __e0.appendChild(__e2);
      const __e9 = document.createElement("span");
      __e9.setAttribute("style", "flex:1");
      __e0.appendChild(__e9);
      const __e10 = document.createElement("div");
      __e10.setAttribute("class", "cb-chip-wrap");
      const __e11 = document.createElement("span");
      __e11.setAttribute("class", "cb-session-chip");
      __e11.setAttribute("role", "button");
      __e11.setAttribute("aria-haspopup", "listbox");
      __cleanups.push(on2(__e11, "click", function() {
        const __h = $scope.toggleSessionPicker;
        return typeof __h === "function" ? __h.bind(undefined) : () => {};
      }(), { modifiers: [] }));
      const __t12 = document.createTextNode("Switch");
      __e11.appendChild(__t12);
      __e10.appendChild(__e11);
      const __ifAnchor13 = document.createComment("if");
      __e10.appendChild(__ifAnchor13);
      __cleanups.push(__wireBoundary_b5(__ifAnchor13));
      __e0.appendChild(__e10);
      const __e52 = document.createElement("kbd");
      __e52.setAttribute("class", "cb-esc");
      __e52.setAttribute("aria-label", "Escape to close");
      const __t53 = document.createTextNode("esc");
      __e52.appendChild(__t53);
      __e0.appendChild(__e52);
      container.appendChild(__e0);
      const __e54 = document.createElement("div");
      __e54.setAttribute("class", "cb-chat-messages");
      const __ifAnchor55 = document.createComment("if");
      __e54.appendChild(__ifAnchor55);
      __cleanups.push(__wireBoundary_b17(__ifAnchor55));
      container.appendChild(__e54);
      const __e72 = document.createElement("div");
      __e72.setAttribute("class", "cb-chat-composer");
      const __e73 = document.createElement("textarea");
      __e73.setAttribute("class", "cb-chat-input");
      __e73.setAttribute("rows", "2");
      __e73.setAttribute("placeholder", "Message the agent…");
      __cleanups.push(bindValue(__e73, $scope.bar.value));
      __cleanups.push(on2(__e73, "keydown", function() {
        const __h = $scope.onComposerKeydown;
        return typeof __h === "function" ? __h.bind(undefined) : () => {};
      }(), { modifiers: [] }));
      __e72.appendChild(__e73);
      const __e74 = document.createElement("button");
      __e74.setAttribute("class", "cb-chat-send");
      __e74.setAttribute("type", "button");
      __cleanups.push(bindAttr(__e74, "disabled", () => !$scope.bar.value().trim()));
      __cleanups.push(on2(__e74, "click", function() {
        const __h = $scope.sendMessage;
        return typeof __h === "function" ? __h.bind(undefined) : () => {};
      }(), { modifiers: [] }));
      const __t75 = document.createTextNode("Send");
      __e74.appendChild(__t75);
      __e72.appendChild(__e74);
      container.appendChild(__e72);
      return () => {
        for (const c of __cleanups)
          try {
            c();
          } catch (e) {}
      };
    },
    hydrate(root, ctx2) {
      const __cleanups = [];
      const __t = collectTextOpens(root);
      const __m = collectElementMarkers(root);
      if (__m["b0"])
        __cleanups.push(hydrateStyle(__m["b0"], "background", () => $scope.headerDotBg()));
      if (__t["b1"])
        __cleanups.push(hydrateText(__t["b1"], () => $scope.bar.session()?.agent_id ?? ""));
      if (__t["b2"])
        __cleanups.push(hydrateText(__t["b2"], () => $scope.bar.session()?.provider ?? "—"));
      if (__t["b3"])
        __cleanups.push(hydrateText(__t["b3"], () => $scope.bar.session()?.workspace_id ?? ""));
      if (__m["b4"])
        __cleanups.push(hydrateOn(__m["b4"], "click", function() {
          const __h = $scope.toggleSessionPicker;
          return typeof __h === "function" ? __h.bind(undefined) : () => {};
        }(), []));
      if (__m["b5"]) {
        const __t2 = __m["b5"];
        const __doc = __t2.ownerDocument ?? document;
        const __anchor = __t2.nodeType === 1 ? __doc.createComment("if") : __t2;
        if (__t2.nodeType === 1 && __t2.parentNode)
          __t2.parentNode.replaceChild(__anchor, __t2);
        __cleanups.push(__wireBoundary_b5(__anchor));
      }
      if (__m["b17"]) {
        const __t2 = __m["b17"];
        const __doc = __t2.ownerDocument ?? document;
        const __anchor = __t2.nodeType === 1 ? __doc.createComment("if") : __t2;
        if (__t2.nodeType === 1 && __t2.parentNode)
          __t2.parentNode.replaceChild(__anchor, __t2);
        __cleanups.push(__wireBoundary_b17(__anchor));
      }
      if (__m["b21"])
        __cleanups.push(hydrateBindValue(__m["b21"], $scope.bar.value));
      if (__m["b22"])
        __cleanups.push(hydrateOn(__m["b22"], "keydown", function() {
          const __h = $scope.onComposerKeydown;
          return typeof __h === "function" ? __h.bind(undefined) : () => {};
        }(), []));
      if (__m["b23"])
        __cleanups.push(hydrateAttr(__m["b23"], "disabled", () => !$scope.bar.value().trim()));
      if (__m["b24"])
        __cleanups.push(hydrateOn(__m["b24"], "click", function() {
          const __h = $scope.sendMessage;
          return typeof __h === "function" ? __h.bind(undefined) : () => {};
        }(), []));
      return () => {
        for (const c of __cleanups)
          try {
            c();
          } catch (e) {}
      };
    }
  });
}), { _compilerVersion: { cargo: "0.1.0", git: "d4751a31", dirty: true, builtAt: "2026-05-14T02:37:23Z" } });

// packages/ui-components/src/islands/sui-command-bar/sui-command-palette.sui
import { bindAttr as bindAttr2, bindClass as bindClass2, bindFor as bindFor2, bindIf as bindIf3, bindText as bindText3, bindValue as bindValue2, effect as effect4, on as on3, signal as signal4, view as view2 } from "@ship/sui-runtime";
import { collectElementMarkers as collectElementMarkers2, collectTextOpens as collectTextOpens2, hydrateAttr as hydrateAttr2, hydrateBindValue as hydrateBindValue2, hydrateClass, hydrateOn as hydrateOn2, hydrateStyle as hydrateStyle2, hydrateText as hydrateText2 } from "@ship/sui-dom";

// packages/ui-components/src/islands/sui-command-bar/sui-command-palette.controller.ts
import { controller as controller3, derived as derived3, useShared as useShared2 } from "@ship/sui-runtime";
var sui_command_palette_controller_default = controller3({
  scope: (_, host) => {
    const bar = useShared2(host, "sui-command-bar");
    const modeGlyph = derived3(() => {
      const glyphs = {
        command: "/",
        find: ">",
        mention: "@",
        create: "+",
        ai: "ai"
      };
      return glyphs[bar.mode()] ?? ">";
    });
    const searchPlaceholder = derived3(() => {
      const placeholders = {
        command: "Run a Ship action…",
        find: "Jump to a view…",
        mention: "Reference an agent, view, file…",
        create: "Create a view, skill, agent…",
        ai: "Ask Ship or type / for actions"
      };
      return placeholders[bar.mode()] ?? "Search…";
    });
    const chipLabel = derived3(() => bar.session()?.agent_id ?? "No session");
    const chipDotBg = derived3(() => {
      const s = bar.session();
      if (s?.status === "thinking")
        return "var(--warning,#d97706)";
      if (s)
        return "var(--success,#16a34a)";
      return "var(--muted-foreground)";
    });
    const activeSessions = derived3(() => bar.allSessions().filter((s) => s.status !== "ended" && s.agent_id));
    function mentionIcon(kind) {
      if (kind === "view")
        return "app-window";
      if (kind === "agent")
        return "bot";
      return "message-square-plus";
    }
    const findItems = derived3(() => {
      const needle = bar.value().replace(/^>+\s*/, "").toLowerCase().trim();
      return NAV_DESTINATIONS.filter((d) => !needle || d.label.toLowerCase().includes(needle) || d.keywords.toLowerCase().includes(needle) || d.path.toLowerCase().includes(needle)).map((d, i) => ({ ...d, idx: i }));
    });
    const cmdItems = derived3(() => {
      const needle = bar.value().replace(/^\/+\s*/, "").toLowerCase().trim();
      return SLASH_COMMANDS.filter((c) => !needle || c.key.includes(needle) || c.label.toLowerCase().includes(needle) || c.hint.toLowerCase().includes(needle)).map((c, i) => ({ ...c, idx: i }));
    });
    const mentionItems = derived3(() => {
      const needle = bar.value().replace(/^@+/, "").toLowerCase().trim();
      const refs = [
        ...bar.views().map((v) => ({ kind: "view", id: v, label: v })),
        ...bar.agents().map((a) => ({ kind: "agent", id: a.id, label: a.id })),
        ...bar.allSessions().filter((s) => s.status !== "ended" && s.agent_id).map((s) => ({ kind: "session", id: s.id, label: `${s.agent_id} (session)` }))
      ];
      return refs.filter((r) => !needle || r.id.toLowerCase().includes(needle)).slice(0, 40).map((r, i) => ({ ...r, idx: i, icon: mentionIcon(r.kind) }));
    });
    const createSurfaces = derived3(() => CREATE_SURFACES);
    const currentSurfaceDef = derived3(() => CREATE_SURFACES.find((s) => s.id === bar.createSurface()));
    const createNameValid = derived3(() => isKebab(bar.createName()));
    function endSession(id) {
      fetch(`/api/v1/sessions/${id}`, { method: "DELETE", credentials: "same-origin" }).catch(() => {}).then(() => {
        if (bar.session()?.id === id)
          bar.session.set(null);
        reloadSessions();
      });
      bar.sessionPickerOpen.set(false);
    }
    function selectSession(id) {
      const s = bar.allSessions().find((x) => x.id === id);
      if (s) {
        bar.session.set(s);
        bar.thread.set([]);
      }
      bar.sessionPickerOpen.set(false);
    }
    function startNewSession() {
      bar.sessionPickerOpen.set(false);
      bar.mode.set("ai");
      bar.session.set(null);
    }
    function viewChats() {
      bar.sessionPickerOpen.set(false);
      navigateTo("/chats");
      bar.open.set(false);
    }
    const hostEl = host;
    function activateNavItem(path) {
      navigateTo(path);
      bar.value.set("");
      bar.open.set(false);
      hostEl?.dispatchEvent(new CustomEvent("sui-command:select", {
        detail: { mode: "find", item: { path } },
        bubbles: true,
        composed: true
      }));
    }
    function activateCmd(key) {
      bar.value.set("");
      switch (key) {
        case "new":
          bar.mode.set("create");
          break;
        case "chats":
          navigateTo("/chats");
          bar.open.set(false);
          break;
        case "settings":
          navigateTo("/settings");
          bar.open.set(false);
          break;
        case "end":
          if (bar.session())
            endSession(bar.session().id);
          bar.open.set(false);
          break;
        case "close":
          bar.open.set(false);
          break;
      }
    }
    function activateMention(item) {
      const ref2 = item.kind === "view" ? `@${item.id}` : `@${item.kind}:${item.id}`;
      const v = bar.value();
      bar.value.set(v.replace(/@[^@\s]*$/, `${ref2} `));
      bar.mode.set(detectMode(bar.value(), bar.mode()));
    }
    async function reloadSessions() {
      try {
        const resp = await fetch("/api/v1/pty-sessions", { credentials: "same-origin" });
        if (resp.ok) {
          const body = await resp.json();
          bar.allSessions.set(body.sessions);
          const active = body.sessions.find((s) => s.status !== "ended" && s.agent_id);
          if (active && active.id !== bar.session()?.id) {
            bar.session.set(active);
            bar.thread.set([]);
          } else if (!active) {
            bar.session.set(null);
          }
        }
      } catch {}
    }
    return {
      bar,
      modeGlyph,
      searchPlaceholder,
      chipLabel,
      chipDotBg,
      activeSessions,
      findItems,
      cmdItems,
      mentionItems,
      createSurfaces,
      currentSurfaceDef,
      createNameValid,
      endSession,
      selectSession,
      startNewSession,
      viewChats,
      activateNavItem,
      activateCmd,
      activateMention
    };
  },
  on: {
    onInput: (s, e) => {
      const v = e.target.value;
      s.bar.value.set(v);
      s.bar.mode.set(detectMode(v, s.bar.mode()));
      s.bar.selectedIdx.set(0);
    },
    toggleSessionPicker: (s) => {
      s.bar.sessionPickerOpen.set(!s.bar.sessionPickerOpen());
    },
    selectSurface: (s, id) => {
      s.bar.createSurface.set(id);
      const def = CREATE_SURFACES.find((x) => x.id === id);
      if (def?.shapes[0])
        s.bar.createShape.set(def.shapes[0]);
    },
    spawnSession: async (s) => {
      const agent = s.bar.spawnAgent();
      const provider = s.bar.spawnProvider();
      if (!agent || !provider)
        return;
      s.bar.spawning.set(true);
      s.bar.spawnError.set(null);
      try {
        const resp = await fetch("/api/v1/sessions", {
          method: "POST",
          credentials: "same-origin",
          headers: { "Content-Type": "application/json" },
          body: JSON.stringify({ agent_id: agent, provider })
        });
        if (!resp.ok) {
          const err = await resp.json().catch(() => ({}));
          s.bar.spawnError.set(err.error ?? `HTTP ${resp.status}`);
        } else {
          s.bar.mode.set("ai");
        }
      } catch (err) {
        s.bar.spawnError.set(err instanceof Error ? err.message : "Unknown error");
      } finally {
        s.bar.spawning.set(false);
      }
    },
    submitCreate: async (s) => {
      if (!s.createNameValid() || s.bar.createSubmitting())
        return;
      s.bar.createSubmitting.set(true);
      s.bar.createError.set(null);
      try {
        const resp = await fetch("/api/v1/scaffold", {
          method: "POST",
          credentials: "same-origin",
          headers: { "Content-Type": "application/json" },
          body: JSON.stringify({
            surface: s.bar.createSurface(),
            name: s.bar.createName(),
            shape: s.bar.createShape(),
            purpose: s.bar.createPurpose()
          })
        });
        if (!resp.ok) {
          const err = await resp.json().catch(() => ({}));
          s.bar.createError.set(err.error ?? `HTTP ${resp.status}`);
        } else {
          s.bar.createName.set("");
          s.bar.createPurpose.set("");
          s.bar.open.set(false);
        }
      } catch (err) {
        s.bar.createError.set(err instanceof Error ? err.message : "Unknown error");
      } finally {
        s.bar.createSubmitting.set(false);
      }
    }
  }
});

// packages/ui-components/src/islands/sui-command-bar/sui-command-palette.sui
/*! _compilerVersion {"cargo":"0.1.0","git":"d4751a31","dirty":true,"builtAt":"2026-05-14T02:37:23Z"} */
var sui_command_palette_default = Object.assign(view2((ctx) => {
  const { ship, route, navigate } = ctx;
  const $scope = Object.create(null);
  $scope.ship = ship;
  $scope.route = route;
  $scope.navigate = navigate;
  if (sui_command_palette_controller_default && typeof sui_command_palette_controller_default.setup === "function") {
    ctx.scope = $scope;
    const __r = sui_command_palette_controller_default.setup(ctx);
    if (__r && typeof __r === "object")
      Object.assign($scope, __r);
  }
  const __wireBoundary_b9 = (anchor) => bindFor2({
    anchor,
    items: () => $scope.activeSessions(),
    key: (s, __i) => {
      const __child = Object.create($scope);
      __child.s = s;
      __child.__i = __i;
      return (($scope2) => s.id)(__child);
    },
    render: (s, __i) => {
      const __childScope16 = Object.create($scope);
      __childScope16.s = s;
      __childScope16.__i = __i;
      __childScope16.__v = signal4(0);
      return (($scope2) => {
        const __frag17 = document.createDocumentFragment();
        const __c18 = [];
        const __e19 = document.createElement("div");
        __e19.setAttribute("class", "cb-session-row");
        __c18.push(bindClass2(__e19, "current", () => (void $scope2.__v(), $scope2.bar.session()?.id === $scope2.s.id)));
        __e19.setAttribute("role", "option");
        __c18.push(on3(__e19, "click", function() {
          const __h = () => $scope2.selectSession($scope2.s.id);
          return typeof __h === "function" ? __h.bind(undefined) : () => {};
        }(), { modifiers: [] }));
        const __e20 = document.createElement("span");
        __e20.setAttribute("class", "cb-dot");
        __c18.push(effect4(() => {
          $scope2.__v();
          const __v = $scope2.s.status === "thinking" ? "var(--warning,#d97706)" : "var(--success,#16a34a)";
          if (__v == null)
            __e20.style.removeProperty("background");
          else
            __e20.style.setProperty("background", String(__v));
        }));
        __e19.appendChild(__e20);
        const __e21 = document.createElement("span");
        const __e22 = document.createElement("strong");
        const __t23 = document.createTextNode("");
        __e22.appendChild(__t23);
        __c18.push(bindText3(__t23, () => (void $scope2.__v(), $scope2.s.agent_id ?? "")));
        __e21.appendChild(__e22);
        const __e24 = document.createElement("span");
        __e24.setAttribute("class", "meta");
        const __t25 = document.createTextNode(" · ");
        __e24.appendChild(__t25);
        const __t26 = document.createTextNode("");
        __e24.appendChild(__t26);
        __c18.push(bindText3(__t26, () => (void $scope2.__v(), $scope2.s.provider ?? "")));
        const __t27 = document.createTextNode(" · ");
        __e24.appendChild(__t27);
        const __t28 = document.createTextNode("");
        __e24.appendChild(__t28);
        __t28.textContent = $scope2.s.workspace_id;
        __e21.appendChild(__e24);
        __e19.appendChild(__e21);
        const __e29 = document.createElement("button");
        __e29.setAttribute("class", "cb-session-end");
        __e29.setAttribute("type", "button");
        __e29.setAttribute("aria-label", "End session");
        __c18.push(on3(__e29, "click", function() {
          const __h = (e) => {
            e.stopPropagation();
            $scope2.endSession($scope2.s.id);
          };
          return typeof __h === "function" ? __h.bind(undefined) : () => {};
        }(), { modifiers: [] }));
        const __t30 = document.createTextNode("×");
        __e29.appendChild(__t30);
        __e19.appendChild(__e29);
        __frag17.appendChild(__e19);
        return { nodes: Array.from(__frag17.childNodes), cleanup: () => {
          for (const c of __c18)
            try {
              c();
            } catch (e) {}
        }, update: (__new31, __newI32) => {
          $scope2.s = __new31;
          $scope2.__i = __newI32;
          __t28.textContent = $scope2.s.workspace_id;
          $scope2.__v.set((c) => c + 1);
        } };
      })(__childScope16);
    }
  });
  const __wireBoundary_b17 = (anchor) => bindIf3({
    anchor,
    test: () => {
      if ($scope.activeSessions().length === 0)
        return 0;
      return -1;
    },
    branches: [
      () => {
        const __frag34 = document.createDocumentFragment();
        const __c35 = [];
        const __e36 = document.createElement("div");
        __e36.setAttribute("class", "cb-session-row");
        const __e37 = document.createElement("span");
        __e36.appendChild(__e37);
        const __e38 = document.createElement("span");
        __e38.setAttribute("class", "meta");
        const __t39 = document.createTextNode("No active sessions");
        __e38.appendChild(__t39);
        __e36.appendChild(__e38);
        const __e40 = document.createElement("span");
        __e36.appendChild(__e40);
        __frag34.appendChild(__e36);
        return { nodes: Array.from(__frag34.childNodes), cleanup: () => {
          for (const c of __c35)
            try {
              c();
            } catch (e) {}
        } };
      }
    ]
  });
  const __wireBoundary_b8 = (anchor) => bindIf3({
    anchor,
    test: () => {
      if ($scope.bar.sessionPickerOpen())
        return 0;
      return -1;
    },
    branches: [
      () => {
        const __frag12 = document.createDocumentFragment();
        const __c13 = [];
        const __e14 = document.createElement("div");
        __e14.setAttribute("class", "cb-session-popover");
        __e14.setAttribute("role", "listbox");
        __e14.setAttribute("aria-label", "Sessions");
        const __forAnchor15 = document.createComment("for");
        __e14.appendChild(__forAnchor15);
        __c13.push(__wireBoundary_b9(__forAnchor15));
        const __ifAnchor33 = document.createComment("if");
        __e14.appendChild(__ifAnchor33);
        __c13.push(__wireBoundary_b17(__ifAnchor33));
        const __e41 = document.createElement("div");
        __e41.setAttribute("class", "cb-session-sep");
        __e41.setAttribute("role", "separator");
        __e14.appendChild(__e41);
        const __e42 = document.createElement("div");
        __e42.setAttribute("class", "cb-session-action");
        __c13.push(on3(__e42, "click", function() {
          const __h = $scope.startNewSession;
          return typeof __h === "function" ? __h.bind(undefined) : () => {};
        }(), { modifiers: [] }));
        const __e43 = document.createElement("sui-icon");
        __e43.setAttribute("name", "plus");
        __e43.setAttribute("size", "sm");
        __e43.setAttribute("aria-hidden", "true");
        __e42.appendChild(__e43);
        const __e44 = document.createElement("span");
        const __t45 = document.createTextNode("Start new session");
        __e44.appendChild(__t45);
        __e42.appendChild(__e44);
        __e14.appendChild(__e42);
        const __e46 = document.createElement("div");
        __e46.setAttribute("class", "cb-session-action");
        __c13.push(on3(__e46, "click", function() {
          const __h = $scope.viewChats;
          return typeof __h === "function" ? __h.bind(undefined) : () => {};
        }(), { modifiers: [] }));
        const __e47 = document.createElement("sui-icon");
        __e47.setAttribute("name", "columns");
        __e47.setAttribute("size", "sm");
        __e47.setAttribute("aria-hidden", "true");
        __e46.appendChild(__e47);
        const __e48 = document.createElement("span");
        const __t49 = document.createTextNode("View all chats");
        __e48.appendChild(__t49);
        __e46.appendChild(__e48);
        __e14.appendChild(__e46);
        __frag12.appendChild(__e14);
        return { nodes: Array.from(__frag12.childNodes), cleanup: () => {
          for (const c of __c13)
            try {
              c();
            } catch (e) {}
        } };
      }
    ]
  });
  const __wireBoundary_b22 = (anchor) => bindFor2({
    anchor,
    items: () => $scope.findItems(),
    key: (item, __i) => {
      const __child = Object.create($scope);
      __child.item = item;
      __child.__i = __i;
      return (($scope2) => item.path)(__child);
    },
    render: (item, __i) => {
      const __childScope61 = Object.create($scope);
      __childScope61.item = item;
      __childScope61.__i = __i;
      __childScope61.__v = signal4(0);
      return (($scope2) => {
        const __frag62 = document.createDocumentFragment();
        const __c63 = [];
        const __e64 = document.createElement("div");
        __e64.setAttribute("class", "cb-item");
        __e64.setAttribute("role", "option");
        __c63.push(bindAttr2(__e64, "aria-selected", () => (void $scope2.__v(), $scope2.bar.selectedIdx() === $scope2.item.idx ? "true" : "false")));
        {
          const __av65 = $scope2.item.path;
          if (__av65 == null || __av65 === false || __av65 === "")
            __e64.removeAttribute("data-path");
          else
            __e64.setAttribute("data-path", __av65 === true ? "" : String(__av65));
        }
        __c63.push(on3(__e64, "click", function() {
          const __h = () => $scope2.activateNavItem($scope2.item.path);
          return typeof __h === "function" ? __h.bind(undefined) : () => {};
        }(), { modifiers: [] }));
        const __e66 = document.createElement("sui-icon");
        {
          const __av67 = $scope2.item.icon;
          if (__av67 == null || __av67 === false || __av67 === "")
            __e66.removeAttribute("name");
          else
            __e66.setAttribute("name", __av67 === true ? "" : String(__av67));
        }
        __e66.setAttribute("size", "sm");
        __e66.setAttribute("aria-hidden", "true");
        __e64.appendChild(__e66);
        const __e68 = document.createElement("span");
        const __t69 = document.createTextNode("");
        __e68.appendChild(__t69);
        __t69.textContent = $scope2.item.label;
        __e64.appendChild(__e68);
        const __e70 = document.createElement("span");
        __e70.setAttribute("class", "hint");
        const __t71 = document.createTextNode("");
        __e70.appendChild(__t71);
        __t71.textContent = $scope2.item.path;
        __e64.appendChild(__e70);
        __frag62.appendChild(__e64);
        return { nodes: Array.from(__frag62.childNodes), cleanup: () => {
          for (const c of __c63)
            try {
              c();
            } catch (e) {}
        }, update: (__new72, __newI73) => {
          $scope2.item = __new72;
          $scope2.__i = __newI73;
          {
            const __av65 = $scope2.item.path;
            if (__av65 == null || __av65 === false || __av65 === "")
              __e64.removeAttribute("data-path");
            else
              __e64.setAttribute("data-path", __av65 === true ? "" : String(__av65));
          }
          {
            const __av67 = $scope2.item.icon;
            if (__av67 == null || __av67 === false || __av67 === "")
              __e66.removeAttribute("name");
            else
              __e66.setAttribute("name", __av67 === true ? "" : String(__av67));
          }
          __t69.textContent = $scope2.item.label;
          __t71.textContent = $scope2.item.path;
          $scope2.__v.set((c) => c + 1);
        } };
      })(__childScope61);
    }
  });
  const __wireBoundary_b21 = (anchor) => bindIf3({
    anchor,
    test: () => {
      if ($scope.findItems().length > 0)
        return 0;
      return 1;
    },
    branches: [
      () => {
        const __frag55 = document.createDocumentFragment();
        const __c56 = [];
        const __e57 = document.createElement("div");
        __e57.setAttribute("class", "cb-mode-hint");
        const __t58 = document.createTextNode("Jump to");
        __e57.appendChild(__t58);
        __frag55.appendChild(__e57);
        const __e59 = document.createElement("div");
        __e59.setAttribute("class", "cb-list");
        __e59.setAttribute("role", "listbox");
        const __forAnchor60 = document.createComment("for");
        __e59.appendChild(__forAnchor60);
        __c56.push(__wireBoundary_b22(__forAnchor60));
        __frag55.appendChild(__e59);
        return { nodes: Array.from(__frag55.childNodes), cleanup: () => {
          for (const c of __c56)
            try {
              c();
            } catch (e) {}
        } };
      },
      () => {
        const __frag74 = document.createDocumentFragment();
        const __c75 = [];
        const __e76 = document.createElement("div");
        __e76.setAttribute("class", "cb-empty");
        const __t77 = document.createTextNode("No matches.");
        __e76.appendChild(__t77);
        __frag74.appendChild(__e76);
        return { nodes: Array.from(__frag74.childNodes), cleanup: () => {
          for (const c of __c75)
            try {
              c();
            } catch (e) {}
        } };
      }
    ]
  });
  const __wireBoundary_b30 = (anchor) => bindFor2({
    anchor,
    items: () => $scope.cmdItems(),
    key: (item, __i) => {
      const __child = Object.create($scope);
      __child.item = item;
      __child.__i = __i;
      return (($scope2) => item.key)(__child);
    },
    render: (item, __i) => {
      const __childScope87 = Object.create($scope);
      __childScope87.item = item;
      __childScope87.__i = __i;
      __childScope87.__v = signal4(0);
      return (($scope2) => {
        const __frag88 = document.createDocumentFragment();
        const __c89 = [];
        const __e90 = document.createElement("div");
        __e90.setAttribute("class", "cb-item");
        __e90.setAttribute("role", "option");
        __c89.push(bindAttr2(__e90, "aria-selected", () => (void $scope2.__v(), $scope2.bar.selectedIdx() === $scope2.item.idx ? "true" : "false")));
        {
          const __av91 = $scope2.item.key;
          if (__av91 == null || __av91 === false || __av91 === "")
            __e90.removeAttribute("data-cmd");
          else
            __e90.setAttribute("data-cmd", __av91 === true ? "" : String(__av91));
        }
        __c89.push(on3(__e90, "click", function() {
          const __h = () => $scope2.activateCmd($scope2.item.key);
          return typeof __h === "function" ? __h.bind(undefined) : () => {};
        }(), { modifiers: [] }));
        const __e92 = document.createElement("span");
        __e92.setAttribute("style", "font-family:var(--font-mono)");
        const __t93 = document.createTextNode("");
        __e92.appendChild(__t93);
        __t93.textContent = $scope2.item.label;
        __e90.appendChild(__e92);
        const __e94 = document.createElement("span");
        __e90.appendChild(__e94);
        const __e95 = document.createElement("span");
        __e95.setAttribute("class", "hint");
        const __t96 = document.createTextNode("");
        __e95.appendChild(__t96);
        __t96.textContent = $scope2.item.hint;
        __e90.appendChild(__e95);
        __frag88.appendChild(__e90);
        return { nodes: Array.from(__frag88.childNodes), cleanup: () => {
          for (const c of __c89)
            try {
              c();
            } catch (e) {}
        }, update: (__new97, __newI98) => {
          $scope2.item = __new97;
          $scope2.__i = __newI98;
          {
            const __av91 = $scope2.item.key;
            if (__av91 == null || __av91 === false || __av91 === "")
              __e90.removeAttribute("data-cmd");
            else
              __e90.setAttribute("data-cmd", __av91 === true ? "" : String(__av91));
          }
          __t93.textContent = $scope2.item.label;
          __t96.textContent = $scope2.item.hint;
          $scope2.__v.set((c) => c + 1);
        } };
      })(__childScope87);
    }
  });
  const __wireBoundary_b29 = (anchor) => bindIf3({
    anchor,
    test: () => {
      if ($scope.cmdItems().length > 0)
        return 0;
      return 1;
    },
    branches: [
      () => {
        const __frag81 = document.createDocumentFragment();
        const __c82 = [];
        const __e83 = document.createElement("div");
        __e83.setAttribute("class", "cb-mode-hint");
        const __t84 = document.createTextNode("Actions");
        __e83.appendChild(__t84);
        __frag81.appendChild(__e83);
        const __e85 = document.createElement("div");
        __e85.setAttribute("class", "cb-list");
        __e85.setAttribute("role", "listbox");
        const __forAnchor86 = document.createComment("for");
        __e85.appendChild(__forAnchor86);
        __c82.push(__wireBoundary_b30(__forAnchor86));
        __frag81.appendChild(__e85);
        return { nodes: Array.from(__frag81.childNodes), cleanup: () => {
          for (const c of __c82)
            try {
              c();
            } catch (e) {}
        } };
      },
      () => {
        const __frag99 = document.createDocumentFragment();
        const __c100 = [];
        const __e101 = document.createElement("div");
        __e101.setAttribute("class", "cb-empty");
        const __t102 = document.createTextNode("No commands match.");
        __e101.appendChild(__t102);
        __frag99.appendChild(__e101);
        return { nodes: Array.from(__frag99.childNodes), cleanup: () => {
          for (const c of __c100)
            try {
              c();
            } catch (e) {}
        } };
      }
    ]
  });
  const __wireBoundary_b37 = (anchor) => bindFor2({
    anchor,
    items: () => $scope.mentionItems(),
    key: (item, __i) => {
      const __child = Object.create($scope);
      __child.item = item;
      __child.__i = __i;
      return (($scope2) => item.id + item.kind)(__child);
    },
    render: (item, __i) => {
      const __childScope112 = Object.create($scope);
      __childScope112.item = item;
      __childScope112.__i = __i;
      __childScope112.__v = signal4(0);
      return (($scope2) => {
        const __frag113 = document.createDocumentFragment();
        const __c114 = [];
        const __e115 = document.createElement("div");
        __e115.setAttribute("class", "cb-item");
        __e115.setAttribute("role", "option");
        __c114.push(bindAttr2(__e115, "aria-selected", () => (void $scope2.__v(), $scope2.bar.selectedIdx() === $scope2.item.idx ? "true" : "false")));
        {
          const __av116 = $scope2.item.kind;
          if (__av116 == null || __av116 === false || __av116 === "")
            __e115.removeAttribute("data-mention-kind");
          else
            __e115.setAttribute("data-mention-kind", __av116 === true ? "" : String(__av116));
        }
        {
          const __av117 = $scope2.item.id;
          if (__av117 == null || __av117 === false || __av117 === "")
            __e115.removeAttribute("data-mention-id");
          else
            __e115.setAttribute("data-mention-id", __av117 === true ? "" : String(__av117));
        }
        __c114.push(on3(__e115, "click", function() {
          const __h = () => $scope2.activateMention($scope2.item);
          return typeof __h === "function" ? __h.bind(undefined) : () => {};
        }(), { modifiers: [] }));
        const __e118 = document.createElement("sui-icon");
        {
          const __av119 = $scope2.item.icon;
          if (__av119 == null || __av119 === false || __av119 === "")
            __e118.removeAttribute("name");
          else
            __e118.setAttribute("name", __av119 === true ? "" : String(__av119));
        }
        __e118.setAttribute("size", "sm");
        __e118.setAttribute("aria-hidden", "true");
        __e115.appendChild(__e118);
        const __e120 = document.createElement("span");
        const __t121 = document.createTextNode("");
        __e120.appendChild(__t121);
        __t121.textContent = $scope2.item.label;
        __e115.appendChild(__e120);
        const __e122 = document.createElement("span");
        __e122.setAttribute("class", "hint");
        const __t123 = document.createTextNode("");
        __e122.appendChild(__t123);
        __t123.textContent = $scope2.item.kind;
        __e115.appendChild(__e122);
        __frag113.appendChild(__e115);
        return { nodes: Array.from(__frag113.childNodes), cleanup: () => {
          for (const c of __c114)
            try {
              c();
            } catch (e) {}
        }, update: (__new124, __newI125) => {
          $scope2.item = __new124;
          $scope2.__i = __newI125;
          {
            const __av116 = $scope2.item.kind;
            if (__av116 == null || __av116 === false || __av116 === "")
              __e115.removeAttribute("data-mention-kind");
            else
              __e115.setAttribute("data-mention-kind", __av116 === true ? "" : String(__av116));
          }
          {
            const __av117 = $scope2.item.id;
            if (__av117 == null || __av117 === false || __av117 === "")
              __e115.removeAttribute("data-mention-id");
            else
              __e115.setAttribute("data-mention-id", __av117 === true ? "" : String(__av117));
          }
          {
            const __av119 = $scope2.item.icon;
            if (__av119 == null || __av119 === false || __av119 === "")
              __e118.removeAttribute("name");
            else
              __e118.setAttribute("name", __av119 === true ? "" : String(__av119));
          }
          __t121.textContent = $scope2.item.label;
          __t123.textContent = $scope2.item.kind;
          $scope2.__v.set((c) => c + 1);
        } };
      })(__childScope112);
    }
  });
  const __wireBoundary_b36 = (anchor) => bindIf3({
    anchor,
    test: () => {
      if ($scope.mentionItems().length > 0)
        return 0;
      return 1;
    },
    branches: [
      () => {
        const __frag106 = document.createDocumentFragment();
        const __c107 = [];
        const __e108 = document.createElement("div");
        __e108.setAttribute("class", "cb-mode-hint");
        const __t109 = document.createTextNode("Reference");
        __e108.appendChild(__t109);
        __frag106.appendChild(__e108);
        const __e110 = document.createElement("div");
        __e110.setAttribute("class", "cb-list");
        __e110.setAttribute("role", "listbox");
        const __forAnchor111 = document.createComment("for");
        __e110.appendChild(__forAnchor111);
        __c107.push(__wireBoundary_b37(__forAnchor111));
        __frag106.appendChild(__e110);
        return { nodes: Array.from(__frag106.childNodes), cleanup: () => {
          for (const c of __c107)
            try {
              c();
            } catch (e) {}
        } };
      },
      () => {
        const __frag126 = document.createDocumentFragment();
        const __c127 = [];
        const __e128 = document.createElement("div");
        __e128.setAttribute("class", "cb-empty");
        const __t129 = document.createTextNode("No matching views or agents.");
        __e128.appendChild(__t129);
        __frag126.appendChild(__e128);
        return { nodes: Array.from(__frag126.childNodes), cleanup: () => {
          for (const c of __c127)
            try {
              c();
            } catch (e) {}
        } };
      }
    ]
  });
  const __wireBoundary_b45 = (anchor) => bindFor2({
    anchor,
    items: () => $scope.createSurfaces(),
    key: (surf, __i) => {
      const __child = Object.create($scope);
      __child.surf = surf;
      __child.__i = __i;
      return (($scope2) => surf.id)(__child);
    },
    render: (surf, __i) => {
      const __childScope136 = Object.create($scope);
      __childScope136.surf = surf;
      __childScope136.__i = __i;
      __childScope136.__v = signal4(0);
      return (($scope2) => {
        const __frag137 = document.createDocumentFragment();
        const __c138 = [];
        const __e139 = document.createElement("button");
        __e139.setAttribute("class", "cb-surface-btn");
        __e139.setAttribute("type", "button");
        __c138.push(bindAttr2(__e139, "aria-pressed", () => (void $scope2.__v(), $scope2.bar.createSurface() === $scope2.surf.id ? "true" : "false")));
        __c138.push(on3(__e139, "click", function() {
          const __h = () => $scope2.selectSurface($scope2.surf.id);
          return typeof __h === "function" ? __h.bind(undefined) : () => {};
        }(), { modifiers: [] }));
        const __e140 = document.createElement("sui-icon");
        {
          const __av141 = $scope2.surf.icon;
          if (__av141 == null || __av141 === false || __av141 === "")
            __e140.removeAttribute("name");
          else
            __e140.setAttribute("name", __av141 === true ? "" : String(__av141));
        }
        __e140.setAttribute("aria-hidden", "true");
        __e139.appendChild(__e140);
        const __e142 = document.createElement("span");
        const __t143 = document.createTextNode("");
        __e142.appendChild(__t143);
        __t143.textContent = $scope2.surf.label;
        __e139.appendChild(__e142);
        __frag137.appendChild(__e139);
        return { nodes: Array.from(__frag137.childNodes), cleanup: () => {
          for (const c of __c138)
            try {
              c();
            } catch (e) {}
        }, update: (__new144, __newI145) => {
          $scope2.surf = __new144;
          $scope2.__i = __newI145;
          {
            const __av141 = $scope2.surf.icon;
            if (__av141 == null || __av141 === false || __av141 === "")
              __e140.removeAttribute("name");
            else
              __e140.setAttribute("name", __av141 === true ? "" : String(__av141));
          }
          __t143.textContent = $scope2.surf.label;
          $scope2.__v.set((c) => c + 1);
        } };
      })(__childScope136);
    }
  });
  const __wireBoundary_b54 = (anchor) => bindFor2({
    anchor,
    items: () => $scope.currentSurfaceDef().shapes,
    key: (sh, __i) => {
      const __child = Object.create($scope);
      __child.sh = sh;
      __child.__i = __i;
      return (($scope2) => sh)(__child);
    },
    render: (sh, __i) => {
      const __childScope156 = Object.create($scope);
      __childScope156.sh = sh;
      __childScope156.__i = __i;
      __childScope156.__v = signal4(0);
      return (($scope2) => {
        const __frag157 = document.createDocumentFragment();
        const __c158 = [];
        const __e159 = document.createElement("option");
        {
          const __av160 = $scope2.sh;
          if (__av160 == null || __av160 === false || __av160 === "")
            __e159.removeAttribute("value");
          else
            __e159.setAttribute("value", __av160 === true ? "" : String(__av160));
        }
        const __t161 = document.createTextNode("");
        __e159.appendChild(__t161);
        __t161.textContent = $scope2.sh;
        __frag157.appendChild(__e159);
        return { nodes: Array.from(__frag157.childNodes), cleanup: () => {
          for (const c of __c158)
            try {
              c();
            } catch (e) {}
        }, update: (__new162, __newI163) => {
          $scope2.sh = __new162;
          $scope2.__i = __newI163;
          {
            const __av160 = $scope2.sh;
            if (__av160 == null || __av160 === false || __av160 === "")
              __e159.removeAttribute("value");
            else
              __e159.setAttribute("value", __av160 === true ? "" : String(__av160));
          }
          __t161.textContent = $scope2.sh;
          $scope2.__v.set((c) => c + 1);
        } };
      })(__childScope156);
    }
  });
  const __wireBoundary_b58 = (anchor) => bindIf3({
    anchor,
    test: () => {
      if ($scope.bar.createError())
        return 0;
      return -1;
    },
    branches: [
      () => {
        const __frag169 = document.createDocumentFragment();
        const __c170 = [];
        const __e171 = document.createElement("div");
        __e171.setAttribute("class", "cb-create-error");
        const __t172 = document.createTextNode("");
        __e171.appendChild(__t172);
        __c170.push(bindText3(__t172, () => $scope.bar.createError()));
        __frag169.appendChild(__e171);
        return { nodes: Array.from(__frag169.childNodes), cleanup: () => {
          for (const c of __c170)
            try {
              c();
            } catch (e) {}
        } };
      }
    ]
  });
  const __wireBoundary_b65 = (anchor) => bindFor2({
    anchor,
    items: () => $scope.bar.agents(),
    key: (a, __i) => {
      const __child = Object.create($scope);
      __child.a = a;
      __child.__i = __i;
      return (($scope2) => a.id)(__child);
    },
    render: (a, __i) => {
      const __childScope193 = Object.create($scope);
      __childScope193.a = a;
      __childScope193.__i = __i;
      __childScope193.__v = signal4(0);
      return (($scope2) => {
        const __frag194 = document.createDocumentFragment();
        const __c195 = [];
        const __e196 = document.createElement("option");
        {
          const __av197 = $scope2.a.id;
          if (__av197 == null || __av197 === false || __av197 === "")
            __e196.removeAttribute("value");
          else
            __e196.setAttribute("value", __av197 === true ? "" : String(__av197));
        }
        const __t198 = document.createTextNode("");
        __e196.appendChild(__t198);
        __c195.push(bindText3(__t198, () => (void $scope2.__v(), $scope2.a.name ?? $scope2.a.label ?? $scope2.a.id)));
        __frag194.appendChild(__e196);
        return { nodes: Array.from(__frag194.childNodes), cleanup: () => {
          for (const c of __c195)
            try {
              c();
            } catch (e) {}
        }, update: (__new199, __newI200) => {
          $scope2.a = __new199;
          $scope2.__i = __newI200;
          {
            const __av197 = $scope2.a.id;
            if (__av197 == null || __av197 === false || __av197 === "")
              __e196.removeAttribute("value");
            else
              __e196.setAttribute("value", __av197 === true ? "" : String(__av197));
          }
          $scope2.__v.set((c) => c + 1);
        } };
      })(__childScope193);
    }
  });
  const __wireBoundary_b64 = (anchor) => bindIf3({
    anchor,
    test: () => {
      if ($scope.bar.agents().length === 0)
        return 0;
      return 1;
    },
    branches: [
      () => {
        const __frag186 = document.createDocumentFragment();
        const __c187 = [];
        const __e188 = document.createElement("option");
        __e188.setAttribute("value", "");
        const __t189 = document.createTextNode("No agents found");
        __e188.appendChild(__t189);
        __frag186.appendChild(__e188);
        return { nodes: Array.from(__frag186.childNodes), cleanup: () => {
          for (const c of __c187)
            try {
              c();
            } catch (e) {}
        } };
      },
      () => {
        const __frag190 = document.createDocumentFragment();
        const __c191 = [];
        const __forAnchor192 = document.createComment("for");
        __frag190.appendChild(__forAnchor192);
        __c191.push(__wireBoundary_b65(__forAnchor192));
        return { nodes: Array.from(__frag190.childNodes), cleanup: () => {
          for (const c of __c191)
            try {
              c();
            } catch (e) {}
        } };
      }
    ]
  });
  const __wireBoundary_b70 = (anchor) => bindFor2({
    anchor,
    items: () => $scope.bar.providers(),
    key: (p, __i) => {
      const __child = Object.create($scope);
      __child.p = p;
      __child.__i = __i;
      return (($scope2) => p)(__child);
    },
    render: (p, __i) => {
      const __childScope213 = Object.create($scope);
      __childScope213.p = p;
      __childScope213.__i = __i;
      __childScope213.__v = signal4(0);
      return (($scope2) => {
        const __frag214 = document.createDocumentFragment();
        const __c215 = [];
        const __e216 = document.createElement("option");
        {
          const __av217 = $scope2.p;
          if (__av217 == null || __av217 === false || __av217 === "")
            __e216.removeAttribute("value");
          else
            __e216.setAttribute("value", __av217 === true ? "" : String(__av217));
        }
        const __t218 = document.createTextNode("");
        __e216.appendChild(__t218);
        __t218.textContent = $scope2.p;
        __frag214.appendChild(__e216);
        return { nodes: Array.from(__frag214.childNodes), cleanup: () => {
          for (const c of __c215)
            try {
              c();
            } catch (e) {}
        }, update: (__new219, __newI220) => {
          $scope2.p = __new219;
          $scope2.__i = __newI220;
          {
            const __av217 = $scope2.p;
            if (__av217 == null || __av217 === false || __av217 === "")
              __e216.removeAttribute("value");
            else
              __e216.setAttribute("value", __av217 === true ? "" : String(__av217));
          }
          __t218.textContent = $scope2.p;
          $scope2.__v.set((c) => c + 1);
        } };
      })(__childScope213);
    }
  });
  const __wireBoundary_b69 = (anchor) => bindIf3({
    anchor,
    test: () => {
      if ($scope.bar.providers().length === 0)
        return 0;
      return 1;
    },
    branches: [
      () => {
        const __frag206 = document.createDocumentFragment();
        const __c207 = [];
        const __e208 = document.createElement("option");
        __e208.setAttribute("value", "");
        const __t209 = document.createTextNode("No providers installed");
        __e208.appendChild(__t209);
        __frag206.appendChild(__e208);
        return { nodes: Array.from(__frag206.childNodes), cleanup: () => {
          for (const c of __c207)
            try {
              c();
            } catch (e) {}
        } };
      },
      () => {
        const __frag210 = document.createDocumentFragment();
        const __c211 = [];
        const __forAnchor212 = document.createComment("for");
        __frag210.appendChild(__forAnchor212);
        __c211.push(__wireBoundary_b70(__forAnchor212));
        return { nodes: Array.from(__frag210.childNodes), cleanup: () => {
          for (const c of __c211)
            try {
              c();
            } catch (e) {}
        } };
      }
    ]
  });
  const __wireBoundary_b73 = (anchor) => bindIf3({
    anchor,
    test: () => {
      if ($scope.bar.spawnError())
        return 0;
      return -1;
    },
    branches: [
      () => {
        const __frag222 = document.createDocumentFragment();
        const __c223 = [];
        const __e224 = document.createElement("div");
        __e224.setAttribute("class", "cb-spawn-error");
        const __t225 = document.createTextNode("");
        __e224.appendChild(__t225);
        __c223.push(bindText3(__t225, () => $scope.bar.spawnError()));
        __frag222.appendChild(__e224);
        return { nodes: Array.from(__frag222.childNodes), cleanup: () => {
          for (const c of __c223)
            try {
              c();
            } catch (e) {}
        } };
      }
    ]
  });
  const __wireBoundary_b20 = (anchor) => bindIf3({
    anchor,
    test: () => {
      const __d = $scope.bar.mode();
      if (__d === "find")
        return 0;
      if (__d === "command")
        return 1;
      if (__d === "mention")
        return 2;
      if (__d === "create")
        return 3;
      return 4;
    },
    branches: [
      () => {
        const __frag52 = document.createDocumentFragment();
        const __c53 = [];
        const __ifAnchor54 = document.createComment("if");
        __frag52.appendChild(__ifAnchor54);
        __c53.push(__wireBoundary_b21(__ifAnchor54));
        return { nodes: Array.from(__frag52.childNodes), cleanup: () => {
          for (const c of __c53)
            try {
              c();
            } catch (e) {}
        } };
      },
      () => {
        const __frag78 = document.createDocumentFragment();
        const __c79 = [];
        const __ifAnchor80 = document.createComment("if");
        __frag78.appendChild(__ifAnchor80);
        __c79.push(__wireBoundary_b29(__ifAnchor80));
        return { nodes: Array.from(__frag78.childNodes), cleanup: () => {
          for (const c of __c79)
            try {
              c();
            } catch (e) {}
        } };
      },
      () => {
        const __frag103 = document.createDocumentFragment();
        const __c104 = [];
        const __ifAnchor105 = document.createComment("if");
        __frag103.appendChild(__ifAnchor105);
        __c104.push(__wireBoundary_b36(__ifAnchor105));
        return { nodes: Array.from(__frag103.childNodes), cleanup: () => {
          for (const c of __c104)
            try {
              c();
            } catch (e) {}
        } };
      },
      () => {
        const __frag130 = document.createDocumentFragment();
        const __c131 = [];
        const __e132 = document.createElement("div");
        __e132.setAttribute("class", "cb-mode-hint");
        const __t133 = document.createTextNode("Create");
        __e132.appendChild(__t133);
        __frag130.appendChild(__e132);
        const __e134 = document.createElement("div");
        __e134.setAttribute("class", "cb-create-surfaces");
        __e134.setAttribute("role", "group");
        __e134.setAttribute("aria-label", "Surface type");
        const __forAnchor135 = document.createComment("for");
        __e134.appendChild(__forAnchor135);
        __c131.push(__wireBoundary_b45(__forAnchor135));
        __frag130.appendChild(__e134);
        const __e146 = document.createElement("div");
        __e146.setAttribute("class", "cb-create-grid");
        const __e147 = document.createElement("label");
        const __e148 = document.createElement("span");
        const __t149 = document.createTextNode("Name (kebab-case)");
        __e148.appendChild(__t149);
        __e147.appendChild(__e148);
        const __e150 = document.createElement("input");
        __e150.setAttribute("type", "text");
        __e150.setAttribute("class", "cb-create-name");
        __c131.push(bindAttr2(__e150, "placeholder", () => "my-" + $scope.bar.createSurface()));
        __c131.push(bindValue2(__e150, $scope.bar.createName));
        __e150.setAttribute("autocomplete", "off");
        __e147.appendChild(__e150);
        __e146.appendChild(__e147);
        const __e151 = document.createElement("label");
        const __e152 = document.createElement("span");
        const __t153 = document.createTextNode("");
        __e152.appendChild(__t153);
        __c131.push(bindText3(__t153, () => $scope.currentSurfaceDef().shapeLabel));
        __e151.appendChild(__e152);
        const __e154 = document.createElement("select");
        __e154.setAttribute("class", "cb-create-shape");
        __c131.push(bindValue2(__e154, $scope.bar.createShape));
        const __forAnchor155 = document.createComment("for");
        __e154.appendChild(__forAnchor155);
        __c131.push(__wireBoundary_b54(__forAnchor155));
        __e151.appendChild(__e154);
        __e146.appendChild(__e151);
        const __e164 = document.createElement("label");
        __e164.setAttribute("class", "span-2");
        const __e165 = document.createElement("span");
        const __t166 = document.createTextNode("Purpose (optional)");
        __e165.appendChild(__t166);
        __e164.appendChild(__e165);
        const __e167 = document.createElement("input");
        __e167.setAttribute("type", "text");
        __e167.setAttribute("class", "cb-create-purpose");
        __e167.setAttribute("placeholder", "One-line description");
        __c131.push(bindValue2(__e167, $scope.bar.createPurpose));
        __e167.setAttribute("autocomplete", "off");
        __e164.appendChild(__e167);
        __e146.appendChild(__e164);
        __frag130.appendChild(__e146);
        const __ifAnchor168 = document.createComment("if");
        __frag130.appendChild(__ifAnchor168);
        __c131.push(__wireBoundary_b58(__ifAnchor168));
        const __e173 = document.createElement("div");
        __e173.setAttribute("class", "cb-create-actions");
        const __e174 = document.createElement("button");
        __e174.setAttribute("class", "cb-create-submit");
        __e174.setAttribute("type", "button");
        __c131.push(bindAttr2(__e174, "disabled", () => $scope.bar.createSubmitting() || !$scope.createNameValid()));
        __c131.push(on3(__e174, "click", function() {
          const __h = $scope.submitCreate;
          return typeof __h === "function" ? __h.bind(undefined) : () => {};
        }(), { modifiers: [] }));
        const __t175 = document.createTextNode("");
        __e174.appendChild(__t175);
        __c131.push(bindText3(__t175, () => $scope.bar.createSubmitting() ? "Creating…" : "Create " + $scope.currentSurfaceDef().label));
        __e173.appendChild(__e174);
        __frag130.appendChild(__e173);
        return { nodes: Array.from(__frag130.childNodes), cleanup: () => {
          for (const c of __c131)
            try {
              c();
            } catch (e) {}
        } };
      },
      () => {
        const __frag176 = document.createDocumentFragment();
        const __c177 = [];
        const __e178 = document.createElement("div");
        __e178.setAttribute("class", "cb-spawn-card");
        const __e179 = document.createElement("div");
        __e179.setAttribute("class", "cb-spawn-title");
        const __t180 = document.createTextNode("Start a session");
        __e179.appendChild(__t180);
        __e178.appendChild(__e179);
        const __e181 = document.createElement("div");
        __e181.setAttribute("class", "cb-spawn-row");
        const __e182 = document.createElement("label");
        const __t183 = document.createTextNode("Agent");
        __e182.appendChild(__t183);
        __e181.appendChild(__e182);
        const __e184 = document.createElement("select");
        __e184.setAttribute("class", "cb-spawn-agent");
        __c177.push(bindValue2(__e184, $scope.bar.spawnAgent));
        const __ifAnchor185 = document.createComment("if");
        __e184.appendChild(__ifAnchor185);
        __c177.push(__wireBoundary_b64(__ifAnchor185));
        __e181.appendChild(__e184);
        __e178.appendChild(__e181);
        const __e201 = document.createElement("div");
        __e201.setAttribute("class", "cb-spawn-row");
        const __e202 = document.createElement("label");
        const __t203 = document.createTextNode("Provider");
        __e202.appendChild(__t203);
        __e201.appendChild(__e202);
        const __e204 = document.createElement("select");
        __e204.setAttribute("class", "cb-spawn-provider");
        __c177.push(bindValue2(__e204, $scope.bar.spawnProvider));
        const __ifAnchor205 = document.createComment("if");
        __e204.appendChild(__ifAnchor205);
        __c177.push(__wireBoundary_b69(__ifAnchor205));
        __e201.appendChild(__e204);
        __e178.appendChild(__e201);
        const __ifAnchor221 = document.createComment("if");
        __e178.appendChild(__ifAnchor221);
        __c177.push(__wireBoundary_b73(__ifAnchor221));
        const __e226 = document.createElement("button");
        __e226.setAttribute("class", "cb-spawn-start");
        __e226.setAttribute("type", "button");
        __c177.push(bindAttr2(__e226, "disabled", () => $scope.bar.spawning() || !$scope.bar.spawnAgent() || !$scope.bar.spawnProvider()));
        __c177.push(on3(__e226, "click", function() {
          const __h = $scope.spawnSession;
          return typeof __h === "function" ? __h.bind(undefined) : () => {};
        }(), { modifiers: [] }));
        const __t227 = document.createTextNode("");
        __e226.appendChild(__t227);
        __c177.push(bindText3(__t227, () => $scope.bar.spawning() ? "Starting…" : "Start session"));
        __e178.appendChild(__e226);
        __frag176.appendChild(__e178);
        return { nodes: Array.from(__frag176.childNodes), cleanup: () => {
          for (const c of __c177)
            try {
              c();
            } catch (e) {}
        } };
      }
    ]
  });
  return Object.assign({}, $scope, {
    __mount(container) {
      const __cleanups = [];
      const __e0 = document.createElement("div");
      __e0.setAttribute("class", "cb-input-row");
      const __e1 = document.createElement("span");
      __e1.setAttribute("class", "cb-mode-glyph");
      __cleanups.push(bindClass2(__e1, "ai", () => $scope.bar.mode() === "ai"));
      const __t2 = document.createTextNode("");
      __e1.appendChild(__t2);
      __cleanups.push(bindText3(__t2, () => $scope.modeGlyph()));
      __e0.appendChild(__e1);
      const __e3 = document.createElement("input");
      __e3.setAttribute("class", "cb-search");
      __e3.setAttribute("type", "text");
      __e3.setAttribute("autocomplete", "off");
      __e3.setAttribute("spellcheck", "false");
      __cleanups.push(bindAttr2(__e3, "placeholder", () => $scope.searchPlaceholder()));
      __cleanups.push(bindValue2(__e3, $scope.bar.value));
      __e3.setAttribute("aria-label", "Command bar search");
      __e3.setAttribute("aria-autocomplete", "list");
      __cleanups.push(on3(__e3, "input", function() {
        const __h = $scope.onInput;
        return typeof __h === "function" ? __h.bind(undefined) : () => {};
      }(), { modifiers: [] }));
      __e0.appendChild(__e3);
      const __e4 = document.createElement("kbd");
      __e4.setAttribute("class", "cb-esc");
      __e4.setAttribute("aria-label", "Escape to close");
      const __t5 = document.createTextNode("esc");
      __e4.appendChild(__t5);
      __e0.appendChild(__e4);
      const __e6 = document.createElement("div");
      __e6.setAttribute("class", "cb-chip-wrap");
      const __e7 = document.createElement("span");
      __e7.setAttribute("class", "cb-session-chip");
      __e7.setAttribute("role", "button");
      __e7.setAttribute("aria-haspopup", "listbox");
      __cleanups.push(on3(__e7, "click", function() {
        const __h = $scope.toggleSessionPicker;
        return typeof __h === "function" ? __h.bind(undefined) : () => {};
      }(), { modifiers: [] }));
      const __e8 = document.createElement("span");
      __e8.setAttribute("class", "cb-dot");
      __cleanups.push(effect4(() => {
        const __v = $scope.chipDotBg();
        if (__v == null)
          __e8.style.removeProperty("background");
        else
          __e8.style.setProperty("background", String(__v));
      }));
      __e7.appendChild(__e8);
      const __e9 = document.createElement("span");
      const __t10 = document.createTextNode("");
      __e9.appendChild(__t10);
      __cleanups.push(bindText3(__t10, () => $scope.chipLabel()));
      __e7.appendChild(__e9);
      __e6.appendChild(__e7);
      const __ifAnchor11 = document.createComment("if");
      __e6.appendChild(__ifAnchor11);
      __cleanups.push(__wireBoundary_b8(__ifAnchor11));
      __e0.appendChild(__e6);
      container.appendChild(__e0);
      const __e50 = document.createElement("div");
      __e50.setAttribute("class", "cb-body");
      const __matchAnchor51 = document.createComment("match");
      __e50.appendChild(__matchAnchor51);
      __cleanups.push(__wireBoundary_b20(__matchAnchor51));
      container.appendChild(__e50);
      return () => {
        for (const c of __cleanups)
          try {
            c();
          } catch (e) {}
      };
    },
    hydrate(root, ctx2) {
      const __cleanups = [];
      const __t = collectTextOpens2(root);
      const __m = collectElementMarkers2(root);
      if (__m["b0"])
        __cleanups.push(hydrateClass(__m["b0"], "ai", () => $scope.bar.mode() === "ai"));
      if (__t["b1"])
        __cleanups.push(hydrateText2(__t["b1"], () => $scope.modeGlyph()));
      if (__m["b2"])
        __cleanups.push(hydrateAttr2(__m["b2"], "placeholder", () => $scope.searchPlaceholder()));
      if (__m["b3"])
        __cleanups.push(hydrateBindValue2(__m["b3"], $scope.bar.value));
      if (__m["b4"])
        __cleanups.push(hydrateOn2(__m["b4"], "input", function() {
          const __h = $scope.onInput;
          return typeof __h === "function" ? __h.bind(undefined) : () => {};
        }(), []));
      if (__m["b5"])
        __cleanups.push(hydrateOn2(__m["b5"], "click", function() {
          const __h = $scope.toggleSessionPicker;
          return typeof __h === "function" ? __h.bind(undefined) : () => {};
        }(), []));
      if (__m["b6"])
        __cleanups.push(hydrateStyle2(__m["b6"], "background", () => $scope.chipDotBg()));
      if (__t["b7"])
        __cleanups.push(hydrateText2(__t["b7"], () => $scope.chipLabel()));
      if (__m["b8"]) {
        const __t2 = __m["b8"];
        const __doc = __t2.ownerDocument ?? document;
        const __anchor = __t2.nodeType === 1 ? __doc.createComment("if") : __t2;
        if (__t2.nodeType === 1 && __t2.parentNode)
          __t2.parentNode.replaceChild(__anchor, __t2);
        __cleanups.push(__wireBoundary_b8(__anchor));
      }
      if (__m["b20"]) {
        const __t2 = __m["b20"];
        const __doc = __t2.ownerDocument ?? document;
        const __anchor = __t2.nodeType === 1 ? __doc.createComment("match") : __t2;
        if (__t2.nodeType === 1 && __t2.parentNode)
          __t2.parentNode.replaceChild(__anchor, __t2);
        __cleanups.push(__wireBoundary_b20(__anchor));
      }
      return () => {
        for (const c of __cleanups)
          try {
            c();
          } catch (e) {}
      };
    }
  });
}), { _compilerVersion: { cargo: "0.1.0", git: "d4751a31", dirty: true, builtAt: "2026-05-14T02:37:23Z" } });
