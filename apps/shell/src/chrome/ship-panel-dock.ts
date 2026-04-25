//! `<ship-panel-dock>` — bottom chrome panel with a tab strip.
//!
//! Hosts the terminal plus shell devtools (Source, Signals, Lint,
//! Schema, Components). Each tab is a named slot; host fills it with
//! a body element. Inactive slots stay mounted in the DOM so their
//! state (e.g. xterm.js) persists across tab switches.
//!
//! Visual model — a polished IDE-tier panel:
//!
//!   • Resize handle on the top edge (drag to set height, double-click
//!     to reset). Persisted to `localStorage` under the configured key.
//!   • Opinionated tab strip with icon + label + optional live-dot +
//!     optional count badge. Active tab gets a 2px primary inset bar.
//!   • Right-side tools cluster: Clear, Pop-out, Close.
//!   • Per-tab toolbar slot (`toolbar:<id>`) below the tabstrip — pane
//!     authors fill it with their own meta + filter input + actions.
//!
//! Attributes:
//!   active-tab   — tab id. Defaults to first declared tab.
//!   storage-key  — localStorage prefix for height + last-active-tab
//!                  persistence (default `ship-dock`).
//!
//! Properties:
//!   tabs — `DockTabDef[]`. Override the default 6-tab list, e.g. to
//!     supply live counts. Setting this replaces the tab list.
//!
//! Events (CustomEvent, bubbles, composed):
//!   ship:dock:activate — detail `{ tab }` on tab change.
//!   ship:dock:close    — close button clicked.
//!   ship:dock:clear    — clear button clicked. detail `{ tab }`.
//!   ship:dock:popout   — pop-out button clicked. detail `{ tab }`.
//!   ship:dock:resize   — drag finished. detail `{ height }`.
//!
//! Slots:
//!   <id>            — body for tab `<id>` (default: terminal, source,
//!                     signals, lint, schema, components).
//!   toolbar:<id>    — slim toolbar row for tab `<id>`. Optional.
//!
//! Visibility (open/closed) is owned by the host via a body class.

import { LitElement, css, html, nothing } from "lit";
import { customElement, property, state } from "lit/decorators.js";

export interface DockTabDef {
  id: string;
  label: string;
  /** Lucide icon name, fed to `<ship-icon>`. */
  icon: string;
  /** Show a small live indicator dot when this tab is active. */
  live?: boolean;
  /** Numeric badge to the right of the label. */
  count?: number | null;
}

const DEFAULT_TABS: DockTabDef[] = [
  { id: "terminal", label: "Terminal", icon: "terminal", live: true },
  { id: "source", label: "Source", icon: "file-code" },
  { id: "signals", label: "Signals", icon: "radio", live: true },
  { id: "lint", label: "Lint", icon: "list" },
  { id: "schema", label: "Schema", icon: "network" },
  { id: "components", label: "Components", icon: "package" },
];

const DEFAULT_HEIGHT = 240;
const MIN_HEIGHT = 120;

@customElement("ship-panel-dock")
export class ShipPanelDock extends LitElement {
  static styles = css`
    :host {
      display: flex;
      flex-direction: column;
      height: 100%;
      min-height: 0;
      position: relative;
      background: var(--card);
      /* Seat the dock above content with a soft top-edge cast. The shadow
       * lands on the page surface above the dock so the dock reads as a
       * floating panel instead of a flush bottom strip. */
      box-shadow: var(--shadow-edge-top);
    }

    /* Resize handle — top edge, 6px hit area. */
    .resize {
      position: absolute;
      top: -3px;
      left: 0;
      right: 0;
      height: 6px;
      cursor: ns-resize;
      z-index: 2;
      touch-action: none;
    }
    .resize::before {
      content: "";
      position: absolute;
      left: 50%;
      top: 2px;
      transform: translateX(-50%);
      width: 28px;
      height: 2px;
      border-radius: 2px;
      background: var(--border);
      opacity: 0;
      transition: opacity var(--duration-fast) var(--ease-out);
    }
    .resize:hover::before,
    :host([resizing]) .resize::before {
      opacity: 1;
    }

    /* Tab strip. */
    .tabstrip {
      display: flex;
      align-items: stretch;
      gap: 0;
      padding-left: var(--space-2);
      border-bottom: 1px solid var(--border-subtle, var(--border));
      background: var(--card);
      font-size: var(--text-xs);
      user-select: none;
      height: 30px;
      flex: none;
    }
    .tab {
      appearance: none;
      background: transparent;
      border: 0;
      color: var(--muted-foreground);
      font: inherit;
      font-size: 11.5px;
      padding: 0 10px;
      display: inline-flex;
      align-items: center;
      gap: 6px;
      cursor: pointer;
      position: relative;
      border-bottom: 2px solid transparent;
      margin-bottom: -1px;
      transition: color var(--duration-fast) var(--ease-out),
        background var(--duration-fast) var(--ease-out);
    }
    .tab:hover {
      color: var(--foreground);
    }
    .tab[aria-selected="true"] {
      color: var(--foreground);
      background: color-mix(
        in oklab,
        var(--primary) 12%,
        transparent
      );
      border-bottom-color: var(--primary);
      font-weight: var(--font-weight-medium, 500);
    }
    .tab:focus-visible {
      outline: none;
      box-shadow: inset 0 0 0 2px var(--ring);
      border-radius: var(--radius-sm);
    }
    .tab .live-dot {
      width: 6px;
      height: 6px;
      border-radius: 50%;
      background: var(--success, var(--status-green, #22c55e));
      box-shadow: 0 0 0 2px
        color-mix(in oklab, var(--success, #22c55e) 25%, transparent);
    }
    .tab .count {
      font-family: var(--mono-font, var(--font-mono));
      font-size: 9.5px;
      background: var(--muted);
      color: var(--muted-foreground);
      border-radius: var(--radius-sm);
      padding: 1px 5px;
      min-width: 18px;
      text-align: center;
      font-variant-numeric: tabular-nums;
    }
    .tab[aria-selected="true"] .count {
      background: color-mix(in oklab, var(--primary) 14%, transparent);
      color: var(--foreground);
    }

    /* Right-side tools. */
    .tools {
      margin-left: auto;
      display: flex;
      align-items: center;
      gap: 0;
      padding-right: 4px;
    }
    .tool {
      appearance: none;
      background: transparent;
      border: 0;
      color: var(--muted-foreground);
      cursor: pointer;
      width: 24px;
      height: 22px;
      display: inline-flex;
      align-items: center;
      justify-content: center;
      border-radius: var(--radius-sm);
      transition: background var(--duration-fast) var(--ease-out),
        color var(--duration-fast) var(--ease-out);
    }
    .tool:hover {
      background: var(--hover-bg, var(--sidebar-accent));
      color: var(--foreground);
    }
    .tool:focus-visible {
      outline: none;
      box-shadow: 0 0 0 2px var(--ring);
    }

    /* Per-tab toolbar (slot). Hidden when slot has no assignedNodes. */
    .toolbar {
      display: none;
      align-items: center;
      gap: 6px;
      padding: 6px 10px;
      border-bottom: 1px solid var(--border-subtle, var(--border));
      background: color-mix(
        in oklab,
        var(--background) 50%,
        var(--card)
      );
      flex: none;
      min-height: 30px;
    }
    .toolbar.has-content {
      display: flex;
    }

    /* Bodies. */
    .bodies {
      flex: 1;
      min-height: 0;
      position: relative;
      overflow: hidden;
      background: var(--background);
    }
    ::slotted(*) {
      display: none;
      height: 100%;
      overflow: auto;
    }
    slot[data-active]::slotted(*) {
      display: block;
    }

    @media (prefers-reduced-motion: reduce) {
      .tab,
      .tool,
      .resize::before {
        transition: none;
      }
    }
  `;

  /** Active tab id. */
  @property({ type: String, attribute: "active-tab", reflect: true })
  activeTab: string = DEFAULT_TABS[0]!.id;

  /** Tab metadata — id/label/icon/live/count. */
  @property({ attribute: false })
  tabs: DockTabDef[] = DEFAULT_TABS;

  /** localStorage prefix for height + last-active persistence. */
  @property({ type: String, attribute: "storage-key" })
  storageKey: string = "ship-dock";

  /** Reflected during a resize drag so consumers can pause transitions. */
  @property({ type: Boolean, reflect: true })
  resizing = false;

  @state() private _toolbarHas = new Set<string>();

  connectedCallback(): void {
    super.connectedCallback();
    // Restore last-active tab and dock height from localStorage. Height
    // applies via the host body class wrapper (e.g. `body.dock-open`)
    // so we set a CSS custom property the host stylesheet reads.
    const lastActive = this._readStored("active");
    if (lastActive && this.tabs.some((t) => t.id === lastActive)) {
      this.activeTab = lastActive;
    }
    const heightStr = this._readStored("height");
    const height = heightStr ? Number.parseInt(heightStr, 10) : NaN;
    if (Number.isFinite(height)) {
      this._applyHeight(height);
    } else {
      this._applyHeight(DEFAULT_HEIGHT);
    }
  }

  protected firstUpdated(): void {
    // Wire toolbar slot visibility — show only when assigned content.
    // The initial sync is deferred to a microtask so we don't mutate
    // reactive state inside the update lifecycle (which would trigger
    // Lit's "scheduled an update after an update completed" warning).
    const sync = (id: string, slot: HTMLSlotElement) => {
      const has = slot.assignedElements({ flatten: true }).length > 0;
      const present = this._toolbarHas.has(id);
      if (has === present) return;
      const next = new Set(this._toolbarHas);
      if (has) next.add(id);
      else next.delete(id);
      this._toolbarHas = next;
    };
    for (const t of this.tabs) {
      const slot = this.renderRoot.querySelector<HTMLSlotElement>(
        `slot[name="toolbar:${t.id}"]`,
      );
      if (!slot) continue;
      slot.addEventListener("slotchange", () => sync(t.id, slot));
      // Defer initial probe out of the update cycle.
      queueMicrotask(() => sync(t.id, slot));
    }
  }

  render() {
    return html`
      <div
        class="resize"
        role="separator"
        aria-orientation="horizontal"
        aria-label="Resize panel"
        @pointerdown=${this._onResizeDown}
        @dblclick=${this._onResizeReset}
      ></div>
      <div class="tabstrip" role="tablist">
        ${this.tabs.map(
          (t) => html`
            <button
              class="tab"
              role="tab"
              type="button"
              aria-selected=${this.activeTab === t.id ? "true" : "false"}
              tabindex=${this.activeTab === t.id ? "0" : "-1"}
              data-tab=${t.id}
              @click=${() => this._activate(t.id)}
            >
              <ship-icon name=${t.icon} size="sm"></ship-icon>
              <span>${t.label}</span>
              ${t.live && this.activeTab === t.id
                ? html`<span class="live-dot" aria-hidden="true"></span>`
                : nothing}
              ${t.count != null
                ? html`<span class="count"
                    >${t.count.toLocaleString()}</span
                  >`
                : nothing}
            </button>
          `,
        )}
        <span class="tools">
          <button
            class="tool"
            type="button"
            aria-label="Clear"
            title="Clear"
            @click=${this._onClear}
          >
            <ship-icon name="refresh-ccw" size="sm"></ship-icon>
          </button>
          <button
            class="tool"
            type="button"
            aria-label="Pop out"
            title="Pop out"
            @click=${this._onPopout}
          >
            <ship-icon name="maximize-2" size="sm"></ship-icon>
          </button>
          <button
            class="tool"
            type="button"
            aria-label="Close panel"
            title="Close (⌘J)"
            @click=${this._onClose}
          >
            <ship-icon name="x" size="sm"></ship-icon>
          </button>
        </span>
      </div>
      ${this.tabs.map(
        (t) => html`
          <div
            class="toolbar ${this._toolbarHas.has(t.id) &&
            this.activeTab === t.id
              ? "has-content"
              : ""}"
            ?hidden=${this.activeTab !== t.id}
          >
            <slot name="toolbar:${t.id}"></slot>
          </div>
        `,
      )}
      <div class="bodies">
        ${this.tabs.map(
          (t) => html`
            <slot
              name=${t.id}
              ?data-active=${this.activeTab === t.id}
            ></slot>
          `,
        )}
      </div>
    `;
  }

  /** Imperative tab activation (used by keyboard shortcuts in the host). */
  activate(tab: string): void {
    this._activate(tab);
  }

  private _activate(tab: string): void {
    if (!this.tabs.some((t) => t.id === tab)) return;
    if (this.activeTab === tab) return;
    this.activeTab = tab;
    this._writeStored("active", tab);
    this.dispatchEvent(
      new CustomEvent("ship:dock:activate", {
        detail: { tab },
        bubbles: true,
        composed: true,
      }),
    );
  }

  private _onClose = (): void => {
    this.dispatchEvent(
      new CustomEvent("ship:dock:close", { bubbles: true, composed: true }),
    );
  };

  private _onClear = (): void => {
    this.dispatchEvent(
      new CustomEvent("ship:dock:clear", {
        detail: { tab: this.activeTab },
        bubbles: true,
        composed: true,
      }),
    );
  };

  private _onPopout = (): void => {
    this.dispatchEvent(
      new CustomEvent("ship:dock:popout", {
        detail: { tab: this.activeTab },
        bubbles: true,
        composed: true,
      }),
    );
  };

  // ── Resize ────────────────────────────────────────────────────────
  private _onResizeDown = (ev: PointerEvent): void => {
    ev.preventDefault();
    const startY = ev.clientY;
    const startH = this._currentHeight();
    this.resizing = true;
    const target = ev.currentTarget as HTMLElement;
    target.setPointerCapture(ev.pointerId);
    const onMove = (e: PointerEvent) => {
      const max = Math.round(window.innerHeight * 0.5);
      const next = Math.min(
        max,
        Math.max(MIN_HEIGHT, startH + (startY - e.clientY)),
      );
      this._applyHeight(next);
    };
    const onUp = (e: PointerEvent) => {
      this.resizing = false;
      target.releasePointerCapture?.(e.pointerId);
      target.removeEventListener("pointermove", onMove);
      target.removeEventListener("pointerup", onUp);
      target.removeEventListener("pointercancel", onUp);
      const final = this._currentHeight();
      this._writeStored("height", String(final));
      this.dispatchEvent(
        new CustomEvent("ship:dock:resize", {
          detail: { height: final },
          bubbles: true,
          composed: true,
        }),
      );
    };
    target.addEventListener("pointermove", onMove);
    target.addEventListener("pointerup", onUp);
    target.addEventListener("pointercancel", onUp);
  };

  private _onResizeReset = (): void => {
    this._applyHeight(DEFAULT_HEIGHT);
    this._writeStored("height", String(DEFAULT_HEIGHT));
    this.dispatchEvent(
      new CustomEvent("ship:dock:resize", {
        detail: { height: DEFAULT_HEIGHT },
        bubbles: true,
        composed: true,
      }),
    );
  };

  private _applyHeight(px: number): void {
    document.documentElement.style.setProperty(
      "--ship-dock-height",
      `${px}px`,
    );
  }

  private _currentHeight(): number {
    const v = getComputedStyle(document.documentElement).getPropertyValue(
      "--ship-dock-height",
    );
    const n = Number.parseInt(v, 10);
    return Number.isFinite(n) ? n : DEFAULT_HEIGHT;
  }

  // ── Persistence ──────────────────────────────────────────────────
  private _readStored(suffix: string): string | null {
    try {
      return localStorage.getItem(`${this.storageKey}-${suffix}`);
    } catch {
      return null;
    }
  }
  private _writeStored(suffix: string, value: string): void {
    try {
      localStorage.setItem(`${this.storageKey}-${suffix}`, value);
    } catch {
      /* ignore */
    }
  }
}

declare global {
  interface HTMLElementTagNameMap {
    "ship-panel-dock": ShipPanelDock;
  }
}

/** Backward-compat alias for prior union type. */
export type DockTab = string;
