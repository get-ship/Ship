// packages/ui-components/src/islands/sui-command-bar/sui-command-bar.sui
import { adoptSuiSheets, attachRef, bindAttr, bindIf, bindText, component, effect as effect2, on, prop, signal as signal2 } from "@ship/sui-runtime";

// packages/ui-components/src/islands/sui-command-bar/sui-command-bar.controller.ts
import { controller, signal, derived, effect, ref, shared } from "@ship/sui-runtime";

// packages/ui-components/src/islands/sui-command-bar/types.ts
import { chrome } from "@ship/sui-runtime";
var SEARCH_KIND_LABELS = {
  view: "Views",
  collection: "Collections",
  collection_item: "Items",
  agent: "Agents",
  skill: "Skills",
  session: "Sessions",
  artifact: "Artifacts"
};
async function searchEntities(q, opts = {}) {
  const params = new URLSearchParams;
  if (q)
    params.set("q", q);
  if (opts.limit != null)
    params.set("limit", String(opts.limit));
  if (opts.types?.length)
    params.set("types", opts.types.join(","));
  if (opts.cursor)
    params.set("cursor", opts.cursor);
  try {
    const resp = await fetch(`/api/v1/search?${params.toString()}`, {
      credentials: "same-origin",
      signal: opts.signal
    });
    if (!resp.ok)
      return null;
    return await resp.json();
  } catch {
    return null;
  }
}
var SLASH_CACHE_MAX_AGE_MS = 60000;
var slashCache = null;
var slashInFlight = null;
async function loadSlashCommands(force = false) {
  const now = performance.now();
  if (!force && slashCache && now - slashCache.at < SLASH_CACHE_MAX_AGE_MS) {
    return slashCache.commands;
  }
  if (slashInFlight)
    return slashInFlight;
  slashInFlight = (async () => {
    try {
      const resp = await fetch("/api/v1/commands", { credentials: "same-origin" });
      if (!resp.ok)
        throw new Error(`HTTP ${resp.status}`);
      const body = await resp.json();
      const commands = body.commands ?? [];
      slashCache = { at: performance.now(), commands };
      return commands;
    } catch {
      if (slashCache)
        return slashCache.commands;
      slashCache = { at: performance.now(), commands: SLASH_COMMANDS };
      return SLASH_COMMANDS;
    } finally {
      slashInFlight = null;
    }
  })();
  return slashInFlight;
}
function cachedSlashCommands() {
  return slashCache?.commands ?? SLASH_COMMANDS;
}
function slashCacheAge() {
  return slashCache ? performance.now() - slashCache.at : Number.POSITIVE_INFINITY;
}
var SLASH_CACHE_TTL_MS = SLASH_CACHE_MAX_AGE_MS;
var CREATE_SURFACES = [
  { id: "view", label: "View", icon: "file", shapes: ["list", "form", "detail", "dashboard", "tabs", "blank"], shapeLabel: "Starting shape" },
  { id: "skill", label: "Skill", icon: "sparkles", shapes: ["task", "reference", "workflow"], shapeLabel: "Skill kind" },
  { id: "agent", label: "Agent", icon: "users", shapes: ["specialist", "generalist", "reviewer"], shapeLabel: "Agent kind" },
  { id: "pipeline", label: "Pipeline", icon: "play", shapes: ["sequential", "fanout", "review-gated"], shapeLabel: "Topology" },
  { id: "trigger", label: "Trigger", icon: "zap", shapes: ["webhook", "schedule", "file-change"], shapeLabel: "Kind" },
  { id: "theme", label: "Theme", icon: "palette", shapes: ["light", "dark", "brand-accent", "high-contrast"], shapeLabel: "Starting shape" }
];
var SLASH_COMMANDS = [
  { key: "new", label: "/new", hint: "Scaffold a view, skill, agent, …" },
  { key: "chats", label: "/chats", hint: "Open Mission Control" },
  { key: "settings", label: "/settings", hint: "Jump to settings" },
  { key: "theme", label: "/theme", hint: "Open Theme Studio", handler_kind: "navigate" },
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
  chrome.navigate(path);
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
    const slashCommands = shared(signal(cachedSlashCommands()));
    const sendKey = shared(signal("enter"));
    const searchHits = shared(signal([]));
    const searching = shared(signal(false));
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
      dialogRef,
      slashCommands,
      sendKey,
      searchHits,
      searching
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
    const slashCommands = st.slashCommands;
    const sendKey = st.sendKey;
    const searchHits = st.searchHits;
    const searching = st.searching;
    const dialogRef = st.dialogRef?.();
    let streamAbort = null;
    let lastSpaceAt = 0;
    async function refreshSlashCommands(force = false) {
      const list = await loadSlashCommands(force);
      slashCommands.set(list);
    }
    async function refreshSendKey() {
      try {
        const resp = await fetch("/api/v1/settings/cmdk.send_key", { credentials: "same-origin" });
        if (!resp.ok)
          return;
        const body = await resp.json();
        const v = typeof body.value === "string" ? body.value : body.data?.value;
        if (v === "enter" || v === "cmd+enter")
          sendKey.set(v);
      } catch {}
    }
    const stopOpenSync = effect(() => {
      const isOpen = open();
      host.setAttribute("data-state", isOpen ? "open" : "closed");
      if (isOpen) {
        if (typeof dialogRef?.show === "function")
          dialogRef.show();
        if (slashCacheAge() > SLASH_CACHE_TTL_MS)
          refreshSlashCommands();
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
    let searchTimer = null;
    let searchAbort = null;
    function cancelSearch() {
      if (searchTimer) {
        clearTimeout(searchTimer);
        searchTimer = null;
      }
      if (searchAbort) {
        searchAbort.abort();
        searchAbort = null;
      }
    }
    async function runSearch(query) {
      cancelSearch();
      const ac = new AbortController;
      searchAbort = ac;
      searching.set(true);
      const resp = await searchEntities(query, { limit: 30, signal: ac.signal });
      if (ac.signal.aborted)
        return;
      searching.set(false);
      searchAbort = null;
      searchHits.set(resp?.data?.hits ?? []);
    }
    const stopSearchSync = effect(() => {
      const isOpen = open();
      const currentMode = mode();
      const q = value().replace(/^>+\s*/, "").trim();
      if (!isOpen || currentMode !== "find") {
        cancelSearch();
        return;
      }
      if (searchTimer)
        clearTimeout(searchTimer);
      searchTimer = setTimeout(() => {
        runSearch(q);
      }, 200);
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
        navigateFirstHit();
    }
    function runSlashCommand(v) {
      const needle = v.replace(/^\/+\s*/, "").toLowerCase();
      const cmds = slashCommands();
      const cmd = cmds.find((c) => c.key === needle) ?? cmds.find((c) => c.key.startsWith(needle));
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
    function navigateFirstHit() {
      const top = searchHits()[0];
      if (top) {
        navigateTo(top.path);
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
    refreshSlashCommands();
    refreshSendKey();
    const onConfigChanged = (e) => {
      const detail = e.detail;
      if (!detail || detail.subject === "cmdk.send_key")
        refreshSendKey();
    };
    document.addEventListener("ship:config:changed", onConfigChanged);
    return () => {
      document.removeEventListener("ship:config:changed", onConfigChanged);
      document.removeEventListener("keydown", onDocKeydown);
      host.removeEventListener("keydown", onPaletteKeydown);
      window.removeEventListener("ship:session:changed", onSessionChanged);
      shell?.removeEventListener("ship:bar:open", onBarOpen);
      shell?.removeEventListener("ship:create:open", onCreateOpen);
      if (dialogRef)
        dialogRef.removeEventListener("sui-close", onDialogClose);
      cancelSearch();
      stopStreamSync();
      stopSearchSync();
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
  /* Host participates in layout so the trigger pill carries its own
     box (probes asserting commandBar.offsetWidth need a non-zero
     measurement). Inline-flex keeps the pill sized to its content
     while the explicit min-width matches the trigger's design width. */
  :host {
    display: inline-flex;
    align-items: center;
    min-width: 200px;
    max-width: 320px;
  }

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
    width: 100%;
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
    const __wireBoundary_b5 = (anchor) => bindIf({
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
    __e0.setAttribute("aria-haspopup", "dialog");
    __e0.setAttribute("aria-controls", "cb-dialog-root");
    __cleanups.push(bindAttr(__e0, "aria-expanded", () => $scope.open() ? "true" : "false"));
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
    __e7.setAttribute("id", "cb-dialog-root");
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
    __cleanups.push(__wireBoundary_b5(__ifAnchor9));
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
import { adoptSuiSheets as adoptSuiSheets2, bindAttr as bindAttr2, bindClass, bindFor, bindIf as bindIf2, bindText as bindText2, bindValue, component as component2, effect as effect3, on as on2, signal as signal3 } from "@ship/sui-runtime";

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
      const sendKey = s.bar.sendKey?.() ?? "enter";
      const isSubmit = sendKey === "cmd+enter" ? e.key === "Enter" && (e.metaKey || e.ctrlKey) : e.key === "Enter" && !e.shiftKey && !e.metaKey && !e.ctrlKey;
      if (!isSubmit)
        return;
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
});

// packages/ui-components/src/islands/sui-command-bar/sui-command-chat.sui
/*! _compilerVersion {"cargo":"0.1.0","git":"d4751a31","dirty":true,"builtAt":"2026-05-14T02:37:23Z"} */
var sui_command_chat_default = component2("sui-command-chat", {
  _compilerVersion: { cargo: "0.1.0", git: "d4751a31", dirty: true, builtAt: "2026-05-14T02:37:23Z" },
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
    __root.appendChild(__e0);
    const __e54 = document.createElement("div");
    __e54.setAttribute("class", "cb-chat-messages");
    const __ifAnchor55 = document.createComment("if");
    __e54.appendChild(__ifAnchor55);
    __cleanups.push(__wireBoundary_b17(__ifAnchor55));
    __root.appendChild(__e54);
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
    __cleanups.push(bindAttr2(__e74, "disabled", () => !$scope.bar.value().trim()));
    __cleanups.push(on2(__e74, "click", function() {
      const __h = $scope.sendMessage;
      return typeof __h === "function" ? __h.bind(undefined) : () => {};
    }(), { modifiers: [] }));
    const __t75 = document.createTextNode("Send");
    __e74.appendChild(__t75);
    __e72.appendChild(__e74);
    __root.appendChild(__e72);
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
import { adoptSuiSheets as adoptSuiSheets3, bindAttr as bindAttr3, bindClass as bindClass2, bindFor as bindFor2, bindIf as bindIf3, bindText as bindText3, bindValue as bindValue2, component as component3, effect as effect4, on as on3, signal as signal4 } from "@ship/sui-runtime";

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
    const KIND_ORDER = [
      "view",
      "collection",
      "agent",
      "skill",
      "session",
      "collection_item",
      "artifact"
    ];
    const searchGroups = derived3(() => {
      const groups = {
        view: [],
        collection: [],
        collection_item: [],
        agent: [],
        skill: [],
        session: [],
        artifact: []
      };
      for (const hit of bar.searchHits()) {
        groups[hit.kind].push(hit);
      }
      let nextIdx = 0;
      return KIND_ORDER.filter((k) => groups[k].length > 0).map((k) => ({
        kind: k,
        label: SEARCH_KIND_LABELS[k],
        items: groups[k].map((hit) => ({ ...hit, idx: nextIdx++ }))
      }));
    });
    const findHasResults = derived3(() => bar.searchHits().length > 0);
    const findSearching = derived3(() => bar.searching());
    const cmdItems = derived3(() => {
      const needle = bar.value().replace(/^\/+\s*/, "").toLowerCase().trim();
      return bar.slashCommands().filter((c) => !needle || c.key.includes(needle) || c.label.toLowerCase().includes(needle) || c.hint.toLowerCase().includes(needle)).map((c, i) => ({ ...c, idx: i }));
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
    const LAUNCH_TIER = new Set([
      "view",
      "skill",
      "agent",
      "theme"
    ]);
    const createSurfaces = derived3(() => CREATE_SURFACES.map((s) => ({ ...s, disabled: !LAUNCH_TIER.has(s.id) })));
    const activeDescendantId = derived3(() => {
      const idx = bar.selectedIdx();
      if (idx < 0)
        return "";
      return `cb-item-${idx}`;
    });
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
      hostEl?.dispatchEvent(new CustomEvent("sui-command:select", {
        detail: { mode: "find", item: { path } },
        bubbles: true,
        composed: true
      }));
      navigateTo(path);
      bar.value.set("");
      bar.open.set(false);
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
        case "theme":
          navigateTo("/theme-studio");
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
      searchGroups,
      findHasResults,
      findSearching,
      cmdItems,
      mentionItems,
      activeDescendantId,
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
      const def = CREATE_SURFACES.find((x) => x.id === id);
      const launch = new Set([
        "view",
        "skill",
        "agent",
        "theme"
      ]);
      if (!launch.has(id))
        return;
      s.bar.createSurface.set(id);
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
        const surface = s.bar.createSurface();
        const resp = await fetch(`/api/v1/create/${encodeURIComponent(surface)}`, {
          method: "POST",
          credentials: "same-origin",
          headers: { "Content-Type": "application/json" },
          body: JSON.stringify({
            name: s.bar.createName(),
            shape: s.bar.createShape(),
            purpose: s.bar.createPurpose() || undefined
          })
        });
        if (!resp.ok) {
          const err = await resp.json().catch(() => ({}));
          s.bar.createError.set(err.error ?? `HTTP ${resp.status}`);
        } else {
          const body = await resp.json().catch(() => ({}));
          s.bar.createName.set("");
          s.bar.createPurpose.set("");
          s.bar.open.set(false);
          const route = body.data?.route;
          if (route)
            navigateTo(route);
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
var sui_command_palette_default = component3("sui-command-palette", {
  _compilerVersion: { cargo: "0.1.0", git: "d4751a31", dirty: true, builtAt: "2026-05-14T02:37:23Z" },
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
  .cb-surface-btn:hover:not([aria-pressed='true']):not([disabled]) { background: var(--muted); }
  .cb-surface-btn[disabled] {
    opacity: 0.55;
    cursor: not-allowed;
  }
  .cb-surface-hint {
    font-size: 10px;
    font-family: var(--font-mono);
    color: var(--muted-foreground);
    padding-left: 4px;
    border-left: 1px solid var(--border);
    margin-left: 2px;
  }
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
    const __wireBoundary_b10 = (anchor) => bindFor2({
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
    const __wireBoundary_b18 = (anchor) => bindIf3({
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
    const __wireBoundary_b9 = (anchor) => bindIf3({
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
          __c13.push(__wireBoundary_b10(__forAnchor15));
          const __ifAnchor33 = document.createComment("if");
          __e14.appendChild(__ifAnchor33);
          __c13.push(__wireBoundary_b18(__ifAnchor33));
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
      items: () => $scope.searchGroups(),
      key: (group, __i) => {
        const __child = Object.create($scope);
        __child.group = group;
        __child.__i = __i;
        return (($scope2) => group.kind)(__child);
      },
      render: (group, __i) => {
        const __childScope55 = Object.create($scope);
        __childScope55.group = group;
        __childScope55.__i = __i;
        __childScope55.__v = signal4(0);
        return (($scope2) => {
          const __wireBoundary_b25 = (anchor2) => bindFor2({
            anchor: anchor2,
            items: () => $scope2.group.items,
            key: (item, __i2) => {
              const __child = Object.create($scope2);
              __child.item = item;
              __child.__i = __i2;
              return (($scope3) => item.kind + ":" + item.id)(__child);
            },
            render: (item, __i2) => {
              const __childScope63 = Object.create($scope2);
              __childScope63.item = item;
              __childScope63.__i = __i2;
              __childScope63.__v = signal4(0);
              return (($scope3) => {
                const __frag64 = document.createDocumentFragment();
                const __c65 = [];
                const __e66 = document.createElement("div");
                __e66.setAttribute("class", "cb-item");
                __e66.setAttribute("role", "option");
                {
                  const __av67 = "cb-item-" + $scope3.item.idx;
                  if (__av67 == null || __av67 === false || __av67 === "")
                    __e66.removeAttribute("id");
                  else
                    __e66.setAttribute("id", __av67 === true ? "" : String(__av67));
                }
                __c65.push(bindAttr3(__e66, "aria-selected", () => (void $scope3.__v(), $scope3.bar.selectedIdx() === $scope3.item.idx ? "true" : "false")));
                {
                  const __av68 = $scope3.item.path;
                  if (__av68 == null || __av68 === false || __av68 === "")
                    __e66.removeAttribute("data-path");
                  else
                    __e66.setAttribute("data-path", __av68 === true ? "" : String(__av68));
                }
                {
                  const __av69 = $scope3.item.kind;
                  if (__av69 == null || __av69 === false || __av69 === "")
                    __e66.removeAttribute("data-kind");
                  else
                    __e66.setAttribute("data-kind", __av69 === true ? "" : String(__av69));
                }
                {
                  const __av70 = $scope3.item.id;
                  if (__av70 == null || __av70 === false || __av70 === "")
                    __e66.removeAttribute("data-id");
                  else
                    __e66.setAttribute("data-id", __av70 === true ? "" : String(__av70));
                }
                __c65.push(on3(__e66, "click", function() {
                  const __h = () => $scope3.activateNavItem($scope3.item.path);
                  return typeof __h === "function" ? __h.bind(undefined) : () => {};
                }(), { modifiers: [] }));
                const __e71 = document.createElement("sui-icon");
                __c65.push(bindAttr3(__e71, "name", () => (void $scope3.__v(), $scope3.item.icon ?? "app-window")));
                __e71.setAttribute("size", "sm");
                __e71.setAttribute("aria-hidden", "true");
                __e66.appendChild(__e71);
                const __e72 = document.createElement("span");
                const __t73 = document.createTextNode("");
                __e72.appendChild(__t73);
                __t73.textContent = $scope3.item.label;
                __e66.appendChild(__e72);
                const __e74 = document.createElement("span");
                __e74.setAttribute("class", "hint");
                const __t75 = document.createTextNode("");
                __e74.appendChild(__t75);
                __t75.textContent = $scope3.group.label;
                __e66.appendChild(__e74);
                __frag64.appendChild(__e66);
                return { nodes: Array.from(__frag64.childNodes), cleanup: () => {
                  for (const c of __c65)
                    try {
                      c();
                    } catch (e) {}
                }, update: (__new76, __newI77) => {
                  $scope3.item = __new76;
                  $scope3.__i = __newI77;
                  {
                    const __av67 = "cb-item-" + $scope3.item.idx;
                    if (__av67 == null || __av67 === false || __av67 === "")
                      __e66.removeAttribute("id");
                    else
                      __e66.setAttribute("id", __av67 === true ? "" : String(__av67));
                  }
                  {
                    const __av68 = $scope3.item.path;
                    if (__av68 == null || __av68 === false || __av68 === "")
                      __e66.removeAttribute("data-path");
                    else
                      __e66.setAttribute("data-path", __av68 === true ? "" : String(__av68));
                  }
                  {
                    const __av69 = $scope3.item.kind;
                    if (__av69 == null || __av69 === false || __av69 === "")
                      __e66.removeAttribute("data-kind");
                    else
                      __e66.setAttribute("data-kind", __av69 === true ? "" : String(__av69));
                  }
                  {
                    const __av70 = $scope3.item.id;
                    if (__av70 == null || __av70 === false || __av70 === "")
                      __e66.removeAttribute("data-id");
                    else
                      __e66.setAttribute("data-id", __av70 === true ? "" : String(__av70));
                  }
                  __t73.textContent = $scope3.item.label;
                  __t75.textContent = $scope3.group.label;
                  $scope3.__v.set((c) => c + 1);
                } };
              })(__childScope63);
            }
          });
          const __frag56 = document.createDocumentFragment();
          const __c57 = [];
          const __e58 = document.createElement("div");
          __e58.setAttribute("class", "cb-mode-hint");
          const __t59 = document.createTextNode("");
          __e58.appendChild(__t59);
          __t59.textContent = $scope2.group.label;
          __frag56.appendChild(__e58);
          const __e60 = document.createElement("div");
          __e60.setAttribute("class", "cb-list");
          __e60.setAttribute("role", "listbox");
          {
            const __av61 = $scope2.group.label;
            if (__av61 == null || __av61 === false || __av61 === "")
              __e60.removeAttribute("aria-label");
            else
              __e60.setAttribute("aria-label", __av61 === true ? "" : String(__av61));
          }
          const __forAnchor62 = document.createComment("for");
          __e60.appendChild(__forAnchor62);
          __c57.push(__wireBoundary_b25(__forAnchor62));
          __frag56.appendChild(__e60);
          return { nodes: Array.from(__frag56.childNodes), cleanup: () => {
            for (const c of __c57)
              try {
                c();
              } catch (e) {}
          }, update: (__new78, __newI79) => {
            $scope2.group = __new78;
            $scope2.__i = __newI79;
            __t59.textContent = $scope2.group.label;
            {
              const __av61 = $scope2.group.label;
              if (__av61 == null || __av61 === false || __av61 === "")
                __e60.removeAttribute("aria-label");
              else
                __e60.setAttribute("aria-label", __av61 === true ? "" : String(__av61));
            }
            $scope2.__v.set((c) => c + 1);
          } };
        })(__childScope55);
      }
    });
    const __wireBoundary_b35 = (anchor) => bindIf3({
      anchor,
      test: () => {
        if (!$scope.findHasResults() && $scope.findSearching())
          return 0;
        return -1;
      },
      branches: [
        () => {
          const __frag81 = document.createDocumentFragment();
          const __c82 = [];
          const __e83 = document.createElement("div");
          __e83.setAttribute("class", "cb-empty");
          const __t84 = document.createTextNode("Searching…");
          __e83.appendChild(__t84);
          __frag81.appendChild(__e83);
          return { nodes: Array.from(__frag81.childNodes), cleanup: () => {
            for (const c of __c82)
              try {
                c();
              } catch (e) {}
          } };
        }
      ]
    });
    const __wireBoundary_b36 = (anchor) => bindIf3({
      anchor,
      test: () => {
        if (!$scope.findHasResults() && !$scope.findSearching())
          return 0;
        return -1;
      },
      branches: [
        () => {
          const __frag86 = document.createDocumentFragment();
          const __c87 = [];
          const __e88 = document.createElement("div");
          __e88.setAttribute("class", "cb-empty");
          const __t89 = document.createTextNode("No matches.");
          __e88.appendChild(__t89);
          __frag86.appendChild(__e88);
          return { nodes: Array.from(__frag86.childNodes), cleanup: () => {
            for (const c of __c87)
              try {
                c();
              } catch (e) {}
          } };
        }
      ]
    });
    const __wireBoundary_b38 = (anchor) => bindFor2({
      anchor,
      items: () => $scope.cmdItems(),
      key: (item, __i) => {
        const __child = Object.create($scope);
        __child.item = item;
        __child.__i = __i;
        return (($scope2) => item.key)(__child);
      },
      render: (item, __i) => {
        const __childScope99 = Object.create($scope);
        __childScope99.item = item;
        __childScope99.__i = __i;
        __childScope99.__v = signal4(0);
        return (($scope2) => {
          const __frag100 = document.createDocumentFragment();
          const __c101 = [];
          const __e102 = document.createElement("div");
          __e102.setAttribute("class", "cb-item");
          __e102.setAttribute("role", "option");
          {
            const __av103 = "cb-item-" + $scope2.item.idx;
            if (__av103 == null || __av103 === false || __av103 === "")
              __e102.removeAttribute("id");
            else
              __e102.setAttribute("id", __av103 === true ? "" : String(__av103));
          }
          __c101.push(bindAttr3(__e102, "aria-selected", () => (void $scope2.__v(), $scope2.bar.selectedIdx() === $scope2.item.idx ? "true" : "false")));
          {
            const __av104 = $scope2.item.key;
            if (__av104 == null || __av104 === false || __av104 === "")
              __e102.removeAttribute("data-cmd");
            else
              __e102.setAttribute("data-cmd", __av104 === true ? "" : String(__av104));
          }
          __c101.push(on3(__e102, "click", function() {
            const __h = () => $scope2.activateCmd($scope2.item.key);
            return typeof __h === "function" ? __h.bind(undefined) : () => {};
          }(), { modifiers: [] }));
          const __e105 = document.createElement("span");
          __e105.setAttribute("style", "font-family:var(--font-mono)");
          const __t106 = document.createTextNode("");
          __e105.appendChild(__t106);
          __t106.textContent = $scope2.item.label;
          __e102.appendChild(__e105);
          const __e107 = document.createElement("span");
          __e102.appendChild(__e107);
          const __e108 = document.createElement("span");
          __e108.setAttribute("class", "hint");
          const __t109 = document.createTextNode("");
          __e108.appendChild(__t109);
          __t109.textContent = $scope2.item.hint;
          __e102.appendChild(__e108);
          __frag100.appendChild(__e102);
          return { nodes: Array.from(__frag100.childNodes), cleanup: () => {
            for (const c of __c101)
              try {
                c();
              } catch (e) {}
          }, update: (__new110, __newI111) => {
            $scope2.item = __new110;
            $scope2.__i = __newI111;
            {
              const __av103 = "cb-item-" + $scope2.item.idx;
              if (__av103 == null || __av103 === false || __av103 === "")
                __e102.removeAttribute("id");
              else
                __e102.setAttribute("id", __av103 === true ? "" : String(__av103));
            }
            {
              const __av104 = $scope2.item.key;
              if (__av104 == null || __av104 === false || __av104 === "")
                __e102.removeAttribute("data-cmd");
              else
                __e102.setAttribute("data-cmd", __av104 === true ? "" : String(__av104));
            }
            __t106.textContent = $scope2.item.label;
            __t109.textContent = $scope2.item.hint;
            $scope2.__v.set((c) => c + 1);
          } };
        })(__childScope99);
      }
    });
    const __wireBoundary_b37 = (anchor) => bindIf3({
      anchor,
      test: () => {
        if ($scope.cmdItems().length > 0)
          return 0;
        return 1;
      },
      branches: [
        () => {
          const __frag93 = document.createDocumentFragment();
          const __c94 = [];
          const __e95 = document.createElement("div");
          __e95.setAttribute("class", "cb-mode-hint");
          const __t96 = document.createTextNode("Actions");
          __e95.appendChild(__t96);
          __frag93.appendChild(__e95);
          const __e97 = document.createElement("div");
          __e97.setAttribute("class", "cb-list");
          __e97.setAttribute("role", "listbox");
          const __forAnchor98 = document.createComment("for");
          __e97.appendChild(__forAnchor98);
          __c94.push(__wireBoundary_b38(__forAnchor98));
          __frag93.appendChild(__e97);
          return { nodes: Array.from(__frag93.childNodes), cleanup: () => {
            for (const c of __c94)
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
          const __t115 = document.createTextNode("No commands match.");
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
    const __wireBoundary_b46 = (anchor) => bindFor2({
      anchor,
      items: () => $scope.mentionItems(),
      key: (item, __i) => {
        const __child = Object.create($scope);
        __child.item = item;
        __child.__i = __i;
        return (($scope2) => item.id + item.kind)(__child);
      },
      render: (item, __i) => {
        const __childScope125 = Object.create($scope);
        __childScope125.item = item;
        __childScope125.__i = __i;
        __childScope125.__v = signal4(0);
        return (($scope2) => {
          const __frag126 = document.createDocumentFragment();
          const __c127 = [];
          const __e128 = document.createElement("div");
          __e128.setAttribute("class", "cb-item");
          __e128.setAttribute("role", "option");
          {
            const __av129 = "cb-item-" + $scope2.item.idx;
            if (__av129 == null || __av129 === false || __av129 === "")
              __e128.removeAttribute("id");
            else
              __e128.setAttribute("id", __av129 === true ? "" : String(__av129));
          }
          __c127.push(bindAttr3(__e128, "aria-selected", () => (void $scope2.__v(), $scope2.bar.selectedIdx() === $scope2.item.idx ? "true" : "false")));
          {
            const __av130 = $scope2.item.kind;
            if (__av130 == null || __av130 === false || __av130 === "")
              __e128.removeAttribute("data-mention-kind");
            else
              __e128.setAttribute("data-mention-kind", __av130 === true ? "" : String(__av130));
          }
          {
            const __av131 = $scope2.item.id;
            if (__av131 == null || __av131 === false || __av131 === "")
              __e128.removeAttribute("data-mention-id");
            else
              __e128.setAttribute("data-mention-id", __av131 === true ? "" : String(__av131));
          }
          __c127.push(on3(__e128, "click", function() {
            const __h = () => $scope2.activateMention($scope2.item);
            return typeof __h === "function" ? __h.bind(undefined) : () => {};
          }(), { modifiers: [] }));
          const __e132 = document.createElement("sui-icon");
          {
            const __av133 = $scope2.item.icon;
            if (__av133 == null || __av133 === false || __av133 === "")
              __e132.removeAttribute("name");
            else
              __e132.setAttribute("name", __av133 === true ? "" : String(__av133));
          }
          __e132.setAttribute("size", "sm");
          __e132.setAttribute("aria-hidden", "true");
          __e128.appendChild(__e132);
          const __e134 = document.createElement("span");
          const __t135 = document.createTextNode("");
          __e134.appendChild(__t135);
          __t135.textContent = $scope2.item.label;
          __e128.appendChild(__e134);
          const __e136 = document.createElement("span");
          __e136.setAttribute("class", "hint");
          const __t137 = document.createTextNode("");
          __e136.appendChild(__t137);
          __t137.textContent = $scope2.item.kind;
          __e128.appendChild(__e136);
          __frag126.appendChild(__e128);
          return { nodes: Array.from(__frag126.childNodes), cleanup: () => {
            for (const c of __c127)
              try {
                c();
              } catch (e) {}
          }, update: (__new138, __newI139) => {
            $scope2.item = __new138;
            $scope2.__i = __newI139;
            {
              const __av129 = "cb-item-" + $scope2.item.idx;
              if (__av129 == null || __av129 === false || __av129 === "")
                __e128.removeAttribute("id");
              else
                __e128.setAttribute("id", __av129 === true ? "" : String(__av129));
            }
            {
              const __av130 = $scope2.item.kind;
              if (__av130 == null || __av130 === false || __av130 === "")
                __e128.removeAttribute("data-mention-kind");
              else
                __e128.setAttribute("data-mention-kind", __av130 === true ? "" : String(__av130));
            }
            {
              const __av131 = $scope2.item.id;
              if (__av131 == null || __av131 === false || __av131 === "")
                __e128.removeAttribute("data-mention-id");
              else
                __e128.setAttribute("data-mention-id", __av131 === true ? "" : String(__av131));
            }
            {
              const __av133 = $scope2.item.icon;
              if (__av133 == null || __av133 === false || __av133 === "")
                __e132.removeAttribute("name");
              else
                __e132.setAttribute("name", __av133 === true ? "" : String(__av133));
            }
            __t135.textContent = $scope2.item.label;
            __t137.textContent = $scope2.item.kind;
            $scope2.__v.set((c) => c + 1);
          } };
        })(__childScope125);
      }
    });
    const __wireBoundary_b45 = (anchor) => bindIf3({
      anchor,
      test: () => {
        if ($scope.mentionItems().length > 0)
          return 0;
        return 1;
      },
      branches: [
        () => {
          const __frag119 = document.createDocumentFragment();
          const __c120 = [];
          const __e121 = document.createElement("div");
          __e121.setAttribute("class", "cb-mode-hint");
          const __t122 = document.createTextNode("Reference");
          __e121.appendChild(__t122);
          __frag119.appendChild(__e121);
          const __e123 = document.createElement("div");
          __e123.setAttribute("class", "cb-list");
          __e123.setAttribute("role", "listbox");
          const __forAnchor124 = document.createComment("for");
          __e123.appendChild(__forAnchor124);
          __c120.push(__wireBoundary_b46(__forAnchor124));
          __frag119.appendChild(__e123);
          return { nodes: Array.from(__frag119.childNodes), cleanup: () => {
            for (const c of __c120)
              try {
                c();
              } catch (e) {}
          } };
        },
        () => {
          const __frag140 = document.createDocumentFragment();
          const __c141 = [];
          const __e142 = document.createElement("div");
          __e142.setAttribute("class", "cb-empty");
          const __t143 = document.createTextNode("No matching views or agents.");
          __e142.appendChild(__t143);
          __frag140.appendChild(__e142);
          return { nodes: Array.from(__frag140.childNodes), cleanup: () => {
            for (const c of __c141)
              try {
                c();
              } catch (e) {}
          } };
        }
      ]
    });
    const __wireBoundary_b55 = (anchor) => bindFor2({
      anchor,
      items: () => $scope.createSurfaces(),
      key: (surf, __i) => {
        const __child = Object.create($scope);
        __child.surf = surf;
        __child.__i = __i;
        return (($scope2) => surf.id)(__child);
      },
      render: (surf, __i) => {
        const __childScope150 = Object.create($scope);
        __childScope150.surf = surf;
        __childScope150.__i = __i;
        __childScope150.__v = signal4(0);
        return (($scope2) => {
          const __wireBoundary_b64 = (anchor2) => bindIf3({
            anchor: anchor2,
            test: () => {
              if ($scope2.surf.disabled)
                return 0;
              return -1;
            },
            branches: [
              () => {
                const __frag163 = document.createDocumentFragment();
                const __c164 = [];
                const __e165 = document.createElement("span");
                __e165.setAttribute("class", "cb-surface-hint");
                const __t166 = document.createTextNode("post-launch");
                __e165.appendChild(__t166);
                __frag163.appendChild(__e165);
                return { nodes: Array.from(__frag163.childNodes), cleanup: () => {
                  for (const c of __c164)
                    try {
                      c();
                    } catch (e) {}
                } };
              }
            ]
          });
          const __frag151 = document.createDocumentFragment();
          const __c152 = [];
          const __e153 = document.createElement("button");
          __e153.setAttribute("class", "cb-surface-btn");
          __e153.setAttribute("type", "button");
          {
            const __av154 = $scope2.surf.id;
            if (__av154 == null || __av154 === false || __av154 === "")
              __e153.removeAttribute("data-surface");
            else
              __e153.setAttribute("data-surface", __av154 === true ? "" : String(__av154));
          }
          __c152.push(bindAttr3(__e153, "aria-pressed", () => (void $scope2.__v(), $scope2.bar.createSurface() === $scope2.surf.id ? "true" : "false")));
          {
            const __av155 = $scope2.surf.disabled ? "true" : "false";
            if (__av155 == null || __av155 === false || __av155 === "")
              __e153.removeAttribute("aria-disabled");
            else
              __e153.setAttribute("aria-disabled", __av155 === true ? "" : String(__av155));
          }
          {
            const __av156 = $scope2.surf.disabled;
            if (__av156 == null || __av156 === false || __av156 === "")
              __e153.removeAttribute("disabled");
            else
              __e153.setAttribute("disabled", __av156 === true ? "" : String(__av156));
          }
          {
            const __av157 = $scope2.surf.disabled ? "post-launch" : $scope2.surf.label;
            if (__av157 == null || __av157 === false || __av157 === "")
              __e153.removeAttribute("title");
            else
              __e153.setAttribute("title", __av157 === true ? "" : String(__av157));
          }
          __c152.push(on3(__e153, "click", function() {
            const __h = () => $scope2.selectSurface($scope2.surf.id);
            return typeof __h === "function" ? __h.bind(undefined) : () => {};
          }(), { modifiers: [] }));
          const __e158 = document.createElement("sui-icon");
          {
            const __av159 = $scope2.surf.icon;
            if (__av159 == null || __av159 === false || __av159 === "")
              __e158.removeAttribute("name");
            else
              __e158.setAttribute("name", __av159 === true ? "" : String(__av159));
          }
          __e158.setAttribute("aria-hidden", "true");
          __e153.appendChild(__e158);
          const __e160 = document.createElement("span");
          const __t161 = document.createTextNode("");
          __e160.appendChild(__t161);
          __t161.textContent = $scope2.surf.label;
          __e153.appendChild(__e160);
          const __ifAnchor162 = document.createComment("if");
          __e153.appendChild(__ifAnchor162);
          __c152.push(__wireBoundary_b64(__ifAnchor162));
          __frag151.appendChild(__e153);
          return { nodes: Array.from(__frag151.childNodes), cleanup: () => {
            for (const c of __c152)
              try {
                c();
              } catch (e) {}
          }, update: (__new167, __newI168) => {
            $scope2.surf = __new167;
            $scope2.__i = __newI168;
            {
              const __av154 = $scope2.surf.id;
              if (__av154 == null || __av154 === false || __av154 === "")
                __e153.removeAttribute("data-surface");
              else
                __e153.setAttribute("data-surface", __av154 === true ? "" : String(__av154));
            }
            {
              const __av155 = $scope2.surf.disabled ? "true" : "false";
              if (__av155 == null || __av155 === false || __av155 === "")
                __e153.removeAttribute("aria-disabled");
              else
                __e153.setAttribute("aria-disabled", __av155 === true ? "" : String(__av155));
            }
            {
              const __av156 = $scope2.surf.disabled;
              if (__av156 == null || __av156 === false || __av156 === "")
                __e153.removeAttribute("disabled");
              else
                __e153.setAttribute("disabled", __av156 === true ? "" : String(__av156));
            }
            {
              const __av157 = $scope2.surf.disabled ? "post-launch" : $scope2.surf.label;
              if (__av157 == null || __av157 === false || __av157 === "")
                __e153.removeAttribute("title");
              else
                __e153.setAttribute("title", __av157 === true ? "" : String(__av157));
            }
            {
              const __av159 = $scope2.surf.icon;
              if (__av159 == null || __av159 === false || __av159 === "")
                __e158.removeAttribute("name");
              else
                __e158.setAttribute("name", __av159 === true ? "" : String(__av159));
            }
            __t161.textContent = $scope2.surf.label;
            $scope2.__v.set((c) => c + 1);
          } };
        })(__childScope150);
      }
    });
    const __wireBoundary_b69 = (anchor) => bindFor2({
      anchor,
      items: () => $scope.currentSurfaceDef().shapes,
      key: (sh, __i) => {
        const __child = Object.create($scope);
        __child.sh = sh;
        __child.__i = __i;
        return (($scope2) => sh)(__child);
      },
      render: (sh, __i) => {
        const __childScope179 = Object.create($scope);
        __childScope179.sh = sh;
        __childScope179.__i = __i;
        __childScope179.__v = signal4(0);
        return (($scope2) => {
          const __frag180 = document.createDocumentFragment();
          const __c181 = [];
          const __e182 = document.createElement("option");
          {
            const __av183 = $scope2.sh;
            if (__av183 == null || __av183 === false || __av183 === "")
              __e182.removeAttribute("value");
            else
              __e182.setAttribute("value", __av183 === true ? "" : String(__av183));
          }
          const __t184 = document.createTextNode("");
          __e182.appendChild(__t184);
          __t184.textContent = $scope2.sh;
          __frag180.appendChild(__e182);
          return { nodes: Array.from(__frag180.childNodes), cleanup: () => {
            for (const c of __c181)
              try {
                c();
              } catch (e) {}
          }, update: (__new185, __newI186) => {
            $scope2.sh = __new185;
            $scope2.__i = __newI186;
            {
              const __av183 = $scope2.sh;
              if (__av183 == null || __av183 === false || __av183 === "")
                __e182.removeAttribute("value");
              else
                __e182.setAttribute("value", __av183 === true ? "" : String(__av183));
            }
            __t184.textContent = $scope2.sh;
            $scope2.__v.set((c) => c + 1);
          } };
        })(__childScope179);
      }
    });
    const __wireBoundary_b73 = (anchor) => bindIf3({
      anchor,
      test: () => {
        if ($scope.bar.createError())
          return 0;
        return -1;
      },
      branches: [
        () => {
          const __frag192 = document.createDocumentFragment();
          const __c193 = [];
          const __e194 = document.createElement("div");
          __e194.setAttribute("class", "cb-create-error");
          const __t195 = document.createTextNode("");
          __e194.appendChild(__t195);
          __c193.push(bindText3(__t195, () => $scope.bar.createError()));
          __frag192.appendChild(__e194);
          return { nodes: Array.from(__frag192.childNodes), cleanup: () => {
            for (const c of __c193)
              try {
                c();
              } catch (e) {}
          } };
        }
      ]
    });
    const __wireBoundary_b80 = (anchor) => bindFor2({
      anchor,
      items: () => $scope.bar.agents(),
      key: (a, __i) => {
        const __child = Object.create($scope);
        __child.a = a;
        __child.__i = __i;
        return (($scope2) => a.id)(__child);
      },
      render: (a, __i) => {
        const __childScope216 = Object.create($scope);
        __childScope216.a = a;
        __childScope216.__i = __i;
        __childScope216.__v = signal4(0);
        return (($scope2) => {
          const __frag217 = document.createDocumentFragment();
          const __c218 = [];
          const __e219 = document.createElement("option");
          {
            const __av220 = $scope2.a.id;
            if (__av220 == null || __av220 === false || __av220 === "")
              __e219.removeAttribute("value");
            else
              __e219.setAttribute("value", __av220 === true ? "" : String(__av220));
          }
          const __t221 = document.createTextNode("");
          __e219.appendChild(__t221);
          __c218.push(bindText3(__t221, () => (void $scope2.__v(), $scope2.a.name ?? $scope2.a.label ?? $scope2.a.id)));
          __frag217.appendChild(__e219);
          return { nodes: Array.from(__frag217.childNodes), cleanup: () => {
            for (const c of __c218)
              try {
                c();
              } catch (e) {}
          }, update: (__new222, __newI223) => {
            $scope2.a = __new222;
            $scope2.__i = __newI223;
            {
              const __av220 = $scope2.a.id;
              if (__av220 == null || __av220 === false || __av220 === "")
                __e219.removeAttribute("value");
              else
                __e219.setAttribute("value", __av220 === true ? "" : String(__av220));
            }
            $scope2.__v.set((c) => c + 1);
          } };
        })(__childScope216);
      }
    });
    const __wireBoundary_b79 = (anchor) => bindIf3({
      anchor,
      test: () => {
        if ($scope.bar.agents().length === 0)
          return 0;
        return 1;
      },
      branches: [
        () => {
          const __frag209 = document.createDocumentFragment();
          const __c210 = [];
          const __e211 = document.createElement("option");
          __e211.setAttribute("value", "");
          const __t212 = document.createTextNode("No agents found");
          __e211.appendChild(__t212);
          __frag209.appendChild(__e211);
          return { nodes: Array.from(__frag209.childNodes), cleanup: () => {
            for (const c of __c210)
              try {
                c();
              } catch (e) {}
          } };
        },
        () => {
          const __frag213 = document.createDocumentFragment();
          const __c214 = [];
          const __forAnchor215 = document.createComment("for");
          __frag213.appendChild(__forAnchor215);
          __c214.push(__wireBoundary_b80(__forAnchor215));
          return { nodes: Array.from(__frag213.childNodes), cleanup: () => {
            for (const c of __c214)
              try {
                c();
              } catch (e) {}
          } };
        }
      ]
    });
    const __wireBoundary_b85 = (anchor) => bindFor2({
      anchor,
      items: () => $scope.bar.providers(),
      key: (p, __i) => {
        const __child = Object.create($scope);
        __child.p = p;
        __child.__i = __i;
        return (($scope2) => p)(__child);
      },
      render: (p, __i) => {
        const __childScope236 = Object.create($scope);
        __childScope236.p = p;
        __childScope236.__i = __i;
        __childScope236.__v = signal4(0);
        return (($scope2) => {
          const __frag237 = document.createDocumentFragment();
          const __c238 = [];
          const __e239 = document.createElement("option");
          {
            const __av240 = $scope2.p;
            if (__av240 == null || __av240 === false || __av240 === "")
              __e239.removeAttribute("value");
            else
              __e239.setAttribute("value", __av240 === true ? "" : String(__av240));
          }
          const __t241 = document.createTextNode("");
          __e239.appendChild(__t241);
          __t241.textContent = $scope2.p;
          __frag237.appendChild(__e239);
          return { nodes: Array.from(__frag237.childNodes), cleanup: () => {
            for (const c of __c238)
              try {
                c();
              } catch (e) {}
          }, update: (__new242, __newI243) => {
            $scope2.p = __new242;
            $scope2.__i = __newI243;
            {
              const __av240 = $scope2.p;
              if (__av240 == null || __av240 === false || __av240 === "")
                __e239.removeAttribute("value");
              else
                __e239.setAttribute("value", __av240 === true ? "" : String(__av240));
            }
            __t241.textContent = $scope2.p;
            $scope2.__v.set((c) => c + 1);
          } };
        })(__childScope236);
      }
    });
    const __wireBoundary_b84 = (anchor) => bindIf3({
      anchor,
      test: () => {
        if ($scope.bar.providers().length === 0)
          return 0;
        return 1;
      },
      branches: [
        () => {
          const __frag229 = document.createDocumentFragment();
          const __c230 = [];
          const __e231 = document.createElement("option");
          __e231.setAttribute("value", "");
          const __t232 = document.createTextNode("No providers installed");
          __e231.appendChild(__t232);
          __frag229.appendChild(__e231);
          return { nodes: Array.from(__frag229.childNodes), cleanup: () => {
            for (const c of __c230)
              try {
                c();
              } catch (e) {}
          } };
        },
        () => {
          const __frag233 = document.createDocumentFragment();
          const __c234 = [];
          const __forAnchor235 = document.createComment("for");
          __frag233.appendChild(__forAnchor235);
          __c234.push(__wireBoundary_b85(__forAnchor235));
          return { nodes: Array.from(__frag233.childNodes), cleanup: () => {
            for (const c of __c234)
              try {
                c();
              } catch (e) {}
          } };
        }
      ]
    });
    const __wireBoundary_b88 = (anchor) => bindIf3({
      anchor,
      test: () => {
        if ($scope.bar.spawnError())
          return 0;
        return -1;
      },
      branches: [
        () => {
          const __frag245 = document.createDocumentFragment();
          const __c246 = [];
          const __e247 = document.createElement("div");
          __e247.setAttribute("class", "cb-spawn-error");
          const __t248 = document.createTextNode("");
          __e247.appendChild(__t248);
          __c246.push(bindText3(__t248, () => $scope.bar.spawnError()));
          __frag245.appendChild(__e247);
          return { nodes: Array.from(__frag245.childNodes), cleanup: () => {
            for (const c of __c246)
              try {
                c();
              } catch (e) {}
          } };
        }
      ]
    });
    const __wireBoundary_b21 = (anchor) => bindIf3({
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
          const __forAnchor54 = document.createComment("for");
          __frag52.appendChild(__forAnchor54);
          __c53.push(__wireBoundary_b22(__forAnchor54));
          const __ifAnchor80 = document.createComment("if");
          __frag52.appendChild(__ifAnchor80);
          __c53.push(__wireBoundary_b35(__ifAnchor80));
          const __ifAnchor85 = document.createComment("if");
          __frag52.appendChild(__ifAnchor85);
          __c53.push(__wireBoundary_b36(__ifAnchor85));
          return { nodes: Array.from(__frag52.childNodes), cleanup: () => {
            for (const c of __c53)
              try {
                c();
              } catch (e) {}
          } };
        },
        () => {
          const __frag90 = document.createDocumentFragment();
          const __c91 = [];
          const __ifAnchor92 = document.createComment("if");
          __frag90.appendChild(__ifAnchor92);
          __c91.push(__wireBoundary_b37(__ifAnchor92));
          return { nodes: Array.from(__frag90.childNodes), cleanup: () => {
            for (const c of __c91)
              try {
                c();
              } catch (e) {}
          } };
        },
        () => {
          const __frag116 = document.createDocumentFragment();
          const __c117 = [];
          const __ifAnchor118 = document.createComment("if");
          __frag116.appendChild(__ifAnchor118);
          __c117.push(__wireBoundary_b45(__ifAnchor118));
          return { nodes: Array.from(__frag116.childNodes), cleanup: () => {
            for (const c of __c117)
              try {
                c();
              } catch (e) {}
          } };
        },
        () => {
          const __frag144 = document.createDocumentFragment();
          const __c145 = [];
          const __e146 = document.createElement("div");
          __e146.setAttribute("class", "cb-mode-hint");
          const __t147 = document.createTextNode("Create");
          __e146.appendChild(__t147);
          __frag144.appendChild(__e146);
          const __e148 = document.createElement("div");
          __e148.setAttribute("class", "cb-create-surfaces");
          __e148.setAttribute("role", "group");
          __e148.setAttribute("aria-label", "Surface type");
          const __forAnchor149 = document.createComment("for");
          __e148.appendChild(__forAnchor149);
          __c145.push(__wireBoundary_b55(__forAnchor149));
          __frag144.appendChild(__e148);
          const __e169 = document.createElement("div");
          __e169.setAttribute("class", "cb-create-grid");
          const __e170 = document.createElement("label");
          const __e171 = document.createElement("span");
          const __t172 = document.createTextNode("Name (kebab-case)");
          __e171.appendChild(__t172);
          __e170.appendChild(__e171);
          const __e173 = document.createElement("input");
          __e173.setAttribute("type", "text");
          __e173.setAttribute("class", "cb-create-name");
          __c145.push(bindAttr3(__e173, "placeholder", () => "my-" + $scope.bar.createSurface()));
          __c145.push(bindValue2(__e173, $scope.bar.createName));
          __e173.setAttribute("autocomplete", "off");
          __e170.appendChild(__e173);
          __e169.appendChild(__e170);
          const __e174 = document.createElement("label");
          const __e175 = document.createElement("span");
          const __t176 = document.createTextNode("");
          __e175.appendChild(__t176);
          __c145.push(bindText3(__t176, () => $scope.currentSurfaceDef().shapeLabel));
          __e174.appendChild(__e175);
          const __e177 = document.createElement("select");
          __e177.setAttribute("class", "cb-create-shape");
          __c145.push(bindValue2(__e177, $scope.bar.createShape));
          const __forAnchor178 = document.createComment("for");
          __e177.appendChild(__forAnchor178);
          __c145.push(__wireBoundary_b69(__forAnchor178));
          __e174.appendChild(__e177);
          __e169.appendChild(__e174);
          const __e187 = document.createElement("label");
          __e187.setAttribute("class", "span-2");
          const __e188 = document.createElement("span");
          const __t189 = document.createTextNode("Purpose (optional)");
          __e188.appendChild(__t189);
          __e187.appendChild(__e188);
          const __e190 = document.createElement("input");
          __e190.setAttribute("type", "text");
          __e190.setAttribute("class", "cb-create-purpose");
          __e190.setAttribute("placeholder", "One-line description");
          __c145.push(bindValue2(__e190, $scope.bar.createPurpose));
          __e190.setAttribute("autocomplete", "off");
          __e187.appendChild(__e190);
          __e169.appendChild(__e187);
          __frag144.appendChild(__e169);
          const __ifAnchor191 = document.createComment("if");
          __frag144.appendChild(__ifAnchor191);
          __c145.push(__wireBoundary_b73(__ifAnchor191));
          const __e196 = document.createElement("div");
          __e196.setAttribute("class", "cb-create-actions");
          const __e197 = document.createElement("button");
          __e197.setAttribute("class", "cb-create-submit");
          __e197.setAttribute("type", "button");
          __c145.push(bindAttr3(__e197, "disabled", () => $scope.bar.createSubmitting() || !$scope.createNameValid()));
          __c145.push(on3(__e197, "click", function() {
            const __h = $scope.submitCreate;
            return typeof __h === "function" ? __h.bind(undefined) : () => {};
          }(), { modifiers: [] }));
          const __t198 = document.createTextNode("");
          __e197.appendChild(__t198);
          __c145.push(bindText3(__t198, () => $scope.bar.createSubmitting() ? "Creating…" : "Create " + $scope.currentSurfaceDef().label));
          __e196.appendChild(__e197);
          __frag144.appendChild(__e196);
          return { nodes: Array.from(__frag144.childNodes), cleanup: () => {
            for (const c of __c145)
              try {
                c();
              } catch (e) {}
          } };
        },
        () => {
          const __frag199 = document.createDocumentFragment();
          const __c200 = [];
          const __e201 = document.createElement("div");
          __e201.setAttribute("class", "cb-spawn-card");
          const __e202 = document.createElement("div");
          __e202.setAttribute("class", "cb-spawn-title");
          const __t203 = document.createTextNode("Start a session");
          __e202.appendChild(__t203);
          __e201.appendChild(__e202);
          const __e204 = document.createElement("div");
          __e204.setAttribute("class", "cb-spawn-row");
          const __e205 = document.createElement("label");
          const __t206 = document.createTextNode("Agent");
          __e205.appendChild(__t206);
          __e204.appendChild(__e205);
          const __e207 = document.createElement("select");
          __e207.setAttribute("class", "cb-spawn-agent");
          __c200.push(bindValue2(__e207, $scope.bar.spawnAgent));
          const __ifAnchor208 = document.createComment("if");
          __e207.appendChild(__ifAnchor208);
          __c200.push(__wireBoundary_b79(__ifAnchor208));
          __e204.appendChild(__e207);
          __e201.appendChild(__e204);
          const __e224 = document.createElement("div");
          __e224.setAttribute("class", "cb-spawn-row");
          const __e225 = document.createElement("label");
          const __t226 = document.createTextNode("Provider");
          __e225.appendChild(__t226);
          __e224.appendChild(__e225);
          const __e227 = document.createElement("select");
          __e227.setAttribute("class", "cb-spawn-provider");
          __c200.push(bindValue2(__e227, $scope.bar.spawnProvider));
          const __ifAnchor228 = document.createComment("if");
          __e227.appendChild(__ifAnchor228);
          __c200.push(__wireBoundary_b84(__ifAnchor228));
          __e224.appendChild(__e227);
          __e201.appendChild(__e224);
          const __ifAnchor244 = document.createComment("if");
          __e201.appendChild(__ifAnchor244);
          __c200.push(__wireBoundary_b88(__ifAnchor244));
          const __e249 = document.createElement("button");
          __e249.setAttribute("class", "cb-spawn-start");
          __e249.setAttribute("type", "button");
          __c200.push(bindAttr3(__e249, "disabled", () => $scope.bar.spawning() || !$scope.bar.spawnAgent() || !$scope.bar.spawnProvider()));
          __c200.push(on3(__e249, "click", function() {
            const __h = $scope.spawnSession;
            return typeof __h === "function" ? __h.bind(undefined) : () => {};
          }(), { modifiers: [] }));
          const __t250 = document.createTextNode("");
          __e249.appendChild(__t250);
          __c200.push(bindText3(__t250, () => $scope.bar.spawning() ? "Starting…" : "Start session"));
          __e201.appendChild(__e249);
          __frag199.appendChild(__e201);
          return { nodes: Array.from(__frag199.childNodes), cleanup: () => {
            for (const c of __c200)
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
    __cleanups.push(bindAttr3(__e3, "placeholder", () => $scope.searchPlaceholder()));
    __cleanups.push(bindValue2(__e3, $scope.bar.value));
    __e3.setAttribute("aria-label", "Command bar search");
    __e3.setAttribute("aria-autocomplete", "list");
    __cleanups.push(bindAttr3(__e3, "aria-activedescendant", () => $scope.activeDescendantId()));
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
    __cleanups.push(__wireBoundary_b9(__ifAnchor11));
    __e0.appendChild(__e6);
    __root.appendChild(__e0);
    const __e50 = document.createElement("div");
    __e50.setAttribute("class", "cb-body");
    const __matchAnchor51 = document.createComment("match");
    __e50.appendChild(__matchAnchor51);
    __cleanups.push(__wireBoundary_b21(__matchAnchor51));
    __root.appendChild(__e50);
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
