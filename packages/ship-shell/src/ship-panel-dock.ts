//! `<ship-panel-dock>` — bottom chrome panel with a tab strip.
//!
//! Hosts the terminal plus shell devtools (Source, Signals, Lint,
//! Schema). Each tab is a named slot; host fills it with a body
//! element. Only the active slot is visible; inactive slots stay
//! mounted in the DOM so their state (e.g. xterm.js) persists across
//! tab switches.
//!
//! Attributes:
//!   active-tab — tab name. Defaults to the first declared tab.
//!
//! Events:
//!   ship:dock:activate — detail `{ tab }` fired on tab change.
//!   ship:dock:close    — fired when the close button is clicked.
//!
//! Slots:
//!   terminal, source, signals, lint, schema — one per tab body.
//!
//! Visibility is owned by the host via a body class (e.g. `dock-open`);
//! the dock itself never hides — it renders what it's told to render.

import { LitElement, css, html } from "lit";
import { customElement, property, state } from "lit/decorators.js";

export type DockTab =
  | "terminal"
  | "source"
  | "signals"
  | "lint"
  | "schema";

interface TabDef {
  name: DockTab;
  label: string;
  icon: string;
}

const TABS: TabDef[] = [
  { name: "terminal", label: "Terminal", icon: "terminal" },
  { name: "source", label: "Source", icon: "file-code" },
  { name: "signals", label: "Signals", icon: "activity" },
  { name: "lint", label: "Lint", icon: "alert-triangle" },
  { name: "schema", label: "Schema", icon: "braces" },
];

@customElement("ship-panel-dock")
export class ShipPanelDock extends LitElement {
  static styles = css`
    :host {
      display: flex;
      flex-direction: column;
      height: 100%;
      min-height: 0;
    }
    .tabstrip {
      display: flex;
      align-items: center;
      gap: 2px;
      padding: 2px var(--space-2, 8px);
      border-bottom: 1px solid var(--border);
      background: var(--sidebar);
      font-size: var(--text-xs);
      user-select: none;
    }
    .tab {
      appearance: none;
      background: transparent;
      border: 0;
      color: var(--muted-foreground);
      font: inherit;
      padding: 4px 10px;
      border-radius: var(--radius-sm, 4px);
      cursor: pointer;
      display: inline-flex;
      align-items: center;
      gap: 6px;
      height: 24px;
      transition: background var(--duration-fast, 120ms) var(--ease-out, ease);
    }
    .tab:hover {
      background: var(--sidebar-accent);
      color: var(--foreground);
    }
    .tab[aria-selected="true"] {
      color: var(--foreground);
      background: var(--background);
      font-weight: var(--font-weight-medium, 500);
    }
    .spacer {
      flex: 1;
    }
    .close {
      appearance: none;
      background: transparent;
      border: 0;
      color: var(--muted-foreground);
      cursor: pointer;
      padding: 4px;
      border-radius: var(--radius-sm, 4px);
      display: inline-flex;
      align-items: center;
      justify-content: center;
    }
    .close:hover {
      background: var(--sidebar-accent);
      color: var(--foreground);
    }
    .bodies {
      flex: 1;
      min-height: 0;
      position: relative;
    }
    ::slotted(*) {
      display: none;
      height: 100%;
    }
    slot[data-active]::slotted(*) {
      display: block;
    }
  `;

  @property({ type: String, attribute: "active-tab", reflect: true })
  activeTab: DockTab = "terminal";

  @state()
  private _tabs = TABS;

  render() {
    return html`
      <div class="tabstrip" role="tablist">
        ${this._tabs.map(
          (t) => html`
            <button
              class="tab"
              role="tab"
              aria-selected=${this.activeTab === t.name ? "true" : "false"}
              data-tab=${t.name}
              @click=${() => this._activate(t.name)}
            >
              <ship-icon name=${t.icon} size="sm"></ship-icon>
              <span>${t.label}</span>
            </button>
          `,
        )}
        <span class="spacer"></span>
        <button
          class="close"
          aria-label="Close panel"
          @click=${this._close}
        >
          <ship-icon name="x" size="sm"></ship-icon>
        </button>
      </div>
      <div class="bodies">
        ${this._tabs.map(
          (t) => html`
            <slot
              name=${t.name}
              ?data-active=${this.activeTab === t.name}
            ></slot>
          `,
        )}
      </div>
    `;
  }

  private _activate(tab: DockTab): void {
    if (this.activeTab === tab) return;
    this.activeTab = tab;
    this.dispatchEvent(
      new CustomEvent("ship:dock:activate", {
        detail: { tab },
        bubbles: true,
        composed: true,
      }),
    );
  }

  private _close = (): void => {
    this.dispatchEvent(
      new CustomEvent("ship:dock:close", { bubbles: true, composed: true }),
    );
  };

  /** Imperative tab activation (used by keyboard shortcuts in the host). */
  activate(tab: DockTab): void {
    this._activate(tab);
  }
}

declare global {
  interface HTMLElementTagNameMap {
    "ship-panel-dock": ShipPanelDock;
  }
}
