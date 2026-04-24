//! `<ship-command-bar>` — the single input surface.
//!
//! Mounts in the nav-top where the `cmdk-trigger` used to live. Clicking
//! or pressing ⌘K focuses it. The input routes by shape:
//!   - `/foo` → slash command (Ship actions: scaffold, lint, fork, ...)
//!   - `>foo` → fuzzy find (views, settings, recent)
//!   - `@foo` → mention (agent, view, file)
//!   - `foo`  → message to the current AI session
//!
//! On input or focus the bar raises a flyout below it; the flyout
//! renders the appropriate affordance for the mode (command list,
//! results list, mention picker, session thread).
//!
//! Session chip on the right shows the current `(workspace, agent,
//! provider)` with a status dot. Clicking opens the session popover
//! (agent picker / "view all chats" / start-new-session).
//!
//! Phase-2 scope: shape + modes + session chip stub + flyout
//! scaffolding. The mode-specific content (spawn card, thread view,
//! command list, results) lands across the next commits.

import { LitElement, css, html, nothing } from "lit";
import { customElement, property, state } from "lit/decorators.js";

type BarMode = "ai" | "command" | "find" | "mention" | "create";

type CreateSurface =
  | "view"
  | "skill"
  | "agent"
  | "pipeline"
  | "trigger"
  | "theme";

interface CreateSurfaceDef {
  id: CreateSurface;
  label: string;
  icon: string;
  shapes: string[];
  shapeLabel: string;
}

const CREATE_SURFACES: CreateSurfaceDef[] = [
  { id: "view", label: "View", icon: "app-window",
    shapes: ["list", "form", "detail", "dashboard", "tabs", "blank"],
    shapeLabel: "Starting shape" },
  { id: "skill", label: "Skill", icon: "sparkles",
    shapes: ["task", "reference", "workflow"], shapeLabel: "Skill kind" },
  { id: "agent", label: "Agent", icon: "bot",
    shapes: ["specialist", "generalist", "reviewer"], shapeLabel: "Agent kind" },
  { id: "pipeline", label: "Pipeline", icon: "workflow",
    shapes: ["sequential", "fanout", "review-gated"], shapeLabel: "Topology" },
  { id: "trigger", label: "Trigger", icon: "zap",
    shapes: ["webhook", "schedule", "file-change"], shapeLabel: "Kind" },
  { id: "theme", label: "Theme", icon: "palette",
    shapes: ["light", "dark", "brand-accent", "high-contrast"],
    shapeLabel: "Starting shape" },
];

const KEBAB_RE = /^[a-z][a-z0-9-]{0,48}$/;

interface SessionSummary {
  id: string;
  workspace_id: string;
  agent_id?: string;
  provider?: string;
  status: string;
}

interface AgentSummary {
  id: string;
  name?: string;
  label?: string;
  providers?: string[];
}

interface ThreadLine {
  id: string;
  sender: "user" | "agent" | "system";
  text: string;
  detail?: string;
}

@customElement("ship-command-bar")
export class ShipCommandBar extends LitElement {
  static styles = css`
    :host {
      display: inline-flex;
      align-items: center;
      flex: 1;
      max-width: 320px;
      min-width: 200px;
      font: inherit;
    }
    /* ── Trigger (slim button in the nav-top) ────────────────────── */
    .trigger {
      display: flex;
      align-items: center;
      gap: 8px;
      padding: 4px 12px;
      width: 100%;
      border: 1px solid var(--border);
      border-radius: var(--radius-md, 6px);
      background: var(--background);
      color: var(--muted-foreground);
      font: inherit;
      font-size: var(--text-xs);
      cursor: pointer;
      transition: border-color var(--duration-fast, 120ms) var(--ease-out, ease);
    }
    .trigger:hover {
      border-color: color-mix(in oklch, var(--foreground) 30%, var(--border));
    }
    .trigger .label {
      flex: 1;
      text-align: left;
    }
    .trigger kbd {
      font-family: var(--font-mono);
      font-size: 10px;
      padding: 1px 5px;
      border: 1px solid var(--border);
      border-radius: 3px;
      background: var(--muted);
      color: var(--muted-foreground);
    }
    .trigger .dot {
      width: 6px;
      height: 6px;
      border-radius: 999px;
      background: var(--muted-foreground);
    }
    .trigger .dot.active {
      background: var(--success, #16a34a);
    }
    .trigger .dot.thinking {
      background: var(--warning, #d97706);
    }
    /* ── Centered modal overlay ─────────────────────────────────── */
    .backdrop {
      position: fixed;
      inset: 0;
      background: rgba(0, 0, 0, 0.4);
      backdrop-filter: blur(2px);
      z-index: 9900;
    }
    :host-context(.dark) .backdrop {
      background: rgba(0, 0, 0, 0.6);
    }
    .panel {
      position: fixed;
      top: max(80px, 12%);
      left: 50%;
      transform: translateX(-50%);
      width: min(720px, 94vw);
      max-height: min(70vh, 600px);
      background: var(--card, var(--background));
      border: 1px solid var(--border);
      border-radius: var(--radius-lg, 10px);
      box-shadow: 0 24px 60px rgba(0, 0, 0, 0.22);
      z-index: 9901;
      display: flex;
      flex-direction: column;
      color: var(--foreground);
      overflow: hidden;
    }
    .panel-input {
      display: flex;
      align-items: center;
      gap: 8px;
      padding: var(--space-3, 12px) var(--space-4, 16px);
      border-bottom: 1px solid var(--border);
    }
    .panel-input kbd {
      font-family: var(--font-mono);
      font-size: 10px;
      padding: 1px 5px;
      border: 1px solid var(--border);
      border-radius: 3px;
      background: var(--muted);
      color: var(--muted-foreground);
    }
    .panel-chip {
      display: inline-flex;
      align-items: center;
      gap: 6px;
      padding: 2px 8px;
      border-left: 1px solid var(--border);
      color: var(--foreground);
      font-size: var(--text-xs);
      cursor: pointer;
    }
    .panel-chip .dot {
      width: 6px;
      height: 6px;
      border-radius: 999px;
      background: var(--muted-foreground);
    }
    .panel-chip .dot.active {
      background: var(--success, #16a34a);
    }
    .panel-chip .dot.thinking {
      background: var(--warning, #d97706);
    }
    .panel-body {
      flex: 1;
      overflow: auto;
      padding: var(--space-3, 12px) var(--space-4, 16px);
      color: var(--foreground);
    }
    input {
      appearance: none;
      border: 0;
      background: transparent;
      color: inherit;
      flex: 1;
      padding: 4px 0;
      font: inherit;
      font-size: var(--text-sm);
      min-width: 0;
    }
    input::placeholder {
      color: var(--muted-foreground);
    }
    input:focus {
      outline: none;
    }
    kbd {
      font-family: var(--font-mono);
      font-size: 10px;
      padding: 1px 5px;
      border: 1px solid var(--border);
      border-radius: 3px;
      background: var(--muted);
      color: var(--muted-foreground);
    }
    .mode-glyph {
      color: var(--muted-foreground);
      width: 18px;
      text-align: center;
      font-family: var(--font-mono);
      font-size: 12px;
    }
    .mode-glyph.ai {
      color: var(--primary);
    }
    .mode-hint {
      font-size: 10px;
      text-transform: uppercase;
      letter-spacing: 0.08em;
      color: var(--muted-foreground);
      margin-bottom: var(--space-2, 8px);
    }
    .empty-msg {
      padding: var(--space-4, 16px);
      color: var(--muted-foreground);
      font-size: var(--text-sm);
      text-align: center;
    }
    .nav-list {
      display: grid;
      gap: 2px;
    }
    .nav-item {
      display: grid;
      grid-template-columns: 20px 1fr auto;
      gap: 8px;
      align-items: center;
      padding: 6px 10px;
      border-radius: var(--radius-sm, 4px);
      font-size: var(--text-sm);
      cursor: pointer;
      color: var(--foreground);
    }
    .nav-item:hover,
    .nav-item.selected {
      background: var(--muted);
    }
    .nav-item .hint {
      color: var(--muted-foreground);
      font-size: var(--text-xs);
    }
    .session-popover {
      position: absolute;
      top: 100%;
      right: 0;
      margin-top: 6px;
      min-width: 280px;
      background: var(--card, var(--background));
      border: 1px solid var(--border);
      border-radius: var(--radius-md, 6px);
      box-shadow: 0 12px 24px rgba(0, 0, 0, 0.18);
      z-index: 9902;
      padding: 6px;
      font-size: var(--text-sm);
    }
    .session-row {
      display: grid;
      grid-template-columns: 10px 1fr auto;
      gap: 8px;
      align-items: center;
      padding: 6px 10px;
      border-radius: var(--radius-sm, 4px);
      cursor: pointer;
    }
    .session-row:hover {
      background: var(--muted);
    }
    .session-row.current {
      color: var(--primary);
    }
    .session-row .meta {
      font-size: var(--text-xs);
      color: var(--muted-foreground);
    }
    .session-row .end {
      appearance: none;
      background: transparent;
      border: 0;
      color: var(--muted-foreground);
      cursor: pointer;
      font: inherit;
      font-size: var(--text-xs);
      padding: 2px 6px;
    }
    .session-row .end:hover {
      color: var(--destructive);
    }
    .session-sep {
      height: 1px;
      background: var(--border);
      margin: 4px 0;
    }
    .session-start-new {
      display: flex;
      align-items: center;
      gap: 6px;
      padding: 6px 10px;
      border-radius: var(--radius-sm, 4px);
      color: var(--muted-foreground);
      cursor: pointer;
    }
    .session-start-new:hover {
      background: var(--muted);
      color: var(--foreground);
    }
    .panel-chip-wrap {
      position: relative;
    }
    .create-surfaces {
      display: grid;
      grid-template-columns: repeat(6, minmax(0, 1fr));
      gap: 8px;
      margin-bottom: var(--space-3, 12px);
    }
    .create-surface {
      appearance: none;
      background: var(--background);
      color: var(--foreground);
      border: 1px solid var(--border);
      border-radius: var(--radius-md, 6px);
      padding: 10px 4px;
      cursor: pointer;
      display: grid;
      justify-items: center;
      gap: 4px;
      font: inherit;
      font-size: var(--text-xs);
    }
    .create-surface[aria-pressed="true"] {
      border-color: var(--primary);
      background: color-mix(in oklch, var(--primary) 10%, transparent);
      color: var(--primary);
    }
    .create-grid {
      display: grid;
      grid-template-columns: 1fr 1fr;
      gap: var(--space-2, 8px) var(--space-3, 12px);
      margin-bottom: var(--space-3, 12px);
    }
    .create-grid label {
      display: grid;
      gap: 4px;
      font-size: var(--text-xs);
      color: var(--muted-foreground);
    }
    .create-grid label.span-2 {
      grid-column: 1 / -1;
    }
    .create-grid input,
    .create-grid select {
      font: inherit;
      font-size: var(--text-sm);
      color: var(--foreground);
      background: var(--background);
      border: 1px solid var(--border);
      border-radius: var(--radius-sm, 4px);
      padding: 6px 10px;
    }
    .create-error {
      color: var(--destructive);
      font-size: var(--text-xs);
      margin-bottom: var(--space-2, 8px);
    }
    .create-actions {
      display: flex;
      justify-content: flex-end;
    }
    .create-submit {
      appearance: none;
      font: inherit;
      font-size: var(--text-sm);
      padding: 6px 14px;
      border-radius: var(--radius-sm, 4px);
      background: var(--primary);
      color: var(--primary-foreground, #fff);
      border: 1px solid var(--primary);
      cursor: pointer;
    }
    .create-submit:disabled {
      opacity: 0.5;
      cursor: not-allowed;
    }
    .spawn-card {
      display: grid;
      gap: var(--space-2, 8px);
    }
    .spawn-title {
      font-weight: var(--font-weight-medium, 500);
      font-size: var(--text-sm);
    }
    .spawn-row {
      display: grid;
      grid-template-columns: 80px 1fr;
      gap: var(--space-2, 8px);
      align-items: center;
    }
    .spawn-row label {
      font-size: var(--text-xs);
      color: var(--muted-foreground);
    }
    .spawn-row select {
      font: inherit;
      font-size: var(--text-sm);
      border: 1px solid var(--border);
      background: var(--background);
      color: var(--foreground);
      border-radius: var(--radius-sm, 4px);
      padding: 4px 8px;
    }
    .spawn-error {
      color: var(--destructive);
      font-size: var(--text-xs);
    }
    .spawn-start {
      appearance: none;
      font: inherit;
      font-size: var(--text-sm);
      padding: 6px 12px;
      border-radius: var(--radius-sm, 4px);
      background: var(--primary);
      color: var(--primary-foreground, #fff);
      border: 1px solid var(--primary);
      cursor: pointer;
      justify-self: end;
    }
    .spawn-start:disabled {
      opacity: 0.5;
      cursor: not-allowed;
    }
    .thread {
      display: grid;
      gap: var(--space-2, 8px);
      font-size: var(--text-sm);
    }
    .thread-empty {
      color: var(--muted-foreground);
      font-size: var(--text-xs);
    }
    .turn {
      display: grid;
      grid-template-columns: 60px 1fr;
      gap: var(--space-2, 8px);
    }
    .turn .sender {
      font-size: 10px;
      text-transform: uppercase;
      letter-spacing: 0.06em;
      color: var(--muted-foreground);
      padding-top: 2px;
    }
    .turn[data-sender="user"] .sender {
      color: var(--primary);
    }
    .turn[data-sender="agent"] .sender {
      color: var(--success, #16a34a);
    }
    .turn .text {
      white-space: pre-wrap;
      overflow-wrap: anywhere;
      font-family: var(--font-mono);
      font-size: var(--text-xs);
    }
  `;

  /** Controlled open state. The shell can imperatively open/close via
   * `focus()` / `blur()` or by setting this property. */
  @property({ type: Boolean, reflect: true })
  open = false;

  @state() private _value = "";
  @state() private _mode: BarMode = "ai";
  @state() private _session: SessionSummary | null = null;
  @state() private _agents: AgentSummary[] = [];
  @state() private _providers: string[] = [];
  @state() private _spawnAgent: string = "";
  @state() private _spawnProvider: string = "";
  @state() private _spawning = false;
  @state() private _spawnError: string | null = null;
  @state() private _thread: ThreadLine[] = [];
  @state() private _workspaceId: string = "main";
  @state() private _sessionPickerOpen = false;
  @state() private _allSessions: SessionSummary[] = [];
  @state() private _createSurface: CreateSurface = "view";
  @state() private _createName = "";
  @state() private _createPurpose = "";
  @state() private _createShape = "list";
  @state() private _createSubmitting = false;
  @state() private _createError: string | null = null;
  private _streamAbort: AbortController | null = null;

  private _lastSpaceAt = 0;

  private _onDocKeydown = (e: KeyboardEvent): void => {
    const meta = e.metaKey || e.ctrlKey;
    if (meta && e.key.toLowerCase() === "k") {
      e.preventDefault();
      this.focus("find");
      return;
    }
    if (e.key === "Escape" && this.open) {
      this.blur();
      return;
    }
    // Space+Space doubletap → open in AI mode. Only when focus isn't
    // inside an input/textarea/contenteditable so regular typing
    // doesn't trigger. Gap ≤ 350ms for "doubletap" feel.
    if (e.key === " " && !this.open) {
      const target = e.target as HTMLElement | null;
      if (
        target &&
        (target.tagName === "INPUT" ||
          target.tagName === "TEXTAREA" ||
          target.isContentEditable ||
          target.closest("ship-code-editor") ||
          target.closest("ship-command-bar"))
      ) {
        return;
      }
      const now = performance.now();
      if (now - this._lastSpaceAt < 350) {
        this._lastSpaceAt = 0;
        e.preventDefault();
        void this.focus("ai");
      } else {
        this._lastSpaceAt = now;
      }
    }
  };

  private _onDocClick = (e: MouseEvent): void => {
    if (!this.open) return;
    const t = e.target as Node | null;
    if (t && !this.contains(t) && !this.renderRoot.contains(t)) {
      this.open = false;
    }
  };

  connectedCallback(): void {
    super.connectedCallback();
    document.addEventListener("keydown", this._onDocKeydown);
    document.addEventListener("click", this._onDocClick, true);
    void this._loadCurrentSession();
    window.addEventListener("ship:session:changed", this._onSessionChanged);
    const shell = document.querySelector("ship-shell");
    shell?.addEventListener("ship:bar:open", this._onBarOpen as EventListener);
    shell?.addEventListener("ship:create:open", this._onCreateOpen);
  }

  disconnectedCallback(): void {
    super.disconnectedCallback();
    document.removeEventListener("keydown", this._onDocKeydown);
    document.removeEventListener("click", this._onDocClick, true);
    window.removeEventListener("ship:session:changed", this._onSessionChanged);
    const shell = document.querySelector("ship-shell");
    shell?.removeEventListener("ship:bar:open", this._onBarOpen as EventListener);
    shell?.removeEventListener("ship:create:open", this._onCreateOpen);
  }

  private _onCreateOpen = (): void => {
    void this.focus("create");
  };

  private _onSessionChanged = (): void => {
    void this._loadCurrentSession();
  };

  private _onBarOpen = (e: CustomEvent<{ mode?: BarMode; prefill?: string }>): void => {
    if (e.detail?.mode) this._mode = e.detail.mode;
    if (typeof e.detail?.prefill === "string") this._value = e.detail.prefill;
    this.focus();
  };

  /** Focus the bar. Seeds the default mode when the input is empty —
   *  ⌘K opens in `find` (nav/command palette), Space+Space opens in
   *  `ai` (chat/spawn). Typing a prefix always overrides. */
  async focus(mode: BarMode = "find"): Promise<void> {
    this.open = true;
    if (this._value.trim().length === 0) {
      this._mode = mode;
    }
    await this.updateComplete;
    const input = this.renderRoot.querySelector("input");
    input?.focus();
  }

  blur(): void {
    this.open = false;
    const input = this.renderRoot.querySelector("input");
    input?.blur();
  }

  private async _loadCurrentSession(): Promise<void> {
    try {
      const resp = await fetch("/api/pty-sessions", {
        credentials: "same-origin",
      });
      if (resp.ok) {
        const body = (await resp.json()) as { sessions: SessionSummary[] };
        this._allSessions = body.sessions;
        const active = body.sessions.find(
          (s) => s.status !== "ended" && s.agent_id,
        );
        if (active && active.id !== this._session?.id) {
          this._session = active;
          this._thread = [];
          this._attachStream(active.id);
        } else if (!active) {
          this._session = null;
          this._detachStream();
        }
      }
    } catch {
      /* leave null */
    }
    if (this._agents.length === 0) void this._loadAgents();
    if (this._providers.length === 0) void this._loadProviders();
  }

  private _switchTo(session: SessionSummary): void {
    if (session.id === this._session?.id) {
      this._sessionPickerOpen = false;
      return;
    }
    this._session = session;
    this._thread = [];
    this._attachStream(session.id);
    this._sessionPickerOpen = false;
    this._mode = "ai";
  }

  private async _loadAgents(): Promise<void> {
    try {
      const resp = await fetch("/api/agents", { credentials: "same-origin" });
      if (!resp.ok) return;
      const body = (await resp.json()) as {
        items?: Array<{ id: string; scope?: string }>;
      };
      const list: AgentSummary[] = (body.items ?? []).map((a) => ({ id: a.id }));
      this._agents = list;
      if (!this._spawnAgent && list.length > 0) {
        const ship = list.find((a) => a.id === "shipwright");
        this._spawnAgent = (ship ?? list[0]).id;
      }
    } catch {
      /* empty */
    }
  }

  private async _loadProviders(): Promise<void> {
    try {
      const resp = await fetch("/api/providers/installed", {
        credentials: "same-origin",
      });
      if (!resp.ok) return;
      const body = (await resp.json()) as { providers: string[] };
      this._providers = body.providers;
      if (!this._spawnProvider && body.providers.length > 0) {
        this._spawnProvider = body.providers[0];
      }
    } catch {
      /* empty */
    }
  }

  private _attachStream(sessionId: string): void {
    this._detachStream();
    const ac = new AbortController();
    this._streamAbort = ac;
    const es = new EventSource(`/api/sessions/${sessionId}/events`, {
      withCredentials: true,
    });
    es.addEventListener("provider", (ev) => {
      const payload = (ev as MessageEvent).data;
      let text = payload;
      try {
        const parsed = JSON.parse(payload);
        text = parsed.kind ?? payload;
      } catch { /* raw */ }
      this._thread = [
        ...this._thread,
        {
          id: String(Date.now() + Math.random()),
          sender: "agent",
          text: String(text),
        },
      ];
    });
    es.onerror = () => {
      if (ac.signal.aborted) return;
      es.close();
    };
    ac.signal.addEventListener("abort", () => es.close());
  }

  private _detachStream(): void {
    this._streamAbort?.abort();
    this._streamAbort = null;
  }

  private async _startSession(): Promise<void> {
    if (!this._spawnAgent || !this._spawnProvider) {
      this._spawnError = "Pick an agent and a provider first.";
      return;
    }
    this._spawning = true;
    this._spawnError = null;
    try {
      const resp = await fetch("/api/sessions/spawn", {
        method: "POST",
        credentials: "same-origin",
        headers: { "Content-Type": "application/json" },
        body: JSON.stringify({
          workspace_id: this._workspaceId,
          kind: "agent",
          agent_id: this._spawnAgent,
          provider: this._spawnProvider,
        }),
      });
      if (!resp.ok) {
        const text = await resp.text().catch(() => "");
        this._spawnError = `spawn failed: ${resp.status} ${text}`;
        return;
      }
      await this._loadCurrentSession();
      window.dispatchEvent(new CustomEvent("ship:session:changed"));
    } catch (e) {
      this._spawnError = e instanceof Error ? e.message : String(e);
    } finally {
      this._spawning = false;
    }
  }

  private _detectMode(value: string, fallback: BarMode): BarMode {
    if (value.startsWith("/")) return "command";
    if (value.startsWith(">")) return "find";
    if (value.startsWith("@")) return "mention";
    if (value.startsWith("+")) return "create";
    if (value.trim().length === 0) return fallback;
    // Default for typed text: switch to AI mode so the user's message
    // lands in the session. `>` prefix is the way to stay in find.
    return "ai";
  }

  private _onInput = (e: Event): void => {
    this._value = (e.target as HTMLInputElement).value;
    this._mode = this._detectMode(this._value, this._mode);
    this.open = true;
  };

  private _onFocus = (): void => {
    this.open = true;
  };

  private _onKeydown = (e: KeyboardEvent): void => {
    if (e.key === "Enter" && !e.shiftKey) {
      e.preventDefault();
      void this._submit();
    } else if (e.key === "Escape") {
      this.blur();
    }
  };

  private async _submit(): Promise<void> {
    const text = this._value.trim();
    if (!text) return;
    if (this._mode === "command") {
      const needle = text.replace(/^\/+\s*/, "").toLowerCase();
      const match = this._slashCommands().find(
        (c) =>
          c.key === needle ||
          c.key.startsWith(needle) ||
          c.label.toLowerCase() === "/" + needle,
      );
      if (match) {
        this._value = "";
        await match.run();
      } else {
        window.ship?.notify?.(`Unknown command: ${text}`, "warning");
      }
      return;
    }
    if (this._mode === "find") {
      const needle = text.replace(/^>+\s*/, "").toLowerCase();
      const match = this._navDestinations().find(
        (d) =>
          d.label.toLowerCase().includes(needle) ||
          d.keywords.toLowerCase().includes(needle) ||
          d.path.toLowerCase().includes(needle),
      );
      if (match) {
        window.ship?.navigate(match.path);
        this._value = "";
        this.blur();
      }
      return;
    }
    if (this._mode === "ai") {
      if (!this._session) {
        // No session — nudge the spawn card. Don't swallow the text
        // silently; keep it in the input so the user can retry after
        // starting a session.
        this._spawnError = "Start a session below first.";
        return;
      }
      try {
        await fetch(`/api/sessions/${this._session.id}/input`, {
          method: "POST",
          credentials: "same-origin",
          headers: { "Content-Type": "application/json" },
          body: JSON.stringify({ data: text + "\n" }),
        });
        this._thread = [
          ...this._thread,
          {
            id: String(Date.now()),
            sender: "user",
            text,
          },
        ];
      } catch (e) {
        this._thread = [
          ...this._thread,
          {
            id: String(Date.now()),
            sender: "system",
            text: `send failed: ${e instanceof Error ? e.message : String(e)}`,
          },
        ];
      }
      this._value = "";
      return;
    }
    // Other modes land in subsequent commits.
    this.dispatchEvent(
      new CustomEvent("ship:bar:submit", {
        detail: { mode: this._mode, value: text },
        bubbles: true,
        composed: true,
      }),
    );
    this._value = "";
    this._mode = "ai";
  }

  private _createDef(): CreateSurfaceDef {
    return CREATE_SURFACES.find((s) => s.id === this._createSurface)!;
  }

  private _createNameValid(): boolean {
    return KEBAB_RE.test(this._createName);
  }

  private _pickCreateSurface(id: CreateSurface): void {
    this._createSurface = id;
    const def = this._createDef();
    if (!def.shapes.includes(this._createShape)) {
      this._createShape = def.shapes[0];
    }
  }

  private async _submitCreate(): Promise<void> {
    if (!this._createNameValid()) {
      this._createError = "Name must be kebab-case.";
      return;
    }
    this._createSubmitting = true;
    this._createError = null;
    try {
      const resp = await fetch(
        `/api/create/${encodeURIComponent(this._createSurface)}`,
        {
          method: "POST",
          credentials: "same-origin",
          headers: { "Content-Type": "application/json" },
          body: JSON.stringify({
            name: this._createName,
            purpose: this._createPurpose,
            shape: this._createShape,
          }),
        },
      );
      if (!resp.ok) {
        const text = await resp.text().catch(() => "");
        this._createError = `create failed: ${resp.status} ${text}`;
        return;
      }
      const body = (await resp.json()) as
        | { ok?: boolean; data?: { route?: string }; error?: string }
        | Record<string, unknown>;
      if ((body as { ok?: boolean }).ok === false) {
        this._createError = (body as { error?: string }).error ?? "create failed";
        return;
      }
      const route = (body as { data?: { route?: string } }).data?.route;
      window.ship?.notify?.(`${this._createName} created`, "success");
      this.closest("ship-shell")?.dispatchEvent(
        new CustomEvent("ship:create:submitted", {
          detail: {
            surface: this._createSurface,
            name: this._createName,
            route,
          },
          bubbles: true,
        }),
      );
      if (route) window.ship?.navigate(route);
      this._resetCreate();
      this.blur();
    } catch (e) {
      this._createError = e instanceof Error ? e.message : String(e);
    } finally {
      this._createSubmitting = false;
    }
  }

  private _resetCreate(): void {
    this._createName = "";
    this._createPurpose = "";
    this._createShape = "list";
    this._createSurface = "view";
  }

  private _openSessionPicker(): void {
    this._sessionPickerOpen = !this._sessionPickerOpen;
    if (this._sessionPickerOpen) {
      void this._loadCurrentSession();
    }
  }

  private async _endSession(id: string): Promise<void> {
    try {
      await fetch(`/api/sessions/${id}`, {
        method: "DELETE",
        credentials: "same-origin",
      });
    } catch {
      /* best effort */
    }
    if (this._session?.id === id) {
      this._session = null;
      this._detachStream();
    }
    await this._loadCurrentSession();
  }

  private _modeGlyph(): { text: string; tone: string; placeholder: string } {
    switch (this._mode) {
      case "command":
        return { text: "/", tone: "", placeholder: "Run a Ship action…" };
      case "find":
        return { text: ">", tone: "", placeholder: "Jump to a view…" };
      case "mention":
        return { text: "@", tone: "", placeholder: "Reference an agent, view, file…" };
      case "create":
        return { text: "+", tone: "", placeholder: "Create a view, skill, agent…" };
      case "ai":
      default:
        return { text: "ai", tone: "ai", placeholder: "Ask Ship or type / for actions" };
    }
  }

  render() {
    const statusClass =
      this._session?.status === "thinking"
        ? "thinking"
        : this._session
          ? "active"
          : "";
    return html`
      ${this._renderTrigger(statusClass)}
      ${this.open ? this._renderModal(statusClass) : nothing}
    `;
  }

  private _renderTrigger(statusClass: string) {
    const label = this._session
      ? `Ask ${this._session.agent_id} or jump to…`
      : "Ask Ship or jump to…";
    return html`
      <button
        class="trigger"
        type="button"
        aria-label="Open command bar"
        @click=${() => void this.focus()}
      >
        <ship-icon name="search" size="sm"></ship-icon>
        <span class="label">${label}</span>
        <span class="dot ${statusClass}"
              title=${this._session
                ? `${this._session.agent_id ?? "—"} · ${this._session.provider ?? "—"}`
                : "No session attached"}></span>
        <kbd>⌘K</kbd>
      </button>
    `;
  }

  private _renderModal(statusClass: string) {
    const { text: glyph, tone, placeholder } = this._modeGlyph();
    return html`
      <div class="backdrop" @click=${() => this.blur()}></div>
      <div class="panel" role="dialog" aria-label="Ship command bar">
        <div class="panel-input">
          <span class="mode-glyph ${tone}">${glyph}</span>
          <input
            type="text"
            .value=${this._value}
            placeholder=${placeholder}
            @input=${this._onInput}
            @focus=${this._onFocus}
            @keydown=${this._onKeydown}
            aria-label="Ship command bar"
          />
          <kbd>esc</kbd>
          <div class="panel-chip-wrap">
            <span
              class="panel-chip"
              title=${this._session
                ? `${this._session.agent_id ?? "—"} · ${this._session.provider ?? "—"}`
                : "No session attached"}
              @click=${(e: MouseEvent) => {
                e.stopPropagation();
                this._openSessionPicker();
              }}
            >
              <span class="dot ${statusClass}"></span>
              <span>${this._session?.agent_id ?? "No session"}</span>
            </span>
            ${this._sessionPickerOpen ? this._renderSessionPopover() : nothing}
          </div>
        </div>
        <div class="panel-body">
          ${this._renderFlyoutBody()}
        </div>
      </div>
    `;
  }

  private _renderFlyoutBody() {
    switch (this._mode) {
      case "command":
        return this._renderCommandList();
      case "find":
        return this._renderFindList();
      case "mention":
        return html`<div class="empty-msg">Mentions coming next commit.</div>`;
      case "create":
        return this._renderCreateForm();
      case "ai":
      default:
        return this._session ? this._renderThread() : this._renderSpawnCard();
    }
  }

  /** Built-in navigation destinations. The legacy command palette
   * kept these 11 — we surface them in the Find mode now. */
  private _navDestinations(): Array<{ label: string; path: string; icon: string; keywords: string }> {
    return [
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
      { label: "Chats (Mission Control)", path: "/chats", icon: "messages-square", keywords: "sessions active" },
    ];
  }

  private _renderSessionPopover() {
    const active = this._allSessions.filter(
      (s) => s.status !== "ended" && s.agent_id,
    );
    return html`
      <div
        class="session-popover"
        @click=${(e: MouseEvent) => e.stopPropagation()}
      >
        ${active.length === 0
          ? html`<div class="session-row">
              <span></span>
              <span class="meta">No active sessions</span>
              <span></span>
            </div>`
          : active.map(
              (s) => html`
                <div
                  class="session-row ${s.id === this._session?.id ? "current" : ""}"
                  @click=${() => this._switchTo(s)}
                >
                  <span class="dot" style="background: ${
                    s.status === "thinking"
                      ? "var(--warning, #d97706)"
                      : "var(--success, #16a34a)"
                  }"></span>
                  <span>
                    <strong>${s.agent_id}</strong>
                    <span class="meta">${s.provider} · ${s.workspace_id}</span>
                  </span>
                  <button
                    class="end"
                    title="End session"
                    @click=${(e: MouseEvent) => {
                      e.stopPropagation();
                      void this._endSession(s.id);
                    }}
                  >
                    ×
                  </button>
                </div>
              `,
            )}
        <div class="session-sep"></div>
        <div
          class="session-start-new"
          @click=${() => {
            this._sessionPickerOpen = false;
            this._session = null;
            this._mode = "ai";
          }}
        >
          <ship-icon name="plus" size="sm"></ship-icon>
          <span>Start new session</span>
        </div>
        <div
          class="session-start-new"
          @click=${() => {
            this._sessionPickerOpen = false;
            window.ship?.navigate("/chats");
            this.blur();
          }}
        >
          <ship-icon name="layout-grid" size="sm"></ship-icon>
          <span>View all chats</span>
        </div>
      </div>
    `;
  }

  /** Slash-command registry. Keep small and discoverable; extend as
   *  actions earn their spot. Each command has an explicit handler so
   *  invariants are tight. */
  private _slashCommands(): Array<{
    key: string;
    label: string;
    hint: string;
    run: () => void | Promise<void>;
  }> {
    return [
      {
        key: "new",
        label: "/new",
        hint: "Scaffold a view, skill, agent, …",
        run: () => {
          this._mode = "create";
          this._value = "";
        },
      },
      {
        key: "chats",
        label: "/chats",
        hint: "Open Mission Control",
        run: () => {
          window.ship?.navigate("/chats");
          this.blur();
        },
      },
      {
        key: "settings",
        label: "/settings",
        hint: "Jump to settings",
        run: () => {
          window.ship?.navigate("/settings");
          this.blur();
        },
      },
      {
        key: "lint",
        label: "/lint",
        hint: "Run lint on the current view",
        run: async () => {
          const current = window.ship?.current?.();
          const name =
            current?.path?.replace(/^\/+/, "").split("/")[0] ?? "";
          if (!name) {
            window.ship?.notify?.("No view in context.", "warning");
            return;
          }
          try {
            const resp = await fetch("/api/lint/view", {
              method: "POST",
              credentials: "same-origin",
              headers: { "Content-Type": "application/json" },
              body: JSON.stringify({ name }),
            });
            if (resp.ok) {
              const body = (await resp.json()) as {
                data?: { findings?: unknown[] };
              };
              const count = body.data?.findings?.length ?? 0;
              window.ship?.notify?.(
                count === 0
                  ? `${name}: no lint findings`
                  : `${name}: ${count} lint findings`,
                count === 0 ? "success" : "warning",
              );
            } else {
              window.ship?.notify?.(`lint failed: ${resp.status}`, "destructive");
            }
          } catch (e) {
            window.ship?.notify?.(String(e), "destructive");
          }
          this.blur();
        },
      },
      {
        key: "end",
        label: "/end",
        hint: "End the current session",
        run: async () => {
          if (!this._session) return;
          await this._endSession(this._session.id);
          this.blur();
        },
      },
      {
        key: "close",
        label: "/close",
        hint: "Close the command bar",
        run: () => this.blur(),
      },
    ];
  }

  private _renderCommandList() {
    const needle = this._value.replace(/^\/+\s*/, "").toLowerCase().trim();
    const items = this._slashCommands().filter((c) => {
      if (!needle) return true;
      return (
        c.key.includes(needle) ||
        c.label.toLowerCase().includes(needle) ||
        c.hint.toLowerCase().includes(needle)
      );
    });
    if (items.length === 0) {
      return html`<div class="empty-msg">No commands match.</div>`;
    }
    return html`
      <div class="mode-hint">Actions</div>
      <div class="nav-list">
        ${items.map(
          (c) => html`
            <div
              class="nav-item"
              role="option"
              @click=${async () => {
                this._value = "";
                await c.run();
              }}
            >
              <span style="font-family: var(--font-mono)">${c.label}</span>
              <span></span>
              <span class="hint">${c.hint}</span>
            </div>
          `,
        )}
      </div>
    `;
  }

  private _renderCreateForm() {
    const def = this._createDef();
    return html`
      <div class="mode-hint">Create</div>
      <div class="create-surfaces">
        ${CREATE_SURFACES.map(
          (s) => html`
            <button
              class="create-surface"
              aria-pressed=${this._createSurface === s.id ? "true" : "false"}
              @click=${() => this._pickCreateSurface(s.id)}
            >
              <ship-icon name=${s.icon}></ship-icon>
              <span>${s.label}</span>
            </button>
          `,
        )}
      </div>
      <div class="create-grid">
        <label>
          <span>Name (kebab-case)</span>
          <input
            type="text"
            placeholder="my-${def.id}"
            .value=${this._createName}
            @input=${(e: Event) =>
              (this._createName = (e.target as HTMLInputElement).value)}
            @keydown=${(e: KeyboardEvent) => {
              if (e.key === "Enter" && this._createNameValid()) {
                e.preventDefault();
                void this._submitCreate();
              }
            }}
          />
        </label>
        <label>
          <span>${def.shapeLabel}</span>
          <select
            .value=${this._createShape}
            @change=${(e: Event) =>
              (this._createShape = (e.target as HTMLSelectElement).value)}
          >
            ${def.shapes.map(
              (s) => html`<option value=${s}>${s}</option>`,
            )}
          </select>
        </label>
        <label class="span-2">
          <span>Purpose (optional)</span>
          <input
            type="text"
            placeholder="One-line description"
            .value=${this._createPurpose}
            @input=${(e: Event) =>
              (this._createPurpose = (e.target as HTMLInputElement).value)}
          />
        </label>
      </div>
      ${this._createError
        ? html`<div class="create-error">${this._createError}</div>`
        : nothing}
      <div class="create-actions">
        <button
          class="create-submit"
          ?disabled=${this._createSubmitting || !this._createNameValid()}
          @click=${() => void this._submitCreate()}
        >
          ${this._createSubmitting ? "Creating…" : `Create ${def.label}`}
        </button>
      </div>
    `;
  }

  private _renderFindList() {
    const needle = this._value.replace(/^>+\s*/, "").toLowerCase().trim();
    const items = this._navDestinations().filter((d) => {
      if (!needle) return true;
      return (
        d.label.toLowerCase().includes(needle) ||
        d.keywords.toLowerCase().includes(needle) ||
        d.path.toLowerCase().includes(needle)
      );
    });
    if (items.length === 0) {
      return html`<div class="empty-msg">No matches.</div>`;
    }
    return html`
      <div class="mode-hint">Jump to</div>
      <div class="nav-list">
        ${items.map(
          (d) => html`
            <div
              class="nav-item"
              role="option"
              @click=${() => {
                window.ship?.navigate(d.path);
                this.blur();
              }}
            >
              <ship-icon name=${d.icon} size="sm"></ship-icon>
              <span>${d.label}</span>
              <span class="hint">${d.path}</span>
            </div>
          `,
        )}
      </div>
    `;
  }

  private _renderSpawnCard() {
    return html`
      <div class="spawn-card">
        <div class="spawn-title">Start a session</div>
        <div class="spawn-row">
          <label>Agent</label>
          <select
            .value=${this._spawnAgent}
            @change=${(e: Event) =>
              (this._spawnAgent = (e.target as HTMLSelectElement).value)}
          >
            ${this._agents.length === 0
              ? html`<option value="">No agents found</option>`
              : this._agents.map(
                  (a) => html`
                    <option value=${a.id}>${a.name ?? a.label ?? a.id}</option>
                  `,
                )}
          </select>
        </div>
        <div class="spawn-row">
          <label>Provider</label>
          <select
            .value=${this._spawnProvider}
            @change=${(e: Event) =>
              (this._spawnProvider = (e.target as HTMLSelectElement).value)}
          >
            ${this._providers.length === 0
              ? html`<option value="">No providers installed</option>`
              : this._providers.map(
                  (p) => html`<option value=${p}>${p}</option>`,
                )}
          </select>
        </div>
        ${this._spawnError
          ? html`<div class="spawn-error">${this._spawnError}</div>`
          : nothing}
        <button
          class="spawn-start"
          ?disabled=${this._spawning ||
          !this._spawnAgent ||
          !this._spawnProvider}
          @click=${() => void this._startSession()}
        >
          ${this._spawning ? "Starting…" : "Start session"}
        </button>
      </div>
    `;
  }

  private _renderThread() {
    return html`
      <div class="thread">
        ${this._thread.length === 0
          ? html`
              <div class="thread-empty">
                Type above and press Enter. The agent's replies land here.
              </div>
            `
          : this._thread.map(
              (m) => html`
                <div class="turn" data-sender=${m.sender}>
                  <span class="sender">${m.sender}</span>
                  <span class="text">${m.text}</span>
                </div>
              `,
            )}
      </div>
    `;
  }
}

declare global {
  interface HTMLElementTagNameMap {
    "ship-command-bar": ShipCommandBar;
  }
}
