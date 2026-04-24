//! `<ship-launcher>` — popover showing all views, with pin/unpin.
//!
//! Listens for `ship:launcher:open` on `<ship-shell>`. Fetches the
//! view universe from `/api/views` (what `ship.views.list()` returns)
//! and cross-references `view-catalog` for display metadata. Click a
//! view → navigate + close. Pin/unpin updates localStorage in place
//! and dispatches a `ship:rail:pins-changed` event so the rail can
//! refresh.

import { LitElement, css, html, nothing } from "lit";
import { customElement, state } from "lit/decorators.js";

import { fetchPins, pin, unpin } from "./pins";
import { humanize } from "./view-catalog";

interface ApiView {
  name: string;
  type: string;
  source: string;
  rail?: { label: string; icon: string };
}

@customElement("ship-launcher")
export class ShipLauncher extends LitElement {
  static styles = css`
    :host {
      position: fixed;
      inset: auto auto 0 0;
      z-index: 9000;
      display: none;
    }
    :host([open]) {
      display: block;
    }
    .backdrop {
      position: fixed;
      inset: 0;
      background: transparent;
    }
    .panel {
      position: fixed;
      left: 52px;
      bottom: 48px;
      width: 360px;
      max-height: 60vh;
      overflow-y: auto;
      background: var(--popover, var(--background));
      color: var(--popover-foreground, var(--foreground));
      border: 1px solid var(--border);
      border-radius: var(--radius-md);
      box-shadow: 0 18px 40px -12px rgb(0 0 0 / 0.4);
      padding: var(--space-2);
    }
    .head {
      display: flex;
      align-items: center;
      gap: var(--space-2);
      padding: var(--space-2) var(--space-3);
      border-bottom: 1px solid var(--border);
      font-size: var(--text-xs);
      color: var(--muted-foreground);
      text-transform: uppercase;
      letter-spacing: 0.06em;
    }
    .grid {
      display: grid;
      grid-template-columns: repeat(3, 1fr);
      gap: var(--space-1);
      padding: var(--space-2);
    }
    .tile {
      display: flex;
      flex-direction: column;
      align-items: center;
      justify-content: center;
      gap: var(--space-1);
      padding: var(--space-2);
      border-radius: var(--radius-sm);
      cursor: pointer;
      background: transparent;
      border: 1px solid transparent;
      color: var(--foreground);
      text-decoration: none;
      position: relative;
      min-height: 80px;
    }
    .tile:hover {
      background: var(--muted, rgb(0 0 0 / 0.05));
      border-color: var(--border);
    }
    .tile .label {
      font-size: var(--text-xs);
      text-align: center;
      line-height: 1.2;
    }
    .tile .badge {
      position: absolute;
      top: 4px;
      right: 4px;
      width: 18px;
      height: 18px;
      display: inline-flex;
      align-items: center;
      justify-content: center;
      border-radius: 999px;
      background: transparent;
      color: var(--muted-foreground);
      font-size: 10px;
      opacity: 0;
      transition: opacity var(--duration-fast, 120ms);
    }
    .tile:hover .badge {
      opacity: 1;
    }
    .tile[data-pinned="true"] .badge {
      color: var(--primary);
      opacity: 1;
    }
    .badge:hover {
      background: var(--muted);
      color: var(--foreground);
    }
  `;

  @state() private open = false;
  @state() private views: ApiView[] = [];
  @state() private pins: string[] = [];
  private anchor: HTMLElement | null = null;

  private onOpen = async (e: Event): Promise<void> => {
    this.anchor = ((e as CustomEvent).detail?.anchor as HTMLElement) ?? null;
    this.open = true;
    this.setAttribute("open", "");
    const [pins] = await Promise.all([fetchPins(), this.fetchViews()]);
    this.pins = pins;
  };

  private onClose = (): void => {
    this.open = false;
    this.removeAttribute("open");
  };

  private get shell(): HTMLElement | null {
    return this.closest("ship-shell");
  }

  connectedCallback(): void {
    super.connectedCallback();
    this.shell?.addEventListener("ship:launcher:open", this.onOpen);
    this.shell?.addEventListener("ship:launcher:close", this.onClose);
    document.addEventListener("keydown", this.onKey);
  }

  disconnectedCallback(): void {
    super.disconnectedCallback();
    this.shell?.removeEventListener("ship:launcher:open", this.onOpen);
    this.shell?.removeEventListener("ship:launcher:close", this.onClose);
    document.removeEventListener("keydown", this.onKey);
  }

  private onKey = (e: KeyboardEvent): void => {
    if (e.key === "Escape" && this.open) this.onClose();
  };

  private async fetchViews(): Promise<void> {
    try {
      const resp = await fetch("/api/views", { credentials: "same-origin" });
      if (!resp.ok) return;
      const body = (await resp.json()) as { data?: { views?: ApiView[] } };
      this.views = body.data?.views ?? [];
    } catch {
      this.views = [];
    }
  }

  private navigate(name: string): void {
    window.ship?.navigate(`/${name === "dashboard" ? "" : name}`);
    this.onClose();
  }

  private async togglePin(e: MouseEvent, name: string): Promise<void> {
    e.preventDefault();
    e.stopPropagation();
    this.pins = this.pins.includes(name)
      ? await unpin(name, this.pins)
      : await pin(name, this.pins);
    this.shell?.dispatchEvent(
      new CustomEvent("ship:rail:pins-changed", {
        detail: { pins: this.pins },
        bubbles: true,
      }),
    );
  }

  render() {
    if (!this.open) return nothing;
    return html`
      <div class="backdrop" @click=${() => this.onClose()}></div>
      <div
        class="panel"
        role="dialog"
        aria-label="App launcher"
        @click=${(e: Event) => e.stopPropagation()}
      >
        <div class="head">All views · ${this.views.length}</div>
        <div class="grid">
          ${this.views.map((v) => this.renderTile(v))}
        </div>
      </div>
    `;
  }

  private renderTile(v: ApiView) {
    const label = v.rail?.label ?? humanize(v.name);
    const icon = v.rail?.icon ?? "app-window";
    const pinned = this.pins.includes(v.name);
    return html`
      <a
        class="tile"
        data-pinned=${pinned ? "true" : "false"}
        href=${`/${v.name === "dashboard" ? "" : v.name}`}
        title=${`${label} · ${v.source}`}
        @click=${(e: MouseEvent) => {
          e.preventDefault();
          this.navigate(v.name);
        }}
      >
        <ship-icon name=${icon} size="lg"></ship-icon>
        <span class="label">${label}</span>
        <span
          class="badge"
          role="button"
          aria-label=${pinned ? "Unpin from rail" : "Pin to rail"}
          title=${pinned ? "Unpin from rail" : "Pin to rail"}
          @click=${(e: MouseEvent) => this.togglePin(e, v.name)}
        >
          ${pinned ? "★" : "☆"}
        </span>
      </a>
    `;
  }
}

declare global {
  interface HTMLElementTagNameMap {
    "ship-launcher": ShipLauncher;
  }
}
