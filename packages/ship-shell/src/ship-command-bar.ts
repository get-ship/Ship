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

interface SessionSummary {
  id: string;
  workspace_id: string;
  agent_id?: string;
  provider?: string;
  status: string;
}

@customElement("ship-command-bar")
export class ShipCommandBar extends LitElement {
  static styles = css`
    :host {
      display: inline-flex;
      align-items: center;
      gap: 6px;
      flex: 1;
      max-width: 640px;
      min-width: 220px;
      position: relative;
      font: inherit;
    }
    .shell {
      display: flex;
      align-items: center;
      gap: 8px;
      padding: 3px 10px;
      width: 100%;
      border: 1px solid var(--border);
      border-radius: var(--radius-md, 6px);
      background: var(--background);
      color: var(--muted-foreground);
      transition: border-color var(--duration-fast, 120ms) var(--ease-out, ease);
    }
    .shell:focus-within {
      border-color: color-mix(in oklch, var(--primary) 60%, var(--border));
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
      width: 14px;
      text-align: center;
      font-family: var(--font-mono);
      font-size: 11px;
    }
    .mode-glyph.ai {
      color: var(--primary);
    }
    .chip {
      display: inline-flex;
      align-items: center;
      gap: 6px;
      padding: 2px 8px;
      border-left: 1px solid var(--border);
      color: var(--foreground);
      font-size: var(--text-xs);
      cursor: pointer;
    }
    .chip .dot {
      width: 6px;
      height: 6px;
      border-radius: 999px;
      background: var(--muted-foreground);
    }
    .chip .dot.active {
      background: var(--success, #16a34a);
    }
    .chip .dot.thinking {
      background: var(--warning, #d97706);
    }
    .flyout {
      position: absolute;
      top: calc(100% + 4px);
      left: 0;
      right: 0;
      background: var(--card, var(--background));
      border: 1px solid var(--border);
      border-radius: var(--radius-md, 6px);
      box-shadow: 0 12px 28px rgba(0, 0, 0, 0.12);
      min-height: 120px;
      max-height: min(70vh, 560px);
      overflow: auto;
      z-index: 9800;
      padding: var(--space-3, 12px);
      color: var(--foreground);
    }
    .flyout.empty {
      color: var(--muted-foreground);
      font-size: var(--text-sm);
    }
    .mode-hint {
      font-size: 10px;
      text-transform: uppercase;
      letter-spacing: 0.08em;
      color: var(--muted-foreground);
      margin-bottom: var(--space-2, 8px);
    }
  `;

  /** Controlled open state. The shell can imperatively open/close via
   * `focus()` / `blur()` or by setting this property. */
  @property({ type: Boolean, reflect: true })
  open = false;

  @state() private _value = "";
  @state() private _mode: BarMode = "ai";
  @state() private _session: SessionSummary | null = null;

  private _onDocKeydown = (e: KeyboardEvent): void => {
    const meta = e.metaKey || e.ctrlKey;
    if (meta && e.key.toLowerCase() === "k") {
      e.preventDefault();
      this.focus();
    } else if (e.key === "Escape" && this.open) {
      this.blur();
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
    document
      .querySelector("ship-shell")
      ?.addEventListener("ship:bar:open", this._onBarOpen as EventListener);
  }

  disconnectedCallback(): void {
    super.disconnectedCallback();
    document.removeEventListener("keydown", this._onDocKeydown);
    document.removeEventListener("click", this._onDocClick, true);
    window.removeEventListener("ship:session:changed", this._onSessionChanged);
    document
      .querySelector("ship-shell")
      ?.removeEventListener("ship:bar:open", this._onBarOpen as EventListener);
  }

  private _onSessionChanged = (): void => {
    void this._loadCurrentSession();
  };

  private _onBarOpen = (e: CustomEvent<{ mode?: BarMode; prefill?: string }>): void => {
    if (e.detail?.mode) this._mode = e.detail.mode;
    if (typeof e.detail?.prefill === "string") this._value = e.detail.prefill;
    this.focus();
  };

  async focus(): Promise<void> {
    this.open = true;
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
      if (!resp.ok) return;
      const body = (await resp.json()) as {
        sessions: SessionSummary[];
      };
      this._session = body.sessions.find((s) => s.status !== "ended") ?? null;
    } catch {
      /* fall back to no session */
    }
  }

  private _detectMode(value: string): BarMode {
    if (value.startsWith("/")) return "command";
    if (value.startsWith(">")) return "find";
    if (value.startsWith("@")) return "mention";
    return "ai";
  }

  private _onInput = (e: Event): void => {
    this._value = (e.target as HTMLInputElement).value;
    this._mode = this._detectMode(this._value);
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
    // Placeholder — subsequent commit wires each mode to its action.
    // AI goes to the current session's input; command to a slash
    // dispatcher; find to navigate; mention to inject context.
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

  private _openSessionPicker(): void {
    this.dispatchEvent(
      new CustomEvent("ship:bar:session-picker", {
        bubbles: true,
        composed: true,
      }),
    );
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
    const { text: glyph, tone, placeholder } = this._modeGlyph();
    const statusClass =
      this._session?.status === "thinking"
        ? "thinking"
        : this._session
          ? "active"
          : "";
    return html`
      <div class="shell" role="combobox" aria-expanded=${this.open ? "true" : "false"}>
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
        ${this._value.length === 0 ? html`<kbd>⌘K</kbd>` : nothing}
        <span
          class="chip"
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
      </div>
      ${this.open ? this._renderFlyout() : nothing}
    `;
  }

  private _renderFlyout() {
    return html`
      <div class="flyout">
        <div class="mode-hint">${this._mode}</div>
        ${this._renderFlyoutBody()}
      </div>
    `;
  }

  private _renderFlyoutBody() {
    switch (this._mode) {
      case "command":
        return html`<div class="flyout empty">Slash commands coming next commit.</div>`;
      case "find":
        return html`<div class="flyout empty">Fuzzy find coming next commit.</div>`;
      case "mention":
        return html`<div class="flyout empty">Mentions coming next commit.</div>`;
      case "create":
        return html`<div class="flyout empty">Create coming next commit.</div>`;
      case "ai":
      default:
        if (!this._session) {
          return html`
            <div class="flyout empty">
              <strong>No session attached.</strong>
              Start one from the session chip on the right.
            </div>
          `;
        }
        return html`
          <div class="flyout empty">
            Session thread view coming next commit. Session:
            <code>${this._session.id}</code> · agent
            <code>${this._session.agent_id ?? "—"}</code> ·
            provider <code>${this._session.provider ?? "—"}</code>
          </div>
        `;
    }
  }
}

declare global {
  interface HTMLElementTagNameMap {
    "ship-command-bar": ShipCommandBar;
  }
}
