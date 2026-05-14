// packages/ui-components/src/islands/sui-command-bar/sui-command-bar.sui
import { adoptSuiSheets, attachRef, bindIf, bindText, component, effect as effect2, on, prop } from "@ship/sui-runtime";

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
var sui_command_bar_default = component("sui-command-bar", {
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
import { adoptSuiSheets as adoptSuiSheets2, bindAttr, bindClass, bindFor, bindIf as bindIf2, bindText as bindText2, bindValue, component as component2, effect as effect3, on as on2 } from "@ship/sui-runtime";

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
var sui_command_chat_default = component2("sui-command-chat", {
  shadow: true,
  styles: `
  :host { display: flex; flex-direction: column; flex: 1; min-height: 0; overflow: hidden; }

  /* ── Header ───────────────────────────────────────────────────── */
  .cb-chat-header {
    display: flex;
    align-items: center;
    gap: 10px;
    padding: 10px 14px;
    border-bottom: 1px solid var(--border);
    flex: 0 0 auto;
  }
  .cb-dot {
    width: 8px; height: 8px;
    border-radius: 999px;
    flex-shrink: 0;
  }
  .cb-chat-header .title { font-size: 13px; font-weight: 600; }
  .cb-chat-header .sub { font-size: 11px; color: var(--muted-foreground); }
  .cb-chip-wrap { position: relative; flex-shrink: 0; }
  .cb-session-chip {
    display: inline-flex;
    align-items: center;
    gap: 5px;
    padding: 2px 8px;
    border: 1px solid var(--border);
    border-radius: 999px;
    font-size: 11px;
    color: var(--muted-foreground);
    cursor: pointer;
    user-select: none;
  }
  .cb-session-chip:hover { background: var(--muted); }
  .cb-esc {
    font-family: var(--font-mono);
    font-size: 10px;
    padding: 1px 5px;
    border: 1px solid var(--border);
    border-radius: 3px;
    background: var(--muted);
    color: var(--muted-foreground);
  }

  /* ── Session picker ───────────────────────────────────────────── */
  .cb-session-popover {
    position: absolute;
    top: calc(100% + 4px);
    right: 0;
    min-width: 280px;
    background: var(--card);
    border: 1px solid var(--border);
    border-radius: var(--radius-md, 6px);
    box-shadow: var(--shadow-lg);
    z-index: 10;
    padding: 4px 0;
  }
  .cb-session-row {
    display: grid;
    grid-template-columns: 14px 1fr auto;
    align-items: center;
    gap: 8px;
    padding: 7px 12px;
    cursor: pointer;
    font-size: 12px;
  }
  .cb-session-row:hover { background: var(--muted); }
  .cb-session-row.current { background: color-mix(in oklch, var(--primary) 8%, transparent); }
  .cb-session-row .meta { color: var(--muted-foreground); font-size: 11px; }
  .cb-session-end {
    background: transparent; border: none; cursor: pointer;
    color: var(--muted-foreground); font-size: 14px; padding: 0 2px;
  }
  .cb-session-end:hover { color: var(--destructive, red); }
  .cb-session-sep { height: 1px; background: var(--border); margin: 4px 0; }
  .cb-session-action {
    display: flex; align-items: center; gap: 8px;
    padding: 7px 12px; cursor: pointer; font-size: 12px;
    color: var(--muted-foreground);
  }
  .cb-session-action:hover { background: var(--muted); color: var(--foreground); }

  /* ── Messages ─────────────────────────────────────────────────── */
  .cb-chat-messages {
    flex: 1;
    min-height: 0;
    overflow-y: auto;
    overscroll-behavior: contain;
    scrollbar-width: thin;
    padding: 12px 14px;
    display: flex;
    flex-direction: column;
    gap: 8px;
  }
  .cb-chat-empty { color: var(--muted-foreground); font-size: 13px; text-align: center; margin-top: 24px; }
  .cb-chat-turn { display: flex; flex-direction: column; }
  .cb-chat-turn[data-sender='user'] { align-items: flex-end; }
  .cb-chat-turn[data-sender='agent'] { align-items: flex-start; }
  .cb-chat-turn[data-sender='system'] { align-items: center; }
  .cb-chat-bubble {
    max-width: 80%;
    padding: 7px 10px;
    border-radius: var(--radius-md, 6px);
    font-size: 13px;
    line-height: 1.5;
    white-space: pre-wrap;
    word-break: break-word;
  }
  .cb-chat-turn[data-sender='user'] .cb-chat-bubble {
    background: var(--primary);
    color: var(--primary-foreground, #fff);
  }
  .cb-chat-turn[data-sender='agent'] .cb-chat-bubble {
    background: var(--muted);
    color: var(--foreground);
  }
  .cb-chat-turn[data-sender='system'] .cb-chat-bubble {
    background: transparent;
    color: var(--muted-foreground);
    font-size: 11px;
    font-family: var(--font-mono);
  }

  /* ── Composer ─────────────────────────────────────────────────── */
  .cb-chat-composer {
    display: flex;
    gap: 8px;
    align-items: flex-end;
    padding: 10px 14px;
    border-top: 1px solid var(--border);
    flex: 0 0 auto;
  }
  .cb-chat-input {
    flex: 1;
    border: 1px solid var(--border);
    border-radius: var(--radius-sm, 4px);
    background: var(--background);
    color: var(--foreground);
    font: inherit;
    font-size: 13px;
    padding: 6px 8px;
    resize: none;
    line-height: 1.4;
  }
  .cb-chat-input:focus { outline: none; border-color: var(--primary); }
  .cb-chat-send {
    height: 32px;
    padding: 0 14px;
    border: none;
    border-radius: var(--radius-sm, 4px);
    background: var(--primary);
    color: var(--primary-foreground, #fff);
    font: inherit;
    font-size: 13px;
    cursor: pointer;
    flex-shrink: 0;
    align-self: flex-end;
  }
  .cb-chat-send:disabled { opacity: 0.5; cursor: not-allowed; }
`,
  setup(ctx) {
    const { props, host } = ctx;
    if (!ctx.cleanups)
      ctx.cleanups = [];
    const __cleanups = ctx.cleanups;
    const $scope = Object.create(null);
    let __r = null;
    if (sui_command_chat_controller_default && typeof sui_command_chat_controller_default.setup === "function") {
      ctx.scope = $scope;
      __r = sui_command_chat_controller_default.setup(ctx);
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
        return (($scope2) => {
          const __frag19 = document.createDocumentFragment();
          const __c20 = [];
          const __e21 = document.createElement("div");
          __e21.setAttribute("class", "cb-session-row");
          __c20.push(bindClass(__e21, "current", () => $scope2.bar.session()?.id === $scope2.s.id));
          __e21.setAttribute("role", "option");
          __c20.push(on2(__e21, "click", function() {
            const __h = () => $scope2.selectSession($scope2.s.id);
            return typeof __h === "function" ? __h.bind(undefined) : () => {};
          }(), { modifiers: [] }));
          const __e22 = document.createElement("span");
          __e22.setAttribute("class", "cb-dot");
          __c20.push(effect3(() => {
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
          __c20.push(bindText2(__t25, () => $scope2.s.agent_id ?? ""));
          __e23.appendChild(__e24);
          const __e26 = document.createElement("span");
          __e26.setAttribute("class", "meta");
          const __t27 = document.createTextNode(" · ");
          __e26.appendChild(__t27);
          const __t28 = document.createTextNode("");
          __e26.appendChild(__t28);
          __c20.push(bindText2(__t28, () => $scope2.s.provider ?? ""));
          const __t29 = document.createTextNode(" · ");
          __e26.appendChild(__t29);
          const __t30 = document.createTextNode("");
          __e26.appendChild(__t30);
          __c20.push(bindText2(__t30, () => $scope2.s.workspace_id));
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
          const __ifAnchor33 = document.createComment("if");
          __e16.appendChild(__ifAnchor33);
          __c15.push(__wireBoundary_b14(__ifAnchor33));
          const __e41 = document.createElement("div");
          __e41.setAttribute("class", "cb-session-sep");
          __e41.setAttribute("role", "separator");
          __e16.appendChild(__e41);
          const __e42 = document.createElement("div");
          __e42.setAttribute("class", "cb-session-action");
          __c15.push(on2(__e42, "click", function() {
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
          __e16.appendChild(__e42);
          const __e46 = document.createElement("div");
          __e46.setAttribute("class", "cb-session-action");
          __c15.push(on2(__e46, "click", function() {
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
          __e16.appendChild(__e46);
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
        const __childScope61 = Object.create($scope);
        __childScope61.msg = msg;
        __childScope61.__i = __i;
        return (($scope2) => {
          const __frag62 = document.createDocumentFragment();
          const __c63 = [];
          const __e64 = document.createElement("div");
          __e64.setAttribute("class", "cb-chat-turn");
          __c63.push(bindAttr(__e64, "data-sender", () => $scope2.msg.sender));
          const __e65 = document.createElement("div");
          __e65.setAttribute("class", "cb-chat-bubble");
          const __t66 = document.createTextNode("");
          __e65.appendChild(__t66);
          __c63.push(bindText2(__t66, () => $scope2.msg.text));
          __e64.appendChild(__e65);
          __frag62.appendChild(__e64);
          return { nodes: Array.from(__frag62.childNodes), cleanup: () => {
            for (const c of __c63)
              try {
                c();
              } catch (e) {}
          } };
        })(__childScope61);
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
          const __frag54 = document.createDocumentFragment();
          const __c55 = [];
          const __e56 = document.createElement("div");
          __e56.setAttribute("class", "cb-chat-empty");
          const __t57 = document.createTextNode("Type below and press Enter. The agent's replies land here.");
          __e56.appendChild(__t57);
          __frag54.appendChild(__e56);
          return { nodes: Array.from(__frag54.childNodes), cleanup: () => {
            for (const c of __c55)
              try {
                c();
              } catch (e) {}
          } };
        },
        () => {
          const __frag58 = document.createDocumentFragment();
          const __c59 = [];
          const __forAnchor60 = document.createComment("for");
          __frag58.appendChild(__forAnchor60);
          __c59.push(__wireBoundary_b18(__forAnchor60));
          return { nodes: Array.from(__frag58.childNodes), cleanup: () => {
            for (const c of __c59)
              try {
                c();
              } catch (e) {}
          } };
        }
      ]
    });
    const __root = host.shadowRoot ?? host;
    adoptSuiSheets2(__root);
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
    __cleanups.push(bindText2(__t6, () => $scope.bar.session()?.provider ?? "â"));
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
    const __e50 = document.createElement("kbd");
    __e50.setAttribute("class", "cb-esc");
    __e50.setAttribute("aria-label", "Escape to close");
    const __t51 = document.createTextNode("esc");
    __e50.appendChild(__t51);
    __e0.appendChild(__e50);
    __root.appendChild(__e0);
    const __e52 = document.createElement("div");
    __e52.setAttribute("class", "cb-chat-messages");
    const __ifAnchor53 = document.createComment("if");
    __e52.appendChild(__ifAnchor53);
    __cleanups.push(__wireBoundary_b17(__ifAnchor53));
    __root.appendChild(__e52);
    const __e67 = document.createElement("div");
    __e67.setAttribute("class", "cb-chat-composer");
    const __e68 = document.createElement("textarea");
    __e68.setAttribute("class", "cb-chat-input");
    __e68.setAttribute("rows", "2");
    __e68.setAttribute("placeholder", "Message the agent…");
    __cleanups.push(bindValue(__e68, $scope.bar.value));
    __cleanups.push(on2(__e68, "keydown", function() {
      const __h = $scope.onComposerKeydown;
      return typeof __h === "function" ? __h.bind(undefined) : () => {};
    }(), { modifiers: [] }));
    __e67.appendChild(__e68);
    const __e69 = document.createElement("button");
    __e69.setAttribute("class", "cb-chat-send");
    __e69.setAttribute("type", "button");
    __cleanups.push(bindAttr(__e69, "disabled", () => !$scope.bar.value().trim()));
    __cleanups.push(on2(__e69, "click", function() {
      const __h = $scope.sendMessage;
      return typeof __h === "function" ? __h.bind(undefined) : () => {};
    }(), { modifiers: [] }));
    const __t70 = document.createTextNode("Send");
    __e69.appendChild(__t70);
    __e67.appendChild(__e69);
    __root.appendChild(__e67);
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

// packages/ui-components/src/islands/sui-command-bar/sui-command-palette.sui
import { adoptSuiSheets as adoptSuiSheets3, bindAttr as bindAttr2, bindClass as bindClass2, bindFor as bindFor2, bindIf as bindIf3, bindText as bindText3, bindValue as bindValue2, component as component3, effect as effect4, on as on3 } from "@ship/sui-runtime";

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
var sui_command_palette_default = component3("sui-command-palette", {
  shadow: true,
  styles: `
  :host { display: flex; flex-direction: column; flex: 1; min-height: 0; overflow: hidden; }

  /* ── Input row ────────────────────────────────────────────────── */
  .cb-input-row {
    display: flex;
    align-items: center;
    gap: 8px;
    padding: 10px 14px;
    border-bottom: 1px solid var(--border);
    flex: 0 0 auto;
  }
  .cb-mode-glyph {
    font-family: var(--font-mono);
    font-size: 13px;
    font-weight: 600;
    color: var(--muted-foreground);
    flex-shrink: 0;
    min-width: 16px;
    text-align: center;
  }
  .cb-mode-glyph.ai {
    font-size: 10px;
    padding: 1px 4px;
    border: 1px solid var(--border);
    border-radius: 3px;
    background: var(--muted);
  }
  .cb-search {
    flex: 1;
    border: none;
    outline: none;
    background: transparent;
    font: inherit;
    font-size: 14px;
    color: var(--foreground);
    caret-color: var(--foreground);
  }
  .cb-search::placeholder { color: var(--muted-foreground); }
  .cb-esc {
    font-family: var(--font-mono);
    font-size: 10px;
    padding: 1px 5px;
    border: 1px solid var(--border);
    border-radius: 3px;
    background: var(--muted);
    color: var(--muted-foreground);
    flex-shrink: 0;
  }

  /* ── Session chip ─────────────────────────────────────────────── */
  .cb-chip-wrap { position: relative; flex-shrink: 0; }
  .cb-session-chip {
    display: inline-flex;
    align-items: center;
    gap: 5px;
    padding: 2px 8px;
    border: 1px solid var(--border);
    border-radius: 999px;
    font-size: 11px;
    color: var(--muted-foreground);
    cursor: pointer;
    user-select: none;
    max-width: 140px;
    overflow: hidden;
    text-overflow: ellipsis;
    white-space: nowrap;
  }
  .cb-session-chip:hover { background: var(--muted); }
  .cb-dot {
    width: 6px; height: 6px;
    border-radius: 999px;
    flex-shrink: 0;
    display: inline-block;
  }

  /* ── Session popover ──────────────────────────────────────────── */
  .cb-session-popover {
    position: absolute;
    top: calc(100% + 4px);
    right: 0;
    min-width: 280px;
    background: var(--card);
    border: 1px solid var(--border);
    border-radius: var(--radius-md, 6px);
    box-shadow: var(--shadow-lg);
    z-index: 10;
    padding: 4px 0;
    overflow: hidden;
  }
  .cb-session-row {
    display: grid;
    grid-template-columns: 14px 1fr auto;
    align-items: center;
    gap: 8px;
    padding: 7px 12px;
    cursor: pointer;
    font-size: 12px;
  }
  .cb-session-row:hover { background: var(--muted); }
  .cb-session-row.current { background: color-mix(in oklch, var(--primary) 8%, transparent); }
  .cb-session-row .meta { color: var(--muted-foreground); font-size: 11px; }
  .cb-session-end {
    background: transparent; border: none; cursor: pointer;
    color: var(--muted-foreground); font-size: 14px; padding: 0 2px;
  }
  .cb-session-end:hover { color: var(--destructive, red); }
  .cb-session-sep { height: 1px; background: var(--border); margin: 4px 0; }
  .cb-session-action {
    display: flex; align-items: center; gap: 8px;
    padding: 7px 12px; cursor: pointer; font-size: 12px;
    color: var(--muted-foreground);
  }
  .cb-session-action:hover { background: var(--muted); color: var(--foreground); }

  /* ── Body / list ──────────────────────────────────────────────── */
  .cb-body {
    flex: 1;
    min-height: 0;
    overflow-y: auto;
    overscroll-behavior: contain;
    scrollbar-width: thin;
    padding: 6px 0;
  }
  .cb-mode-hint {
    padding: 4px 14px 2px;
    font-size: 11px;
    font-weight: 600;
    color: var(--muted-foreground);
    letter-spacing: 0.04em;
    text-transform: uppercase;
  }
  .cb-list { padding: 2px 6px; }
  .cb-item {
    display: grid;
    grid-template-columns: 16px 1fr auto;
    align-items: center;
    gap: 8px;
    padding: 7px 8px;
    border-radius: var(--radius-sm, 4px);
    cursor: pointer;
    font-size: 13px;
  }
  .cb-item:hover,
  .cb-item[aria-selected='true'] { background: var(--muted); }
  .cb-item .hint {
    color: var(--muted-foreground);
    font-size: 11px;
    font-family: var(--font-mono);
    overflow: hidden;
    text-overflow: ellipsis;
    white-space: nowrap;
  }
  .cb-empty {
    padding: 24px 16px;
    text-align: center;
    color: var(--muted-foreground);
    font-size: 13px;
  }

  /* ── Create form ──────────────────────────────────────────────── */
  .cb-create-surfaces {
    display: flex;
    flex-wrap: wrap;
    gap: 6px;
    padding: 8px 14px;
  }
  .cb-surface-btn {
    display: inline-flex;
    align-items: center;
    gap: 5px;
    padding: 4px 10px;
    border: 1px solid var(--border);
    border-radius: 999px;
    background: transparent;
    font: inherit;
    font-size: 12px;
    color: var(--muted-foreground);
    cursor: pointer;
  }
  .cb-surface-btn[aria-pressed='true'] {
    background: var(--primary);
    color: var(--primary-foreground, #fff);
    border-color: var(--primary);
  }
  .cb-surface-btn:hover:not([aria-pressed='true']) { background: var(--muted); }
  .cb-create-grid {
    display: grid;
    grid-template-columns: 1fr 1fr;
    gap: 10px;
    padding: 0 14px 10px;
  }
  .cb-create-grid label { display: flex; flex-direction: column; gap: 4px; font-size: 12px; color: var(--muted-foreground); }
  .cb-create-grid input,
  .cb-create-grid select {
    height: 30px;
    border: 1px solid var(--border);
    border-radius: var(--radius-sm, 4px);
    background: var(--background);
    color: var(--foreground);
    font: inherit;
    font-size: 13px;
    padding: 0 8px;
  }
  .span-2 { grid-column: span 2; }
  .cb-create-error { padding: 0 14px 6px; font-size: 12px; color: var(--destructive, red); }
  .cb-create-actions { padding: 0 14px 14px; display: flex; justify-content: flex-end; }
  .cb-create-submit {
    height: 32px;
    padding: 0 16px;
    border: none;
    border-radius: var(--radius-sm, 4px);
    background: var(--primary);
    color: var(--primary-foreground, #fff);
    font: inherit;
    font-size: 13px;
    cursor: pointer;
  }
  .cb-create-submit:disabled { opacity: 0.5; cursor: not-allowed; }

  /* ── AI spawn card ────────────────────────────────────────────── */
  .cb-spawn-card {
    margin: 20px 16px;
    padding: 16px;
    border: 1px solid var(--border);
    border-radius: var(--radius-md, 6px);
    background: var(--muted);
  }
  .cb-spawn-title { font-weight: 600; font-size: 13px; margin-bottom: 12px; }
  .cb-spawn-row {
    display: grid;
    grid-template-columns: 64px 1fr;
    align-items: center;
    gap: 8px;
    margin-bottom: 8px;
  }
  .cb-spawn-row label { font-size: 12px; color: var(--muted-foreground); }
  .cb-spawn-row select {
    height: 30px;
    border: 1px solid var(--border);
    border-radius: var(--radius-sm, 4px);
    background: var(--background);
    color: var(--foreground);
    font: inherit;
    font-size: 13px;
    padding: 0 6px;
  }
  .cb-spawn-error { font-size: 12px; color: var(--destructive, red); margin-bottom: 8px; }
  .cb-spawn-start {
    margin-top: 4px;
    height: 32px;
    width: 100%;
    border: none;
    border-radius: var(--radius-sm, 4px);
    background: var(--primary);
    color: var(--primary-foreground, #fff);
    font: inherit;
    font-size: 13px;
    cursor: pointer;
  }
  .cb-spawn-start:disabled { opacity: 0.5; cursor: not-allowed; }
`,
  setup(ctx) {
    const { props, host } = ctx;
    if (!ctx.cleanups)
      ctx.cleanups = [];
    const __cleanups = ctx.cleanups;
    const $scope = Object.create(null);
    let __r = null;
    if (sui_command_palette_controller_default && typeof sui_command_palette_controller_default.setup === "function") {
      ctx.scope = $scope;
      __r = sui_command_palette_controller_default.setup(ctx);
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
        return (($scope2) => {
          const __frag17 = document.createDocumentFragment();
          const __c18 = [];
          const __e19 = document.createElement("div");
          __e19.setAttribute("class", "cb-session-row");
          __c18.push(bindClass2(__e19, "current", () => $scope2.bar.session()?.id === $scope2.s.id));
          __e19.setAttribute("role", "option");
          __c18.push(on3(__e19, "click", function() {
            const __h = () => $scope2.selectSession($scope2.s.id);
            return typeof __h === "function" ? __h.bind(undefined) : () => {};
          }(), { modifiers: [] }));
          const __e20 = document.createElement("span");
          __e20.setAttribute("class", "cb-dot");
          __c18.push(effect4(() => {
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
          __c18.push(bindText3(__t23, () => $scope2.s.agent_id ?? ""));
          __e21.appendChild(__e22);
          const __e24 = document.createElement("span");
          __e24.setAttribute("class", "meta");
          const __t25 = document.createTextNode(" · ");
          __e24.appendChild(__t25);
          const __t26 = document.createTextNode("");
          __e24.appendChild(__t26);
          __c18.push(bindText3(__t26, () => $scope2.s.provider ?? ""));
          const __t27 = document.createTextNode(" · ");
          __e24.appendChild(__t27);
          const __t28 = document.createTextNode("");
          __e24.appendChild(__t28);
          __c18.push(bindText3(__t28, () => $scope2.s.workspace_id));
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
          const __frag32 = document.createDocumentFragment();
          const __c33 = [];
          const __e34 = document.createElement("div");
          __e34.setAttribute("class", "cb-session-row");
          const __e35 = document.createElement("span");
          __e34.appendChild(__e35);
          const __e36 = document.createElement("span");
          __e36.setAttribute("class", "meta");
          const __t37 = document.createTextNode("No active sessions");
          __e36.appendChild(__t37);
          __e34.appendChild(__e36);
          const __e38 = document.createElement("span");
          __e34.appendChild(__e38);
          __frag32.appendChild(__e34);
          return { nodes: Array.from(__frag32.childNodes), cleanup: () => {
            for (const c of __c33)
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
          const __ifAnchor31 = document.createComment("if");
          __e14.appendChild(__ifAnchor31);
          __c13.push(__wireBoundary_b17(__ifAnchor31));
          const __e39 = document.createElement("div");
          __e39.setAttribute("class", "cb-session-sep");
          __e39.setAttribute("role", "separator");
          __e14.appendChild(__e39);
          const __e40 = document.createElement("div");
          __e40.setAttribute("class", "cb-session-action");
          __c13.push(on3(__e40, "click", function() {
            const __h = $scope.startNewSession;
            return typeof __h === "function" ? __h.bind(undefined) : () => {};
          }(), { modifiers: [] }));
          const __e41 = document.createElement("sui-icon");
          __e41.setAttribute("name", "plus");
          __e41.setAttribute("size", "sm");
          __e41.setAttribute("aria-hidden", "true");
          __e40.appendChild(__e41);
          const __e42 = document.createElement("span");
          const __t43 = document.createTextNode("Start new session");
          __e42.appendChild(__t43);
          __e40.appendChild(__e42);
          __e14.appendChild(__e40);
          const __e44 = document.createElement("div");
          __e44.setAttribute("class", "cb-session-action");
          __c13.push(on3(__e44, "click", function() {
            const __h = $scope.viewChats;
            return typeof __h === "function" ? __h.bind(undefined) : () => {};
          }(), { modifiers: [] }));
          const __e45 = document.createElement("sui-icon");
          __e45.setAttribute("name", "columns");
          __e45.setAttribute("size", "sm");
          __e45.setAttribute("aria-hidden", "true");
          __e44.appendChild(__e45);
          const __e46 = document.createElement("span");
          const __t47 = document.createTextNode("View all chats");
          __e46.appendChild(__t47);
          __e44.appendChild(__e46);
          __e14.appendChild(__e44);
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
        const __childScope59 = Object.create($scope);
        __childScope59.item = item;
        __childScope59.__i = __i;
        return (($scope2) => {
          const __frag60 = document.createDocumentFragment();
          const __c61 = [];
          const __e62 = document.createElement("div");
          __e62.setAttribute("class", "cb-item");
          __e62.setAttribute("role", "option");
          __c61.push(bindAttr2(__e62, "aria-selected", () => $scope2.bar.selectedIdx() === $scope2.item.idx ? "true" : "false"));
          __c61.push(bindAttr2(__e62, "data-path", () => $scope2.item.path));
          __c61.push(on3(__e62, "click", function() {
            const __h = () => $scope2.activateNavItem($scope2.item.path);
            return typeof __h === "function" ? __h.bind(undefined) : () => {};
          }(), { modifiers: [] }));
          const __e63 = document.createElement("sui-icon");
          __c61.push(bindAttr2(__e63, "name", () => $scope2.item.icon));
          __e63.setAttribute("size", "sm");
          __e63.setAttribute("aria-hidden", "true");
          __e62.appendChild(__e63);
          const __e64 = document.createElement("span");
          const __t65 = document.createTextNode("");
          __e64.appendChild(__t65);
          __c61.push(bindText3(__t65, () => $scope2.item.label));
          __e62.appendChild(__e64);
          const __e66 = document.createElement("span");
          __e66.setAttribute("class", "hint");
          const __t67 = document.createTextNode("");
          __e66.appendChild(__t67);
          __c61.push(bindText3(__t67, () => $scope2.item.path));
          __e62.appendChild(__e66);
          __frag60.appendChild(__e62);
          return { nodes: Array.from(__frag60.childNodes), cleanup: () => {
            for (const c of __c61)
              try {
                c();
              } catch (e) {}
          } };
        })(__childScope59);
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
          const __frag53 = document.createDocumentFragment();
          const __c54 = [];
          const __e55 = document.createElement("div");
          __e55.setAttribute("class", "cb-mode-hint");
          const __t56 = document.createTextNode("Jump to");
          __e55.appendChild(__t56);
          __frag53.appendChild(__e55);
          const __e57 = document.createElement("div");
          __e57.setAttribute("class", "cb-list");
          __e57.setAttribute("role", "listbox");
          const __forAnchor58 = document.createComment("for");
          __e57.appendChild(__forAnchor58);
          __c54.push(__wireBoundary_b22(__forAnchor58));
          __frag53.appendChild(__e57);
          return { nodes: Array.from(__frag53.childNodes), cleanup: () => {
            for (const c of __c54)
              try {
                c();
              } catch (e) {}
          } };
        },
        () => {
          const __frag68 = document.createDocumentFragment();
          const __c69 = [];
          const __e70 = document.createElement("div");
          __e70.setAttribute("class", "cb-empty");
          const __t71 = document.createTextNode("No matches.");
          __e70.appendChild(__t71);
          __frag68.appendChild(__e70);
          return { nodes: Array.from(__frag68.childNodes), cleanup: () => {
            for (const c of __c69)
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
        const __childScope81 = Object.create($scope);
        __childScope81.item = item;
        __childScope81.__i = __i;
        return (($scope2) => {
          const __frag82 = document.createDocumentFragment();
          const __c83 = [];
          const __e84 = document.createElement("div");
          __e84.setAttribute("class", "cb-item");
          __e84.setAttribute("role", "option");
          __c83.push(bindAttr2(__e84, "aria-selected", () => $scope2.bar.selectedIdx() === $scope2.item.idx ? "true" : "false"));
          __c83.push(bindAttr2(__e84, "data-cmd", () => $scope2.item.key));
          __c83.push(on3(__e84, "click", function() {
            const __h = () => $scope2.activateCmd($scope2.item.key);
            return typeof __h === "function" ? __h.bind(undefined) : () => {};
          }(), { modifiers: [] }));
          const __e85 = document.createElement("span");
          __e85.setAttribute("style", "font-family:var(--font-mono)");
          const __t86 = document.createTextNode("");
          __e85.appendChild(__t86);
          __c83.push(bindText3(__t86, () => $scope2.item.label));
          __e84.appendChild(__e85);
          const __e87 = document.createElement("span");
          __e84.appendChild(__e87);
          const __e88 = document.createElement("span");
          __e88.setAttribute("class", "hint");
          const __t89 = document.createTextNode("");
          __e88.appendChild(__t89);
          __c83.push(bindText3(__t89, () => $scope2.item.hint));
          __e84.appendChild(__e88);
          __frag82.appendChild(__e84);
          return { nodes: Array.from(__frag82.childNodes), cleanup: () => {
            for (const c of __c83)
              try {
                c();
              } catch (e) {}
          } };
        })(__childScope81);
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
          const __frag75 = document.createDocumentFragment();
          const __c76 = [];
          const __e77 = document.createElement("div");
          __e77.setAttribute("class", "cb-mode-hint");
          const __t78 = document.createTextNode("Actions");
          __e77.appendChild(__t78);
          __frag75.appendChild(__e77);
          const __e79 = document.createElement("div");
          __e79.setAttribute("class", "cb-list");
          __e79.setAttribute("role", "listbox");
          const __forAnchor80 = document.createComment("for");
          __e79.appendChild(__forAnchor80);
          __c76.push(__wireBoundary_b30(__forAnchor80));
          __frag75.appendChild(__e79);
          return { nodes: Array.from(__frag75.childNodes), cleanup: () => {
            for (const c of __c76)
              try {
                c();
              } catch (e) {}
          } };
        },
        () => {
          const __frag90 = document.createDocumentFragment();
          const __c91 = [];
          const __e92 = document.createElement("div");
          __e92.setAttribute("class", "cb-empty");
          const __t93 = document.createTextNode("No commands match.");
          __e92.appendChild(__t93);
          __frag90.appendChild(__e92);
          return { nodes: Array.from(__frag90.childNodes), cleanup: () => {
            for (const c of __c91)
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
        const __childScope103 = Object.create($scope);
        __childScope103.item = item;
        __childScope103.__i = __i;
        return (($scope2) => {
          const __frag104 = document.createDocumentFragment();
          const __c105 = [];
          const __e106 = document.createElement("div");
          __e106.setAttribute("class", "cb-item");
          __e106.setAttribute("role", "option");
          __c105.push(bindAttr2(__e106, "aria-selected", () => $scope2.bar.selectedIdx() === $scope2.item.idx ? "true" : "false"));
          __c105.push(bindAttr2(__e106, "data-mention-kind", () => $scope2.item.kind));
          __c105.push(bindAttr2(__e106, "data-mention-id", () => $scope2.item.id));
          __c105.push(on3(__e106, "click", function() {
            const __h = () => $scope2.activateMention($scope2.item);
            return typeof __h === "function" ? __h.bind(undefined) : () => {};
          }(), { modifiers: [] }));
          const __e107 = document.createElement("sui-icon");
          __c105.push(bindAttr2(__e107, "name", () => $scope2.item.icon));
          __e107.setAttribute("size", "sm");
          __e107.setAttribute("aria-hidden", "true");
          __e106.appendChild(__e107);
          const __e108 = document.createElement("span");
          const __t109 = document.createTextNode("");
          __e108.appendChild(__t109);
          __c105.push(bindText3(__t109, () => $scope2.item.label));
          __e106.appendChild(__e108);
          const __e110 = document.createElement("span");
          __e110.setAttribute("class", "hint");
          const __t111 = document.createTextNode("");
          __e110.appendChild(__t111);
          __c105.push(bindText3(__t111, () => $scope2.item.kind));
          __e106.appendChild(__e110);
          __frag104.appendChild(__e106);
          return { nodes: Array.from(__frag104.childNodes), cleanup: () => {
            for (const c of __c105)
              try {
                c();
              } catch (e) {}
          } };
        })(__childScope103);
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
          const __frag97 = document.createDocumentFragment();
          const __c98 = [];
          const __e99 = document.createElement("div");
          __e99.setAttribute("class", "cb-mode-hint");
          const __t100 = document.createTextNode("Reference");
          __e99.appendChild(__t100);
          __frag97.appendChild(__e99);
          const __e101 = document.createElement("div");
          __e101.setAttribute("class", "cb-list");
          __e101.setAttribute("role", "listbox");
          const __forAnchor102 = document.createComment("for");
          __e101.appendChild(__forAnchor102);
          __c98.push(__wireBoundary_b37(__forAnchor102));
          __frag97.appendChild(__e101);
          return { nodes: Array.from(__frag97.childNodes), cleanup: () => {
            for (const c of __c98)
              try {
                c();
              } catch (e) {}
          } };
        },
        () => {
          const __frag112 = document.createDocumentFragment();
          const __c113 = [];
          const __e114 = document.createElement("div");
          __e114.setAttribute("class", "cb-empty");
          const __t115 = document.createTextNode("No matching views or agents.");
          __e114.appendChild(__t115);
          __frag112.appendChild(__e114);
          return { nodes: Array.from(__frag112.childNodes), cleanup: () => {
            for (const c of __c113)
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
        const __childScope122 = Object.create($scope);
        __childScope122.surf = surf;
        __childScope122.__i = __i;
        return (($scope2) => {
          const __frag123 = document.createDocumentFragment();
          const __c124 = [];
          const __e125 = document.createElement("button");
          __e125.setAttribute("class", "cb-surface-btn");
          __e125.setAttribute("type", "button");
          __c124.push(bindAttr2(__e125, "aria-pressed", () => $scope2.bar.createSurface() === $scope2.surf.id ? "true" : "false"));
          __c124.push(on3(__e125, "click", function() {
            const __h = () => $scope2.selectSurface($scope2.surf.id);
            return typeof __h === "function" ? __h.bind(undefined) : () => {};
          }(), { modifiers: [] }));
          const __e126 = document.createElement("sui-icon");
          __c124.push(bindAttr2(__e126, "name", () => $scope2.surf.icon));
          __e126.setAttribute("aria-hidden", "true");
          __e125.appendChild(__e126);
          const __e127 = document.createElement("span");
          const __t128 = document.createTextNode("");
          __e127.appendChild(__t128);
          __c124.push(bindText3(__t128, () => $scope2.surf.label));
          __e125.appendChild(__e127);
          __frag123.appendChild(__e125);
          return { nodes: Array.from(__frag123.childNodes), cleanup: () => {
            for (const c of __c124)
              try {
                c();
              } catch (e) {}
          } };
        })(__childScope122);
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
        const __childScope139 = Object.create($scope);
        __childScope139.sh = sh;
        __childScope139.__i = __i;
        return (($scope2) => {
          const __frag140 = document.createDocumentFragment();
          const __c141 = [];
          const __e142 = document.createElement("option");
          __c141.push(bindAttr2(__e142, "value", () => $scope2.sh));
          const __t143 = document.createTextNode("");
          __e142.appendChild(__t143);
          __c141.push(bindText3(__t143, () => $scope2.sh));
          __frag140.appendChild(__e142);
          return { nodes: Array.from(__frag140.childNodes), cleanup: () => {
            for (const c of __c141)
              try {
                c();
              } catch (e) {}
          } };
        })(__childScope139);
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
          const __frag149 = document.createDocumentFragment();
          const __c150 = [];
          const __e151 = document.createElement("div");
          __e151.setAttribute("class", "cb-create-error");
          const __t152 = document.createTextNode("");
          __e151.appendChild(__t152);
          __c150.push(bindText3(__t152, () => $scope.bar.createError()));
          __frag149.appendChild(__e151);
          return { nodes: Array.from(__frag149.childNodes), cleanup: () => {
            for (const c of __c150)
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
        const __childScope173 = Object.create($scope);
        __childScope173.a = a;
        __childScope173.__i = __i;
        return (($scope2) => {
          const __frag174 = document.createDocumentFragment();
          const __c175 = [];
          const __e176 = document.createElement("option");
          __c175.push(bindAttr2(__e176, "value", () => $scope2.a.id));
          const __t177 = document.createTextNode("");
          __e176.appendChild(__t177);
          __c175.push(bindText3(__t177, () => $scope2.a.name ?? $scope2.a.label ?? $scope2.a.id));
          __frag174.appendChild(__e176);
          return { nodes: Array.from(__frag174.childNodes), cleanup: () => {
            for (const c of __c175)
              try {
                c();
              } catch (e) {}
          } };
        })(__childScope173);
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
          const __frag166 = document.createDocumentFragment();
          const __c167 = [];
          const __e168 = document.createElement("option");
          __e168.setAttribute("value", "");
          const __t169 = document.createTextNode("No agents found");
          __e168.appendChild(__t169);
          __frag166.appendChild(__e168);
          return { nodes: Array.from(__frag166.childNodes), cleanup: () => {
            for (const c of __c167)
              try {
                c();
              } catch (e) {}
          } };
        },
        () => {
          const __frag170 = document.createDocumentFragment();
          const __c171 = [];
          const __forAnchor172 = document.createComment("for");
          __frag170.appendChild(__forAnchor172);
          __c171.push(__wireBoundary_b65(__forAnchor172));
          return { nodes: Array.from(__frag170.childNodes), cleanup: () => {
            for (const c of __c171)
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
        const __childScope190 = Object.create($scope);
        __childScope190.p = p;
        __childScope190.__i = __i;
        return (($scope2) => {
          const __frag191 = document.createDocumentFragment();
          const __c192 = [];
          const __e193 = document.createElement("option");
          __c192.push(bindAttr2(__e193, "value", () => $scope2.p));
          const __t194 = document.createTextNode("");
          __e193.appendChild(__t194);
          __c192.push(bindText3(__t194, () => $scope2.p));
          __frag191.appendChild(__e193);
          return { nodes: Array.from(__frag191.childNodes), cleanup: () => {
            for (const c of __c192)
              try {
                c();
              } catch (e) {}
          } };
        })(__childScope190);
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
          const __frag183 = document.createDocumentFragment();
          const __c184 = [];
          const __e185 = document.createElement("option");
          __e185.setAttribute("value", "");
          const __t186 = document.createTextNode("No providers installed");
          __e185.appendChild(__t186);
          __frag183.appendChild(__e185);
          return { nodes: Array.from(__frag183.childNodes), cleanup: () => {
            for (const c of __c184)
              try {
                c();
              } catch (e) {}
          } };
        },
        () => {
          const __frag187 = document.createDocumentFragment();
          const __c188 = [];
          const __forAnchor189 = document.createComment("for");
          __frag187.appendChild(__forAnchor189);
          __c188.push(__wireBoundary_b70(__forAnchor189));
          return { nodes: Array.from(__frag187.childNodes), cleanup: () => {
            for (const c of __c188)
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
          const __frag196 = document.createDocumentFragment();
          const __c197 = [];
          const __e198 = document.createElement("div");
          __e198.setAttribute("class", "cb-spawn-error");
          const __t199 = document.createTextNode("");
          __e198.appendChild(__t199);
          __c197.push(bindText3(__t199, () => $scope.bar.spawnError()));
          __frag196.appendChild(__e198);
          return { nodes: Array.from(__frag196.childNodes), cleanup: () => {
            for (const c of __c197)
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
          const __frag50 = document.createDocumentFragment();
          const __c51 = [];
          const __ifAnchor52 = document.createComment("if");
          __frag50.appendChild(__ifAnchor52);
          __c51.push(__wireBoundary_b21(__ifAnchor52));
          return { nodes: Array.from(__frag50.childNodes), cleanup: () => {
            for (const c of __c51)
              try {
                c();
              } catch (e) {}
          } };
        },
        () => {
          const __frag72 = document.createDocumentFragment();
          const __c73 = [];
          const __ifAnchor74 = document.createComment("if");
          __frag72.appendChild(__ifAnchor74);
          __c73.push(__wireBoundary_b29(__ifAnchor74));
          return { nodes: Array.from(__frag72.childNodes), cleanup: () => {
            for (const c of __c73)
              try {
                c();
              } catch (e) {}
          } };
        },
        () => {
          const __frag94 = document.createDocumentFragment();
          const __c95 = [];
          const __ifAnchor96 = document.createComment("if");
          __frag94.appendChild(__ifAnchor96);
          __c95.push(__wireBoundary_b36(__ifAnchor96));
          return { nodes: Array.from(__frag94.childNodes), cleanup: () => {
            for (const c of __c95)
              try {
                c();
              } catch (e) {}
          } };
        },
        () => {
          const __frag116 = document.createDocumentFragment();
          const __c117 = [];
          const __e118 = document.createElement("div");
          __e118.setAttribute("class", "cb-mode-hint");
          const __t119 = document.createTextNode("Create");
          __e118.appendChild(__t119);
          __frag116.appendChild(__e118);
          const __e120 = document.createElement("div");
          __e120.setAttribute("class", "cb-create-surfaces");
          __e120.setAttribute("role", "group");
          __e120.setAttribute("aria-label", "Surface type");
          const __forAnchor121 = document.createComment("for");
          __e120.appendChild(__forAnchor121);
          __c117.push(__wireBoundary_b45(__forAnchor121));
          __frag116.appendChild(__e120);
          const __e129 = document.createElement("div");
          __e129.setAttribute("class", "cb-create-grid");
          const __e130 = document.createElement("label");
          const __e131 = document.createElement("span");
          const __t132 = document.createTextNode("Name (kebab-case)");
          __e131.appendChild(__t132);
          __e130.appendChild(__e131);
          const __e133 = document.createElement("input");
          __e133.setAttribute("type", "text");
          __e133.setAttribute("class", "cb-create-name");
          __c117.push(bindAttr2(__e133, "placeholder", () => "my-" + $scope.bar.createSurface()));
          __c117.push(bindValue2(__e133, $scope.bar.createName));
          __e133.setAttribute("autocomplete", "off");
          __e130.appendChild(__e133);
          __e129.appendChild(__e130);
          const __e134 = document.createElement("label");
          const __e135 = document.createElement("span");
          const __t136 = document.createTextNode("");
          __e135.appendChild(__t136);
          __c117.push(bindText3(__t136, () => $scope.currentSurfaceDef().shapeLabel));
          __e134.appendChild(__e135);
          const __e137 = document.createElement("select");
          __e137.setAttribute("class", "cb-create-shape");
          __c117.push(bindValue2(__e137, $scope.bar.createShape));
          const __forAnchor138 = document.createComment("for");
          __e137.appendChild(__forAnchor138);
          __c117.push(__wireBoundary_b54(__forAnchor138));
          __e134.appendChild(__e137);
          __e129.appendChild(__e134);
          const __e144 = document.createElement("label");
          __e144.setAttribute("class", "span-2");
          const __e145 = document.createElement("span");
          const __t146 = document.createTextNode("Purpose (optional)");
          __e145.appendChild(__t146);
          __e144.appendChild(__e145);
          const __e147 = document.createElement("input");
          __e147.setAttribute("type", "text");
          __e147.setAttribute("class", "cb-create-purpose");
          __e147.setAttribute("placeholder", "One-line description");
          __c117.push(bindValue2(__e147, $scope.bar.createPurpose));
          __e147.setAttribute("autocomplete", "off");
          __e144.appendChild(__e147);
          __e129.appendChild(__e144);
          __frag116.appendChild(__e129);
          const __ifAnchor148 = document.createComment("if");
          __frag116.appendChild(__ifAnchor148);
          __c117.push(__wireBoundary_b58(__ifAnchor148));
          const __e153 = document.createElement("div");
          __e153.setAttribute("class", "cb-create-actions");
          const __e154 = document.createElement("button");
          __e154.setAttribute("class", "cb-create-submit");
          __e154.setAttribute("type", "button");
          __c117.push(bindAttr2(__e154, "disabled", () => $scope.bar.createSubmitting() || !$scope.createNameValid()));
          __c117.push(on3(__e154, "click", function() {
            const __h = $scope.submitCreate;
            return typeof __h === "function" ? __h.bind(undefined) : () => {};
          }(), { modifiers: [] }));
          const __t155 = document.createTextNode("");
          __e154.appendChild(__t155);
          __c117.push(bindText3(__t155, () => $scope.bar.createSubmitting() ? "Creatingâ¦" : "Create " + $scope.currentSurfaceDef().label));
          __e153.appendChild(__e154);
          __frag116.appendChild(__e153);
          return { nodes: Array.from(__frag116.childNodes), cleanup: () => {
            for (const c of __c117)
              try {
                c();
              } catch (e) {}
          } };
        },
        () => {
          const __frag156 = document.createDocumentFragment();
          const __c157 = [];
          const __e158 = document.createElement("div");
          __e158.setAttribute("class", "cb-spawn-card");
          const __e159 = document.createElement("div");
          __e159.setAttribute("class", "cb-spawn-title");
          const __t160 = document.createTextNode("Start a session");
          __e159.appendChild(__t160);
          __e158.appendChild(__e159);
          const __e161 = document.createElement("div");
          __e161.setAttribute("class", "cb-spawn-row");
          const __e162 = document.createElement("label");
          const __t163 = document.createTextNode("Agent");
          __e162.appendChild(__t163);
          __e161.appendChild(__e162);
          const __e164 = document.createElement("select");
          __e164.setAttribute("class", "cb-spawn-agent");
          __c157.push(bindValue2(__e164, $scope.bar.spawnAgent));
          const __ifAnchor165 = document.createComment("if");
          __e164.appendChild(__ifAnchor165);
          __c157.push(__wireBoundary_b64(__ifAnchor165));
          __e161.appendChild(__e164);
          __e158.appendChild(__e161);
          const __e178 = document.createElement("div");
          __e178.setAttribute("class", "cb-spawn-row");
          const __e179 = document.createElement("label");
          const __t180 = document.createTextNode("Provider");
          __e179.appendChild(__t180);
          __e178.appendChild(__e179);
          const __e181 = document.createElement("select");
          __e181.setAttribute("class", "cb-spawn-provider");
          __c157.push(bindValue2(__e181, $scope.bar.spawnProvider));
          const __ifAnchor182 = document.createComment("if");
          __e181.appendChild(__ifAnchor182);
          __c157.push(__wireBoundary_b69(__ifAnchor182));
          __e178.appendChild(__e181);
          __e158.appendChild(__e178);
          const __ifAnchor195 = document.createComment("if");
          __e158.appendChild(__ifAnchor195);
          __c157.push(__wireBoundary_b73(__ifAnchor195));
          const __e200 = document.createElement("button");
          __e200.setAttribute("class", "cb-spawn-start");
          __e200.setAttribute("type", "button");
          __c157.push(bindAttr2(__e200, "disabled", () => $scope.bar.spawning() || !$scope.bar.spawnAgent() || !$scope.bar.spawnProvider()));
          __c157.push(on3(__e200, "click", function() {
            const __h = $scope.spawnSession;
            return typeof __h === "function" ? __h.bind(undefined) : () => {};
          }(), { modifiers: [] }));
          const __t201 = document.createTextNode("");
          __e200.appendChild(__t201);
          __c157.push(bindText3(__t201, () => $scope.bar.spawning() ? "Startingâ¦" : "Start session"));
          __e158.appendChild(__e200);
          __frag156.appendChild(__e158);
          return { nodes: Array.from(__frag156.childNodes), cleanup: () => {
            for (const c of __c157)
              try {
                c();
              } catch (e) {}
          } };
        }
      ]
    });
    const __root = host.shadowRoot ?? host;
    adoptSuiSheets3(__root);
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
    __root.appendChild(__e0);
    const __e48 = document.createElement("div");
    __e48.setAttribute("class", "cb-body");
    const __matchAnchor49 = document.createComment("match");
    __e48.appendChild(__matchAnchor49);
    __cleanups.push(__wireBoundary_b20(__matchAnchor49));
    __root.appendChild(__e48);
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
